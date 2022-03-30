use std::cmp::{max, min};
use std::time::Instant;
use crate::bitboards::{RANK_2_BITS, RANK_7_BITS};
use crate::engine_constants::{DEBUG, DEPTH_REMAINING_FOR_RD_INCREASE, HISTORY_BELOW_ALPHA_MULTIPLIER, HISTORY_BELOW_BETA_MULTIPLIER, HISTORY_GOOD_MULTIPLIER, LMR_LEGALMOVES_BEFORE_ATTEMPT, LMR_MIN_DEPTH, LMR_REDUCTION, MAX_DEPTH, MAX_QUIESCE_DEPTH, NULL_MOVE_REDUCE_DEPTH, NUM_HASH_ENTRIES, PAWN_VALUE, QUEEN_VALUE, ROOK_VALUE};
use crate::evaluate::{evaluate, material_score, pawn_material, piece_material};
use crate::fen::algebraic_move_from_move;
use crate::hash::{en_passant_zobrist_key_index, ZOBRIST_KEY_MOVER_SWITCH, ZOBRIST_KEYS_EN_PASSANT};
use crate::make_move::make_move;
use crate::move_constants::{EN_PASSANT_NOT_AVAILABLE, PIECE_MASK_BISHOP, PIECE_MASK_FULL, PIECE_MASK_KING, PIECE_MASK_KNIGHT, PIECE_MASK_PAWN, PIECE_MASK_QUEEN, PIECE_MASK_ROOK, PROMOTION_FULL_MOVE_MASK};
use crate::move_scores::{score_move, score_quiesce_move};
use crate::moves::{is_check, moves, quiesce_moves, verify_move};
use crate::opponent;
use crate::types::{Bitboard, BLACK, BoundType, HashEntry, HashLock, Move, Mover, MoveScore, MoveScoreList, PathScore, Position, Score, SearchState, WHITE, Window};
use crate::types::BoundType::{Exact, Lower, Upper};
use crate::utils::{captured_piece_value, from_square_part, pawn_push, to_square_part};

pub const MATE_SCORE: Score = 10000;
pub const MATE_MARGIN: Score = 1000;
pub const MATE_START: Score = MATE_SCORE - MATE_MARGIN;

pub const WHITE_PASSED_PAWN_MASK: [Bitboard; 64] =
[
    0,0,0,0,0,0,0,0,
    0x0003030303030000,0x0007070707070000,0x000E0E0E0E0E0000,0x001C1C1C1C1C0000,0x0038383838380000,0x0070707070700000,0x00E0E0E0E0E00000,0x00C0C0C0C0C00000,
    0x0003030303000000,0x0007070707000000,0x000E0E0E0E000000,0x001C1C1C1C000000,0x0038383838000000,0x0070707070000000,0x00E0E0E0E0000000,0x00C0C0C0C0000000,
    0x0003030300000000,0x0007070700000000,0x000E0E0E00000000,0x001C1C1C00000000,0x0038383800000000,0x0070707000000000,0x00E0E0E000000000,0x00C0C0C000000000,
    0x0003030000000000,0x0007070000000000,0x000E0E0000000000,0x001C1C0000000000,0x0038380000000000,0x0070700000000000,0x00E0E00000000000,0x00C0C00000000000,
    0x0003000000000000,0x0007000000000000,0x000E000000000000,0x001C000000000000,0x0038000000000000,0x0070000000000000,0x00E0000000000000,0x00C0000000000000,
    0,0,0,0,0,0,0,0,
    0,0,0,0,0,0,0,0
];

pub const BLACK_PASSED_PAWN_MASK: [Bitboard; 64] =
[
    0,0,0,0,0,0,0,0,
    0,0,0,0,0,0,0,0,
    0x0000000000000300,0x0000000000000700,0x0000000000000E00,0x0000000000001C00,0x0000000000003800,0x0000000000007000,0x000000000000E000,0x000000000000C000,
    0x0000000000030300,0x0000000000070700,0x00000000000E0E00,0x00000000001C1C00,0x0000000000383800,0x0000000000707000,0x0000000000E0E000,0x0000000000C0C000,
    0x0000000003030300,0x0000000007070700,0x000000000E0E0E00,0x000000001C1C1C00,0x0000000038383800,0x0000000070707000,0x00000000E0E0E000,0x00000000C0C0C000,
    0x0000000303030300,0x0000000707070700,0x0000000E0E0E0E00,0x0000001C1C1C1C00,0x0000003838383800,0x0000007070707000,0x000000E0E0E0E000,0x000000C0C0C0C000,
    0x0000030303030300,0x0000070707070700,0x00000E0E0E0E0E00,0x00001C1C1C1C1C00,0x0000383838383800,0x0000707070707000,0x0000E0E0E0E0E000,0x0000C0C0C0C0C000,
    0,0,0,0,0,0,0,0
];

