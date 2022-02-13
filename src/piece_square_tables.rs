use crate::engine_constants::{BISHOP_VALUE, KNIGHT_VALUE, QUEEN_VALUE, ROOK_VALUE};
use crate::evaluate::material;
use crate::move_scores::{BIT_FLIPPED_HORIZONTAL_AXIS, KNIGHT_STAGE_MATERIAL_HIGH, KNIGHT_STAGE_MATERIAL_LOW, OPENING_PHASE_MATERIAL, PAWN_STAGE_MATERIAL_HIGH, PAWN_STAGE_MATERIAL_LOW, piece_type};
use crate::{get_and_unset_lsb, opponent};
use crate::types::{BLACK, Move, Pieces, Position, Score, Square, WHITE};
use crate::utils::{from_square_part, linear_scale, to_square_part};
use crate::types::Piece::{Bishop, King, Knight, Pawn, Queen, Rook};

pub const PAWN_PIECE_SQUARE_TABLE: [Score; 64] = [
0, 0, 0, 0, 0, 0, 0, 0,
-6, 4, 4, -15, -15, 4, 4, -6,
-6, 4, 2, 5, 5, 2, 4, -6,
-6, 4, 5, 16, 16, 5, 4, -6,
-5, 4, 10, 20, 20, 10, 4, -5,
3, 12, 20, 28, 28, 20, 12, 3,
8, 16, 24, 32, 32, 24, 16, 8,
0, 0, 0, 0, 0, 0, 0, 0];

pub const PAWN_END_GAME_PIECE_SQUARE_TABLE: [Score; 64] = [
0, 0, 0, 0, 0, 0, 0, 0,
-20, 0, 0, 0, 0, 0, 0, -20,
-15, 5, 5, 5, 5, 5, 5, -15,
-10, 10, 10, 10, 10, 10, 10, -10,
5, 25, 25, 25, 25, 25, 25, 5,
20, 30, 35, 35, 35, 35, 30, 20,
25, 40, 45, 45, 45, 45, 40, 25,
0, 0, 0, 0, 0, 0, 0, 0
];

pub const KNIGHT_PIECE_SQUARE_TABLE: [Score; 64] = [
-50, -40, -30, -20, -20, -30, -40, -50,
-40, -30, -10, 0, 0, -10, -30, -40,
-20, -10, 0, 0, 0, 0, -10, -20,
-17, 0, 3, 20, 20, 3, 0, -17,
-17, 0, 10, 20, 20, 10, 0, -17,
-20, 5, 7, 15, 15, 7, 5, -20,
-40, -30, -10, 0, 0, -10, -30, -40,
-50, -40, -30, -20, -20, -30, -40, -50
];

pub const KNIGHT_END_GAME_PIECE_SQUARE_TABLE: [Score; 64] = [
-50, -40, -30, -20, -20, -30, -40, -50,
-40, -30, -10, -5, -5, -10, -30, -40,
-30, -10, 0, 10, 10, 0, -10, -30,
-20, -5, 10, 20, 20, 10, -5, -20,
-20, -5, 10, 20, 20, 10, -5, -20,
-30, -10, 0, 10, 10, 0, -10, -30,
-40, -30, -10, -5, -5, -10, -30, -40,
-50, -40, -30, -20, -20, -30, -40, -50
];

pub const BISHOP_PIECE_SQUARE_TABLE: [Score; 64] = [
0, 0, 0, 0, 0, 0, 0, 0,
0, 5, 2, 2, 2, 2, 5, 0,
0, 3, 5, 5, 5, 5, 3, 0,
0, 2, 5, 5, 5, 5, 2, 0,
0, 2, 5, 5, 5, 5, 2, 0,
0, 2, 5, 5, 5, 5, 2, 0,
0, 5, 2, 2, 2, 2, 5, 0,
0, 0, 0, 0, 0, 0, 0, 0
];

pub const ROOK_PIECE_SQUARE_TABLE: [Score; 64] = [
0, 3, 5, 5, 5, 5, 3, 0,
-3, 2, 5, 5, 5, 5, 2, -3,
-2, 0, 0, 2, 2, 0, 0, -2,
-2, 0, 0, 0, 0, 0, 0, -2,
0, 0, 0, 0, 0, 0, 0, 0,
0, 0, 0, 0, 0, 0, 0, 0,
15, 20, 20, 20, 20, 20, 20, 15,
0, 3, 5, 5, 5, 5, 3, 0
];

pub const QUEEN_PIECE_SQUARE_TABLE: [Score; 64] = [
-10, -5, 0, 0, 0, 0, -5, -10,
-5, 0, 5, 5, 5, 5, 0, -5,
0, 5, 5, 6, 6, 5, 5, 0,
0, 5, 6, 6, 6, 6, 5, 0,
0, 5, 6, 6, 6, 6, 5, 0,
0, 5, 5, 6, 6, 5, 5, 0,
-5, 0, 5, 5, 5, 5, 0, -5,
-10, -5, 0, 0, 0, 0, -5, -10
];

