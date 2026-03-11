use crate::engine_constants::{
    lmr_reduction, ALPHA_PRUNE_MARGINS, BETA_PRUNE_MARGIN_PER_DEPTH, BETA_PRUNE_MAX_DEPTH, CORRECTION_SCALE, CORRECTION_WEIGHT,
    IID_MIN_DEPTH, IID_REDUCE_DEPTH, LMP_MAX_DEPTH, LMP_MOVE_THRESHOLDS, LMR_CONTINUATION_BAD_THRESHOLD, LMR_CONTINUATION_GOOD_THRESHOLD,
    LMR_HISTORY_BAD_DIVISOR, LMR_HISTORY_GOOD_DIVISOR, LMR_LEGAL_MOVES_BEFORE_ATTEMPT, LMR_MIN_DEPTH, MAX_DEPTH, MAX_QUIESCE_DEPTH,
    MULTICUT_DEPTH_REDUCTION, MULTICUT_MIN_DEPTH, MULTICUT_MOVES_TO_TRY, MULTICUT_REQUIRED_CUTOFFS, NULL_MOVE_MIN_DEPTH,
    NULL_MOVE_REDUCE_DEPTH_BASE, NUM_CORRECTION_ENTRIES, PROBCUT_DEPTH_REDUCTION, PROBCUT_MARGIN, PROBCUT_MIN_DEPTH, ROOK_VALUE_AVERAGE,
    SEE_PRUNE_MARGIN, SEE_PRUNE_MAX_DEPTH, SINGULAR_EXTENSION_DEPTH_MARGIN, SINGULAR_EXTENSION_DEPTH_REDUCTION,
    SINGULAR_EXTENSION_MARGIN_MULTIPLIER, SINGULAR_EXTENSION_MIN_DEPTH, THREAT_EXTENSION_MARGIN, TM_INSTABILITY_EXTEND,
    TM_MIN_DEPTH_FOR_TM, TM_SCORE_DROP_EXTEND, TM_SCORE_DROP_THRESHOLD, TM_STABILITY_THRESHOLD,
};
use crate::evaluate::{evaluate_with_pawn_hash, insufficient_material, pawn_material, piece_material};
use crate::fen::algebraic_move_from_move;
use crate::hash::pawn_zobrist_key;
use crate::tablebase::{probe_dtz, tablebase_available, TB_MAX_PIECES};

use crate::bitboards::{bit, north_fill, south_fill, FILE_A_BITS, FILE_H_BITS};
use crate::hash::{en_passant_zobrist_key_index, ZOBRIST_KEYS_EN_PASSANT, ZOBRIST_KEY_MOVER_SWITCH};
use crate::make_move::{make_move_in_place, unmake_move, CAPTURED_NONE};
use crate::move_constants::{
    EN_PASSANT_NOT_AVAILABLE, PIECE_MASK_BISHOP, PIECE_MASK_FULL, PIECE_MASK_KING, PIECE_MASK_KNIGHT, PIECE_MASK_PAWN, PIECE_MASK_QUEEN,
    PIECE_MASK_ROOK, PROMOTION_FULL_MOVE_MASK,
};
use crate::move_scores::score_move;
use crate::moves::{generate_captures, generate_check_evasions, generate_moves, generate_quiet_moves, is_check, verify_move};
use crate::opponent;
use crate::quiesce::quiesce;
use crate::see::static_exchange_evaluation;
use crate::types::BoundType::{Exact, Lower, Upper};
use crate::types::{
    is_stopped, pv_prepend, pv_single, set_stop, BoundType, HashEntry, Move, MoveScore, MoveScoreArray, MoveScoreList, Mover, PathScore,
    Position, Score, SearchState, Square, Window, BLACK, WHITE,
};
use crate::utils::{captured_piece_value, from_square_part, send_info, to_square_part};
use std::cmp::{max, min};
use std::sync::atomic::Ordering;
use std::time::Instant;

pub const MAX_WINDOW: Score = 20000;
pub const MATE_SCORE: Score = 10000;
pub const MATE_MARGIN: Score = 1000;
pub const MATE_START: Score = MATE_SCORE - MATE_MARGIN;

pub const LAST_EXTENSION_LAYER: u8 = 4;

pub const MAX_NEW_EXTENSIONS_TREE_PART: [u8; 5] = [1, 0, 0, 0, 0];

#[macro_export]
macro_rules! time_remains {
    ($end_time:expr) => {
        $end_time > Instant::now()
    };
}

#[macro_export]
macro_rules! time_expired {
    ($search_state:expr) => {
        if is_stopped(&$search_state.stop) || Instant::now() >= $search_state.end_time {
            if !is_stopped(&$search_state.stop) {
                set_stop(&$search_state.stop, true);
                send_info($search_state, false);
            }
            true
        } else {
            false
        }
    };
}

#[macro_export]
macro_rules! check_time {
    ($search_state:expr) => {
        if !is_stopped(&$search_state.stop) && $search_state.nodes % 1000 == 0 {
            if $search_state.is_ponder_search && !$search_state.ponder_applied {
                if !$search_state.pondering.load(std::sync::atomic::Ordering::Relaxed) {
                    let hard_ms = $search_state.ponder_hard_ms.load(std::sync::atomic::Ordering::Relaxed);
                    let soft_ms = $search_state.ponder_soft_ms.load(std::sync::atomic::Ordering::Relaxed);
                    if hard_ms > 0 {
                        let now = std::time::Instant::now();
                        $search_state.end_time = now + std::time::Duration::from_millis(hard_ms);
                        $search_state.soft_time_limit = now + std::time::Duration::from_millis(soft_ms);
                        $search_state.time_management_active = true;
                    }
                    $search_state.ponder_applied = true;
                }
            }
            if $search_state.end_time < Instant::now() || $search_state.nodes >= $search_state.nodes_limit {
                set_stop(&$search_state.stop, true);
                send_info($search_state, false);
            }
        }
    };
}

#[macro_export]
macro_rules! debug_out {
    ($s:expr) => {
        if DEBUG {
            $s
        }
    };
}

