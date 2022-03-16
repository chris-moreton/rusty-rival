use std::cmp::min;
use std::time::{Instant};
use crate::bitboards::{RANK_2_BITS, RANK_7_BITS};
use crate::engine_constants::{DEPTH_REMAINING_FOR_NULLMOVE_RD_INCREASE, LMR_LEGALMOVES_BEFORE_ATTEMPT, LMR_MIN_DEPTH, MAX_QUIESCE_DEPTH, NULL_MOVE_REDUCE_DEPTH, NUM_HASH_ENTRIES, PAWN_VALUE, QUEEN_VALUE};
use crate::evaluate::{evaluate};
use crate::fen::{algebraic_move_from_move};
use crate::hash::{ZOBRIST_KEY_MOVER_SWITCH};
use crate::make_move::make_move;
use crate::move_constants::{PIECE_MASK_FULL, PIECE_MASK_PAWN, PIECE_MASK_QUEEN, PIECE_MASK_KING, PIECE_MASK_BISHOP, PIECE_MASK_ROOK, PIECE_MASK_KNIGHT, PROMOTION_FULL_MOVE_MASK};
use crate::move_scores::{score_move, score_quiesce_move};
use crate::moves::{is_check, moves, quiesce_moves, verify_move};
use crate::opponent;
use crate::types::{Move, Position, Score, SearchState, Window, MoveScoreList, MoveScore, HashLock, HashEntry, BoundType, WHITE, Mover, Bitboard, BLACK, PathScore};
use crate::types::BoundType::{Exact, Lower, Upper};
use crate::utils::{captured_piece_value, from_square_part, to_square_part};

