use std::mem;
use crate::make_move::{make_move};
use crate::moves::{is_check, moves};
use crate::types::{Position};

pub fn perft(position: &Position, depth: u8) -> u64 {
    let mut count = 0;
    let mover = position.mover;

    for m in moves(position) {
        unsafe {
            let mut new_position = mem::MaybeUninit::<Position>::uninit();
            make_move(position, m, &mut *new_position.as_mut_ptr());
            if !is_check(&*new_position.as_ptr(), mover) {
                count += if depth == 0 {
                    1
                } else {
                    perft(&*new_position.as_ptr(), depth - 1)
                }
            }
        }
    };

    count
}