pub fn iterative_deepening(position: &mut Position, max_depth: u8, search_state: &mut SearchState, start_depth: u8) -> Move {
    search_state.start_time = Instant::now();
    set_stop(&search_state.stop, false);
    search_state.hash_table_version += 1;
    search_state.best_move_stability = 0;
    search_state.prev_best_move = 0;
    search_state.prev_score = 0;

    let original_mover = position.mover;
    let all_moves = generate_moves(position);
    let mut legal_moves: MoveScoreList = Vec::with_capacity(all_moves.len());

    for m in all_moves {
        if m == search_state.ignore_root_move {
            continue;
        }
        // If searchmoves specified, only include those moves
        if let Some(ref search_moves) = search_state.search_moves {
            if !search_moves.contains(&m) {
                continue;
            }
        }
        let unmake = make_move_in_place(position, m);
        let legal = !is_check(position, original_mover);
        unmake_move(position, m, &unmake);
        if legal {
            legal_moves.push((m, -MATE_SCORE));
        }
    }

    // No legal moves = checkmate or stalemate, return null move
    if legal_moves.is_empty() {
        return 0;
    }

    // Single legal move: return immediately, no need to search
    if legal_moves.len() == 1 && search_state.time_management_active {
        return legal_moves[0].0;
    }

    // Tablebase probe at root: if ≤6 pieces, return best move immediately
    // The tablebase knows the perfect result - no need to search
    let all_pieces = position.pieces[WHITE as usize].all_pieces_bitboard | position.pieces[BLACK as usize].all_pieces_bitboard;
    if tablebase_available() && all_pieces.count_ones() <= TB_MAX_PIECES {
        let mut best_move: Move = 0;
        let mut best_score: Score = -MATE_SCORE;

        for (m, _) in &legal_moves {
            let unmake = make_move_in_place(position, *m);
            // Probe DTZ after the move (from opponent's perspective, so negate score)
            if let Some(tb_score) = probe_dtz(position) {
                let score = -tb_score;
                if score > best_score {
                    best_score = score;
                    best_move = *m;
                }
            }
            unmake_move(position, *m, &unmake);
        }

        // If we found a valid TB move, return it immediately
        if best_move != 0 {
            if search_state.thread_id == 0 {
                println!("info depth 1 score cp {} pv {}", best_score, algebraic_move_from_move(best_move));
            }
            return best_move;
        }
    }

    age_history_table(search_state);
    clear_killers(search_state);

    if search_state.history.is_empty() {
        search_state.history.push(position.zobrist_lock)
    }

    let mut aspiration_window = (-MAX_WINDOW, MAX_WINDOW);

    // Initialize with the first legal move so we always have a valid move to return
    // even if time expires before the first search iteration completes
    search_state.current_best = (pv_single(legal_moves[0].0), -MATE_SCORE);

    const ASPIRATION_RADIUS: [Score; 6] = [25, 50, 100, 200, 400, 800];

    let mut prev_synced_nodes: u64 = 0;

    for iterative_depth in start_depth..=max_depth {
        //println!("Iterative depth {}", iterative_depth);
        let mut c = 0;
        search_state.iterative_depth = iterative_depth;

        loop {
            //println!("Searching with aspiration window {} {} at [{}]", aspiration_window.0, aspiration_window.1, c);
            let aspire_best = start_search(position, &mut legal_moves, search_state, aspiration_window);
            if time_expired!(search_state) {
                return search_state.current_best.0[0];
            }

            if aspire_best.1 > aspiration_window.0 && aspire_best.1 < aspiration_window.1 {
                //println!("Found a move within the aspiration window {} {}", algebraic_move_from_move(aspire_best.0[0]), aspire_best.1);
                search_state.current_best = aspire_best;
                //println!("Current best move is {} {}", algebraic_move_from_move(search_state.current_best.0[0]), search_state.current_best.1);
                break;
            } else {
                //println!("Move score was outside the aspiration window {} {} {} {}", aspire_best.1, aspiration_window.0, aspiration_window.1, c);
                c += 1;
                if c == ASPIRATION_RADIUS.len() {
                    aspiration_window = (-MAX_WINDOW, MAX_WINDOW);
                } else if aspire_best.1 <= aspiration_window.0 {
                    aspiration_window.0 = max(-MAX_WINDOW, aspiration_window.0 - ASPIRATION_RADIUS[c]);
                } else if aspire_best.1 >= aspiration_window.1 {
                    aspiration_window.1 = min(MAX_WINDOW, aspiration_window.1 + ASPIRATION_RADIUS[c]);
                }
                //println!("New aspiration window {} {}", aspiration_window.0, aspiration_window.1);
            };
        }

        legal_moves.sort_by(|(_, a), (_, b)| b.cmp(a));
        for mv in legal_moves.iter_mut() {
            mv.1 = -MATE_SCORE;
        }
        search_state.root_moves.clone_from(&legal_moves);

        if search_state.multi_pv == 1 {
            aspiration_window = (
                search_state.current_best.1 - ASPIRATION_RADIUS[0],
                search_state.current_best.1 + ASPIRATION_RADIUS[0],
            );
        }

        send_info(search_state, true);

        // Time management decisions (thread 0 only)
        if search_state.time_management_active && search_state.thread_id == 0 && iterative_depth >= TM_MIN_DEPTH_FOR_TM {
            let current_move = search_state.current_best.0[0];
            let current_score = search_state.current_best.1;

            // Track best move stability
            if current_move == search_state.prev_best_move {
                search_state.best_move_stability = search_state.best_move_stability.saturating_add(1);
            } else {
                search_state.best_move_stability = 0;
            }

            // Track score drop
            let score_dropped = search_state.prev_score - current_score > TM_SCORE_DROP_THRESHOLD && search_state.prev_score != 0;

            let best_move_changed = search_state.prev_best_move != 0 && current_move != search_state.prev_best_move;

            search_state.prev_best_move = current_move;
            search_state.prev_score = current_score;

            let now = Instant::now();

            // Early stop: stable best move and past soft limit
            if search_state.best_move_stability >= TM_STABILITY_THRESHOLD && now >= search_state.soft_time_limit {
                set_stop(&search_state.stop, true);
                break;
            }

            // Extend soft limit when best move changed
            if best_move_changed && now < search_state.end_time {
                let remaining_soft = search_state.soft_time_limit.saturating_duration_since(now);
                let extension = remaining_soft.mul_f64(TM_INSTABILITY_EXTEND - 1.0);
                let extended = search_state.soft_time_limit + extension;
                search_state.soft_time_limit = min(extended, search_state.end_time);
            }

            // Extend soft limit on score drop
            if score_dropped && now < search_state.end_time {
                let remaining_soft = search_state.soft_time_limit.saturating_duration_since(now);
                let extension = remaining_soft.mul_f64(TM_SCORE_DROP_EXTEND - 1.0);
                let extended = search_state.soft_time_limit + extension;
                search_state.soft_time_limit = min(extended, search_state.end_time);
            }

            // Past soft limit (possibly extended) — stop
            if now >= search_state.soft_time_limit {
                set_stop(&search_state.stop, true);
                break;
            }
        }

        // Sync local node count to the shared counter
        let new_nodes = search_state.nodes - prev_synced_nodes;
        if new_nodes > 0 {
            search_state.shared_nodes.fetch_add(new_nodes, Ordering::Relaxed);
            prev_synced_nodes = search_state.nodes;
        }
    }

    // Final sync of any remaining nodes
    let remaining = search_state.nodes - prev_synced_nodes;
    if remaining > 0 {
        search_state.shared_nodes.fetch_add(remaining, Ordering::Relaxed);
    }

    legal_moves[0].0
}

pub fn start_search(position: &mut Position, legal_moves: &mut MoveScoreList, search_state: &mut SearchState, window: Window) -> PathScore {
    let mut current_best: PathScore = (pv_single(legal_moves[0].0), window.0);
    let hash_len_u128 = search_state.hash_table.len() as u128;

    for mv in legal_moves {
        let unmake = make_move_in_place(position, mv.0);
        prefetch_hash(position, search_state, hash_len_u128); // Prefetch child position's hash entry
        search_state.history.push(position.zobrist_lock);

        let mut path_score = search(
            position,
            search_state.iterative_depth - 1,
            1,
            (-window.1, -current_best.1),
            search_state,
            false,
            0,
        );
        path_score.1 = -path_score.1;

        search_state.history.pop();
        unmake_move(position, mv.0, &unmake);
        mv.1 = path_score.1;

        search_state.pv.insert(mv.0, (pv_prepend(mv.0, &path_score.0), mv.1));

        if mv.1 > current_best.1 && time_remains!(search_state.end_time) {
            current_best = (pv_prepend(mv.0, &path_score.0), mv.1);
            send_info(search_state, false);
        }

        if time_expired!(search_state) {
            return current_best;
        }
    }
    current_best
}

fn clear_killers(search_state: &mut SearchState) {
    for i in 0..MAX_DEPTH as usize {
        search_state.mate_killer[i] = 0;
        for j in 0..2 {
            search_state.killer_moves[i][j] = 0;
        }
    }
}

