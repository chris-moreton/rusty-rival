use crate::make_move::{default_position_history, make_move, switch_side};
use crate::moves::{allocate_magic_boxes, is_check, moves};
use crate::types::{MagicBox, Position, PositionHistory};

pub fn perft(position: &mut Position, depth: u8) -> u64 {

    pub fn perft(depth: u8, history: &mut PositionHistory, magic_box: &MagicBox) -> u64 {
        let mut count = 0;

        let position = history.history[history.move_pointer as usize];
        moves(&position, magic_box).into_iter().for_each(|m| {
            make_move(m, history);
            let position = history.history[history.move_pointer as usize];
            if !is_check(&position, switch_side(position.mover), magic_box) {
                count += if depth == 0 {
                    1
                } else {
                    perft(depth-1, history, magic_box)
                }
            }
            history.move_pointer -= 1;
        });

        count

    }

    let mut history = default_position_history();
    let magic_box = allocate_magic_boxes();

    history.history[0] = *position;
    history.move_pointer = 0;

    perft(depth, &mut history, &magic_box)

}