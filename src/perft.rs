use crate::make_move::{default_position_history, make_move, unmake_move};
use crate::moves::{is_check, moves};
use crate::types::{Position, PositionHistory};

pub fn perft(position: &mut Position, depth: u8) -> u64 {

    pub fn perft(position: &mut Position, depth: u8, history: &mut PositionHistory) -> u64 {
        let mut count = 0;
        let mover = position.mover;

        moves(position).into_iter().for_each(|m| {
            make_move(position, m, history);
            if !is_check(position, mover) {
                count += if depth == 0 {
                    1
                } else {
                    perft(position, depth - 1, history)
                }
            }
            unmake_move(position, history)
        });

        count
    }

    let mut history = default_position_history();

    perft(position, depth, &mut history)
}