fn age_history_table(search_state: &mut SearchState) {
    for piece in search_state.history_moves.iter_mut() {
        for from_sq in piece.iter_mut() {
            for val in from_sq.iter_mut() {
                *val /= 2;
            }
        }
    }
    search_state.highest_history_score /= 2;

    // Age countermove history table
    for prev_piece in search_state.countermove_history.iter_mut() {
        for prev_to in prev_piece.iter_mut() {
            for curr_piece in prev_to.iter_mut() {
                for val in curr_piece.iter_mut() {
                    *val /= 2;
                }
            }
        }
    }

    // Age followup history table
    for prev_piece in search_state.followup_history.iter_mut() {
        for prev_to in prev_piece.iter_mut() {
            for curr_piece in prev_to.iter_mut() {
                for val in curr_piece.iter_mut() {
                    *val /= 2;
                }
            }
        }
    }

    // Age capture history table
    for attacker in &mut search_state.capture_history {
        for victim in attacker {
            for val in victim.iter_mut() {
                *val /= 2;
            }
        }
    }

    // Age correction history table
    for val in search_state.correction_history.iter_mut() {
        *val /= 2;
    }
}

#[inline(always)]
#[allow(clippy::too_many_arguments)]
pub fn store_hash_entry(
    position: &Position,
    height: u8,
    existing_height: u8,
    existing_version: u32,
    bound: BoundType,
    movescore: MoveScore,
    search_state: &mut SearchState,
    ply: u8,
    hash_index: usize,
) {
    if height >= existing_height || search_state.hash_table_version > existing_version {
        search_state.hash_table.set(
            hash_index,
            HashEntry {
                // adjust any mate score so that the score appears calculated as if this ply were the root
                score: match movescore.1 {
                    x if x > MATE_START => movescore.1 + ply as Score,
                    x if x < MATE_START => movescore.1 - ply as Score,
                    _ => movescore.1,
                },
                version: search_state.hash_table_version,
                height,
                mv: movescore.0,
                bound,
                lock: position.zobrist_lock,
            },
        );
    }
}

#[inline(always)]
fn total_material_value(position: &Position) -> Score {
    let piece_material = piece_material(position, WHITE) + piece_material(position, BLACK);
    let pawn_material = pawn_material(position, WHITE) + pawn_material(position, BLACK);
    piece_material + pawn_material
}

#[inline(always)]
fn is_end_game(position: &Position) -> bool {
    total_material_value(position) < ROOK_VALUE_AVERAGE * 4
}

#[inline(always)]
pub fn draw_value(position: &Position, search_state: &SearchState) -> Score {
    if is_end_game(position) {
        0
    } else {
        search_state.contempt
    }
}

/// Check if a move is a pawn push to the 7th rank (about to promote)
/// This is worth extending as promotions are game-changing
#[inline(always)]
pub fn is_pawn_push_to_7th(position: &Position, m: Move) -> bool {
    // Square layout: h1=0, a1=7, h2=8, ..., a8=63
    // 7th rank (white): 48-55, 2nd rank (black): 8-15
    let piece = m & PIECE_MASK_FULL;
    if piece != PIECE_MASK_PAWN {
        return false;
    }
    let to_sq = to_square_part(m);
    if position.mover == WHITE {
        (48..=55).contains(&to_sq) // 7th rank for white
    } else {
        (8..=15).contains(&to_sq) // 2nd rank for black
    }
}

/// Check if a move is a passed pawn push (passed pawn advancing to 5th rank or beyond)
/// Passed pawns are critical in endgames and worth extending
/// Extends for advanced passed pawns (5th/6th for white, 3rd/4th for black)
/// Note: 7th rank (white) / 2nd rank (black) already gets extension via is_pawn_push_to_7th
#[inline(always)]
pub fn is_passed_pawn_push(position: &Position, m: Move) -> bool {
    let piece = m & PIECE_MASK_FULL;
    if piece != PIECE_MASK_PAWN {
        return false;
    }

    let from_sq = from_square_part(m) as i8;
    let to_sq = to_square_part(m);

    // Extend for pawns reaching 5th or 6th rank (7th already gets extension separately)
    // White 5th rank: squares 32-39, 6th rank: 40-47
    // Black 4th rank: squares 24-31, 3rd rank: 16-23
    let is_advanced = if position.mover == WHITE {
        (32..=47).contains(&to_sq) // 5th or 6th rank for white
    } else {
        (16..=31).contains(&to_sq) // 3rd or 4th rank for black
    };

    if !is_advanced {
        return false;
    }

    // Check if this pawn is passed
    // A pawn is passed if there are no enemy pawns on the same file or adjacent files ahead of it
    let white_pawns = position.pieces[WHITE as usize].pawn_bitboard;
    let black_pawns = position.pieces[BLACK as usize].pawn_bitboard;

    // Calculate pawn attacks for blocking detection
    let white_pawn_attacks = ((white_pawns & !FILE_A_BITS) << 9) | ((white_pawns & !FILE_H_BITS) << 7);
    let black_pawn_attacks = ((black_pawns & !FILE_A_BITS) >> 7) | ((black_pawns & !FILE_H_BITS) >> 9);

    let pawn_bit = bit(from_sq);

    if position.mover == WHITE {
        // Check if this white pawn is passed
        // Passed = no black pawns or attacks ahead on same/adjacent files
        let blockers = south_fill(black_pawns | black_pawn_attacks | (white_pawns >> 8));
        (pawn_bit & !blockers) != 0
    } else {
        // Check if this black pawn is passed
        let blockers = north_fill(white_pawns | white_pawn_attacks | (black_pawns << 8));
        (pawn_bit & !blockers) != 0
    }
}

#[inline(always)]
pub fn null_move_reduced_depth(depth: u8) -> u8 {
    match depth {
        d if d > NULL_MOVE_REDUCE_DEPTH_BASE + 1 => depth - 1 - (NULL_MOVE_REDUCE_DEPTH_BASE + d / 6),
        _ => 1,
    }
}

/// Prefetch the hash entry for the current position
/// Call this right after making a move to hide memory latency
#[inline(always)]
fn prefetch_hash(position: &Position, search_state: &SearchState, hash_len_u128: u128) {
    let index = (position.zobrist_lock % hash_len_u128) as usize;
    search_state.hash_table.prefetch(index);
}

