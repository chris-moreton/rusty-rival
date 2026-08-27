use crate::engine_constants::{HASH_SIZE_MB, TM_HARD_FACTOR, TM_HARD_MAX_FRACTION, TM_SOFT_FACTOR};
use crate::tablebase::init_tablebase;

use either::{Either, Left, Right};

use regex::Regex;
use std::cmp::{max, min};
use std::ops::Add;
use std::process::exit;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use crate::evaluate::evaluate_position;
use crate::fen::{algebraic_move_from_move, get_fen, get_position};
use crate::make_move::make_move;
use crate::move_constants::START_POS;
use crate::moves::{generate_moves, is_check};

use crate::datagen::cmd_datagen;
use crate::perft::perft;
use crate::search::{clear_countermoves, clear_history_table, clear_killers, iterative_deepening};
use crate::types::{set_stop, Move, Position, SearchHandle, SearchState, SharedHashTable, StopReason, UciState, BLACK, WHITE};
use crate::uci_bench::cmd_benchmark;
use crate::utils::hydrate_move_from_algebraic_move;

fn replace_shortcuts(l: &str) -> &str {
    match l {
        "pv" => "setoption name multipv value 10",
        "c3draw" => "position fen 6k1/5pp1/8/4KP1p/8/P3N1Pn/3p1P1P/2rR4 b - - 5 43",
        "pawnf4" => "position fen 8/7R/1pqp1k2/p3p3/P1n1P3/1Q3P2/2Pr4/1KB5 w - - 2 42",
        "bl1" => "position fen 6k1/5pp1/5b1p/1Pp1pP2/2Pq4/3p1Q2/3B1PPP/r4RK1 w - - 2 34",
        "bl2" => "position fen 6k1/3q1pp1/5b1p/1Pp1pP2/2Pp4/3Q4/5PPP/r1B2RK1 w - - 0 32", // d3f3 is a blunder, bm d3e4

        "bl4" => "position fen 2r3k1/3q1pp1/p7/3n2R1/2Nb3p/1Pp2P1P/P1Q5/K3R3 w - - 0 35", // bm a3
        "bl5" => "position fen 2r3k1/3q1p2/p5p1/3n4/2Nb3R/1Pp2P1P/P1Q5/K3R3 b - - 0 36",  // bm Nb4

        "p3" => "position fen 8/6nk/8/1p2P1Q1/1P4PP/P4q2/8/6K1 w - - 3 56",
        "p5" => "position fen 2r5/p2Qbkpp/4p3/5p2/2P4q/P2P2N1/1r3P1P/R3K2b w Q - 0 20",
        "mate201" => "position fen 8/8/8/8/4Q3/2P4k/8/5K2 w - - 0 1",
        "mate301" => "position fen 1k5r/pP3ppp/3p2b1/1BN1n3/1Q2P3/P1B5/KP3P1P/7q w - - 1 0",
        "mate302" => "position fen 3r4/pR2N3/2pkb3/5p2/8/2B5/qP3PPP/4R1K1 w - - 1 0",
        "mate303" => "position fen R6R/1r3pp1/4p1kp/3pP3/1r2qPP1/7P/1P1Q3K/8 w - - 1 0",
        "mate304" => "position fen 4r1k1/5bpp/2p5/3pr3/8/1B3pPq/PPR2P2/2R2QK1 b - - 0 1",
        "mate305" => "position fen 8/8/8/8/4Q3/2P3k1/4K3/8 w - - 0 1",
        "mate401" => "position fen 7R/r1p1q1pp/3k4/1p1n1Q2/3N4/8/1PP2PPP/2B3K1 w - - 1 0",
        "mate402" => "position fen 8/8/8/8/4Q3/2PK3k/8/8 w - - 0 1",
        "mate501" => "position fen 6k1/3b3r/1p1p4/p1n2p2/1PPNpP1q/P3Q1p1/1R1RB1P1/5K2 b - - 0 1",
        "mate502" => "position fen 8/8/8/8/2K1Q3/2P3k1/8/8 w - - 0 1",
        "mate601" => "position fen 8/8/8/1K6/4Q3/2P5/5k2/8 w - - 0 1",
        "mate602" => "position fen 4r1k1/p1qr1p2/2pb1Bp1/1p5p/3P1n1R/1B3P2/PP3PK1/2Q4R w - - 0 1",
        "tf01" => "position fen 3Nk3/4p3/2p2p2/1bp2p2/3b1Pn1/2N5/1PP3PP/2BQK2R b K - 0 1",
        "tf02" => "position fen 3Nk3/4p3/2p2p2/1bp2p2/3b1Pn1/2N5/1PP3PP/2BQK2R b K - 0 1 moves d4f2 e1d2 f2e3 d2e1",
        "tf03" => "position fen 3Nk3/4p3/2p2p2/1bp2p2/3b1Pn1/2N5/1PP3PP/2BQK2R b K - 0 1 moves d4f2 e1d2 f2e3 d2e1 e3f2 e1d2",
        "st01" => "position fen 8/8/8/8/4Q3/2P4k/8/5K2 w - - 0 1",
        "st02" => "position fen 8/8/8/8/4Q3/2P3k1/4K3/8 w - - 0 1",
        "st03" => "position fen 8/8/8/8/4Q3/2PK3k/8/8 w - - 0 1",
        "wac8" => "position fen r4q1k/p2bR1rp/2p2Q1N/5p2/5p2/2P5/PP3PPP/R5K1 w - - 0 1",
        "bench01" => "position fen 8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
        "bench02" => "position fen 8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
        "bench03" => "position fen 8/7p/p5pb/4k3/P1pPn3/8/P5PP/1rB2RK1 b - d3 0 28",
        "bench04" => "position fen r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R3K2R w KQkq - 0 1",
        "bench05" => "position fen r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        "bench06" => "position fen 8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
        "bench07" => "position fen 8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
        "bench08" => "position fen r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        "bench09" => "position startpos",
        "bench10" => "position fen r1bqk2r/1ppp1ppp/p1n2n2/2b1p3/B3P3/2N2N2/PPPP1PPP/R1BQ1RK1 w kq - 0 1",
        "bench11" => "position fen 8/7R/1pqp1k2/p3p3/P1n1P3/1Q3P2/2Pr4/1KB5 w - - 2 42",
        "i" => "go infinite",
        _ => l,
    }
}

