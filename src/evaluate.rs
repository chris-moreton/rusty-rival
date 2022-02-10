use crate::types::{BLACK, Pieces, Position, Score, WHITE};

const PAWN_VALUE: Score = 100;
const KNIGHT_VALUE: Score = 350;
const BISHOP_VALUE: Score = 350;
const ROOK_VALUE: Score = 550;
const QUEEN_VALUE: Score = 900;

#[inline(always)]
pub fn material(pieces: &Pieces) -> Score {
    (pieces.pawn_bitboard.count_ones() as i16 * PAWN_VALUE +
    pieces.knight_bitboard.count_ones() as i16 * KNIGHT_VALUE +
    pieces.rook_bitboard.count_ones() as i16 * ROOK_VALUE +
    pieces.bishop_bitboard.count_ones() as i16 * BISHOP_VALUE +
    pieces.queen_bitboard.count_ones() as i16 * QUEEN_VALUE) as Score
}

#[inline(always)]
pub fn evaluate(position: &Position) -> Score {
    let white = position.pieces[WHITE as usize];
    let black = position.pieces[BLACK as usize];

    if position.mover == WHITE {
        material(&white) - material(&black)
    } else {
        material(&black) - material(&white)
    }
}
