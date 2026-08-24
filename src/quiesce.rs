use crate::bitboards::{bit, epsbit, KING_MOVES_BITBOARDS, PAWN_MOVES_CAPTURE, RANK_2_BITS, RANK_7_BITS};
use crate::engine_constants::{DELTA_MARGIN, PAWN_VALUE_AVERAGE, QUEEN_VALUE_AVERAGE};
use crate::evaluate::evaluate_position;
use crate::move_constants::{
    PIECE_MASK_BISHOP, PIECE_MASK_FULL, PIECE_MASK_KING, PIECE_MASK_PAWN, PIECE_MASK_QUEEN, PIECE_MASK_ROOK, PROMOTION_FULL_MOVE_MASK,
    PROMOTION_QUEEN_MOVE_MASK, PROMOTION_SQUARES,
};
use crate::move_scores::{attacker_bonus, piece_value, PAWN_ATTACKER_BONUS};
use crate::moves::{
    generate_check_evasions, generate_diagonal_slider_moves, generate_knight_moves, generate_straight_slider_moves, is_check,
};
use crate::search::MATE_SCORE;
use crate::see::{captured_piece_value_see, static_exchange_evaluation_with_value};
use crate::types::{
    is_stopped, pv_single, set_stop, Bitboard, Move, MoveList, MoveScoreArray, PathScore, Pieces, Position, Score, SearchState, Square,
    Window, BLACK, WHITE,
};
use crate::utils::{from_square_mask, send_info, to_square_part};
use crate::{add_moves, check_time, get_and_unset_lsb, opponent};
use std::time::Instant;

#[inline(always)]
pub fn quiesce_moves(position: &Position) -> MoveList {
    let mut move_list = MoveList::new();

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

    // Quiet queen promotions: a push to the last rank is as tactical as any
    // capture and must be visible at the horizon
    let promo_rank_pawns = friendly.pawn_bitboard & if position.mover == WHITE { RANK_7_BITS } else { RANK_2_BITS };
    if promo_rank_pawns != 0 {
        let empty = !all_pieces;
        let mut to_bitboard = if position.mover == WHITE {
            promo_rank_pawns << 8
        } else {
            promo_rank_pawns >> 8
        } & empty;
        while to_bitboard != 0 {
            let to_square = get_and_unset_lsb!(to_bitboard);
            let from_square = if position.mover == WHITE { to_square - 8 } else { to_square + 8 };
            move_list.push(from_square_mask(from_square) | to_square as Move | PROMOTION_QUEEN_MOVE_MASK);
        }
    }

    move_list
}

#[inline(always)]
fn generate_capture_pawn_moves(position: &Position, move_list: &mut MoveList, colour_index: usize, mut from_squares: Bitboard) {
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
        // MVV-LVA: attacker_bonus is scaled cheap-attacker-high (pawn 300 .. king 50),
        // so it is ADDED, matching the EP branch below and score_move_with_see.
        // (NET-366: a 2022 refactor flipped this to `-`, searching the most
        // valuable attacker first for four years.)
        piece_value(enemy, to_square) + attacker_bonus(m & PIECE_MASK_FULL)
    } else if m & PIECE_MASK_FULL == PIECE_MASK_PAWN && to_square == position.en_passant_square {
        // The pawn guard matters when in check: evasions include quiet king and
        // slider moves, and one landing on the EP square was being scored as a
        // capture worth 475 (NET-374). Matches see.rs:97 and utils.rs:39.
        PAWN_VALUE_AVERAGE + PAWN_ATTACKER_BONUS
    } else {
        0
    };

    score
}

use crate::search::{make_move_nnue, unmake_move_nnue};