pub fn run_command_test(uci_state: &mut UciState, search_state: &mut SearchState, l: &str) -> Either<String, Option<String>> {
    // Tests expect synchronous behavior, so use the sync version
    run_command_sync(uci_state, search_state, replace_shortcuts(l))
}

/// Run a command synchronously (blocking). Used for benchmarking where we need
/// to read results from search_state after the search completes.
pub fn run_command_sync(uci_state: &mut UciState, search_state: &mut SearchState, l: &str) -> Either<String, Option<String>> {
    let mut trimmed_line = replace_shortcuts(l).trim().replace("  ", " ");
    if trimmed_line.starts_with("position startpos") {
        trimmed_line = trimmed_line.replace("startpos", &("fen ".to_string() + START_POS));
    }
    let parts = trimmed_line.split(' ').collect::<Vec<&str>>();

    match *parts.first().unwrap() {
        "go" => cmd_go_sync(uci_state, search_state, parts),
        "ucinewgame" => {
            // Simplified ucinewgame for sync mode (no search handle needed)
            search_state.nodes = 0;
            search_state.qnodes = 0;
            search_state.tt_probes = 0;
            search_state.tt_hits = 0;
            search_state.tt_deep_enough = 0;
            search_state.tt_slot_taken = 0;
            search_state.scout_searches = 0;
            search_state.research_lmr_full = 0;
            search_state.research_full_depth = 0;
            search_state.research_pvs = 0;
            search_state.children_searched = 0;
            search_state.no_cutoff_nodes = 0;
            search_state.no_cutoff_children = 0;
            search_state.cutoffs = 0;
            search_state.cutoffs_first_move = 0;
            search_state.cutoff_by_kind = [0; 7];
            search_state.cutoff_by_index = [0; 5];
            search_state.root_moves.clear();
            search_state.pv.clear();
            search_state.hash_table.clear();
            clear_history_table(search_state);
            uci_state.fen = START_POS.parse().unwrap();
            Right(None)
        }
        "position" => cmd_position(uci_state, search_state, parts),
        _ => {
            let mut search_handle: Option<SearchHandle> = None;
            run_command(uci_state, search_state, &mut search_handle, l)
        }
    }
}

/// Synchronous version of cmd_go for benchmarking
fn cmd_go_sync(uci_state: &mut UciState, search_state: &mut SearchState, parts: Vec<&str>) -> Either<String, Option<String>> {
    // Bare `go` behaves as `go infinite`.
    let t = *parts.get(1).unwrap_or(&"infinite");
    search_state.nodes = 0;
    search_state.qnodes = 0;
    search_state.tt_probes = 0;
    search_state.tt_hits = 0;
    search_state.tt_deep_enough = 0;
    search_state.tt_slot_taken = 0;
    search_state.scout_searches = 0;
    search_state.research_lmr_full = 0;
    search_state.research_full_depth = 0;
    search_state.research_pvs = 0;
    search_state.children_searched = 0;
    search_state.no_cutoff_nodes = 0;
    search_state.no_cutoff_children = 0;
    search_state.cutoffs = 0;
    search_state.cutoffs_first_move = 0;
    search_state.cutoff_by_kind = [0; 7];
    search_state.cutoff_by_index = [0; 5];
    search_state.nodes_limit = u64::MAX;
    set_stop(&search_state.stop, false);
    search_state.stop_reason.store(StopReason::None as u8, Ordering::Relaxed);

    match t {
        "perft" => {
            let depth = match parts.get(2).and_then(|s| s.parse::<u8>().ok()) {
                Some(d) if d >= 1 => d,
                _ => return Left("usage: go perft <depth>".parse().unwrap()),
            };
            cmd_perft(depth, uci_state);
            Right(None)
        }
        "infinite" => {
            let mut position = get_position(uci_state.fen.trim());
            let end = Instant::now().add(Duration::from_secs(86400));
            search_state.end_time = end;
            search_state.soft_time_limit = end;
            search_state.original_soft_time_limit = end;
            search_state.time_management_active = false;
            let mv = iterative_deepening(&mut position, 200, search_state, 1);
            Right(Some(format_bestmove(mv, search_state)))
        }
        "mate" => {
            let mate_depth = parts.get(2).and_then(|s| s.parse::<u8>().ok()).unwrap_or(100);
            let mut position = get_position(uci_state.fen.trim());
            let end = Instant::now().add(Duration::from_secs(86400));
            search_state.end_time = end;
            search_state.soft_time_limit = end;
            search_state.original_soft_time_limit = end;
            search_state.time_management_active = false;
            let mv = iterative_deepening(&mut position, mate_depth.saturating_mul(2), search_state, 1);
            Right(Some(format_bestmove(mv, search_state)))
        }
        _ => {
            let line = parts.join(" ");
            uci_state.wtime = extract_go_param("wtime", &line, 0);
            uci_state.btime = extract_go_param("btime", &line, 0);
            uci_state.winc = extract_go_param("winc", &line, 0);
            uci_state.binc = extract_go_param("binc", &line, 0);
            uci_state.moves_to_go = extract_go_param("movestogo", &line, 0);
            uci_state.depth = extract_go_param("depth", &line, 250).min(250);
            uci_state.nodes = extract_go_param("nodes", &line, u64::MAX);
            let movetime = extract_go_param_opt("movetime", &line);
            uci_state.move_time = movetime.unwrap_or(10000000);

            search_state.nodes_limit = uci_state.nodes;

            let mut position = get_position(uci_state.fen.trim());

            // Parse searchmoves if present
            search_state.search_moves = parse_searchmoves(&line, &position);

            // Mirrors cmd_go (NET-362): clock presence keyed on tokens, explicit
            // movetime is exact and never rescaled by the clock allocation.
            let clock_present = parts.contains(&"wtime") || parts.contains(&"btime");

            if movetime.is_none() && clock_present {
                if position.mover == WHITE {
                    calc_from_colour_times(uci_state, uci_state.wtime, uci_state.winc);
                } else {
                    calc_from_colour_times(uci_state, uci_state.btime, uci_state.binc);
                }
            }

            uci_state.move_time = max(10, uci_state.move_time - min(uci_state.move_time, uci_state.move_overhead));

            let base_time_ms = uci_state.move_time;
            let (time_remaining, increment) = if position.mover == WHITE {
                (uci_state.wtime, uci_state.winc)
            } else {
                (uci_state.btime, uci_state.binc)
            };

            if movetime.is_none() && clock_present {
                let (soft_ms, hard_ms) = if time_remaining == 0 {
                    // Zero/negative clock: emergency budget, not the 10M default.
                    let emergency = (increment / 2).clamp(10, 50);
                    (emergency, emergency)
                } else {
                    (
                        max(10, (base_time_ms as f64 * TM_SOFT_FACTOR) as u64),
                        max(
                            10,
                            min(
                                (base_time_ms as f64 * TM_HARD_FACTOR) as u64,
                                (time_remaining as f64 * TM_HARD_MAX_FRACTION) as u64,
                            ),
                        ),
                    )
                };

                let now = Instant::now();
                search_state.end_time = now.add(Duration::from_millis(hard_ms));
                search_state.soft_time_limit = now.add(Duration::from_millis(soft_ms));
                search_state.original_soft_time_limit = search_state.soft_time_limit;
                search_state.time_management_active = true;
            } else {
                // Exact deadline: explicit movetime, or no clock at all.
                let end = Instant::now().add(Duration::from_millis(uci_state.move_time));
                search_state.end_time = end;
                search_state.soft_time_limit = end;
                search_state.original_soft_time_limit = end;
                search_state.time_management_active = false;
            }

            let mv = iterative_deepening(&mut position, uci_state.depth as u8, search_state, 1);

            // Clear search_moves after search completes
            search_state.search_moves = None;

            Right(Some(format_bestmove(mv, search_state)))
        }
    }
}