#[inline(always)]
pub fn search(
    position: &mut Position,
    depth: u8,
    ply: u8,
    window: Window,
    search_state: &mut SearchState,
    on_null_move: bool,
    excluded_move: Move, // For singular extension search: skip this move (0 = no exclusion)
) -> PathScore {
    // Check stop flag at TOP before any moves are made - safe to return here
    if is_stopped(&search_state.stop) {
        return (pv_single(0), 0);
    }

    check_time!(search_state);
    if is_stopped(&search_state.stop) {
        return (pv_single(0), 0);
    }

    if is_draw(position, search_state, ply) {
        search_state.nodes += 1;
        return (pv_single(0), draw_value(position, search_state));
    }

    // Prevent stack overflow and array out of bounds at extreme depths
    if ply >= MAX_DEPTH {
        return (pv_single(0), evaluate_with_pawn_hash(position, &search_state.pawn_hash_table));
    }

    // NOTE: Tablebase probing during search is disabled for performance.
    // The position_to_chess() conversion is too expensive to call at every node.
    // Instead, we probe DTZ at the root level in iterative_deepening to select
    // the best move in TB positions.

    let scouting = window.1 - window.0 == 1;

    if depth == 0 {
        return quiesce(position, MAX_QUIESCE_DEPTH, ply, window, search_state);
    }

    search_state.nodes += 1;

    let mut alpha = window.0;
    let mut beta = window.1;

    // Mate distance pruning: tighten bounds based on ply
    alpha = max(alpha, -MATE_SCORE + ply as Score);
    beta = min(beta, MATE_SCORE - ply as Score - 1);
    if alpha >= beta {
        return (pv_single(0), alpha);
    }

    let mut legal_move_count = 0;
    let mut hash_flag = Upper;
    let mut hash_height = 0;
    let mut hash_version = 0;
    let mut best_pathscore: PathScore = (pv_single(0), -MATE_SCORE);

    let hash_len_u128 = search_state.hash_table.len() as u128;
    let hash_index: usize = (position.zobrist_lock % hash_len_u128) as usize;
    let hash_entry = search_state.hash_table.get(hash_index);
    // Cache hash hit check to avoid repeated 128-bit comparisons
    let hash_hit = hash_entry.lock == position.zobrist_lock;
    // Track hash entry info for singular extension (needed even if depth isn't sufficient for cutoff)
    let hash_entry_height = if hash_hit { hash_entry.height } else { 0 };
    let hash_entry_bound = if hash_hit { hash_entry.bound } else { Upper };
    let mut hash_move = if hash_hit {
        // Adjust any mate score so that the score appears calculated from the current root rather than the root when the position was stored
        // When we found the mate, we set the score to reflect the distance from the root, and then, when we stored the score in the TT, we
        // adjusted it again such that it represented the distance from the root at which it was stored - e.g. we found it at ply 7, and wound
        // up needing to store that score when it went back up to ply 5, so we adjusted it so it looked like it was found at ply 2.
        // In other words, we marked it as a mate in 7 plies (99.93) when we found it (even though it's already mate), but if we need to store it in the
        // has table at ply 5, we should record it as a mate in two plies (99.98).
        // Now that we are retrieving it, we need to adjust it for the current ply. Following the previous example, if the current ply is 10, then
        // we adjust the score to make it look like it was found at ply 12.
        let score = match hash_entry.score {
            s if s > MATE_START => s - ply as Score,
            s if s < -MATE_START => s + ply as Score,
            s => s,
        };

        // Skip hash cutoffs during singular extension search - the hash entry's score
        // includes the excluded move's contribution, so we can't use it for cutoffs
        if excluded_move == 0 && hash_entry.height >= depth {
            hash_height = hash_entry.height;
            hash_version = hash_entry.version;
            if hash_entry.bound == Exact {
                search_state.hash_hits_exact += 1;
                return (pv_single(hash_entry.mv), score);
            }
            if hash_entry.bound == Lower && score > alpha {
                alpha = score;
            }
            if hash_entry.bound == Upper && score < beta {
                beta = score
            }
            if alpha >= beta {
                return (pv_single(hash_entry.mv), score);
            }
        }
        hash_entry.mv
    } else if hash_entry.lock != 0 {
        search_state.hash_clashes += 1;
        0
    } else {
        0
    };

    let in_check = is_check(position, position.mover);

    // Correction history: look up pawn-structure-indexed eval correction
    let correction_index = (pawn_zobrist_key(position) as usize) % NUM_CORRECTION_ENTRIES;
    let correction = search_state.correction_history[correction_index] / CORRECTION_SCALE;

    let mut lazy_eval: Score = -Score::MAX;

    if scouting && depth <= BETA_PRUNE_MAX_DEPTH && !in_check && beta.abs() < MATE_START {
        lazy_eval = evaluate_with_pawn_hash(position, &search_state.pawn_hash_table) + correction;
        let margin = BETA_PRUNE_MARGIN_PER_DEPTH * depth as Score;
        if lazy_eval - margin as Score >= beta {
            return (pv_single(0), lazy_eval - margin);
        }
    }

    let alpha_prune_flag = if depth <= ALPHA_PRUNE_MARGINS.len() as u8 && scouting && !in_check && alpha.abs() < MATE_START {
        if lazy_eval == -Score::MAX {
            lazy_eval = evaluate_with_pawn_hash(position, &search_state.pawn_hash_table) + correction;
        }

        lazy_eval + ALPHA_PRUNE_MARGINS[depth as usize - 1] < alpha
    } else {
        false
    };

    // Threat detection: when null move fails badly, opponent has a dangerous threat
    // We'll use this to reduce LMR aggressiveness rather than extending
    let mut threat_detected = false;

    if !on_null_move && scouting && depth >= NULL_MOVE_MIN_DEPTH && null_move_material(position) && !in_check {
        let old_ep = make_null_move(position);

        let score = -search(
            position,
            null_move_reduced_depth(depth),
            ply + 1,
            (-beta, (-beta) + 1),
            search_state,
            true,
            0,
        )
        .1;

        unmake_null_move(position, old_ep);

        if is_stopped(&search_state.stop) {
            return (pv_single(0), 0);
        }

        if score >= beta {
            return (pv_single(0), beta);
        }

        // If null move fails significantly below alpha, opponent has a threat
        // Use higher threshold (400 = losing a piece) to be selective
        if score < alpha - THREAT_EXTENSION_MARGIN {
            threat_detected = true;
        }
    }

    // Probcut: at high depth, do a shallow search with raised beta
    // If a capture fails high, the position is probably winning and can be cut
    if scouting && !in_check && depth >= PROBCUT_MIN_DEPTH && beta.abs() < MATE_START {
        let probcut_beta = beta + PROBCUT_MARGIN;
        let probcut_depth = depth - PROBCUT_DEPTH_REDUCTION;

        // Generate captures and try them at reduced depth
        let captures = generate_captures(position);
        for m in captures {
            // Only try captures with non-negative SEE
            if static_exchange_evaluation(position, m) < 0 {
                continue;
            }

            let old_mover = position.mover;
            let unmake = make_move_in_place(position, m);
            prefetch_hash(position, search_state, hash_len_u128);

            if !is_check(position, old_mover) {
                let score = -search(
                    position,
                    probcut_depth,
                    ply + 1,
                    (-probcut_beta, -probcut_beta + 1),
                    search_state,
                    false,
                    0,
                )
                .1;

                unmake_move(position, m, &unmake);

                if is_stopped(&search_state.stop) {
                    return (pv_single(0), 0);
                }

                if score >= probcut_beta {
                    return (pv_single(0), beta);
                }
            } else {
                unmake_move(position, m, &unmake);
            }
        }
    }

    // Multi-cut: at high depth, if multiple moves fail high at shallow depth,
    // the position is probably good and can be cut
    if scouting && !in_check && depth >= MULTICUT_MIN_DEPTH && beta.abs() < MATE_START {
        let multicut_depth = depth - MULTICUT_DEPTH_REDUCTION;
        let mut fail_high_count: u8 = 0;

        // Generate and score captures for multi-cut
        let captures = generate_captures(position);
        let enemy = &position.pieces[opponent!(position.mover) as usize];
        let mut scored_captures: MoveScoreArray = captures
            .into_iter()
            .map(|m| (m, score_move(position, m, search_state, ply as usize, enemy)))
            .collect();

        // Sort by score descending to try best captures first
        scored_captures.sort_by(|a, b| b.1.cmp(&a.1));

        for (m, _) in scored_captures.iter().take(MULTICUT_MOVES_TO_TRY as usize) {
            let old_mover = position.mover;
            let unmake = make_move_in_place(position, *m);
            prefetch_hash(position, search_state, hash_len_u128);

            if !is_check(position, old_mover) {
                let score = -search(position, multicut_depth, ply + 1, (-beta, -beta + 1), search_state, false, 0).1;

                unmake_move(position, *m, &unmake);

                if is_stopped(&search_state.stop) {
                    return (pv_single(0), 0);
                }

                if score >= beta {
                    fail_high_count += 1;
                    if fail_high_count >= MULTICUT_REQUIRED_CUTOFFS {
                        return (pv_single(0), beta);
                    }
                }
            } else {
                unmake_move(position, *m, &unmake);
            }
        }
    }

    let mut scout_search = false;

    // Check extension: extend by 1 ply when in check
    let check_extension: u8 = if in_check && ply < search_state.iterative_depth * 2 { 1 } else { 0 };
    let real_depth = depth + check_extension;

    let verified_hash_move = if !scouting && hash_move == 0 && depth + check_extension > IID_MIN_DEPTH {
        hash_move = search_wrapper(depth - IID_REDUCE_DEPTH, ply, search_state, (-alpha - 1, -alpha), position, 0, 0).0[0];
        hash_move != 0
    } else {
        hash_move != 0 && verify_move(position, hash_move)
    };

    // Try hash move first if valid
    // Singular extension: determine if hash move is much better than alternatives
    // Only apply when:
    // - Hash move is valid
    // - Not in check (check extension handles that)
    // - Depth is sufficient (overhead of singular search not worth it at low depth)
    // - Hash entry has sufficient depth (reliable information)
    // - Hash entry has Lower bound (hash move caused a fail-high, so it's proven good)
    // - Not already in a singular search (excluded_move == 0)
    // - Not searching for mate (mate lines need full exploration)
    let singular_extension: u8 = if verified_hash_move
        && !in_check
        && depth >= SINGULAR_EXTENSION_MIN_DEPTH
        && hash_entry_height >= depth.saturating_sub(SINGULAR_EXTENSION_DEPTH_MARGIN)
        && hash_entry_bound == Lower // Only extend if hash move caused a fail-high
        && excluded_move == 0 // Don't do singular search within singular search
        && alpha.abs() < MATE_START
    // Don't interfere with mate searches
    {
        // Do a reduced search excluding the hash move
        // If all alternatives fail low, the hash move is singular
        let singular_beta = alpha - (depth as Score * SINGULAR_EXTENSION_MARGIN_MULTIPLIER);
        let singular_depth = (depth - SINGULAR_EXTENSION_DEPTH_REDUCTION).max(1);

        // Note: no history push/pop needed here - we're searching from the current position
        // and search() handles history internally for each move it tries
        let singular_score = search(
            position,
            singular_depth,
            ply,
            (singular_beta - 1, singular_beta),
            search_state,
            false,
            hash_move, // Exclude hash move from this search
        )
        .1;

        if is_stopped(&search_state.stop) {
            return (pv_single(0), 0);
        }

        // If all alternatives fail low, extend the hash move
        if singular_score < singular_beta {
            1
        } else {
            0
        }
    } else {
        0
    };

    if verified_hash_move {
        let hash_captured_value = captured_piece_value(position, hash_move);
        let old_mover = position.mover;
        let unmake = make_move_in_place(position, hash_move);
        prefetch_hash(position, search_state, hash_len_u128); // Prefetch child position's hash entry
        let hash_is_capture = unmake.captured_piece != CAPTURED_NONE;

        if !is_check(position, old_mover) {
            legal_move_count += 1;
            let hash_search_depth = real_depth + singular_extension;
            let path_score = search_wrapper(hash_search_depth, ply, search_state, (-beta, -alpha), position, 0, 0);
            let score = path_score.1;
            let singular_depth = hash_search_depth;

            unmake_move(position, hash_move, &unmake);
            check_time!(search_state);
            if is_stopped(&search_state.stop) {
                return best_pathscore;
            }

            if score > best_pathscore.1 {
                best_pathscore = (pv_prepend(hash_move, &path_score.0), score);
                if best_pathscore.1 > alpha {
                    alpha = best_pathscore.1;
                    if alpha >= beta {
                        return cutoff_unmake(
                            position,
                            singular_depth,
                            ply,
                            search_state,
                            best_pathscore,
                            hash_height,
                            hash_version,
                            hash_move,
                            hash_is_capture,
                            hash_captured_value,
                            hash_index,
                        );
                    }
                    hash_flag = Exact;
                }
                scout_search = true;
            }
        } else {
            unmake_move(position, hash_move, &unmake);
        }
    }

    // MOVE GENERATION: Use check evasions when in check, staged generation otherwise
    let enemy = position.pieces[opponent!(position.mover) as usize];
    let mut move_scores: MoveScoreArray;
    let mut quiets_added: bool;

    if in_check {
        // When in check, generate only check evasion moves
        let mut evasions = generate_check_evasions(position);
        if verified_hash_move {
            evasions.retain(|m| *m != hash_move);
        }
        // Also exclude the singular extension search move if set
        if excluded_move != 0 {
            evasions.retain(|m| *m != excluded_move);
        }
        move_scores = evasions
            .into_iter()
            .map(|m| (m, score_move(position, m, search_state, ply as usize, &enemy)))
            .collect();
        quiets_added = true; // No staged generation when in check
    } else {
        // Normal staged move generation: captures first, then quiets
        let mut captures = generate_captures(position);
        if verified_hash_move {
            captures.retain(|m| *m != hash_move);
        }
        // Also exclude the singular extension search move if set
        if excluded_move != 0 {
            captures.retain(|m| *m != excluded_move);
        }
        move_scores = captures
            .into_iter()
            .map(|m| (m, score_move(position, m, search_state, ply as usize, &enemy)))
            .collect();
        quiets_added = false;
    }

    // Cache endgame status for current position - used in extension decisions
    let current_is_end_game = is_end_game(position);

    loop {
        // If we've exhausted the current move list, add quiet moves if we haven't yet
        if move_scores.is_empty() {
            if quiets_added {
                break; // All moves processed
            }
            // Add quiet moves
            quiets_added = true;
            let mut quiets = generate_quiet_moves(position);
            if verified_hash_move {
                quiets.retain(|m| *m != hash_move);
            }
            // Also exclude the singular extension search move if set
            if excluded_move != 0 {
                quiets.retain(|m| *m != excluded_move);
            }
            for m in quiets {
                move_scores.push((m, score_move(position, m, search_state, ply as usize, &enemy)));
            }
            if move_scores.is_empty() {
                break; // No quiet moves either
            }
        }

        let m = pick_high_score_move(&mut move_scores);
        let captured_value = captured_piece_value(position, m);
        let old_mover = position.mover;
        // For alpha pruning and LMR, treat promotions like captures (don't prune/reduce them)
        let is_tactical = captured_value > 0;
        let is_promotion = m & PROMOTION_FULL_MOVE_MASK != 0;

        // SEE pruning: skip bad captures at low depths
        // Only in scout (null-window) searches to avoid missing important PV moves
        // Don't prune promotions (they change material dramatically) or when in check
        // Don't prune when searching for mate (alpha/beta near mate scores)
        if scouting && is_tactical && !is_promotion && !in_check && depth <= SEE_PRUNE_MAX_DEPTH && alpha.abs() < MATE_START {
            let see_threshold = -(SEE_PRUNE_MARGIN * (depth as Score) * (depth as Score));
            if static_exchange_evaluation(position, m) < see_threshold {
                continue;
            }
        }

        // Pawn push extension: extend by 1 ply for pawn push to 7th rank
        // Only if no check extension already applied (avoid over-extending)
        let pawn_push_ext: u8 = if check_extension == 0 && is_pawn_push_to_7th(position, m) && ply < search_state.iterative_depth * 2 {
            1
        } else {
            0
        };

        // Passed pawn push extension: extend by 1 ply for passed pawn reaching 5th/6th rank
        // Only apply in endgames to avoid search explosion in complex middlegames
        // Only if no other extension already applied (avoid over-extending)
        let passed_pawn_ext: u8 = if check_extension == 0
            && pawn_push_ext == 0
            && current_is_end_game
            && is_passed_pawn_push(position, m)
            && ply < search_state.iterative_depth * 2
        {
            1
        } else {
            0
        };

        let move_extension = check_extension + pawn_push_ext + passed_pawn_ext;

        let unmake = make_move_in_place(position, m);
        prefetch_hash(position, search_state, hash_len_u128); // Prefetch child position's hash entry
                                                              // Track move at this ply for countermove heuristic
        search_state.ply_move[ply as usize] = m;
        // For killer moves, use actual capture detection from unmake info
        let move_is_capture = unmake.captured_piece != CAPTURED_NONE;

        // Check if move is legal (king not in check after move)
        if !is_check(position, old_mover) {
            legal_move_count += 1;

            // Cache whether this move gives check (opponent's king in check)
            // Used for pruning decisions - moves that give check should not be pruned/reduced
            let gives_check = is_check(position, position.mover);

            if legal_move_count > 1 && alpha_prune_flag && !is_tactical && !gives_check {
                unmake_move(position, m, &unmake);
                continue;
            }

            // Late Move Pruning (LMP): skip late quiet moves at shallow depths
            // More aggressive than LMR - completely skips the move instead of reducing
            // Don't prune in endgames (every move matters) or near mate scores
            if scouting
                && depth <= LMP_MAX_DEPTH
                && !in_check
                && !is_tactical
                && !is_promotion
                && !current_is_end_game
                && legal_move_count > LMP_MOVE_THRESHOLDS[depth as usize]
                && m != search_state.killer_moves[ply as usize][0]
                && m != search_state.killer_moves[ply as usize][1]
                && !gives_check
                && alpha.abs() < MATE_START
            {
                unmake_move(position, m, &unmake);
                continue;
            }

            let lmr = if move_extension == 0
                && legal_move_count > LMR_LEGAL_MOVES_BEFORE_ATTEMPT
                && real_depth > LMR_MIN_DEPTH
                && !is_tactical
                && m != search_state.killer_moves[ply as usize][0]
                && m != search_state.killer_moves[ply as usize][1]
                && !gives_check
            {
                let mut reduction = lmr_reduction(real_depth, legal_move_count);
                // Threat-based LMR: reduce less when opponent has a detected threat
                // This ensures we don't miss tactical replies to threats
                if threat_detected && reduction > 0 {
                    reduction -= 1;
                }
                // PV-node LMR: reduce less at principal variation nodes
                // PV nodes are more likely to contain the best line
                if !scout_search && reduction > 0 {
                    reduction -= 1;
                }
                // Cache move parts once for all LMR adjustments
                let curr_piece = piece_type_to_index(m);
                let curr_to = to_square_part(m) as usize;
                let curr_from = from_square_part(m) as usize;

                // Cache previous move info (used for countermove and continuation history)
                let (has_prev_move, prev_piece_12, prev_to) = if ply > 0 {
                    let prev_move = search_state.ply_move[ply as usize - 1];
                    if prev_move != 0 {
                        let opponent_side = position.mover ^ 1;
                        (
                            true,
                            piece_type_to_index(prev_move) + (opponent_side as usize * 6),
                            to_square_part(prev_move) as usize,
                        )
                    } else {
                        (false, 0, 0)
                    }
                } else {
                    (false, 0, 0)
                };

                // Countermove bonus: reduce less for the stored countermove
                // The countermove has historically been a good response to the previous move
                if has_prev_move && reduction > 0 && search_state.countermoves[prev_piece_12][prev_to] == m {
                    reduction -= 1;
                }

                // History-based LMR: adjust reduction based on move's historical performance
                // Moves that have worked well before get searched deeper (less reduction)
                // Moves that have failed often get searched shallower (more reduction)
                let piece_12 = piece_index_12(position, m);
                let hist = search_state.history_moves[piece_12][curr_from][curr_to];
                let good_threshold = search_state.highest_history_score / LMR_HISTORY_GOOD_DIVISOR as i64;
                let bad_threshold = -(search_state.highest_history_score / LMR_HISTORY_BAD_DIVISOR as i64);
                if hist > good_threshold && reduction > 0 {
                    reduction -= 1;
                } else if hist < bad_threshold {
                    reduction += 1;
                }

                // Continuation history: use countermove_history and followup_history for additional adjustment
                // These capture context-specific move quality (what works after certain moves)
                let mut continuation_score: i32 = 0;

                // Add countermove history contribution (reuse cached prev_piece_12/prev_to)
                if has_prev_move {
                    continuation_score += search_state.countermove_history[prev_piece_12][prev_to][curr_piece][curr_to] as i32;
                }
                // Add followup history contribution
                if ply >= 2 {
                    let our_prev_move = search_state.ply_move[ply as usize - 2];
                    if our_prev_move != 0 {
                        let our_prev_piece = piece_type_to_index(our_prev_move);
                        let our_prev_to = to_square_part(our_prev_move) as usize;
                        continuation_score += search_state.followup_history[our_prev_piece][our_prev_to][curr_piece][curr_to] as i32;
                    }
                }
                if continuation_score > LMR_CONTINUATION_GOOD_THRESHOLD && reduction > 0 {
                    reduction -= 1;
                } else if continuation_score < LMR_CONTINUATION_BAD_THRESHOLD {
                    reduction += 1;
                }
                reduction
            } else {
                0
            };

            // Forward futility pruning: if a quiet move at reduced depth can't raise alpha, skip it
            // This extends the node-level alpha pruning to also catch moves at higher nominal depths
            // that would be heavily reduced by LMR
            if lmr > 0 && !gives_check && !is_tactical && !is_promotion && alpha.abs() < MATE_START {
                let reduced_depth = (depth as i32 - 1 - lmr as i32 + move_extension as i32).max(0) as usize;
                if reduced_depth < ALPHA_PRUNE_MARGINS.len()
                    && lazy_eval != -Score::MAX
                    && lazy_eval + ALPHA_PRUNE_MARGINS[reduced_depth] <= alpha
                {
                    unmake_move(position, m, &unmake);
                    continue;
                }
            }

            // Apply extensions to search depth
            let search_depth = depth + move_extension;

            let path_score = if scout_search {
                lmr_scout_search(lmr, ply, search_state, (alpha, beta), search_depth, position)
            } else {
                search_wrapper(search_depth, ply, search_state, (-beta, -alpha), position, 0, 0)
            };

            let score = path_score.1;

            unmake_move(position, m, &unmake);

            check_time!(search_state);
            if is_stopped(&search_state.stop) {
                break;
            }

            if score < beta {
                let penalty = -(real_depth as i32) * if score < alpha { 2 } else { 1 };
                update_history(position, search_state, m, penalty as i64, captured_value);
                update_countermove_history_for_move(position, ply, search_state, m, penalty, captured_value, is_promotion);
                update_followup_history_for_move(position, ply, search_state, m, penalty, captured_value, is_promotion);
                update_capture_history_for_move(position, search_state, m, penalty, captured_value);
            }

            if score > best_pathscore.1 {
                best_pathscore = (pv_prepend(m, &path_score.0), score);
                if best_pathscore.1 > alpha {
                    alpha = best_pathscore.1;
                    if alpha >= beta {
                        return cutoff_unmake(
                            position,
                            real_depth,
                            ply,
                            search_state,
                            best_pathscore,
                            hash_height,
                            hash_version,
                            m,
                            move_is_capture,
                            captured_value,
                            hash_index,
                        );
                    }
                    hash_flag = Exact;
                }
                scout_search = true;
            }
        } else {
            unmake_move(position, m, &unmake);
        }
    }

    if legal_move_count == 0 {
        if in_check {
            best_pathscore.1 = -MATE_SCORE + ply as Score
        } else {
            best_pathscore.1 = draw_value(position, search_state)
        }
    };

    // Update correction history: learn the error between static eval and search result
    // Only update when we have a valid eval and the result isn't a mate score
    if !in_check && lazy_eval != -Score::MAX && best_pathscore.1.abs() < MATE_START {
        let raw_eval = lazy_eval - correction; // Remove old correction to get raw eval
        let error = best_pathscore.1 - raw_eval;
        let entry = &mut search_state.correction_history[correction_index];
        let current = *entry;
        // Gravity formula: blend towards observed error
        *entry = current + (error - current / CORRECTION_SCALE) * CORRECTION_WEIGHT / CORRECTION_SCALE;
    }

    store_hash_entry(
        position,
        real_depth,
        hash_height,
        hash_version,
        hash_flag,
        (best_pathscore.0[0], best_pathscore.1),
        search_state,
        ply,
        hash_index,
    );

    best_pathscore
}

