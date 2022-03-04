use either::{Either, Left, Right};
use rusty_rival::fen::{get_position};
use rusty_rival::move_constants::{START_POS};
use rusty_rival::types::{BoundType, default_search_state, default_uci_state, HashEntry};
use rusty_rival::uci::{extract_go_param, is_legal_move, run_command_test};

#[test]
pub fn it_sets_a_fen() {
    let mut uci_state = default_uci_state();
    let mut search_state = default_search_state();

    assert_eq!(run_command_test(&mut uci_state, &mut search_state, "position fen rnbqkbnr/pppppppp/8/8/PPPPPPPP/8/8/RNBQKBNR w KQkq - 0 1"), Right(None));
    assert_eq!(uci_state.fen.to_string(), "rnbqkbnr/pppppppp/8/8/PPPPPPPP/8/8/RNBQKBNR w KQkq - 0 1".to_string());
    assert_eq!(run_command_test(&mut uci_state, &mut search_state, "go perft 1"), Right(None))
}

#[test]
pub fn it_tests_something() {
    let mut uci_state = default_uci_state();
    let mut search_state = default_search_state();

    assert_eq!(run_command_test(&mut uci_state, &mut search_state, "st03"), Right(None));
    let result = run_command_test(&mut uci_state, &mut search_state, "go depth 9");

    assert_success_message(result, |message| {
        message.contains("bestmove d3e2")
    });
}

#[test]
pub fn it_knows_legal_moves() {
    let position = &get_position("r3k1nr/pppp1ppp/1bn5/4p1q1/3PP3/1BNB1N1b/PPP1QPPP/R4RK1 w kq - 0 1");
    assert!(is_legal_move(position, "a1b1"));
    assert!(is_legal_move(position, "f3g5"));
    assert!(!is_legal_move(position, "a1a2"));
    assert!(!is_legal_move(position, "g2h3"));
    assert!(!is_legal_move(position, "a4a5"));
    assert!(!is_legal_move(position, ""));
    assert!(!is_legal_move(position, "aaaa"));
    assert!(!is_legal_move(position, "!garbage__"));
}

#[test]
pub fn it_runs_a_perft_test() {
    let mut uci_state = default_uci_state();
    let mut search_state = default_search_state();

    assert_eq!(run_command_test(&mut uci_state, &mut search_state, "position fen rnbqkbnr/pppppppp/8/8/PPPPPPPP/8/8/RNBQKBNR w KQkq - 0 1"), Right(None));
    assert_eq!(run_command_test(&mut uci_state, &mut search_state, "go perft 2"), Right(None))
}

#[test]
pub fn it_handles_startpos() {
    let mut uci_state = default_uci_state();
    let mut search_state = default_search_state();

    assert_eq!(run_command_test(&mut uci_state, &mut search_state, "position fen rnbqkbnr/pppppppp/8/8/8/8/1PPPPPPP/RNBQKBNR w KQkq - 0 1"), Right(None));
    assert_ne!(uci_state.fen, START_POS);

    assert_eq!(run_command_test(&mut uci_state, &mut search_state, "position startpos"), Right(None));
    assert_eq!(uci_state.fen, START_POS);
}

