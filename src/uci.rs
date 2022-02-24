use std::ops::Add;
use std::process::exit;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};
use either::{Either, Left, Right};
use regex::Regex;

use crate::fen::{algebraic_move_from_move, get_fen, get_position};
use crate::make_move::make_move;
use crate::move_constants::{START_POS};
use crate::moves::{is_check, moves};
use crate::perft::perft;
use crate::search::{iterative_deepening};
use crate::types::{Position, SearchState, UciState};
use crate::utils::hydrate_move_from_algebraic_move;

fn replace_shortcuts(l: &str) -> &str {
    match l {
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
        "tf01" => "position fen 3Nk3/4p3/2p2p2/1bp2p2/3b1Pn1/2N5/1PP3PP/2BQK2R b K - 0 1",
        "tf02" => "position fen 3Nk3/4p3/2p2p2/1bp2p2/3b1Pn1/2N5/1PP3PP/2BQK2R b K - 0 1 moves d4f2 e1d2 f2e3 d2e1",
        "tf03" => "position fen 3Nk3/4p3/2p2p2/1bp2p2/3b1Pn1/2N5/1PP3PP/2BQK2R b K - 0 1 moves d4f2 e1d2 f2e3 d2e1 e3f2 e1d2",
        "st01" => "position fen 8/8/8/8/4Q3/2P4k/8/5K2 w - - 0 1",
        "st02" => "position fen 8/8/8/8/4Q3/2P3k1/4K3/8 w - - 0 1",
        "st03" => "position fen 8/8/8/8/4Q3/2PK3k/8/8 w - - 0 1",
        "bench01" => "position fen 8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
        "bench02" => "position fen 8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
        "bench03" => "position fen 8/7p/p5pb/4k3/P1pPn3/8/P5PP/1rB2RK1 b - d3 0 28",
        "bench04" => "position fen r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R3K2R w KQkq - 0 1",
        "bench05" => "position fen r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        "bench06" => "position fen 8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
        "bench07" => "position fen 8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
        "bench08" => "position fen r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        "bench09" => "position startpos",
        "i" => "go infinite",
        _ => l
    }
}

pub fn run_command_test(uci_state: &mut UciState, search_state: &mut SearchState, l: &str) -> Either<String, Option<String>> {
    let (tx, _rx) = mpsc::channel();
    run_command(uci_state, search_state, replace_shortcuts(l), &tx)
}

