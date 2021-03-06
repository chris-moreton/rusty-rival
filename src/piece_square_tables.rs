use crate::engine_constants::{BISHOP_VALUE_AVERAGE, KNIGHT_VALUE_AVERAGE, PAWN_VALUE_AVERAGE, QUEEN_VALUE_AVERAGE, ROOK_VALUE_AVERAGE};
use crate::get_and_unset_lsb;
use crate::move_scores::{
    BIT_FLIPPED_HORIZONTAL_AXIS, KNIGHT_STAGE_MATERIAL_HIGH, KNIGHT_STAGE_MATERIAL_LOW, OPENING_PHASE_MATERIAL, PAWN_STAGE_MATERIAL_HIGH,
    PAWN_STAGE_MATERIAL_LOW,
};
use crate::types::{Pieces, Position, Score, Square, BLACK, WHITE};
use crate::utils::linear_scale;

pub const PAWN_PIECE_SQUARE_TABLE: [Score; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, -6, 4, 4, -15, -15, 4, 4, -6, -6, 4, 2, 5, 5, 2, 4, -6, -6, 4, 5, 16, 16, 5, 4, -6, -5, 4, 10, 20, 20, 10, 4,
    -5, 3, 12, 20, 28, 28, 20, 12, 3, 8, 16, 24, 32, 32, 24, 16, 8, 0, 0, 0, 0, 0, 0, 0, 0,
];

pub const PAWN_END_GAME_PIECE_SQUARE_TABLE: [Score; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, -20, 0, 0, 0, 0, 0, 0, -20, -15, 5, 5, 5, 5, 5, 5, -15, -10, 10, 10, 10, 10, 10, 10, -10, 5, 25, 25, 25, 25,
    25, 25, 5, 20, 30, 35, 35, 35, 35, 30, 20, 25, 40, 45, 45, 45, 45, 40, 25, 0, 0, 0, 0, 0, 0, 0, 0,
];

pub const KNIGHT_PIECE_SQUARE_TABLE: [Score; 64] = [
    -50, -40, -30, -20, -20, -30, -40, -50, -40, -30, -10, 0, 0, -10, -30, -40, -20, -10, 0, 0, 0, 0, -10, -20, -17, 0, 3, 20, 20, 3, 0,
    -17, -17, 0, 10, 20, 20, 10, 0, -17, -20, 5, 7, 15, 15, 7, 5, -20, -40, -30, -10, 0, 0, -10, -30, -40, -50, -40, -30, -20, -20, -30,
    -40, -50,
];

pub const KNIGHT_END_GAME_PIECE_SQUARE_TABLE: [Score; 64] = [
    -50, -40, -30, -20, -20, -30, -40, -50, -40, -30, -10, -5, -5, -10, -30, -40, -30, -10, 0, 10, 10, 0, -10, -30, -20, -5, 10, 20, 20,
    10, -5, -20, -20, -5, 10, 20, 20, 10, -5, -20, -30, -10, 0, 10, 10, 0, -10, -30, -40, -30, -10, -5, -5, -10, -30, -40, -50, -40, -30,
    -20, -20, -30, -40, -50,
];

