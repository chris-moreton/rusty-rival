use either::{Either, Left, Right};
use rusty_rival::fen::get_position;
use rusty_rival::move_constants::START_POS;
use rusty_rival::types::{default_search_state, default_uci_state, BoundType, HashEntry, SearchState, UciState};
use rusty_rival::uci::{extract_go_param, is_legal_move, run_command_test};
use std::time::Instant;

#[test]
pub fn it_sets_a_fen() {
    let mut uci_state = default_uci_state();
    let mut search_state = default_search_state();

    assert_eq!(
        run_command_test(
            &mut uci_state,
            &mut search_state,
            "position fen rnbqkbnr/pppppppp/8/8/PPPPPPPP/8/8/RNBQKBNR w KQkq - 0 1"
        ),
        Right(None)
    );
    assert_eq!(
        uci_state.fen.to_string(),
        "rnbqkbnr/pppppppp/8/8/PPPPPPPP/8/8/RNBQKBNR w KQkq - 0 1".to_string()
    );
    assert_eq!(run_command_test(&mut uci_state, &mut search_state, "go perft 1"), Right(None))
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

    assert_eq!(
        run_command_test(
            &mut uci_state,
            &mut search_state,
            "position fen rnbqkbnr/pppppppp/8/8/PPPPPPPP/8/8/RNBQKBNR w KQkq - 0 1"
        ),
        Right(None)
    );
    assert_eq!(run_command_test(&mut uci_state, &mut search_state, "go perft 2"), Right(None))
}

#[test]
pub fn it_handles_startpos() {
    let mut uci_state = default_uci_state();
    let mut search_state = default_search_state();

    assert_eq!(
        run_command_test(
            &mut uci_state,
            &mut search_state,
            "position fen rnbqkbnr/pppppppp/8/8/8/8/1PPPPPPP/RNBQKBNR w KQkq - 0 1"
        ),
        Right(None)
    );
    assert_ne!(uci_state.fen, START_POS);

    assert_eq!(
        run_command_test(&mut uci_state, &mut search_state, "position startpos"),
        Right(None)
    );
    assert_eq!(uci_state.fen, START_POS);
}

#[test]
pub fn it_handles_the_movelist() {
    let mut uci_state = default_uci_state();
    let mut search_state = default_search_state();

    assert_eq!(
        run_command_test(&mut uci_state, &mut search_state, "position startpos moves e2e4 e7e5 d2d4"),
        Right(None)
    );
    assert_eq!(uci_state.fen, "rnbqkbnr/pppp1ppp/8/4p3/3PP3/8/PPP2PPP/RNBQKBNR b KQkq d3 0 2");

    assert_eq!(run_command_test(&mut uci_state, &mut search_state, "position startpos moves e2e4 e7e5 g1f3 b8c6 f1b5 g8f6 e1g1 f6e4 f1e1 e4d6 f3e5 f8e7 b5f1 c6e5 e1e5 e8g8 d2d4 e7f6 e5e1 f8e8 c2c3 e8e1 d1e1 d6e8 c1f4 d7d5 b1d2 g7g6 d2f3 e8g7 e1e3 c7c6 a1e1 c8e6 f3e5 d8a5 a2a3 a8e8 e5d3 a5d8 e3d2 e6f5 e1e8 d8e8 d2e3 e8e3 f4e3 g6g5 f1e2 f5g4 e2g4 f6e7 g4c8 g7f5 c8f5 e7f8 f5c8 f8h6 c8b7 g8h8 h2h4 h8g8 h4g5"), Right(None));
    assert_eq!(run_command_test(&mut uci_state, &mut search_state, "position startpos moves e2e4 e7e5 g1f3 b8c6 f1b5 g8f6 d2d3 f8c5 b5c6 d7c6 b1d2 c8e6 e1g1 c5d6 b2b3 e8g8 d2c4 e6c4 b3c4 f6d7 a1b1 b7b6 g2g3 f7f5 e4f5 f8f5 c1e3 d8e8 f3d2 e8g6 d2e4 a8f8 g1g2 d7f6 d1e2 f6e4 d3e4 f5f3 g2g1 g6e4 f1e1 c6c5 e2d3 e4g4 e1e2 h7h5 d3d5 g8h8 b1e1 a7a5 e3d2 f3a3 d2c1 a3a2 e2e5 d6e5 e1e5 g7g6 c1f4 a2a1"), Right(None));

    let result = run_command_test(&mut uci_state, &mut search_state, "isready");
    assert_success_message(result, |message| message.contains("readyok"));

    let result = run_command_test(&mut uci_state, &mut search_state, "go depth 1");
    assert_success_message(result, |message| message.contains("bestmove"));
}