#[inline(always)]
pub fn is_draw(position: &Position, search_state: &mut SearchState, ply: u8) -> bool {
    // Order checks from cheapest to most expensive (short-circuit evaluation)
    // 1. 50-move rule: single comparison
    // 2. Repetition: iterates history but usually short
    // 3. Insufficient material: piece counting, only at ply > 6
    position.half_moves >= 100
        || is_repeat_position(position, search_state)
        || (ply > 6
            && insufficient_material(
                position,
                (position.pieces[WHITE as usize].all_pieces_bitboard.count_ones()
                    + position.pieces[BLACK as usize].all_pieces_bitboard.count_ones()) as u8,
                true,
            ))
}

#[inline(always)]
fn is_repeat_position(position: &Position, search_state: &mut SearchState) -> bool {
    let mut repeats = 0;
    for lock in search_state.history.iter().rev().take(position.half_moves as usize) {
        if position.zobrist_lock == *lock {
            repeats += 1;
            if repeats > 1 {
                return true;
            }
        }
    }
    false
}

#[inline(always)]
#[allow(clippy::needless_range_loop)]
pub fn pick_high_score_move(move_scores: &mut MoveScoreArray) -> Move {
    let mut best_index = 0;
    let mut best_score = move_scores[0].1;
    for j in 1..move_scores.len() {
        if move_scores[j].1 > best_score {
            best_score = move_scores[j].1;
            best_index = j;
        }
    }

    move_scores.swap_remove(best_index).0
}