pub const LAST_EXTENSION_LAYER: u8 = 4;

pub const MAX_NEW_EXTENSIONS_TREE_PART: [u8; 5] = [ 1, 0, 0, 0, 0 ];

#[macro_export]
macro_rules! time_remains {
    ($end_time:expr) => {
        $end_time > Instant::now()
    }
}

#[macro_export]
macro_rules! time_expired {
    ($search_state:expr) => {
        if Instant::now() >= $search_state.end_time {
            send_info($search_state);
            true
        } else {
            false
        }
    }
}

#[macro_export]
macro_rules! check_time {
    ($search_state:expr) => {
        if $search_state.nodes % 1000 == 0 {
            if $search_state.end_time < Instant::now() {
                send_info($search_state);
                return (vec![0], 0);
            }
        }
        if $search_state.nodes % 1000000 == 0 {
            send_info($search_state);
        }
    }
}

#[macro_export]
macro_rules! debug_out {
    ($s:expr) => {
        if DEBUG {
            $s
        }
    }
}

pub fn iterative_deepening(position: &Position, max_depth: u8, search_state: &mut SearchState) -> Move {

    search_state.start_time = Instant::now();

    let mut legal_moves: MoveScoreList = moves(position).into_iter().filter(|m| {
        let mut new_position = *position;
        make_move(position, *m, &mut new_position);
        !is_check(&new_position, position.mover)
    }).map(|m| {
        (m, -MATE_SCORE)
    }).collect();

    clear_history_table(search_state);
    clear_killers(search_state);

    if search_state.history.is_empty() {
        search_state.history.push(position.zobrist_lock)
    }

    let mut aspiration_window = (-MATE_SCORE, MATE_SCORE);

    search_state.current_best = (vec![0], -MATE_SCORE);

    let aspiration_radius: Vec<Score> = vec![
        25, 50, 100, 200, 400, 800
    ];

    for iterative_depth in 1..=max_depth {
        let mut c = 0;
        let extension_limit = iterative_depth;
        search_state.iterative_depth = iterative_depth;

        loop {
            let mut aspire_best = start_search(position, &mut legal_moves, search_state, aspiration_window, extension_limit);
            if time_expired!(search_state) {
                return search_state.current_best.0[0]
            }

            if aspire_best.1 > aspiration_window.0 && aspire_best.1 < aspiration_window.1 {
                search_state.current_best = aspire_best;
                break
            } else {
                c += 1;
                if c == aspiration_radius.len() {
                    aspiration_window = (-MATE_SCORE, MATE_SCORE);
                } else {
                    if aspire_best.1 <= aspiration_window.0 {
                        aspiration_window.0 = max(-MATE_SCORE, aspiration_window.0 - aspiration_radius[c]);
                    } else if aspire_best.1 >= aspiration_window.1 {
                        aspiration_window.1 = min(MATE_SCORE, aspiration_window.1 + aspiration_radius[c]);
                    };
                }
            };

        }

        legal_moves.sort_by(|(_, a), (_, b) | b.cmp(a));
        legal_moves = legal_moves.into_iter().map(|m| {
            (m.0, -MATE_SCORE)
        }).collect();

        aspiration_window = (search_state.current_best.1 - aspiration_radius[0], search_state.current_best.1 + aspiration_radius[0])
    }

    send_info(search_state);
    legal_moves[0].0
}

fn clear_killers(search_state: &mut SearchState) {
    for i in 0..MAX_DEPTH as usize {
        search_state.mate_killer[i] = 0;
        for j in 0..2 {
            search_state.killer_moves[i][j] = 0;
        }
    }
}

