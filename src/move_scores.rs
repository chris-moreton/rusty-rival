use crate::bitboards::{bit, BLACK_PASSED_PAWN_MASK, WHITE_PASSED_PAWN_MASK};
use crate::engine_constants::{BISHOP_VALUE_AVERAGE, KNIGHT_VALUE_AVERAGE, PAWN_VALUE_AVERAGE, QUEEN_VALUE_AVERAGE, ROOK_VALUE_AVERAGE};

use crate::move_constants::{
    PIECE_MASK_BISHOP, PIECE_MASK_FULL, PIECE_MASK_KING, PIECE_MASK_KNIGHT, PIECE_MASK_PAWN, PIECE_MASK_QUEEN, PIECE_MASK_ROOK,
    PROMOTION_BISHOP_MOVE_MASK, PROMOTION_FULL_MOVE_MASK, PROMOTION_KNIGHT_MOVE_MASK, PROMOTION_ROOK_MOVE_MASK,
};
use crate::search::piece_index_12;

use crate::types::{Move, Pieces, Position, Score, SearchState, Square, BLACK, WHITE};
use crate::utils::{from_square_part, linear_scale, to_square_part};

pub const BIT_FLIPPED_HORIZONTAL_AXIS: [Square; 64] = [
    56, 57, 58, 59, 60, 61, 62, 63, 48, 49, 50, 51, 52, 53, 54, 55, 40, 41, 42, 43, 44, 45, 46, 47, 32, 33, 34, 35, 36, 37, 38, 39, 24, 25,
    26, 27, 28, 29, 30, 31, 16, 17, 18, 19, 20, 21, 22, 23, 8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7,
];

pub const KNIGHT_STAGE_MATERIAL_LOW: Score = KNIGHT_VALUE_AVERAGE + 8 * PAWN_VALUE_AVERAGE;
pub const KNIGHT_STAGE_MATERIAL_HIGH: Score =
    QUEEN_VALUE_AVERAGE + 2 * ROOK_VALUE_AVERAGE + 2 * BISHOP_VALUE_AVERAGE + 6 * PAWN_VALUE_AVERAGE;
pub const PAWN_STAGE_MATERIAL_LOW: Score = ROOK_VALUE_AVERAGE;
pub const PAWN_STAGE_MATERIAL_HIGH: Score = QUEEN_VALUE_AVERAGE + 2 * ROOK_VALUE_AVERAGE + 2 * BISHOP_VALUE_AVERAGE;
pub const OPENING_PHASE_MATERIAL: Score = (TOTAL_PIECE_VALUE_PER_SIDE_AT_START as f32 * 0.8) as Score;
pub const TOTAL_PIECE_VALUE_PER_SIDE_AT_START: Score =
    KNIGHT_VALUE_AVERAGE * 2 + BISHOP_VALUE_AVERAGE * 2 + ROOK_VALUE_AVERAGE * 2 + QUEEN_VALUE_AVERAGE;

pub const PAWN_ATTACKER_BONUS: Score = 300;

const GOOD_CAPTURE_START: Score = 3000;
const MATE_KILLER_SCORE: Score = 1000;
const CURRENT_PLY_KILLER_1: Score = 750;
const CURRENT_PLY_KILLER_2: Score = 400;
const HISTORY_TOP: Score = 500;
const DISTANT_KILLER_1: Score = 300;
const DISTANT_KILLER_2: Score = 200;
const GOOD_CAPTURE_BONUS: Score = 300;
const HISTORY_START: Score = 0;
const PAWN_PUSH_1: Score = 250;
const PAWN_PUSH_2: Score = 50;

#[inline(always)]
pub fn attacker_bonus(piece: Move) -> Score {
    match piece {
        PIECE_MASK_PAWN => PAWN_ATTACKER_BONUS,
        PIECE_MASK_KNIGHT => 250,
        PIECE_MASK_BISHOP => 200,
        PIECE_MASK_ROOK => 150,
        PIECE_MASK_QUEEN => 100,
        PIECE_MASK_KING => 50,
        _ => {
            panic!("Expected piece")
        }
    }
}