pub fn run_command(
    uci_state: &mut UciState,
    search_state: &mut SearchState,
    search_handle: &mut Option<SearchHandle>,
    l: &str,
) -> Either<String, Option<String>> {
    let mut trimmed_line = replace_shortcuts(l).trim().replace("  ", " ");
    if trimmed_line.starts_with("position startpos") {
        trimmed_line = trimmed_line.replace("startpos", &("fen ".to_string() + START_POS));
    }
    let parts = trimmed_line.split(' ').collect::<Vec<&str>>();

    match *parts.first().unwrap() {
        "bench" => cmd_benchmark(uci_state, search_state, parts),
        "datagen" => cmd_datagen(uci_state, search_state, parts),
        "uci" => cmd_uci(),
        "isready" => cmd_isready(),
        "state" => cmd_state(uci_state, search_state),
        "eval" => cmd_eval(uci_state, search_state),
        "go" => cmd_go(uci_state, search_state, search_handle, parts),
        "stop" => cmd_stop(search_state, search_handle),
        "ponderhit" => cmd_ponderhit(search_handle),
        "setoption" => cmd_setoption(parts, search_state, uci_state),
        "register" => cmd_register(),
        "ucinewgame" => cmd_ucinewgame(uci_state, search_state, search_handle),
        "debug" => cmd_debug(uci_state, parts),
        "quit" => {
            // Stop any running search before quitting. Absorb anyway: it costs
            // nothing here and keeps the "no join without write-back" rule
            // uniform across every site.
            if let Some(handle) = search_handle.take() {
                handle.stop_and_wait(search_state);
            }
            exit(0)
        }
        "mvm" => cmd_mvm(search_state, parts),
        "position" => cmd_position(uci_state, search_state, parts),
        _ => Left("Unknown command".parse().unwrap()),
    }
}

fn fen_and_moves(parts: Vec<&str>) -> (String, Vec<String>) {
    if !parts.contains(&"moves") {
        let fen = parts.join(" ").replace("position fen", "");
        return (fen.trim().parse().unwrap(), vec![]);
    }
    let fen_and_moves_string = parts.join(" ").replace("position fen", "");
    let two_parts = fen_and_moves_string.split("moves").collect::<Vec<&str>>();
    let fen = two_parts[0];
    let moves = two_parts[1]
        .trim()
        .split(' ')
        .collect::<Vec<&str>>()
        .into_iter()
        .map(|move_string| move_string.to_string())
        .collect();

    (fen.trim().parse().unwrap(), moves)
}

pub fn is_legal_move(position: &Position, algebraic_move: &str) -> bool {
    let moves = generate_moves(position);
    for m in moves {
        let am = algebraic_move_from_move(m);
        if am == algebraic_move {
            let mut new_position = *position;
            make_move(position, m, &mut new_position);
            if !is_check(&new_position, position.mover) {
                return true;
            }
        }
    }
    false
}

