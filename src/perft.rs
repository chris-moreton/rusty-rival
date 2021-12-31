use crate::make_move::make_move::{make_move};
use crate::moves::moves::{is_check, moves};
use crate::types::types::Position;

pub fn perft(position: &Position, depth: u8) -> u64 {

    let mut count = 0;

    moves(&position).into_iter().for_each(|m| {
        let mut new_position = position.clone();
        make_move(&mut new_position, m);
        if !is_check(&new_position, &position.mover) {
            count += if depth == 0 {
                1
            } else {
                perft(&new_position, depth-1)
            }
        }
    });

    count

}