use std::sync::mpsc::Sender;
use std::time::Instant;
use rand::Rng;
use crate::evaluate::evaluate;
use crate::fen::algebraic_move_from_move;
use crate::make_move::make_move;
use crate::moves::{is_check, moves};
use crate::types::{Bound, Move, Position, MoveList, Score, SearchState, Window, WHITE, MoveScoreList};

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

    for iterative_depth in max_depth..=max_depth {
        for move_num in 1..legal_moves.len() {
            let mut new_position = *position;
            make_move(position, legal_moves[move_num].0, &mut new_position);
            legal_moves[move_num].1 = -search(&new_position, iterative_depth, aspiration_window, &tx);
            println!("{} {}", algebraic_move_from_move(legal_moves[move_num].0), legal_moves[move_num].1)
        }
        legal_moves.sort_by(|(_, a), (_, b) | b.cmp(a))
    }

    legal_moves[0].0

}

pub fn search(position: &Position, depth: u8, window: Window, tx: &Sender<String>) -> Score {
    if depth == 0 {
        evaluate(position)
    } else {
        let mut best_score = -MAX_SCORE;
        for m in moves(position) {
            let mut new_position = *position;
            make_move(position, m, &mut new_position);
            if !is_check(&new_position, position.mover) {
                let score = -search(&new_position, depth-1, (-window.1, -best_score), tx);
                if score > best_score {
                    best_score = score;
                    if best_score >= window.1 {
                        return best_score;
                    }
                }
            }
        }
        best_score
    }
}
