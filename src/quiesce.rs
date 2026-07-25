use crate::bitboards::{bit, epsbit, KING_MOVES_BITBOARDS, PAWN_MOVES_CAPTURE, RANK_2_BITS, RANK_7_BITS};
use crate::engine_constants::{PAWN_VALUE_AVERAGE, QUEEN_VALUE_AVERAGE};
use crate::evaluate::evaluate_position;
use crate::move_constants::{
    PIECE_MASK_BISHOP, PIECE_MASK_FULL, PIECE_MASK_KING, PIECE_MASK_QUEEN, PIECE_MASK_ROOK, PROMOTION_FULL_MOVE_MASK,
    PROMOTION_QUEEN_MOVE_MASK, PROMOTION_SQUARES,
};
use crate::move_scores::{attacker_bonus, piece_value, PAWN_ATTACKER_BONUS};
use crate::moves::{
    generate_check_evasions, generate_diagonal_slider_moves, generate_knight_moves, generate_straight_slider_moves, is_check, verify_move,
};
use crate::search::{MATE_SCORE, MATE_START};
use crate::see::{captured_piece_value_see, see};
use crate::types::BoundType::{Exact, Lower, Upper};
use crate::types::{
    is_stopped, pv_single, set_stop, Bitboard, BoundType, HashEntry, Move, MoveList, MoveScoreArray, PathScore, Pieces, Position, Score,
    SearchState, Square, Window, BLACK, STATIC_EVAL_NONE, WHITE,
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
        piece_value(enemy, to_square) - attacker_bonus(m & PIECE_MASK_FULL)
    } else if to_square == position.en_passant_square {
        PAWN_VALUE_AVERAGE + PAWN_ATTACKER_BONUS
    } else {
        0
    };

    score
}

use crate::search::{make_move_nnue, unmake_move_nnue};

/// Store a quiescence result in the shared TT at height 0.
///
/// Height 0 is deliberate and load-bearing: `search()` only probes for a cutoff
/// when `hash_entry.height >= depth`, and it never reaches the probe at depth 0
/// (it tail-calls `quiesce` first). So a qsearch entry can never satisfy a
/// depth >= 1 cutoff, and cannot pollute the main search. It also means the
/// replacement guard below only ever displaces other height-0 entries or entries
/// left over from an earlier search generation - deep entries are never clobbered.
#[inline(always)]
#[allow(clippy::too_many_arguments)]
fn store_quiesce(
    search_state: &mut SearchState,
    index: usize,
    position: &Position,
    score: Score,
    bound: BoundType,
    mv: Move,
    ply: u8,
    static_eval: Score,
) {
    let (existing_height, existing_version, _) = search_state.hash_table.entry_meta(index);
    let table_version = search_state.hash_table.version();
    // Height 0 only ever displaces another height-0 entry, or one left over
    // from an earlier search generation - deep entries are never clobbered.
    if existing_height == 0 || table_version != existing_version {
        search_state.hash_table.store(
            index,
            HashEntry {
                // Mate scores are stored relative to the storing ply, exactly as
                // search() does, so a probe at a different ply can re-derive them
                score: match score {
                    x if x > MATE_START => score + ply as Score,
                    x if x < -MATE_START => score - ply as Score,
                    _ => score,
                },
                version: table_version,
                height: 0,
                mv,
                bound,
                lock: position.zobrist_lock,
                static_eval,
            },
        );
    }
}