#[inline(always)]
fn lmr_scout_search(
    lmr: u8,
    ply: u8,
    search_state: &mut SearchState,
    window: Window,
    real_depth: u8,
    new_position: &mut Position,
) -> PathScore {
    let alpha = window.0;
    let beta = window.1;
    let mut scout_path = search_wrapper(real_depth, ply, search_state, (-alpha - 1, -alpha), new_position, lmr, 0);

    if scout_path.1 > alpha && lmr > 0 {
        // We are in an LMR search and we Need to research with full window. but still with late move reduction
        scout_path = search_wrapper(real_depth, ply, search_state, (-beta, -alpha), new_position, lmr, 0);
        if scout_path.1 > alpha {
            // Need to research with full window and no reduction
            scout_path = search_wrapper(real_depth, ply, search_state, (-beta, -alpha), new_position, 0, 0)
        }
    } else if scout_path.1 > alpha && scout_path.1 < beta {
        // Not doing a LMR search, but still need to research with a full window
        scout_path = search_wrapper(real_depth, ply, search_state, (-beta, -alpha), new_position, 0, 0)
    }

    scout_path
}

#[inline(always)]
fn make_null_move(position: &mut Position) -> Square {
    let old_ep = position.en_passant_square;
    if position.en_passant_square != EN_PASSANT_NOT_AVAILABLE {
        position.zobrist_lock ^= ZOBRIST_KEYS_EN_PASSANT[en_passant_zobrist_key_index(position.en_passant_square)];
        position.en_passant_square = EN_PASSANT_NOT_AVAILABLE;
    }
    position.mover ^= 1;
    position.zobrist_lock ^= ZOBRIST_KEY_MOVER_SWITCH;
    old_ep
}

