use std::time::{Instant};
use rusty_rival::fen::{algebraic_move_from_move, get_position};
use rusty_rival::perft::perft;
use std::io::{self, BufRead};
use std::process::exit;
use std::sync::mpsc;
use std::{thread};
use rusty_rival::search::{search_zero, start_search};
use rusty_rival::types::SearchState;

fn main() {

    // Everything here is hacked together at the moment

    let stdin = io::stdin();
    let mut fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string();
    println!("Rusty Rival");
    println!("READY");
    for line in stdin.lock().lines() {
        match line {
            Ok(l) => {
                let parts = l.split(' ').collect::<Vec<&str>>();
                run_command(&mut fen, parts)
            },
            Err(e) => {
                panic!("{}", e)
            }
        }
    }
}

fn run_command(mut fen: &mut String, parts: Vec<&str>) {

    match *parts.get(0).unwrap() {
        "bench" => {
            cmd_benchmark(parts);
        },
        "go" => {
            cmd_go(fen, parts)
        },
        "quit" => {
            exit(0);
        },
        "test" => {
            cmd_msg_test(fen)
        }
        "position" => {
            cmd_position(fen, parts)
        }
        _ => {}
    }
}

fn cmd_position(fen: &mut String, parts: Vec<&str>) {
    let t = parts.get(1).unwrap();
    match *t {
        "fen" => {
            *fen = parts.join(" ").replace("position fen", "").to_string();
            println!("{}", *fen)
        },
        _ => {
            println!("Unknown position command")
        }
    }
}

fn cmd_msg_test(mut fen: &mut String) {
    let (tx, rx) = mpsc::channel();
    let position = get_position(fen.trim());

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
}

fn cmd_go(mut fen: &mut String, parts: Vec<&str>) {
    let t = parts.get(1).unwrap();
    let depth = parts.get(2).unwrap().to_string().parse().unwrap();
    match *t {
        "perft" => {
            cmd_perft(depth, &fen)
        },
        "depth" => {
            let mut search_state = SearchState {
                hash_table: Default::default(),
                pv: vec![],
                pv_score: 0
            };
            let position = get_position(fen.trim());
            let mv = start_search(&position, depth, Instant::now(), &mut search_state);
            println!("bestmove {}", algebraic_move_from_move(mv));
        },
        _ => {
            println!("Unknown go command")
        }
    }
}

fn cmd_benchmark(parts: Vec<&str>) {
    let depth: u8 = parts.get(1).unwrap().to_string().parse().unwrap();
    cmd_perft(depth, &"r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".to_string());
}

fn cmd_perft(depth: u8, fen: &str) {
    let start = Instant::now();
    let nodes = perft(&get_position(fen.trim()), depth - 1);
    let duration = start.elapsed();
    println!("Time elapsed in perft is: {:?}", duration);
    println!("{} nodes {} nps", nodes, (nodes as f64 / (duration.as_millis() as f64)) * 1000.0);
}