fn clear_history_table(search_state: &mut SearchState) {
    for i in 0..12 {
        for j in 0..64 {
            for k in 0..64 {
                search_state.history_moves[i][j][k] = 0;
            }
        }
    }
    search_state.highest_history_score = 0;
}

pub fn start_search(position: &Position, legal_moves: &mut MoveScoreList, search_state: &mut SearchState, aspiration_window: Window, extension_limit: u8) -> PathScore {

    let mut current_best: PathScore = (vec![legal_moves[0].0], -MATE_SCORE);

    for mv in legal_moves {
        let mut new_position = *position;
        make_move(position, mv.0, &mut new_position);
        search_state.history.push(new_position.zobrist_lock);

        let mut path_score = search(&new_position, search_state.iterative_depth-1, 1, (-aspiration_window.1, -aspiration_window.0), search_state, extension_limit);
        path_score.1 = -path_score.1;

        search_state.history.pop();
        mv.1 = path_score.1;
        if path_score.1 > current_best.1 && time_remains!(search_state.end_time) {
            current_best.0[0] = mv.0;
            current_best.1 = mv.1;
        }

        if time_expired!(search_state) {
            return current_best;
        }
    }
    current_best

}

#[inline(always)]
pub fn store_hash_entry(index: usize, lock: HashLock, height: u8, existing_height: u8, existing_version: u32, bound: BoundType, movescore: MoveScore, search_state: &mut SearchState) {
    if height >= existing_height || search_state.hash_table_version > existing_version {
        search_state.hash_table_height[index] = HashEntry {
            score: movescore.1,
            version: search_state.hash_table_version,
            height,
            mv: movescore.0,
            bound,
            lock,
        };
    }
}

#[inline(always)]
fn is_end_game(position: &Position) -> bool {
    let piece_material = piece_material(position, WHITE) + piece_material(position, BLACK);
    let pawn_material = pawn_material(position, WHITE) + pawn_material(position, BLACK);
    return piece_material + pawn_material < ROOK_VALUE * 4;
}

#[inline(always)]
fn draw_value(position: &Position) -> Score {
    if is_end_game(position) {
        0
    } else {
        -75
    }
}

