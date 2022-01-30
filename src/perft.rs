use std::mem;
use crate::make_move::{make_move};
use crate::moves::{is_check, moves};
use crate::opponent;
use crate::types::{BLACK, Position, WHITE};
use crate::utils::to_square_part;

pub unsafe fn perft(position: &Position, depth: u8) -> u64 {

    pub unsafe fn perft(position: &Position, depth: u8) -> u64 {
        let mut count = 0;
        let mover = position.mover;
        let king_square = position.pieces[opponent!(mover) as usize].king_square;

        for m in moves(position) {
            let mut new_position = mem::MaybeUninit::<Position>::uninit();
            if to_square_part(m) == king_square {
                return 9999
            }
            make_move(position, m, &mut *new_position.as_mut_ptr());
            count += if depth == 0 {
                if is_check(&*new_position.as_ptr(), mover) { 0 } else { 1 }
            } else {
                let a = perft(&*new_position.as_ptr(), depth - 1);
                if a == 9999 { 0 } else { a }
            }
        };

        count
    }

    perft(position, depth)
}