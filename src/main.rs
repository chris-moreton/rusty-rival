use std::io;
use std::io::BufRead;
use rusty_rival::uci::run_command;

fn main() {

    // Everything here is hacked together at the moment

    let stdin = io::stdin();
    let mut fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string();
    println!("Rusty Rival");
    println!("READY");
    for line in stdin.lock().lines() {
        match line {
            Ok(l) => {
                run_command(&mut fen, l.as_str())
            },
            Err(e) => {
                panic!("{}", e)
            }
        }
    }
}