#[inline(always)]
pub fn attacker_value(piece: Move) -> Score {
    match piece {
        PIECE_MASK_PAWN => PAWN_VALUE_AVERAGE,
        PIECE_MASK_KNIGHT => KNIGHT_VALUE_AVERAGE,
        PIECE_MASK_BISHOP => BISHOP_VALUE_AVERAGE,
        PIECE_MASK_ROOK => ROOK_VALUE_AVERAGE,
        PIECE_MASK_QUEEN => QUEEN_VALUE_AVERAGE,
        PIECE_MASK_KING => 10000,
        _ => {
            panic!("Expected piece")
        }
    }
}

#[inline(always)]
pub fn score_move(position: &Position, m: Move, search_state: &SearchState, ply: usize, enemy: &Pieces) -> Score {
    let to_square = to_square_part(m);

    let score = if enemy.all_pieces_bitboard & bit(to_square) != 0 {
        // let mut new_position = *position;
        // make_see_move(m, &mut new_position);
        // GOOD_CAPTURE_START + see(captured_piece_value_see(position, m), bit(to_square_part(m)), &new_position)

        let pv = piece_value(enemy, to_square);
        GOOD_CAPTURE_START
            + pv
            + attacker_bonus(m & PIECE_MASK_FULL)
            + if pv < attacker_value(m & PIECE_MASK_FULL) {
                GOOD_CAPTURE_BONUS
            } else {
                0
            }
    } else if m & PROMOTION_FULL_MOVE_MASK != 0 {
        let mask = m & PROMOTION_FULL_MOVE_MASK;
        if mask == PROMOTION_ROOK_MOVE_MASK {
            3
        } else if mask == PROMOTION_BISHOP_MOVE_MASK {
            2
        } else if mask == PROMOTION_KNIGHT_MOVE_MASK {
            1
        } else {
            GOOD_CAPTURE_START + QUEEN_VALUE_AVERAGE
        }
    } else if to_square == position.en_passant_square {
        GOOD_CAPTURE_START + PAWN_VALUE_AVERAGE + PAWN_ATTACKER_BONUS
    } else if m == search_state.mate_killer[ply] {
        MATE_KILLER_SCORE
    } else {
        let killer_moves = search_state.killer_moves[ply];
        if m == killer_moves[0] {
            CURRENT_PLY_KILLER_1
        } else if m == killer_moves[1] {
            CURRENT_PLY_KILLER_2
        } else if ply > 2 {
            let killer_moves = search_state.killer_moves[ply - 2];
            if m == killer_moves[0] {
                DISTANT_KILLER_1
            } else if m == killer_moves[1] {
                DISTANT_KILLER_2
            } else {
                0
            }
        } else {
            0
        }
    };

    let pawn_push_score = if m & PIECE_MASK_FULL == PIECE_MASK_PAWN {
        let to_square = to_square_part(m);
        if to_square >= 48 || to_square <= 15 {
            PAWN_PUSH_1
        } else if position.mover == WHITE {
            if (40..=47).contains(&to_square)
                && position.pieces[BLACK as usize].pawn_bitboard & WHITE_PASSED_PAWN_MASK[to_square as usize] == 0
            {
                PAWN_PUSH_2
            } else {
                0
            }
        } else if (16..=23).contains(&to_square)
            && position.pieces[WHITE as usize].pawn_bitboard & BLACK_PASSED_PAWN_MASK[to_square as usize] == 0
        {
            PAWN_PUSH_2
        } else {
            0
        }
    } else {
        0
    };

    score + pawn_push_score + history_score(position, m, search_state, to_square)
}

#[inline(always)]
pub fn history_score(position: &Position, m: Move, search_state: &SearchState, to_square: Square) -> Score {
    linear_scale(
        search_state.history_moves[piece_index_12(position, m)][from_square_part(m) as usize][to_square as usize],
        0,
        search_state.highest_history_score,
        HISTORY_START as i64,
        HISTORY_TOP as i64,
    ) as Score
}

#[inline(always)]
pub fn piece_value(pieces: &Pieces, sq: Square) -> Score {
    let bb = bit(sq);
    if pieces.pawn_bitboard & bb != 0 {
        return PAWN_VALUE_AVERAGE;
    }
    if pieces.knight_bitboard & bb != 0 {
        return KNIGHT_VALUE_AVERAGE;
    }
    if pieces.rook_bitboard & bb != 0 {
        return ROOK_VALUE_AVERAGE;
    }
    if pieces.queen_bitboard & bb != 0 {
        return QUEEN_VALUE_AVERAGE;
    }
    if pieces.bishop_bitboard & bb != 0 {
        return BISHOP_VALUE_AVERAGE;
    }
    0
}
