use std::sync::mpsc::{Sender};
use std::time::{Instant};
use crate::bitboards::{RANK_2_BITS, RANK_7_BITS};
use crate::engine_constants::{DEBUG, DEPTH_REMAINING_FOR_RD_INCREASE, MAX_QUIESCE_DEPTH, NULL_MOVE_REDUCE_DEPTH, PAWN_VALUE, QUEEN_VALUE};
use crate::evaluate::{evaluate};
use crate::fen::{algebraic_move_from_move};
use crate::hash::{zobrist_index, ZOBRIST_KEY_MOVER_SWITCH};
use crate::make_move::make_move;
use crate::move_constants::PROMOTION_FULL_MOVE_MASK;
use crate::move_scores::{score_move, score_quiesce_move};
use crate::moves::{is_check, moves, quiesce_moves, verify_move};
use crate::opponent;
use crate::types::{Move, Position, MoveList, Score, SearchState, Window, MoveScoreList, MoveScore, HashIndex, HashLock, HashEntry, BoundType, WHITE, Mover, HistoryScore};
use crate::types::BoundType::{Exact, Lower, Upper};
use crate::utils::{captured_piece_value, from_square_part, to_square_part};

pub const MAX_SCORE: Score = 30000;
pub const MATE_MARGIN: Score = 1000;
pub const MATE_START: Score = MAX_SCORE - MATE_MARGIN;

#[macro_export]
macro_rules! time_remains {
    ($end_time:expr) => {
        $end_time > Instant::now()
    }
}

#[macro_export]
macro_rules! time_expired {
    ($end_time:expr) => {
        Instant::now() >= $end_time
    }
}