fn cmd_position(uci_state: &mut UciState, search_state: &mut SearchState, parts: Vec<&str>) -> Either<String, Option<String>> {
    //    cmd_ucinewgame(uci_state, search_state);
    let t = match parts.get(1) {
        Some(t) => *t,
        None => return Left("usage: position fen <fen> [moves ...]".parse().unwrap()),
    };
    match t {
        "fen" => {
            search_state.history = vec![];

            let re = Regex::new(
                r"\s*^(((?:[rnbqkpRNBQKP1-8]+/){7})[rnbqkpRNBQKP1-8]+)\s([b|w])\s([K|Q|k|q]{1,4}|-)\s(-|[a-h][1-8])\s(\d+\s\d+)$",
            )
            .unwrap();
            let (raw_fen, moves) = fen_and_moves(parts);

            // Default both move counters when a FEN omits them entirely (4 fields),
            // so legal FENs like "... w - -" are accepted. A partial-counter FEN
            // (5 fields) is malformed and left to fail validation. Validate BEFORE
            // committing to uci_state so a malformed position can never poison state
            // and crash the next `go`.
            let raw_fen = raw_fen.trim();
            let fen = if raw_fen.split_whitespace().count() == 4 {
                format!("{} 0 1", raw_fen)
            } else {
                raw_fen.to_string()
            };

            // The regex checks shape, not legality: a board with no king passes
            // it, and get_position then indexes ZOBRIST_KEYS_PIECES[..][64]
            // (king_square = trailing_zeros() of an empty bitboard) and panics
            // on the MAIN thread, killing the engine outright - the catch_unwind
            // in cmd_go only guards the search thread. Checked here, before
            // get_position is ever called (NET-369).
            let board = fen.split_whitespace().next().unwrap_or("");
            if board.matches('K').count() != 1 || board.matches('k').count() != 1 {
                return Left("Invalid FEN: needs exactly one king of each colour".parse().unwrap());
            }

            if re.is_match(&fen) {
                uci_state.fen = fen.clone();
                let mut position = get_position(&uci_state.fen);
                let mut new_position = position;
                search_state.history.push(new_position.zobrist_lock);
                if !moves.is_empty() {
                    for m in moves {
                        if !is_legal_move(&new_position, &m) {
                            return Left("Illegal move found".parse::<String>().unwrap() + " " + &*m);
                        }
                        let hydrated_move = hydrate_move_from_algebraic_move(&position, m.to_string());
                        make_move(&position, hydrated_move, &mut new_position);
                        search_state.history.push(new_position.zobrist_lock);
                        position = new_position
                    }
                }
                uci_state.fen = get_fen(&position);
                Right(None)
            } else {
                Left("Invalid FEN".parse().unwrap())
            }
        }
        _ => Left("Unknown position command".parse().unwrap()),
    }
}

/// Like `extract_go_param` but distinguishes "parameter absent" (None) from
/// "parameter present with a valid value" — the distinction matters for time
/// management (NET-362): `go movetime 0`/`go wtime 0` mean "present, zero" and
/// must not be confused with the parameter being missing.
///
/// Out-of-range values (negative, or too large for u64) are treated as absent
/// rather than clamped, preserving the long-standing fall-through-to-default
/// behaviour for every parameter: `depth -3` must NOT become `depth 0` (an
/// instant unsearched move) and a 20-digit `winc` must NOT become u64::MAX
/// (overflowing the time allocation). Zero/negative CLOCKS still trigger the
/// emergency budget because clock presence is keyed on the wtime/btime tokens,
/// not on these parsed values, and the clock defaults are already 0.
pub fn extract_go_param_opt(needle: &str, haystack: &str) -> Option<u64> {
    // `-?` so a negative value (cutechess sends negative clocks near
    // flag-fall) is consumed and rejected here, not left to mismatch.
    let re = needle.to_string() + " (-?[0-9]+)";
    let regex = Regex::new(&re).unwrap();
    regex
        .captures(haystack)
        .and_then(|x| x.get(1))
        .and_then(|m| match m.as_str().parse::<i128>() {
            Ok(v) if (0..=u64::MAX as i128).contains(&v) => Some(v as u64),
            _ => None, // negative, or overflows u64/i128: treat as absent
        })
}

pub fn extract_go_param(needle: &str, haystack: &str, default: u64) -> u64 {
    // Malformed, negative, or oversized values fall through to `default`
    // instead of panicking (NET-214).
    extract_go_param_opt(needle, haystack).unwrap_or(default)
}

