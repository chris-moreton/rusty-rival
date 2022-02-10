use std::cmp::Ordering;
use std::sync::mpsc::{Sender};
use std::time::{Instant};
use crate::bitboards::bit;
use crate::evaluate::evaluate;
use crate::fen::algebraic_move_from_move;
use crate::hash::{zobrist_index};
use crate::make_move::make_move;
use crate::moves::{capture_moves, is_check, moves};
use crate::opponent;
use crate::types::{Move, Position, MoveList, Score, SearchState, Window, MoveScoreList, MoveScore, HashIndex, HashLock, HashEntry, BoundType};
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

    let aspiration_window: Window = (-30000, 30000);

    for iterative_depth in 1..=max_depth {
        let mut current_best: MoveScore = (legal_moves[0].0, -MAX_SCORE);
        for move_num in 0..legal_moves.len() {
            let mut new_position = *position;
            make_move(position, legal_moves[move_num].0, &mut new_position);
            let score = -search(&new_position, iterative_depth, aspiration_window, end_time, search_state, &tx, start_time);
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

pub fn store_hash_entry(index: HashIndex, lock: HashLock, height: u8, bound: BoundType, mv: Move, score: Score, search_state: &mut SearchState) {
    search_state.hash_table.insert(index, HashEntry { score, height, mv, bound, lock, });
}

pub fn search(position: &Position, depth: u8, window: Window, end_time: Instant, search_state: &mut SearchState, tx: &Sender<String>, start_time: Instant) -> Score {
    search_state.nodes += 1;
    check_time!(search_state.nodes, end_time);

    if depth == 0 {
        quiesce(position, 8, window, end_time, search_state, tx, start_time)
    } else {
        let index = zobrist_index(position.zobrist_lock);

        let mut best_score = -MAX_SCORE;
        let mut best_move = 0;
        let mut alpha = window.0;
        let mut beta = window.1;
        let mut move_list: MoveList;

        let hash_entry = search_state.hash_table.get(&index);
        match hash_entry {
            Some(x) => {
                if x.lock == position.zobrist_lock && x.height >= depth {
                    if x.bound == BoundType::Exact  {
                        search_state.hash_hits_exact += 1;
                        return x.score;
                    }
                    if x.bound == BoundType::Lower  {
                        alpha = x.score
                    }
                    if x.bound == BoundType::Upper  {
                        beta = x.score
                    }
                    if alpha >= beta {
                        return x.score
                    }
                }
                move_list = moves(position);
                move_list.sort_by(|a, _b| if *a == x.mv { Ordering::Less } else { Ordering::Equal } )
            },
            None => {
                move_list = moves(position);
            }
        }

        let enemy = position.pieces[opponent!(position.mover) as usize];

        let mut move_scores: Vec<(Move, Score)> = move_list.into_iter().map(|m| {
            if enemy.all_pieces_bitboard & bit(to_square_part(m)) != 0 { (m,0) } else { (m,1) }
        }).collect();

        move_scores.sort_by(|(_, a), (_, b) | a.cmp(b));

        move_list = move_scores.into_iter().map(|(m,_)| {
            m
        }).collect();

        for m in move_list {
            let mut new_position = *position;
            make_move(position, m, &mut new_position);
            if !is_check(&new_position, position.mover) {
                let score = -search(&new_position, depth-1, (-beta, -alpha), end_time, search_state, tx, start_time);
                check_time!(search_state.nodes, end_time);
                if score > best_score {
                    best_score = score;
                    best_move = m;
                    if best_score > alpha {
                        alpha = best_score;
                        if alpha >= beta {
                            store_hash_entry(index, position.zobrist_lock, depth, BoundType::Lower, best_move, best_score, search_state);
                            return best_score;
                        }
                    }
                }
            }
        }
        store_hash_entry(index, position.zobrist_lock, depth, if best_score > -MAX_SCORE { BoundType::Exact } else { BoundType::Lower }, best_move, best_score, search_state);
        best_score
    }
}

pub fn quiesce(position: &Position, depth: u8, window: Window, end_time: Instant, search_state: &mut SearchState, tx: &Sender<String>, start_time: Instant) -> Score {
//    evaluate(position)

    check_time!(search_state.nodes, end_time);

    search_state.nodes += 1;

    let eval = evaluate(position);
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