#[allow(clippy::only_used_in_recursion)]
pub fn quiesce(position: &mut Position, depth: u8, ply: u8, window: Window, search_state: &mut SearchState) -> PathScore {
    // Check stop flag at TOP before any moves are made - safe to return here
    if is_stopped(&search_state.stop) {
        return (pv_single(0), 0);
    }

    check_time!(search_state);
    if is_stopped(&search_state.stop) {
        return (pv_single(0), 0);
    }
    search_state.nodes += 1;

    // TT probe. No height gate: any stored entry was produced by a search at
    // least as deep as this quiescence node, so it is never worse than what we
    // would compute here. A hit also saves the NNUE evaluate_position() below,
    // which is the dominant per-node cost in qsearch.
    let hash_index: usize = (position.zobrist_lock as u64 & search_state.hash_table.mask()) as usize;
    let mut tt_static_eval: Option<Score> = None;
    if let Some(entry) = search_state.hash_table.probe(hash_index, position.zobrist_lock) {
        let tt_score = match entry.score {
            s if s > MATE_START => s - ply as Score,
            s if s < -MATE_START => s + ply as Score,
            s => s,
        };
        let usable = match entry.bound {
            Exact => true,
            Lower => tt_score >= window.1,
            Upper => tt_score <= window.0,
        };
        if usable {
            let pv_mv = if entry.mv != 0 && verify_move(position, entry.mv) {
                entry.mv
            } else {
                0
            };
            return (pv_single(pv_mv), tt_score);
        }
        // Not usable as a bound, but the cached static eval still saves the
        // NNUE forward pass below
        if entry.static_eval != STATIC_EVAL_NONE {
            tt_static_eval = Some(entry.static_eval);
        }
    }

    let in_check = is_check(position, position.mover);

    // Raw NNUE output, reused from the TT when present - this is the single
    // most expensive thing a qsearch node does. Captured before eval noise so
    // the cached value stays a clean static eval.
    let raw_static_eval: Score = match tt_static_eval {
        Some(v) => v,
        None => evaluate_position(position, search_state),
    };
    let mut eval = raw_static_eval;
    if search_state.eval_noise > 0 {
        let noise_bits = (position.zobrist_lock >> 17) as i32;
        let noise_max = search_state.eval_noise;
        eval += (noise_bits % (2 * noise_max + 1)) - noise_max;
    }

    // Depth cap terminates evasion chains; otherwise stand-pat is only a
    // valid bound when not in check
    if depth == 0 || (!in_check && eval >= window.1) {
        // A stand-pat cutoff is a lower bound (the true score is >= eval); the
        // depth-cap case is just the static eval, which we do not want to cache
        // as a bound at all since no search backed it up.
        if !in_check && eval >= window.1 {
            store_quiesce(search_state, hash_index, position, eval, Lower, 0, ply, raw_static_eval);
        }
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

    move_scores.sort_unstable_by_key(|b| std::cmp::Reverse(b.1));

    // Delta pruning margin: skip captures that can't raise alpha
    // even with full captured piece value plus this margin
    const DELTA_MARGIN: Score = 200;

    let mut legal_move_count = 0;

    for (m, _) in move_scores {
        let is_promotion = m & PROMOTION_FULL_MOVE_MASK != 0;
        let see_value = captured_piece_value_see(position, m);

        // Delta pruning: if capturing this piece can't possibly raise alpha,
        // skip it - never prune evasions or promotions
        if !in_check && !is_promotion && eval + see_value + DELTA_MARGIN < alpha {
            continue;
        }

        let old_mover = position.mover;
        let unmake = make_move_nnue(position, m, search_state);

        if !is_check(position, old_mover) {
            legal_move_count += 1;

            // SEE gate: skip losing captures (equal exchanges are searched);
            // evasions and promotions are never gated
            if in_check || is_promotion || see(see_value, bit(to_square_part(m)), position) >= 0 {
                let score = -quiesce(position, depth - 1, ply + 1, (-window.1, -alpha), search_state).1;

                unmake_move_nnue(position, m, &unmake, search_state);

                check_time!(search_state);
                if is_stopped(&search_state.stop) {
                    break;
                }

                if score >= window.1 {
                    store_quiesce(search_state, hash_index, position, window.1, Lower, m, ply, raw_static_eval);
                    return (pv_single(m), window.1);
                }
                if score > alpha {
                    alpha = score;
                    best_move = m;
                }
            } else {
                unmake_move_nnue(position, m, &unmake, search_state);
            }
        } else {
            unmake_move_nnue(position, m, &unmake, search_state);
        }
    }

    // All evasions were illegal (pins) - the check is mate
    if in_check && legal_move_count == 0 {
        let mate = -MATE_SCORE + ply as Score;
        store_quiesce(search_state, hash_index, position, mate, Exact, 0, ply, raw_static_eval);
        return (pv_single(0), mate);
    }

    // Only store when the search actually ran to completion; a stop mid-loop
    // leaves `alpha` reflecting a truncated search and must not be cached.
    if !is_stopped(&search_state.stop) {
        // Raised alpha above the caller's window => a real (exact) value;
        // otherwise everything failed low and alpha is only an upper bound.
        let bound = if alpha > window.0 { Exact } else { Upper };
        store_quiesce(search_state, hash_index, position, alpha, bound, best_move, ply, raw_static_eval);
    }

    (pv_single(best_move), alpha)
}
