use std::sync::mpsc::Sender;
use std::time::Instant;
use rand::Rng;
use crate::evaluate::evaluate;
use crate::make_move::make_move;
use crate::moves::{is_check, moves};
use crate::types::{Bound, Move, Position, MoveList, Score, SearchState, Window, WHITE, MoveScoreList};

pub fn start_search(position: &Position, max_depth: u8, end_time: Instant, search_state: &mut SearchState, tx: Sender<String>) -> Move {

    let mut legal_moves: MoveScoreList = moves(position).into_iter().filter(|m| {
        let mut new_position = *position;
        make_move(position, *m, &mut new_position);
        !is_check(&new_position, position.mover)
    }).map(|m| {
        (m, 0)
    }).collect();

    let aspiration_window: Window = (-30000, 30000);

    for iterative_depth in 1..=1 {
        for move_num in 1..legal_moves.len() {
            let mut new_position = *position;
            make_move(position, legal_moves[move_num].0, &mut new_position);
            legal_moves[move_num].1 = -search(&new_position, iterative_depth, aspiration_window, &tx);
        }
        legal_moves.sort_by(|(_, a), (_, b) | b.cmp(a))
    }

    legal_moves[0].0

}


pub fn search_zero(position: &Position, depth: u8, tx: Sender<String>) {
}

pub fn search(position: &Position, depth: u8, window: Window, _tx: &Sender<String>) -> Score {
    evaluate(position)
}

pub fn is_book_move_available(position: &Position) -> bool {
    false
}