#[inline(always)]
pub fn search(position: &Position, depth: u8, ply: u8, window: Window, search_state: &mut SearchState, extension_limit: u8) -> PathScore {

    check_time!(search_state);

    if search_state.history.iter().rev().take(position.half_moves as usize).filter(|p| position.zobrist_lock == **p).count() > 1 {
        return (vec![0], draw_value(position));
    }

    if position.half_moves >= 100 {
        return (vec![0], draw_value(position));
    }

    let scouting = window.1 - window.0 == 1;

    let index: usize = (position.zobrist_lock % NUM_HASH_ENTRIES as u128) as usize;
    let mut alpha = window.0;
    let mut beta = window.1;

    let mut legal_move_count = 0;
    let mut hash_height = 0;
    let mut hash_flag = Upper;
    let mut hash_version = 0;
    let mut best_movescore: MoveScore = (0,-MATE_SCORE);

    let hash_move = match search_state.hash_table_height.get(index) {
        Some(x) => {
            if x.lock == position.zobrist_lock {
                let score = match x.score {
                    s if s > MATE_START => s - ply as Score,
                    s if s < -MATE_START => s + ply as Score,
                    s => s,
                };
                if x.height >= depth {
                    hash_height = x.height;
                    hash_version = x.version;
                    if x.bound == Exact {
                        search_state.hash_hits_exact += 1;
                        return (vec![x.mv], score)
                    }
                    if x.bound == Lower && score > alpha { alpha = score }
                    if x.bound == Upper && score < beta { beta = score }
                    if alpha >= beta { return (vec![x.mv], score) }
                }
                x.mv
            } else {
                search_state.hash_clashes += 1;
                0
            }
        },
        None => {
            0
        }
    };

    if depth == 0 {
        let q = quiesce(position, MAX_QUIESCE_DEPTH, ply, (alpha, beta), search_state);
        let bound = if q.1 <= alpha { Upper } else if q.1 >= beta { Lower } else { Exact };
        if bound == Exact {
           store_hash_entry(index, position.zobrist_lock, 0, hash_height, hash_version, bound, (0, q.1), search_state);
        }
        return q;
    }

    search_state.nodes += 1;

    let null_move_reduce_depth = if depth > DEPTH_REMAINING_FOR_RD_INCREASE { NULL_MOVE_REDUCE_DEPTH + 1 } else { NULL_MOVE_REDUCE_DEPTH };

    let in_check = is_check(position, position.mover);

    if scouting && depth > null_move_reduce_depth && null_move_material(position) && !in_check && evaluate(position) > beta {
        let mut new_position = *position;
        make_null_move(&mut new_position);
        let score = -search(&new_position, depth - 1 - NULL_MOVE_REDUCE_DEPTH, ply + 1, (-beta, (-beta) + 1), search_state, extension_limit).1;
        if score >= beta {
            return (vec![0], beta);
        }
    }

    let mut scout_search = false;

    let these_extentions = min(extension_limit, if in_check { 1 } else { 0 });

    let real_depth = depth + these_extentions;

    let these_moves = if verify_move(position, hash_move) {
        let mut new_position = *position;
        make_move(position, hash_move, &mut new_position);

        if !is_check(&new_position, position.mover) {
            legal_move_count += 1;
            let score = search_wrapper(real_depth, ply, search_state, (-beta, -alpha), &mut new_position, 0, extension_limit - these_extentions);
            check_time!(search_state);
            if score > best_movescore.1 {
                best_movescore = (hash_move, score);
                if best_movescore.1 > alpha {
                    alpha = best_movescore.1;
                    if alpha >= beta {
                        return cutoff(position, index, real_depth, ply, search_state, best_movescore, hash_height, hash_version, hash_move, &mut new_position);
                    }
                    hash_flag = Exact;
                }
                scout_search = true;
            }
            moves(position).into_iter().filter(|m| { *m != hash_move }).collect()
        } else {
            moves(position)
        }
    } else {
        moves(position)
    };

    let enemy = &position.pieces[opponent!(position.mover) as usize];
    let mut move_scores: Vec<(Move, Score)> = these_moves.into_iter().map(|m| {
        (m, score_move(position, m, search_state, ply as usize, enemy))
    }).collect();
    move_scores.sort_by(|(_, a), (_, b) | b.cmp(a));

    for ms in move_scores {
        let m = ms.0;
        let mut new_position = *position;
        make_move(position, m, &mut new_position);
        if !is_check(&new_position, position.mover) {
            legal_move_count += 1;

            let lmr = if these_extentions == 0 && legal_move_count > LMR_LEGALMOVES_BEFORE_ATTEMPT && real_depth > LMR_MIN_DEPTH && !is_check(&new_position, new_position.mover) && captured_piece_value(position, m) == 0 && !pawn_push(position, m) {
                LMR_REDUCTION
            } else {
                0
            };

            let score = lmr_scout_search(lmr, ply, search_state, extension_limit, alpha, beta, scout_search, these_extentions, real_depth, &mut new_position);

            check_time!(search_state);
            if score < alpha {
                update_history(position, search_state, m, -(real_depth as i64 * HISTORY_BELOW_ALPHA_MULTIPLIER));
            }
            if score < beta {
                update_history(position, search_state, m, -(real_depth as i64 * HISTORY_BELOW_BETA_MULTIPLIER));
            }
            if score > best_movescore.1 {
                best_movescore = (m, score);
                if best_movescore.1 > alpha {
                    alpha = best_movescore.1;
                    if alpha >= beta {
                        return cutoff(position, index, real_depth, ply, search_state, best_movescore, hash_height, hash_version, m, &mut new_position);
                    }
                    hash_flag = Exact;
                }
                scout_search = true;
            }
        }
    }

    if legal_move_count == 0 {
        if is_check(position, position.mover) {
            best_movescore.1 = -MATE_SCORE + ply as Score
        } else {
            best_movescore.1 = draw_value(position)
        }
    };

    store_hash_entry(index, position.zobrist_lock, real_depth, hash_height, hash_version, hash_flag, best_movescore, search_state);

    (vec![best_movescore.0], best_movescore.1)
}

