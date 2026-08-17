use crate::engine_constants::{
    lmr_reduction, ALPHA_PRUNE_MARGINS, ASPIRATION_RADIUS, BETA_PRUNE_MARGIN_PER_DEPTH, BETA_PRUNE_MAX_DEPTH, CORRECTION_HISTORY_GRAIN,
    CORRECTION_HISTORY_MAX, CORRECTION_HISTORY_SIZE, CORRECTION_HISTORY_WEIGHT_MAX, HISTORY_MAX, LMP_MAX_DEPTH, LMP_MOVE_THRESHOLDS,
    LMR_CONTINUATION_BAD_THRESHOLD, LMR_CONTINUATION_GOOD_THRESHOLD, LMR_HISTORY_BAD_THRESHOLD, LMR_HISTORY_GOOD_THRESHOLD,
    LMR_LEGAL_MOVES_BEFORE_ATTEMPT, LMR_MIN_DEPTH, MAX_DEPTH, MAX_QUIESCE_DEPTH, MULTICUT_DEPTH_REDUCTION, MULTICUT_MIN_DEPTH,
    MULTICUT_MOVES_TO_TRY, MULTICUT_REQUIRED_CUTOFFS, NULL_MOVE_MIN_DEPTH, NULL_MOVE_REDUCE_DEPTH_BASE, PROBCUT_DEPTH_REDUCTION,
    PROBCUT_MARGIN, PROBCUT_MIN_DEPTH, RAZOR_MARGINS, RAZOR_MAX_DEPTH, ROOK_VALUE_AVERAGE, SEE_PRUNE_MARGIN, SEE_PRUNE_MAX_DEPTH,
    SINGULAR_EXTENSION_DEPTH_MARGIN, SINGULAR_EXTENSION_DEPTH_REDUCTION, SINGULAR_EXTENSION_MARGIN_MULTIPLIER,
    SINGULAR_EXTENSION_MIN_DEPTH, THREAT_EXTENSION_MARGIN, TM_INSTABILITY_EXTEND, TM_ITERATION_GROWTH, TM_MAX_EXTENSION_FACTOR,
    TM_MIN_DEPTH_FOR_TM, TM_SCORE_DROP_EXTEND, TM_SCORE_DROP_THRESHOLD, TM_STABILITY_THRESHOLD,
};
use crate::evaluate::{evaluate_position, insufficient_material, pawn_material, piece_material};
use arrayvec::ArrayVec;

/// Make a move with lazy NNUE tracking. Saves pieces state and advances ply,
/// but defers accumulator computation until evaluate_position is called.
#[inline(always)]
pub fn make_move_nnue(position: &mut Position, mv: Move, search_state: &mut SearchState) -> UnmakeInfo {
    let unmake = make_move_in_place(position, mv);
    if search_state.use_nnue {
        search_state.nnue_ply += 1;
        search_state.nnue_pieces[search_state.nnue_ply] = position.pieces;
        search_state.nnue_computed[search_state.nnue_ply] = false;
    }
    unmake
}

/// Unmake a move and restore NNUE ply.
#[inline(always)]
pub fn unmake_move_nnue(position: &mut Position, mv: Move, unmake: &UnmakeInfo, search_state: &mut SearchState) {
    unmake_move(position, mv, unmake);
    if search_state.use_nnue {
        search_state.nnue_ply -= 1;
    }
}

/// Add deterministic noise to an eval score based on position hash.
/// Returns the score with noise in range [-noise_max, +noise_max].
/// Uses the zobrist hash to produce consistent noise for the same position.
#[inline(always)]
fn apply_eval_noise(score: Score, hash: u64, noise_max: Score) -> Score {
    if noise_max == 0 {
        return score;
    }
    // Simple hash-derived noise: take bits from hash, map to [-noise_max, noise_max]
    let noise_bits = (hash >> 17) as i32; // Arbitrary bit selection
    let noise = (noise_bits % (2 * noise_max + 1)) - noise_max;
    score + noise
}

use crate::fen::algebraic_move_from_move;
use crate::tablebase::{probe_dtz, tablebase_available, TB_MAX_PIECES};