#[inline(always)]
fn unmake_null_move(position: &mut Position, old_ep: Square) {
    position.mover ^= 1;
    position.zobrist_lock ^= ZOBRIST_KEY_MOVER_SWITCH;
    if old_ep != EN_PASSANT_NOT_AVAILABLE {
        position.en_passant_square = old_ep;
        position.zobrist_lock ^= ZOBRIST_KEYS_EN_PASSANT[en_passant_zobrist_key_index(old_ep)];
    }
}

#[inline(always)]
fn search_wrapper(
    depth: u8,
    ply: u8,
    search_state: &mut SearchState,
    window: Window,
    position: &mut Position,
    lmr: u8,
    excluded_move: Move,
) -> PathScore {
    search_state.history.push(position.zobrist_lock);
    let path_score = search(position, depth - 1 - lmr, ply + 1, window, search_state, false, excluded_move);
    search_state.history.pop();
    (path_score.0, -path_score.1)
}

#[inline(always)]
#[allow(clippy::too_many_arguments)]
fn cutoff_unmake(
    position: &Position,
    depth: u8,
    ply: u8,
    search_state: &mut SearchState,
    best_pathscore: PathScore,
    hash_height: u8,
    hash_version: u32,
    m: Move,
    is_capture: bool,
    captured_value: Score,
    hash_index: usize,
) -> PathScore {
    store_hash_entry(
        position,
        depth,
        hash_height,
        hash_version,
        Lower,
        (m, best_pathscore.1),
        search_state,
        ply,
        hash_index,
    );
    let bonus = (depth as i32) * (depth as i32);
    update_history(position, search_state, m, bonus as i64, captured_value);
    update_killers(ply, search_state, m, best_pathscore.1, is_capture);
    update_countermove(position, ply, search_state, m, is_capture, depth);
    // Update followup history with positive bonus (quiet moves only)
    update_followup_history_for_move(
        position,
        ply,
        search_state,
        m,
        bonus,
        captured_value,
        m & PROMOTION_FULL_MOVE_MASK != 0,
    );
    // Update capture history with positive bonus (captures only)
    update_capture_history_for_move(position, search_state, m, bonus, captured_value);
    best_pathscore
}

/// Update countermove table: store move m as a good response to the previous opponent's move
/// Also update countermove history with depth-based bonus
#[inline(always)]
fn update_countermove(position: &Position, ply: u8, search_state: &mut SearchState, m: Move, is_capture: bool, depth: u8) {
    // Only store quiet moves as countermoves (captures are handled by MVV-LVA)
    if is_capture || (m & PROMOTION_FULL_MOVE_MASK != 0) {
        return;
    }
    // Need a previous move to respond to
    if ply == 0 {
        return;
    }
    let prev_move = search_state.ply_move[(ply - 1) as usize];
    if prev_move == 0 {
        return;
    }
    // Index by [piece_12][to_square] of the previous move
    // The previous move was made by the opponent (position.mover ^ 1)
    let opponent_side = position.mover ^ 1;
    let prev_piece = piece_type_to_index(prev_move) + (opponent_side as usize * 6);
    let prev_to = to_square_part(prev_move) as usize;

    // Store as THE countermove
    search_state.countermoves[prev_piece][prev_to] = m;

    // Update countermove history with gravity formula
    let curr_piece = piece_type_to_index(m);
    let curr_to = to_square_part(m) as usize;
    let bonus = (depth as i32) * (depth as i32);
    update_countermove_history(search_state, prev_piece, prev_to, curr_piece, curr_to, bonus);
}