fn cmd_state(mut _uci_state: &mut UciState, search_state: &mut SearchState) -> Either<String, Option<String>> {
    Right(Some(format!(r#"Nodes {}"#, search_state.nodes)))
}

/// Report the raw evaluator output without search or correction history.
///
/// Both side-to-move and White-relative scores are included so analysis tools
/// do not have to infer the perspective from the FEN. This intentionally uses
/// the same evaluator entry point as search while excluding all search effects.
fn cmd_eval(uci_state: &UciState, search_state: &mut SearchState) -> Either<String, Option<String>> {
    let position = get_position(&uci_state.fen);
    let stm_score = evaluate_position(&position, search_state);
    let white_score = if position.mover == WHITE { stm_score } else { -stm_score };
    let evaluator = if search_state.use_nnue && search_state.nnue_network.is_some() {
        "nnue"
    } else {
        "hce"
    };

    Right(Some(format!(
        "info string eval raw cp {} white_cp {} evaluator {}",
        stm_score, white_score, evaluator
    )))
}

fn cmd_mvm(search_state: &mut SearchState, parts: Vec<&str>) -> Either<String, Option<String>> {
    let (millis, count) = match (
        parts.get(1).and_then(|s| s.parse::<u64>().ok()),
        parts.get(2).and_then(|s| s.parse::<u32>().ok()),
    ) {
        (Some(m), Some(c)) => (m, c),
        _ => return Left("usage: mvm <millis> <count>".parse().unwrap()),
    };
    let mut engine_1_wins = 0;
    let mut engine_2_wins = 0;
    let mut draws = 0;

    search_state.show_info = false;

    for g in 0..count {
        let engine_1_colour = if g % 2 == 0 { WHITE } else { BLACK };
        let mut position = get_position(START_POS);
        let final_position = loop {
            set_stop(&search_state.stop, false);
            search_state.end_time = Instant::now().add(Duration::from_millis(millis));
            let mv = iterative_deepening(&mut position, 100_u8, search_state, 1);
            let mut new_position = position;
            make_move(&position, mv, &mut new_position);

            let mut legal_move_count = 0;
            for m in generate_moves(&new_position) {
                let mut p = new_position;
                make_move(&new_position, m, &mut p);
                if !is_check(&p, new_position.mover) {
                    legal_move_count += 1;
                }
            }
            if new_position.half_moves > 100 || legal_move_count == 0 {
                break new_position;
            }

            position = new_position
        };
        if final_position.half_moves > 100 || !is_check(&final_position, final_position.mover) {
            draws += 1;
        } else if final_position.mover == engine_1_colour {
            engine_2_wins += 1;
        } else {
            engine_1_wins += 1;
        }
        println!("{}", get_fen(&final_position));
        println!("{} {} {}", engine_1_wins, engine_2_wins, draws);
    }
    Right(Some("Done".parse().unwrap()))
}

fn cmd_go(
    uci_state: &mut UciState,
    search_state: &mut SearchState,
    search_handle: &mut Option<SearchHandle>,
    parts: Vec<&str>,
) -> Either<String, Option<String>> {
    // If there's already a search running, wait for it first, and take its
    // learned tables with us - this is the ordinary path by which one move's
    // learning reaches the next (NET-372).
    if let Some(handle) = search_handle.take() {
        handle.stop_and_wait(search_state);
    }

    // Bare `go` behaves as `go infinite`.
    let t = *parts.get(1).unwrap_or(&"infinite");

    // perft runs synchronously (no threading needed)
    if t == "perft" {
        let depth = match parts.get(2).and_then(|s| s.parse::<u8>().ok()) {
            Some(d) if d >= 1 => d,
            _ => return Left("usage: go perft <depth>".parse().unwrap()),
        };
        cmd_perft(depth, uci_state);
        return Right(None);
    }

    // Parse go parameters
    // Note: end_time is calculated here (before clone/spawn) to ensure we don't
    // exceed tournament time limits. The clone is now fast (Arc-based hash table)
    // so the overhead is minimal.
    let line = parts.join(" ");
    let is_ponder = parts.contains(&"ponder");
    // A ponder with any budget (clock, movetime, depth, nodes) must carry that
    // budget through to ponderhit; only a truly limitless `go ponder` is infinite.
    let has_budget = ["wtime", "btime", "movetime", "depth", "nodes"].iter().any(|k| line.contains(k));

    let (max_depth, end_time, soft_time_limit, nodes_limit, tm_active, ponder_soft, ponder_hard) =
        if t == "infinite" || (t == "ponder" && !has_budget) {
            let end = Instant::now().add(Duration::from_secs(86400));
            (200u8, end, end, u64::MAX, false, 0u64, 0u64)
        } else if t == "mate" {
            let mate_depth = parts.get(2).and_then(|s| s.parse::<u8>().ok()).unwrap_or(100);
            let end = Instant::now().add(Duration::from_secs(86400));
            (mate_depth.saturating_mul(2), end, end, u64::MAX, false, 0u64, 0u64)
        } else {
            uci_state.wtime = extract_go_param("wtime", &line, 0);
            uci_state.btime = extract_go_param("btime", &line, 0);
            uci_state.winc = extract_go_param("winc", &line, 0);
            uci_state.binc = extract_go_param("binc", &line, 0);
            uci_state.moves_to_go = extract_go_param("movestogo", &line, 0);
            uci_state.depth = extract_go_param("depth", &line, 250).min(250);
            uci_state.nodes = extract_go_param("nodes", &line, u64::MAX);
            let movetime = extract_go_param_opt("movetime", &line);
            uci_state.move_time = movetime.unwrap_or(10000000);

            let position = get_position(uci_state.fen.trim());
            // Clock presence is keyed on the TOKENS, not the parsed values:
            // `go wtime 0` is a present-but-empty clock and must produce an
            // emergency budget, never the no-clock default budget (NET-362).
            let clock_present = parts.contains(&"wtime") || parts.contains(&"btime");

            // An explicit `movetime` is an exact budget per the UCI spec; clock
            // times must not rescale it (NET-362), so skip the allocation.
            if movetime.is_none() && clock_present {
                if position.mover == WHITE {
                    calc_from_colour_times(uci_state, uci_state.wtime, uci_state.winc);
                } else {
                    calc_from_colour_times(uci_state, uci_state.btime, uci_state.binc);
                }
            }

            uci_state.move_time = max(10, uci_state.move_time - min(uci_state.move_time, uci_state.move_overhead));

            let base_time_ms = uci_state.move_time;
            let (time_remaining, increment) = if position.mover == WHITE {
                (uci_state.wtime, uci_state.winc)
            } else {
                (uci_state.btime, uci_state.binc)
            };

            if movetime.is_none() && clock_present {
                let (soft_ms, hard_ms) = if time_remaining == 0 {
                    // Clock reported zero (or negative — cutechess sends these
                    // near flag-fall with timemargin): emergency budget. Falling
                    // through to the 10,000,000ms default here searched for ~2.8
                    // hours and forfeited otherwise-winnable increment games.
                    let emergency = (increment / 2).clamp(10, 50);
                    (emergency, emergency)
                } else {
                    (
                        max(10, (base_time_ms as f64 * TM_SOFT_FACTOR) as u64),
                        max(
                            10,
                            min(
                                (base_time_ms as f64 * TM_HARD_FACTOR) as u64,
                                (time_remaining as f64 * TM_HARD_MAX_FRACTION) as u64,
                            ),
                        ),
                    )
                };

                if is_ponder {
                    let end = Instant::now().add(Duration::from_secs(86400));
                    (uci_state.depth as u8, end, end, uci_state.nodes, false, soft_ms, hard_ms)
                } else {
                    let now = Instant::now();
                    (
                        uci_state.depth as u8,
                        now.add(Duration::from_millis(hard_ms)),
                        now.add(Duration::from_millis(soft_ms)),
                        uci_state.nodes,
                        true,
                        0u64,
                        0u64,
                    )
                }
            } else if is_ponder {
                // Movetime/depth/nodes ponder: defer the deadline until
                // ponderhit, carrying the movetime budget in ponder_hard/soft so the
                // ponderhit conversion installs a real deadline instead of hanging.
                // (An explicit movetime lands here even with clocks present: the
                // exact budget carries through to ponderhit unrescaled.)
                let end = Instant::now().add(Duration::from_secs(86400));
                (
                    uci_state.depth as u8,
                    end,
                    end,
                    uci_state.nodes,
                    false,
                    uci_state.move_time,
                    uci_state.move_time,
                )
            } else {
                // Exact deadline: explicit movetime (regardless of clocks), or
                // no clock at all (depth/nodes/default budget).
                let now = Instant::now();
                let end = now.add(Duration::from_millis(uci_state.move_time));
                (uci_state.depth as u8, end, end, uci_state.nodes, false, 0u64, 0u64)
            }
        };

    // Clone position for the search thread
    let position = get_position(uci_state.fen.trim());

    // Fallback move (first legal move, else "0000") printed if the search thread
    // panics, so a search bug never leaves the engine silent → time forfeit.
    let fallback_move = {
        let mut fb = String::from("0000");
        for m in generate_moves(&position) {
            let mut np = position;
            make_move(&position, m, &mut np);
            if !is_check(&np, position.mover) {
                fb = algebraic_move_from_move(m);
                break;
            }
        }
        fb
    };

    // Parse searchmoves if present
    let search_moves = parse_searchmoves(&line, &position);

    // Create shared flags for this search
    let stop_flag = Arc::new(AtomicBool::new(false));
    let stop_reason = Arc::new(AtomicU8::new(StopReason::None as u8));
    let shared_nodes = Arc::new(AtomicU64::new(0));
    let pondering = Arc::new(AtomicBool::new(is_ponder));
    let ponder_soft_ms = Arc::new(AtomicU64::new(ponder_soft));
    let ponder_hard_ms = Arc::new(AtomicU64::new(ponder_hard));

    let num_threads = uci_state.threads;
    let mut handles = Vec::with_capacity(num_threads);

    for thread_id in 0..num_threads {
        // Clone search_state for each thread, sharing hash tables and stop flag
        let mut thread_search_state = search_state.clone();
        thread_search_state.nodes = 0;
        thread_search_state.qnodes = 0;
        thread_search_state.tt_probes = 0;
        thread_search_state.tt_hits = 0;
        thread_search_state.tt_deep_enough = 0;
        thread_search_state.tt_slot_taken = 0;
        thread_search_state.scout_searches = 0;
        thread_search_state.research_lmr_full = 0;
        thread_search_state.research_full_depth = 0;
        thread_search_state.research_pvs = 0;
        thread_search_state.children_searched = 0;
        thread_search_state.no_cutoff_nodes = 0;
        thread_search_state.no_cutoff_children = 0;
        thread_search_state.cutoffs = 0;
        thread_search_state.cutoffs_first_move = 0;
        thread_search_state.cutoff_by_kind = [0; 7];
        thread_search_state.cutoff_by_index = [0; 5];
        thread_search_state.nodes_limit = nodes_limit;
        thread_search_state.end_time = end_time;
        thread_search_state.soft_time_limit = soft_time_limit;
        thread_search_state.original_soft_time_limit = soft_time_limit;
        thread_search_state.time_management_active = tm_active;
        thread_search_state.stop = stop_flag.clone();
        thread_search_state.stop_reason = stop_reason.clone();
        thread_search_state.shared_nodes = shared_nodes.clone();
        thread_search_state.thread_id = thread_id;
        thread_search_state.pondering = pondering.clone();
        thread_search_state.ponder_soft_ms = ponder_soft_ms.clone();
        thread_search_state.ponder_hard_ms = ponder_hard_ms.clone();
        thread_search_state.is_ponder_search = is_ponder;
        thread_search_state.ponder_applied = false;

        if thread_id == 0 {
            // Main thread: shows info and prints bestmove
            thread_search_state.show_info = true;
            thread_search_state.search_moves = search_moves.clone();
        } else {
            // Helper threads: only populate TT, no UCI output
            thread_search_state.show_info = false;
            thread_search_state.search_moves = None;
        }

        let mut thread_position = position;

        // Helper threads start at offset depths to explore different tree parts
        let start_depth: u8 = if thread_id == 0 { 1 } else { ((thread_id % 255) + 1) as u8 };

        let thread_fallback = fallback_move.clone();

        let handle = thread::Builder::new()
            .stack_size(16 * 1024 * 1024)
            .spawn(move || -> Option<Box<SearchState>> {
                // Catch a panic in the search so thread 0 still emits a bestmove
                // (a silent search thread = time forfeit for the whole game).
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    let mv = iterative_deepening(&mut thread_position, max_depth, &mut thread_search_state, start_depth);
                    (mv, thread_search_state)
                }));
                if thread_id == 0 {
                    match result {
                        Ok((mv, ss)) => {
                            println!("{}", format_bestmove(mv, &ss));
                            // Hand thread 0's learned tables back so the join
                            // site can move them into the master (NET-372).
                            // Boxed: SearchState is large and this must not be
                            // returned on the stack.
                            Some(Box::new(ss))
                        }
                        Err(_) => {
                            println!("bestmove {}", thread_fallback);
                            // Panicked: contribute nothing rather than risk
                            // handing back half-updated tables.
                            None
                        }
                    }
                } else {
                    // Helper threads never write back. That gives ONE
                    // authoritative writer and stops helper histories being
                    // mixed together - it does NOT make the merged result
                    // independent of the thread count. Helpers share and mutate
                    // the TT while thread 0 searches, so thread 0's tree, and
                    // therefore its learned tables, still depend on how many
                    // helpers there are, how they are scheduled, and how
                    // quickly they observe the stop flag.
                    None
                }
            })
            .expect("Failed to spawn search thread");

        handles.push(handle);
    }

    // Store the search handle
    *search_handle = Some(SearchHandle {
        stop: stop_flag,
        pondering,
        ponder_soft_ms,
        ponder_hard_ms,
        handles,
    });

    Right(None)
}

