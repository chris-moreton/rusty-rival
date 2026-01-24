use crate::engine_constants::UCI_MILLIS_REDUCTION;
use crate::tablebase::init_tablebase;

use either::{Either, Left, Right};

use regex::Regex;
use std::cmp::{max, min};
use std::ops::Add;
use std::process::exit;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use crate::fen::{algebraic_move_from_move, get_fen, get_position};
use crate::make_move::make_move;
use crate::move_constants::START_POS;
use crate::moves::{generate_moves, is_check};

use crate::perft::perft;
use crate::search::iterative_deepening;
use crate::types::{set_stop, Position, SearchHandle, SearchState, SharedHashTable, UciState, BLACK, WHITE};
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
            search_state.root_moves.clear();
            search_state.pv.clear();
            search_state.hash_table.clear();
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
    let t = parts.get(1).unwrap();
    search_state.nodes = 0;
    search_state.nodes_limit = u64::MAX;
    set_stop(&search_state.stop, false);

    match *t {
        "perft" => {
            let depth = parts.get(2).unwrap().to_string().parse().unwrap();
            cmd_perft(depth, uci_state);
            Right(None)
        }
        "infinite" => {
            let mut position = get_position(uci_state.fen.trim());
            search_state.end_time = Instant::now().add(Duration::from_secs(86400));
            let mv = iterative_deepening(&mut position, 200, search_state);
            Right(Some(format_bestmove(mv, search_state)))
        }
        "mate" => {
            let mate_depth = parts.get(2).and_then(|s| s.parse::<u8>().ok()).unwrap_or(100);
            let mut position = get_position(uci_state.fen.trim());
            search_state.end_time = Instant::now().add(Duration::from_secs(86400));
            let mv = iterative_deepening(&mut position, mate_depth.saturating_mul(2), search_state);
            Right(Some(format_bestmove(mv, search_state)))
        }
        _ => {
            let line = parts.join(" ");
            uci_state.wtime = extract_go_param("wtime", &line, 0);
            uci_state.btime = extract_go_param("btime", &line, 0);
            uci_state.winc = extract_go_param("winc", &line, 0);
            uci_state.binc = extract_go_param("binc", &line, 0);
            uci_state.moves_to_go = extract_go_param("movestogo", &line, 0);
            uci_state.depth = extract_go_param("depth", &line, 250);
            uci_state.nodes = extract_go_param("nodes", &line, u64::MAX);
            uci_state.move_time = extract_go_param("movetime", &line, 10000000);

            search_state.nodes_limit = uci_state.nodes;

            let mut position = get_position(uci_state.fen.trim());

            if position.mover == WHITE {
                calc_from_colour_times(uci_state, uci_state.wtime, uci_state.winc);
            } else {
                calc_from_colour_times(uci_state, uci_state.btime, uci_state.binc);
            }

            uci_state.move_time = max(10, uci_state.move_time - min(uci_state.move_time, UCI_MILLIS_REDUCTION as u64));

            search_state.end_time = Instant::now().add(Duration::from_millis(uci_state.move_time));
            let mv = iterative_deepening(&mut position, uci_state.depth as u8, search_state);

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
        "uci" => cmd_uci(),
        "isready" => cmd_isready(),
        "state" => cmd_state(uci_state, search_state),
        "go" => cmd_go(uci_state, search_state, search_handle, parts),
        "stop" => cmd_stop(search_handle),
        "setoption" => cmd_setoption(parts, search_state),
        "register" => cmd_register(),
        "ucinewgame" => cmd_ucinewgame(uci_state, search_state, search_handle),
        "debug" => cmd_debug(uci_state, parts),
        "quit" => {
            // Stop any running search before quitting
            if let Some(handle) = search_handle.take() {
                handle.stop_and_wait();
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
    let t = parts.get(1).unwrap();
    match *t {
        "fen" => {
            search_state.history = vec![];

            let re = Regex::new(
                r"\s*^(((?:[rnbqkpRNBQKP1-8]+/){7})[rnbqkpRNBQKP1-8]+)\s([b|w])\s([K|Q|k|q]{1,4}|-)\s(-|[a-h][1-8])\s(\d+\s\d+)$",
            )
            .unwrap();
            let (fen, moves) = fen_and_moves(parts);

            uci_state.fen = fen.parse().unwrap();

            if re.is_match(&fen) {
                uci_state.fen = fen;
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

pub fn extract_go_param(needle: &str, haystack: &str, default: u64) -> u64 {
    let re = r"".to_string() + &*needle.to_string() + &*" ([0-9]*)".to_string();
    let regex = Regex::new(&re).unwrap();
    let caps = regex.captures(haystack);
    match caps {
        Some(x) => {
            let s = x.get(1).unwrap().as_str();
            s.parse::<u64>().unwrap()
        }
        None => default,
    }
}

fn cmd_state(mut _uci_state: &mut UciState, search_state: &mut SearchState) -> Either<String, Option<String>> {
    Right(Some(format!(r#"Nodes {}"#, search_state.nodes)))
}

fn cmd_mvm(search_state: &mut SearchState, parts: Vec<&str>) -> Either<String, Option<String>> {
    let millis = parts.get(1).unwrap().to_string().parse().unwrap();
    let count = parts.get(2).unwrap().to_string().parse().unwrap();
    let mut engine_1_wins = 0;
    let mut engine_2_wins = 0;
    let mut draws = 0;

    search_state.show_info = false;

    for g in 0..count {
        let engine_1_colour = if g % 2 == 0 { WHITE } else { BLACK };
        let mut position = get_position(START_POS);
        let final_position = loop {
            search_state.end_time = Instant::now().add(Duration::from_millis(millis));
            let mv = iterative_deepening(&mut position, 100_u8, search_state);
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
    // If there's already a search running, wait for it first
    if let Some(handle) = search_handle.take() {
        handle.stop_and_wait();
    }

    let t = parts.get(1).unwrap();

    // perft runs synchronously (no threading needed)
    if *t == "perft" {
        let depth = parts.get(2).unwrap().to_string().parse().unwrap();
        cmd_perft(depth, uci_state);
        return Right(None);
    }

    // Parse go parameters - calculate move_time_ms to pass to thread
    // The actual end_time will be calculated INSIDE the thread to avoid
    // counting clone/spawn overhead against search time
    let line = parts.join(" ");
    let (max_depth, move_time_ms, nodes_limit) = if *t == "infinite" {
        (200u8, 86400 * 1000u64, u64::MAX) // 24 hours in ms
    } else if *t == "mate" {
        let mate_depth = parts.get(2).and_then(|s| s.parse::<u8>().ok()).unwrap_or(100);
        (mate_depth.saturating_mul(2), 86400 * 1000u64, u64::MAX)
    } else {
        uci_state.wtime = extract_go_param("wtime", &line, 0);
        uci_state.btime = extract_go_param("btime", &line, 0);
        uci_state.winc = extract_go_param("winc", &line, 0);
        uci_state.binc = extract_go_param("binc", &line, 0);
        uci_state.moves_to_go = extract_go_param("movestogo", &line, 0);
        uci_state.depth = extract_go_param("depth", &line, 250);
        uci_state.nodes = extract_go_param("nodes", &line, u64::MAX);
        uci_state.move_time = extract_go_param("movetime", &line, 10000000);

        let position = get_position(uci_state.fen.trim());
        if position.mover == WHITE {
            calc_from_colour_times(uci_state, uci_state.wtime, uci_state.winc);
        } else {
            calc_from_colour_times(uci_state, uci_state.btime, uci_state.binc);
        }

        uci_state.move_time = max(10, uci_state.move_time - min(uci_state.move_time, UCI_MILLIS_REDUCTION as u64));

        (uci_state.depth as u8, uci_state.move_time, uci_state.nodes)
    };

    // Clone position for the search thread
    let mut position = get_position(uci_state.fen.trim());

    // Create a new stop flag for this search
    let stop_flag = Arc::new(AtomicBool::new(false));

    // Clone search_state for the thread, but use the new stop flag
    // Note: hash_table is shared via Arc (no 128MB copy!)
    let mut thread_search_state = search_state.clone();
    thread_search_state.nodes = 0;
    thread_search_state.nodes_limit = nodes_limit;
    thread_search_state.stop = stop_flag.clone();

    // Spawn the search thread with a larger stack size to prevent stack overflow
    // during deep searches (default 2MB is not enough for very deep positions)
    let handle = thread::Builder::new()
        .stack_size(16 * 1024 * 1024) // 16 MB stack (matches RUST_MIN_STACK recommendation)
        .spawn(move || {
            // Calculate end_time HERE inside the thread - this ensures clone/spawn
            // overhead doesn't eat into search time
            thread_search_state.end_time = Instant::now().add(Duration::from_millis(move_time_ms));
            let mv = iterative_deepening(&mut position, max_depth, &mut thread_search_state);
            println!("{}", format_bestmove(mv, &thread_search_state));
        })
        .expect("Failed to spawn search thread");

    // Store the search handle
    *search_handle = Some(SearchHandle { stop: stop_flag, handle });

    Right(None)
}

fn cmd_stop(search_handle: &mut Option<SearchHandle>) -> Either<String, Option<String>> {
    if let Some(handle) = search_handle.take() {
        set_stop(&handle.stop, true);
        let _ = handle.handle.join();
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
option name Hash type spin default 128 min 1 max 16384
option name Clear Hash type button
option name MultiPV type spin default 1 min 1 max 20
option name Contempt type spin default 0 min -1000 max 1000
option name SyzygyPath type string default <empty>
uciok",
        env!("CARGO_PKG_VERSION")
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
    let nodes = perft(&mut get_position(uci_state.fen.trim()), depth - 1);
    let duration = start.elapsed();
    println!("Time elapsed in perft is: {:?}", duration);
    println!("{} nodes {} nps", nodes, (nodes as f64 / (duration.as_millis() as f64)) * 1000.0);
    Right(None)
}

fn cmd_setoption(parts: Vec<&str>, search_state: &mut SearchState) -> Either<String, Option<String>> {
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
                            search_state.hash_table_version += 1;
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
                if parts.len() == 5 {
                    search_state.multi_pv = parts[4].parse().unwrap();
                    Right(None)
                } else {
                    Left("Invalid option command".parse().unwrap())
                }
            }
            "contempt" => {
                if parts.len() == 5 {
                    search_state.contempt = parts[4].parse().unwrap();
                    Right(None)
                } else {
                    Left("Invalid option command".parse().unwrap())
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
    // Stop any running search first
    if let Some(handle) = search_handle.take() {
        handle.stop_and_wait();
    }

    search_state.nodes = 0;
    // Clear root_moves and pv to prevent stale data from previous games
    // being output if time expires before the first search iteration completes
    search_state.root_moves.clear();
    search_state.pv.clear();
    search_state.hash_table.clear();
    uci_state.fen = START_POS.parse().unwrap();
    Right(None)
}
