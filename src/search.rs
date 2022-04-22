use crate::engine_constants::{
    ALPHA_PRUNE_MARGINS, BETA_PRUNE_MARGIN_PER_DEPTH, BETA_PRUNE_MAX_DEPTH, IID_MIN_DEPTH, IID_REDUCE_DEPTH,
    LMR_LEGAL_MOVES_BEFORE_ATTEMPT, LMR_MIN_DEPTH, LMR_REDUCTION, MAX_DEPTH, MAX_QUIESCE_DEPTH, NULL_MOVE_MIN_DEPTH, NUM_HASH_ENTRIES,
    ROOK_VALUE_AVERAGE,
};
use crate::evaluate::{evaluate, insufficient_material, pawn_material, piece_material};

use crate::hash::{en_passant_zobrist_key_index, ZOBRIST_KEYS_EN_PASSANT, ZOBRIST_KEY_MOVER_SWITCH};
use crate::make_move::make_move;
use crate::move_constants::{
    EN_PASSANT_NOT_AVAILABLE, PIECE_MASK_BISHOP, PIECE_MASK_FULL, PIECE_MASK_KING, PIECE_MASK_KNIGHT, PIECE_MASK_PAWN, PIECE_MASK_QUEEN,
    PIECE_MASK_ROOK, PROMOTION_FULL_MOVE_MASK,
};
use crate::move_scores::score_move;
use crate::moves::{is_check, moves, verify_move};
use crate::opponent;
use crate::quiesce::quiesce;
use crate::types::BoundType::{Exact, Lower, Upper};
use crate::types::{
    BoundType, HashEntry, Move, MoveScore, MoveScoreList, Mover, PathScore, Position, Score, SearchState, Window, BLACK, WHITE,
};
use crate::utils::{captured_piece_value, from_square_part, penultimate_pawn_push, send_info, to_square_part};
use std::borrow::Borrow;
use std::cmp::{max, min};
use std::time::Instant;
use crate::fen::{algebraic_move_from_move, get_fen};

pub const MAX_WINDOW: Score = 20000;
pub const MATE_SCORE: Score = 10000;
pub const MATE_MARGIN: Score = 1000;
pub const MATE_START: Score = MATE_SCORE - MATE_MARGIN;

pub const LAST_EXTENSION_LAYER: u8 = 4;

pub const MAX_NEW_EXTENSIONS_TREE_PART: [u8; 5] = [1, 0, 0, 0, 0];

#[macro_export]
macro_rules! time_remains {
    ($end_time:expr) => {
        $end_time > Instant::now()
    };
}

#[macro_export]
macro_rules! time_expired {
    ($search_state:expr) => {
        if Instant::now() >= $search_state.end_time {
            send_info($search_state, false);
            true
        } else {
            false
        }
    };
}

#[macro_export]
macro_rules! check_time {
    ($search_state:expr) => {
        if $search_state.nodes % 1000 == 0 {
            if $search_state.end_time < Instant::now() {
                send_info($search_state, false);
                return (vec![0], 0);
            }
        }
        if $search_state.nodes % 1000000 == 0 {
            send_info($search_state, false);
        }
    };
}

#[macro_export]
macro_rules! debug_out {
    ($s:expr) => {
        if DEBUG {
            $s
        }
    };
}

