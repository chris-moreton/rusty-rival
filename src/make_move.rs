use crate::bitboards::{A1_BIT, A8_BIT, bit, H1_BIT, H8_BIT, test_bit};
use crate::move_constants::*;
use crate::{opponent};
use crate::types::{Bitboard, BLACK, Move, Position, Square, WHITE};
use crate::utils::{from_square_part, to_square_part};

#[inline(always)]
pub fn make_move(position: &Position, mv: Move, new_position: &mut Position) {

    *new_position = *position;

    let from = from_square_part(mv);
    let to = to_square_part(mv);

    let piece_mask = mv & PIECE_MASK_FULL;

    match piece_mask {
        PIECE_MASK_PAWN => {
            if mv & PROMOTION_FULL_MOVE_MASK != 0 {
                make_move_with_promotion(new_position, from, to, mv & PROMOTION_FULL_MOVE_MASK);
            } else if (from - to) % 8 != 0 {
                make_pawn_capture_move(new_position, from, to);
            } else {
                make_simple_pawn_move(new_position, from, to)
            }
            new_position.move_number += position.mover as u16;
            new_position.mover ^= 1;
        },
        PIECE_MASK_KING => {
            if mv >= BLACK_QUEEN_CASTLE_MOVE_MASK {
                make_castle_move(new_position, mv)
            } else {
                make_king_move(new_position, from, to);
            }
            new_position.en_passant_square = EN_PASSANT_NOT_AVAILABLE;
        },
        _ => {
            if test_bit(all_pieces(position), to) {
                make_non_pawn_non_king_capture_move(new_position, from, to, piece_mask);
            } else {
                make_non_pawn_non_king_non_capture_move(new_position, from, to, piece_mask);
            };
            new_position.en_passant_square = EN_PASSANT_NOT_AVAILABLE;
            new_position.move_number += position.mover as u16;
            new_position.mover ^= 1;
        }
    }

}

#[inline(always)]
fn make_castle_move(position: &mut Position, mv: Move) {
    match mv {
        WHITE_KING_CASTLE_MOVE => { perform_castle(position, CASTLE_INDEX_WHITE_KING); },
        WHITE_QUEEN_CASTLE_MOVE => { perform_castle(position, CASTLE_INDEX_WHITE_QUEEN); },
        BLACK_KING_CASTLE_MOVE => { perform_castle(position, CASTLE_INDEX_BLACK_KING); },
        BLACK_QUEEN_CASTLE_MOVE => { perform_castle(position, CASTLE_INDEX_BLACK_QUEEN); },
        _ => { panic!("Was expecting a castle move"); }
    };
}

#[inline(always)]
pub fn perform_castle(position: &mut Position, index: usize) {
    position.castle_flags &= CASTLE_VARS_CLEAR_FLAGS_MASK[index];
    position.pieces[position.mover as usize].rook_bitboard ^= CASTLE_VARS_ROOK_MASK[index];
    position.pieces[position.mover as usize].all_pieces_bitboard ^= CASTLE_VARS_ALL_PIECES_MASK[index];
    position.pieces[position.mover as usize].king_square = CASTLE_VARS_KING_TO[index];
    position.mover = CASTLE_VARS_NEW_MOVER[index];
    position.half_moves += 1;
    position.move_number += CASTLE_VARS_FULL_MOVE_INC[index];
}

#[inline(always)]
fn make_simple_pawn_move(position: &mut Position, from: Square, to: Square) {

    let switch = bit(from) | bit(to);
    position.pieces[position.mover as usize].pawn_bitboard ^= switch;
    position.pieces[position.mover as usize].all_pieces_bitboard ^= switch;

    position.en_passant_square = if from ^ to == 16 {
        from + if position.mover == WHITE { 8 } else { -8 }
    } else {
        EN_PASSANT_NOT_AVAILABLE
    };

    position.half_moves = 0;

}

#[inline(always)]
fn make_non_pawn_non_king_non_capture_move(position: &mut Position, from: Square, to: Square, piece_mask: Move) {

    let switch = bit(from) | bit(to);
    position.pieces[position.mover as usize].all_pieces_bitboard ^= switch;

    position.half_moves += 1;
    match piece_mask {
        PIECE_MASK_KNIGHT => position.pieces[position.mover as usize].knight_bitboard ^= switch,
        PIECE_MASK_BISHOP => position.pieces[position.mover as usize].bishop_bitboard ^= switch,
        PIECE_MASK_ROOK => {
            position.pieces[position.mover as usize].rook_bitboard ^= switch;
            update_castle_flags_if_square(position, from);
        },
        PIECE_MASK_QUEEN => position.pieces[position.mover as usize].queen_bitboard ^= switch,
        _ => panic!("Piece panic")
    }
}