#[test]
pub fn it_handles_the_movelist() {
    let mut uci_state = default_uci_state();
    let mut search_state = default_search_state();

    assert_eq!(run_command_test(&mut uci_state, &mut search_state, "position startpos moves e2e4 e7e5 d2d4"), Right(None));
    assert_eq!(uci_state.fen, "rnbqkbnr/pppp1ppp/8/4p3/3PP3/8/PPP2PPP/RNBQKBNR b KQkq d3 0 2");

    assert_eq!(run_command_test(&mut uci_state, &mut search_state, "position startpos moves e2e4 e7e5 g1f3 b8c6 f1b5 g8f6 e1g1 f6e4 f1e1 e4d6 f3e5 f8e7 b5f1 c6e5 e1e5 e8g8 d2d4 e7f6 e5e1 f8e8 c2c3 e8e1 d1e1 d6e8 c1f4 d7d5 b1d2 g7g6 d2f3 e8g7 e1e3 c7c6 a1e1 c8e6 f3e5 d8a5 a2a3 a8e8 e5d3 a5d8 e3d2 e6f5 e1e8 d8e8 d2e3 e8e3 f4e3 g6g5 f1e2 f5g4 e2g4 f6e7 g4c8 g7f5 c8f5 e7f8 f5c8 f8h6 c8b7 g8h8 h2h4 h8g8 h4g5"), Right(None));
    assert_eq!(run_command_test(&mut uci_state, &mut search_state, "position startpos moves e2e4 e7e5 g1f3 b8c6 f1b5 g8f6 d2d3 f8c5 b5c6 d7c6 b1d2 c8e6 e1g1 c5d6 b2b3 e8g8 d2c4 e6c4 b3c4 f6d7 a1b1 b7b6 g2g3 f7f5 e4f5 f8f5 c1e3 d8e8 f3d2 e8g6 d2e4 a8f8 g1g2 d7f6 d1e2 f6e4 d3e4 f5f3 g2g1 g6e4 f1e1 c6c5 e2d3 e4g4 e1e2 h7h5 d3d5 g8h8 b1e1 a7a5 e3d2 f3a3 d2c1 a3a2 e2e5 d6e5 e1e5 g7g6 c1f4 a2a1"), Right(None));

    let result = run_command_test(&mut uci_state, &mut search_state, "isready");
    assert_success_message(result, |message| { message.contains("readyok") });

    let result = run_command_test(&mut uci_state, &mut search_state, "go depth 1");
    assert_success_message(result, |message| { message.contains("bestmove") });

}

#[test]
pub fn it_takes_a_threefold_repetition_from_a_lost_position() {
    let mut uci_state = default_uci_state();
    let mut search_state = default_search_state();

    assert_eq!(run_command_test(&mut uci_state, &mut search_state, "position fen 1n1Nk2r/pp2p2p/3p2p1/1bp5/3b1Pn1/2N5/PPP3PP/R1BQK2R b KQk - 0 1"), Right(None));
    let result = run_command_test(&mut uci_state, &mut search_state, "go depth 7");
    assert_success_message(result, |message| { message.contains("bestmove d4f2") });
}

#[test]
pub fn it_handles_a_bad_fen() {
    let mut uci_state = default_uci_state();
    let mut search_state = default_search_state();

    let command = "position fen rnbqkbnr/pppppppp/8/8/PPPPPPPP/8/8/RNBQKBNR w KQkq - 0";
    assert_eq!(run_command_test(&mut uci_state, &mut search_state, command), Left("Invalid FEN".to_string()));
}

