use std::cmp::{max, min};
use std::ops::Add;
use std::process::exit;
use std::time::{Duration, Instant};
use either::{Either, Left, Right};
use num_format::{Locale, ToFormattedString};
use regex::Regex;
use crate::engine_constants::{NUM_HASH_ENTRIES, UCI_MILLIS_REDUCTION};

use crate::fen::{algebraic_move_from_move, get_fen, get_position};
use crate::make_move::make_move;
use crate::move_constants::{START_POS};
use crate::moves::{is_check, moves};
use crate::perft::perft;
use crate::search::{iterative_deepening};
use crate::types::{BoundType, HashEntry, Position, SearchState, UciState, WHITE};
use crate::utils::hydrate_move_from_algebraic_move;

fn replace_shortcuts(l: &str) -> &str {
    match l {
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
        "i" => "go infinite",
        _ => l
    }
}

pub fn run_command_test(uci_state: &mut UciState, search_state: &mut SearchState, l: &str) -> Either<String, Option<String>> {
    run_command(uci_state, search_state, replace_shortcuts(l))
}

pub fn run_command(uci_state: &mut UciState, search_state: &mut SearchState, l: &str) -> Either<String, Option<String>> {

    let mut trimmed_line = replace_shortcuts(l).trim().replace("  ", " ");
    if trimmed_line.starts_with("position startpos") {
        trimmed_line = trimmed_line.replace("startpos", &*("fen ".to_string() + START_POS));
    }
    let parts = trimmed_line.split(' ').collect::<Vec<&str>>();

    match *parts.get(0).unwrap() {
        "bench" => {
            cmd_benchmark(uci_state, search_state)
        },
        "uci" => {
            cmd_uci()
        },
        "isready" => {
            cmd_isready()
        },
        "state" => {
            cmd_state(uci_state, search_state)
        },
        "go" => {
            cmd_go(uci_state, search_state, parts)
        },
        "setoption" => {
            cmd_setoption(parts, search_state)
        },
        "register" => {
            cmd_register()
        },
        "ucinewgame" => {
            cmd_ucinewgame(uci_state, search_state)
        },
        "debug" => {
            cmd_debug(uci_state, parts)
        },
        "quit" => {
            exit(0)
        },
        "position" => {
            cmd_position(uci_state, search_state, parts)
        }
        _ => {
            Left("Unknown command".parse().unwrap())
        }
    }
}

fn fen_and_moves(parts: Vec<&str>) -> (String, Vec<String>) {

    if !parts.contains(&"moves") {
        let fen = parts.join(" ").replace("position fen", "");
        return (fen.trim().parse().unwrap(), vec![])
    }
    let fen_and_moves_string = parts.join(" ").replace("position fen", "");
    let two_parts = fen_and_moves_string.split("moves").collect::<Vec<&str>>();
    let fen = two_parts[0];
    let moves = two_parts[1].trim().split(' ').collect::<Vec<&str>>().into_iter().map(|move_string| {
       move_string.to_string()
    }).collect();

    (fen.trim().parse().unwrap(), moves)
}