pub const MAX_SCORE: Score = 30000;
pub const MATE_MARGIN: Score = 1000;
pub const MATE_START: Score = MAX_SCORE - MATE_MARGIN;

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
        if $search_state.nodes % 100000 == 0 {
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

pub const ASPIRATION_RADIUS: Score = 50;

pub fn iterative_deepening(position: &Position, max_depth: u8, search_state: &mut SearchState) -> Move {

    search_state.start_time = Instant::now();

    let mut legal_moves: MoveScoreList = moves(position).into_iter().filter(|m| {
        let mut new_position = *position;
        make_move(position, *m, &mut new_position);
        !is_check(&new_position, position.mover)
    }).map(|m| {
        (m, -MAX_SCORE)
    }).collect();

    for i in 0..12 {
        for j in 0..64 {
            for k in 0..64 {
                search_state.history_moves[i][j][k] = 0;
            }
        }
    }
    search_state.highest_history_score = 0;
    search_state.lowest_history_score = 0;

    if search_state.history.is_empty() {
        search_state.history.push(position.zobrist_lock)
    }

    let mut aspiration_window = (-MAX_SCORE, MAX_SCORE);

    search_state.current_best = (vec![0], -MAX_SCORE);

    for iterative_depth in 1..=max_depth {
        let extension_limit = iterative_depth;
        search_state.iterative_depth = iterative_depth;
        let mut aspire_best = start_search(position, &mut legal_moves, search_state, aspiration_window, extension_limit);

        if aspire_best.1 > aspiration_window.0 && aspire_best.1 < aspiration_window.1 {
            search_state.current_best = aspire_best
        } else {
            if time_remains!(search_state.end_time) {
                if aspire_best.1 <= aspiration_window.0 {
                    aspire_best = start_search(position, &mut legal_moves, search_state, (-MAX_SCORE, aspiration_window.1), extension_limit);
                } else if aspire_best.1 >= aspiration_window.1 {
                    aspire_best = start_search(position, &mut legal_moves, search_state, (aspiration_window.0, MAX_SCORE), extension_limit);
                };
            }
            if time_remains!(search_state.end_time) {
                search_state.current_best = aspire_best
            }
        };

        if time_expired!(search_state) {
            if search_state.current_best.0[0] == 0 {
                panic!("Didn't have time to do anything.")
            }
            return search_state.current_best.0[0]
        }

        legal_moves.sort_by(|(_, a), (_, b) | b.cmp(a));
        legal_moves = legal_moves.into_iter().map(|m| {
            (m.0, -MAX_SCORE)
        }).collect();

        aspiration_window = (search_state.current_best.1 - ASPIRATION_RADIUS, search_state.current_best.1 + ASPIRATION_RADIUS)
    }

    send_info(search_state);
    legal_moves[0].0
}

pub fn start_search(position: &Position, legal_moves: &mut MoveScoreList, search_state: &mut SearchState, aspiration_window: Window, extension_limit: u8) -> PathScore {

    let mut current_best: PathScore = (vec![legal_moves[0].0], -MAX_SCORE);

    for mv in legal_moves {
        let mut new_position = *position;
        make_move(position, mv.0, &mut new_position);
        search_state.history.push(new_position.zobrist_lock);

        let mut path_score = search(&new_position, search_state.iterative_depth-1, 1, (-aspiration_window.1, -aspiration_window.0), search_state, extension_limit);
        path_score.1 = -path_score.1;
        if path_score.1 > MATE_START { path_score.1 -= 1 } else if path_score.1 < -MATE_START { path_score.1 += 1 };

        search_state.history.pop();
        mv.1 = path_score.1;
        if path_score.1 > current_best.1 && time_remains!(search_state.end_time){
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
pub fn pawn_push(position: &Position, m: Move) -> bool {
    let move_piece = m & PIECE_MASK_FULL;
    if move_piece == PIECE_MASK_PAWN {
        let to_square = to_square_part(m);
        if to_square >= 48 || to_square <= 15 {
            return true;
        }
        if position.mover == WHITE {
            if (40..=47).contains(&to_square) {
                return position.pieces[BLACK as usize].pawn_bitboard & WHITE_PASSED_PAWN_MASK[to_square as usize] == 0;
            }
        } else if (16..=23).contains(&to_square) {
            return position.pieces[WHITE as usize].pawn_bitboard & BLACK_PASSED_PAWN_MASK[to_square as usize] == 0;
        }
    }
    false
}

#[inline(always)]
pub fn store_hash_entry(index: usize, lock: HashLock, height: u8, existing_height: u8, existing_version: u32, bound: BoundType, movescore: MoveScore, search_state: &mut SearchState) {
    if height >= existing_height || search_state.hash_table_version > existing_version {
        search_state.hash_table_height[index] = HashEntry { score: movescore.1, version: search_state.hash_table_version, height, mv: movescore.0, bound, lock, };
    } else if bound == Lower {
        search_state.hash_table_replace[index] = HashEntry { score: movescore.1, version: search_state.hash_table_version, height, mv: movescore.0, bound, lock, };
    }
}

#[inline(always)]
pub fn search(position: &Position, depth: u8, ply: u8, window: Window, search_state: &mut SearchState, extension_limit: u8) -> PathScore {

    check_time!(search_state);

    if search_state.history.iter().rev().take(position.half_moves as usize).filter(|p| position.zobrist_lock == **p).count() > 1 {
        return (vec![0], 0);
    }

    let index: usize = (position.zobrist_lock % NUM_HASH_ENTRIES as u128) as usize;
    let mut alpha = window.0;
    let mut beta = window.1;

    let mut legal_move_count = 0;
    let mut hash_height = 0;
    let mut hash_flag = Upper;
    let mut hash_version = 0;
    let mut best_movescore: MoveScore = (0,-MAX_SCORE);

    let mut hash_move = match search_state.hash_table_height.get(index) {
        Some(x) => {
            if x.lock == position.zobrist_lock {
                if x.height >= depth {
                    hash_height = x.height;
                    hash_version = x.version;
                    if x.bound == BoundType::Exact {
                        search_state.hash_hits_exact += 1;
                        return (vec![x.mv], x.score)
                    }
                    if x.bound == BoundType::Lower { alpha = x.score }
                    if x.bound == BoundType::Upper { beta = x.score }
                    if alpha >= beta { return (vec![x.mv], x.score) }
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

    if hash_move == 0 {
        hash_move = match search_state.hash_table_replace.get(index) {
            Some(x) => {
                if x.lock == position.zobrist_lock {
                    if x.height >= depth {
                        hash_height = x.height;
                        hash_version = x.version;
                        if x.bound == BoundType::Exact {
                            search_state.hash_hits_exact += 1;
                            return (vec![x.mv], x.score)
                        }
                        if x.bound == BoundType::Lower { alpha = x.score }
                        if x.bound == BoundType::Upper { beta = x.score }
                        if alpha >= beta { return (vec![x.mv], x.score) }
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
        }
    }

    if depth == 0 {
        let q = quiesce(position, MAX_QUIESCE_DEPTH, ply, (alpha, beta), search_state);
        let bound = if q.1 <= alpha { Upper } else if q.1 >= beta { Lower } else { Exact };
        if bound == Exact {
           store_hash_entry(index, position.zobrist_lock, 0, hash_height, hash_version, bound, (0, q.1), search_state);
        }
        return q;
    }

    search_state.nodes += 1;

    let null_move_reduce_depth = if depth > DEPTH_REMAINING_FOR_NULLMOVE_RD_INCREASE { NULL_MOVE_REDUCE_DEPTH + 1 } else { NULL_MOVE_REDUCE_DEPTH };

    let in_check = is_check(position, position.mover);

    if !search_state.is_on_null_move && depth > null_move_reduce_depth && null_move_material(position) && !in_check && evaluate(position) > beta {
        let mut new_position = *position;
        switch_mover(&mut new_position);
        let score = adjust_mate_score_for_ply(1, -search(&new_position, depth - 1 - NULL_MOVE_REDUCE_DEPTH, ply + 1, (-beta, (-beta) + 1), search_state, extension_limit).1);
        if score >= beta {
            return (vec![0], beta);
        }
        switch_mover(&mut new_position);
    }

    if hash_move == 0 && depth > 5 {
        search_state.history.push(position.zobrist_lock);
        hash_move = search(position, depth - 2, ply, window, search_state, extension_limit).0[0];
        search_state.history.pop();
    }

    let mut scout_search = false;

    let these_extentions = min(extension_limit, if in_check { 1 } else { 0 });
    let real_depth = depth + these_extentions;

    if verify_move(position, hash_move) {
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
        }
    }

    let these_moves = moves(position);
    let enemy = &position.pieces[opponent!(position.mover) as usize];
    let mut move_scores: Vec<(Move, Score)> = these_moves.into_iter().map(|m| {
        (m, score_move(position, m, search_state, ply as usize, enemy))
    }).collect();
    move_scores.sort_by(|(_, a), (_, b) | b.cmp(a));
    let move_list: Vec<Move> = move_scores.into_iter().map(|(m,_)| { m }).filter(|m| { *m != hash_move }).collect();

    for m in move_list {
        let mut new_position = *position;
        make_move(position, m, &mut new_position);
        if !is_check(&new_position, position.mover) {
            legal_move_count += 1;
            let lmr = if these_extentions == 0 && legal_move_count > LMR_LEGALMOVES_BEFORE_ATTEMPT && depth > LMR_MIN_DEPTH && !is_check(&new_position, new_position.mover) && captured_piece_value(position, m) == 0 {
                1
            } else {
                0
            };

            let score = if scout_search {
                let scout_score = search_wrapper(real_depth, ply, search_state, (-alpha-1, -alpha), &mut new_position, lmr, extension_limit - these_extentions);
                if scout_score > alpha {
                    search_wrapper(real_depth, ply, search_state, (-beta, -alpha), &mut new_position, 0, extension_limit - these_extentions)
                } else {
                    alpha
                }
            } else {
                search_wrapper(real_depth, ply, search_state, (-beta, -alpha), &mut new_position, 0, extension_limit - these_extentions)
            };

            check_time!(search_state);
            if score < beta {
                update_history(position, search_state, m, -(real_depth as i64));
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

    if legal_move_count == 0 && !is_check(position, position.mover) {
        best_movescore.1 = 0
    };

    store_hash_entry(index, position.zobrist_lock, depth, hash_height, hash_version, hash_flag, best_movescore, search_state);

    (vec![best_movescore.0], adjust_mate_score_for_ply(0, best_movescore.1))
}

#[inline(always)]
fn switch_mover(new_position: &mut Position) {
    new_position.mover ^= 1;
    new_position.zobrist_lock ^= ZOBRIST_KEY_MOVER_SWITCH;
}

#[inline(always)]
fn search_wrapper(depth: u8, ply: u8, search_state: &mut SearchState, window: Window, new_position: &mut Position, lmr: u8, extension_limit: u8) -> Score {
    search_state.history.push(new_position.zobrist_lock);
    let score = adjust_mate_score_for_ply(1, -search(new_position, depth - 1 - lmr, ply + 1, window, search_state, extension_limit).1);
    search_state.history.pop();
    score
}

#[inline(always)]
fn cutoff(position: &Position, hash_index: usize, depth: u8, ply: u8, search_state: &mut SearchState, best_movescore: MoveScore, hash_height: u8, hash_version: u32, m: Move, new_position: &mut Position) -> PathScore {
    store_hash_entry(hash_index, position.zobrist_lock, depth, hash_height, hash_version, Lower, best_movescore, search_state);
    update_history(position, search_state, m, depth as i64);
    update_killers(position, ply, search_state, m, new_position);
    (vec![best_movescore.0], best_movescore.1)
}

#[inline(always)]
fn update_history(position: &Position, search_state: &mut SearchState, m: Move, score: i64) {
    if captured_piece_value(position, m) == 0 {
        let f = from_square_part(m) as usize;
        let t = to_square_part(m) as usize;

        let piece_index = piece_index_12(position, m);

        search_state.history_moves[piece_index][f][t] += score;

        if search_state.history_moves[piece_index][f][t] < 0 {
             search_state.history_moves[piece_index][f][t] = 0;
        }

        if search_state.history_moves[piece_index][f][t] > search_state.highest_history_score {
            search_state.highest_history_score = search_state.history_moves[piece_index][f][t];
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
fn adjust_mate_score_for_ply(ply: u8, score: Score) -> Score {
    if score > MATE_START { score - ply as Score } else if score < -MATE_START { score + ply as Score } else { score }
}

#[inline(always)]
fn update_killers(position: &Position, ply: u8, search_state: &mut SearchState, m: Move, new_position: &mut Position) {
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
        return (vec![0], beta);
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

        if eval + captured_piece_value(position, m) + 125 > alpha {
            make_move(position, m, &mut new_position);
            if !is_check(&new_position, position.mover) {
                legal_move_count += 1;
                let score = adjust_mate_score_for_ply(ply, -quiesce(&new_position, depth-1, ply+1, (-beta, -alpha), search_state).1);
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

    if legal_move_count == 0 && in_check {
        (vec![0], -MAX_SCORE + ply as Score)
    } else {
        (vec![0], alpha)
    }
}

fn send_info(search_state: &mut SearchState) {
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