/// Parse searchmoves from go command line
/// Returns None if searchmoves not specified, Some(vec) with the moves otherwise
fn parse_searchmoves(line: &str, position: &Position) -> Option<Vec<Move>> {
    // Find "searchmoves" keyword
    if let Some(idx) = line.find("searchmoves") {
        let after_keyword = &line[idx + "searchmoves".len()..];
        let mut moves = Vec::new();

        // UCI keywords that end the searchmoves list
        let keywords = [
            "ponder",
            "wtime",
            "btime",
            "winc",
            "binc",
            "movestogo",
            "depth",
            "nodes",
            "mate",
            "movetime",
            "infinite",
        ];

        for token in after_keyword.split_whitespace() {
            // Stop if we hit another keyword
            if keywords.contains(&token) {
                break;
            }
            // Parse move and add to list
            let mv = hydrate_move_from_algebraic_move(position, token.to_string());
            if mv != 0 {
                moves.push(mv);
            }
        }

        if moves.is_empty() {
            None
        } else {
            Some(moves)
        }
    } else {
        None
    }
}

fn cmd_stop(search_state: &mut SearchState, search_handle: &mut Option<SearchHandle>) -> Either<String, Option<String>> {
    // Takes the master state because a stopped search has still learned
    // something, and discarding it here would silently lose a move's worth of
    // history every time the GUI stops us early.
    if let Some(handle) = search_handle.take() {
        handle.stop_and_wait(search_state);
    }
    Right(None)
}

