use crate::bitboards::{bit, KNIGHT_MOVES_BITBOARDS, PAWN_MOVES_CAPTURE, RANK_1_BITS, south_fill};
use crate::engine_constants::{BISHOP_VALUE, DOUBLED_PAWN_PENALTY, KNIGHT_VALUE, PAWN_TRADE_BONUS_MAX, PAWN_VALUE, QUEEN_VALUE, ROOK_VALUE, VALUE_ROOKS_ON_SAME_FILE};
use crate::move_scores::TOTAL_PIECE_VALUE_PER_SIDE_AT_START;
use crate::{get_and_unset_lsb, opponent};
use crate::magic_bitboards::{magic_moves_bishop, magic_moves_rook};
use crate::piece_square_tables::piece_square_values;
use crate::types::{Bitboard, BLACK, Mover, Pieces, Position, Score, Square, WHITE};
use crate::utils::linear_scale;

pub const VALUE_BISHOP_MOBILITY: [Score; 14] = [-15, -10, -6, -2, 2, 6, 10, 13, 16, 18, 20, 22, 23, 24];
pub const VALUE_ONE_PAWN_SHIELDING: Score = 15;
pub const VALUE_NEAR_KING_ATTACKS: Score = 10;

pub const WHITE_KING_SHIELD_MASK: [Bitboard; 8] = [
    bit(9)| (bit(8)),
    bit(10)| (bit(9))| (bit(8)),
    bit(11)| (bit(10))| (bit(9)),
    bit(12)| (bit(11))| (bit(10)),
    bit(13)| (bit(12))| (bit(11)),
    bit(14)| (bit(13))| (bit(12)),
    bit(15)| (bit(14))| (bit(13)),
    bit(15)| (bit(14))
];

pub const KING_ADJACENT: [Bitboard; 64] = [
    0b0000000000000000000000000000000000000000000000000000001100000011,
    0b0000000000000000000000000000000000000000000000000000011100000111,
    0b0000000000000000000000000000000000000000000000000000111000001110,
    0b0000000000000000000000000000000000000000000000000001110000011100,
    0b0000000000000000000000000000000000000000000000000011100000111000,
    0b0000000000000000000000000000000000000000000000000111000001110000,
    0b0000000000000000000000000000000000000000000000001110000011100000,
    0b0000000000000000000000000000000000000000000000001100000011000000,

    0b0000000000000000000000000000000000000000000000110000001100000011,
    0b0000000000000000000000000000000000000000000001110000011100000111,
    0b0000000000000000000000000000000000000000000011100000111000001110,
    0b0000000000000000000000000000000000000000000111000001110000011100,
    0b0000000000000000000000000000000000000000001110000011100000111000,
    0b0000000000000000000000000000000000000000011100000111000001110000,
    0b0000000000000000000000000000000000000000111000001110000011100000,
    0b0000000000000000000000000000000000000000110000001100000110000000,

    0b0000000000000000000000000000000000000011000000110000001100000000,
    0b0000000000000000000000000000000000000111000001110000011100000000,
    0b0000000000000000000000000000000000001110000011100000111000000000,
    0b0000000000000000000000000000000000011100000111000001110000000000,
    0b0000000000000000000000000000000000111000001110000011100000000000,
    0b0000000000000000000000000000000001110000011100000111000000000000,
    0b0000000000000000000000000000000011100000111000001110000000000000,
    0b0000000000000000000000000000000011000000110000011000000000000000,

    0b0000000000000000000000000000001100000011000000110000000000000000,
    0b0000000000000000000000000000011100000111000001110000000000000000,
    0b0000000000000000000000000000111000001110000011100000000000000000,
    0b0000000000000000000000000001110000011100000111000000000000000000,
    0b0000000000000000000000000011100000111000001110000000000000000000,
    0b0000000000000000000000000111000001110000011100000000000000000000,
    0b0000000000000000000000001110000011100000111000000000000000000000,
    0b0000000000000000000000001100000011000001100000000000000000000000,

    0b0000000000000000000000110000001100000011000000000000000000000000,
    0b0000000000000000000001110000011100000111000000000000000000000000,
    0b0000000000000000000011100000111000001110000000000000000000000000,
    0b0000000000000000000111000001110000011100000000000000000000000000,
    0b0000000000000000001110000011100000111000000000000000000000000000,
    0b0000000000000000011100000111000001110000000000000000000000000000,
    0b0000000000000000111000001110000011100000000000000000000000000000,
    0b0000000000000000110000001100000110000000000000000000000000000000,

    0b0000000000000011000000110000001100000000000000000000000000000000,
    0b0000000000000111000001110000011100000000000000000000000000000000,
    0b0000000000001110000011100000111000000000000000000000000000000000,
    0b0000000000011100000111000001110000000000000000000000000000000000,
    0b0000000000111000001110000011100000000000000000000000000000000000,
    0b0000000001110000011100000111000000000000000000000000000000000000,
    0b0000000011100000111000001110000000000000000000000000000000000000,
    0b0000000011000000110000011000000000000000000000000000000000000000,

    0b0000001100000011000000110000000000000000000000000000000000000000,
    0b0000011100000111000001110000000000000000000000000000000000000000,
    0b0000111000001110000011100000000000000000000000000000000000000000,
    0b0001110000011100000111000000000000000000000000000000000000000000,
    0b0011100000111000001110000000000000000000000000000000000000000000,
    0b0111000001110000011100000000000000000000000000000000000000000000,
    0b1110000011100000111000000000000000000000000000000000000000000000,
    0b1100000011000001100000000000000000000000000000000000000000000000,

    0b0000001100000011000000000000000000000000000000000000000000000000,
    0b0000011100000111000000000000000000000000000000000000000000000000,
    0b0000111000001110000000000000000000000000000000000000000000000000,
    0b0001110000011100000000000000000000000000000000000000000000000000,
    0b0011100000111000000000000000000000000000000000000000000000000000,
    0b0111000001110000000000000000000000000000000000000000000000000000,
    0b1110000011100000000000000000000000000000000000000000000000000000,
    0b1100000110000000000000000000000000000000000000000000000000000000,

];