#[allow(clippy::only_used_in_recursion)]
pub fn quiesce(
    position: &mut Position,
    depth: u8,
    ply: u8,
    window: Window,
    search_state: &mut SearchState,
    // Whether the side to move is in check, when the caller already knows it
    // (NET-355) - e.g. search's depth==0 forward or the !in_check razor path
    known_in_check: Option<bool>,
) -> PathScore {
    // Check stop flag at TOP before any moves are made - safe to return here
    if is_stopped(&search_state.stop) {
        return (pv_single(0), 0);
    }

    check_time!(search_state);
    if is_stopped(&search_state.stop) {
        return (pv_single(0), 0);
    }
    search_state.nodes += 1;
    search_state.qnodes += 1;

    let in_check = known_in_check.unwrap_or_else(|| is_check(position, position.mover));

    // NET-352: when in check above the depth cap the stand-pat eval is provably
    // never read - stand-pat and delta pruning are !in_check-gated, alpha starts
    // at window.0, and the empty-evasion path returns a mate score - so skip the
    // NNUE forward pass entirely. (The lazy accumulator chain computes this ply
    // as an intermediate if a descendant evaluates, so nothing downstream changes.)
    let eval = if in_check && depth > 0 {
        0
    } else {
        let mut e = evaluate_position(position, search_state);
        if search_state.eval_noise > 0 {
            let noise_bits = (position.zobrist_lock >> 17) as i32;
            let noise_max = search_state.eval_noise;
            e += (noise_bits % (2 * noise_max + 1)) - noise_max;
        }
        e
    };

    // Depth cap terminates evasion chains; otherwise stand-pat is only a
    // valid bound when not in check
    if depth == 0 || (!in_check && eval >= window.1) {
        return (pv_single(0), eval);
    }

    let mut alpha = if in_check { window.0 } else { window.0.max(eval) };
    let mut best_move: Move = 0;

    // In check: every evasion must be considered, not just captures
    let ms = if in_check {
        generate_check_evasions(position)
    } else {
        quiesce_moves(position)
    };

    if ms.is_empty() {
        // No pseudo-legal evasions while in check = mated at the horizon
        return if in_check {
            (pv_single(0), -MATE_SCORE + ply as Score)
        } else {
            (pv_single(0), eval)
        };
    }

    let mut move_scores: MoveScoreArray = MoveScoreArray::new();

    for &m in &ms {
        let score = score_quiesce_move(position, m, &position.pieces[opponent!(position.mover) as usize], search_state);
        move_scores.push((m, score));
    }

    // NOTE (NET-352): replacing this sort with lazy selection
    // (pick_high_score_move) changes tie ordering vs sort_unstable and altered
    // the bench signature (+1.6% nodes) - not behavior-neutral, so it needs its
    // own SPRT rather than riding a speed-only release.
    move_scores.sort_unstable_by_key(|b| std::cmp::Reverse(b.1));

    let mut legal_move_count = 0;

    for (m, _) in move_scores {
        let is_promotion = m & PROMOTION_FULL_MOVE_MASK != 0;
        let see_value = captured_piece_value_see(position, m);

        // Evasions and promotions are never delta-pruned or SEE-gated
        if !in_check && !is_promotion {
            // Delta pruning: if capturing this piece can't possibly raise
            // alpha, skip it
            if eval + see_value + DELTA_MARGIN < alpha {
                continue;
            }

            // SEE gate BEFORE the move is made (NET-352): a losing capture
            // must not pay make/unmake plus NNUE bookkeeping just to be
            // discarded. The compact SEE state reads the same
            // piece-bitboard/king/mover/EP subset as the old copy here.
            // (Equal exchanges pass and are searched, as before.)
            if static_exchange_evaluation_with_value(position, m, see_value) < 0 {
                continue;
            }
        }

        let old_mover = position.mover;
        let unmake = make_move_nnue(position, m, search_state);

        if !is_check(position, old_mover) {
            legal_move_count += 1;

            let score = -quiesce(position, depth - 1, ply + 1, (-window.1, -alpha), search_state, None).1;

            unmake_move_nnue(position, m, &unmake, search_state);

            check_time!(search_state);
            if is_stopped(&search_state.stop) {
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
            unmake_move_nnue(position, m, &unmake, search_state);
        }
    }

    // All evasions were illegal (pins) - the check is mate
    if in_check && legal_move_count == 0 {
        return (pv_single(0), -MATE_SCORE + ply as Score);
    }

    (pv_single(best_move), alpha)
}