pub const KING_PIECE_SQUARE_TABLE: [Score; 64] = [
24, 24, 9, 0, 0, 9, 24, 24,
16, 14, 7, -3, -3, 7, 14, 16,
4, -2, -5, -15, -15, -5, -2, 4,
-10, -15, -20, -25, -25, -20, -15, -10,
-15, -30, -35, -40, -40, -35, -30, -15,
-25, -35, -40, -45, -45, -40, -35, -25,
-22, -35, -40, -40, -40, -40, -35, -22,
-22, -35, -40, -40, -40, -40, -35, -22
];

pub const KING_END_GAME_PIECE_SQUARE_TABLE: [Score; 64] = [
0, 8, 16, 24, 24, 16, 8, 0,
8, 16, 24, 32, 32, 24, 16, 8,
16, 24, 32, 40, 40, 32, 24, 16,
24, 32, 40, 48, 48, 40, 32, 24,
24, 32, 40, 48, 48, 40, 32, 24,
16, 24, 32, 40, 40, 32, 24, 16,
8, 16, 24, 32, 32, 24, 16, 8,
0, 8, 16, 24, 24, 16, 8, 0
];

pub const KING_IN_CORNER_PIECE_SQUARE_TABLE: [Score; 64] = [
14, 13, 12, 11, 11, 12, 13, 14,
13, 12, 11, 10, 10, 11, 12, 13,
12, 11, 10,  9,  9, 10, 11, 12,
11, 10,  9,  8,  8,  9, 10, 11,
11, 10,  9,  8,  8,  9, 10, 11,
12, 11, 10,  9,  9, 10, 11, 12,
13, 12, 11, 10, 10, 11, 12, 13,
14, 13, 12, 11, 11, 12, 13, 14
];

pub fn non_pawn_piece_values(pieces: &Pieces) -> Score {
    (pieces.knight_bitboard.count_ones() as i16 * KNIGHT_VALUE +
        pieces.rook_bitboard.count_ones() as i16 * ROOK_VALUE +
        pieces.bishop_bitboard.count_ones() as i16 * BISHOP_VALUE +
        pieces.queen_bitboard.count_ones() as i16 * QUEEN_VALUE) as Score
}

pub fn score_piece_square_values(position: &Position, mv: Move) -> Score {
    let from = from_square_part(mv) as usize;
    let to = to_square_part(mv) as usize;
    let piece = piece_type(position, to as Square);
    let from_adjusted = if position.mover == WHITE { from } else { BIT_FLIPPED_HORIZONTAL_AXIS[from] as usize };
    let to_adjusted = if position.mover == WHITE { to } else { BIT_FLIPPED_HORIZONTAL_AXIS[to] as usize };
    let enemy = position.pieces[opponent!(position.mover) as usize];

    match piece {
        Pawn => {
            linear_scale(non_pawn_piece_values(&enemy), PAWN_STAGE_MATERIAL_LOW, PAWN_STAGE_MATERIAL_HIGH, PAWN_END_GAME_PIECE_SQUARE_TABLE[to_adjusted] - PAWN_END_GAME_PIECE_SQUARE_TABLE[from_adjusted], PAWN_PIECE_SQUARE_TABLE[to_adjusted] - PAWN_PIECE_SQUARE_TABLE[from_adjusted])
        },
        Rook => {
            ROOK_PIECE_SQUARE_TABLE[to_adjusted] - ROOK_PIECE_SQUARE_TABLE[from_adjusted]
        },
        Knight => {
            linear_scale(material(&enemy), KNIGHT_STAGE_MATERIAL_LOW, KNIGHT_STAGE_MATERIAL_HIGH, KNIGHT_END_GAME_PIECE_SQUARE_TABLE[to_adjusted] - KNIGHT_END_GAME_PIECE_SQUARE_TABLE[from_adjusted], KNIGHT_PIECE_SQUARE_TABLE[to_adjusted] - KNIGHT_PIECE_SQUARE_TABLE[from_adjusted])
        },
        Bishop => {
            BISHOP_PIECE_SQUARE_TABLE[to_adjusted] - BISHOP_PIECE_SQUARE_TABLE[from_adjusted]
        },
        Queen => {
            QUEEN_PIECE_SQUARE_TABLE[to_adjusted] - QUEEN_PIECE_SQUARE_TABLE[from_adjusted]
        },
        King => {
            linear_scale(non_pawn_piece_values(&enemy), ROOK_VALUE, OPENING_PHASE_MATERIAL, KING_END_GAME_PIECE_SQUARE_TABLE[to_adjusted] - KING_END_GAME_PIECE_SQUARE_TABLE[from_adjusted], KING_PIECE_SQUARE_TABLE[to_adjusted] - KING_PIECE_SQUARE_TABLE[from_adjusted])
        },
        _ => {
            panic!("Expected a piece");
        }
    }
}

