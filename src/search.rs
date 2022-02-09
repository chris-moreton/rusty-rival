use std::ops::Add;
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};
use rand::Rng;
use crate::evaluate::evaluate;
use crate::fen::algebraic_move_from_move;
use crate::hash::{zobrist_index, zobrist_lock};
use crate::make_move::make_move;
use crate::moves::{is_check, moves};
use crate::types::{Bound, Move, Position, MoveList, Score, SearchState, Window, WHITE, MoveScoreList, MoveScore, UciState, HashIndex, HashLock, HashEntry, BoundType};

pub const MAX_SCORE: Score = 30000;

pub fn start_search(position: &Position, max_depth: u8, end_time: Instant, search_state: &mut SearchState, tx: Sender<String>) -> Move {

    let mut legal_moves: MoveScoreList = moves(position).into_iter().filter(|m| {
        let mut new_position = *position;
        make_move(position, *m, &mut new_position);
        !is_check(&new_position, position.mover)
    }).map(|m| {
        (m, 0)
    }).collect();

    let aspiration_window: Window = (-30000, 30000);

    for iterative_depth in 1..=max_depth {
        let mut current_best: MoveScore = legal_moves[0];
            for move_num in 1..legal_moves.len() {
            if end_time < Instant::now() {
                return legal_moves[0].0;
            }
            let mut new_position = *position;
            make_move(position, legal_moves[move_num].0, &mut new_position);
            let score = -search(&new_position, iterative_depth, aspiration_window, end_time, search_state, &tx);
            legal_moves[move_num].1 = score;
            if score > current_best.1 {
                current_best = legal_moves[move_num];
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

pub fn store_hash_entry(index: HashIndex, lock: HashLock, height: u8, bound: BoundType, mv: Move, score: Score, search_state: &mut SearchState) {
    search_state.hash_table.insert(index, HashEntry { score, height, mv, bound, lock, });
}

pub fn search(position: &Position, depth: u8, window: Window, end_time: Instant, search_state: &mut SearchState, tx: &Sender<String>) -> Score {
    search_state.nodes += 1;
    check_time!(search_state.nodes, end_time);
    if depth == 0 {
        evaluate(position)
    } else {
        let lock = zobrist_lock(position);
        let index = zobrist_index(lock);
        let mut hash_move = 0;

        let mut best_score = -MAX_SCORE;
        let mut best_move = 0;
        let mut alpha = window.0;
        let beta = window.1;
        let mut moves = moves(position);

        let hash_entry = search_state.hash_table.get(&index);
        match hash_entry {
            Some(x) => {
                if x.lock == lock && x.height >= depth {
                    if x.bound == BoundType::Exact  {
                        search_state.hash_hits_exact += 1;
                        return x.score;
                    }
                    if x.bound == BoundType::Lower  {
                        search_state.hash_hits_exact += 1;
                        if x.score > beta {
                            return x.score
                        }
                    }
                }

            },
            None => { }
        }

        for m in moves {
            let mut new_position = *position;
            make_move(position, m, &mut new_position);
            if !is_check(&new_position, position.mover) {
                let score = -search(&new_position, depth-1, (-beta, -alpha), end_time, search_state, tx);
                check_time!(search_state.nodes, end_time);
                if score > alpha {
                    alpha = score;
                    if score > best_score {
                        best_score = score;
                        best_move = m;
                        if best_score >= beta {
                            if best_score > -MAX_SCORE {
                                store_hash_entry(index, lock, depth, BoundType::Lower, best_move, best_score, search_state);
                            }
                            return best_score;
                        }
                    }
                }
            }
        }
        if best_score > -MAX_SCORE {
            store_hash_entry(index, lock, depth, BoundType::Exact, best_move, best_score, search_state);
        }
        best_score
    }
}
