use crate::bitboards::bit;
use crate::make_move::{en_passant_captured_piece_square, make_move_with_promotion, make_pawn_capture_move};
use crate::moves::{is_check, see_moves};
use crate::types::{Bitboard, BLACK, Move, Position, Score, Square, WHITE};
use crate::utils::{captured_piece_value, from_square_part, to_square_part};
use std::cmp::min;
use crate::hash::{ZOBRIST_KEYS_PIECES, ZOBRIST_PIECE_INDEX_BISHOP, ZOBRIST_PIECE_INDEX_KNIGHT, ZOBRIST_PIECE_INDEX_PAWN, ZOBRIST_PIECE_INDEX_QUEEN, ZOBRIST_PIECE_INDEX_ROOK};
use crate::move_constants::{EN_PASSANT_NOT_AVAILABLE, PIECE_MASK_FULL, PIECE_MASK_PAWN, PIECE_MASK_KING, PIECE_MASK_KNIGHT, PIECE_MASK_QUEEN, PIECE_MASK_BISHOP, PIECE_MASK_ROOK, PROMOTION_FULL_MOVE_MASK, PROMOTION_KNIGHT_MOVE_MASK, PROMOTION_QUEEN_MOVE_MASK, PROMOTION_BISHOP_MOVE_MASK, PROMOTION_ROOK_MOVE_MASK, EN_PASSANT_CAPTURE_MASK};
use crate::opponent;


#[inline(always)]
pub fn static_exchange_evaluation(position: &Position, mv: Move) -> Score {
    let mut new_position = *position;
    make_see_move(mv, &mut new_position);
    if is_check(&new_position, position.mover) {
        0
    } else {
        see(captured_piece_value(position, mv), bit(to_square_part(mv)), &new_position)
    }
}

#[inline(always)]
fn see(score: Score, capture_square: Bitboard, position: &Position) -> Score {
    for m in see_moves(position, capture_square) {
        let mut new_position = *position;
        make_see_move(m, &mut new_position);
        if !is_check(&new_position, position.mover) {
            return min(score, score - see(captured_piece_value(position, m), capture_square, &new_position));
        }
    }

    score
}

#[inline(always)]
pub fn make_see_move(mv: Move, new_position: &mut Position) {
    let from = from_square_part(mv);
    let to = to_square_part(mv);

    let piece_mask = mv & PIECE_MASK_FULL;

    match piece_mask {
        PIECE_MASK_PAWN => {

            if mv & PROMOTION_FULL_MOVE_MASK != 0 {
                make_see_move_with_promotion(new_position, from, to);
            } else {
                make_see_pawn_capture_move(new_position, from, to);
            }
        }
        _ => {
            let bit_to = bit(to);

            if (new_position.pieces[WHITE as usize].all_pieces_bitboard | new_position.pieces[BLACK as usize].all_pieces_bitboard) & bit_to != 0 {
                let enemy = &mut new_position.pieces[opponent!(new_position.mover) as usize];

                if enemy.pawn_bitboard & bit_to != 0 {
                    enemy.pawn_bitboard &= !bit_to;
                } else if enemy.knight_bitboard & bit_to != 0 {
                    enemy.knight_bitboard &= !bit_to;
                } else if enemy.rook_bitboard & bit_to != 0 {
                    enemy.rook_bitboard &= !bit_to;
                } else if enemy.bishop_bitboard & bit_to != 0 {
                    enemy.bishop_bitboard &= !bit_to;
                } else if enemy.queen_bitboard & bit_to != 0 {
                    enemy.queen_bitboard &= !bit_to;
                }

                enemy.all_pieces_bitboard &= !bit_to;
            }

            let switch = bit(from) | bit_to;
            new_position.pieces[new_position.mover as usize].all_pieces_bitboard ^= switch;

            match piece_mask {
                PIECE_MASK_KNIGHT => new_position.pieces[new_position.mover as usize].knight_bitboard ^= switch,
                PIECE_MASK_BISHOP => new_position.pieces[new_position.mover as usize].bishop_bitboard ^= switch,
                PIECE_MASK_ROOK => new_position.pieces[new_position.mover as usize].rook_bitboard ^= switch,
                PIECE_MASK_QUEEN => new_position.pieces[new_position.mover as usize].queen_bitboard ^= switch,
                PIECE_MASK_KING => new_position.pieces[new_position.mover as usize].king_square = to,
                _ => panic!("Piece panic"),
            }
        }
    }

    new_position.mover ^= 1;
}

#[inline(always)]
pub fn make_see_move_with_promotion(position: &mut Position, from: Square, to: Square) {
    let bit_to = bit(to);
    let bit_from = bit(from);

    position.pieces[position.mover as usize].queen_bitboard |= bit_to;
    position.pieces[position.mover as usize].pawn_bitboard ^= bit_from;

    let opponent = opponent!(position.mover) as usize;

    position.pieces[position.mover as usize].all_pieces_bitboard ^= bit_from | bit_to;

    if position.pieces[opponent].all_pieces_bitboard & bit_to != 0 {
        let enemy = &mut position.pieces[opponent];

        if enemy.knight_bitboard & bit_to != 0 {
            enemy.knight_bitboard &= !bit_to;
        } else if enemy.rook_bitboard & bit_to != 0 {
            enemy.rook_bitboard &= !bit_to;
        } else if enemy.bishop_bitboard & bit_to != 0 {
            enemy.bishop_bitboard &= !bit_to;
        } else if enemy.queen_bitboard & bit_to != 0 {
            enemy.queen_bitboard &= !bit_to;
        }
        enemy.all_pieces_bitboard &= !bit_to;
    }

    position.en_passant_square = EN_PASSANT_NOT_AVAILABLE;
}

#[inline(always)]
pub fn make_see_pawn_capture_move(position: &mut Position, from: Square, to: Square) {
    let opponent = opponent!(position.mover) as usize;
    let enemy = &mut position.pieces[opponent];

    let bit_to = bit(to);

    if position.en_passant_square == to {
        let pawn_off = EN_PASSANT_CAPTURE_MASK[to as usize];

        enemy.pawn_bitboard &= pawn_off;
        enemy.all_pieces_bitboard &= pawn_off;
    } else {
        if enemy.pawn_bitboard & bit_to != 0 {
            enemy.pawn_bitboard &= !bit_to;
        }
        if enemy.knight_bitboard & bit_to != 0 {
            enemy.knight_bitboard &= !bit_to;
        }
        if enemy.rook_bitboard & bit_to != 0 {
            enemy.rook_bitboard &= !bit_to;
        }
        if enemy.bishop_bitboard & bit_to != 0 {
            enemy.bishop_bitboard &= !bit_to;
        }
        if enemy.queen_bitboard & bit_to != 0 {
            enemy.queen_bitboard &= !bit_to;
        }
        enemy.all_pieces_bitboard &= !bit_to;
    }

    let switch = bit(from) | bit_to;
    position.pieces[position.mover as usize].all_pieces_bitboard ^= switch;
    position.pieces[position.mover as usize].pawn_bitboard ^= switch;

    position.en_passant_square = EN_PASSANT_NOT_AVAILABLE;
}