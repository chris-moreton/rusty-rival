use crate::bitboards::{RANK_1_BITS, south_fill};
use crate::engine_constants::{BISHOP_VALUE, DOUBLED_PAWN_PENALTY, KNIGHT_VALUE, PAWN_VALUE, QUEEN_VALUE, ROOK_VALUE};
use crate::piece_square_tables::piece_square_values;
use crate::types::{Bitboard, BLACK, Pieces, Position, Score, WHITE};

#[inline(always)]
pub fn material(pieces: &Pieces) -> Score {
    (pieces.pawn_bitboard.count_ones() as i16 * PAWN_VALUE +
    pieces.knight_bitboard.count_ones() as i16 * KNIGHT_VALUE +
    pieces.rook_bitboard.count_ones() as i16 * ROOK_VALUE +
    pieces.bishop_bitboard.count_ones() as i16 * BISHOP_VALUE +
    pieces.queen_bitboard.count_ones() as i16 * QUEEN_VALUE) as Score
}

#[inline(always)]
pub fn material_score(position: &Position) -> Score {
    ((position.pieces[WHITE as usize].pawn_bitboard.count_ones() - position.pieces[BLACK as usize].pawn_bitboard.count_ones()) as i16 * PAWN_VALUE +
        (position.pieces[WHITE as usize].knight_bitboard.count_ones() - position.pieces[BLACK as usize].knight_bitboard.count_ones()) as i16 * KNIGHT_VALUE +
        (position.pieces[WHITE as usize].rook_bitboard.count_ones() - position.pieces[BLACK as usize].rook_bitboard.count_ones()) as i16 * ROOK_VALUE +
        (position.pieces[WHITE as usize].bishop_bitboard.count_ones() - position.pieces[BLACK as usize].bishop_bitboard.count_ones()) as i16 * BISHOP_VALUE +
        (position.pieces[WHITE as usize].queen_bitboard.count_ones() - position.pieces[BLACK as usize].queen_bitboard.count_ones()) as i16 * QUEEN_VALUE) as Score
}

#[inline(always)]
pub fn doubled_pawn_count(pawn_bitboard: Bitboard) -> Score {
    (pawn_bitboard.count_ones() as u8 - (south_fill(pawn_bitboard) & RANK_1_BITS).count_ones() as u8) as u8 as Score
}

#[inline(always)]
pub fn pawn_score(position: &Position) -> Score {
    ((doubled_pawn_count(position.pieces[BLACK as usize].pawn_bitboard) -
        doubled_pawn_count(position.pieces[WHITE as usize].pawn_bitboard)) as Score
        * DOUBLED_PAWN_PENALTY) as Score
}

#[inline(always)]
pub fn evaluate(position: &Position) -> Score {

    let score =
        material_score(position) +
        pawn_score(position);

    if position.mover == WHITE { score } else { -score }
}