pub fn is_legal_move(position: &Position, algebraic_move: &str) -> bool {
    let moves = moves(position);
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
    let t = parts.get(1).unwrap();
    match *t {
        "fen" => {

            search_state.history = vec![];
            search_state.hash_table_version += 1;

            let re = Regex::new(r"\s*^(((?:[rnbqkpRNBQKP1-8]+/){7})[rnbqkpRNBQKP1-8]+)\s([b|w])\s([K|Q|k|q]{1,4}|-)\s(-|[a-h][1-8])\s(\d+\s\d+)$").unwrap();
            let (fen, moves) = fen_and_moves(parts);

            uci_state.fen = fen.parse().unwrap();

            if re.is_match(&*fen) {
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
        },
        _ => {
            Left("Unknown position command".parse().unwrap())
        }
    }
}

pub fn extract_go_param(needle: &str, haystack: &str, default: u64) -> u64 {
    let re = r"".to_string() + &*needle.to_string() + &*" ([0-9]*)".to_string();
    let regex = Regex::new(&*re).unwrap();
    let caps = regex.captures(&*haystack);
    match caps {
        Some(x) => {
            let s = x.get(1).unwrap().as_str();
            s.parse::<u64>().unwrap()
        },
        None => {
            default
        }
    }
}

fn cmd_state(mut _uci_state: &mut UciState, search_state: &mut SearchState) -> Either<String, Option<String>> {
    Right(Some(format!(r#"Nodes {}"#, search_state.nodes)))
}

fn cmd_go(mut uci_state: &mut UciState, search_state: &mut SearchState, parts: Vec<&str>) -> Either<String, Option<String>> {
    let t = parts.get(1).unwrap();
    search_state.nodes = 0;

    match *t {
        "perft" => {
            let depth = parts.get(2).unwrap().to_string().parse().unwrap();
            cmd_perft(depth, uci_state);
            Right(None)
        },
        "infinite" => {
            let position = get_position(uci_state.fen.trim());
            search_state.end_time = Instant::now().add(Duration::from_secs(86400));
            let mv = iterative_deepening(&position, 200, search_state);
            Right(Some("bestmove ".to_owned() + &algebraic_move_from_move(mv)))
        },
        "mate" => {
            let position = get_position(uci_state.fen.trim());
            search_state.end_time = Instant::now().add(Duration::from_secs(86400));
            let mv = iterative_deepening(&position, 200, search_state);
            Right(Some("bestmove ".to_owned() + &algebraic_move_from_move(mv)))
        },
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

            let position = get_position(uci_state.fen.trim());

            if position.mover == WHITE {
                calc_from_colour_times(uci_state, uci_state.wtime, uci_state.winc);
            } else {
                calc_from_colour_times(uci_state, uci_state.btime, uci_state.binc);
            }

            uci_state.move_time = max(10, uci_state.move_time - min(uci_state.move_time, UCI_MILLIS_REDUCTION as u64) as u64) as u64;

            search_state.end_time = Instant::now().add(Duration::from_millis(uci_state.move_time));
            let mv = iterative_deepening(&position, uci_state.depth as u8, search_state);

            Right(Some("bestmove ".to_owned() + &algebraic_move_from_move(mv)))
        }
    }
}

fn calc_from_colour_times(mut uci_state: &mut UciState, millis: u64, inc_millis: u64) {
    if millis > 0 {
        uci_state.move_time = if uci_state.moves_to_go == 0 {
            millis
        } else {
            min(uci_state.move_time, (millis as f64 / (uci_state.moves_to_go as f64 + 1.0) as f64) as u64)
        };
        uci_state.move_time = (uci_state.move_time as f64 * 0.95) as u64 + inc_millis;
    }
}

fn cmd_uci() -> Either<String, Option<String>> {
    Right(Some("id name Rusty Rival |Contest-Abandon|\nid author Chris Moreton\noption name Clear Hash type button\nuciok".parse().unwrap()))
}

fn cmd_isready() -> Either<String, Option<String>> {
    Right(Some("readyok".parse().unwrap()))
}

fn cmd_debug(mut uci_state: &mut UciState, parts: Vec<&str>) -> Either<String, Option<String>> {
    if parts.len() != 2 || !["on", "off"].contains(&parts[1]) {
        return Left::<String, Option<String>> ("usage: debug [on|off]".parse().unwrap());
    }

    uci_state.debug = parts[1] == "on";

    Right(None)
}


fn cmd_benchmark(uci_state: &mut UciState, search_state: &mut SearchState) -> Either<String, Option<String>> {
    let start = Instant::now();

    let positions = vec![
        ("P2", "8/5p2/5k1p/p6P/PP2K1P1/8/8/8 b - - 0 67", "a5b4", 16),
        ("P3", "8/6nk/8/1p2P1Q1/1P4PP/P4q2/8/6K1 w - - 3 56", "h4h5", 12),
        ("Rusty1", "1R6/1brk2p1/4p2p/p1P1Pp2/P7/6P1/1P4P1/2R3K1 w - - 0 1", "b8b7", 13),
        ("Rusty2", "r1b2rk1/ppq1bppp/2p1pn2/8/2NP4/2N1P3/PP2BPPP/2RQK2R w K - 0 1", "e1g1", 8),
        ("Rusty4", "N7/8/2KQ2rp/6k1/3p3p/2p4P/4PP2/5N2 w - - 0 1", "a8c7", 11),
        ("Passant", "rnbqkbnr/ppppp1pp/8/4P3/5pP1/8/PPPP1P1P/RNBQKBNR b QKqk g3 0 3", "d7d6", 9),
        ("Start", "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", "g1f3", 9),
        ("KQkrp 3", "8/8/1P6/5pr1/8/4R3/7k/2K5 w - - 0 1", "b6b7", 10),
        ("KQkrp 1", "8/2p4P/8/kr6/6R1/8/8/1K6 w - - 0 1", "b1c2", 12),
        ("Promote", "rnb1qbnr/pppPk1pp/8/8/8/8/PPPP1PPP/RNBQKBNR w QK - 1 5", "d7e8q", 9),
        ("P5", "2r5/p2Qbkpp/4p3/5p2/2P4q/P2P2N1/1r3P1P/R3K2b w Q - 0 20", "g3h1", 7),
        ("Ruy", "r1bqk2r/2pp1ppp/p1n2n2/1pb1p3/4P3/1B3N2/PPPP1PPP/RNBQ1RK1 w qk - 4 7", "d2d3", 9),
        ("EndSwap", "5k2/5p1p/p3n1p1/P5P1/2BK1P1P/8/8/8 w - - 5 1", "c4e6", 14),
        ("HPawn3", "8/8/7p/P2k3p/7p/2b5/8/6K1 w - - 0 1", "a5a6", 16),
        ("WAC 3", "5rk1/1ppb3p/p1pb4/6q1/3P1p1r/2P1R2P/PP1BQ1P1/5RKN w - - 0 1", "e3g3", 12),
        ("WAC 6", "7k/p7/1R5K/6r1/6p1/6P1/8/8 w - - 0 1 ", "b6b7", 17),
        ("WAC 7", "rnbqkb1r/pppp1ppp/8/4P3/6n1/7P/PPPNPPP1/R1BQKBNR b KQkq - 0 1 ", "g4e3", 12),
        ("WAC 8", "r4q1k/p2bR1rp/2p2Q1N/5p2/5p2/2P5/PP3PPP/R5K1 w - - 0 1", "e7f7", 10),
        ("WAC 10", "2br2k1/2q3rn/p2NppQ1/2p1P3/Pp5R/4P3/1P3PPP/3R2K1 w - - 0 1 ", "h4h7", 10),
        ("WAC 11", "r1b1kb1r/3q1ppp/pBp1pn2/8/Np3P2/5B2/PPP3PP/R2Q1RK1 w kq - 0 1 ", "f3c6", 9),
        ("P1", "r3kbnr/pp1qpppp/3pb3/8/3BP3/2NB1N2/PPP2PPP/R3K2R b QKqk - 4 8", "a7a6", 9),
        ("Qc3 bad", "2r3k1/q4ppp/3p4/1r1Pp3/2n3P1/pN1R1P1P/P1P5/2KRQ3 w - - 8 1", "d3c3", 11),
        ("B Prom", "2r5/Br4pp/1P1k4/3P1p2/1R1R1P2/p6P/1n3P2/6K1 w - - 2 35", "g1g2", 12),
        ("Qa6 bad", "2r2rk1/p4ppp/2Q5/4b3/1q6/4P3/PP3PPP/R1B2K1R w - - 3 19", "c6a6", 11),
        ("P4", "5rk1/1b3p1p/pp3p2/3n1N2/1P6/P1qB1PP1/3Q3P/4R1K1 w - - 0 1", "d2h6", 8),
        ("Pawn Up", "6k1/6p1/1p2q2p/1p5P/1P3RP1/2PK1B2/1r2N3/8 b - g3 0 56", "e6d6", 10),
        ("Heavy D", "4k3/1p3pp1/7p/1p6/1P2P2P/2PRKB2/4N1P1/2r1q3 b - - 0 40 ", "g7g5", 10),
        ("P6", "r2r3k/5ppp/p7/3p4/P1pN1Nq1/5nPP/1PR5/4RK2 w - - 0 1", "h3g4", 12),
        ("P7", "r1b4k/1p1n4/p7/3Q4/8/2qBB2P/PrP2PP1/b2R2RK w - - 9 6", "e3d4", 7),
        ("P8", "8/8/6p1/7p/7P/R4k2/p6K/r7 b - - 95 105", "f3f2", 15),
        ("P9", "8/8/R5p1/3k3p/7P/8/p6K/r7 b - - 95 104", "a1c1", 14),
        ("P10", "rq3rk1/2bn1ppn/2p1p3/p2p2PR/P1PP4/N3B3/4PP2/R2QKB2 b Q - 0 21", "g7g6", 8),
        ("P11", "r2r3k/5ppp/p7/Q2p4/P1pN1Nq1/5nPP/1PR5/4RK2 b - - 0 1", "g4g3", 11),
        ("P12", "8/2pb4/4k3/5pp1/8/r4PbB/8/3R2K1 w - f6 0 35", "h3f5", 11),
        ("P13", "8/p5pp/4pk2/5p2/1QP5/P7/5P1P/R3K2N b Q - 0 5", "g7g5", 9),
        ("P14", "2r3k1/1rnq1pb1/2Rp1npp/1p1Pp3/1P2P3/3BBPPN/3Q2K1/4R3 b - - 3 1", "g6g5", 9),
        ("P15", "r1b1kbnr/2ppqppp/p1n5/1p2p3/4P3/1B3N2/PPPP1PPP/RNBQK2R w QKqk - 2 6", "b1c3", 9),
        ("B+N", "8/2k5/8/8/4K3/2B5/4N3/8 w - - 0 1", "e4d5", 11),
        ("RvR", "8/8/3k4/6R1/8/4K3/r7/8 b - - 54 117", "d6c7", 10),

    ];

    let total = positions.len();

    let show_info = search_state.show_info;
    search_state.show_info = false;

    let mut total_nodes = 0;
    let mut total_correct = 0;
    for p in positions {
        run_command(uci_state, search_state, "ucinewgame");
        let mut owned = "position fen ".to_owned();
        owned.push_str(p.1);
        run_command(uci_state, search_state, &owned);
        run_command(uci_state, search_state, &format!("go depth {}", p.3));
        let am = algebraic_move_from_move(search_state.current_best.0[0]);
        if am == p.2 {
            total_correct += 1;
        }
        //println!("(\"{}\", \"{}\", \"{}\", {}),", p.0, p.1, am, p.3);
        println!("{}: Nodes {}", p.0, search_state.nodes.to_formatted_string(&Locale::en));
        total_nodes += search_state.nodes;
    }
    let duration = start.elapsed();
    println!("Time elapsed is: {:?}", duration);
    println!("Correct: {:?}/{}", total_correct, total);
    let nps = (total_nodes as f64 / start.elapsed().as_millis() as f64) * 1000.0;

    println!("{} nodes {} nps", total_nodes.to_formatted_string(&Locale::en), &*(nps as u64).to_string());

    search_state.show_info = show_info;

    Right(None)
}

fn cmd_perft(depth: u8, uci_state: &UciState) -> Either<String, Option<String>> {
    let start = Instant::now();
    let nodes = perft(&get_position(uci_state.fen.trim()), depth - 1);
    let duration = start.elapsed();
    println!("Time elapsed in perft is: {:?}", duration);
    println!("{} nodes {} nps", nodes, (nodes as f64 / (duration.as_millis() as f64)) * 1000.0);
    Right(None)
}

fn cmd_setoption(parts: Vec<&str>, search_state: &mut SearchState) -> Either<String, Option<String>> {
    if parts.len() < 3 || parts[1] != "name" {
        Left("usage: setoption name <name> [value <value>]".parse().unwrap())
    } else {
        match parts[2] {
            "Clear" => {
                for i in 0..NUM_HASH_ENTRIES {
                    search_state.hash_table_height[i as usize] = HashEntry {
                        score: 0,
                        version: 0,
                        height: 0,
                        mv: 0,
                        bound: BoundType::Exact,
                        lock: 0
                    }
                };
                Right(None)
            },
            _ => {
                Left("Unknown option".parse().unwrap())
            }
        }
    }
}

fn cmd_register() -> Either<String, Option<String>> {
    Right(None)
}

fn cmd_ucinewgame(mut uci_state: &mut UciState, mut search_state: &mut SearchState) -> Either<String, Option<String>> {
    search_state.nodes = 0;
    for i in 0..NUM_HASH_ENTRIES {
        search_state.hash_table_height[i as usize] = HashEntry {
            score: 0,
            version: 0,
            height: 0,
            mv: 0,
            bound: BoundType::Exact,
            lock: 0
        }
    };
    uci_state.fen = START_POS.parse().unwrap();
    Right(None)
}
