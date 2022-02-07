use either::{Either, Left, Right};
use rusty_rival::move_constants::START_POS;
use rusty_rival::types::{default_uci_state, HashEntry, HashIndex, UciState};
use rusty_rival::uci::run_command;

#[test]
pub fn it_sets_a_fen() {
    let mut uci_state = default_uci_state();

    assert_eq!(run_command(&mut uci_state, "position fen rnbqkbnr/pppppppp/8/8/PPPPPPPP/8/8/RNBQKBNR w KQkq - 0 1"), Right(None));
    assert_eq!(uci_state.fen.to_string(), "rnbqkbnr/pppppppp/8/8/PPPPPPPP/8/8/RNBQKBNR w KQkq - 0 1".to_string());
    assert_eq!(run_command(&mut uci_state, "go perft 1"), Right(None))
}

#[test]
pub fn it_runs_a_perft_test() {
    let mut uci_state = default_uci_state();

    assert_eq!(run_command(&mut uci_state, "position fen rnbqkbnr/pppppppp/8/8/PPPPPPPP/8/8/RNBQKBNR w KQkq - 0 1"), Right(None));
    assert_eq!(run_command(&mut uci_state, "go perft 2"), Right(None))
}

#[test]
pub fn it_handles_startpos() {
    let mut uci_state = default_uci_state();

    assert_eq!(run_command(&mut uci_state, "position fen rnbqkbnr/pppppppp/8/8/8/8/1PPPPPPP/RNBQKBNR w KQkq - 0 1"), Right(None));
    assert_ne!(uci_state.fen, START_POS);
    assert_eq!(run_command(&mut uci_state, "position startpos"), Right(None));
    assert_eq!(uci_state.fen, START_POS);
}

#[test]
#[ignore]
pub fn it_handles_the_movelist() {
    let mut uci_state = default_uci_state();

    assert_eq!(run_command(&mut uci_state, "position startpos moves e2e4 e7e5 d2d4"), Right(None));
    assert_eq!(uci_state.fen, "rnbqkbnr/pppp1ppp/8/4p3/3PP3/8/PPP2PPP/RNBQKBNR w KQkq - 0 2");
}

#[test]
pub fn it_handles_a_bad_fen() {
    let mut uci_state = default_uci_state();

    let command = "position fen rnbqkbnr/pppppppp/8/8/PPPPPPPP/8/8/RNBQKBNR w KQkq - 0";
    assert_eq!(run_command(&mut uci_state, command), Left("Invalid FEN".to_string()));
}

fn assert_success_message(result: Either<String, Option<String>>, f: fn(&str) -> bool) -> bool {
    match result {
        Left(error) => panic!("Fail"),
        Right(Some(message)) => {
            assert!(f(&*message))
        },
        _ => {
            panic!()
        }

    }
    true
}

fn assert_error_message(result: Either<String, Option<String>>, f: fn(&str) -> bool) -> bool {
    match result {
        Left(error) => assert!(f(&*error)),
        Right(Some(message)) => panic!(),
        _ => {
            panic!("Fail")
        }

    }
    true
}

#[test]
pub fn it_returns_a_best_move() {
    let mut uci_state = default_uci_state();

    assert_eq!(run_command(&mut uci_state, "position fen rnbqkbnr/pppppppp/8/8/PPPPPPPP/8/8/RNBQKBNR w KQkq - 0 1"), Right(None));
    let result = run_command(&mut uci_state, "go depth 3");
    assert_success_message(result, |message| {
        message.contains("bestmove")
    });
}

#[test]
pub fn it_handles_the_uci_command() {
    let mut uci_state = default_uci_state();

    let result = run_command(&mut uci_state, "uci");
    assert_success_message(result, |message| {
        message.starts_with("id rustival") && message.ends_with("uciok") && message.contains("option")
    });
}

#[test]
pub fn it_handles_the_debug_command() {
    let mut uci_state = default_uci_state();

    let result = run_command(&mut uci_state, "debug onn");
    assert_eq!(result, Left("usage: debug [on|off]".to_string()));
    assert_eq!(uci_state.debug, false);

    let result = run_command(&mut uci_state, "debug on");
    assert_eq!(result, Right(None));
    assert_eq!(uci_state.debug, true);

    let result = run_command(&mut uci_state, "debug off");
    assert_eq!(result, Right(None));
    assert_eq!(uci_state.debug, false);

}

#[test]
pub fn it_handles_the_isready_command() {
    let mut uci_state = default_uci_state();

    let result = run_command(&mut uci_state, "isready");
    assert_success_message(result, |message| {
        message == "readyok"
    });
}

#[test]
pub fn it_handles_the_setoption_clear_hash_command() {
    let mut uci_state = default_uci_state();

    let he = HashEntry{
        score: 100,
        path: vec![],
        bound: 0,
        lock: 0
    };

    uci_state.hash_table.insert(0, he);
    match uci_state.hash_table.get(&0) {
        Some(he) => assert_eq!(he.score, 100),
        None => panic!()
    }

    let result = run_command(&mut uci_state, "setoption name Clear Hash");
    assert_eq!(result, Right(None));
    match uci_state.hash_table.get(&0) {
        Some(he) => panic!(),
        None => {}
    }

}

#[test]
pub fn it_handles_a_bad_setoption_name() {
    let mut uci_state = default_uci_state();

    let result = run_command(&mut uci_state, "setoption name asd");
    assert_error_message(result, |message| {
        message == "Unknown option"
    });
}

#[test]
pub fn it_handles_a_bad_setoption_cmd() {
    let mut uci_state = default_uci_state();

    let result = run_command(&mut uci_state, "setoption asd asd");
    assert_error_message(result, |message| {
        message == "usage: setoption name <name> [value <value>]"
    });
}

#[test]
pub fn it_handles_an_unknown_command() {
    let mut uci_state = default_uci_state();

    let result = run_command(&mut uci_state, "blah 123");
    assert_error_message(result, |message| {
        message == "Unknown command"
    });
}

#[test]
pub fn it_handles_the_register_command() {
    let mut uci_state = default_uci_state();

    let result = run_command(&mut uci_state, "register all of this is ignored");
    assert_eq!(result, Right(None))
}

#[test]
pub fn it_handles_the_ucinewgame_command() {
    let mut uci_state = default_uci_state();

    let result = run_command(&mut uci_state, "ucinewgame");
    assert_eq!(result, Right(None))
}
