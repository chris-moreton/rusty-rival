use std::cmp::{min};
use crate::engine_constants::{BISHOP_VALUE, KNIGHT_VALUE, PAWN_VALUE, QUEEN_VALUE, ROOK_VALUE};
use crate::make_move::make_move;
use crate::moves::{is_check, quiesce_moves};
use crate::types::{Move, Position, Score};
use crate::utils::{captured_piece_value, moving_piece_mask, to_square_part};
use crate::move_constants::{PIECE_MASK_PAWN, PIECE_MASK_QUEEN, PIECE_MASK_ROOK, PIECE_MASK_BISHOP, PIECE_MASK_KNIGHT, PIECE_MASK_KING};

#[inline(always)]
pub fn static_exchange_evaluation(position: &Position, mv: Move) -> Score {

    let capture_square = to_square_part(mv);
    let v = captured_piece_value(position, mv);
    let mut new_position = *position;
    make_move(position, mv, &mut new_position);
    if is_check(&new_position, position.mover) {
        return 0;
    }

    let moves: Vec<Move> = quiesce_moves(&new_position).into_iter().filter(|m| { to_square_part(*m) == capture_square }).collect();

    let mut legal_move_count = 0;
    let mut least_valuable_attacker = 30000;
    let mut least_valuable_move = 0;
    for m in moves {
        let av = attacker_value(&new_position, m);
        if av < least_valuable_attacker {
            let mut new_new_position = new_position.clone();
            make_move(&new_position, m, &mut new_new_position);
            if !is_check(&new_new_position, new_position.mover) {
                legal_move_count += 1;
                least_valuable_attacker = av;
                least_valuable_move = m;
            }
        }
    }

    if legal_move_count == 0 {
        return v;
    }

    let see = -static_exchange_evaluation(&new_position, least_valuable_move);
    min(v, v + see)
}

fn attacker_value(position: &Position, m: Move) -> Score {
    match moving_piece_mask(position, m) {
        PIECE_MASK_PAWN => {
            PAWN_VALUE
        },
        PIECE_MASK_KING => {
            10000
        },
        PIECE_MASK_QUEEN => {
            QUEEN_VALUE
        },
        PIECE_MASK_ROOK => {
            ROOK_VALUE
        },
        PIECE_MASK_KNIGHT => {
            KNIGHT_VALUE
        },
        PIECE_MASK_BISHOP => {
            BISHOP_VALUE
        },
        _ => {
            panic!("Unable to determine moving piece.")
        }
    }
}