use crate::bitboards::{A1_BIT, A8_BIT, bit, C1_BIT, C8_BIT, G1_BIT, G8_BIT, H1_BIT, H8_BIT};
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
                make_non_simple_pawn_move(new_position, from, to);
            } else {
                make_simple_pawn_move(new_position, from, to)
            }
            new_position.move_number += position.mover as u16;
            new_position.mover = opponent!(position.mover);
        },
        PIECE_MASK_KING => {
            if mv >= BLACK_QUEEN_CASTLE_MOVE_MASK {
                make_castle_move(mv, new_position)
            } else {
                make_king_move(new_position, from, to);
            }
            new_position.en_passant_square = EN_PASSANT_NOT_AVAILABLE;
        },
        _ => {
            if all_pieces(position) & bit(to) != 0 {
                make_non_pawn_non_king_capture_move(new_position, from, to, piece_mask);
            } else {
                make_simple_non_pawn_non_king_move(new_position, from, to, piece_mask);
            };
            new_position.en_passant_square = EN_PASSANT_NOT_AVAILABLE;
            new_position.move_number += position.mover as u16;
            new_position.mover = opponent!(position.mover);
        }
    }

}

#[inline(always)]
fn make_castle_move(mv: Move, position: &mut Position) {

    match mv {
        WHITE_KING_CASTLE_MOVE => {
            position.castle_flags &= !(WK_CASTLE | WQ_CASTLE);
            position.pieces[WHITE as usize].rook_bitboard ^= 0b0000000000000000000000000000000000000000000000000000000000000101;
            position.pieces[WHITE as usize].all_pieces_bitboard ^= 0b0000000000000000000000000000000000000000000000000000000000001111;
            position.pieces[WHITE as usize].king_square = G1_BIT;
            position.mover = BLACK;
            position.half_moves += 1;
        },
        WHITE_QUEEN_CASTLE_MOVE => {
            position.castle_flags &= !(WK_CASTLE | WQ_CASTLE);
            position.pieces[WHITE as usize].rook_bitboard ^= 0b0000000000000000000000000000000000000000000000000000000010010000;
            position.pieces[WHITE as usize].all_pieces_bitboard ^= 0b0000000000000000000000000000000000000000000000000000000010111000;
            position.pieces[WHITE as usize].king_square = C1_BIT;
            position.mover = BLACK;
            position.half_moves += 1;
        },
        BLACK_KING_CASTLE_MOVE => {
            position.castle_flags &= !(BK_CASTLE | BQ_CASTLE);
            position.pieces[BLACK as usize].rook_bitboard ^= 0b0000010100000000000000000000000000000000000000000000000000000000;
            position.pieces[BLACK as usize].all_pieces_bitboard ^= 0b0000111100000000000000000000000000000000000000000000000000000000;
            position.pieces[BLACK as usize].king_square = G8_BIT;
            position.mover = WHITE;
            position.move_number += 1;
            position.half_moves += 1;
        },
        BLACK_QUEEN_CASTLE_MOVE => {
            position.castle_flags &= !(BK_CASTLE | BQ_CASTLE);
            position.pieces[BLACK as usize].rook_bitboard ^= 0b1001000000000000000000000000000000000000000000000000000000000000;
            position.pieces[BLACK as usize].all_pieces_bitboard ^= 0b1011100000000000000000000000000000000000000000000000000000000000;
            position.pieces[BLACK as usize].king_square = C8_BIT;
            position.mover = WHITE;
            position.move_number += 1;
            position.half_moves += 1;
        },
        _ => {
            panic!("Was expecting a castle move")
        }
    }
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
fn make_simple_non_pawn_non_king_move(position: &mut Position, from: Square, to: Square, piece_mask: Move) {

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
        _ => panic!("Piece panic")
    }
}

