use std::cmp::{min};
use crate::bitboards::bit;
use crate::make_move::make_move;
use crate::moves::{is_check, see_moves};
use crate::types::{Move, Position, Score};
use crate::utils::{captured_piece_value, to_square_part};

#[inline(always)]
pub fn static_exchange_evaluation(position: &Position, mv: Move) -> Score {

    let capture_square = to_square_part(mv);
    let v = captured_piece_value(position, mv);
    let mut new_position = *position;
    make_move(position, mv, &mut new_position);
    if is_check(&new_position, position.mover) {
        return 0;
    }

    let mut legal_move_count = 0;
    let mut least_valuable_move = 0;
    for m in see_moves(&new_position, bit(capture_square)) {
        let mut new_new_position = new_position.clone();
        make_move(&new_position, m, &mut new_new_position);
        if !is_check(&new_new_position, new_position.mover) {
            legal_move_count += 1;
            least_valuable_move = m;
            break;
        }
    }

    if legal_move_count == 0 {
        return v;
    }

    let see = -static_exchange_evaluation(&new_position, least_valuable_move);
    min(v, v + see)
}
