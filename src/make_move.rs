use crate::bitboards::{A1_BIT, A8_BIT, bit, C1_BIT, C8_BIT, clear_bit, D1_BIT, E1_BIT, E8_BIT, F1_BIT, G1_BIT, G8_BIT, H1_BIT, H8_BIT, test_bit};
use crate::move_constants::*;
use crate::{move_mover, opponent};
use crate::types::{BLACK, is_any_black_castle_available, is_any_white_castle_available, Move, Piece, Position, Square, WHITE};
use crate::types::Piece::{Bishop, Empty, Knight, Queen, Rook};
use crate::utils::{from_square_part, to_square_part};

pub fn make_move(position: &Position, mv: Move, new_position: &mut Position) {

    *new_position = *position;

    let from = from_square_part(mv);
    let to = to_square_part(mv);
    let to_mask = bit(to);

    let piece_mask = mv & PIECE_MASK_FULL;

    if (position.pieces[WHITE as usize].all_pieces_bitboard | position.pieces[BLACK as usize].all_pieces_bitboard) & to_mask != 0 ||
        (piece_mask == PIECE_MASK_PAWN && ((from - to) % 8 != 0 || PROMOTION_SQUARES & to_mask != 0)) ||
        (piece_mask == PIECE_MASK_KING && KING_START_POSITIONS & bit(from) != 0) {

        let promoted_piece = promotion_piece_from_move(mv);
        if promoted_piece != Empty {
            make_move_with_promotion(new_position, from, to, promoted_piece);
        } else if (from == E1_BIT && (to == G1_BIT || to == C1_BIT) && is_any_white_castle_available(position)) ||
            (from == E8_BIT && (to == G8_BIT || to == C8_BIT) && is_any_black_castle_available(position)) {
            make_castle_move(new_position, to);
        } else {
            make_capture_or_king_move_when_castles_available(new_position, from, to, piece_mask)
        }
        new_position.en_passant_square = EN_PASSANT_NOT_AVAILABLE;

    } else {
        make_simple_move(new_position, from, to, piece_mask)
    };

    new_position.move_number += if position.mover == WHITE { 0 } else { 1 };
    new_position.mover = opponent!(position.mover);
}

fn make_simple_move(position: &mut Position, from: Square, to: Square, piece_mask: Move) {

    let friendly = unsafe {
        &mut position.pieces.get_unchecked_mut(position.mover as usize)
    };

    friendly.all_pieces_bitboard ^= bit(from) | bit(to);
    position.en_passant_square = EN_PASSANT_NOT_AVAILABLE;

    if piece_mask == PIECE_MASK_PAWN {
        friendly.pawn_bitboard = clear_bit(friendly.pawn_bitboard, from) | bit(to);

        if from ^ to == 16 {
            position.en_passant_square = from + if position.mover == WHITE { 8 } else { -8 }
        }

        position.half_moves = 0;
    } else {
        position.half_moves += 1;
        match piece_mask {
            PIECE_MASK_KNIGHT => friendly.knight_bitboard = clear_bit(friendly.knight_bitboard, from) | bit(to),
            PIECE_MASK_BISHOP => friendly.bishop_bitboard = clear_bit(friendly.bishop_bitboard, from) | bit(to),
            PIECE_MASK_ROOK => {
                friendly.rook_bitboard = clear_bit(friendly.rook_bitboard, from) | bit(to);
                if from == H1_BIT { position.castle_flags &= !WK_CASTLE }
                if from == A1_BIT { position.castle_flags &= !WQ_CASTLE }
                if from == H8_BIT { position.castle_flags &= !BK_CASTLE }
                if from == A8_BIT { position.castle_flags &= !BQ_CASTLE }
            },
            PIECE_MASK_QUEEN => friendly.queen_bitboard = clear_bit(friendly.queen_bitboard, from) | bit(to),
            PIECE_MASK_KING => friendly.king_square = to,
            _ => panic!("Piece panic")
        }
    }
}