pub fn iterative_deepening(position: &Position, max_depth: u8, search_state: &mut SearchState) -> Move {
    search_state.start_time = Instant::now();
    search_state.hash_table_version += 1;

    let mut legal_moves: MoveScoreList = moves(position)
        .into_iter()
        .filter(|m| {
            let mut new_position = *position;
            make_move(position, *m, &mut new_position);
            !is_check(&new_position, position.mover) && *m != search_state.ignore_root_move
        })
        .map(|m| (m, -MATE_SCORE))
        .collect();

    clear_history_table(search_state);
    clear_killers(search_state);

    if search_state.history.is_empty() {
        search_state.history.push(position.zobrist_lock)
    }

    let mut aspiration_window = (-MAX_WINDOW, MAX_WINDOW);

    search_state.current_best = (vec![0], -MATE_SCORE);

    let aspiration_radius: Vec<Score> = vec![25, 50, 100, 200, 400, 800];

    for iterative_depth in 1..=max_depth {
        let mut c = 0;
        search_state.iterative_depth = iterative_depth;

        loop {
            let aspire_best = start_search(position, &mut legal_moves, search_state, aspiration_window);
            if time_expired!(search_state) {
                return search_state.current_best.0[0];
            }

            if aspire_best.1 > aspiration_window.0 && aspire_best.1 < aspiration_window.1 {
                search_state.current_best = aspire_best;
                break;
            } else {
                c += 1;
                if c == aspiration_radius.len() {
                    aspiration_window = (-MAX_WINDOW, MAX_WINDOW);
                } else if aspire_best.1 <= aspiration_window.0 {
                    aspiration_window.0 = max(-MAX_WINDOW, aspiration_window.0 - aspiration_radius[c]);
                } else if aspire_best.1 >= aspiration_window.1 {
                    aspiration_window.1 = min(MAX_WINDOW, aspiration_window.1 + aspiration_radius[c]);
                }
            };
        }

        legal_moves.sort_by(|(_, a), (_, b)| b.cmp(a));

        legal_moves = legal_moves.into_iter().map(|m| (m.0, -MATE_SCORE)).collect();
        search_state.root_moves = legal_moves.clone();

        aspiration_window = (
            search_state.current_best.1 - aspiration_radius[0],
            search_state.current_best.1 + aspiration_radius[0],
        );

        send_info(search_state, true);
    }

    send_info(search_state, true);
    legal_moves[0].0
}

