use crate::bitboards::bit;
use crate::engine_constants::{BISHOP_VALUE, KNIGHT_VALUE, PAWN_VALUE, QUEEN_VALUE, ROOK_VALUE};
use crate::move_constants::{PROMOTION_BISHOP_MOVE_MASK, PROMOTION_FULL_MOVE_MASK, PROMOTION_KNIGHT_MOVE_MASK, PROMOTION_ROOK_MOVE_MASK};
use crate::opponent;
use crate::piece_square_tables::score_piece_square_values;
use crate::types::{Move, Piece, Pieces, Position, Score, SearchState, Square};
use crate::types::Piece::{Bishop, King, Knight, Pawn, Queen, Rook};
use crate::utils::{from_square_part, to_square_part};

pub const BIT_FLIPPED_HORIZONTAL_AXIS: [Square; 64] = [
    56, 57, 58, 59, 60, 61, 62, 63, 48, 49, 50, 51, 52, 53, 54, 55, 40, 41, 42, 43, 44, 45, 46, 47, 32, 33, 34, 35, 36, 37, 38, 39, 24, 25, 26, 27, 28, 29, 30, 31, 16, 17, 18, 19, 20, 21, 22, 23, 8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7
];

pub const KNIGHT_STAGE_MATERIAL_LOW: Score = KNIGHT_VALUE + 8 * PAWN_VALUE;
pub const KNIGHT_STAGE_MATERIAL_HIGH: Score = QUEEN_VALUE + 2 * ROOK_VALUE + 2 * BISHOP_VALUE + 6 * PAWN_VALUE;
pub const PAWN_STAGE_MATERIAL_LOW: Score = ROOK_VALUE;
pub const PAWN_STAGE_MATERIAL_HIGH: Score  = QUEEN_VALUE + 2 * ROOK_VALUE + 2 * BISHOP_VALUE;
pub const OPENING_PHASE_MATERIAL: Score = (TOTAL_PIECE_VALUE_PER_SIDE_AT_START as f32 * 0.8) as Score;
pub const TOTAL_PIECE_VALUE_PER_SIDE_AT_START: Score = KNIGHT_VALUE * 2 + BISHOP_VALUE * 2 + ROOK_VALUE * 2 + QUEEN_VALUE;

#[inline(always)]
fn attacker_bonus(piece: Piece) -> Score {
    match piece {
        Pawn => 6,
        Knight => 5,
        Bishop => 4,
        Rook => 3,
        Queen => 2,
        King => 1,
        _ => {
            panic!("Expected piece")
        }
    }
}

#[inline(always)]
pub fn score_move(position: &Position, hash_move: Move, m: Move, search_state: &SearchState, ply: usize) -> Score {
    let enemy = position.pieces[opponent!(position.mover) as usize];
    let to_square = to_square_part(m);

    if m == hash_move {
        10000
    } else if enemy.all_pieces_bitboard & bit(to_square) != 0 {
        piece_value(&enemy, to_square) + attacker_bonus(piece_type(position, from_square_part(m)))
    } else if m & PROMOTION_FULL_MOVE_MASK != 0 {
        let mask = m & PROMOTION_FULL_MOVE_MASK;
        if mask == PROMOTION_ROOK_MOVE_MASK {
            3
        } else if mask == PROMOTION_BISHOP_MOVE_MASK {
            2
        } else if mask == PROMOTION_KNIGHT_MOVE_MASK {
            1
        } else {
            QUEEN_VALUE
        }
    } else if to_square == position.en_passant_square {
        PAWN_VALUE + attacker_bonus(Pawn)
    } else {
        let killer_moves = search_state.killer_moves[ply];
        if m == killer_moves[0] { 75 } else if m == killer_moves[1] { 50 } else if ply > 2 {
            let killer_moves = search_state.killer_moves[ply - 2];
            if m == killer_moves[0] { 65 } else if m == killer_moves[1] { 40 } else { 0 }
        } else {
            0
        }
    }
}

#[inline(always)]
pub fn piece_type(position: &Position, sq: Square) -> Piece {

    let pieces = position.pieces[position.mover as usize];

    let bb = bit(sq);
    if pieces.pawn_bitboard & bb != 0 {
        return Pawn;
    }
    if pieces.knight_bitboard & bb != 0 {
        return Knight;
    }
    if pieces.rook_bitboard & bb != 0 {
        return Rook;
    }
    if pieces.queen_bitboard & bb != 0 {
        return Queen;
    }
    if pieces.bishop_bitboard & bb != 0 {
        return Bishop;
    }
    King
}

#[inline(always)]
pub fn piece_value(pieces: &Pieces, sq: Square) -> Score {
    let bb = bit(sq);
    if pieces.pawn_bitboard & bb != 0 {
        return PAWN_VALUE;
    }
    if pieces.knight_bitboard & bb != 0 {
        return KNIGHT_VALUE;
    }
    if pieces.rook_bitboard & bb != 0 {
        return ROOK_VALUE;
    }
    if pieces.queen_bitboard & bb != 0 {
        return QUEEN_VALUE;
    }
    if pieces.bishop_bitboard & bb != 0 {
        return BISHOP_VALUE;
    }
    0
}