#[inline(always)]
pub fn make_castle_move(position: &mut Position, to: Square) {
    let offset = if position.mover == WHITE { 0 } else { 56 };
    let friendly = unsafe {
        &mut position.pieces.get_unchecked_mut(position.mover as usize)
    };

    friendly.rook_bitboard = if to == C1_BIT + offset {
        friendly.king_square = C1_BIT + offset;

        friendly.all_pieces_bitboard ^= if position.mover == WHITE {
            position.castle_flags &= !(WK_CASTLE | WQ_CASTLE);
            0b0000000000000000000000000000000000000000000000000000000010111000
        } else {
            position.castle_flags &= !(BK_CASTLE | BQ_CASTLE);
            0b1011100000000000000000000000000000000000000000000000000000000000
        };

        clear_bit(friendly.rook_bitboard, A1_BIT + offset) | bit(D1_BIT + offset)
    } else {
        friendly.king_square = G1_BIT + offset;
        friendly.all_pieces_bitboard ^= if position.mover == WHITE {
            position.castle_flags &= !(WK_CASTLE | WQ_CASTLE);
            0b0000000000000000000000000000000000000000000000000000000000001111
        } else {
            position.castle_flags &= !(BK_CASTLE | BQ_CASTLE);
            0b0000111100000000000000000000000000000000000000000000000000000000
        };

        clear_bit(friendly.rook_bitboard, H1_BIT + offset) | bit(F1_BIT + offset)
    };

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

    let to_mask = bit(to);
    let from_mask = bit(from);
    let all_pieces = unsafe {
        position.pieces.get_unchecked(WHITE as usize).all_pieces_bitboard | position.pieces.get_unchecked(BLACK as usize).all_pieces_bitboard
    };

    let enemy = unsafe {
        &mut position.pieces.get_unchecked_mut(opponent!(position.mover) as usize)
    };

    if position.en_passant_square == to && piece == PIECE_MASK_PAWN {
        let pawn_off = !bit(en_passant_captured_piece_square(to));
        enemy.pawn_bitboard &= pawn_off;
        enemy.all_pieces_bitboard &= pawn_off;
    } else if all_pieces & to_mask != 0 {
        let to_mask_inverted = !to_mask;
        enemy.pawn_bitboard &= to_mask_inverted;
        enemy.knight_bitboard &= to_mask_inverted;
        enemy.bishop_bitboard &= to_mask_inverted;
        enemy.rook_bitboard &= to_mask_inverted;
        enemy.queen_bitboard &= to_mask_inverted;
        enemy.all_pieces_bitboard &= to_mask_inverted;
    }

    let friendly = &mut position.pieces[position.mover as usize];

    match piece {
        PIECE_MASK_PAWN => move_mover!(friendly.pawn_bitboard, from_mask, to_mask, friendly),
        PIECE_MASK_KNIGHT => move_mover!(friendly.knight_bitboard, from_mask, to_mask, friendly),
        PIECE_MASK_BISHOP => move_mover!(friendly.bishop_bitboard, from_mask, to_mask, friendly),
        PIECE_MASK_ROOK => move_mover!(friendly.rook_bitboard, from_mask, to_mask, friendly),
        PIECE_MASK_QUEEN => move_mover!(friendly.queen_bitboard, from_mask, to_mask, friendly),
        _ => {
            friendly.king_square = to;
            friendly.all_pieces_bitboard ^= from_mask | to_mask;
        }
    }
    position.half_moves = if all_pieces & to_mask != 0 || piece == PIECE_MASK_PAWN { 0 } else { position.half_moves + 1 };

    if from == E1_BIT || from == H1_BIT || to == H1_BIT { position.castle_flags &= !WK_CASTLE }
    if from == E1_BIT || from == A1_BIT || to == A1_BIT { position.castle_flags &= !WQ_CASTLE }
    if from == E8_BIT || from == H8_BIT || to == H8_BIT { position.castle_flags &= !BK_CASTLE }
    if from == E8_BIT || from == A8_BIT || to == A8_BIT { position.castle_flags &= !BQ_CASTLE }

}

#[inline(always)]
pub fn en_passant_captured_piece_square(square: Square) -> Square {
    square + if square < 40 {  8 } else { -8 }
}

