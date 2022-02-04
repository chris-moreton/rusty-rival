use std::sync::mpsc::Sender;
use std::time::Instant;
use crate::make_move::make_move;
use crate::moves::moves;
use crate::types::{Bound, Position, Score, SearchState, Window};

pub fn start_search(position: &Position, max_depth: u8, end_time: Instant, search_state: &mut SearchState) {
    let these_moves = moves(position);
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