pub const BISHOP_PIECE_SQUARE_TABLE: [Score; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 2, 2, 2, 2, 5, 0, 0, 3, 5, 5, 5, 5, 3, 0, 0, 2, 5, 5, 5, 5, 2, 0, 0, 2, 5, 5, 5, 5, 2, 0, 0, 2, 5, 5, 5,
    5, 2, 0, 0, 5, 2, 2, 2, 2, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

pub const ROOK_PIECE_SQUARE_TABLE: [Score; 64] = [
    0, 3, 5, 5, 5, 5, 3, 0, -3, 2, 5, 5, 5, 5, 2, -3, -2, 0, 0, 2, 2, 0, 0, -2, -2, 0, 0, 0, 0, 0, 0, -2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 15, 20, 20, 20, 20, 20, 20, 15, 0, 3, 5, 5, 5, 5, 3, 0,
];

pub const QUEEN_PIECE_SQUARE_TABLE: [Score; 64] = [
    -10, -5, 0, 0, 0, 0, -5, -10, -5, 0, 5, 5, 5, 5, 0, -5, 0, 5, 5, 6, 6, 5, 5, 0, 0, 5, 6, 6, 6, 6, 5, 0, 0, 5, 6, 6, 6, 6, 5, 0, 0, 5,
    5, 6, 6, 5, 5, 0, -5, 0, 5, 5, 5, 5, 0, -5, -10, -5, 0, 0, 0, 0, -5, -10,
];

pub const KING_PIECE_SQUARE_TABLE: [Score; 64] = [
    24, 24, 9, 0, 0, 9, 24, 24, 16, 14, 7, -3, -3, 7, 14, 16, 4, -2, -5, -15, -15, -5, -2, 4, -10, -15, -20, -25, -25, -20, -15, -10, -15,
    -30, -35, -40, -40, -35, -30, -15, -25, -35, -40, -45, -45, -40, -35, -25, -22, -35, -40, -40, -40, -40, -35, -22, -22, -35, -40, -40,
    -40, -40, -35, -22,
];

pub const KING_END_GAME_PIECE_SQUARE_TABLE: [Score; 64] = [
    0, 8, 16, 24, 24, 16, 8, 0, 8, 16, 24, 32, 32, 24, 16, 8, 16, 24, 32, 40, 40, 32, 24, 16, 24, 32, 40, 48, 48, 40, 32, 24, 24, 32, 40,
    48, 48, 40, 32, 24, 16, 24, 32, 40, 40, 32, 24, 16, 8, 16, 24, 32, 32, 24, 16, 8, 0, 8, 16, 24, 24, 16, 8, 0,
];

pub const KING_IN_CORNER_PIECE_SQUARE_TABLE: [Score; 64] = [
    14, 13, 12, 11, 11, 12, 13, 14, 13, 12, 11, 10, 10, 11, 12, 13, 12, 11, 10, 9, 9, 10, 11, 12, 11, 10, 9, 8, 8, 9, 10, 11, 11, 10, 9, 8,
    8, 9, 10, 11, 12, 11, 10, 9, 9, 10, 11, 12, 13, 12, 11, 10, 10, 11, 12, 13, 14, 13, 12, 11, 11, 12, 13, 14,
];

#[inline(always)]
pub fn non_pawn_piece_values(pieces: &Pieces) -> Score {
    (pieces.knight_bitboard.count_ones() as Score * KNIGHT_VALUE_AVERAGE
        + pieces.rook_bitboard.count_ones() as Score * ROOK_VALUE_AVERAGE
        + pieces.bishop_bitboard.count_ones() as Score * BISHOP_VALUE_AVERAGE
        + pieces.queen_bitboard.count_ones() as Score * QUEEN_VALUE_AVERAGE) as Score
}

#[inline(always)]
pub fn pawn_values(pieces: &Pieces) -> Score {
    pieces.pawn_bitboard.count_ones() as Score * PAWN_VALUE_AVERAGE
}

#[inline(always)]
pub fn piece_square_values(position: &Position) -> Score {
    let bnppv = non_pawn_piece_values(&position.pieces[BLACK as usize]);
    let wnppv = non_pawn_piece_values(&position.pieces[WHITE as usize]);
    let wpv = pawn_values(&position.pieces[WHITE as usize]);
    let bpv = pawn_values(&position.pieces[BLACK as usize]);

    white_pawn_piece_square_values(position, bnppv)
        + white_rook_piece_square_values(position)
        + white_queen_piece_square_values(position)
        + white_knight_piece_square_values(position, bnppv + bpv)
        + white_king_piece_square_values(position, bnppv)
        + white_bishop_piece_square_values(position)
        - black_pawn_piece_square_values(position, wnppv)
        - black_rook_piece_square_values(position)
        - black_queen_piece_square_values(position)
        - black_knight_piece_square_values(position, wnppv + wpv)
        + black_king_piece_square_values(position, wnppv)
        + black_bishop_piece_square_values(position)
}

#[inline(always)]
fn white_queen_piece_square_values(position: &Position) -> Score {
    let mut bb = position.pieces[WHITE as usize].queen_bitboard;
    let mut score = 0;
    while bb != 0 {
        let sq = get_and_unset_lsb!(bb) as usize;
        score += QUEEN_PIECE_SQUARE_TABLE[sq];
    }
    score
}

#[inline(always)]
fn black_queen_piece_square_values(position: &Position) -> Score {
    let mut bb = position.pieces[BLACK as usize].queen_bitboard;
    let mut score = 0;
    while bb != 0 {
        let sq = BIT_FLIPPED_HORIZONTAL_AXIS[get_and_unset_lsb!(bb) as usize] as usize;
        score += QUEEN_PIECE_SQUARE_TABLE[sq];
    }
    score
}

#[inline(always)]
pub fn white_bishop_piece_square_values(position: &Position) -> Score {
    let mut bb = position.pieces[WHITE as usize].bishop_bitboard;
    let mut score = 0;
    while bb != 0 {
        let sq = get_and_unset_lsb!(bb);
        score += BISHOP_PIECE_SQUARE_TABLE[sq as usize];
    }

    score
}

#[inline(always)]
pub fn black_bishop_piece_square_values(position: &Position) -> Score {
    let mut bb = position.pieces[BLACK as usize].bishop_bitboard;
    let mut score = 0;
    while bb != 0 {
        let sq = BIT_FLIPPED_HORIZONTAL_AXIS[get_and_unset_lsb!(bb) as usize];
        score += BISHOP_PIECE_SQUARE_TABLE[sq as usize];
    }

    score
}

#[inline(always)]
fn white_rook_piece_square_values(position: &Position) -> Score {
    let mut bb = position.pieces[WHITE as usize].rook_bitboard;
    let mut score = 0;
    while bb != 0 {
        let sq = get_and_unset_lsb!(bb) as usize;
        score += ROOK_PIECE_SQUARE_TABLE[sq];
    }
    score
}

#[inline(always)]
fn black_rook_piece_square_values(position: &Position) -> Score {
    let mut bb = position.pieces[BLACK as usize].rook_bitboard;
    let mut score = 0;
    while bb != 0 {
        let sq = BIT_FLIPPED_HORIZONTAL_AXIS[get_and_unset_lsb!(bb) as usize] as usize;
        score += ROOK_PIECE_SQUARE_TABLE[sq];
    }
    score
}

#[inline(always)]
pub fn white_pawn_piece_square_values(position: &Position, nppv: Score) -> Score {
    let mut bb = position.pieces[WHITE as usize].pawn_bitboard;
    let mut pawn_count = 0;
    let mut min_total = 0;
    let mut max_total = 0;
    while bb != 0 {
        pawn_count += 1;
        let sq = get_and_unset_lsb!(bb) as usize;
        min_total += PAWN_END_GAME_PIECE_SQUARE_TABLE[sq];
        max_total += PAWN_PIECE_SQUARE_TABLE[sq];
    }
    linear_scale(
        (nppv * pawn_count) as i64,
        (PAWN_STAGE_MATERIAL_LOW * pawn_count) as i64,
        (PAWN_STAGE_MATERIAL_HIGH * pawn_count) as i64,
        min_total as i64,
        max_total as i64,
    ) as Score
}

#[inline(always)]
pub fn black_pawn_piece_square_values(position: &Position, nppv: Score) -> Score {
    let mut bb = position.pieces[BLACK as usize].pawn_bitboard;
    let mut pawn_count = 0;
    let mut min_total = 0;
    let mut max_total = 0;
    while bb != 0 {
        pawn_count += 1;
        let sq = BIT_FLIPPED_HORIZONTAL_AXIS[get_and_unset_lsb!(bb) as usize] as usize;
        min_total += PAWN_END_GAME_PIECE_SQUARE_TABLE[sq];
        max_total += PAWN_PIECE_SQUARE_TABLE[sq];
    }
    linear_scale(
        (nppv * pawn_count) as i64,
        (PAWN_STAGE_MATERIAL_LOW * pawn_count) as i64,
        (PAWN_STAGE_MATERIAL_HIGH * pawn_count) as i64,
        min_total as i64,
        max_total as i64,
    ) as Score
}

#[inline(always)]
pub fn white_knight_piece_square_values(position: &Position, pv: Score) -> Score {
    let mut bb = position.pieces[WHITE as usize].knight_bitboard;
    let mut score = 0;
    while bb != 0 {
        let sq = get_and_unset_lsb!(bb) as usize;
        score += linear_scale(
            pv as i64,
            KNIGHT_STAGE_MATERIAL_LOW as i64,
            KNIGHT_STAGE_MATERIAL_HIGH as i64,
            KNIGHT_END_GAME_PIECE_SQUARE_TABLE[sq] as i64,
            KNIGHT_PIECE_SQUARE_TABLE[sq] as i64,
        ) as Score
    }
    score
}

#[inline(always)]
pub fn black_knight_piece_square_values(position: &Position, pv: Score) -> Score {
    let mut bb = position.pieces[BLACK as usize].knight_bitboard;
    let mut score = 0;
    while bb != 0 {
        let sq = BIT_FLIPPED_HORIZONTAL_AXIS[get_and_unset_lsb!(bb) as usize] as usize;
        score += linear_scale(
            pv as i64,
            KNIGHT_STAGE_MATERIAL_LOW as i64,
            KNIGHT_STAGE_MATERIAL_HIGH as i64,
            KNIGHT_END_GAME_PIECE_SQUARE_TABLE[sq] as i64,
            KNIGHT_PIECE_SQUARE_TABLE[sq] as i64,
        ) as Score
    }
    score
}

#[inline(always)]
pub fn white_king_piece_square_values(position: &Position, nppv: Score) -> Score {
    let sq = position.pieces[WHITE as usize].king_square as usize;
    linear_scale(
        nppv as i64,
        ROOK_VALUE_AVERAGE as i64,
        OPENING_PHASE_MATERIAL as i64,
        KING_END_GAME_PIECE_SQUARE_TABLE[sq] as i64,
        KING_PIECE_SQUARE_TABLE[sq] as i64,
    ) as Score
}

#[inline(always)]
pub fn black_king_piece_square_values(position: &Position, nppv: Score) -> Score {
    let sq = BIT_FLIPPED_HORIZONTAL_AXIS[position.pieces[BLACK as usize].king_square as usize] as usize;
    linear_scale(
        nppv as i64,
        ROOK_VALUE_AVERAGE as i64,
        OPENING_PHASE_MATERIAL as i64,
        KING_END_GAME_PIECE_SQUARE_TABLE[sq] as i64,
        KING_PIECE_SQUARE_TABLE[sq] as i64,
    ) as Score
}
