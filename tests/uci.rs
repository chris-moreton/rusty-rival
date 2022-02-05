use either::{Either, Left, Right};
use rusty_rival::types::UciState;
use rusty_rival::uci::run_command;

#[test]
pub fn it_sets_a_fen() {
    let mut uci_state = UciState {
        fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string(),
        debug: false,
    };

    assert_eq!(run_command(&mut uci_state, "position fen rnbqkbnr/pppppppp/8/8/PPPPPPPP/8/8/RNBQKBNR w KQkq - 0 1"), Right(None));
    assert_eq!(uci_state.fen.to_string(), "rnbqkbnr/pppppppp/8/8/PPPPPPPP/8/8/RNBQKBNR w KQkq - 0 1".to_string());
    assert_eq!(run_command(&mut uci_state, "go perft 1"), Right(None))
}

#[test]
pub fn it_runs_a_perft_test() {
    let mut uci_state = UciState {
        fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string(),
        debug: false,
    };

    assert_eq!(run_command(&mut uci_state, "position fen rnbqkbnr/pppppppp/8/8/PPPPPPPP/8/8/RNBQKBNR w KQkq - 0 1"), Right(None));
    assert_eq!(run_command(&mut uci_state, "go perft 2"), Right(None))
}

#[test]
pub fn it_handles_a_bad_fen() {
    let mut uci_state = UciState {
        fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string(),
        debug: false,
    };

    let command = "position fen rnbqkbnr/pppppppp/8/8/PPPPPPPP/8/8/RNBQKBNR w KQkq - 0";
    assert_eq!(run_command(&mut uci_state, command), Left("Invalid FEN".to_string()));
}

fn assert_message(result: Either<String, Option<String>>, f: fn(&str) -> bool) -> bool {
    match result {
        Left(error) => panic!("Fail"),
        Right(Some(message)) => {
            assert!(f(&*message))
        },
        _ => {
            panic!("Fail")
        }

    }
    true
}

#[test]
pub fn it_returns_a_best_move() {
    let mut uci_state = UciState {
        fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string(),
        debug: false,
    };

    assert_eq!(run_command(&mut uci_state, "position fen rnbqkbnr/pppppppp/8/8/PPPPPPPP/8/8/RNBQKBNR w KQkq - 0 1"), Right(None));
    let result = run_command(&mut uci_state, "go depth 3");
    assert_message(result, |message| {
        message.contains("bestmove")
    });
}

#[test]
pub fn it_handles_the_uci_command() {
    let mut uci_state = UciState {
        fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string(),
        debug: false,
    };

    let result = run_command(&mut uci_state, "uci");
    assert_message(result, |message| {
        message.contains("id rustival") && message.contains("uciok")
    });
}

#[test]
pub fn it_handles_the_debug_command() {
    let mut uci_state = UciState {
        fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string(),
        debug: false,
    };

    let result = run_command(&mut uci_state, "debug on");
    assert_eq!(result, Right(None));

    let result = run_command(&mut uci_state, "debug onn");
    assert_eq!(result, Left("usage: debug [ on | off ]".to_string()));
}