/// Update countermove history using gravity formula: history += bonus - history * |bonus| / 16384
/// This keeps values bounded without explicit clamping
#[inline(always)]
fn update_countermove_history(
    search_state: &mut SearchState,
    prev_piece: usize,
    prev_to: usize,
    curr_piece: usize,
    curr_to: usize,
    bonus: i32,
) {
    let entry = &mut search_state.countermove_history[prev_piece][prev_to][curr_piece][curr_to];
    let current = *entry as i32;
    // Gravity formula: new = current + bonus - current * |bonus| / 16384
    let abs_bonus = bonus.abs();
    let new_value = current + bonus - current * abs_bonus / 16384;
    // Clamp to i16 range
    *entry = new_value.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
}

/// Update countermove history for a move that failed to cause cutoff (negative bonus)
/// Called when a quiet move doesn't improve alpha
#[inline(always)]
fn update_countermove_history_for_move(
    position: &Position,
    ply: u8,
    search_state: &mut SearchState,
    m: Move,
    bonus: i32,
    captured_value: Score,
    is_promotion: bool,
) {
    // Only for quiet moves
    if captured_value != 0 || is_promotion {
        return;
    }
    if ply == 0 {
        return;
    }
    let prev_move = search_state.ply_move[(ply - 1) as usize];
    if prev_move == 0 {
        return;
    }
    let opponent_side = position.mover ^ 1;
    let prev_piece = piece_type_to_index(prev_move) + (opponent_side as usize * 6);
    let prev_to = to_square_part(prev_move) as usize;
    let curr_piece = piece_type_to_index(m);
    let curr_to = to_square_part(m) as usize;
    update_countermove_history(search_state, prev_piece, prev_to, curr_piece, curr_to, bonus);
}

/// Update followup history: tracks what works after our own move from 2 plies ago
#[inline(always)]
fn update_followup_history_for_move(
    _position: &Position,
    ply: u8,
    search_state: &mut SearchState,
    m: Move,
    bonus: i32,
    captured_value: Score,
    is_promotion: bool,
) {
    // Only for quiet moves
    if captured_value != 0 || is_promotion {
        return;
    }
    if ply < 2 {
        return;
    }
    let our_prev_move = search_state.ply_move[(ply - 2) as usize];
    if our_prev_move == 0 {
        return;
    }
    // Index by our move's piece (0-5) - we know it's our piece so no color offset needed
    let our_prev_piece = piece_type_to_index(our_prev_move);
    let our_prev_to = to_square_part(our_prev_move) as usize;
    let curr_piece = piece_type_to_index(m);
    let curr_to = to_square_part(m) as usize;

    let entry = &mut search_state.followup_history[our_prev_piece][our_prev_to][curr_piece][curr_to];
    let current = *entry as i32;
    // Gravity formula
    let abs_bonus = bonus.abs();
    let new_value = current + bonus - current * abs_bonus / 16384;
    *entry = new_value.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
}

/// Update capture history: tracks how well captures perform
/// Indexed by [attacker_piece][victim_piece][to_square]
#[inline(always)]
fn update_capture_history_for_move(_position: &Position, search_state: &mut SearchState, m: Move, bonus: i32, captured_value: Score) {
    if captured_value == 0 {
        return; // Not a capture
    }

    let attacker = piece_type_to_index(m);
    let victim = captured_piece_to_index(captured_value);
    let to_sq = to_square_part(m) as usize;

    let entry = &mut search_state.capture_history[attacker][victim][to_sq];
    let current = *entry as i32;
    // Gravity formula
    let abs_bonus = bonus.abs();
    let new_value = current + bonus - current * abs_bonus / 16384;
    *entry = new_value.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
}

/// Convert captured piece value to index 0-5
#[inline(always)]
fn captured_piece_to_index(value: Score) -> usize {
    use crate::engine_constants::*;
    if value == PAWN_VALUE_AVERAGE {
        0
    } else if value == KNIGHT_VALUE_AVERAGE {
        1
    } else if value == BISHOP_VALUE_AVERAGE {
        2
    } else if value == ROOK_VALUE_AVERAGE {
        3
    } else if value == QUEEN_VALUE_AVERAGE {
        4
    } else {
        5 // King (shouldn't happen)
    }
}

/// Convert move's piece mask to index 0-5
#[inline(always)]
fn piece_type_to_index(m: Move) -> usize {
    match m & PIECE_MASK_FULL {
        PIECE_MASK_PAWN => 0,
        PIECE_MASK_KNIGHT => 1,
        PIECE_MASK_BISHOP => 2,
        PIECE_MASK_ROOK => 3,
        PIECE_MASK_QUEEN => 4,
        PIECE_MASK_KING => 5,
        _ => 0,
    }
}

#[inline(always)]
fn update_history(position: &Position, search_state: &mut SearchState, m: Move, score: i64, captured_value: Score) {
    if captured_value == 0 {
        let f = from_square_part(m) as usize;
        let t = to_square_part(m) as usize;

        let piece_index = piece_index_12(position, m);

        search_state.history_moves[piece_index][f][t] += score;

        if search_state.history_moves[piece_index][f][t] < 0 {
            search_state.history_moves[piece_index][f][t] = 0;
        }

        halve_history_scores_if_required(search_state, f, t, piece_index)
    }
}

#[inline(always)]
fn halve_history_scores_if_required(search_state: &mut SearchState, f: usize, t: usize, piece_index: usize) {
    if search_state.history_moves[piece_index][f][t] > search_state.highest_history_score {
        search_state.highest_history_score = search_state.history_moves[piece_index][f][t];
        if search_state.highest_history_score > (i64::MAX / 2) {
            for i in 0..12 {
                for j in 0..64 {
                    for k in 0..64 {
                        search_state.history_moves[i][j][k] /= 2;
                    }
                }
            }
        }
    }
}

#[inline(always)]
pub fn piece_index_12(position: &Position, m: Move) -> usize {
    ((position.mover * 6)
        + match m & PIECE_MASK_FULL {
            PIECE_MASK_PAWN => 0,
            PIECE_MASK_KNIGHT => 1,
            PIECE_MASK_BISHOP => 2,
            PIECE_MASK_ROOK => 3,
            PIECE_MASK_QUEEN => 4,
            PIECE_MASK_KING => 5,
            _ => {
                panic!("Expected a valid piece mask.");
            }
        }) as usize
}

#[inline(always)]
fn update_killers(ply: u8, search_state: &mut SearchState, m: Move, score: Score, is_capture: bool) {
    if score > MATE_START {
        search_state.mate_killer[ply as usize] = m;
    }
    if search_state.killer_moves[ply as usize][0] != m && (m & PROMOTION_FULL_MOVE_MASK == 0) && !is_capture {
        search_state.killer_moves[ply as usize][1] = search_state.killer_moves[ply as usize][0];
        search_state.killer_moves[ply as usize][0] = m;
    }
}

#[inline(always)]
fn null_move_material(position: &Position) -> bool {
    side_total_non_pawn_count(position, position.mover) + side_total_non_pawn_count(position, opponent!(position.mover)) >= 2
}

#[inline(always)]
fn side_total_non_pawn_count(position: &Position, side: Mover) -> u32 {
    (position.pieces[side as usize].bishop_bitboard
        | position.pieces[side as usize].knight_bitboard
        | position.pieces[side as usize].rook_bitboard
        | position.pieces[side as usize].queen_bitboard)
        .count_ones()
}