pub fn start_search(
    position: &Position,
    legal_moves: &mut MoveScoreList,
    search_state: &mut SearchState,
    aspiration_window: Window,
) -> PathScore {
    let mut current_best: PathScore = (vec![legal_moves[0].0], -MATE_SCORE);

    for mv in legal_moves {
        let mut new_position = *position;
        make_move(position, mv.0, &mut new_position);
        search_state.history.push(new_position.zobrist_lock);

        let mut path_score = search(
            &new_position,
            search_state.iterative_depth - 1,
            1,
            (-aspiration_window.1, -aspiration_window.0),
            search_state,
            false,
        );
        path_score.1 = -path_score.1;

        search_state.history.pop();
        mv.1 = path_score.1;

        // We can change this so that we do a research but block the making of the best move
        // We will do the search to the same depth as was achieved here, so the hash table should make it quick

        if path_score.1 > current_best.1 && time_remains!(search_state.end_time) {
            let mut p = vec![mv.0];
            p.extend(path_score.0.clone());
            current_best = (p, mv.1);
        }

        let mut p = vec![mv.0];
        p.extend(path_score.0.clone());
        search_state.pv.insert(mv.0, (p, mv.1));

        if time_expired!(search_state) {
            return current_best;
        }
    }
    current_best
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

#[inline(always)]
#[allow(clippy::too_many_arguments)]
pub fn store_hash_entry(
    position: &Position,
    height: u8,
    existing_height: u8,
    existing_version: u32,
    bound: BoundType,
    movescore: MoveScore,
    search_state: &mut SearchState,
    ply: u8,
) {
    if height >= existing_height || search_state.hash_table_version > existing_version {
        let index: usize = (position.zobrist_lock % NUM_HASH_ENTRIES as u128) as usize;
        search_state.hash_table_height[index] = HashEntry {
            // adjust any mate score so that the score appears calculated as if this ply were the root
            score: match movescore.1 {
                x if x > MATE_START => movescore.1 + ply as Score,
                x if x < MATE_START => movescore.1 - ply as Score,
                _ => movescore.1,
            },
            version: search_state.hash_table_version,
            height,
            mv: movescore.0,
            bound,
            lock: position.zobrist_lock,
        };
    }
}

#[inline(always)]
fn total_material_value(position: &Position) -> Score {
    let piece_material = piece_material(position, WHITE) + piece_material(position, BLACK);
    let pawn_material = pawn_material(position, WHITE) + pawn_material(position, BLACK);
    piece_material + pawn_material
}

#[inline(always)]
fn is_end_game(position: &Position) -> bool {
    total_material_value(position) < ROOK_VALUE_AVERAGE * 4
}

#[inline(always)]
fn draw_value(position: &Position, search_state: &SearchState) -> Score {
    if is_end_game(position) {
        0
    } else {
        search_state.contempt
    }
}

#[inline(always)]
pub fn extend(predicate: bool, these_extentions: u8, ply: u8, search_state: &SearchState) -> u8 {
    if these_extentions == 0 && predicate && ply < search_state.iterative_depth * 2 {
        1
    } else {
        0
    }
}

#[inline(always)]
pub fn null_move_reduce_depth(depth: u8) -> u8 {
    match depth {
        d if d >= 12 => 4,
        d if d >= 5 => 3,
        d if d >= NULL_MOVE_MIN_DEPTH => NULL_MOVE_MIN_DEPTH - 2,
        _ => panic!("Shouldn't be here"),
    }
}

#[inline(always)]
pub fn search(position: &Position, depth: u8, ply: u8, window: Window, search_state: &mut SearchState, on_null_move: bool) -> PathScore {
    check_time!(search_state);

    search_state.nodes += 1;

    if is_draw(position, search_state) {
        return (vec![0], draw_value(position, search_state));
    }

    let scouting = window.1 - window.0 == 1;

    let mut alpha = window.0;
    let mut beta = window.1;

    let mut legal_move_count = 0;
    let mut hash_height = 0;
    let mut hash_flag = Upper;
    let mut hash_version = 0;
    let mut best_pathscore: PathScore = (vec![0], -MATE_SCORE);
    let mut hash_score_was_a_lower_bound = false;

    let index: usize = (position.zobrist_lock % NUM_HASH_ENTRIES as u128) as usize;
    let mut hash_move = match search_state.hash_table_height.get(index) {
        Some(x) => {
            if x.lock == position.zobrist_lock {

                // if x.mv != 0 && !verify_move(position, x.mv) {
                //     panic!("{} {}", get_fen(&position), algebraic_move_from_move(x.mv));
                // }

                // Adjust any mate score so that the score appears calculated from the current root rather than the root when the position was stored
                // When we found the mate, we set the score to reflect the distance from the root, and then, when we stored the score in the TT, we
                // adjusted it again such that it represented the distance from the root at which it was stored - e.g. we found it at ply 7, and wound
                // up needing to store that score when it went back up to ply 5, so we adjusted it so it looked like it was found at ply 2.
                // Now that we are retrieving it, we need to adjust it for the current ply. Following the previous example, if the current ply is 10, then
                // we adjust the score to make it look like it was found at ply 12.
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
                        return (vec![x.mv], score);
                    }
                    if x.bound == Lower && score > alpha {
                        alpha = score;
                        hash_score_was_a_lower_bound = true
                    }
                    if x.bound == Upper && score < beta {
                        beta = score
                    }
                    if alpha >= beta {
                        return (vec![x.mv], score);
                    }
                }
                x.mv
            } else {
                search_state.hash_clashes += 1;
                0
            }
        }
        None => 0,
    };

    let in_check = is_check(position, position.mover);

    let mut lazy_eval: Score = -Score::MAX;

    if scouting && depth <= BETA_PRUNE_MAX_DEPTH && !in_check && beta.abs() < MATE_START {
        lazy_eval = evaluate(position);
        let margin = BETA_PRUNE_MARGIN_PER_DEPTH * depth as Score;
        if lazy_eval - margin as Score >= beta {
            return (vec![0], lazy_eval - margin);
        }
    }

    if depth == 0 {
        // Need to do this, else we'll get +2 for this node, as quiesce does a +1 on entry
        search_state.nodes -= 1;
        let q = quiesce(position, MAX_QUIESCE_DEPTH, ply, (alpha, beta), search_state);
        let bound = if q.1 <= alpha {
            Upper
        } else if q.1 >= beta {
            Lower
        } else {
            Exact
        };
        if bound == Exact {
            store_hash_entry(position, 0, hash_height, hash_version, bound, (0, q.1), search_state, ply);
        }
        return q;
    }

    let alpha_prune_flag = if depth <= ALPHA_PRUNE_MARGINS.len() as u8 && scouting && !in_check && alpha.abs() < MATE_START {
        if lazy_eval == -Score::MAX {
            lazy_eval = evaluate(position);
        }

        lazy_eval + ALPHA_PRUNE_MARGINS[depth as usize - 1] < alpha
    } else {
        false
    };

    let mut these_extensions = 0;

    if !on_null_move && scouting && depth >= NULL_MOVE_MIN_DEPTH && null_move_material(position) && !in_check {
        let mut new_position = *position;
        make_null_move(&mut new_position);

        let score = -search(
            &new_position,
            depth - 1 - null_move_reduce_depth(depth),
            ply + 1,
            (-beta, (-beta) + 1),
            search_state,
            true,
        ).1;

        if score >= beta {
            return (vec![0], beta);
        }
    }

    let mut scout_search = false;

    these_extensions = extend(in_check, these_extensions, ply, search_state);

    let mut real_depth = depth + these_extensions;

    let verified_hash_move = if !scouting && hash_move == 0 && depth + these_extensions > IID_MIN_DEPTH {
        hash_move = search_wrapper(depth - IID_REDUCE_DEPTH, ply, search_state, (-alpha-1, -alpha), position, 0).0[0];
        hash_move != 0
    } else {
        hash_move != 0 && verify_move(position, hash_move)
    };

    let these_moves = if verified_hash_move {

        let mut new_position = *position;
        make_move(position, hash_move, &mut new_position);

        if !is_check(&new_position, position.mover) {
            legal_move_count += 1;
            let path_score = search_wrapper(real_depth, ply, search_state, (-beta, -alpha), &new_position, 0);
            check_time!(search_state);
            let mut score = path_score.1;
            let mut singular_depth = real_depth;

            if !scouting && hash_score_was_a_lower_bound && these_extensions == 0 {
                let new_beta = max(beta - 400, alpha + 1);
                let mvs = moves(position);
                let mut found_one = false;

                let enemy = &position.pieces[opponent!(position.mover) as usize];
                let mut move_scores: Vec<(Move, Score)> = mvs
                    .into_iter()
                    .map(|m| (m, score_move(position, m, search_state, ply as usize, enemy)))
                    .collect();

                while !move_scores.is_empty() {
                    let mut new_position = *position;
                    make_move(position, pick_high_score_move(&mut move_scores), &mut new_position);
                    if !is_check(&new_position, position.mover) {
                        if -search_wrapper(real_depth, ply, search_state, (-new_beta, -alpha), &new_position, 0).1 > new_beta {
                            found_one = true;
                            break;
                        }
                    }
                }
                if !found_one {
                    singular_depth += 1;
                    score = search_wrapper(singular_depth, ply, search_state, (-beta, -alpha), &new_position, 0).1;
                }
            }

            if score > best_pathscore.1 {
                let mut p = vec![hash_move];
                p.extend(path_score.0);
                best_pathscore = (p, score);
                if best_pathscore.1 > alpha {
                    alpha = best_pathscore.1;
                    if alpha >= beta {
                        return cutoff(position, singular_depth, ply, search_state, best_pathscore, hash_height, hash_version,hash_move, &mut new_position);
                    }
                    hash_flag = Exact;
                }
                scout_search = true;
            }
            moves(position).into_iter().filter(|m| *m != hash_move).collect()
        } else {
            moves(position)
        }
    } else {
        moves(position)
    };

    let enemy = &position.pieces[opponent!(position.mover) as usize];
    let mut move_scores: Vec<(Move, Score)> = these_moves
        .into_iter()
        .map(|m| (m, score_move(position, m, search_state, ply as usize, enemy)))
        .collect();

    while !move_scores.is_empty() {
        let m = pick_high_score_move(&mut move_scores);

        let mut new_position = *position;
        make_move(position, m, &mut new_position);
        if !is_check(&new_position, position.mover) {
            legal_move_count += 1;

            let is_capture = captured_piece_value(position, m) > 0;
            if legal_move_count > 1 && alpha_prune_flag && !is_capture && !is_check(&new_position, new_position.mover) {
                continue;
            }

            these_extensions = extend(penultimate_pawn_push(m), these_extensions, ply, search_state);
            real_depth = depth + these_extensions;

            let lmr = if these_extensions == 0
                && legal_move_count > LMR_LEGAL_MOVES_BEFORE_ATTEMPT
                && real_depth > LMR_MIN_DEPTH
                && !is_capture
                && !search_state.killer_moves[ply as usize].contains(m.borrow())
                && !is_check(&new_position, new_position.mover)
            {
                LMR_REDUCTION
            } else {
                0
            };

            let path_score = lmr_scout_search(lmr, ply, search_state, (alpha, beta), scout_search, real_depth, &mut new_position);
            let score = path_score.1;

            check_time!(search_state);
            if score < alpha {
                update_history(position, search_state, m, -(real_depth as i64));
            }
            if score < beta {
                update_history(position, search_state, m, -(real_depth as i64));
            }
            if score > best_pathscore.1 {
                let mut p = vec![m];
                p.extend(path_score.0);
                best_pathscore = (p, score);
                if best_pathscore.1 > alpha {
                    alpha = best_pathscore.1;
                    if alpha >= beta {
                        return cutoff(position, real_depth, ply, search_state, best_pathscore, hash_height, hash_version, m, &mut new_position);
                    }
                    hash_flag = Exact;
                }
                scout_search = true;
            }
        }
    }

    if legal_move_count == 0 {
        if in_check {
            best_pathscore.1 = -MATE_SCORE + ply as Score
        } else {
            best_pathscore.1 = draw_value(position, search_state)
        }
    };

    store_hash_entry(
        position,
        real_depth,
        hash_height,
        hash_version,
        hash_flag,
        (best_pathscore.0[0], best_pathscore.1),
        search_state,
        ply,
    );

    best_pathscore
}

