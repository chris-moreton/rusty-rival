use crate::bitboards::{A1_BIT, A8_BIT, bit, C1_BIT, C8_BIT, E1_BIT, E8_BIT, G1_BIT, G8_BIT, H1_BIT, H8_BIT, test_bit};
use crate::move_constants::*;
use crate::{opponent};
use crate::types::{BLACK, Move, Piece, Position, Square, WHITE};
use crate::types::Piece::{Bishop, Empty, Knight, Queen, Rook};
use crate::utils::{from_square_part, to_square_part};

#[inline(always)]
pub fn make_move(position: &Position, mv: Move, new_position: &mut Position) {

    *new_position = *position;

    let from = from_square_part(mv);
    let to = to_square_part(mv);
    let to_mask = bit(to);

    let piece_mask = mv & PIECE_MASK_FULL;

    unsafe {

        match piece_mask {
            PIECE_MASK_PAWN => {
                if mv & PROMOTION_FULL_MOVE_MASK != 0 {
                    make_move_with_promotion(new_position, from, to, promotion_piece_from_move(mv));
                    new_position.en_passant_square = EN_PASSANT_NOT_AVAILABLE;
                } else if (position.pieces.get_unchecked(WHITE as usize).all_pieces_bitboard | position.pieces.get_unchecked(BLACK as usize).all_pieces_bitboard) & to_mask != 0 || (piece_mask == PIECE_MASK_PAWN && (from - to) % 8 != 0) {
                    make_capture_or_king_move_when_castles_available(new_position, from, to, piece_mask);
                    new_position.en_passant_square = EN_PASSANT_NOT_AVAILABLE;
                } else {
                    make_simple_pawn_move(new_position, from, to)
                }
            },
            PIECE_MASK_KING => {
                if from == E1_BIT && (to == G1_BIT || to == C1_BIT) || from == E8_BIT && (to == G8_BIT || to == C8_BIT) {
                    make_castle_move(new_position, to);
                } else {
                    make_capture_or_king_move_when_castles_available(new_position, from, to, piece_mask)
                }
                new_position.en_passant_square = EN_PASSANT_NOT_AVAILABLE;
            },
            _ => {
                if (position.pieces.get_unchecked(WHITE as usize).all_pieces_bitboard | position.pieces.get_unchecked(BLACK as usize).all_pieces_bitboard) & to_mask != 0 {
                    make_capture_or_king_move_when_castles_available(new_position, from, to, piece_mask);
                } else {
                    make_simple_non_pawn_move(new_position, from, to, piece_mask);
                };
                new_position.en_passant_square = EN_PASSANT_NOT_AVAILABLE;
            }
        }
    }

    new_position.move_number += if position.mover == WHITE { 0 } else { 1 };
    new_position.mover = opponent!(position.mover);
}

#[inline(always)]
fn make_simple_pawn_move(position: &mut Position, from: Square, to: Square) {

    let friendly = unsafe {
        &mut position.pieces.get_unchecked_mut(position.mover as usize)
    };

    let switch = bit(from) | bit(to);
    friendly.pawn_bitboard ^= switch;
    friendly.all_pieces_bitboard ^= switch;

    position.en_passant_square = if from ^ to == 16 {
        from + if position.mover == WHITE { 8 } else { -8 }
    } else {
        EN_PASSANT_NOT_AVAILABLE
    };

    position.half_moves = 0;

}

#[inline(always)]
fn make_simple_non_pawn_move(position: &mut Position, from: Square, to: Square, piece_mask: Move) {

    let friendly = unsafe {
        &mut position.pieces.get_unchecked_mut(position.mover as usize)
    };

    let switch = bit(from) | bit(to);
    friendly.all_pieces_bitboard ^= switch;

    position.half_moves += 1;
    match piece_mask {
        PIECE_MASK_KNIGHT => friendly.knight_bitboard ^= switch,
        PIECE_MASK_BISHOP => friendly.bishop_bitboard ^= switch,
        PIECE_MASK_ROOK => {
            friendly.rook_bitboard ^= switch;
            if position.castle_flags != 0 {
                if from == H1_BIT { position.castle_flags &= !WK_CASTLE }
                if from == A1_BIT { position.castle_flags &= !WQ_CASTLE }
                if from == H8_BIT { position.castle_flags &= !BK_CASTLE }
                if from == A8_BIT { position.castle_flags &= !BQ_CASTLE }
            }
        },
        PIECE_MASK_QUEEN => friendly.queen_bitboard ^= switch,
        PIECE_MASK_KING => friendly.king_square = to,
        _ => panic!("Piece panic")
    }
}

#[inline(always)]
pub fn make_castle_move(position: &mut Position, to: Square) {
    let friendly = unsafe {
        &mut position.pieces.get_unchecked_mut(position.mover as usize)
    };

    friendly.king_square = to;

    if position.mover == WHITE {
        position.castle_flags &= !(WK_CASTLE | WQ_CASTLE);
        if to == C1_BIT {
            friendly.rook_bitboard ^= 0b0000000000000000000000000000000000000000000000000000000010010000;
            friendly.all_pieces_bitboard ^= 0b0000000000000000000000000000000000000000000000000000000010111000
        } else {
            friendly.rook_bitboard ^= 0b0000000000000000000000000000000000000000000000000000000000000101;
            friendly.all_pieces_bitboard ^= 0b0000000000000000000000000000000000000000000000000000000000001111
        };
    } else {
        position.castle_flags &= !(BK_CASTLE | BQ_CASTLE);
        if to == C8_BIT {
            friendly.rook_bitboard ^= 0b1001000000000000000000000000000000000000000000000000000000000000;
            friendly.all_pieces_bitboard ^= 0b1011100000000000000000000000000000000000000000000000000000000000
        } else {
            friendly.rook_bitboard ^= 0b0000010100000000000000000000000000000000000000000000000000000000;
            friendly.all_pieces_bitboard ^= 0b0000111100000000000000000000000000000000000000000000000000000000
        };
    }

    position.half_moves += 1;
}