#[inline(always)]
pub fn evaluate(position: &Position) -> Score {

    let material_score = material_score(position);

    let score =
        material_score +
            piece_square_values(position) +
            rook_eval(position) +
            king_threats(position) +
            //pawn_shield(position) +
            pawn_score(position);

    if position.mover == WHITE { score } else { -score }
}

#[inline(always)]
pub fn king_threats(position: &Position) -> Score {
    VALUE_NEAR_KING_ATTACKS * (near_king_attack_count(position, WHITE) - near_king_attack_count(position, BLACK))
}

#[inline(always)]
pub fn near_king_attack_count(position: &Position, attacker: Mover) -> Score {

    let mut bb = KING_ADJACENT[position.pieces[opponent!(attacker) as usize].king_square as usize];
    let mut count = 0;

    while bb != 0 {
        let sq = get_and_unset_lsb!(bb);
        count += attack_count(position, sq, attacker);
    }

    count
}

#[inline(always)]
pub fn attack_count(position: &Position, attacked_square: Square, attacker: Mover) -> Score {
    let all_pieces = position.pieces[WHITE as usize].all_pieces_bitboard | position.pieces[BLACK as usize].all_pieces_bitboard;

    knight_attack_count(position.pieces[attacker as usize].knight_bitboard, attacked_square) +
        slider_attack_count(position.pieces[attacker as usize].rook_bitboard | position.pieces[attacker as usize].queen_bitboard, all_pieces, attacked_square, magic_moves_rook) +
        slider_attack_count(position.pieces[attacker as usize].bishop_bitboard | position.pieces[attacker as usize].queen_bitboard, all_pieces, attacked_square, magic_moves_bishop)
}

#[inline(always)]
pub fn slider_attack_count(mut slider_bitboard: Bitboard, all_pieces_bitboard: Bitboard, attacked_square: Square, magic_func: fn(Square, Bitboard) -> Bitboard) -> Score {
    (magic_func(attacked_square, all_pieces_bitboard) & slider_bitboard).count_ones() as Score
}

#[inline(always)]
pub fn knight_attack_count(mut knight_bitboard: Bitboard, attacked_square: Square) -> Score {
    (KNIGHT_MOVES_BITBOARDS[attacked_square as usize] & knight_bitboard).count_ones() as Score
}

#[inline(always)]
pub fn pawn_shield(position: &Position) -> Score {
    let mut score: Score = 0;
    let ks = position.pieces[WHITE as usize].king_square as usize;
    if ks <= 7 {
        let mask = WHITE_KING_SHIELD_MASK[ks];
        let pawns_shielding = (mask & position.pieces[WHITE as usize].pawn_bitboard).count_ones() as Score;
        score += pawns_shielding * VALUE_ONE_PAWN_SHIELDING;
    }
    let ks = position.pieces[BLACK as usize].king_square as usize;
    if ks >= 56 {
        let mask = WHITE_KING_SHIELD_MASK[ks % 8] << 40;
        let pawns_shielding = (mask & position.pieces[BLACK as usize].pawn_bitboard).count_ones() as Score;
        score -= pawns_shielding * VALUE_ONE_PAWN_SHIELDING;
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

#[inline(always)]
pub fn trade_piece_bonus_when_more_material(material_difference: Score, white_piece_values: Score, black_piece_values: Score, white_pawn_values: Score, black_pawn_values: Score) -> Score {
    linear_scale(
        if material_difference > 0 { black_piece_values + black_pawn_values } else { white_piece_values + white_pawn_values },
        0,
        TOTAL_PIECE_VALUE_PER_SIDE_AT_START,
        30 * material_difference / 100,
        0)
}

#[inline(always)]
pub fn trade_pawn_bonus_when_more_material(material_difference: Score, white_pawn_values: Score, black_pawn_values: Score) -> Score {
    linear_scale(
        if material_difference > 0 { white_pawn_values } else { black_pawn_values },
        0,
        PAWN_TRADE_BONUS_MAX,
        30 * material_difference / 100,
        0)
}