pub fn piece_square_values(position: &Position) -> Score {

    let mut score: Score = 0;
    let enemy = position.pieces[BLACK as usize];

    let mut bb = position.pieces[WHITE as usize].pawn_bitboard;
    while bb != 0 {
        let sq = get_and_unset_lsb!(bb) as usize;
        score += linear_scale(non_pawn_piece_values(&enemy), PAWN_STAGE_MATERIAL_LOW, PAWN_STAGE_MATERIAL_HIGH, PAWN_END_GAME_PIECE_SQUARE_TABLE[sq], PAWN_PIECE_SQUARE_TABLE[sq])
    }

    let mut bb = position.pieces[WHITE as usize].rook_bitboard;
    while bb != 0 {
        let sq = get_and_unset_lsb!(bb) as usize;
        score += ROOK_PIECE_SQUARE_TABLE[sq];
    }

    let mut bb = position.pieces[WHITE as usize].bishop_bitboard;
    while bb != 0 {
        let sq = BIT_FLIPPED_HORIZONTAL_AXIS[get_and_unset_lsb!(bb) as usize] as usize;
        score += BISHOP_PIECE_SQUARE_TABLE[sq];
    }

    let mut bb = position.pieces[WHITE as usize].queen_bitboard;
    while bb != 0 {
        let sq = get_and_unset_lsb!(bb) as usize;
        score += QUEEN_PIECE_SQUARE_TABLE[sq];
    }

    let mut bb = position.pieces[WHITE as usize].knight_bitboard;
    while bb != 0 {
        let sq = get_and_unset_lsb!(bb) as usize;
        score += linear_scale(non_pawn_piece_values(&enemy), KNIGHT_STAGE_MATERIAL_LOW, KNIGHT_STAGE_MATERIAL_HIGH, KNIGHT_END_GAME_PIECE_SQUARE_TABLE[sq], KNIGHT_PIECE_SQUARE_TABLE[sq])
    }

    let sq = position.pieces[WHITE as usize].king_square as usize;
    score += linear_scale(non_pawn_piece_values(&enemy), ROOK_VALUE, OPENING_PHASE_MATERIAL, KING_END_GAME_PIECE_SQUARE_TABLE[sq], KING_PIECE_SQUARE_TABLE[sq]);

    let enemy = position.pieces[WHITE as usize];

    let mut bb = position.pieces[BLACK as usize].pawn_bitboard;
    while bb != 0 {
        let sq = BIT_FLIPPED_HORIZONTAL_AXIS[get_and_unset_lsb!(bb) as usize] as usize;
        score -= linear_scale(non_pawn_piece_values(&enemy), PAWN_STAGE_MATERIAL_LOW, PAWN_STAGE_MATERIAL_HIGH, PAWN_END_GAME_PIECE_SQUARE_TABLE[sq], PAWN_PIECE_SQUARE_TABLE[sq])
    }

    let mut bb = position.pieces[BLACK as usize].rook_bitboard;
    while bb != 0 {
        let sq = BIT_FLIPPED_HORIZONTAL_AXIS[get_and_unset_lsb!(bb) as usize] as usize;
        score -= ROOK_PIECE_SQUARE_TABLE[sq];
    }

    let mut bb = position.pieces[BLACK as usize].bishop_bitboard;
    while bb != 0 {
        let sq = BIT_FLIPPED_HORIZONTAL_AXIS[get_and_unset_lsb!(bb) as usize] as usize;
        score -= BISHOP_PIECE_SQUARE_TABLE[sq];
    }

    let mut bb = position.pieces[BLACK as usize].queen_bitboard;
    while bb != 0 {
        let sq = BIT_FLIPPED_HORIZONTAL_AXIS[get_and_unset_lsb!(bb) as usize] as usize;
        score -= QUEEN_PIECE_SQUARE_TABLE[sq];
    }

    let mut bb = position.pieces[BLACK as usize].knight_bitboard;
    while bb != 0 {
        let sq = BIT_FLIPPED_HORIZONTAL_AXIS[get_and_unset_lsb!(bb) as usize] as usize;
        score -= linear_scale(non_pawn_piece_values(&enemy), KNIGHT_STAGE_MATERIAL_LOW, KNIGHT_STAGE_MATERIAL_HIGH, KNIGHT_END_GAME_PIECE_SQUARE_TABLE[sq], KNIGHT_PIECE_SQUARE_TABLE[sq])
    }

    let sq = BIT_FLIPPED_HORIZONTAL_AXIS[position.pieces[BLACK as usize].king_square as usize] as usize;
    score -= linear_scale(non_pawn_piece_values(&enemy), ROOK_VALUE, OPENING_PHASE_MATERIAL, KING_END_GAME_PIECE_SQUARE_TABLE[sq], KING_PIECE_SQUARE_TABLE[sq]);

    score

}