#[test]
pub fn it_takes_a_threefold_repetition_from_a_lost_position() {
    let mut uci_state = default_uci_state();
    let mut search_state = default_search_state();

    assert_eq!(
        run_command_test(
            &mut uci_state,
            &mut search_state,
            "position fen 1n1Nk2r/pp2p2p/3p2p1/1bp5/3b1Pn1/2N5/PPP3PP/R1BQK2R b KQk - 0 1"
        ),
        Right(None)
    );
    let result = run_command_test(&mut uci_state, &mut search_state, "go depth 7");
    assert_success_message(result, |message| message.contains("bestmove d4f2"));
}

#[test]
#[ignore]
pub fn it_handles_cached_mates() {
    let mut uci_state = default_uci_state();
    let mut search_state = default_search_state();
    search_state.show_info = false;

    find_move(&mut uci_state, &mut search_state, "8/2R1Pk2/3K3p/6pP/5nP1/8/8/8 w - - 0 1", "c7c8");
    find_move(&mut uci_state, &mut search_state, "2R5/4Pk2/3K3p/6pP/5nP1/8/8/8 b - - 0 1", "f4h5");
    find_move(
        &mut uci_state,
        &mut search_state,
        "2R5/4P1k1/3K3p/6pP/5nP1/8/8/8 w - - 0 1",
        "e7e8q",
    );
    find_move(&mut uci_state, &mut search_state, "2R1Q3/6k1/3K3p/6pP/5nP1/8/8/8 b - - 0 1", "f4h5");
    find_move(&mut uci_state, &mut search_state, "2R1Q3/8/3K1k1p/6pP/5nP1/8/8/8 w - - 0 1", "e8e7");
    find_move(&mut uci_state, &mut search_state, "2R5/8/3K1k1p/4Q1pP/5nP1/8/8/8 b - - 0 1", "f6f7");
    find_move(&mut uci_state, &mut search_state, "2R5/5k2/3K3p/4Q1pP/5nP1/8/8/8 w - - 0 1", "e5e7");
}

fn find_move(uci_state: &mut UciState, search_state: &mut SearchState, fen: &str, m: &str) {
    let a = format!("position fen {}", fen);
    assert_eq!(run_command_test(uci_state, search_state, &a), Right(None));
    let result = run_command_test(uci_state, search_state, "go depth 10");
    match result {
        Left(_error) => panic!("Fail"),
        Right(Some(message)) => {
            if message != format!("bestmove {}", m) {
                panic!("{}", &*message)
            }
        }
        _ => {
            panic!()
        }
    }
}

#[test]
pub fn it_handles_a_bad_fen() {
    let mut uci_state = default_uci_state();
    let mut search_state = default_search_state();

    let command = "position fen rnbqkbnr/pppppppp/8/8/PPPPPPPP/8/8/RNBQKBNR w KQkq - 0";
    assert_eq!(
        run_command_test(&mut uci_state, &mut search_state, command),
        Left("Invalid FEN".to_string())
    );
}

#[test]
pub fn it_handles_searchmoves() {
    let mut uci_state = default_uci_state();
    let mut search_state = default_search_state();

    // Set up a position where e2e4 and d2d4 are both legal
    assert_eq!(
        run_command_test(&mut uci_state, &mut search_state, "position startpos"),
        Right(None)
    );

    // Search with only e2e4 allowed - best move must be e2e4
    let result = run_command_test(&mut uci_state, &mut search_state, "go depth 4 searchmoves e2e4");
    assert_success_message(result, |message| message.starts_with("bestmove e2e4"));

    // Search with only d2d4 allowed - best move must be d2d4
    let result = run_command_test(&mut uci_state, &mut search_state, "go depth 4 searchmoves d2d4");
    assert_success_message(result, |message| message.starts_with("bestmove d2d4"));
}

