use crate::bitboards::{bit, epsbit, KING_MOVES_BITBOARDS, PAWN_MOVES_CAPTURE, RANK_2_BITS, RANK_7_BITS};
use crate::engine_constants::{PAWN_VALUE, QUEEN_VALUE};
use crate::evaluate::evaluate;
use crate::make_move::make_move;
use crate::move_constants::{
    EN_PASSANT_NOT_AVAILABLE, PIECE_MASK_BISHOP, PIECE_MASK_FULL, PIECE_MASK_KING, PIECE_MASK_QUEEN, PIECE_MASK_ROOK,
    PROMOTION_FULL_MOVE_MASK, PROMOTION_QUEEN_MOVE_MASK, PROMOTION_SQUARES,
};
use crate::move_scores::{attacker_bonus, piece_value, PAWN_ATTACKER_BONUS};
use crate::moves::{generate_diagonal_slider_moves, generate_knight_moves, generate_straight_slider_moves, is_check};
use crate::types::{
    Bitboard, Move, MoveList, MoveScoreList, PathScore, Pieces, Position, Score, SearchState, Square, Window, BLACK, WHITE,
};
use crate::utils::{captured_piece_value, from_square_mask, send_info, to_square_part};
use crate::{add_moves, check_time, get_and_unset_lsb, opponent};
use std::time::Instant;

#[inline(always)]
pub fn quiesce_moves(position: &Position) -> MoveList {
    let mut move_list = Vec::with_capacity(4);

    let all_pieces = position.pieces[WHITE as usize].all_pieces_bitboard | position.pieces[BLACK as usize].all_pieces_bitboard;
    let friendly = position.pieces[position.mover as usize];
    let enemy = position.pieces[opponent!(position.mover) as usize];
    let valid_destinations = enemy.all_pieces_bitboard
        | (if position.en_passant_square != EN_PASSANT_NOT_AVAILABLE {
            bit(position.en_passant_square)
        } else {
            0
        });

    generate_capture_pawn_moves(position, &mut move_list, position.mover as usize, friendly.pawn_bitboard);
    generate_knight_moves(&mut move_list, valid_destinations, friendly.knight_bitboard);
    generate_diagonal_slider_moves(
        friendly.bishop_bitboard,
        all_pieces,
        &mut move_list,
        valid_destinations,
        PIECE_MASK_BISHOP,
    );
    generate_straight_slider_moves(
        friendly.rook_bitboard,
        all_pieces,
        &mut move_list,
        valid_destinations,
        PIECE_MASK_ROOK,
    );
    generate_straight_slider_moves(
        friendly.queen_bitboard,
        all_pieces,
        &mut move_list,
        valid_destinations,
        PIECE_MASK_QUEEN,
    );
    generate_diagonal_slider_moves(
        friendly.queen_bitboard,
        all_pieces,
        &mut move_list,
        valid_destinations,
        PIECE_MASK_QUEEN,
    );

    add_moves!(
        move_list,
        from_square_mask(friendly.king_square) | PIECE_MASK_KING,
        KING_MOVES_BITBOARDS[friendly.king_square as usize] & valid_destinations
    );

    move_list
}

#[inline(always)]
fn generate_capture_pawn_moves(position: &Position, move_list: &mut Vec<Move>, colour_index: usize, mut from_squares: Bitboard) {
    while from_squares != 0 {
        let from_square = get_and_unset_lsb!(from_squares);

        let enemy_pawns_capture_bitboard =
            position.pieces[opponent!(position.mover) as usize].all_pieces_bitboard | epsbit(position.en_passant_square);

        let mut to_bitboard = PAWN_MOVES_CAPTURE[colour_index][from_square as usize] & enemy_pawns_capture_bitboard;

        let fsm = from_square_mask(from_square);
        let is_promotion = to_bitboard & PROMOTION_SQUARES != 0;
        while to_bitboard != 0 {
            let base_move = fsm | get_and_unset_lsb!(to_bitboard) as Move;
            if is_promotion {
                move_list.push(base_move | PROMOTION_QUEEN_MOVE_MASK);
            } else {
                move_list.push(base_move);
            }
        }
    }
}

#[inline(always)]
pub fn score_quiesce_move(position: &Position, m: Move, enemy: &Pieces) -> Score {
    let to_square = to_square_part(m);

    let mut score = if m & PROMOTION_FULL_MOVE_MASK == PROMOTION_QUEEN_MOVE_MASK {
        QUEEN_VALUE
    } else {
        0
    };

    score += if enemy.all_pieces_bitboard & bit(to_square) != 0 {
        piece_value(enemy, to_square) + attacker_bonus(m & PIECE_MASK_FULL)
    } else if to_square == position.en_passant_square {
        PAWN_VALUE + PAWN_ATTACKER_BONUS
    } else {
        0
    };

    score
}

#[inline(always)]
pub fn quiesce(position: &Position, depth: u8, ply: u8, window: Window, search_state: &mut SearchState) -> PathScore {
    check_time!(search_state);

    search_state.nodes += 1;

    let eval = evaluate(position);

    if depth == 0 {
        return (vec![0], eval);
    }

    let beta = window.1;

    if eval >= beta {
        return (vec![0], eval);
    }

    let promote_from_rank = if position.mover == WHITE { RANK_7_BITS } else { RANK_2_BITS };
    let mut delta = QUEEN_VALUE;
    if position.pieces[position.mover as usize].pawn_bitboard & promote_from_rank != 0 {
        delta += QUEEN_VALUE - PAWN_VALUE
    }

    let mut alpha = window.0;

    if eval < alpha - delta {
        return (vec![0], alpha);
    }

    if alpha < eval {
        alpha = eval;
    }

    let ms = quiesce_moves(position);
    if !ms.is_empty() {
        let move_scores = if ms.len() > 1 {
            let enemy = &position.pieces[opponent!(position.mover) as usize];
            let mut move_scores: MoveScoreList = ms.into_iter().map(|m| (m, score_quiesce_move(position, m, enemy))).collect();

            move_scores.sort_by(|(_, a), (_, b)| b.cmp(a));
            move_scores
        } else {
            vec![(ms[0], 0)]
        };

        for ms in move_scores {
            let m = ms.0;

            if eval + captured_piece_value(position, m) > alpha {
                let mut new_position = *position;
                make_move(position, m, &mut new_position);
                if !is_check(&new_position, position.mover) {
                    let score = -quiesce(&new_position, depth - 1, ply + 1, (-beta, -alpha), search_state).1;
                    check_time!(search_state);
                    if score >= beta {
                        return (vec![m], beta);
                    }
                    if score > alpha {
                        alpha = score;
                    }
                }
            }
        }
    }

    (vec![0], alpha)
}