#[inline(always)]
fn is_draw(position: &Position, search_state: &mut SearchState) -> bool {
    repeat_position(position, search_state) || position.half_moves >= 100 || {
        let piece_count = (position.pieces[WHITE as usize].all_pieces_bitboard.count_ones()
            + position.pieces[BLACK as usize].all_pieces_bitboard.count_ones()) as u8;
        insufficient_material(position, piece_count)
    }
}

#[inline(always)]
fn repeat_position(position: &Position, search_state: &mut SearchState) -> bool {
    search_state
        .history
        .iter()
        .rev()
        .take(position.half_moves as usize)
        .filter(|p| position.zobrist_lock == **p)
        .count()
        > 1
}

#[allow(clippy::needless_range_loop)]
pub fn pick_high_score_move(move_scores: &mut Vec<(Move, Score)>) -> Move {
    let mut best_index = 0;
    let mut best_score = move_scores[0].1;
    for j in 1..move_scores.len() {
        if move_scores[j].1 > best_score {
            best_score = move_scores[j].1;
            best_index = j;
        }
    }

    move_scores.swap_remove(best_index).0
}

#[inline(always)]
fn lmr_scout_search(
    mut lmr: u8,
    ply: u8,
    search_state: &mut SearchState,
    window: Window,
    scout_search: bool,
    real_depth: u8,
    new_position: &mut Position,
) -> PathScore {
    let alpha = window.0;
    let beta = window.1;
    loop {
        let path_score = if scout_search {
            let scout_path = search_wrapper(real_depth, ply, search_state, (-alpha - 1, -alpha), new_position, lmr);
            if scout_path.1 > alpha {
                search_wrapper(real_depth, ply, search_state, (-beta, -alpha), new_position, 0)
            } else {
                (vec![0], alpha)
            }
        } else {
            search_wrapper(real_depth, ply, search_state, (-beta, -alpha), new_position, 0)
        };
        let score = path_score.1;
        if lmr == 0 || score < beta {
            break path_score;
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
fn search_wrapper(depth: u8, ply: u8, search_state: &mut SearchState, window: Window, new_position: &Position, lmr: u8) -> PathScore {
    search_state.history.push(new_position.zobrist_lock);
    let path_score = search(new_position, depth - 1 - lmr, ply + 1, window, search_state, false);
    search_state.history.pop();
    (path_score.0, -path_score.1)
}

#[inline(always)]
#[allow(clippy::too_many_arguments)]
fn cutoff(
    position: &Position,
    depth: u8,
    ply: u8,
    search_state: &mut SearchState,
    best_pathscore: PathScore,
    hash_height: u8,
    hash_version: u32,
    m: Move,
    new_position: &mut Position,
) -> PathScore {
    store_hash_entry(
        position,
        depth,
        hash_height,
        hash_version,
        Lower,
        (m, best_pathscore.1),
        search_state,
        ply,
    );
    update_history(position, search_state, m, depth as i64 * depth as i64);
    update_killers(position, ply, search_state, m, new_position, best_pathscore.1);
    best_pathscore
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
    ((position.mover * 6)
        + match m & PIECE_MASK_FULL {
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
fn update_killers(position: &Position, ply: u8, search_state: &mut SearchState, m: Move, new_position: &mut Position, score: Score) {
    if score > MATE_START {
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
    side_total_non_pawn_count(position, position.mover) + side_total_non_pawn_count(position, opponent!(position.mover)) >= 2
}

#[inline(always)]
fn side_total_non_pawn_count(position: &Position, side: Mover) -> u32 {
    (position.pieces[side as usize].bishop_bitboard
        | position.pieces[side as usize].knight_bitboard
        | position.pieces[side as usize].rook_bitboard
        | position.pieces[side as usize].queen_bitboard)
        .count_ones()
}