#[macro_export]
macro_rules! check_time {
    ($nodes:expr, $end_time:expr) => {
        if $nodes % 100000 == 0 {
            if $end_time < Instant::now() {
                return 0;
            }
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

pub fn iterative_deepening(position: &Position, max_depth: u8, end_time: Instant, search_state: &mut SearchState, tx: &Sender<String>) -> Move {

    let start_time = Instant::now();

    let mut legal_moves: MoveScoreList = moves(position).into_iter().filter(|m| {
        let mut new_position = *position;
        make_move(position, *m, &mut new_position);
        !is_check(&new_position, position.mover)
    }).map(|m| {
        (m, -MAX_SCORE)
    }).collect();

    for i in 0..2 as usize {
        for j in 0..64 as usize {
            for k in 0..64 as usize {
                search_state.history_moves[i][j][k] = 0;
            }
        }
    }
    search_state.highest_history_score = 0;

    if search_state.history.len() == 0 {
        search_state.history.push(position.zobrist_lock)
    }

    let mut aspiration_window = (-MAX_SCORE, MAX_SCORE);

    let mut current_best: MoveScore = (0, -MAX_SCORE);

    for iterative_depth in 1..=max_depth {
        let mut aspire_best = start_search(position, &mut legal_moves, end_time, search_state, tx, iterative_depth, start_time, aspiration_window);

        debug_out!(println!("=========================================================================================================================="));
        debug_out!(println!("Best at depth {} within aspiration window ({},{}) is {} with score of {}", iterative_depth, aspiration_window.0, aspiration_window.1,
                 algebraic_move_from_move(aspire_best.0), aspire_best.1));

        if aspire_best.1 > aspiration_window.0 && aspire_best.1 < aspiration_window.1 {
            current_best = aspire_best
        } else {
            if time_remains!(end_time) {
                if aspire_best.1 <= aspiration_window.0 {
                    debug_out!(println!("Failed low"));
                    aspire_best = start_search(position, &mut legal_moves, end_time, search_state, tx, iterative_depth, start_time, (-MAX_SCORE, aspiration_window.1));
                    debug_out!(println!("Research gives best at depth {} within aspiration window ({},{}) is {} with score of {}", iterative_depth, -MAX_SCORE, aspiration_window.1,
                             algebraic_move_from_move(aspire_best.0), aspire_best.1));
                } else if aspire_best.1 >= aspiration_window.1 {
                    debug_out!(println!("Failed high"));
                    aspire_best = start_search(position, &mut legal_moves, end_time, search_state, tx, iterative_depth, start_time, (aspiration_window.0, MAX_SCORE));
                    debug_out!(println!("Research gives best at depth {} within aspiration window ({},{}) is {} with score of {}", iterative_depth, aspiration_window.0, MAX_SCORE,
                             algebraic_move_from_move(aspire_best.0), aspire_best.1));
                };
            }
            if time_remains!(end_time) {
                current_best = aspire_best
            }
        };

        if time_expired!(end_time) {
            if current_best.0 == 0 {
                panic!("Didn't have time to do anything.")
            }
            debug_out!(println!("Time expired, returning {}", algebraic_move_from_move(current_best.0)));
            return current_best.0
        }

        debug_out!(println!("Sorting legal_moves"));
        legal_moves.sort_by(|(_, a), (_, b) | b.cmp(a));
        debug_out!(println!("Resetting scores to minimum"));
        legal_moves = legal_moves.into_iter().map(|m| {
            (m.0, -MAX_SCORE)
        }).collect();

        debug_out!(println!("Setting aspiration window to ({},{})", current_best.1 - ASPIRATION_RADIUS, current_best.1 + ASPIRATION_RADIUS));
        aspiration_window = (current_best.1 - ASPIRATION_RADIUS, current_best.1 + ASPIRATION_RADIUS)
    }

    legal_moves[0].0
}

pub fn start_search(position: &Position, legal_moves: &mut MoveScoreList, end_time: Instant, search_state: &mut SearchState, tx: &Sender<String>, iterative_depth: u8, start_time: Instant, aspiration_window: Window) -> MoveScore {

    let mut current_best: MoveScore = (legal_moves[0].0, -MAX_SCORE);
    for move_num in 0..legal_moves.len() {
        let mut new_position = *position;
        make_move(position, legal_moves[move_num].0, &mut new_position);
        search_state.history.push(new_position.zobrist_lock);

        let mut score = -search(&new_position, iterative_depth, 1, (-aspiration_window.1, -aspiration_window.0), end_time, search_state, &tx, start_time);
        if score > MATE_START { score -= 1 } else if score < -MATE_START { score += 1 };

        search_state.history.pop();
        legal_moves[move_num].1 = score;
        if score > current_best.1 {
            if time_remains!(end_time) {
                current_best = legal_moves[move_num];
                if score > aspiration_window.0 && score < aspiration_window.1 {
                    send_info(search_state, tx, start_time, iterative_depth, &mut current_best)
                }
            }
        }

        if time_expired!(end_time) {
            return current_best;
        }
    }
    current_best

}

#[inline(always)]
pub fn store_hash_entry(index: HashIndex, lock: HashLock, height: u8, existing_height: u8, existing_version: u32, bound: BoundType, movescore: MoveScore, search_state: &mut SearchState) {
    if height >= existing_height && search_state.hash_table_version > existing_version {
        search_state.hash_table.insert(index, HashEntry { score: movescore.1, version: search_state.hash_table_version, height, mv: movescore.0, bound, lock, });
    }
}

#[inline(always)]
pub fn search(position: &Position, depth: u8, ply: u8, window: Window, end_time: Instant, search_state: &mut SearchState, tx: &Sender<String>, start_time: Instant) -> Score {

    search_state.nodes += 1;
    check_time!(search_state.nodes, end_time);

    if search_state.history.iter().rev().take(position.half_moves as usize).filter(|p| position.zobrist_lock == **p).count() > 1 {
        return 0
    }

    if depth == 0 {
        quiesce(position, MAX_QUIESCE_DEPTH, ply, window, end_time, search_state, tx, start_time)
    } else {

        let index = zobrist_index(position.zobrist_lock);

        let mut best_movescore: MoveScore = (0,-MAX_SCORE);
        let mut alpha = window.0;
        let mut beta = window.1;
        let mut hash_height = 0;
        let mut hash_flag = Upper;
        let mut hash_version = 0;

        let hash_entry = search_state.hash_table.get(&index);
        let hash_move = match hash_entry {
            Some(x) => {
                if x.lock == position.zobrist_lock {
                    if x.height >= depth {
                        hash_height = x.height;
                        hash_version = x.version;
                        if x.bound == BoundType::Exact {
                            search_state.hash_hits_exact += 1;
                            return x.score
                        }
                        if x.bound == BoundType::Lower { alpha = x.score }
                        if x.bound == BoundType::Upper { beta = x.score }
                        if alpha >= beta { return x.score }
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

        let null_move_reduce_depth = if depth > DEPTH_REMAINING_FOR_RD_INCREASE { NULL_MOVE_REDUCE_DEPTH + 1 } else { NULL_MOVE_REDUCE_DEPTH };

        if !search_state.is_on_null_move && depth > null_move_reduce_depth && null_move_material(position) && !is_check(position, position.mover) {
            if evaluate(position) > beta {
                let mut new_position = *position;
                switch_mover(&mut new_position);
                let score = adjust_mate_score_for_ply(1, -search(&new_position, depth - 1 - NULL_MOVE_REDUCE_DEPTH, ply + 1, (-beta, (-beta) + 1), end_time, search_state, tx, start_time));
                if score >= beta {
                    return beta;
                }
                switch_mover(&mut new_position);
            }
        }

        let mut legal_move_count = 0;

        if verify_move(position, hash_move) {
            let mut new_position = *position;
            make_move(position, hash_move, &mut new_position);
            if !is_check(&new_position, position.mover) {
                legal_move_count += 1;
                let score = search_wrapper(depth, ply, end_time, search_state, tx, start_time, (-beta, -alpha), &mut new_position, 0);
                check_time!(search_state.nodes, end_time);
                if score > best_movescore.1 {
                    best_movescore = (hash_move, score);
                    if best_movescore.1 > alpha {
                        alpha = best_movescore.1;
                        if alpha >= beta {
                            return cutoff(position, depth, ply, search_state, index, best_movescore, hash_height, hash_version, hash_move, &mut new_position);
                        }
                        hash_flag = Exact;
                    }
                }
            }
        }

        let these_moves = moves(position);
        let mut move_scores: Vec<(Move, Score)> = these_moves.into_iter().map(|m| {
            (m, score_move(position, m, search_state, ply as usize))
        }).collect();
        move_scores.sort_by(|(_, a), (_, b) | b.cmp(a));
        let move_list: Vec<Move> = move_scores.into_iter().map(|(m,_)| { m }).filter(|m| { *m != hash_move }).collect();

        for m in move_list {
            let mut new_position = *position;
            make_move(position, m, &mut new_position);
            if !is_check(&new_position, position.mover) {
                legal_move_count += 1;
                let lmr = if legal_move_count > 6 && depth > 3 && !is_check(position, position.mover) && !is_check(&new_position, new_position.mover) && captured_piece_value(position, m) == 0 { 1 } else { 0 };
                let score = search_wrapper(depth, ply, end_time, search_state, tx, start_time, (-beta, -alpha), &mut new_position, lmr);
                check_time!(search_state.nodes, end_time);
                if score < beta {
                    update_history(position, depth, search_state, m, false);
                }
                if score > best_movescore.1 {
                    best_movescore = (m, score);
                    if best_movescore.1 > alpha {
                        alpha = best_movescore.1;
                        if alpha >= beta {
                            return cutoff(position, depth, ply, search_state, index, best_movescore, hash_height, hash_version, m, &mut new_position);
                        }
                        hash_flag = Exact;
                    }
                }
            }
        }

        if legal_move_count == 0 {
            if !is_check(position, position.mover) {
                best_movescore.1 = 0
            }
        };

        store_hash_entry(index, position.zobrist_lock, depth, hash_height, hash_version, hash_flag, best_movescore, search_state);

        adjust_mate_score_for_ply(0, best_movescore.1)
    }
}

#[inline(always)]
fn switch_mover(new_position: &mut Position) {
    new_position.mover ^= 1;
    new_position.zobrist_lock ^= ZOBRIST_KEY_MOVER_SWITCH;
}

#[inline(always)]
fn search_wrapper(depth: u8, ply: u8, end_time: Instant, search_state: &mut SearchState, tx: &Sender<String>, start_time: Instant, window: Window, new_position: &mut Position, lmr: u8) -> Score {
    search_state.history.push(new_position.zobrist_lock);
    let score = adjust_mate_score_for_ply(1, -search(&new_position, depth - 1 - lmr, ply + 1, window, end_time, search_state, tx, start_time));
    search_state.history.pop();
    score
}

#[inline(always)]
fn cutoff(position: &Position, depth: u8, ply: u8, search_state: &mut SearchState, index: HashIndex, best_movescore: MoveScore, hash_height: u8, hash_version: u32, m: Move, mut new_position: &mut Position) -> Score {
    store_hash_entry(index, position.zobrist_lock, depth, hash_height, hash_version, Lower, best_movescore, search_state);
    update_history(position, depth, search_state, m, true);
    update_killers(position, ply, search_state, m, &mut new_position);
    best_movescore.1
}

#[inline(always)]
fn update_history(position: &Position, depth: u8, search_state: &mut SearchState, m: Move, cutoff: bool) {
    if depth < 8 && captured_piece_value(position, m) == 0 {
        let f = from_square_part(m) as usize;
        let t = to_square_part(m) as usize;

        search_state.history_moves[position.mover as usize][f][t] += if cutoff {
            (depth * depth) as HistoryScore
        } else {
            0// - (depth * depth) as HistoryScore
        };

        if search_state.history_moves[position.mover as usize][f][t] < 0 {
            search_state.history_moves[position.mover as usize][f][t] = 0;
        }

        if search_state.history_moves[position.mover as usize][f][t] > search_state.highest_history_score {
            search_state.highest_history_score = search_state.history_moves[position.mover as usize][f][t];
        }
    }
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
pub fn quiesce(position: &Position, depth: u8, ply: u8, window: Window, end_time: Instant, search_state: &mut SearchState, tx: &Sender<String>, start_time: Instant) -> Score {

    check_time!(search_state.nodes, end_time);

    search_state.nodes += 1;

    let eval = evaluate(position);

    if depth == 0 {
        return eval;
    }

    let mut alpha = window.0;
    let beta = window.1;

    if eval >= beta {
        return beta;
    }

    let promote_from_rank = if position.mover == WHITE { RANK_7_BITS } else { RANK_2_BITS };
    let mut delta = QUEEN_VALUE;
    if position.pieces[position.mover as usize].pawn_bitboard & promote_from_rank != 0 {
        delta += QUEEN_VALUE - (PAWN_VALUE * 2)
    }

    if eval < alpha - delta {
        return alpha;
    }

    if alpha < eval {
        alpha = eval;
    }

    let in_check = is_check(position, position.mover);

    let mut move_scores: Vec<(Move, Score)> =
        if in_check {
            moves(position).into_iter().map(|m| {
                (m, score_move(position, m, search_state, ply as usize))
            }).collect()
        } else {
            quiesce_moves(position).into_iter().map(|m| {
                (m, score_quiesce_move(position, m))
            }).collect()
        };

    move_scores.sort_by(|(_, a), (_, b) | b.cmp(a));
    let move_list: MoveList = move_scores.into_iter().map(|(m,_)| { m }).collect();

    let mut legal_move_count = 0;

    for m in move_list {
        let mut new_position = *position;

        if eval + captured_piece_value(position, m) + 200 > alpha {
            make_move(position, m, &mut new_position);
            if !is_check(&new_position, position.mover) {
                legal_move_count += 1;
                let score = adjust_mate_score_for_ply(ply, -quiesce(&new_position, depth - 1, ply+1, (-beta, -alpha), end_time, search_state, tx, start_time));
                check_time!(search_state.nodes, end_time);
                if score >= beta {
                    return beta;
                }
                if score > alpha {
                    alpha = score;
                }
            }
        }
    }

    if legal_move_count == 0 && in_check {
        -MAX_SCORE + ply as Score
    } else {
        alpha
    }
}

fn send_info(search_state: &mut SearchState, tx: &Sender<String>, start_time: Instant, iterative_depth: u8, current_best: &mut MoveScore) {
    if start_time.elapsed().as_millis() > 0 {
        let nps = (search_state.nodes as f64 / start_time.elapsed().as_millis() as f64) * 1000.0;
        let s = "info score cp ".to_string() + &*(current_best.1 as i64).to_string() +
            &*" depth ".to_string() + &*iterative_depth.to_string() +
            &*" time ".to_string() + &*start_time.elapsed().as_millis().to_string() +
            &*" nodes ".to_string() + &*search_state.nodes.to_string() +
            &*" pv ".to_string() + &*algebraic_move_from_move(current_best.0).to_string() +
            &*" nps ".to_string() + &*(nps as u64).to_string();

        let result = tx.send(s.clone());
        debug_out!(println!("{}", s));
        match result {
            Err(_e) => {},
            _ => {}
        }
    }
}