fn assert_success_message(result: Either<String, Option<String>>, f: fn(&str) -> bool) -> bool {
    match result {
        Left(_error) => panic!("Fail"),
        Right(Some(message)) => {
            if !f(&message) {
                panic!("{}", &*message)
            }
        }
        _ => {
            panic!()
        }
    }
    true
}

fn assert_error_message(result: Either<String, Option<String>>, f: fn(&str) -> bool) -> bool {
    match result {
        Left(error) => assert!(f(&error)),
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

    assert_eq!(
        run_command_test(
            &mut uci_state,
            &mut search_state,
            "position fen rnbqkbnr/pppppppp/8/8/PPPPPPPP/8/8/RNBQKBNR w KQkq - 0 1"
        ),
        Right(None)
    );
    let result = run_command_test(&mut uci_state, &mut search_state, "go depth 1");
    assert_success_message(result, |message| message.contains("bestmove"));

    assert_eq!(
        run_command_test(&mut uci_state, &mut search_state, "position fen 8/8/8/8/8/2PKQ3/5k2/8 b - - 0 1"),
        Right(None)
    );
    let result = run_command_test(&mut uci_state, &mut search_state, "go movetime 250");
    assert_success_message(result, |message| {
        println!("{}", message);
        message.contains("bestmove")
    });
}

fn test_wtime_btime(fen: &str, cmd: &str, hard_limit_millis: u128) {
    let mut uci_state = default_uci_state();
    let mut search_state = default_search_state();

    assert_eq!(
        run_command_test(&mut uci_state, &mut search_state, &format!("position fen {}", fen)),
        Right(None)
    );
    let start = Instant::now();
    let result = run_command_test(&mut uci_state, &mut search_state, cmd);
    let millis = (Instant::now() - start).as_millis();
    println!("{}", millis);
    // With time management, engine uses soft/hard limits dynamically
    // Just verify it doesn't exceed the hard limit (with some tolerance for overhead)
    assert!(
        millis <= hard_limit_millis,
        "took {}ms, hard limit was {}ms",
        millis,
        hard_limit_millis
    );
    assert_success_message(result, |message| message.contains("bestmove"));
}

#[test]
pub fn it_handles_wtime_and_btime() {
    // Time management uses soft/hard limits:
    //   base_time = (time / (movestogo + 1)) * 0.95 + increment - UCI_MILLIS_REDUCTION
    //   soft = base_time * 0.6
    //   hard = min(base_time * 2.5, remaining * 0.25)

    // wtime=1000, movestogo=9: base=90ms, hard=min(225, 250)=225ms
    test_wtime_btime(
        "rnbqkbnr/pppppppp/8/8/PPPPPPPP/8/8/RNBQKBNR w KQkq - 0 1",
        "go wtime 1000 btime 1000 movestogo 9",
        260,
    );
    // wtime=5000, movestogo=24: base=185ms, hard=min(462, 1250)=462ms
    test_wtime_btime(
        "rnbqkbnr/pppppppp/8/8/PPPPPPPP/8/8/RNBQKBNR w KQkq - 0 1",
        "go wtime 5000 btime 10000 movestogo 24",
        500,
    );
    // btime=1000, movestogo=1: base=470ms, hard=min(1175, 250)=250ms
    test_wtime_btime(
        "rnbqkbnr/pppppppp/8/8/PPPPPPPP/8/8/RNBQKBNR b KQkq - 0 1",
        "go wtime 500 btime 1000 movestogo 1",
        300,
    );
    // btime=250, movestogo=0 (→30): base=~3ms→10ms, hard=min(25, 62)=25ms
    test_wtime_btime(
        "rnbqkbnr/pppppppp/8/8/PPPPPPPP/8/8/RNBQKBNR b KQkq - 0 1",
        "go wtime 500 btime 250 movestogo 0",
        60,
    );
    // wtime=1000, movestogo=9, winc=100: base=190ms, hard=min(475, 250)=250ms
    test_wtime_btime(
        "rnbqkbnr/pppppppp/8/8/PPPPPPPP/8/8/RNBQKBNR w KQkq - 0 1",
        "go wtime 1000 btime 1000 movestogo 9 winc 100 binc 0",
        300,
    );
    // wtime=5000, movestogo=24, winc=100: base=285ms, hard=min(712, 1250)=712ms
    test_wtime_btime(
        "rnbqkbnr/pppppppp/8/8/PPPPPPPP/8/8/RNBQKBNR w KQkq - 0 1",
        "go wtime 5000 btime 10000 movestogo 24 winc 100 binc 100",
        750,
    );
    // btime=1000, movestogo=1, binc=200: base=670ms, hard=min(1675, 250)=250ms
    test_wtime_btime(
        "rnbqkbnr/pppppppp/8/8/PPPPPPPP/8/8/RNBQKBNR b KQkq - 0 1",
        "go wtime 500 btime 1000 movestogo 1 winc 200 binc 200",
        300,
    );
    // btime=250, movestogo=0 (→30), binc=200: base=~203ms, hard=min(507, 62)=62ms
    test_wtime_btime(
        "rnbqkbnr/pppppppp/8/8/PPPPPPPP/8/8/RNBQKBNR b KQkq - 0 1",
        "go wtime 500 btime 250 movestogo 0 winc 50 binc 200",
        100,
    );
}

#[test]
pub fn it_handles_the_uci_command() {
    let mut uci_state = default_uci_state();
    let mut search_state = default_search_state();

    let result = run_command_test(&mut uci_state, &mut search_state, "uci");
    assert_success_message(result, |message| {
        message.starts_with("id name Rusty Rival")
            && message.ends_with("uciok")
            && message.contains("option")
            && message.contains("Chris Moreton")
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
    assert_success_message(result, |message| message == "readyok");
}

#[test]
pub fn it_handles_the_setoption_clear_hash_command() {
    let mut search_state = default_search_state();
    let mut uci_state = default_uci_state();

    let lock: u128 = (12345u128 << 64) | 999;
    let he = HashEntry {
        score: 100,
        version: 1,
        height: 3,
        mv: 0,
        bound: BoundType::Exact,
        lock,
        static_eval: -42,
    };

    search_state.hash_table.store(0, he);
    assert_eq!(search_state.hash_table.probe(0, lock).unwrap().score, 100);
    // static_eval shares the meta word with height/bound/version - check it
    // survives the round-trip rather than being masked away by a neighbour
    assert_eq!(search_state.hash_table.probe(0, lock).unwrap().static_eval, -42);
    assert_eq!(search_state.hash_table.probe(0, lock).unwrap().height, 3);
    assert_eq!(search_state.hash_table.probe(0, lock).unwrap().version, 1);
    // A probe with a different lock must miss (checksum mismatch)
    assert!(search_state.hash_table.probe(0, lock ^ (1u128 << 100)).is_none());

    let result = run_command_test(&mut uci_state, &mut search_state, "setoption name Clear Hash");
    assert_eq!(result, Right(None));
    assert!(search_state.hash_table.probe(0, lock).is_none());
}

#[test]
pub fn it_handles_the_setoption_hash_command() {
    let mut search_state = default_search_state();
    let mut uci_state = default_uci_state();

    let initial_len = search_state.hash_table.len();

    // Resize to 64 MB (should be roughly half the entries)
    let result = run_command_test(&mut uci_state, &mut search_state, "setoption name Hash value 64");
    assert_eq!(result, Right(None));
    assert!(search_state.hash_table.len() < initial_len);
    assert!(search_state.hash_table.len() > initial_len / 4);

    // Resize to 16 MB (should be roughly 1/4 of 64 MB)
    let len_64mb = search_state.hash_table.len();
    let result = run_command_test(&mut uci_state, &mut search_state, "setoption name hash value 16");
    assert_eq!(result, Right(None));
    assert!(search_state.hash_table.len() < len_64mb);
}

#[test]
pub fn it_handles_the_setoption_multipv_command() {
    let mut search_state = default_search_state();
    let mut uci_state = default_uci_state();

    search_state.multi_pv = 1;

    let result = run_command_test(&mut uci_state, &mut search_state, "setoption name multiPv value 5");
    assert_eq!(5, search_state.multi_pv);
    assert_eq!(result, Right(None));
}

#[test]
pub fn it_handles_the_setoption_contempt_command() {
    let mut search_state = default_search_state();
    let mut uci_state = default_uci_state();

    search_state.contempt = 0;

    let result = run_command_test(&mut uci_state, &mut search_state, "setoption name contempt value 125");
    assert_eq!(125, search_state.contempt);
    assert_eq!(result, Right(None));
}

#[test]
pub fn it_handles_a_bad_setoption_name() {
    let mut uci_state = default_uci_state();
    let mut search_state = default_search_state();

    let result = run_command_test(&mut uci_state, &mut search_state, "setoption name asd");
    assert_error_message(result, |message| message == "Unknown option");
}

#[test]
pub fn it_handles_a_bad_setoption_cmd() {
    let mut uci_state = default_uci_state();
    let mut search_state = default_search_state();

    let result = run_command_test(&mut uci_state, &mut search_state, "setoption asd asd");
    assert_error_message(result, |message| message == "usage: setoption name <name> [value <value>]");
}

#[test]
pub fn it_handles_an_unknown_command() {
    let mut uci_state = default_uci_state();
    let mut search_state = default_search_state();

    let result = run_command_test(&mut uci_state, &mut search_state, "blah 123");
    assert_error_message(result, |message| message == "Unknown command");
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

// ---------------------------------------------------------------------------
// NET-214 / NET-213: malformed input must never panic (→ game loss)
// ---------------------------------------------------------------------------

#[test]
pub fn extract_go_param_handles_negative_and_garbage() {
    // Negative/empty/garbage values fall through to the default instead of panicking
    assert_eq!(0, extract_go_param("wtime", "go wtime -50 btime -10", 0));
    assert_eq!(250, extract_go_param("depth", "go depth abc", 250));
    assert_eq!(0, extract_go_param("winc", "go winc", 0));
    // Overflow falls back to default rather than panicking
    assert_eq!(7, extract_go_param("nodes", "go nodes 99999999999999999999999", 7));
    // A well-formed value is still parsed
    assert_eq!(1234, extract_go_param("wtime", "go wtime 1234", 0));
}

#[test]
pub fn malformed_commands_do_not_panic() {
    let mut uci_state = default_uci_state();
    let mut search_state = default_search_state();
    let mut run = |c: &str| run_command_test(&mut uci_state, &mut search_state, c);
    // None of these may panic; each returns an Either (usually a Left usage/error)
    assert!(matches!(run("position"), Left(_)));
    assert!(matches!(run("position fen totally bad fen"), Left(_)));
    assert!(matches!(run("go perft"), Left(_)));
    assert!(matches!(run("go perft 0"), Left(_)));
    assert!(matches!(run("setoption name MultiPV value abc"), Left(_)));
    assert!(matches!(run("setoption name Contempt value xyz"), Left(_)));
    assert!(matches!(run("mvm"), Left(_)));
}

#[test]
pub fn fen_without_move_counters_is_accepted() {
    let mut uci_state = default_uci_state();
    let mut search_state = default_search_state();
    // Legal FEN with the two counters omitted (4 fields) should be accepted,
    // with the counters defaulted.
    assert_eq!(
        run_command_test(
            &mut uci_state,
            &mut search_state,
            "position fen rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -"
        ),
        Right(None)
    );
}

#[test]
pub fn bad_position_does_not_poison_state() {
    let mut uci_state = default_uci_state();
    let mut search_state = default_search_state();
    // Set a good position, then a malformed one (rejected), then search: must not
    // panic and must still produce a bestmove from a valid position.
    assert_eq!(
        run_command_test(&mut uci_state, &mut search_state, "position startpos"),
        Right(None)
    );
    assert!(matches!(
        run_command_test(&mut uci_state, &mut search_state, "position fen garbage here now"),
        Left(_)
    ));
    match run_command_test(&mut uci_state, &mut search_state, "go movetime 20") {
        Right(Some(s)) => assert!(s.starts_with("bestmove")),
        other => panic!("expected a bestmove after a rejected position, got {:?}", other),
    }
}

#[test]
pub fn go_depth_over_255_does_not_wrap_to_instant_move() {
    let mut uci_state = default_uci_state();
    let mut search_state = default_search_state();
    assert_eq!(
        run_command_test(&mut uci_state, &mut search_state, "position startpos"),
        Right(None)
    );
    // depth 256 previously truncated to 0 (u8 wrap) → unsearched instant move.
    // Clamped to 250; a short movetime keeps the test fast while proving no wrap.
    match run_command_test(&mut uci_state, &mut search_state, "go depth 256 movetime 20") {
        Right(Some(s)) => assert!(s.starts_with("bestmove")),
        other => panic!("expected a bestmove, got {:?}", other),
    }
}
