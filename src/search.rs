use std::sync::mpsc::Sender;
use std::time::Instant;
use rand::Rng;
use crate::make_move::make_move;
use crate::moves::{is_check, moves};
use crate::types::{Bound, Move, Position, MoveList, Score, SearchState, Window};

pub fn start_search(position: &Position, max_depth: u8, end_time: Instant, search_state: &mut SearchState) -> Move {

    let legal_moves: MoveList = moves(position).into_iter().filter(|m| {
        let mut new_position = *position;
        make_move(position, *m, &mut new_position);
        !is_check(&new_position, position.mover);
        true
    }).collect();

    let mut rng = rand::thread_rng();
    legal_moves[rng.gen_range(0..legal_moves.len())]
}


pub fn search_zero(position: &Position, depth: u8, tx: Sender<String>) {
    let aspiration_window: Window = (-30000, 30000);

    for iterative_depth in 1..200 {
        for mv in moves(position) {
            let mut new_position = *position;
            make_move(position, mv, &mut new_position);
            search(&new_position, depth, aspiration_window, &tx);
        }
    }
}

pub fn search(position: &Position, depth: u8, window: Window, _tx: &Sender<String>) -> Score {

    return 0;
}

pub fn is_book_move_available(position: &Position) -> bool {
    false
}