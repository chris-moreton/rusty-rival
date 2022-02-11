use std::process::exit;
use std::sync::mpsc::{Sender};
use std::time::{Instant};
use crate::bitboards::bit;
use crate::engine_constants::MAX_QUIESCE_DEPTH;
use crate::evaluate::{BISHOP_VALUE, evaluate, KNIGHT_VALUE, PAWN_VALUE, QUEEN_VALUE, ROOK_VALUE};
use crate::fen::algebraic_move_from_move;
use crate::hash::{zobrist_index};
use crate::make_move::make_move;
use crate::move_constants::{PROMOTION_BISHOP_MOVE_MASK, PROMOTION_FULL_MOVE_MASK, PROMOTION_KNIGHT_MOVE_MASK, PROMOTION_ROOK_MOVE_MASK};
use crate::moves::{capture_moves, is_check, moves};
use crate::opponent;
use crate::types::{Move, Position, MoveList, Score, SearchState, Window, MoveScoreList, MoveScore, HashIndex, HashLock, HashEntry, BoundType, Pieces, Square, WHITE, BLACK};
use crate::types::BoundType::{Exact, Lower, Upper};
use crate::utils::to_square_part;

pub const MAX_SCORE: Score = 30000;

pub fn start_search(position: &Position, max_depth: u8, end_time: Instant, search_state: &mut SearchState, tx: &Sender<String>) -> Move {

    let start_time = Instant::now();

    let mut legal_moves: MoveScoreList = moves(position).into_iter().filter(|m| {
        let mut new_position = *position;
        make_move(position, *m, &mut new_position);
        !is_check(&new_position, position.mover)
    }).map(|m| {
        (m, 0)
    }).collect();

    let aspiration_window: Window = (-MAX_SCORE, MAX_SCORE);

    if search_state.history.len() == 0 {
        search_state.history.push(position.zobrist_lock)
    }

    for iterative_depth in 1..=max_depth {
        let mut current_best: MoveScore = (legal_moves[0].0, -MAX_SCORE);
        for move_num in 0..legal_moves.len() {
            let mut new_position = *position;
            make_move(position, legal_moves[move_num].0, &mut new_position);
            search_state.history.push(new_position.zobrist_lock);
            let score = -search(&new_position, iterative_depth, 1, aspiration_window, end_time, search_state, &tx, start_time, false);
            search_state.history.pop();
            if Instant::now() > end_time {
                return legal_moves[0].0;
            }
            legal_moves[move_num].1 = score;
            if score > current_best.1 {
                current_best = legal_moves[move_num];
                if start_time.elapsed().as_millis() > 0 {
                    let nps = (search_state.nodes as f64 / start_time.elapsed().as_millis() as f64) * 1000.0;
                    let result = tx.send("info score cp ".to_string() + &*(current_best.1 as i64).to_string() +
                        &*" depth ".to_string() + &*iterative_depth.to_string() +
                        &*" time ".to_string() + &*start_time.elapsed().as_millis().to_string() +
                        &*" nodes ".to_string() + &*search_state.nodes.to_string() +
                        &*" pv ".to_string() + &*algebraic_move_from_move(current_best.0).to_string() +
                        &*" nps ".to_string() + &*(nps as u64).to_string()
                    );
                    match result {
                        Err(_e) => { },
                        _ => {}
                    }
                }
            }

        }
        legal_moves.sort_by(|(_, a), (_, b) | b.cmp(a))
    }

    legal_moves[0].0

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

#[inline(always)]
pub fn store_hash_entry(index: HashIndex, lock: HashLock, height: u8, existing_height: u8, bound: BoundType, mv: Move, score: Score, search_state: &mut SearchState) {
    if score < 29000 && score > -29000 && height >= existing_height {
        search_state.hash_table.insert(index, HashEntry { score, height, mv, bound, lock, });
    }
}

#[inline(always)]
pub fn search(position: &Position, depth: u8, ply: u8, window: Window, end_time: Instant, search_state: &mut SearchState, tx: &Sender<String>, start_time: Instant, on_null_move: bool) -> Score {

    // assert_eq!(search_state.history.len() - 1, ply as usize);

    search_state.nodes += 1;
    check_time!(search_state.nodes, end_time);

    if search_state.history.iter().filter(|p| position.zobrist_lock == **p).count() > 1 {
        return 0
    }

    if depth == 0 {
        quiesce(position, MAX_QUIESCE_DEPTH, window, end_time, search_state, tx, start_time)
    } else {

        let index = zobrist_index(position.zobrist_lock);

        let worst_case = -(MAX_SCORE-ply as Score);
        let mut best_score = worst_case;
        let mut best_move = 0;
        let mut alpha = window.0;
        let mut beta = window.1;
        let mut hash_move: Move = 0;
        let mut hash_height = 0;
        let mut hash_flag = Upper;

        let hash_entry = search_state.hash_table.get(&index);
        match hash_entry {
            Some(x) => {
                if x.lock == position.zobrist_lock && x.height >= depth {
                    hash_height = x.height;
                    let adjusted_score = x.score;
                    if x.bound == BoundType::Exact  {
                        search_state.hash_hits_exact += 1;
                        return adjusted_score;
                    }
                    if x.bound == BoundType::Lower  {
                        alpha = adjusted_score
                    }
                    if x.bound == BoundType::Upper  {
                        beta = adjusted_score
                    }
                    if alpha >= beta {
                        return adjusted_score
                    }
                }
                hash_move = x.mv;
            },
            None => {
            }
        }

        if !on_null_move && depth > 2 && null_move_material(position) && !is_check(position, position.mover) {
            let mut new_position = *position;
            new_position.mover ^= 1;
            let score = -search(&new_position, depth-1-1, ply+1, (-beta, (-beta)+1), end_time, search_state, tx, start_time, true);
            if score >= beta {
                return beta;
            }
            new_position.mover ^= 1;
        }

        let enemy = position.pieces[opponent!(position.mover) as usize];
        let mut move_scores: Vec<(Move, Score)> = moves(position).into_iter().map(|m| {
            (m, score_move(position, hash_move, &enemy, m, to_square_part(m)))
        }).collect();
        move_scores.sort_by(|(_, a), (_, b) | b.cmp(a));
        let move_list: MoveList = move_scores.into_iter().map(|(m,_)| { m }).collect();

        for m in move_list {
            let mut new_position = *position;
            make_move(position, m, &mut new_position);
            search_state.history.push(new_position.zobrist_lock);
            if !is_check(&new_position, position.mover) {
                let score = -search(&new_position, depth-1, ply+1, (-beta, -alpha), end_time, search_state, tx, start_time, false);
                search_state.history.pop();
                check_time!(search_state.nodes, end_time);
                if score > best_score {
                    best_score = score;
                    best_move = m;
                    if best_score > alpha {
                        alpha = best_score;
                        if alpha >= beta {
                            store_hash_entry(index, position.zobrist_lock, depth, hash_height, Lower, best_move,
                                             best_score,
                                             search_state);
                            return best_score;
                        }
                        hash_flag = Exact;
                    }
                }
            } else {
                search_state.history.pop();
            }
        }
        if best_score == worst_case && !is_check(position, position.mover) {
            best_score = 0;
        }
        store_hash_entry(index, position.zobrist_lock, depth, hash_height, hash_flag, best_move,
                         best_score,
                         search_state);
        best_score
    }
}

#[inline(always)]
fn null_move_material(position: &Position) -> bool {
    let white_total =
        position.pieces[WHITE as usize].bishop_bitboard.count_ones() +
        position.pieces[WHITE as usize].knight_bitboard.count_ones() +
        position.pieces[WHITE as usize].rook_bitboard.count_ones() +
        position.pieces[WHITE as usize].queen_bitboard.count_ones();

    let black_total =
        position.pieces[BLACK as usize].bishop_bitboard.count_ones() +
        position.pieces[BLACK as usize].knight_bitboard.count_ones() +
        position.pieces[BLACK as usize].rook_bitboard.count_ones() +
        position.pieces[BLACK as usize].queen_bitboard.count_ones();

    white_total >= 2 && black_total >= 2
}

#[inline(always)]
fn score_move(position: &Position, hash_move: Move, enemy: &Pieces, m: Move, tsp: Square) -> Score {
    let score = if m == hash_move {
        10000
    } else if enemy.all_pieces_bitboard & bit(tsp) != 0 {
        piece_value(&enemy, tsp)
    } else if m & PROMOTION_FULL_MOVE_MASK != 0 {
        let mask = m & PROMOTION_FULL_MOVE_MASK;
        let score = if mask == PROMOTION_ROOK_MOVE_MASK {
            ROOK_VALUE
        } else if mask == PROMOTION_BISHOP_MOVE_MASK {
            BISHOP_VALUE
        } else if mask == PROMOTION_KNIGHT_MOVE_MASK {
            KNIGHT_VALUE
        } else {
            QUEEN_VALUE
        };
        score
    } else if tsp == position.en_passant_square {
        PAWN_VALUE
    } else {
        0
    };
    score
}

#[inline(always)]
pub fn piece_value(pieces: &Pieces, to: Square) -> Score {
    let bb = bit(to);
    if pieces.pawn_bitboard & bb != 0 {
        return PAWN_VALUE;
    }
    if pieces.knight_bitboard & bb != 0 {
        return KNIGHT_VALUE;
    }
    if pieces.rook_bitboard & bb != 0 {
        return ROOK_VALUE;
    }
    if pieces.queen_bitboard & bb != 0 {
        return QUEEN_VALUE;
    }
    if pieces.bishop_bitboard & bb != 0 {
        return BISHOP_VALUE;
    }
    0
}

#[inline(always)]
pub fn quiesce(position: &Position, depth: u8, window: Window, end_time: Instant, search_state: &mut SearchState, tx: &Sender<String>, start_time: Instant) -> Score {

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

    if alpha < eval {
        alpha = eval;
    }

    for m in capture_moves(position) {
        let mut new_position = *position;
        make_move(position, m, &mut new_position);
        if !is_check(&new_position, position.mover) {
            let score = -quiesce(&new_position, depth-1, (-beta, -alpha), end_time, search_state, tx, start_time);
            check_time!(search_state.nodes, end_time);
            if score >= beta {
                return beta;
            }
            if score > alpha {
                alpha = score;
            }
        }
    }

    alpha
}
