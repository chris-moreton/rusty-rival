use crate::make_move::make_move::{make_move, switch_side, unmake_move};
use crate::moves::moves::{is_check, moves};
use crate::types::types::{Position, PositionHistory};

pub fn perft(position: &mut Position, depth: u8, history: &mut PositionHistory) -> u64 {

    let mut count = 0;

    moves(&position).into_iter().for_each(|m| {
        make_move(position, m, history);
        if !is_check(&position, &switch_side(position.mover)) {
            count += if depth == 0 {
                1
            } else {
                perft(position, depth-1, history)
            }
        }
        unmake_move(position, history)
    });

    count

}