#[inline(always)]
pub fn make_move_with_promotion(position: &mut Position, from: Square, to: Square, promotion_mask: Move) {

    let bit_to = bit(to);
    let bit_from = bit(from);

    let is_capture = unsafe {
        position.pieces.get_unchecked(opponent!(position.mover) as usize).all_pieces_bitboard & bit_to != 0
    };

    let friendly = unsafe {
        &mut position.pieces.get_unchecked_mut(position.mover as usize)
    };

    match promotion_mask {
        PROMOTION_KNIGHT_MOVE_MASK => friendly.knight_bitboard |= bit_to,
        PROMOTION_BISHOP_MOVE_MASK => friendly.bishop_bitboard |= bit_to,
        PROMOTION_ROOK_MOVE_MASK => friendly.rook_bitboard |= bit_to,
        PROMOTION_QUEEN_MOVE_MASK => friendly.queen_bitboard |= bit_to,
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

    update_castle_flags_on_capture(position, to);

    position.en_passant_square = EN_PASSANT_NOT_AVAILABLE;

    position.half_moves = 0;
}

#[inline(always)]
fn update_castle_flags_on_capture(position: &mut Position, to: Square) {
    if position.castle_flags != 0 {
        match to {
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

    if unsafe { position.pieces.get_unchecked(WHITE as usize).all_pieces_bitboard | position.pieces.get_unchecked(BLACK as usize).all_pieces_bitboard } & to_mask != 0 {
        let enemy = unsafe {
            &mut position.pieces.get_unchecked_mut(opponent!(position.mover) as usize)
        };
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

    let friendly = unsafe {
        &mut position.pieces.get_unchecked_mut(position.mover as usize)
    };

    let switch = bit(from) | to_mask;
    friendly.all_pieces_bitboard ^= switch;

    match piece {
        PIECE_MASK_KNIGHT => friendly.knight_bitboard ^= switch,
        PIECE_MASK_BISHOP => friendly.bishop_bitboard ^= switch,
        PIECE_MASK_ROOK => {
            if position.castle_flags != 0 {
                match from {
                    H8_BIT => { position.castle_flags &= !BK_CASTLE },
                    A8_BIT => { position.castle_flags &= !BQ_CASTLE },
                    H1_BIT => { position.castle_flags &= !WK_CASTLE },
                    A1_BIT => { position.castle_flags &= !WQ_CASTLE },
                    _ => {}
                }
            }
            friendly.rook_bitboard ^= switch
        },
        PIECE_MASK_QUEEN => friendly.queen_bitboard ^= switch,
        _ => panic!("Unexpected piece")
    }

    update_castle_flags_on_capture(position, to);
}

#[inline(always)]
pub fn make_non_simple_pawn_move(position: &mut Position, from: Square, to: Square) {

    let all_pieces = unsafe {
        position.pieces.get_unchecked(WHITE as usize).all_pieces_bitboard | position.pieces.get_unchecked(BLACK as usize).all_pieces_bitboard
    };

    let enemy = unsafe {
        &mut position.pieces.get_unchecked_mut(opponent!(position.mover) as usize)
    };

    position.half_moves = 0;

    let to_mask = bit(to);

    if position.en_passant_square == to {
        let pawn_off = EN_PASSANT_CAPTURE_MASK[to as usize];
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

    let friendly = unsafe {
        &mut position.pieces.get_unchecked_mut(position.mover as usize)
    };

    let switch = bit(from) | to_mask;
    friendly.all_pieces_bitboard ^= switch;
    friendly.pawn_bitboard ^= switch;

    position.en_passant_square = EN_PASSANT_NOT_AVAILABLE;

    update_castle_flags_on_capture(position, to);

}

#[inline(always)]
pub fn make_king_move(position: &mut Position, from: Square, to: Square) {

    let all_pieces = unsafe {
        position.pieces.get_unchecked(WHITE as usize).all_pieces_bitboard | position.pieces.get_unchecked(BLACK as usize).all_pieces_bitboard
    };

    let enemy = unsafe {
        &mut position.pieces.get_unchecked_mut(opponent!(position.mover) as usize)
    };

    let to_mask = bit(to);

    if all_pieces & to_mask != 0 {
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

    let friendly = unsafe {
        &mut position.pieces.get_unchecked_mut(position.mover as usize)
    };

    let switch = bit(from) | to_mask;
    friendly.all_pieces_bitboard ^= switch;

    if position.mover == WHITE {
        position.castle_flags &= !(WK_CASTLE | WQ_CASTLE)
    } else {
        position.castle_flags &= !(BK_CASTLE | BQ_CASTLE)
    }
    friendly.king_square = to;

    position.move_number += position.mover as u16;
    position.mover = opponent!(position.mover);

}

#[inline(always)]
pub fn en_passant_captured_piece_square(square: Square) -> Square {
    square + if square < 40 { 8 } else { -8 }
}

#[inline(always)]
pub fn all_pieces(position: &Position) -> Bitboard {
    unsafe {
        position.pieces.get_unchecked(WHITE as usize).all_pieces_bitboard | position.pieces.get_unchecked(BLACK as usize).all_pieces_bitboard
    }
}
