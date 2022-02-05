use either::{Left, Right};
use rusty_rival::uci::run_command;

#[test]
pub fn it_sets_a_fen() {
    let mut fen = "".to_string();
    assert_eq!(run_command(&mut fen, "position fen rnbqkbnr/pppppppp/8/8/PPPPPPPP/8/8/RNBQKBNR w KQkq - 0 1"), Right(None));
    assert_eq!(*fen.to_string(), "rnbqkbnr/pppppppp/8/8/PPPPPPPP/8/8/RNBQKBNR w KQkq - 0 1".to_string());
    assert_eq!(run_command(&mut fen, "go perft 1"), Right(None))
}

#[test]
pub fn it_runs_a_perft_test() {
    let mut fen = "".to_string();
    assert_eq!(run_command(&mut fen, "position fen rnbqkbnr/pppppppp/8/8/PPPPPPPP/8/8/RNBQKBNR w KQkq - 0 1"), Right(None));
    assert_eq!(run_command(&mut fen, "go perft 2"), Right(None))
}

#[test]
pub fn it_handles_a_bad_fen() {
    let mut fen = "".to_string();
    let command = "position fen rnbqkbnr/pppppppp/8/8/PPPPPPPP/8/8/RNBQKBNR w KQkq - 0";
    assert_eq!(run_command(&mut fen, command), Left("Invalid FEN".to_string()));
}

#[test]
pub fn it_returns_a_best_move() {
    let mut fen = "".to_string();
    assert_eq!(run_command(&mut fen, "position fen rnbqkbnr/pppppppp/8/8/PPPPPPPP/8/8/RNBQKBNR w KQkq - 0 1"), Right(None));
    let result = run_command(&mut fen, "go depth 3");
    match result {
        Left(error) => panic!("Fail"),
        Right(Some(message)) => {
            assert!(message.contains("bestmove"))
        },
        _ => {
            panic!("Fail")
        }

    }
}