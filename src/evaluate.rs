use crate::bitboards::{bit, RANK_1_BITS, south_fill};
use crate::engine_constants::{BISHOP_VALUE, DOUBLED_PAWN_PENALTY, KNIGHT_VALUE, PAWN_VALUE, QUEEN_VALUE, ROOK_VALUE, VALUE_ROOKS_ON_SAME_FILE};
use crate::piece_square_tables::piece_square_values;
use crate::types::{Bitboard, BLACK, Mover, Pieces, Position, Score, WHITE};

pub const VALUE_BISHOP_MOBILITY: [Score; 14] = [-15, -10, -6, -2, 2, 6, 10, 13, 16, 18, 20, 22, 23, 24];

#[inline(always)]
pub fn evaluate(position: &Position) -> Score {

    let piece_count = position.pieces[WHITE as usize].all_pieces_bitboard.count_ones() + position.pieces[BLACK as usize].all_pieces_bitboard.count_ones();
    if piece_count == 2 {
        return 0;
    }

    let material_score = material_score(position);

    let score =
        material_score +
            piece_square_values(position) +
            king_score(position, piece_count) +
            rook_eval(position) +
            pawn_score(position);

    if position.mover == WHITE { score } else { -score }
}

pub fn king_score(position: &Position, piece_count: u32) -> Score {
    let mut score = 0 as Score;

    if piece_count > 10 {
        score += king_early_safety(position);
    }

    score
}

pub fn contains_all_bits(bitboard: Bitboard, mask: Bitboard) -> bool {
    bitboard & mask == mask
}

pub fn king_early_safety(position: &Position) -> Score {
    let mut white_score: Score = white_king_early_safety(position);
    let mut black_score: Score = black_king_early_safety(position);

    white_score - black_score
}

pub fn black_king_early_safety(position: &Position) -> Score {
    let mut score = 0;
    let black = position.pieces[BLACK as usize];
    if bit(black.king_square) & 0b0000001100000000000000000000000000000000000000000000000000000000 != 0 {
        if black.rook_bitboard & 0b0000010000000000000000000000000000000000000000000000000000000000 != 0 {
            if contains_all_bits(black.pawn_bitboard, 0b0000000000000111000000000000000000000000000000000000000000000000) {
                score += 30 // (A)
            } else if contains_all_bits(black.pawn_bitboard, 0b0000000000000101000000100000000000000000000000000000000000000000) {
                score += 10; // (B)
                if black.bishop_bitboard & 0b0000000000000010000000000000000000000000000000000000000000000000 != 0 {
                    score += 10; // (B)
                }
            }
        }
    }
    score
}

pub fn white_king_early_safety(position: &Position) -> Score {
    let mut score = 0;
    let white = position.pieces[WHITE as usize];
    if bit(white.king_square) & 0b0000000000000000000000000000000000000000000000000000000000000011 != 0 {
        if white.rook_bitboard & 0b0000000000000000000000000000000000000000000000000000000000000100 != 0 {
            if contains_all_bits(white.pawn_bitboard, 0b0000000000000000000000000000000000000000000000000000011100000000) {
                score += 30 // (A)
            } else if contains_all_bits(white.pawn_bitboard, 0b0000000000000000000000000000000000000000000000100000010100000000) {
                score += 10; // (B)
                if white.bishop_bitboard & 0b0000000000000000000000000000000000000000000000000000001000000000 != 0 {
                    score += 10; // (B)
                }
            }
        }
    }
    score
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
pub fn on_same_file_count(pawn_bitboard: Bitboard, pawn_files: u8) -> Score {
    pawn_bitboard.count_ones() as Score - (pawn_files.count_ones() as Score)
}

#[inline(always)]
pub fn isolated_pawn_count(pawn_files: u8) -> Score {
    let left: u8 = (pawn_files & (pawn_files << 1)) as u8;
    let right: u8 = (pawn_files & (pawn_files >> 1)) as u8;

    let not_isolated: u8 = (left | right).count_ones() as u8;
    (pawn_files.count_ones() - not_isolated as u32) as Score
}

#[inline(always)]
pub fn pawn_score(position: &Position) -> Score {

    let white_pawn_files: u8 = (south_fill(position.pieces[WHITE as usize].pawn_bitboard) & RANK_1_BITS) as u8;
    let black_pawn_files: u8 = (south_fill(position.pieces[BLACK as usize].pawn_bitboard) & RANK_1_BITS) as u8;

    let doubled = ((on_same_file_count(position.pieces[BLACK as usize].pawn_bitboard, black_pawn_files) -
        on_same_file_count(position.pieces[WHITE as usize].pawn_bitboard, white_pawn_files)) as Score
        * DOUBLED_PAWN_PENALTY) as Score;

    // let isolated = (isolated_pawn_count(black_pawn_files) - isolated_pawn_count(white_pawn_files)) * ISOLATED_PAWN_PENALTY;

    doubled
}

#[inline(always)]
pub fn rook_eval(position: &Position) -> Score {
    let white_rook_files: u8 = (south_fill(position.pieces[WHITE as usize].rook_bitboard) & RANK_1_BITS) as u8;
    let black_rook_files: u8 = (south_fill(position.pieces[BLACK as usize].rook_bitboard) & RANK_1_BITS) as u8;

    (on_same_file_count(position.pieces[WHITE as usize].rook_bitboard, white_rook_files) -
        on_same_file_count(position.pieces[BLACK as usize].rook_bitboard, black_rook_files)) * VALUE_ROOKS_ON_SAME_FILE
}