fn cmd_ponderhit(search_handle: &mut Option<SearchHandle>) -> Either<String, Option<String>> {
    if let Some(ref handle) = search_handle {
        handle.pondering.store(false, Ordering::Relaxed);
    }
    Right(None)
}

fn format_bestmove(mv: u32, search_state: &SearchState) -> String {
    let bestmove = algebraic_move_from_move(mv);
    // Include ponder move if we have a second move in the PV
    if let Some(&ponder_mv) = search_state.current_best.0.get(1) {
        if ponder_mv != 0 {
            return format!("bestmove {} ponder {}", bestmove, algebraic_move_from_move(ponder_mv));
        }
    }
    format!("bestmove {}", bestmove)
}

fn calc_from_colour_times(uci_state: &mut UciState, millis: u64, inc_millis: u64) {
    if millis > 0 {
        // When movestogo is not specified (0), assume 30 moves remaining
        // Previously this used ALL remaining time on the first move!
        let moves_remaining = if uci_state.moves_to_go == 0 { 30 } else { uci_state.moves_to_go };
        uci_state.move_time = min(uci_state.move_time, (millis as f64 / (moves_remaining as f64 + 1.0)) as u64);
        uci_state.move_time = (uci_state.move_time as f64 * 0.95) as u64 + inc_millis;
    }
}

fn cmd_uci() -> Either<String, Option<String>> {
    Right(Some(format!(
        "id name Rusty Rival {}
id author Chris Moreton
option name Hash type spin default {} min 1 max 16384
option name Clear Hash type button
option name MultiPV type spin default 1 min 1 max 20
option name Contempt type spin default 0 min -1000 max 1000
option name SyzygyPath type string default <empty>
option name Threads type spin default 1 min 1 max 256
option name EvalNoise type spin default 0 min 0 max 100
option name UseNNUE type check default true
option name Ponder type check default false
option name Move Overhead type spin default 10 min 0 max 5000
option name UCI_ShowWDL type check default false
uciok",
        env!("CARGO_PKG_VERSION"),
        HASH_SIZE_MB
    )))
}

fn cmd_isready() -> Either<String, Option<String>> {
    Right(Some("readyok".parse().unwrap()))
}

fn cmd_debug(uci_state: &mut UciState, parts: Vec<&str>) -> Either<String, Option<String>> {
    if parts.len() != 2 || !["on", "off"].contains(&parts[1]) {
        return Left::<String, Option<String>>("usage: debug [on|off]".parse().unwrap());
    }

    uci_state.debug = parts[1] == "on";

    Right(None)
}

fn cmd_perft(depth: u8, uci_state: &UciState) -> Either<String, Option<String>> {
    let start = Instant::now();
    let nodes = perft(&mut get_position(uci_state.fen.trim()), depth.saturating_sub(1));
    let duration = start.elapsed();
    println!("Time elapsed in perft is: {:?}", duration);
    println!("{} nodes {} nps", nodes, (nodes as f64 / (duration.as_millis() as f64)) * 1000.0);
    Right(None)
}