pub fn run_command(uci_state: &mut UciState, search_state: &mut SearchState, l: &str, tx: &Sender<String>) -> Either<String, Option<String>> {

    let mut trimmed_line = replace_shortcuts(l).trim().clone().replace("  ", " ");
    if trimmed_line.starts_with("position startpos") {
        trimmed_line = trimmed_line.replace("startpos", &*("fen ".to_string() + START_POS));
    }
    let parts = trimmed_line.split(' ').collect::<Vec<&str>>();

    match *parts.get(0).unwrap() {
        "bench" => {
            cmd_benchmark(uci_state, search_state, tx)
        },
        "uci" => {
            cmd_uci()
        },
        "isready" => {
            cmd_isready()
        },
        "go" => {
            cmd_go(uci_state, search_state, parts, tx)
        },
        "setoption" => {
            cmd_setoption(search_state, parts)
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
                if moves.len() > 0 {
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
                return Right(None)
            } else {
                return Left("Invalid FEN".parse().unwrap())
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

fn cmd_go(mut uci_state: &mut UciState, search_state: &mut SearchState, parts: Vec<&str>, tx: &Sender<String>) -> Either<String, Option<String>> {
    let t = parts.get(1).unwrap();
    search_state.nodes = 0;

    match *t {
        "perft" => {
            let depth = parts.get(2).unwrap().to_string().parse().unwrap();
            cmd_perft(depth, &uci_state);
            Right(None)
        },
        "infinite" => {
            let position = get_position(uci_state.fen.trim());
            let mv = iterative_deepening(&position, 200, Instant::now().add(Duration::from_secs(86400)), search_state, tx);
            Right(Some("bestmove ".to_owned() + &algebraic_move_from_move(mv).clone()))
        },
        "mate" => {
            let position = get_position(uci_state.fen.trim());
            let mv = iterative_deepening(&position, 200, Instant::now().add(Duration::from_secs(86400)), search_state, tx);
            Right(Some("bestmove ".to_owned() + &algebraic_move_from_move(mv).clone()))
        },
        _ => {
            let line = parts.join(" ").to_string();
            uci_state.wtime = extract_go_param("wtime", &line, 0);
            uci_state.btime = extract_go_param("btime", &line, 0);
            uci_state.winc = extract_go_param("winc", &line, 0);
            uci_state.binc = extract_go_param("binc", &line, 0);
            uci_state.moves_to_go = extract_go_param("movestogo", &line, 0);
            uci_state.depth = extract_go_param("depth", &line, 250);
            uci_state.nodes = extract_go_param("nodes", &line, u64::MAX);
            uci_state.move_time = extract_go_param("movetime", &line, 10000000);

            let position = get_position(uci_state.fen.trim());
            let mv = iterative_deepening(&position, uci_state.depth as u8, Instant::now().add(Duration::from_millis(uci_state.move_time)), search_state, tx);

            Right(Some("bestmove ".to_owned() + &algebraic_move_from_move(mv).clone()))
        }
    }
}

fn cmd_uci() -> Either<String, Option<String>> {
    Right(Some("id name Rusty Rival |20220224-03-Isolated-Pawns|\nid author Chris Moreton\noption name Clear Hash type button\nuciok".parse().unwrap()))
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


fn cmd_benchmark(uci_state: &mut UciState, search_state: &mut SearchState, tx: &Sender<String>) -> Either<String, Option<String>> {
    let start = Instant::now();

    let positions = vec![
        ("7R/r1p1q1pp/3k4/1p1n1Q2/3N4/8/1PP2PPP/2B3K1 w - - 1 0", 6),
        ("8/8/8/8/4Q3/2PK3k/8/8 w - - 0 1", 6),
        ("6k1/3b3r/1p1p4/p1n2p2/1PPNpP1q/P3Q1p1/1R1RB1P1/5K2 b - - 0 1", 6),
        ("8/8/8/1K6/4Q3/2P5/5k2/8 w - - 0 1", 6),
        ("3Nk3/4p3/2p2p2/1bp2p2/3b1Pn1/2N5/1PP3PP/2BQK2R b K - 0 1", 7),
        ("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", 7),
        ("8/7p/p5pb/4k3/P1pPn3/8/P5PP/1rB2RK1 b - d3 0 28", 6),
        ("r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R3K2R w KQkq - 0 1", 6),
        ("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", 4),
        ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 6),
        ("r1b2k1b/1p3p2/pP2pNp1/P1p1P1P1/6q1/7P/3B3R/R4K2 w - - 0 32", 6),
        ("8/R1N2k2/3R3p/6p1/2P3P1/1P5P/P4P2/4r1K1 w - - 3 45", 6),
        ("5rk1/5ppp/8/1pPpR3/1p3P2/2nB4/q5PP/4Q1K1 w - - 1 24", 6),
        ("r2q1rk1/3b2p1/p2p3p/1ppPbp2/4P3/1P1P3P/P3Q1PN/R1B2RK1 w - - 0 19", 6),
        ("3r4/2p2p2/p7/1pb1p3/4Bk1p/1PN2P1P/P1P3K1/8 b - - 0 29", 6),
        ("rnbqkbnr/1pp1pppp/p7/8/2pP4/2N2N2/PP2PPPP/R1BQKB1R b KQkq - 1 4", 6),
        ("3r2k1/4bpp1/p3p3/1ppn1r1p/4q3/P1B1PNPP/1P2QPK1/4R1R1 b - - 1 26", 6),
        ("rn1qkb1r/1pp1ppp1/p4n1p/3p1b2/3P1B2/2N1PN1P/PPP2PP1/R2QKB1R b KQkq - 2 6", 6),
        ("2r1kb1r/p2p2pp/3n3n/1b2Nq2/4pP2/1B4P1/1BPN3P/R2QK2R w KQk - 0 18", 6),
    ];

    let mut total_nodes = 0;
    for p in positions {
        println!("{}", p.0);
        let mut owned = "position fen ".to_owned();
        owned.push_str(p.0);
        run_command(uci_state, search_state, &owned, tx);
        run_command(uci_state, search_state, &format!("go depth {}", p.1), tx);
        total_nodes += search_state.nodes;
    }
    let duration = start.elapsed();
    println!("Time elapsed is: {:?}", duration);
    let nps = (total_nodes as f64 / start.elapsed().as_millis() as f64) * 1000.0;

    println!("{} nodes {} nps", total_nodes, &*(nps as u64).to_string());

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

fn cmd_setoption(search_state: &mut SearchState, parts: Vec<&str>) -> Either<String, Option<String>> {
    if parts.len() < 3 || parts[1] != "name" {
        Left("usage: setoption name <name> [value <value>]".parse().unwrap())
    } else {
        match parts[2] {
            "Clear" => {
                search_state.hash_table.clear();
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
    search_state.hash_table.clear();
    uci_state.fen = START_POS.parse().unwrap();
    Right(None)
}