#[inline(always)]
pub fn make_move_with_promotion(position: &mut Position, from: Square, to: Square, promotion_mask: Move) {

    let bit_to = bit(to);
    let bit_from = bit(from);

    let is_capture = position.pieces[opponent!(position.mover) as usize].all_pieces_bitboard & bit_to != 0;

    match promotion_mask {
        PROMOTION_KNIGHT_MOVE_MASK => position.pieces[position.mover as usize].knight_bitboard |= bit_to,
        PROMOTION_BISHOP_MOVE_MASK => position.pieces[position.mover as usize].bishop_bitboard |= bit_to,
        PROMOTION_ROOK_MOVE_MASK => position.pieces[position.mover as usize].rook_bitboard |= bit_to,
        PROMOTION_QUEEN_MOVE_MASK => position.pieces[position.mover as usize].queen_bitboard |= bit_to,
        _ => panic!("Invalid promotion piece")
    }

    position.pieces[position.mover as usize].pawn_bitboard ^= bit_from;
    position.pieces[position.mover as usize].all_pieces_bitboard ^= bit_from | bit_to;

    if is_capture {
        let enemy = &mut position.pieces[opponent!(position.mover) as usize];
        let not_bit_to = !bit_to;
        enemy.knight_bitboard &= not_bit_to;
        enemy.rook_bitboard &= not_bit_to;
        enemy.bishop_bitboard &= not_bit_to;
        enemy.queen_bitboard &= not_bit_to;
        enemy.all_pieces_bitboard &= not_bit_to;
    }

    update_castle_flags_if_square(position, to);

    position.en_passant_square = EN_PASSANT_NOT_AVAILABLE;
    position.half_moves = 0;
}

#[inline(always)]
fn update_castle_flags_if_square(position: &mut Position, sq: Square) {
    if position.castle_flags != 0 {
        match sq {
            H8_BIT => { position.castle_flags &= !BK_CASTLE },
            A8_BIT => { position.castle_flags &= !BQ_CASTLE },
            H1_BIT => { position.castle_flags &= !WK_CASTLE },
            A1_BIT => { position.castle_flags &= !WQ_CASTLE },
            _ => {}
        }
    }
}

#[inline(always)]
pub fn make_non_pawn_non_king_capture_move(position: &mut Position, from: Square, to: Square, piece: Move) {

    let to_mask = bit(to);

    remove_captured_piece(position, to_mask);

    let switch = bit(from) | to_mask;
    position.pieces[position.mover as usize].all_pieces_bitboard ^= switch;

    match piece {
        PIECE_MASK_KNIGHT => position.pieces[position.mover as usize].knight_bitboard ^= switch,
        PIECE_MASK_BISHOP => position.pieces[position.mover as usize].bishop_bitboard ^= switch,
        PIECE_MASK_ROOK => {
            update_castle_flags_if_square(position, from);
            position.pieces[position.mover as usize].rook_bitboard ^= switch
        },
        PIECE_MASK_QUEEN => position.pieces[position.mover as usize].queen_bitboard ^= switch,
        _ => panic!("Unexpected piece")
    }

    update_castle_flags_if_square(position, to);
}

#[inline(always)]
pub fn make_pawn_capture_move(position: &mut Position, from: Square, to: Square) {

    let enemy = &mut position.pieces[opponent!(position.mover) as usize];

    position.half_moves = 0;

    let to_mask = bit(to);

    if position.en_passant_square == to {
        let pawn_off = EN_PASSANT_CAPTURE_MASK[to as usize];
        enemy.pawn_bitboard &= pawn_off;
        enemy.all_pieces_bitboard &= pawn_off;
    } else {
        let to_mask_inverted = !to_mask;
        enemy.pawn_bitboard &= to_mask_inverted;
        enemy.knight_bitboard &= to_mask_inverted;
        enemy.bishop_bitboard &= to_mask_inverted;
        enemy.rook_bitboard &= to_mask_inverted;
        enemy.queen_bitboard &= to_mask_inverted;
        enemy.all_pieces_bitboard &= to_mask_inverted;
    }

    let switch = bit(from) | to_mask;
    position.pieces[position.mover as usize].all_pieces_bitboard ^= switch;
    position.pieces[position.mover as usize].pawn_bitboard ^= switch;

    position.en_passant_square = EN_PASSANT_NOT_AVAILABLE;

    update_castle_flags_if_square(position, to);

}

#[inline(always)]
pub fn make_king_move(position: &mut Position, from: Square, to: Square) {

    let to_mask = bit(to);

    remove_captured_piece(position, to_mask);

    let friendly = &mut position.pieces[position.mover as usize];

    let switch = bit(from) | to_mask;
    friendly.all_pieces_bitboard ^= switch;

    position.castle_flags &= CLEAR_CASTLE_FLAGS_MASK[position.mover as usize];
    friendly.king_square = to;

    position.move_number += position.mover as u16;
    position.mover ^= 1;

}

#[inline(always)]
fn remove_captured_piece(position: &mut Position, to_mask: Bitboard) {
    if (position.pieces[WHITE as usize].all_pieces_bitboard | position.pieces[BLACK as usize].all_pieces_bitboard) & to_mask != 0 {
        let enemy = &mut position.pieces[opponent!(position.mover) as usize];
        position.half_moves = 0;
        let to_mask_inverted = !to_mask;
        enemy.pawn_bitboard &= to_mask_inverted;
        enemy.knight_bitboard &= to_mask_inverted;
        enemy.bishop_bitboard &= to_mask_inverted;
        enemy.rook_bitboard &= to_mask_inverted;
        enemy.queen_bitboard &= to_mask_inverted;
        enemy.all_pieces_bitboard &= to_mask_inverted;
    } else {
        position.half_moves += 1;
    }
}

#[inline(always)]
pub fn en_passant_captured_piece_square(square: Square) -> Square {
    square + if square < 40 { 8 } else { -8 }
}

#[inline(always)]
pub fn all_pieces(position: &Position) -> Bitboard {
    position.pieces[WHITE as usize].all_pieces_bitboard | position.pieces[BLACK as usize].all_pieces_bitboard
}
