use crate::bitboards::bit;
use crate::make_move::make_see_move;
use crate::moves::{is_check, see_moves};
use crate::types::{Bitboard, Move, Position, Score};
use crate::utils::{captured_piece_value, to_square_part};
use std::cmp::min;

#[inline(always)]
pub fn static_exchange_evaluation(position: &Position, mv: Move) -> Score {
    let mut new_position = *position;
    make_see_move(mv, &mut new_position);
    if is_check(&new_position, position.mover) {
        0
    } else {
        see(captured_piece_value(position, mv), bit(to_square_part(mv)), &new_position)
    }
}

#[inline(always)]
fn see(v: Score, capture_square: Bitboard, position: &Position) -> Score {
    for m in see_moves(position, capture_square) {
        let mut new_position = *position;
        make_see_move(m, &mut new_position);
        if !is_check(&new_position, position.mover) {
            return min(v, v - see(captured_piece_value(position, m), capture_square, &new_position));
        }
    }

    v
}