#[inline(always)]
pub fn promotion_piece_from_move(mv: Move) -> Piece {
    match PROMOTION_FULL_MOVE_MASK & mv {
        PROMOTION_QUEEN_MOVE_MASK => Queen,
        PROMOTION_ROOK_MOVE_MASK => Rook,
        PROMOTION_BISHOP_MOVE_MASK => Bishop,
        PROMOTION_KNIGHT_MOVE_MASK => Knight,
        _ => Empty
    }
}

#[inline(always)]
pub fn is_promotion_square(square: Square) -> bool {
    test_bit(PROMOTION_SQUARES, square)
}

#[inline(always)]
pub fn make_move_with_promotion(position: &mut Position, from: Square, to: Square, promotion_piece: Piece) {

    let bit_to = bit(to);
    let bit_from = bit(from);

    let is_capture = unsafe {
        position.pieces.get_unchecked(opponent!(position.mover) as usize).all_pieces_bitboard & bit_to != 0
    };

    let friendly = unsafe {
        &mut position.pieces.get_unchecked_mut(position.mover as usize)
    };

    match promotion_piece {
        Knight => friendly.knight_bitboard |= bit_to,
        Bishop => friendly.bishop_bitboard |= bit_to,
        Rook => friendly.rook_bitboard |= bit_to,
        Queen => friendly.queen_bitboard |= bit_to,
        _ => panic!("Invalid promotion piece")
    }

    friendly.pawn_bitboard ^= bit_from;
    friendly.all_pieces_bitboard ^= bit_from | bit_to;

    if is_capture {
        let enemy = unsafe {
            &mut position.pieces.get_unchecked_mut(opponent!(position.mover) as usize)
        };
        let not_bit_to = !bit_to;
        enemy.knight_bitboard &= not_bit_to;
        enemy.rook_bitboard &= not_bit_to;
        enemy.bishop_bitboard &= not_bit_to;
        enemy.queen_bitboard &= not_bit_to;
        enemy.all_pieces_bitboard &= not_bit_to;
    }

    if to == H8_BIT { position.castle_flags &= !BK_CASTLE }
    if to == A8_BIT { position.castle_flags &= !BQ_CASTLE }
    if to == H1_BIT { position.castle_flags &= !WK_CASTLE }
    if to == A1_BIT { position.castle_flags &= !WQ_CASTLE }

    position.half_moves = 0;
}

#[inline(always)]
pub fn make_capture_or_king_move_when_castles_available(position: &mut Position, from: Square, to: Square, piece: Move) {

    let all_pieces = unsafe {
        position.pieces.get_unchecked(WHITE as usize).all_pieces_bitboard | position.pieces.get_unchecked(BLACK as usize).all_pieces_bitboard
    };

    let enemy = unsafe {
        &mut position.pieces.get_unchecked_mut(opponent!(position.mover) as usize)
    };

    position.half_moves += 1;

    let to_mask = bit(to);

    if position.en_passant_square == to && piece == PIECE_MASK_PAWN {
        let pawn_off = !bit(en_passant_captured_piece_square(to));
        enemy.pawn_bitboard &= pawn_off;
        enemy.all_pieces_bitboard &= pawn_off;
    } else if all_pieces & to_mask != 0 {
        position.half_moves = 0;
        let to_mask_inverted = !to_mask;
        enemy.pawn_bitboard &= to_mask_inverted;
        enemy.knight_bitboard &= to_mask_inverted;
        enemy.bishop_bitboard &= to_mask_inverted;
        enemy.rook_bitboard &= to_mask_inverted;
        enemy.queen_bitboard &= to_mask_inverted;
        enemy.all_pieces_bitboard &= to_mask_inverted;
    }

    let friendly = unsafe {
        &mut position.pieces.get_unchecked_mut(position.mover as usize)
    };

    let switch = bit(from) | to_mask;
    friendly.all_pieces_bitboard ^= switch;

    match piece {
        PIECE_MASK_PAWN => {
            position.half_moves = 0;
            friendly.pawn_bitboard ^= switch;
        },
        PIECE_MASK_KNIGHT => friendly.knight_bitboard ^= switch,
        PIECE_MASK_BISHOP => friendly.bishop_bitboard ^= switch,
        PIECE_MASK_ROOK => friendly.rook_bitboard ^= switch,
        PIECE_MASK_QUEEN => friendly.queen_bitboard ^= switch,
        _ => {
            if position.mover == WHITE {
                position.castle_flags &= !(WK_CASTLE | WQ_CASTLE)
            } else {
                position.castle_flags &= !(BK_CASTLE | BQ_CASTLE)
            }
            friendly.king_square = to;
        }
    }

    if switch & bit(H1_BIT) != 0 { position.castle_flags &= !WK_CASTLE }
    if switch & bit(A1_BIT) != 0 { position.castle_flags &= !WQ_CASTLE }
    if switch & bit(H8_BIT) != 0 { position.castle_flags &= !BK_CASTLE }
    if switch & bit(A8_BIT) != 0 { position.castle_flags &= !BQ_CASTLE }

}

#[inline(always)]
pub fn en_passant_captured_piece_square(square: Square) -> Square {
    square + if square < 40 { 8 } else { -8 }
}

