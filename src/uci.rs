use std::process::exit;
use std::sync::mpsc;
use std::thread;
use std::time::Instant;
use either::{Either, Left, Right};
use regex::Regex;
use crate::fen::{algebraic_move_from_move, get_position};
use crate::perft::perft;
use crate::search::{search_zero, start_search};
use crate::types::{SearchState, UciState};

pub fn run_command(mut uci_state: &mut UciState, l: &str) -> Either<String, Option<String>> {
    let parts = l.trim().split(' ').collect::<Vec<&str>>();
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
        "go" => {
            cmd_go(uci_state, parts)
        },
        "debug" => {
            cmd_debug(parts)
        },
        "quit" => {
            exit(0)
        },
        "test" => {
            cmd_msg_test(uci_state)
        }
        "position" => {
            cmd_position(uci_state, parts)
        }
        _ => {
            Left("Unknown command".parse().unwrap())
        }
    }
}

fn cmd_position(uci_state: &mut UciState, parts: Vec<&str>) -> Either<String, Option<String>> {
    let t = parts.get(1).unwrap();
    match *t {
        "fen" => {
            let re = Regex::new(r"\s*^(((?:[rnbqkpRNBQKP1-8]+/){7})[rnbqkpRNBQKP1-8]+)\s([b|w])\s([K|Q|k|q]{1,4}|-)\s(-|[a-h][1-8])\s(\d+\s\d+)$").unwrap();
            uci_state.fen = parts.join(" ").replace("position fen", "").trim().to_string();
            if re.is_match(&*uci_state.fen) {
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

fn cmd_msg_test(mut uci_state: &mut UciState) -> Either<String, Option<String>> {
    let (tx, rx) = mpsc::channel();
    let position = get_position(uci_state.fen.trim());

    thread::spawn(move || {
        search_zero(&position, 0, tx);
    });

    let mut start = Instant::now();

    loop {
        let received = rx.recv().unwrap();
        if start.elapsed().as_secs() >= 1 {
            println!("Got: {}", received);
            if received == "done" {
                break;
            }
            start = Instant::now();
        }
    }
    Right(None)
}

fn cmd_go(mut uci_state: &mut UciState, parts: Vec<&str>) -> Either<String, Option<String>> {
    let t = parts.get(1).unwrap();
    let depth = parts.get(2).unwrap().to_string().parse().unwrap();
    match *t {
        "perft" => {
            cmd_perft(depth, &uci_state);
            Right(None)
        },
        "depth" => {
            let mut search_state = SearchState {
                hash_table: Default::default(),
                pv: vec![],
                pv_score: 0
            };
            let position = get_position(uci_state.fen.trim());
            let mv = start_search(&position, depth, Instant::now(), &mut search_state);
            Right(Some("bestmove ".to_owned() + &algebraic_move_from_move(mv).clone()))
        },
        _ => {
            Left("Unknown go command".parse().unwrap())
        }
    }
}

fn cmd_uci() -> Either<String, Option<String>> {
    Right(Some("id rustival\nuciok".parse().unwrap()))
}

fn cmd_debug(parts: Vec<&str>) -> Either<String, Option<String>> {
    if parts.len() != 2 || !["on", "off"].contains(&parts[1]) {
        return Left::<String, Option<String>> ("usage: debug [ on | off ]".parse().unwrap());
    }
    Right(None)
}

fn cmd_benchmark(parts: Vec<&str>) -> Either<String, Option<String>> {
    let depth: u8 = parts.get(1).unwrap().to_string().parse().unwrap();
    cmd_perft(depth, &UciState {
        debug: false,
        fen: "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".parse().unwrap() }
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