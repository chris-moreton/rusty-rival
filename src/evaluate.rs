use crate::bitboards::{bit, KNIGHT_MOVES_BITBOARDS, PAWN_MOVES_CAPTURE, RANK_1_BITS, south_fill};
use crate::engine_constants::{BISHOP_VALUE, DOUBLED_PAWN_PENALTY, KNIGHT_VALUE, PAWN_TRADE_BONUS_MAX, PAWN_VALUE, QUEEN_VALUE, ROOK_VALUE, VALUE_ROOKS_ON_SAME_FILE};
use crate::move_scores::TOTAL_PIECE_VALUE_PER_SIDE_AT_START;
use crate::{get_and_unset_lsb, opponent};
use crate::magic_bitboards::{magic_moves_bishop, magic_moves_rook};
use crate::piece_square_tables::piece_square_values;
use crate::types::{Bitboard, BLACK, Mover, Pieces, Position, Score, Square, WHITE};
use crate::utils::linear_scale;

pub const VALUE_BISHOP_MOBILITY: [Score; 14] = [-15, -10, -6, -2, 2, 6, 10, 13, 16, 18, 20, 22, 23, 24];

#[inline(always)]
pub fn evaluate(position: &Position) -> Score {

    let material_score = material_score(position);

    let score =
        material_score +
            piece_square_values(position) +
            rook_eval(position) +
            pawn_score(position);

    if position.mover == WHITE { score } else { -score }
}

#[inline(always)]
pub fn material(pieces: &Pieces) -> Score {
    (pieces.pawn_bitboard.count_ones() as Score * PAWN_VALUE +
    pieces.knight_bitboard.count_ones() as Score * KNIGHT_VALUE +
    pieces.rook_bitboard.count_ones() as Score * ROOK_VALUE +
    pieces.bishop_bitboard.count_ones() as Score * BISHOP_VALUE +
    pieces.queen_bitboard.count_ones() as Score * QUEEN_VALUE) as Score
}

#[inline(always)]
pub fn material_score(position: &Position) -> Score {
    ((position.pieces[WHITE as usize].pawn_bitboard.count_ones() as Score - position.pieces[BLACK as usize].pawn_bitboard.count_ones() as Score) as Score * PAWN_VALUE +
    (position.pieces[WHITE as usize].knight_bitboard.count_ones() as Score - position.pieces[BLACK as usize].knight_bitboard.count_ones() as Score) as Score * KNIGHT_VALUE +
    (position.pieces[WHITE as usize].rook_bitboard.count_ones() as Score - position.pieces[BLACK as usize].rook_bitboard.count_ones() as Score) as Score * ROOK_VALUE +
    (position.pieces[WHITE as usize].bishop_bitboard.count_ones() as Score - position.pieces[BLACK as usize].bishop_bitboard.count_ones() as Score) as Score * BISHOP_VALUE +
    (position.pieces[WHITE as usize].queen_bitboard.count_ones() as Score - position.pieces[BLACK as usize].queen_bitboard.count_ones() as Score) as Score * QUEEN_VALUE) as Score
}

#[inline(always)]
pub fn piece_material(position: &Position, mover: Mover) -> Score {
    position.pieces[mover as usize].knight_bitboard.count_ones() as Score +
    position.pieces[mover as usize].rook_bitboard.count_ones() as Score +
    position.pieces[mover as usize].bishop_bitboard.count_ones() as Score +
    position.pieces[mover as usize].queen_bitboard.count_ones() as Score
}

#[inline(always)]
pub fn pawn_material(position: &Position, mover: Mover) -> Score {
    position.pieces[mover as usize].pawn_bitboard.count_ones() as Score
}

#[inline(always)]
pub fn on_same_file_count(pawn_bitboard: Bitboard) -> Score {
    (pawn_bitboard.count_ones() as u8 - (south_fill(pawn_bitboard) & RANK_1_BITS).count_ones() as u8) as u8 as Score
}

#[inline(always)]
pub fn pawn_score(position: &Position) -> Score {
    ((on_same_file_count(position.pieces[BLACK as usize].pawn_bitboard) -
        on_same_file_count(position.pieces[WHITE as usize].pawn_bitboard)) as Score
        * DOUBLED_PAWN_PENALTY) as Score
}

#[inline(always)]
pub fn rook_eval(position: &Position) -> Score {
    (on_same_file_count(position.pieces[WHITE as usize].rook_bitboard) -
        on_same_file_count(position.pieces[BLACK as usize].rook_bitboard)) * VALUE_ROOKS_ON_SAME_FILE
}
