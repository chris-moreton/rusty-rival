use either::{Left, Right};
use mimalloc::MiMalloc;
use rusty_rival::types::{default_search_state, default_uci_state, SearchHandle, SearchState, UciState};
use rusty_rival::uci::run_command;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() {
    repl().unwrap();
}

fn repl() -> Result<()> {
    let mut uci_state = default_uci_state();
    let mut search_state = default_search_state();
    let mut search_handle: Option<SearchHandle> = None;

    let mut rl = DefaultEditor::new()?;
    loop {
        let readline = rl.readline("");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str()).or(Err(ReadlineError::Eof))?;
                handle_cmd_line(&mut uci_state, &mut search_state, &mut search_handle, line)
            }
            Err(ReadlineError::Interrupted) => break,
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    // EOF or interrupt leaves the REPL by `break`, not through the `quit`
    // command, so nothing had joined the search: dropping a JoinHandle detaches
    // the thread rather than waiting for it. Nothing later could consume the
    // learning here, but the point of taking the master in stop_and_wait is
    // that a join site cannot be forgotten - so this one is not either
    // (NET-372 review).
    if let Some(handle) = search_handle.take() {
        handle.stop_and_wait(&mut search_state);
    }
    Ok(())
}

fn handle_cmd_line(uci_state: &mut UciState, search_state: &mut SearchState, search_handle: &mut Option<SearchHandle>, l: String) {
    let result = run_command(uci_state, search_state, search_handle, l.as_str());
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
