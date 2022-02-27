use std::{io};
use std::io::BufRead;
use either::{Left, Right};
use rusty_rival::types::{default_search_state, default_uci_state};
use rusty_rival::uci::run_command;

fn main() {
    repl();
}

fn repl() {
    let stdin = io::stdin();
    let mut uci_state = default_uci_state();
    let mut search_state = default_search_state();

    println!("Rusty Rival");
    println!("READY");
    for line in stdin.lock().lines() {
        match line {
            Ok(l) => {
                let result = run_command(&mut uci_state, &mut search_state, l.as_str());
                match result {
                    Right(message) => {
                        match message {
                            Some(m) => {
                                println!("{}", m);
                            },
                            None => {}
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
