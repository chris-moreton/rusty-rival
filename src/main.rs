use std::time::Instant;
use rusty_rival::fen::get_position;
use rusty_rival::perft::perft;
use std::io::{self, BufRead};
use std::process::exit;

fn main() {
    let stdin = io::stdin();
    println!("Rusty Rival");
    for line in stdin.lock().lines() {
        match line {
            Ok(l) => {
                let parts = l.split(" ").collect::<Vec<&str>>();
                match parts.get(0).unwrap() {
                    &"bench" => {
                        let depth: u8 = parts.get(1).unwrap().to_string().parse().unwrap();
                        let start = Instant::now();

                        let nodes = perft(&get_position(&"r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".to_string()), depth - 1);
                        let duration = start.elapsed();
                        println!("Time elapsed in perft is: {:?}", duration);
                        println!("{} nodes {} nps", nodes, (nodes as f64 / (duration.as_millis() as f64)) * 1000.0);
                    },
                    &"quit" => {
                        exit(0);
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
