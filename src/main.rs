use std::io;
use std::io::BufRead;
use either::{Left, Right};
use rusty_rival::types::{default_uci_state, UciState};
use rusty_rival::uci::run_command;

fn main() {

    // Everything here is hacked together at the moment

    let stdin = io::stdin();
    let mut uci_state = default_uci_state();

    println!("Rusty Rival");
    println!("READY");
    for line in stdin.lock().lines() {
        match line {
            Ok(l) => {
                let result = run_command(&mut uci_state, l.as_str());
                match result {
                    Right(message) => {
                        match message {
                            Some(m) => {
                                println!("{}", m);
                            },
                            None => { }
                        }
                    },
                    Left(error) => {
                        println!("Error: {}", error);
                    }
                }
            },
            Err(e) => {
                panic!("{}", e)
            }
        }
    }
}