#[inline(always)]
fn lmr_scout_search(mut lmr: u8, ply: u8, search_state: &mut SearchState, extension_limit: u8, alpha: Score, beta: Score, scout_search: bool, these_extentions: u8, real_depth: u8, new_position: &mut Position) -> Score {
    loop {
        let score = if scout_search {
            let scout_score = search_wrapper(real_depth, ply, search_state, (-alpha-1, -alpha), new_position, lmr, extension_limit - these_extentions);
            if scout_score > alpha {
                search_wrapper(real_depth, ply, search_state, (-beta, -alpha), new_position, 0, extension_limit - these_extentions)
            } else {
                alpha
            }
        } else {
            search_wrapper(real_depth, ply, search_state, (-beta, -alpha), new_position, 0, extension_limit - these_extentions)
        };
        if lmr == 0 || score < beta {
            break score
        } else {
            // search to normal depth because score was >= beta
            lmr = 0
        }
    }
}

#[inline(always)]
fn make_null_move(new_position: &mut Position) {
    if new_position.en_passant_square != EN_PASSANT_NOT_AVAILABLE {
        new_position.zobrist_lock ^= ZOBRIST_KEYS_EN_PASSANT[en_passant_zobrist_key_index(new_position.en_passant_square)];
        new_position.en_passant_square = EN_PASSANT_NOT_AVAILABLE;
    }
    new_position.mover ^= 1;
    new_position.zobrist_lock ^= ZOBRIST_KEY_MOVER_SWITCH;
}

#[inline(always)]
fn search_wrapper(depth: u8, ply: u8, search_state: &mut SearchState, window: Window, new_position: &mut Position, lmr: u8, extension_limit: u8) -> Score {
    search_state.history.push(new_position.zobrist_lock);
    let s = -search(new_position, depth - 1 - lmr, ply + 1, window, search_state, extension_limit).1;
    let score = s;
    search_state.history.pop();
    score
}

#[inline(always)]
fn cutoff(position: &Position, hash_index: usize, depth: u8, ply: u8, search_state: &mut SearchState, best_movescore: MoveScore, hash_height: u8, hash_version: u32, m: Move, new_position: &mut Position) -> PathScore {
    store_hash_entry(hash_index, position.zobrist_lock, depth, hash_height, hash_version, Lower, best_movescore, search_state);
    update_history(position, search_state, m, depth as i64 * depth as i64);
    update_killers(position, ply, search_state, m, new_position, best_movescore);
    (vec![best_movescore.0], best_movescore.1)
}

#[inline(always)]
fn update_history(position: &Position, search_state: &mut SearchState, m: Move, score: i64) {
    if captured_piece_value(position, m) == 0 {
        let f = from_square_part(m) as usize;
        let t = to_square_part(m) as usize;

        let piece_index = piece_index_12(position, m);

        search_state.history_moves[piece_index][f][t] += score * HISTORY_GOOD_MULTIPLIER;

        if search_state.history_moves[piece_index][f][t] < 0 {
             search_state.history_moves[piece_index][f][t] = 0;
        }

        halve_history_scores_if_required(search_state, f, t, piece_index)

    }
}

#[inline(always)]
fn halve_history_scores_if_required(search_state: &mut SearchState, f: usize, t: usize, piece_index: usize) {
    if search_state.history_moves[piece_index][f][t] > search_state.highest_history_score {
        search_state.highest_history_score = search_state.history_moves[piece_index][f][t];
        if search_state.highest_history_score > (i64::MAX / 2) {
            for i in 0..12 {
                for j in 0..64 {
                    for k in 0..64 {
                        search_state.history_moves[i][j][k] /= 2;
                    }
                }
            }
        }
    }
}

#[inline(always)]
pub fn piece_index_12(position: &Position, m: Move) -> usize {
    ((position.mover * 6) + match m & PIECE_MASK_FULL {
        PIECE_MASK_PAWN => 0,
        PIECE_MASK_KNIGHT => 1,
        PIECE_MASK_BISHOP => 2,
        PIECE_MASK_ROOK => 3,
        PIECE_MASK_QUEEN => 4,
        PIECE_MASK_KING => 5,
        _ => {
            panic!("Expected a valid piece mask.");
        }
    }) as usize
}

