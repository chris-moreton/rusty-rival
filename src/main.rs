use std::time::{Duration, Instant};
use rusty_rival::fen::get_position;
use rusty_rival::perft::perft;
use std::io::{self, BufRead};
use std::process::exit;
use std::sync::mpsc;
use std::{thread, time};
use rusty_rival::search::search;

fn main() {

    // Everything here is hacked together at the moment

    let stdin = io::stdin();
    let mut fen = "".to_string();
    println!("Rusty Rival");
    println!("READY");
    for line in stdin.lock().lines() {
        match line {
            Ok(l) => {
                let parts = l.split(' ').collect::<Vec<&str>>();
                match *parts.get(0).unwrap() {
                    "bench" => {
                        let depth: u8 = parts.get(1).unwrap().to_string().parse().unwrap();
                        cmd_perft(depth, &"r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".to_string());
                    },
                    "go" => {
                        let t = parts.get(1).unwrap();
                        let depth = parts.get(2).unwrap().to_string().parse().unwrap();
                        match *t {
                            "perft" => {
                                cmd_perft(depth, &fen)
                            },
                            _ => {
                                println!("Unknown go command")
                            }
                        }
                    },
                    "quit" => {
                        exit(0);
                    },
                    "test" => {
                        let (tx, rx) = mpsc::channel();

                        thread::spawn(move || {
                            search(tx);
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
                    "position" => {
                        let t = parts.get(1).unwrap();
                        match *t {
                            "fen" => {
                                fen = l.replace("position fen", "").to_string();
                            },
                            _ => {
                                println!("Unknown position command")
                            }
                        }
                    }
                    _ => {}
                }
            },
            Err(e) => {
                panic!("{}", e)
            }
        }

    }
}

fn cmd_perft(depth: u8, fen: &str) {
    let start = Instant::now();
    let nodes = perft(&get_position(fen.trim()), depth - 1);
    let duration = start.elapsed();
    println!("Time elapsed in perft is: {:?}", duration);
    println!("{} nodes {} nps", nodes, (nodes as f64 / (duration.as_millis() as f64)) * 1000.0);
}