fn assert_success_message(result: Either<String, Option<String>>, f: fn(&str) -> bool) -> bool {
    match result {
        Left(_error) => panic!("Fail"),
        Right(Some(message)) => {
            if !f(&*message) {
                panic!("{}", &*message)
            }
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
        Right(Some(_message)) => panic!(),
        _ => {
            panic!("Fail")
        }

    }
    true
}

#[test]
pub fn it_returns_a_best_move() {
    let mut uci_state = default_uci_state();
    let mut search_state = default_search_state();

    assert_eq!(run_command_test(&mut uci_state, &mut search_state, "position fen rnbqkbnr/pppppppp/8/8/PPPPPPPP/8/8/RNBQKBNR w KQkq - 0 1"), Right(None));
    let result = run_command_test(&mut uci_state, &mut search_state, "go depth 1");
    assert_success_message(result, |message| {
        message.contains("bestmove")
    });

    assert_eq!(run_command_test(&mut uci_state, &mut search_state, "position fen 8/8/8/8/8/2PKQ3/5k2/8 b - - 0 1"), Right(None));
    let result = run_command_test(&mut uci_state, &mut search_state, "go movetime 250");
    assert_success_message(result, |message| {
        println!("{}", message);
        message.contains("bestmove")
    });
}

#[test]
pub fn it_handles_the_uci_command() {
    let mut uci_state = default_uci_state();
    let mut search_state = default_search_state();

    let result = run_command_test(&mut uci_state, &mut search_state, "uci");
    assert_success_message(result, |message| {
        message.starts_with("id name Rusty Rival") && message.ends_with("uciok") && message.contains("option") && message.contains("Chris Moreton")
    });
}

#[test]
pub fn it_handles_the_debug_command() {
    let mut uci_state = default_uci_state();
    let mut search_state = default_search_state();

    let result = run_command_test(&mut uci_state, &mut search_state, "debug onn");
    assert_eq!(result, Left("usage: debug [on|off]".to_string()));
    assert_eq!(uci_state.debug, false);

    let result = run_command_test(&mut uci_state, &mut search_state, "debug on");
    assert_eq!(result, Right(None));
    assert_eq!(uci_state.debug, true);

    let result = run_command_test(&mut uci_state, &mut search_state, "debug off");
    assert_eq!(result, Right(None));
    assert_eq!(uci_state.debug, false);

}

#[test]
pub fn it_handles_the_isready_command() {
    let mut uci_state = default_uci_state();
    let mut search_state = default_search_state();

    let result = run_command_test(&mut uci_state, &mut search_state, "isready");
    assert_success_message(result, |message| {
        message == "readyok"
    });
}

#[test]
pub fn it_handles_the_setoption_clear_hash_command() {
    let mut search_state = default_search_state();
    let mut uci_state = default_uci_state();

    let he = HashEntry{
        score: 100,
        version: 0,
        height: 0,
        mv: 0,
        bound: BoundType::Exact,
        lock: 0
    };

    search_state.hash_table_height[0] = he;
    match search_state.hash_table_height.get(0) {
        Some(he) => assert_eq!(he.score, 100),
        None => panic!()
    }

    let result = run_command_test(&mut uci_state, &mut search_state, "setoption name Clear Hash");
    assert_eq!(result, Right(None));
}

#[test]
pub fn it_handles_a_bad_setoption_name() {
    let mut uci_state = default_uci_state();
    let mut search_state = default_search_state();

    let result = run_command_test(&mut uci_state, &mut search_state, "setoption name asd");
    assert_error_message(result, |message| {
        message == "Unknown option"
    });
}

#[test]
pub fn it_handles_a_bad_setoption_cmd() {
    let mut uci_state = default_uci_state();
    let mut search_state = default_search_state();

    let result = run_command_test(&mut uci_state, &mut search_state, "setoption asd asd");
    assert_error_message(result, |message| {
        message == "usage: setoption name <name> [value <value>]"
    });
}

#[test]
pub fn it_handles_an_unknown_command() {
    let mut uci_state = default_uci_state();
    let mut search_state = default_search_state();

    let result = run_command_test(&mut uci_state, &mut search_state, "blah 123");
    assert_error_message(result, |message| {
        message == "Unknown command"
    });
}

#[test]
pub fn it_handles_the_register_command() {
    let mut uci_state = default_uci_state();
    let mut search_state = default_search_state();

    let result = run_command_test(&mut uci_state, &mut search_state, "register all of this is ignored");
    assert_eq!(result, Right(None))
}

#[test]
pub fn it_handles_the_ucinewgame_command() {
    let mut uci_state = default_uci_state();
    let mut search_state = default_search_state();

    let result = run_command_test(&mut uci_state, &mut search_state, "ucinewgame");
    assert_eq!(result, Right(None))
}

#[test]
pub fn it_parses_params_from_a_go_command() {
    let mut uci_state = default_uci_state();
    let mut search_state = default_search_state();

    run_command_test(&mut uci_state, &mut search_state, "go blah 123 wtime 728 btime 182 depth 1");
    assert_eq!(uci_state.wtime, 728);
    assert_eq!(uci_state.btime, 182)
}

#[test]
pub fn it_extracts_a_u64_param() {
    assert_eq!(456, extract_go_param("cat", "dog 123 cat 456 fox 789", 0))
}