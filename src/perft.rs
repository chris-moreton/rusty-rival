use crate::make_move::make_move::{make_move, switch_side, unmake_move};
use crate::moves::moves::{is_check, moves};
use crate::types::types::{MagicBox, Position, PositionHistory};

pub fn perft(position: &mut Position, depth: u8, history: &mut PositionHistory, magic_box: &MagicBox) -> u64 {

    let mut count = 0;

    moves(&position, magic_box).into_iter().for_each(|m| {
        make_move(position, m, history);
        if !is_check(&position, &switch_side(position.mover), magic_box) {
            count += if depth == 0 {
                1
            } else {
                perft(position, depth-1, history, magic_box)
            }
        }
        unmake_move(position, history)
    });

    count

}