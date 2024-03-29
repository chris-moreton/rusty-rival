use either::{Left, Right};
use rusty_rival::types::{default_search_state, default_uci_state, SearchState, UciState};
use rusty_rival::uci::run_command;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};

fn main() {
    repl().unwrap();
}

fn repl() -> Result<()> {

    let mut uci_state = default_uci_state();
    let mut search_state = default_search_state();

    // `()` can be used when no completer is required
    let mut rl = DefaultEditor::new()?;
    rl.load_history("history.txt").unwrap();
    loop {
        let readline = rl.readline("");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str()).or(Err(ReadlineError::Eof))?;
                handle_cmd_line(&mut uci_state, &mut search_state, line)
            },
            Err(ReadlineError::Interrupted) => {
                break
            },
            Err(ReadlineError::Eof) => {
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
        rl.save_history("history.txt").expect("TODO: panic message");
    }
    Ok(())
}

fn handle_cmd_line(mut uci_state: &mut UciState, mut search_state: &mut SearchState, l: String) {
    let result = run_command(&mut uci_state, &mut search_state, l.as_str());
    match result {
        Right(message) => {
            if let Some(m) = message {
                println!("{}", m);
            }
        }
        Left(error) => {
            println!("Error: {}", error);
        }
    }
}
