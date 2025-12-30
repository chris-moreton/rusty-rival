use crate::bitboards::{bit, BISHOP_RAYS, KING_MOVES_BITBOARDS, KNIGHT_MOVES_BITBOARDS, PAWN_MOVES_CAPTURE, ROOK_RAYS};

use crate::engine_constants::{BISHOP_VALUE_AVERAGE, KNIGHT_VALUE_AVERAGE, PAWN_VALUE_AVERAGE, QUEEN_VALUE_AVERAGE, ROOK_VALUE_AVERAGE};
use crate::moves::is_check;
use crate::types::{Bitboard, Move, MoveList, Position, Score, Square, BLACK, WHITE};
use crate::utils::{from_square_mask, from_square_part, to_square_part};
use std::cmp::min;

use crate::magic_bitboards::{magic_moves_bishop, magic_moves_rook};
use crate::move_constants::{
    EN_PASSANT_CAPTURE_MASK, EN_PASSANT_NOT_AVAILABLE, PIECE_MASK_BISHOP, PIECE_MASK_FULL, PIECE_MASK_KING, PIECE_MASK_KNIGHT,
    PIECE_MASK_PAWN, PIECE_MASK_QUEEN, PIECE_MASK_ROOK, PROMOTION_FULL_MOVE_MASK, PROMOTION_QUEEN_MOVE_MASK, PROMOTION_SQUARES,
};
use crate::{get_and_unset_lsb, get_lsb, opponent};

#[inline(always)]
pub fn static_exchange_evaluation(position: &Position, mv: Move) -> Score {
    let mut new_position = *position;
    make_see_move(mv, &mut new_position);
    see(captured_piece_value_see(position, mv), bit(to_square_part(mv)), &new_position)
}

#[inline(always)]
pub fn see(score: Score, capture_square: Bitboard, position: &Position) -> Score {
    for m in see_moves(position, capture_square) {
        let mut new_position = *position;
        make_see_move(m, &mut new_position);
        if !is_check(&new_position, position.mover) {
            return min(
                score,
                score - see(captured_piece_value_see(position, m), capture_square, &new_position),
            );
        }
    }

    score
}

#[inline(always)]
pub fn make_see_move(mv: Move, new_position: &mut Position) {
    let from = from_square_part(mv);
    let to = to_square_part(mv);

    let piece_mask = mv & PIECE_MASK_FULL;
    let bit_to = bit(to);
    let enemy = &mut new_position.pieces[opponent!(new_position.mover) as usize];
    let switch = bit(from) | bit_to;

    if piece_mask == PIECE_MASK_PAWN && new_position.en_passant_square == to {
        let pawn_off = EN_PASSANT_CAPTURE_MASK[to as usize];

        enemy.pawn_bitboard &= pawn_off;
        enemy.all_pieces_bitboard &= pawn_off;
    } else {
        enemy.pawn_bitboard &= !bit_to;
        enemy.knight_bitboard &= !bit_to;
        enemy.rook_bitboard &= !bit_to;
        enemy.bishop_bitboard &= !bit_to;
        enemy.queen_bitboard &= !bit_to;

        enemy.all_pieces_bitboard &= !bit_to;
    }
    new_position.en_passant_square = EN_PASSANT_NOT_AVAILABLE;

    if mv & PROMOTION_FULL_MOVE_MASK == PROMOTION_QUEEN_MOVE_MASK {
        new_position.pieces[new_position.mover as usize].queen_bitboard |= bit_to;
        new_position.pieces[new_position.mover as usize].pawn_bitboard &= !bit(from);
    } else {
        match piece_mask {
            PIECE_MASK_PAWN => new_position.pieces[new_position.mover as usize].pawn_bitboard ^= switch,
            PIECE_MASK_KNIGHT => new_position.pieces[new_position.mover as usize].knight_bitboard ^= switch,
            PIECE_MASK_BISHOP => new_position.pieces[new_position.mover as usize].bishop_bitboard ^= switch,
            PIECE_MASK_ROOK => new_position.pieces[new_position.mover as usize].rook_bitboard ^= switch,
            PIECE_MASK_QUEEN => new_position.pieces[new_position.mover as usize].queen_bitboard ^= switch,
            PIECE_MASK_KING => new_position.pieces[new_position.mover as usize].king_square = to,
            _ => panic!("Piece panic"),
        }
    }

    new_position.pieces[new_position.mover as usize].all_pieces_bitboard ^= switch;
    new_position.mover ^= 1;
}

#[inline(always)]
pub fn captured_piece_value_see(position: &Position, mv: Move) -> Score {
    let enemy = &position.pieces[opponent!(position.mover) as usize];
    let tsp = to_square_part(mv);
    let to_bb = bit(tsp);

    let promote_value = if mv & PROMOTION_FULL_MOVE_MASK == PROMOTION_QUEEN_MOVE_MASK {
        QUEEN_VALUE_AVERAGE - PAWN_VALUE_AVERAGE
    } else {
        0
    };

    promote_value
        + (if tsp == position.en_passant_square || enemy.pawn_bitboard & to_bb != 0 {
            PAWN_VALUE_AVERAGE
        } else if enemy.knight_bitboard & to_bb != 0 {
            KNIGHT_VALUE_AVERAGE
        } else if enemy.bishop_bitboard & to_bb != 0 {
            BISHOP_VALUE_AVERAGE
        } else if enemy.rook_bitboard & to_bb != 0 {
            ROOK_VALUE_AVERAGE
        } else if enemy.queen_bitboard & to_bb != 0 {
            QUEEN_VALUE_AVERAGE
        } else {
            0
        })
}