#[inline(always)]
fn update_killers(position: &Position, ply: u8, search_state: &mut SearchState, m: Move, new_position: &mut Position, move_score: MoveScore) {
    if move_score.1 > MATE_START {
        search_state.mate_killer[ply as usize] = m;
    }
    if search_state.killer_moves[ply as usize][0] != m {
        let opponent_index = opponent!(position.mover) as usize;
        let was_capture = position.pieces[opponent_index].all_pieces_bitboard != new_position.pieces[opponent_index].all_pieces_bitboard;
        if (m & PROMOTION_FULL_MOVE_MASK == 0) && !was_capture {
            search_state.killer_moves[ply as usize][1] = search_state.killer_moves[ply as usize][0];
            search_state.killer_moves[ply as usize][0] = m;
        }
    }
}

#[inline(always)]
fn null_move_material(position: &Position) -> bool {
    side_total_non_pawn_values(position, position.mover) >= 2
}

#[inline(always)]
fn side_total_non_pawn_values(position: &Position, side: Mover) -> u32 {
    (position.pieces[side as usize].bishop_bitboard |
    position.pieces[side as usize].knight_bitboard |
    position.pieces[side as usize].rook_bitboard |
    position.pieces[side as usize].queen_bitboard).count_ones()
}

#[inline(always)]
pub fn quiesce(position: &Position, depth: u8, ply: u8, window: Window, search_state: &mut SearchState) -> PathScore {

    check_time!(search_state);

    search_state.nodes += 1;

    let eval = evaluate(position);

    if depth == 0 {
        return (vec![0], eval);
    }

    let beta = window.1;

    if eval >= beta {
        return (vec![0], eval);
    }

    let promote_from_rank = if position.mover == WHITE { RANK_7_BITS } else { RANK_2_BITS };
    let mut delta = QUEEN_VALUE;
    if position.pieces[position.mover as usize].pawn_bitboard & promote_from_rank != 0 {
        delta += QUEEN_VALUE - PAWN_VALUE
    }

    let mut alpha = window.0;

    if eval < alpha - delta {
        return (vec![0], alpha);
    }

    if alpha < eval {
        alpha = eval;
    }

    let in_check = is_check(position, position.mover);
    let enemy = &position.pieces[opponent!(position.mover) as usize];

    let mut move_scores: MoveScoreList =
        if in_check {
            moves(position).into_iter().map(|m| {
                (m, score_move(position, m, search_state, ply as usize, enemy))
            }).collect()
        } else {
            quiesce_moves(position).into_iter().map(|m| {
                (m, score_quiesce_move(position, m, enemy))
            }).collect()
        };

    move_scores.sort_by(|(_, a), (_, b) | b.cmp(a));

    let mut legal_move_count = 0;

    for ms in move_scores {
        let m = ms.0;
        let mut new_position = *position;

        if in_check || (eval + captured_piece_value(position, m) + 125 > alpha) {
            make_move(position, m, &mut new_position);
            if !is_check(&new_position, position.mover) {
                legal_move_count += 1;
                let score = -quiesce(&new_position, depth - 1, ply + 1, (-beta, -alpha), search_state).1;
                check_time!(search_state);
                if score >= beta {
                    return (vec![m], beta);
                }
                if score > alpha {
                    alpha = score;
                }
            }
        }
    }

    // if legal_move_count == 0 && in_check {
    //     (vec![0], -MATE_SCORE + ply as Score)
    // } else {
        (vec![0], alpha)
//    }
}

fn send_info(search_state: &mut SearchState) {
    if !search_state.show_info {
        return
    }
    if search_state.start_time.elapsed().as_millis() > 0 {
        let nps = (search_state.nodes as f64 / search_state.start_time.elapsed().as_millis() as f64) * 1000.0;
        let s = "info score cp ".to_string() + &*(search_state.current_best.1 as i64).to_string() +
            &*" depth ".to_string() + &*search_state.iterative_depth.to_string() +
            &*" time ".to_string() + &*search_state.start_time.elapsed().as_millis().to_string() +
            &*" nodes ".to_string() + &*search_state.nodes.to_string() +
            &*" pv ".to_string() + &*algebraic_move_from_move(search_state.current_best.0[0]) +
            &*" nps ".to_string() + &*(nps as u64).to_string();

        println!("{}", s);
    }
}