fn cmd_setoption(parts: Vec<&str>, search_state: &mut SearchState, uci_state: &mut UciState) -> Either<String, Option<String>> {
    if parts.len() < 3 || parts[1] != "name" {
        Left("usage: setoption name <name> [value <value>]".parse().unwrap())
    } else {
        let option = parts[2].to_lowercase();
        match option.as_str() {
            "hash" => {
                if parts.len() == 5 && parts[3] == "value" {
                    match parts[4].parse::<usize>() {
                        Ok(mb) if (1..=16384).contains(&mb) => {
                            search_state.hash_table = Arc::new(SharedHashTable::new_with_mb(mb));
                            Right(None)
                        }
                        _ => Left("Hash size must be between 1 and 16384 MB".parse().unwrap()),
                    }
                } else {
                    Left("usage: setoption name Hash value <MB>".parse().unwrap())
                }
            }
            "clear" => {
                search_state.hash_table.clear();
                Right(None)
            }
            "multipv" => {
                if parts.len() == 5 && parts[3] == "value" {
                    match parts[4].parse::<u8>() {
                        Ok(n) if (1..=20).contains(&n) => {
                            search_state.multi_pv = n;
                            Right(None)
                        }
                        _ => Left("MultiPV must be between 1 and 20".parse().unwrap()),
                    }
                } else {
                    Left("usage: setoption name MultiPV value <N>".parse().unwrap())
                }
            }
            "contempt" => {
                if parts.len() == 5 && parts[3] == "value" {
                    match parts[4].parse::<i32>() {
                        Ok(c) if (-1000..=1000).contains(&c) => {
                            search_state.contempt = c;
                            Right(None)
                        }
                        _ => Left("Contempt must be between -1000 and 1000".parse().unwrap()),
                    }
                } else {
                    Left("usage: setoption name Contempt value <N>".parse().unwrap())
                }
            }
            "threads" => {
                if parts.len() == 5 && parts[3] == "value" {
                    match parts[4].parse::<usize>() {
                        Ok(threads) if (1..=256).contains(&threads) => {
                            uci_state.threads = threads;
                            Right(None)
                        }
                        _ => Left("Threads must be between 1 and 256".parse().unwrap()),
                    }
                } else {
                    Left("usage: setoption name Threads value <N>".parse().unwrap())
                }
            }
            "syzygypath" => {
                // Handle path with spaces by joining everything after "value"
                if parts.len() >= 5 && parts[3] == "value" {
                    let path = parts[4..].join(" ");
                    match init_tablebase(&path) {
                        Ok(count) => {
                            println!("info string Loaded {} tablebase files from {}", count, path);
                            Right(None)
                        }
                        Err(e) => Left(format!("Failed to load tablebases: {}", e)),
                    }
                } else {
                    Left("usage: setoption name SyzygyPath value <path>".parse().unwrap())
                }
            }
            "evalnoise" => {
                if parts.len() == 5 && parts[3] == "value" {
                    match parts[4].parse::<i32>() {
                        Ok(noise) if (0..=100).contains(&noise) => {
                            search_state.eval_noise = noise;
                            Right(None)
                        }
                        _ => Left("EvalNoise must be between 0 and 100".parse().unwrap()),
                    }
                } else {
                    Left("usage: setoption name EvalNoise value <N>".parse().unwrap())
                }
            }
            "usennue" => {
                if parts.len() == 5 && parts[3] == "value" {
                    search_state.use_nnue = parts[4].eq_ignore_ascii_case("true");
                }
                Right(None)
            }
            "ponder" => {
                if parts.len() == 5 && parts[3] == "value" {
                    uci_state.ponder_enabled = parts[4].eq_ignore_ascii_case("true");
                }
                Right(None)
            }
            "move" if parts.len() >= 4 && parts[3].to_lowercase() == "overhead" => {
                if parts.len() == 6 && parts[4] == "value" {
                    if let Ok(ms) = parts[5].parse::<u64>() {
                        uci_state.move_overhead = ms.min(5000);
                    }
                }
                Right(None)
            }
            "uci_showwdl" => Right(None),
            _ => Left("Unknown option".parse().unwrap()),
        }
    }
}

fn cmd_register() -> Either<String, Option<String>> {
    Right(None)
}

fn cmd_ucinewgame(
    uci_state: &mut UciState,
    search_state: &mut SearchState,
    search_handle: &mut Option<SearchHandle>,
) -> Either<String, Option<String>> {
    // Stop any running search first. It still absorbs, so the merge-then-clear
    // order is explicit rather than accidental: the tables are moved into the
    // master and then wiped below, which is what a new game requires.
    if let Some(handle) = search_handle.take() {
        handle.stop_and_wait(search_state);
    }

    search_state.nodes = 0;

    search_state.qnodes = 0;

    search_state.tt_probes = 0;

    search_state.tt_hits = 0;

    search_state.tt_deep_enough = 0;

    search_state.tt_slot_taken = 0;

    search_state.scout_searches = 0;

    search_state.research_lmr_full = 0;

    search_state.research_full_depth = 0;

    search_state.research_pvs = 0;

    search_state.children_searched = 0;

    search_state.no_cutoff_nodes = 0;

    search_state.no_cutoff_children = 0;

    search_state.cutoffs = 0;

    search_state.cutoffs_first_move = 0;

    search_state.cutoff_by_kind = [0; 7];

    search_state.cutoff_by_index = [0; 5];
    // Clear root_moves and pv to prevent stale data from previous games
    // being output if time expires before the first search iteration completes
    search_state.root_moves.clear();
    search_state.pv.clear();
    search_state.hash_table.clear();
    // History tables persist across searches within a game - a new game needs
    // a full reset, not the usual decay.
    //
    // NET-372 makes that persistence real, and so makes this reset load-bearing
    // for the first time. Before, the master's tables were never written, so
    // clearing them was a no-op. COUNTERMOVES now genuinely carry, and without
    // this a new game would start with the previous game's countermoves.
    //
    // clear_killers is redundant rather than load-bearing: iterative_deepening
    // already clears killers at the start of every search, so they cannot
    // survive to a new game by any route. It is kept because this is the one
    // place a reader looks for the game-boundary reset, and having the boundary
    // state the full set is worth more than saving four lines.
    //
    // Only the UCI path clears these three. The sync path used by `bench`
    // deliberately still does not, so the deterministic bench signature is
    // unaffected - `bench` clears countermoves itself, per position.
    clear_history_table(search_state);
    clear_killers(search_state);
    clear_countermoves(search_state);
    uci_state.fen = START_POS.parse().unwrap();
    Right(None)
}