#[inline(always)]
pub fn generate_capture_pawn_moves_with_destinations_see(
    move_list: &mut MoveList,
    colour_index: usize,
    mut from_squares: Bitboard,
    valid_destinations: Bitboard,
) {
    while from_squares != 0 {
        let from_square = get_and_unset_lsb!(from_squares);

        let to_bitboard = PAWN_MOVES_CAPTURE[colour_index][from_square as usize] & valid_destinations;

        let fsm = from_square_mask(from_square);
        let is_promotion = to_bitboard & PROMOTION_SQUARES != 0;
        if to_bitboard != 0 {
            let base_move = fsm | get_lsb!(to_bitboard) as Move;
            if is_promotion {
                move_list.push(base_move | PROMOTION_QUEEN_MOVE_MASK);
            } else {
                move_list.push(base_move);
            }
            break;
        }
    }
}

#[inline(always)]
pub fn generate_diagonal_slider_moves_see(
    slider_bitboard: Bitboard,
    all_pieces_bitboard: Bitboard,
    move_list: &mut MoveList,
    valid_destinations: Bitboard,
    piece_mask: Move,
) {
    let capture_square = valid_destinations.trailing_zeros();
    let bb = magic_moves_bishop(capture_square as Square, all_pieces_bitboard) & slider_bitboard;
    if bb != 0 {
        move_list.push(from_square_mask(bb.trailing_zeros() as Square) | piece_mask | capture_square);
    }
}

#[inline(always)]
pub fn generate_straight_slider_moves_see(
    slider_bitboard: Bitboard,
    all_pieces_bitboard: Bitboard,
    move_list: &mut MoveList,
    valid_destinations: Bitboard,
    piece_mask: Move,
) {
    let capture_square = valid_destinations.trailing_zeros();
    let bb = magic_moves_rook(capture_square as Square, all_pieces_bitboard) & slider_bitboard;
    if bb != 0 {
        move_list.push(from_square_mask(bb.trailing_zeros() as Square) | piece_mask | capture_square);
    }
}

#[inline(always)]
pub fn see_moves(position: &Position, valid_destinations: Bitboard) -> MoveList {
    let mut move_list = MoveList::new();
    let capture_square = valid_destinations.trailing_zeros() as usize;

    let all_pieces = position.pieces[WHITE as usize].all_pieces_bitboard | position.pieces[BLACK as usize].all_pieces_bitboard;
    let friendly = position.pieces[position.mover as usize];

    generate_capture_pawn_moves_with_destinations_see(&mut move_list, position.mover as usize, friendly.pawn_bitboard, valid_destinations);

    if move_list.is_empty() {
        let knights = KNIGHT_MOVES_BITBOARDS[capture_square] & friendly.knight_bitboard;
        if knights != 0 {
            let fsm = from_square_mask(knights.trailing_zeros() as Square) | PIECE_MASK_KNIGHT;
            move_list.push(fsm | capture_square as Move);
        }
    }

    if move_list.is_empty() && BISHOP_RAYS[capture_square] & friendly.bishop_bitboard != 0 {
        generate_diagonal_slider_moves_see(
            friendly.bishop_bitboard,
            all_pieces,
            &mut move_list,
            valid_destinations,
            PIECE_MASK_BISHOP,
        );
    }
    if move_list.is_empty() && ROOK_RAYS[capture_square] & friendly.rook_bitboard != 0 {
        generate_straight_slider_moves_see(
            friendly.rook_bitboard,
            all_pieces,
            &mut move_list,
            valid_destinations,
            PIECE_MASK_ROOK,
        );
    }
    if move_list.is_empty() && ROOK_RAYS[capture_square] & friendly.queen_bitboard != 0 {
        generate_straight_slider_moves_see(
            friendly.queen_bitboard,
            all_pieces,
            &mut move_list,
            valid_destinations,
            PIECE_MASK_QUEEN,
        );
    }
    if move_list.is_empty() && BISHOP_RAYS[capture_square] & friendly.queen_bitboard != 0 {
        generate_diagonal_slider_moves_see(
            friendly.queen_bitboard,
            all_pieces,
            &mut move_list,
            valid_destinations,
            PIECE_MASK_QUEEN,
        );
    }

    if move_list.is_empty() {
        let bb = KING_MOVES_BITBOARDS[friendly.king_square as usize] & valid_destinations;
        if bb != 0 {
            move_list.push(from_square_mask(friendly.king_square) | PIECE_MASK_KING | bb.trailing_zeros() as Move);
        }
    }

    move_list
}
