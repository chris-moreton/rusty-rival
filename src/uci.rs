use std::process::exit;
use std::sync::mpsc;
use std::thread;
use std::time::Instant;
use either::{Either, Left, Right};
use regex::Regex;

use crate::fen::{algebraic_move_from_move, get_fen, get_position, move_from_algebraic_move};
use crate::make_move::make_move;
use crate::move_constants::{PIECE_MASK_FULL, START_POS};
use crate::moves::{is_check, moves};
use crate::perft::perft;
use crate::search::{start_search};
use crate::types::{Move, Position, SearchState, UciState};
use crate::utils::hydrate_move_from_algebraic_move;

pub fn run_command(mut uci_state: &mut UciState, l: &str) -> Either<String, Option<String>> {
    let mut trimmed_line = l.trim().clone().replace("  ", " ");
    if trimmed_line.starts_with("position startpos") {
        trimmed_line = trimmed_line.replace("startpos", &*("fen ".to_string() + START_POS));
    }
    let parts = trimmed_line.split(' ').collect::<Vec<&str>>();
    run_parts(&mut uci_state, parts)
}

fn run_parts(mut uci_state: &mut UciState, parts: Vec<&str>) -> Either<String, Option<String>> {

    match *parts.get(0).unwrap() {
        "bench" => {
            cmd_benchmark(parts)
        },
        "uci" => {
            cmd_uci()
        },
        "isready" => {
            cmd_isready()
        },
        "go" => {
            cmd_go(uci_state, parts)
        },
        "setoption" => {
            cmd_setoption(uci_state, parts)
        },
        "register" => {
            cmd_register(uci_state, parts)
        },
        "ucinewgame" => {
            cmd_ucinewgame(uci_state, parts)
        },
        "debug" => {
            cmd_debug(uci_state, parts)
        },
        "quit" => {
            exit(0)
        },
        "position" => {
            cmd_position(uci_state, parts)
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

fn cmd_position(uci_state: &mut UciState, parts: Vec<&str>) -> Either<String, Option<String>> {
    let t = parts.get(1).unwrap();
    match *t {
        "fen" => {
            let re = Regex::new(r"\s*^(((?:[rnbqkpRNBQKP1-8]+/){7})[rnbqkpRNBQKP1-8]+)\s([b|w])\s([K|Q|k|q]{1,4}|-)\s(-|[a-h][1-8])\s(\d+\s\d+)$").unwrap();
            let (fen, moves) = fen_and_moves(parts);

            uci_state.fen = fen.parse().unwrap();

            if re.is_match(&*fen) {
                uci_state.fen = fen;
                let mut position = get_position(&uci_state.fen);
                let mut new_position = position;
                if moves.len() > 0 {
                    for m in moves {
                        if !is_legal_move(&new_position, &m) {
                            return Left("Illegal move found".parse::<String>().unwrap() + " " + &*m);
                        }
                        if m == "f8c8" {

                        }
                        make_move(&position, hydrate_move_from_algebraic_move(&position, m.to_string()), &mut new_position);
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

// fn cmd_msg_test(mut uci_state: &mut UciState) -> Either<String, Option<String>> {
//     let (tx, rx) = mpsc::channel();
//     let position = get_position(uci_state.fen.trim());
//
//     thread::spawn(move || {
//         start_search(&position, 0, tx, &mut SearchState {}, ());
//     });
//
//     let mut start = Instant::now();
//
//     loop {
//         let received = rx.recv().unwrap();
//         if start.elapsed().as_secs() >= 1 {
//             println!("Got: {}", received);
//             if received == "done" {
//                 break;
//             }
//             start = Instant::now();
//         }
//     }
//     Right(None)
// }

pub fn extract_go_param(needle: &str, haystack: &str) -> u64 {
    let re = r"".to_string() + &*needle.to_string() + &*" ([0-9]*)".to_string();
    let regex = Regex::new(&*re).unwrap();
    let caps = regex.captures(&*haystack);
    match caps {
        Some(x) => {
            let s = x.get(1).unwrap().as_str();
            s.parse::<u64>().unwrap()
        },
        None => {
            0
        }
    }
}

fn cmd_go(mut uci_state: &mut UciState, parts: Vec<&str>) -> Either<String, Option<String>> {
    let t = parts.get(1).unwrap();

    let mut search_state = SearchState {
        hash_table: Default::default(),
        pv: vec![],
        pv_score: 0
    };
    let (tx, rx) = mpsc::channel();

    match *t {
        "perft" => {
            let depth = parts.get(2).unwrap().to_string().parse().unwrap();
            cmd_perft(depth, &uci_state);
            Right(None)
        },
        "infinite" => {
            let position = get_position(uci_state.fen.trim());
            let mv = start_search(&position, 200, Instant::now(), &mut search_state, tx);
            Right(Some("bestmove ".to_owned() + &algebraic_move_from_move(mv).clone()))
        },
        "mate" => {
            let position = get_position(uci_state.fen.trim());
            let mv = start_search(&position, 200, Instant::now(), &mut search_state, tx);
            Right(Some("bestmove ".to_owned() + &algebraic_move_from_move(mv).clone()))
        },
        _ => {
            let line = parts.join(" ").to_string();
            uci_state.wtime = extract_go_param("wtime", &line);
            uci_state.btime = extract_go_param("btime", &line);
            uci_state.winc = extract_go_param("winc", &line);
            uci_state.binc = extract_go_param("binc", &line);
            uci_state.moves_to_go = extract_go_param("movestogo", &line);
            uci_state.depth = extract_go_param("depth", &line);
            uci_state.nodes = extract_go_param("nodes", &line);
            uci_state.move_time = extract_go_param("movetime", &line);

            let position = get_position(uci_state.fen.trim());
            let mv = start_search(&position, uci_state.depth as u8, Instant::now(), &mut search_state, tx);

            Right(Some("bestmove ".to_owned() + &algebraic_move_from_move(mv).clone()))
        }
    }
}

fn cmd_uci() -> Either<String, Option<String>> {
    Right(Some("id rustival\noption name Clear Hash type button\nuciok".parse().unwrap()))
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

fn cmd_benchmark(parts: Vec<&str>) -> Either<String, Option<String>> {
    let depth: u8 = parts.get(1).unwrap().to_string().parse().unwrap();
    cmd_perft(depth, &UciState {
        debug: false,
        fen: "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".parse().unwrap(),
        hash_table: Default::default(),
        registered_name: "".to_string(),
        wtime: 0,
        btime: 0,
        winc: 0,
        binc: 0,
        moves_to_go: 0,
        depth: 0,
        nodes: 0,
        mate: false,
        move_time: 0,
        infinite: false
    }
    );
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

fn cmd_setoption(mut uci_state: &mut UciState, parts: Vec<&str>) -> Either<String, Option<String>> {
    if parts.len() < 3 || parts[1] != "name" {
        Left("usage: setoption name <name> [value <value>]".parse().unwrap())
    } else {
        match parts[2] {
            "Clear" => {
                uci_state.hash_table.clear();
                Right(None)
            },
            _ => {
                Left("Unknown option".parse().unwrap())
            }
        }
    }
}

fn cmd_register(mut uci_state: &mut UciState, parts: Vec<&str>) -> Either<String, Option<String>> {
    Right(None)
}

fn cmd_ucinewgame(mut uci_state: &mut UciState, parts: Vec<&str>) -> Either<String, Option<String>> {
    Right(None)
}
