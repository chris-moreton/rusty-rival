use crate::bitboards::{BISHOP_RAYS, bit, KING_MOVES_BITBOARDS, KNIGHT_MOVES_BITBOARDS, ROOK_RAYS};

use crate::moves::{generate_capture_pawn_moves_with_destinations, generate_diagonal_slider_moves, generate_straight_slider_moves, is_check};
use crate::types::{Bitboard, BLACK, Move, MoveList, Position, Score, Square, WHITE};
use crate::utils::{from_square_mask, from_square_part, to_square_part};
use std::cmp::min;
use crate::engine_constants::{BISHOP_VALUE_AVERAGE, KNIGHT_VALUE_AVERAGE, PAWN_VALUE_AVERAGE, QUEEN_VALUE_AVERAGE, ROOK_VALUE_AVERAGE};


use crate::move_constants::{EN_PASSANT_NOT_AVAILABLE, PIECE_MASK_FULL, PIECE_MASK_PAWN, PIECE_MASK_KING, PIECE_MASK_KNIGHT, PIECE_MASK_QUEEN, PIECE_MASK_BISHOP, PIECE_MASK_ROOK, PROMOTION_FULL_MOVE_MASK, PROMOTION_QUEEN_MOVE_MASK, EN_PASSANT_CAPTURE_MASK};
use crate::{add_moves, get_and_unset_lsb, opponent};


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
            return min(score, score - see(captured_piece_value_see(position, m), capture_square, &new_position));
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

    promote_value + (if tsp == position.en_passant_square || enemy.pawn_bitboard & to_bb != 0 {
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
pub fn see_moves(position: &Position, valid_destinations: Bitboard) -> MoveList {
    let mut move_list = Vec::with_capacity(1);
    let capture_square = valid_destinations.trailing_zeros() as usize;

    let all_pieces = position.pieces[WHITE as usize].all_pieces_bitboard | position.pieces[BLACK as usize].all_pieces_bitboard;
    let friendly = position.pieces[position.mover as usize];

    generate_capture_pawn_moves_with_destinations(&mut move_list, position.mover as usize, friendly.pawn_bitboard, valid_destinations);

    if move_list.is_empty() {
        let mut knights = KNIGHT_MOVES_BITBOARDS[capture_square] & friendly.knight_bitboard;
        while knights != 0 {
            let fsm = from_square_mask(get_and_unset_lsb!(knights)) | PIECE_MASK_KNIGHT;
            move_list.push(fsm | capture_square as Move);
        }
    }

    if move_list.is_empty() && BISHOP_RAYS[capture_square] & friendly.bishop_bitboard != 0 {
        generate_diagonal_slider_moves(
            friendly.bishop_bitboard,
            all_pieces,
            &mut move_list,
            valid_destinations,
            PIECE_MASK_BISHOP,
        );
    }
    if move_list.is_empty() && ROOK_RAYS[capture_square] & friendly.rook_bitboard != 0 {
        generate_straight_slider_moves(
            friendly.rook_bitboard,
            all_pieces,
            &mut move_list,
            valid_destinations,
            PIECE_MASK_ROOK,
        );
    }
    if move_list.is_empty() && ROOK_RAYS[capture_square] & friendly.queen_bitboard != 0 {
        generate_straight_slider_moves(
            friendly.queen_bitboard,
            all_pieces,
            &mut move_list,
            valid_destinations,
            PIECE_MASK_QUEEN,
        );
    }
    if move_list.is_empty() && BISHOP_RAYS[capture_square] & friendly.queen_bitboard != 0 {
        generate_diagonal_slider_moves(
            friendly.queen_bitboard,
            all_pieces,
            &mut move_list,
            valid_destinations,
            PIECE_MASK_QUEEN,
        );
    }

    if move_list.is_empty() {
        add_moves!(
            move_list,
            from_square_mask(friendly.king_square) | PIECE_MASK_KING,
            KING_MOVES_BITBOARDS[friendly.king_square as usize] & valid_destinations
        );
    }

    move_list
}