use crate::bitboards::{bit, north_fill, south_fill, FILE_A_BITS, FILE_H_BITS};
use crate::hash::{en_passant_zobrist_key_index, ZOBRIST_KEYS_EN_PASSANT, ZOBRIST_KEY_MOVER_SWITCH};
use crate::make_move::{make_move_in_place, unmake_move, CAPTURED_NONE};
use crate::move_constants::{
    EN_PASSANT_NOT_AVAILABLE, PIECE_MASK_BISHOP, PIECE_MASK_FULL, PIECE_MASK_KING, PIECE_MASK_KNIGHT, PIECE_MASK_PAWN, PIECE_MASK_QUEEN,
    PIECE_MASK_ROOK, PROMOTION_FULL_MOVE_MASK,
};
use crate::move_scores::{score_move, score_move_with_see, victim_piece_index};
use crate::moves::{generate_captures, generate_check_evasions, generate_moves, generate_quiet_moves, is_check, verify_move};
use crate::opponent;
use crate::quiesce::quiesce;
use crate::see::static_exchange_evaluation;
use crate::types::BoundType::{Exact, Lower, Upper};
use crate::types::{
    is_stopped, pv_prepend, pv_single, set_stop, BoundType, HashEntry, Move, MoveList, MoveScore, MoveScoreArray, MoveScoreList, Mover,
    PathScore, Position, Score, SearchState, Square, UnmakeInfo, Window, BLACK, STATIC_EVAL_NONE, WHITE,
};
use crate::utils::{captured_piece_value, from_square_part, send_info, to_square_part};
use std::cmp::{max, min};
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

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
                    let now = std::time::Instant::now();
                    if hard_ms > 0 {
                        $search_state.end_time = now + std::time::Duration::from_millis(hard_ms);
                        $search_state.soft_time_limit = now + std::time::Duration::from_millis(soft_ms);
                        // Rebase the extension ceiling on the REAL soft budget
                        // (NET-362): the value computed at search start came from
                        // the 24h ponder placeholder, which would disable the
                        // NET-339 cap for the whole pondered move.
                        $search_state.max_soft_time_limit =
                            now + std::time::Duration::from_millis(soft_ms).mul_f64($crate::engine_constants::TM_MAX_EXTENSION_FACTOR);
                        // Exact budgets (movetime, emergency: soft == hard) must run
                        // to the deadline, matching the non-ponder exact path — TM's
                        // predictive cutoff would otherwise stop a commanded
                        // movetime as early as ~1/2.8 of it (NET-362). Clock-derived
                        // budgets (soft < hard) get full time management.
                        $search_state.time_management_active = soft_ms != hard_ms;
                    } else {
                        // Safety net: ponderhit on a search that was never given a
                        // budget must still terminate. Stop now and return the best
                        // move found so far rather than hang the engine (time forfeit).
                        $search_state.end_time = now;
                        $search_state.soft_time_limit = now;
                        $search_state.max_soft_time_limit = now;
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

/// Whether another iterative-deepening iteration can be started without
/// overrunning the soft time limit, given how long the last one took.
///
/// Pure and public so the decision is regression-testable without running a
/// search. This matters: the NET-339 time-forfeit bug was invisible to every
/// existing timing test because they all asserted against the HARD limit, which
/// the engine honours to within a millisecond, while the soft limit - the budget
/// the engine is actually supposed to spend - was being exceeded ~4x.
pub fn next_iteration_fits(last_iteration: Duration, remaining_to_soft: Duration, growth: f64) -> bool {
    last_iteration.mul_f64(growth) <= remaining_to_soft
}

pub fn iterative_deepening(position: &mut Position, max_depth: u8, search_state: &mut SearchState, start_depth: u8) -> Move {
    search_state.start_time = Instant::now();
    // NOTE: the stop flag is NOT reset here. Each real `go` creates a fresh stop
    // flag (see cmd_go), so resetting here would race with a stop/quit/second-go
    // that arrived before this line ran, erasing the request and hanging the
    // engine in join(). Callers that reuse a stop flag across searches (cmd_go_sync,
    // the mvm self-play loop) reset it themselves before each call.
    // Advance the shared TT generation once per search - helper threads share
    // the table and must not bump it again
    if search_state.thread_id == 0 {
        search_state.hash_table.bump_version();
    }
    search_state.best_move_stability = 0;
    search_state.prev_best_move = 0;
    search_state.prev_score = 0;

    // Initialize NNUE accumulator from root position
    if search_state.use_nnue {
        if let Some(ref net) = search_state.nnue_network {
            search_state.nnue_ply = 0;
            search_state.nnue_accumulators[0].compute(net, position);
            search_state.nnue_pieces[0] = position.pieces;
            search_state.nnue_computed[0] = true;
        }
    }

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

    // Tablebase probe at root: if ≤6 pieces, return best move immediately.
    // Only for real timed moves (time_management_active): a ponder or infinite
    // search must NOT return bestmove before ponderhit/stop, or thread 0 emits an
    // unsolicited bestmove (UCI violation / desync — e.g. Ponder + SyzygyPath on
    // Lichess into any ≤6-man position). Those searches fall through to the normal
    // search, which still handles the TB position correctly and respects stop.
    let all_pieces = position.pieces[WHITE as usize].all_pieces_bitboard | position.pieces[BLACK as usize].all_pieces_bitboard;
    if search_state.time_management_active && tablebase_available() && all_pieces.count_ones() <= TB_MAX_PIECES {
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

    decay_history_tables(search_state);
    clear_killers(search_state);

    if search_state.history.is_empty() {
        search_state.history.push(position.zobrist_lock)
    }

    let mut aspiration_window = (-MAX_WINDOW, MAX_WINDOW);

    // Initialize with the first legal move so we always have a valid move to return
    // even if time expires before the first search iteration completes
    search_state.current_best = (pv_single(legal_moves[0].0), -MATE_SCORE);

    let mut prev_synced_nodes: u64 = 0;

    // Ceiling for the cumulative soft-limit extension, fixed against the ORIGINAL
    // soft budget before any iteration can move it (NET-339). Without this the
    // per-iteration extensions compound until soft == hard.
    //
    // Stored on SearchState (NET-362): for a pondered search the soft limit here
    // is the 24h placeholder, so this initial value is meaningless — check_time!
    // rebases the ceiling on the REAL budget when ponderhit installs it.
    search_state.max_soft_time_limit = search_state.start_time
        + search_state
            .soft_time_limit
            .saturating_duration_since(search_state.start_time)
            .mul_f64(TM_MAX_EXTENSION_FACTOR);

    for iterative_depth in start_depth..=max_depth {
        //println!("Iterative depth {}", iterative_depth);
        let mut c = 0;
        search_state.iterative_depth = iterative_depth;
        // Cost of this iteration, used to predict whether the next one fits in
        // the soft budget. Includes any aspiration re-searches, which is what we
        // want - a re-search storm is exactly the case that overruns.
        let iteration_start = Instant::now();

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
                search_state.soft_time_limit = min(min(extended, search_state.max_soft_time_limit), search_state.end_time);
            }

            // Extend soft limit on score drop
            if score_dropped && now < search_state.end_time {
                let remaining_soft = search_state.soft_time_limit.saturating_duration_since(now);
                let extension = remaining_soft.mul_f64(TM_SCORE_DROP_EXTEND - 1.0);
                let extended = search_state.soft_time_limit + extension;
                search_state.soft_time_limit = min(min(extended, search_state.max_soft_time_limit), search_state.end_time);
            }

            // Past soft limit (possibly extended) — stop
            if now >= search_state.soft_time_limit {
                set_stop(&search_state.stop, true);
                break;
            }
        }

        // Predictive iteration cutoff (NET-339). Everything above only decides
        // whether we are ALREADY past the soft limit; none of it can stop an
        // iteration once it has begun, because check_time!/time_expired! test
        // end_time (the hard limit) and nothing tests soft_time_limit inside the
        // search. So an iteration started just under the soft limit runs to the
        // hard limit instead - ~4.2x the intended budget. That is what lost games
        // on the clock. Predict the next iteration from the cost of this one and
        // decline to start it if it cannot finish in time.
        //
        // Deliberately not gated on TM_MIN_DEPTH_FOR_TM: shallow iterations cost
        // microseconds, so the prediction cannot fire early enough to matter, and
        // current_best always holds a legal move from before the loop.
        if search_state.time_management_active && search_state.thread_id == 0 {
            let now = Instant::now();
            let last_iteration = now.saturating_duration_since(iteration_start);
            let remaining_to_soft = search_state.soft_time_limit.saturating_duration_since(now);
            if !next_iteration_fits(last_iteration, remaining_to_soft, TM_ITERATION_GROWTH) {
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

    // Return the move whose score was actually validated, not whichever sorts
    // first (NET-610 review). These agree today - a probe over bench and the
    // whole suite found no divergence - but only by an invariant nothing
    // enforces: current_best takes the maximum mv.1, and the sort orders by
    // mv.1, so index 0 happens to be the same move.
    //
    // Root PVS makes that coupling fragile. mv.1 is now heterogeneous: an exact
    // score for the first move and for any scout that was re-searched, a
    // fail-low bound for every scout that was not. Sorting exact scores against
    // bounds is already only approximately meaningful for move ordering, which
    // is a use that tolerates being wrong. Choosing the returned move is not.
    //
    // The time-expiry path a few lines up already returns current_best, so this
    // also removes a state/return mismatch between the two exits.
    search_state.current_best.0[0]
}

pub fn start_search(position: &mut Position, legal_moves: &mut MoveScoreList, search_state: &mut SearchState, window: Window) -> PathScore {
    let mut current_best: PathScore = (pv_single(legal_moves[0].0), window.0);
    let hash_mask = search_state.hash_table.mask();
    let mut first_root_move = true;

    for mv in legal_moves {
        let unmake = make_move_nnue(position, mv.0, search_state);
        prefetch_hash(position, search_state, hash_mask); // Prefetch child position's hash entry
        search_state.history.push(position.zobrist_lock);

        // Principal-variation search at the root (NET-610). Before this, every
        // root child got the full aspiration window, which made each of them a
        // PV node - and since a PV node's first child also gets a full window,
        // the whole principal variation was searched with `scouting == false`
        // and therefore with razoring, null-move, probcut, multicut and the
        // futility prunes all switched off. That is a very expensive way to
        // protect the PV: it cost about 12% of the tree.
        //
        // Scouting the non-first root moves exposed a pre-existing unsoundness
        // rather than creating one - see the razoring note in RAZOR_MAX_DEPTH.
        let mut path_score = if first_root_move {
            search(
                position,
                search_state.iterative_depth - 1,
                1,
                (-window.1, -current_best.1),
                search_state,
                false,
                0,
                None,
            )
        } else {
            // Null window around the current best. Note the asymmetry: from
            // the child's point of view this is a fail-LOW test. The child
            // returns the negated score, so "this root move beats the current
            // best" means the child scored at or below its own alpha.
            let scout = search(
                position,
                search_state.iterative_depth - 1,
                1,
                (-current_best.1 - 1, -current_best.1),
                search_state,
                false,
                0,
                None,
            );
            if -scout.1 > current_best.1 {
                search(
                    position,
                    search_state.iterative_depth - 1,
                    1,
                    (-window.1, -current_best.1),
                    search_state,
                    false,
                    0,
                    None,
                )
            } else {
                scout
            }
        };
        first_root_move = false;
        path_score.1 = -path_score.1;

        search_state.history.pop();
        unmake_move_nnue(position, mv.0, &unmake, search_state);
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

pub fn clear_killers(search_state: &mut SearchState) {
    for i in 0..MAX_DEPTH as usize {
        search_state.mate_killer[i] = 0;
        for j in 0..2 {
            search_state.killer_moves[i][j] = 0;
        }
    }
}

pub fn clear_history_table(search_state: &mut SearchState) {
    for piece in search_state.history_moves.iter_mut() {
        for from_sq in piece.iter_mut() {
            from_sq.fill(0);
        }
    }

    // Clear countermove history table
    for prev_piece in search_state.countermove_history.iter_mut() {
        for prev_to in prev_piece.iter_mut() {
            for curr_piece in prev_to.iter_mut() {
                curr_piece.fill(0);
            }
        }
    }

    // Clear followup history table
    for prev_piece in search_state.followup_history.iter_mut() {
        for prev_to in prev_piece.iter_mut() {
            for curr_piece in prev_to.iter_mut() {
                curr_piece.fill(0);
            }
        }
    }

    // Clear capture history table
    for attacker in &mut search_state.capture_history {
        for victim in attacker {
            victim.fill(0);
        }
    }

    // Clear eval-correction tables (they persist across searches otherwise -
    // the EMA re-converges within a game but must not leak between games)
    for side in search_state.correction_history.iter_mut() {
        side.fill(0);
    }
}

/// Countermoves are the one move-ordering table with no reset at all: the
/// history tables are decayed per search and cleared per game, the killers are
/// cleared per search, but a countermove learned in the first game is still
/// steering move ordering in the fiftieth.
///
/// That is deliberately *not* done by `clear_history_table`, so `ucinewgame`
/// keeps the warm table: clearing it costs 12% more nodes on `bench`, which
/// runs sixteen unrelated positions and so pays the cold-table price fifteen
/// times over. Whether a real match would rather have the clean table is an
/// open question (NET-396) and needs a match to answer, not a node count.
///
/// Self-play datagen calls this per game regardless, because there the warmth
/// is a liability: it makes a game's labels depend on which *other* games that
/// worker happened to be handed, which is both a reproducibility hole and a
/// source of label noise.
pub fn clear_countermoves(search_state: &mut SearchState) {
    for prev_piece in search_state.countermoves.iter_mut() {
        prev_piece.fill(0);
    }
}

/// Halve every history table at the start of a search: old signal decays but
/// still seeds move ordering for the early iterations of the next search
fn decay_history_tables(search_state: &mut SearchState) {
    for piece in search_state.history_moves.iter_mut() {
        for from_sq in piece.iter_mut() {
            for entry in from_sq.iter_mut() {
                *entry /= 2;
            }
        }
    }
    for prev_piece in search_state.countermove_history.iter_mut() {
        for prev_to in prev_piece.iter_mut() {
            for curr_piece in prev_to.iter_mut() {
                for entry in curr_piece.iter_mut() {
                    *entry /= 2;
                }
            }
        }
    }
    for prev_piece in search_state.followup_history.iter_mut() {
        for prev_to in prev_piece.iter_mut() {
            for curr_piece in prev_to.iter_mut() {
                for entry in curr_piece.iter_mut() {
                    *entry /= 2;
                }
            }
        }
    }
    for attacker in &mut search_state.capture_history {
        for victim in attacker {
            for entry in victim.iter_mut() {
                *entry /= 2;
            }
        }
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
    static_eval: Score,
) {
    let table_version = search_state.hash_table.version();
    // Replace when at least as deep, or when the incumbent is from an older
    // search (real aging - stale entries no longer block stores forever)
    if height >= existing_height || table_version != existing_version {
        search_state.hash_table.store(
            hash_index,
            HashEntry {
                // adjust any mate score so that the score appears calculated as if this ply were the root
                score: match movescore.1 {
                    x if x > MATE_START => movescore.1 + ply as Score,
                    x if x < -MATE_START => movescore.1 - ply as Score,
                    _ => movescore.1,
                },
                version: table_version,
                height,
                mv: movescore.0,
                bound,
                lock: position.zobrist_lock,
                static_eval,
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
fn prefetch_hash(position: &Position, search_state: &SearchState, hash_mask: u64) {
    let index = (position.zobrist_lock as u64 & hash_mask) as usize;
    search_state.hash_table.prefetch(index);
}

#[inline(always)]
#[allow(clippy::too_many_arguments)]
pub fn search(
    position: &mut Position,
    depth: u8,
    ply: u8,
    window: Window,
    search_state: &mut SearchState,
    on_null_move: bool,
    excluded_move: Move, // For singular extension search: skip this move (0 = no exclusion)
    // Whether the side to move is in check, when the caller already knows it
    // (NET-355): the parent's gives_check IS the child's in_check, so passing
    // it down saves a redundant is_check at nearly every node entry.
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

    if is_draw(position, search_state, ply) {
        search_state.nodes += 1;
        return (pv_single(0), draw_value(position, search_state));
    }

    // Prevent stack overflow and array out of bounds at extreme depths
    if ply >= MAX_DEPTH {
        let eval = evaluate_position(position, search_state);
        return (
            pv_single(0),
            apply_eval_noise(eval, position.zobrist_lock as u64, search_state.eval_noise),
        );
    }

    // NOTE: Tablebase probing during search is disabled for performance.
    // The position_to_chess() conversion is too expensive to call at every node.
    // Instead, we probe DTZ at the root level in iterative_deepening to select
    // the best move in TB positions.

    let scouting = window.1 - window.0 == 1;

    if depth == 0 {
        return quiesce(position, MAX_QUIESCE_DEPTH, ply, window, search_state, known_in_check);
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
    // Children actually searched at THIS node (NET-493). Distinct from
    // legal_move_count, which increments before alpha/LMP pruning.
    let mut children_here: u32 = 0;
    let mut hash_flag = Upper;
    let mut best_pathscore: PathScore = (pv_single(0), -MATE_SCORE);

    let hash_mask = search_state.hash_table.mask();
    let hash_index: usize = (position.zobrist_lock as u64 & hash_mask) as usize;
    // Checksum-verified probe: None on miss, foreign entry, or torn write
    let probed_entry = search_state.hash_table.probe(hash_index, position.zobrist_lock);
    // Whatever occupies the slot (matching or not) informs the replacement
    // decision for stores from this node
    let (hash_height, hash_version, slot_occupied) = search_state.hash_table.entry_meta(hash_index);
    // Track hash entry info for singular extension (needed even if depth isn't sufficient for cutoff)
    search_state.tt_probes += 1;
    match probed_entry {
        Some(e) => {
            search_state.tt_hits += 1;
            if e.height >= depth {
                search_state.tt_deep_enough += 1;
            }
        }
        // Occupied by a different position - the slot exists, the entry we
        // wanted is gone. Distinguishes "table too small / bad replacement"
        // from "genuinely never searched this".
        None if slot_occupied => search_state.tt_slot_taken += 1,
        None => {}
    }

    let hash_entry_height = probed_entry.map_or(0, |e| e.height);
    let hash_entry_bound = probed_entry.map_or(Upper, |e| e.bound);
    let hash_move = if let Some(hash_entry) = probed_entry {
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
            if hash_entry.bound == Exact {
                search_state.hash_hits_exact += 1;
                // The checksum rules out torn entries, but a TT move still
                // never enters the PV unverified
                let pv_mv = if hash_entry.mv != 0 && verify_move(position, hash_entry.mv) {
                    hash_entry.mv
                } else {
                    0
                };
                return (pv_single(pv_mv), score);
            }
            if hash_entry.bound == Lower && score > alpha {
                alpha = score;
            }
            if hash_entry.bound == Upper && score < beta {
                beta = score
            }
            if alpha >= beta {
                let pv_mv = if hash_entry.mv != 0 && verify_move(position, hash_entry.mv) {
                    hash_entry.mv
                } else {
                    0
                };
                return (pv_single(pv_mv), score);
            }
        }
        hash_entry.mv
    } else if slot_occupied {
        search_state.hash_clashes += 1;
        0
    } else {
        0
    };

    let in_check = known_in_check.unwrap_or_else(|| is_check(position, position.mover));

    // Static eval for this node (correction-history adjusted), recorded per
    // ply so descendants can compute the improving flag; there is no
    // meaningful static eval while in check
    // Raw static eval, reused from the TT when this slot already holds one.
    // static_eval is depth-independent, so it is valid even when the entry's
    // height was too shallow to permit a score cutoff above.
    let raw_static_eval: Score = if in_check {
        STATIC_EVAL_NONE
    } else {
        match probed_entry {
            Some(e) if e.static_eval != STATIC_EVAL_NONE => e.static_eval,
            _ => evaluate_position(position, search_state),
        }
    };

    let lazy_eval: Score = if in_check {
        -Score::MAX
    } else {
        // Correction history is re-applied on every read rather than cached, so
        // a stale correction from an earlier search can never be resurrected.
        raw_static_eval + eval_correction(position, search_state)
    };
    search_state.eval_stack[ply as usize] = lazy_eval;

    // Improving: our static eval is better than it was 2 plies ago (same side
    // to move); fall back 4 plies when 2 back was in check, default true when
    // unknown so pruning stays conservative
    let improving = if in_check {
        false
    } else if ply >= 2 && search_state.eval_stack[ply as usize - 2] != -Score::MAX {
        lazy_eval > search_state.eval_stack[ply as usize - 2]
    } else if ply >= 4 && search_state.eval_stack[ply as usize - 4] != -Score::MAX {
        lazy_eval > search_state.eval_stack[ply as usize - 4]
    } else {
        true
    };

    if scouting && depth <= BETA_PRUNE_MAX_DEPTH && !in_check && beta.abs() < MATE_START {
        // An improving eval supports the fail-high conclusion, so a smaller
        // margin suffices; when not improving demand more
        let margin = BETA_PRUNE_MARGIN_PER_DEPTH * (depth as Score - improving as Score);
        if lazy_eval - margin >= beta {
            return (pv_single(0), lazy_eval - margin);
        }
    }

    // Razoring: at very shallow depth a static eval far below alpha usually means
    // the node is hopeless. Verify before failing low - an unverified cut here
    // would drop real tactics.
    //
    // The verification used to be quiescence alone, and that is unsound in a
    // specific, structural way (NET-618). `quiesce_moves` generates captures,
    // and check evasions when already in check. It never generates a quiet
    // move. So quiescence cannot refute the razor whenever the refutation ends
    // in a quiet move - and "quiet" includes a quiet check, and a quiet mate.
    //
    // That is not a rare shape. A sacrifice looks lost to the static eval
    // (material is down) and lost to quiescence (the recaptures resolve badly),
    // while the point of the combination is a quiet move two plies later. The
    // measured case is Re1! Rxe1 Rxe1 Rc1 Qg2#, where g2 is empty, so Qg2 is
    // quiet and quiescence never generates it at all.
    //
    // The rule this gives: razor only where a verification that can generate
    // quiet moves exists. From depth 2 up that is a reduced-depth null-window
    // search - it generates every move, so a quiet refutation is visible to it,
    // and at depth - 1 it is still far cheaper than searching the node in full,
    // which is the point of razoring.
    //
    // At depth 1 no such verification exists: depth - 1 IS quiescence, so there
    // is nothing cheaper that can see a quiet move. Razoring at depth 1 is
    // therefore unverifiable by construction, and it is the case that actually
    // hid the mate - measured, by gating depth 1 alone. So depth 1 does not
    // razor. That is not a special case for the test position; it is the same
    // rule applied where it happens to exclude everything.
    if scouting && !in_check && (2..=RAZOR_MAX_DEPTH).contains(&depth) && alpha.abs() < MATE_START && excluded_move == 0 {
        let razor_margin = RAZOR_MARGINS[depth as usize];
        if lazy_eval + razor_margin < alpha {
            // This block is !in_check-gated, so the verification knows its
            // in_check exactly.
            // depth - 1, not something shallower. Two cheaper variants were
            // measured and both cost MORE nodes overall, because a verification
            // that cuts less often is paid for and then the node is searched in
            // full anyway:
            //   depth - 1 verification            3,082,896
            //   quiescence pre-filter first       3,152,526
            //   always depth 1 when depth >= 3    3,502,601
            let verified = search(position, depth - 1, ply, (alpha - 1, alpha), search_state, false, 0, Some(false)).1;
            if is_stopped(&search_state.stop) {
                return (pv_single(0), 0);
            }
            if verified < alpha {
                return (pv_single(0), verified);
            }
        }
    }

    let alpha_prune_flag = if depth <= ALPHA_PRUNE_MARGINS.len() as u8 && scouting && !in_check && alpha.abs() < MATE_START {
        lazy_eval + ALPHA_PRUNE_MARGINS[depth as usize - 1] < alpha
    } else {
        false
    };

    // Threat detection: when null move fails badly, opponent has a dangerous threat
    // We'll use this to reduce LMR aggressiveness rather than extending
    let mut threat_detected = false;

    if !on_null_move && scouting && depth >= NULL_MOVE_MIN_DEPTH && null_move_material(position) && !in_check {
        let old_ep = make_null_move(position);
        // No real previous move exists for the null-move child's countermove context
        search_state.ply_move[ply as usize] = 0;
        // NOTE: deliberately NOT pushing the null position onto the repetition
        // history. Positions either side of a null move do not form a legal
        // sequence, so matching across one yields a FICTITIOUS repetition.
        // Standard practice (Stockfish et al.) is the opposite of pushing:
        // bound the scan so it cannot cross a null move at all, via a
        // plies-from-null counter. This engine has no such bound yet, so the
        // scan already crosses nulls; pushing here made that worse and
        // measured -19 (+/-25) over 200 games. See NET-367.

        let score = -search(
            position,
            null_move_reduced_depth(depth),
            ply + 1,
            (-beta, (-beta) + 1),
            search_state,
            true,
            0,
            // After a null move the side to move is the side that did NOT just
            // move in the (legality-checked) parent, which cannot be in check
            Some(false),
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
            let unmake = make_move_nnue(position, m, search_state);
            prefetch_hash(position, search_state, hash_mask);
            search_state.ply_move[ply as usize] = m;

            if !is_check(position, old_mover) {
                // Real-move child: push the lock like every other recursion
                // does, or this subtree cannot see repetitions correctly and
                // the position is absent from the stack throughout (NET-367)
                search_state.history.push(position.zobrist_lock);
                let score = -search(
                    position,
                    probcut_depth,
                    ply + 1,
                    (-probcut_beta, -probcut_beta + 1),
                    search_state,
                    false,
                    0,
                    None,
                )
                .1;
                search_state.history.pop();

                unmake_move_nnue(position, m, &unmake, search_state);

                if is_stopped(&search_state.stop) {
                    return (pv_single(0), 0);
                }

                if score >= probcut_beta {
                    return (pv_single(0), beta);
                }
            } else {
                unmake_move_nnue(position, m, &unmake, search_state);
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
        let mut scored_captures: MoveScoreArray = MoveScoreArray::new();
        for m in captures {
            scored_captures.push((m, score_move(position, m, search_state, ply as usize, enemy)));
        }

        // Sort by score descending to try best captures first
        scored_captures.sort_by_key(|b| std::cmp::Reverse(b.1));

        for (m, _) in scored_captures.iter().take(MULTICUT_MOVES_TO_TRY as usize) {
            let old_mover = position.mover;
            let unmake = make_move_nnue(position, *m, search_state);
            prefetch_hash(position, search_state, hash_mask);
            search_state.ply_move[ply as usize] = *m;

            if !is_check(position, old_mover) {
                // Real-move child: see the probcut note above (NET-367)
                search_state.history.push(position.zobrist_lock);
                let score = -search(position, multicut_depth, ply + 1, (-beta, -beta + 1), search_state, false, 0, None).1;
                search_state.history.pop();

                unmake_move_nnue(position, *m, &unmake, search_state);

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
                unmake_move_nnue(position, *m, &unmake, search_state);
            }
        }
    }

    let mut scout_search = false;

    // Quiets and captures already searched at this node without cutting off -
    // they get a history malus if a later move turns out to be the best one
    let mut searched_quiets: MoveList = MoveList::new();
    let mut searched_captures: MoveScoreArray = MoveScoreArray::new();

    let verified_hash_move = hash_move != 0 && verify_move(position, hash_move);

    // Check extension: extend by 1 ply when in check
    let check_extension: u8 = if in_check && (ply as u16) < (search_state.iterative_depth as u16) * 2 {
        1
    } else {
        0
    };
    let real_depth = depth + check_extension;

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
            hash_move,      // Exclude hash move from this search
            Some(in_check), // Same position, already computed
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
        let unmake = make_move_nnue(position, hash_move, search_state);
        prefetch_hash(position, search_state, hash_mask); // Prefetch child position's hash entry
        search_state.ply_move[ply as usize] = hash_move;
        let hash_is_capture = unmake.captured_piece != CAPTURED_NONE;

        if !is_check(position, old_mover) {
            legal_move_count += 1;
            // The hash move is searched here, outside the staged move loop, so
            // it must be counted here too (NET-493). Missing this undercounted
            // every no-cut node by one, and silently dropped from no_cutoff_nodes any
            // node whose only searched child was the hash move - which biased
            // the reported children-per-no-cutoff-node figure. Caught in review.
            children_here += 1;
            search_state.children_searched += 1;
            let hash_search_depth = real_depth + singular_extension;
            let path_score = search_wrapper(hash_search_depth, ply, search_state, (-beta, -alpha), position, 0, 0, None);
            let score = path_score.1;
            let singular_depth = hash_search_depth;

            unmake_move_nnue(position, hash_move, &unmake, search_state);
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
                            excluded_move,
                            &searched_quiets,
                            &searched_captures,
                            lazy_eval,
                            raw_static_eval,
                            children_here.min(u8::MAX as u32) as u8,
                            true,
                        );
                    }
                    hash_flag = Exact;
                }
                scout_search = true;
            }
            if hash_captured_value == 0 && hash_move & PROMOTION_FULL_MOVE_MASK == 0 {
                searched_quiets.push(hash_move);
            } else if hash_captured_value != 0 {
                searched_captures.push((hash_move, hash_captured_value));
            }
        } else {
            unmake_move_nnue(position, hash_move, &unmake, search_state);
        }
    }

    // MOVE GENERATION: Use check evasions when in check, staged generation otherwise
    let enemy = position.pieces[opponent!(position.mover) as usize];
    let mut move_scores: MoveScoreArray;
    let mut quiets_added: bool;
    // SEE-losing captures, deferred to a third stage behind the quiets (NET-229).
    // Previously captures were one stage consumed in full before any quiet was
    // generated, so a losing capture was always searched before killers and
    // high-history quiets - scoring alone could not fix that, because the
    // staging, not the score, decided the order.
    // Entries carry (move, ordering score, SEE) - the staging SEE is reused by
    // SEE pruning at pick time instead of recomputing it (NET-353).
    let mut bad_captures: ArrayVec<(Move, Score, Score), 256> = ArrayVec::new();
    let mut bad_captures_added: bool = false;

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
        move_scores = MoveScoreArray::new();
        for m in evasions {
            move_scores.push((m, score_move(position, m, search_state, ply as usize, &enemy)));
        }
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
        move_scores = MoveScoreArray::new();
        for m in captures {
            let (score, see_score) = score_move_with_see(position, m, search_state, ply as usize, &enemy);
            // Promotions change material too much to demote on SEE alone
            if see_score < 0 && m & PROMOTION_FULL_MOVE_MASK == 0 {
                bad_captures.push((m, score, see_score));
            } else {
                move_scores.push((m, score));
            }
        }
        quiets_added = false;
    }

    // Cache endgame status for current position - used in extension decisions
    let current_is_end_game = is_end_game(position);

    loop {
        // Stage order: good captures -> quiets -> SEE-losing captures.
        // Advance a stage whenever the current list runs dry, and keep advancing
        // while stages come back empty (a position can easily have no quiets but
        // still have bad captures left, and vice versa).
        while move_scores.is_empty() {
            if !quiets_added {
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
            } else if !bad_captures_added {
                bad_captures_added = true;
                // Copy only the live entries (mem::take of the full-capacity
                // ArrayVec was a ~2KB memcpy even when the list was empty);
                // bad_captures itself is kept for its stored SEE values.
                for &(bm, bscore, _) in bad_captures.iter() {
                    move_scores.push((bm, bscore));
                }
            } else {
                break; // All stages exhausted
            }
        }
        if move_scores.is_empty() {
            break; // All moves processed
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
        //
        // NET-353: staging already ran SEE for every generated capture, so reuse
        // it rather than recomputing the Position-copying SEE per picked move.
        // Good-stage tactical moves all have SEE >= 0 (EP captures skip staging
        // SEE but are provably >= 0), and the threshold is always negative, so
        // the prune can only ever fire in the bad-captures stage.
        // `legal_move_count >= 1` (NET-374): this `continue` fires BEFORE the
        // move is made and before legal_move_count is incremented, and unlike
        // alpha-pruning and LMP it had no first-move protection. If every legal
        // move at a node were SEE-pruned, legal_move_count would stay 0 and the
        // node would be adjudicated stalemate/mate - a fabricated score, which
        // then gets stored in the TT.
        if legal_move_count >= 1
            && bad_captures_added
            && scouting
            && is_tactical
            && !is_promotion
            && !in_check
            && depth <= SEE_PRUNE_MAX_DEPTH
            && alpha.abs() < MATE_START
        {
            let see_threshold = -(SEE_PRUNE_MARGIN * (depth as Score) * (depth as Score));
            let stored_see = bad_captures.iter().find(|e| e.0 == m).map_or(0, |e| e.2);
            if stored_see < see_threshold {
                continue;
            }
        }

        // Pawn push extension: extend by 1 ply for pawn push to 7th rank
        // Only if no check extension already applied (avoid over-extending)
        let pawn_push_ext: u8 =
            if check_extension == 0 && is_pawn_push_to_7th(position, m) && (ply as u16) < (search_state.iterative_depth as u16) * 2 {
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
            && (ply as u16) < (search_state.iterative_depth as u16) * 2
        {
            1
        } else {
            0
        };

        let move_extension = check_extension + pawn_push_ext + passed_pawn_ext;

        let unmake = make_move_nnue(position, m, search_state);
        prefetch_hash(position, search_state, hash_mask); // Prefetch child position's hash entry
                                                          // Track move at this ply for countermove heuristic
        search_state.ply_move[ply as usize] = m;
        // For killer moves, use actual capture detection from unmake info
        let move_is_capture = unmake.captured_piece != CAPTURED_NONE;

        // Check if move is legal (king not in check after move)
        if !is_check(position, old_mover) {
            legal_move_count += 1;

            // Cache whether this move gives check (opponent's king in check)
            // Used for pruning decisions - moves that give check should not be pruned/reduced
            //
            // NET-355: computed lazily - every consumer (alpha-prune, LMP, LMR)
            // requires !is_tactical, and all are unreachable when the check
            // extension applies (alpha_prune_flag and LMP are !in_check-gated,
            // LMR needs move_extension == 0). When computed, the value is passed
            // to the child as its in_check, saving the entry recompute there.
            let gives_check_known: Option<bool> = if is_tactical || check_extension != 0 {
                None
            } else {
                Some(is_check(position, position.mover))
            };
            let gives_check = gives_check_known == Some(true);

            if legal_move_count > 1 && alpha_prune_flag && !is_tactical && !gives_check {
                unmake_move_nnue(position, m, &unmake, search_state);
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
                && legal_move_count
                    > if improving {
                        LMP_MOVE_THRESHOLDS[depth as usize]
                    } else {
                        // Falling eval: give late quiets half the budget
                        LMP_MOVE_THRESHOLDS[depth as usize] / 2
                    }
                && m != search_state.killer_moves[ply as usize][0]
                && m != search_state.killer_moves[ply as usize][1]
                && !gives_check
                && alpha.abs() < MATE_START
            {
                unmake_move_nnue(position, m, &unmake, search_state);
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
                        // m is already made here, so the side that played prev_move is
                        // old_mover ^ 1, not position.mover ^ 1
                        let opponent_side = old_mover ^ 1;
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
                // m is already made, so index by old_mover (piece_index_12 would use the
                // flipped position.mover and read the opponent's table half)
                let piece_12 = curr_piece + (old_mover as usize * 6);
                let hist = search_state.history_moves[piece_12][curr_from][curr_to] as i32;
                if hist > LMR_HISTORY_GOOD_THRESHOLD && reduction > 0 {
                    reduction -= 1;
                } else if hist < LMR_HISTORY_BAD_THRESHOLD {
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

                // Reduce late moves one ply deeper when our eval is falling
                if !improving {
                    reduction += 1;
                }
                reduction
            } else {
                0
            };

            // Apply extensions to search depth
            let search_depth = depth + move_extension;

            children_here += 1;
            search_state.children_searched += 1;
            let path_score = if scout_search {
                // Cap the reduction so the child depth (search_depth - 1 - lmr)
                // cannot wrap below zero: stacked LMR adjustments (table + bad
                // history + bad continuation) can exceed the remaining depth
                lmr_scout_search(
                    lmr.min(search_depth - 1),
                    ply,
                    search_state,
                    (alpha, beta),
                    search_depth,
                    position,
                    gives_check_known,
                )
            } else {
                search_wrapper(search_depth, ply, search_state, (-beta, -alpha), position, 0, 0, gives_check_known)
            };

            let score = path_score.1;

            unmake_move_nnue(position, m, &unmake, search_state);

            check_time!(search_state);
            if is_stopped(&search_state.stop) {
                // Don't fall through to the TT store: the move list is only
                // partially searched, so the bound would be unsound
                return best_pathscore;
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
                            excluded_move,
                            &searched_quiets,
                            &searched_captures,
                            lazy_eval,
                            raw_static_eval,
                            children_here.min(u8::MAX as u32) as u8,
                            false,
                        );
                    }
                    hash_flag = Exact;
                }
                scout_search = true;
            }
            // Track searched moves that didn't cut off - malus candidates if a
            // later move proves best (the eventual best move itself is exempt)
            if captured_value == 0 && !is_promotion {
                searched_quiets.push(m);
            } else if captured_value != 0 {
                searched_captures.push((m, captured_value));
            }
        } else {
            unmake_move_nnue(position, m, &unmake, search_state);
        }
    }

    // Reaching here means the move loop finished without a beta cutoff - an
    // completed no-cut move-loop node. This includes PV nodes and excludes
    // forward-pruned fail-low nodes, so it is NOT an alpha-beta all-node. Cut
    // nodes return early
    // through cutoff_unmake and are not counted.
    if children_here > 0 {
        search_state.no_cutoff_nodes += 1;
        search_state.no_cutoff_children += children_here as u64;
    }

    if legal_move_count == 0 {
        if in_check {
            best_pathscore.1 = -MATE_SCORE + ply as Score
        } else {
            best_pathscore.1 = draw_value(position, search_state)
        }
    };

    // PV completion: the move that raised alpha at an exact node is a proven
    // best move - reward it and push down the quiets tried before it
    if hash_flag == Exact && excluded_move == 0 {
        let best_move = best_pathscore.0[0];
        let best_captured = captured_piece_value(position, best_move);
        let bonus = (real_depth as i32) * (real_depth as i32);
        update_history(position, search_state, best_move, bonus, best_captured);
        update_countermove_history_for_move(
            position,
            ply,
            search_state,
            best_move,
            bonus,
            best_captured,
            best_move & PROMOTION_FULL_MOVE_MASK != 0,
        );
        update_followup_history_for_move(
            position,
            ply,
            search_state,
            best_move,
            bonus,
            best_captured,
            best_move & PROMOTION_FULL_MOVE_MASK != 0,
        );
        update_capture_history_for_move(position, search_state, best_move, bonus, best_captured);
        let malus = -bonus;
        for &qm in &searched_quiets {
            if qm != best_move {
                update_history(position, search_state, qm, malus, 0);
                update_countermove_history_for_move(position, ply, search_state, qm, malus, 0, false);
                update_followup_history_for_move(position, ply, search_state, qm, malus, 0, false);
            }
        }
        for &(cm, cv) in &searched_captures {
            if cm != best_move {
                update_capture_history_for_move(position, search_state, cm, malus, cv);
            }
        }
    }

    // Eval-correction update: exact scores always teach; a fail-low only
    // teaches when it lands below the static eval (usable upper bound)
    if excluded_move == 0 && !in_check && legal_move_count > 0 && best_pathscore.1.abs() < MATE_START {
        let best_move = best_pathscore.0[0];
        let usable_bound = hash_flag == Exact || best_pathscore.1 < lazy_eval;
        if usable_bound && (best_move == 0 || captured_piece_value(position, best_move) == 0) {
            update_correction_history(position, search_state, real_depth, best_pathscore.1, lazy_eval);
        }
    }

    // A singular-verification search (excluded_move != 0) produces scores that
    // exclude the best move - storing them would poison this position's entry
    if excluded_move == 0 {
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
            raw_static_eval,
        );
    }

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
    // KNOWN INCOMPLETE - see NET-391 before changing this line. Two defects are
    // understood and deliberately NOT fixed here, because each is correct in
    // principle but measured a large bench-node cost that no match has yet
    // shown to be worth paying:
    //
    //   1. The window is one short. Reverse index 0 is the current position
    //      itself (every caller pushes the child's lock before recursing), so
    //      a slot is spent self-matching and a recurrence at distance exactly
    //      half_moves - the position right after the last irreversible move -
    //      can never be seen. `half_moves + 1` is the provably correct window
    //      and cost +15.4% nodes.
    //   2. There is no plies-from-null bound, so this scan reaches ACROSS null
    //      moves. Positions either side of a null do not form a legal
    //      sequence, so such a match is fictitious. Adding the bound (the
    //      standard rule) cost +12.9% nodes.
    //
    // Both make the engine more correct about draws and both make the tree
    // markedly bigger; NET-391 exists to settle whether either is affordable.
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
    known_in_check: Option<bool>,
) -> PathScore {
    let alpha = window.0;
    let beta = window.1;
    search_state.scout_searches += 1;
    let mut scout_path = search_wrapper(
        real_depth,
        ply,
        search_state,
        (-alpha - 1, -alpha),
        new_position,
        lmr,
        0,
        known_in_check,
    );

    if scout_path.1 > alpha && lmr > 0 {
        search_state.research_lmr_full += 1;
        // We are in an LMR search and we Need to research with full window. but still with late move reduction
        scout_path = search_wrapper(real_depth, ply, search_state, (-beta, -alpha), new_position, lmr, 0, known_in_check);
        if scout_path.1 > alpha {
            // Need to research with full window and no reduction
            search_state.research_full_depth += 1;
            scout_path = search_wrapper(real_depth, ply, search_state, (-beta, -alpha), new_position, 0, 0, known_in_check)
        }
    } else if scout_path.1 > alpha && scout_path.1 < beta {
        search_state.research_pvs += 1;
        // Not doing a LMR search, but still need to research with a full window
        scout_path = search_wrapper(real_depth, ply, search_state, (-beta, -alpha), new_position, 0, 0, known_in_check)
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
    // Deliberately NOT advancing half_moves: it widens the repetition scan
    // window inside the null subtree, and that window should not be reaching
    // across the null move in the first place (see the note at the call site).
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
#[allow(clippy::too_many_arguments)]
fn search_wrapper(
    depth: u8,
    ply: u8,
    search_state: &mut SearchState,
    window: Window,
    position: &mut Position,
    lmr: u8,
    excluded_move: Move,
    known_in_check: Option<bool>,
) -> PathScore {
    search_state.history.push(position.zobrist_lock);
    let path_score = search(
        position,
        depth - 1 - lmr,
        ply + 1,
        window,
        search_state,
        false,
        excluded_move,
        known_in_check,
    );
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
    excluded_move: Move,
    searched_quiets: &MoveList,
    searched_captures: &MoveScoreArray,
    static_eval: Score,
    // Raw (pre-correction) eval for the TT entry; `static_eval` above is the
    // correction-adjusted value and feeds correction history instead.
    raw_static_eval: Score,
    // 1-based index *among moves actually searched* of the move that caused
    // this cutoff, for the ordering statistic (NET-493).
    //
    // Deliberately not legal_move_count: that increments before the alpha and
    // LMP prunes, so moves that were skipped without ever being searched still
    // advanced it. A cutoff on the 2nd searched move landed in the "4-6" bucket
    // whenever four moves were pruned ahead of it, making move ordering look
    // worse than it is. Pruned moves cost no search, so they must not count
    // towards the ordering position. Caught by review of NET-493.
    searched_move_count: u8,
    // True only at the hash-move call site, which is tried before the loop.
    is_hash_move: bool,
) -> PathScore {
    // Singular-verification cutoffs are artifacts of the excluded move and the
    // shifted window - keep them out of the TT and the ordering heuristics
    if excluded_move != 0 {
        return best_pathscore;
    }

    // Counted after the singular-verification bail above, so the statistic
    // reflects real cutoffs rather than artifacts of the excluded-move window.
    search_state.cutoffs += 1;
    if searched_move_count <= 1 {
        search_state.cutoffs_first_move += 1;
    }

    // Which heuristic supplied the cutting move (NET-493). This must mirror
    // the precedence in move_scores.rs::score_move exactly, because the point
    // of the statistic is "which heuristic got this move searched first". Two
    // ways it was wrong before, both found in review:
    //
    //   - non-capture promotions were falling through to "other quiet", when
    //     score_move ranks a queen promotion alongside a good capture and the
    //     under-promotions just below. They are not quiet moves in any sense
    //     the ordering cares about.
    //   - ply-2 distant killers were unclassified, so they landed in
    //     countermove or "other quiet". Distant killers outrank countermoves
    //     in score_move, so a move that is both was credited to the wrong
    //     heuristic - the classifier promised generator order and did not
    //     deliver it.
    let kind = if is_hash_move {
        0
    } else if is_capture {
        // Covers en passant too: the unmake records a captured piece for it.
        1
    } else if m & PROMOTION_FULL_MOVE_MASK != 0 {
        2
    } else if m == search_state.mate_killer[ply as usize]
        || m == search_state.killer_moves[ply as usize][0]
        || m == search_state.killer_moves[ply as usize][1]
    {
        3
    } else if ply > 2 && (m == search_state.killer_moves[(ply - 2) as usize][0] || m == search_state.killer_moves[(ply - 2) as usize][1]) {
        // `ply > 2` matches score_move's own guard, not `>= 2`.
        4
    } else if ply > 0 && {
        // Same derivation as update_countermove: index by the opponent's
        // previous move.
        let prev = search_state.ply_move[(ply - 1) as usize];
        prev != 0 && {
            let prev_piece = piece_type_to_index(prev) + ((position.mover ^ 1) as usize * 6);
            search_state.countermoves[prev_piece][to_square_part(prev) as usize] == m
        }
    } {
        5
    } else {
        6
    };
    search_state.cutoff_by_kind[kind] += 1;

    let idx = match searched_move_count {
        0 | 1 => 0,
        2 => 1,
        3 => 2,
        4..=6 => 3,
        _ => 4,
    };
    search_state.cutoff_by_index[idx] += 1;
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
        raw_static_eval,
    );
    let bonus = (depth as i32) * (depth as i32);
    update_history(position, search_state, m, bonus, captured_value);
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

    // Malus: the moves tried before the cutoff move failed to do the job -
    // push their history down so they order later next time
    let malus = -bonus;
    for &qm in searched_quiets {
        if qm != m {
            update_history(position, search_state, qm, malus, 0);
            update_countermove_history_for_move(position, ply, search_state, qm, malus, 0, false);
            update_followup_history_for_move(position, ply, search_state, qm, malus, 0, false);
        }
    }
    for &(cm, cv) in searched_captures {
        if cm != m {
            update_capture_history_for_move(position, search_state, cm, malus, cv);
        }
    }

    // A quiet fail-high above the static eval is a usable lower-bound signal
    // for the eval-correction table
    if static_eval != -Score::MAX && captured_value == 0 && best_pathscore.1 > static_eval && best_pathscore.1.abs() < MATE_START {
        update_correction_history(position, search_state, depth, best_pathscore.1, static_eval);
    }
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
fn update_capture_history_for_move(position: &Position, search_state: &mut SearchState, m: Move, bonus: i32, captured_value: Score) {
    if captured_value == 0 {
        return; // Not a capture
    }

    // Index the victim from the BOARD, exactly as the read side does
    // (capture_history_score_cached -> victim_piece_index). Deriving it from
    // captured_value instead was wrong for promotions, because that value
    // includes the promotion delta, so captured_piece_to_index matched no
    // piece value and fell through to bucket 5 - a bucket the reader never
    // consults. Every promotion, capturing or not, wrote its history into a
    // dead slot (NET-374).
    //
    // The move has been unmade by the time this runs, so `position` is the
    // pre-move board and the victim is still on the destination square.
    let enemy = &position.pieces[opponent!(position.mover) as usize];
    let to_square = to_square_part(m);
    if enemy.all_pieces_bitboard & bit(to_square) == 0 {
        // Quiet promotion (non-zero captured_value from the promo delta, but no
        // victim), or en passant - where the victim is not on the destination
        // square and which the read side likewise never looks up.
        return;
    }

    let attacker = piece_type_to_index(m);
    let victim = victim_piece_index(to_square, enemy);
    let to_sq = to_square as usize;

    let entry = &mut search_state.capture_history[attacker][victim][to_sq];
    let current = *entry as i32;
    // Gravity formula
    let abs_bonus = bonus.abs();
    let new_value = current + bonus - current * abs_bonus / 16384;
    *entry = new_value.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
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

/// Correction to apply to the raw static eval: what the search has learned
/// about this pawn structure's systematic eval error for the side to move
#[inline(always)]
fn eval_correction(position: &Position, search_state: &SearchState) -> Score {
    // Incremental pawn key (NET-356) - identical value to the old
    // pawn_zobrist_key(position) as u64, without the per-node pawn loop
    let idx = (position.pawn_key as usize) & (CORRECTION_HISTORY_SIZE - 1);
    (search_state.correction_history[position.mover as usize][idx] as i32 / CORRECTION_HISTORY_GRAIN) as Score
}

/// Blend the observed (search score - static eval) gap into the correction
/// table as a depth-weighted exponential moving average
#[inline(always)]
fn update_correction_history(position: &Position, search_state: &mut SearchState, depth: u8, score: Score, static_eval: Score) {
    let idx = (position.pawn_key as usize) & (CORRECTION_HISTORY_SIZE - 1);
    let entry = &mut search_state.correction_history[position.mover as usize][idx];
    let diff = (score - static_eval) * CORRECTION_HISTORY_GRAIN;
    let weight = (depth as i32 + 1).min(CORRECTION_HISTORY_WEIGHT_MAX);
    let current = *entry as i32;
    let new_value = (current * (256 - weight) + diff * weight) / 256;
    *entry = new_value.clamp(-CORRECTION_HISTORY_MAX, CORRECTION_HISTORY_MAX) as i16;
}

#[inline(always)]
fn update_history(position: &Position, search_state: &mut SearchState, m: Move, bonus: i32, captured_value: Score) {
    if captured_value == 0 {
        let f = from_square_part(m) as usize;
        let t = to_square_part(m) as usize;

        let piece_index = piece_index_12(position, m);

        let entry = &mut search_state.history_moves[piece_index][f][t];
        let current = *entry as i32;
        // Gravity formula keeps values bounded within roughly +/- HISTORY_MAX
        let new_value = current + bonus - current * bonus.abs() / HISTORY_MAX;
        *entry = new_value.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
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
    // The side to move must have at least one non-pawn piece: with king+pawns
    // only, zugzwang makes a null-move "pass" wildly optimistic
    let stm_non_pawn = side_total_non_pawn_count(position, position.mover);
    stm_non_pawn >= 1 && stm_non_pawn + side_total_non_pawn_count(position, opponent!(position.mover)) >= 2
}

#[inline(always)]
fn side_total_non_pawn_count(position: &Position, side: Mover) -> u32 {
    (position.pieces[side as usize].bishop_bitboard
        | position.pieces[side as usize].knight_bitboard
        | position.pieces[side as usize].rook_bitboard
        | position.pieces[side as usize].queen_bitboard)
        .count_ones()
}
