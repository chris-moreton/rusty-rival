use crate::bitboards::{bit, epsbit, KING_MOVES_BITBOARDS, PAWN_MOVES_CAPTURE};
use crate::engine_constants::{PAWN_VALUE_AVERAGE, QUEEN_VALUE_AVERAGE};
use crate::evaluate::evaluate;
use crate::make_move::{make_move_in_place, unmake_move};
use crate::move_constants::{
    PIECE_MASK_BISHOP, PIECE_MASK_FULL, PIECE_MASK_KING, PIECE_MASK_QUEEN, PIECE_MASK_ROOK,
    PROMOTION_FULL_MOVE_MASK, PROMOTION_QUEEN_MOVE_MASK, PROMOTION_SQUARES,
};
use crate::move_scores::{attacker_bonus, piece_value, PAWN_ATTACKER_BONUS};
use crate::moves::{generate_diagonal_slider_moves, generate_knight_moves, generate_straight_slider_moves, is_check};
use crate::types::{
    pv_single, Bitboard, Move, MoveList, MoveScoreList, PathScore, Pieces, Position, Score, SearchState, Square, Window, BLACK, WHITE,
};
use crate::utils::{from_square_mask, send_info, to_square_part};
use crate::{add_moves, check_time, get_and_unset_lsb, opponent};
use std::time::Instant;
use crate::see::{captured_piece_value_see, see};


#[inline(always)]
pub fn quiesce_moves(position: &Position) -> MoveList {
    let mut move_list = Vec::with_capacity(4);

    let all_pieces = position.pieces[WHITE as usize].all_pieces_bitboard | position.pieces[BLACK as usize].all_pieces_bitboard;
    let friendly = position.pieces[position.mover as usize];
    let valid_destinations = position.pieces[opponent!(position.mover) as usize].all_pieces_bitboard;

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
pub fn score_quiesce_move(position: &Position, m: Move, enemy: &Pieces, _search_state: &mut SearchState) -> Score {
    let to_square = to_square_part(m);

    let mut score = if m & PROMOTION_FULL_MOVE_MASK == PROMOTION_QUEEN_MOVE_MASK {
        QUEEN_VALUE_AVERAGE
    } else {
        0
    };

    score += if enemy.all_pieces_bitboard & bit(to_square) != 0 {
        piece_value(enemy, to_square) - attacker_bonus(m & PIECE_MASK_FULL)
    } else if to_square == position.en_passant_square {
        PAWN_VALUE_AVERAGE + PAWN_ATTACKER_BONUS
    } else {
        0
    };

    score
}

#[inline(always)]
pub fn quiesce(position: &mut Position, depth: u8, ply: u8, window: Window, search_state: &mut SearchState) -> PathScore {
    // Check stop flag at TOP before any moves are made - safe to return here
    if search_state.stop {
        return (pv_single(0), 0);
    }

    check_time!(search_state);
    if search_state.stop {
        return (pv_single(0), 0);
    }
    search_state.nodes += 1;

    let eval = evaluate(position);

    if depth == 0 || eval >= window.1 {
        return (pv_single(0), eval);
    }

    let mut alpha = window.0.max(eval);
    let mut best_move: Move = 0;

    let ms = quiesce_moves(position);

    // If there are no legal moves, return the evaluation score
    if ms.is_empty() {
        return (pv_single(0), eval);
    }

    let mut move_scores: MoveScoreList = vec![];

    for &m in &ms {
        let score = score_quiesce_move(position, m, &position.pieces[opponent!(position.mover) as usize], search_state);
        move_scores.push((m, score));
    }

    move_scores.sort_unstable_by(|a, b| b.1.cmp(&a.1));

    for (m, _) in move_scores {
        let old_mover = position.mover;
        let see_value = captured_piece_value_see(position, m);
        let unmake = make_move_in_place(position, m);

        if !is_check(position, old_mover) && see(see_value, bit(to_square_part(m)), position) > 0 {
            let score = -quiesce(position, depth - 1, ply + 1, (-window.1, -alpha), search_state).1;

            unmake_move(position, m, &unmake);

            check_time!(search_state);
            if search_state.stop {
                break;
            }

            if score >= window.1 {
                return (pv_single(m), window.1);
            }
            if score > alpha {
                alpha = score;
                best_move = m;
            }
        } else {
            unmake_move(position, m, &unmake);
        }
    }

    (pv_single(best_move), alpha)
}
