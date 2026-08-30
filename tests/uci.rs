use either::{Either, Left, Right};
use rusty_rival::fen::get_position;
use rusty_rival::move_constants::START_POS;
use rusty_rival::search::next_iteration_fits;
use rusty_rival::types::{default_search_state, default_uci_state, BoundType, HashEntry, SearchState, UciState};
use rusty_rival::uci::{extract_go_param, extract_go_param_opt, is_legal_move, run_command_test};
use std::time::{Duration, Instant};

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
    assert_eq!(uci_state.fen, "rnbqkbnr/pppp1ppp/8/4p3/3PP3/8/PPP2PPP/RNBQKBNR b KQkq - 0 2");

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

/// NET-339 regression: the engine must respect the SOFT limit, not merely the hard one.
///
/// Every pre-existing timing assertion in this file checks only that the search
/// returned inside the HARD limit — which it always did, to within 1ms. That is
/// precisely why this shipped: the soft limit is the budget the engine intends to
/// spend, and it was being exceeded by 2-4x on a typical move, because nothing
/// could stop an iteration once it had started. At bullet that is the whole game
/// (31 of 33 losses on v1.0.48 were time forfeits).
///
/// The tolerance is deliberately loose so a loaded CI box cannot flake it, while
/// still sitting far below the hard limit — the failure being guarded against is
/// the engine treating `hard` as its budget.
#[test]
pub fn it_respects_the_soft_time_limit() {
    // 60+0.6 bullet — the control the Lichess bot actually plays.
    //   base = (60000/31) * 0.95 + 600 = 2438ms
    //   soft = base * 0.6  = 1462ms      <- the budget
    //   hard = base * 2.5  = 6095ms      <- the emergency ceiling
    // Pre-fix, this position consumed the full 6095ms.
    const SOFT_MS: u128 = 1462;
    const TOLERANCE: u128 = 2; // 2x soft, i.e. 2924ms — less than half of hard

    let mut uci_state = default_uci_state();
    let mut search_state = default_search_state();

    assert_eq!(
        run_command_test(&mut uci_state, &mut search_state, &format!("position fen {}", START_POS)),
        Right(None)
    );

    let start = Instant::now();
    let result = run_command_test(&mut uci_state, &mut search_state, "go wtime 60000 btime 60000 winc 600 binc 600");
    let millis = (Instant::now() - start).as_millis();

    assert_success_message(result, |message| message.contains("bestmove"));
    assert!(
        millis <= SOFT_MS * TOLERANCE,
        "search spent {}ms against a soft limit of {}ms ({:.1}x). The hard limit is 6095ms, \
         so an assertion against `hard` alone would still pass here — that is the bug this \
         test exists to catch (NET-339).",
        millis,
        SOFT_MS,
        millis as f64 / SOFT_MS as f64
    );
}

/// Unit coverage for the predictive cutoff itself, independent of any search, so
/// the decision logic is pinned deterministically rather than via wall-clock.
#[test]
pub fn it_declines_an_iteration_that_cannot_finish() {
    let growth = 1.8;

    // Last iteration took 100ms; predicted next is 180ms.
    assert!(
        next_iteration_fits(Duration::from_millis(100), Duration::from_millis(200), growth),
        "180ms prediction fits in 200ms remaining"
    );
    assert!(
        !next_iteration_fits(Duration::from_millis(100), Duration::from_millis(150), growth),
        "180ms prediction must NOT be started with only 150ms left — this is the \
         overrun that ran the move to the hard limit"
    );

    // Already past the soft limit: nothing further may be started.
    assert!(!next_iteration_fits(Duration::from_millis(1), Duration::ZERO, growth));

    // A free iteration is always allowed, so shallow depths are never starved
    // and a legal move is always available.
    assert!(next_iteration_fits(Duration::ZERO, Duration::ZERO, growth));
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
pub fn it_reports_raw_eval_with_an_explicit_perspective() {
    let mut uci_state = default_uci_state();
    let mut search_state = default_search_state();

    let white = run_command_test(&mut uci_state, &mut search_state, "eval");
    assert_success_message(white, |message| {
        message.starts_with("info string eval raw cp ") && message.contains(" white_cp ") && message.ends_with(" evaluator nnue")
    });

    assert_eq!(
        run_command_test(
            &mut uci_state,
            &mut search_state,
            "position fen rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1"
        ),
        Right(None)
    );
    let black = run_command_test(&mut uci_state, &mut search_state, "eval");
    assert_success_message(black, |message| {
        let fields = message.split_whitespace().collect::<Vec<_>>();
        let stm = fields[5].parse::<i32>().unwrap();
        let white = fields[7].parse::<i32>().unwrap();
        stm == -white
    });

    assert_eq!(
        run_command_test(&mut uci_state, &mut search_state, "setoption name UseNNUE value false"),
        Right(None)
    );
    let hce = run_command_test(&mut uci_state, &mut search_state, "eval");
    assert_success_message(hce, |message| message.ends_with(" evaluator hce"));
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
    // Negative/garbage/oversized values fall through to the default instead of
    // panicking — and negative must NOT clamp to 0 for value-params like depth,
    // where 0 would mean an instant unsearched move (`go depth -3`).
    assert_eq!(0, extract_go_param("wtime", "go wtime -50 btime -10", 0));
    assert_eq!(3, extract_go_param("wtime", "go wtime -50 btime -10", 3));
    assert_eq!(250, extract_go_param("depth", "go depth -3", 250));
    assert_eq!(250, extract_go_param("depth", "go depth abc", 250));
    assert_eq!(0, extract_go_param("winc", "go winc", 0));
    // Overflow falls back to default rather than panicking or clamping (a
    // 20-digit winc must not become u64::MAX and overflow the allocation)
    assert_eq!(7, extract_go_param("nodes", "go nodes 99999999999999999999999", 7));
    assert_eq!(0, extract_go_param("winc", "go winc 99999999999999999999999", 0));
    // A well-formed value is still parsed
    assert_eq!(1234, extract_go_param("wtime", "go wtime 1234", 0));
}

#[test]
pub fn extract_go_param_opt_distinguishes_absent_from_zero() {
    // NET-362: `wtime 0` / `movetime 0` are present-with-zero; absence is None.
    assert_eq!(Some(0), extract_go_param_opt("wtime", "go wtime 0 btime 5000"));
    assert_eq!(None, extract_go_param_opt("wtime", "go btime 5000"));
    assert_eq!(Some(1234), extract_go_param_opt("movetime", "go movetime 1234 wtime 60000"));
    assert_eq!(None, extract_go_param_opt("movetime", "go wtime 60000"));
    // Out-of-range values are treated as absent (fall through to the caller's
    // default), whatever their sign or size
    assert_eq!(None, extract_go_param_opt("wtime", "go wtime -37 btime 5000"));
    assert_eq!(None, extract_go_param_opt("movetime", "go movetime -500 wtime 60000"));
    assert_eq!(None, extract_go_param_opt("wtime", "go wtime 99999999999999999999999"));
    assert_eq!(
        None,
        extract_go_param_opt("wtime", "go wtime 9999999999999999999999999999999999999999999999")
    );
    assert_eq!(
        None,
        extract_go_param_opt("wtime", "go wtime -9999999999999999999999999999999999999999999999")
    );
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

// ---------------------------------------------------------------------------
// NET-362: time-management defects found by the v1.0.49 correctness audit
// ---------------------------------------------------------------------------

/// A zero (or negative) clock must install a small emergency budget, not fall
/// through to the 10,000,000ms no-clock default. Pre-fix, `go wtime 0 ...`
/// searched for ~2.8 hours: `has_clock` keyed on the parsed VALUE being > 0,
/// so an empty clock looked like "no clock at all" and got the default budget.
/// cutechess-cli sends zero/negative clocks near flag-fall when timemargin > 0,
/// so with increment this forfeited otherwise-winnable games.
#[test]
pub fn it_moves_immediately_when_the_clock_is_empty() {
    // The nodes cap converts a regression from a ~2.8h hang into a fast, loud
    // failure: pre-fix code stops at 20M nodes (~10-20s) instead of end_time.
    for go in [
        "go wtime 0 btime 5000 winc 100 binc 100 nodes 20000000",
        "go wtime -37 btime 5000 winc 100 binc 100 nodes 20000000",
    ] {
        let mut uci_state = default_uci_state();
        let mut search_state = default_search_state();
        assert_eq!(
            run_command_test(&mut uci_state, &mut search_state, &format!("position fen {}", START_POS)),
            Right(None)
        );
        let start = Instant::now();
        let result = run_command_test(&mut uci_state, &mut search_state, go);
        let millis = (Instant::now() - start).as_millis();
        assert_success_message(result, |message| message.contains("bestmove"));
        // Emergency budget is ~50ms; the generous bound keeps CI honest while
        // still being 5 orders of magnitude below the pre-fix 2.8h behaviour.
        assert!(millis <= 1000, "'{}' spent {}ms against an emergency budget of ~50ms", go, millis);
    }
}

/// UCI `movetime` is an exact budget; accompanying clock times must not rescale
/// it. Pre-fix, the clock allocation multiplied the commanded movetime by 0.95,
/// ADDED the full increment, then applied the TM soft (0.6x) / hard (2.5x)
/// factors on top: `movetime 300 winc 5000` gave soft=3165ms / hard=13187ms —
/// up to ~44x the commanded time, and never an exact stop.
#[test]
pub fn it_treats_movetime_as_exact_when_clocks_are_present() {
    let mut uci_state = default_uci_state();
    let mut search_state = default_search_state();
    assert_eq!(
        run_command_test(&mut uci_state, &mut search_state, &format!("position fen {}", START_POS)),
        Right(None)
    );
    let start = Instant::now();
    let result = run_command_test(
        &mut uci_state,
        &mut search_state,
        "go movetime 300 wtime 60000 btime 60000 winc 5000 binc 5000",
    );
    let millis = (Instant::now() - start).as_millis();
    assert_success_message(result, |message| message.contains("bestmove"));
    // Pre-fix, the earliest possible return was ~1130ms (predictive cutoff
    // against the 3165ms soft budget), so the 700ms bound discriminates with
    // a ~430ms margin while staying loose enough for a loaded CI box.
    assert!(
        (150..=700).contains(&millis),
        "movetime 300 with clocks spent {}ms; must be ~300ms exact (pre-fix budget was soft 3165ms / hard 13187ms, earliest pre-fix stop ~1130ms)",
        millis
    );
}

// ---------------------------------------------------------------------------
// NET-369: malformed FEN / command input must never panic or corrupt the board
// ---------------------------------------------------------------------------

/// A board with no king passed the FEN regex, and get_position then indexed
/// ZOBRIST_KEYS_PIECES[..][64] - king_square is trailing_zeros() of an empty
/// bitboard - panicking on the MAIN thread and killing the engine. The
/// catch_unwind in cmd_go only ever guarded the search thread.
#[test]
pub fn kingless_fen_is_rejected_not_fatal() {
    let mut uci_state = default_uci_state();
    let mut search_state = default_search_state();
    for fen in [
        "8/8/8/8/8/8/8/8 w - - 0 1",      // no kings at all
        "4k3/8/8/8/8/8/8/8 w - - 0 1",    // no white king
        "8/8/8/8/8/8/8/4K3 w - - 0 1",    // no black king
        "4k3/8/8/8/8/8/8/3KK3 w - - 0 1", // two white kings
    ] {
        let cmd = format!("position fen {}", fen);
        assert!(
            matches!(run_command_test(&mut uci_state, &mut search_state, &cmd), Left(_)),
            "expected a rejection for {}",
            fen
        );
    }
    // ...and the engine is still alive and usable afterwards
    assert_eq!(
        run_command_test(&mut uci_state, &mut search_state, "position startpos"),
        Right(None)
    );
    match run_command_test(&mut uci_state, &mut search_state, "go movetime 20") {
        Right(Some(s)) => assert!(s.starts_with("bestmove")),
        other => panic!("expected a bestmove, got {:?}", other),
    }
}

/// An en-passant square no legal double push could have produced used to be
/// trusted by make_pawn_capture_move, which applies EN_PASSANT_CAPTURE_MASK -
/// zero outside ranks 3 and 6 - and so wiped the enemy pawn and all-pieces
/// bitboards wholesale. Sanitised at parse time.
#[test]
pub fn impossible_en_passant_square_is_dropped() {
    // "e4" can never be an EP square: EP squares live on rank 3 or 6 only
    let p = get_position("4k3/8/8/3p4/4P3/8/8/4K3 b - e4 0 1");
    assert_eq!(p.en_passant_square, -1, "impossible EP square should be dropped");

    // Right rank, but no white pawn on e4 to be captured
    let p = get_position("4k3/8/8/8/8/8/8/4K3 b - e3 0 1");
    assert_eq!(p.en_passant_square, -1, "unbacked EP square should be dropped");

    // Right rank and side, but the EP square itself is occupied
    let p = get_position("4k3/8/8/8/4P3/4N3/8/4K3 b - e3 0 1");
    assert_eq!(p.en_passant_square, -1, "occupied EP square should be dropped");

    // A genuine en passant is preserved
    let p = get_position("4k3/8/8/8/3pP3/8/8/4K3 b - e3 0 1");
    assert_ne!(p.en_passant_square, -1, "a real EP square must survive");
}

/// generate_castle_moves never checked that the rook exists, and
/// perform_castle XORs one onto f1/d1 while teleporting the king to its
/// castled square from wherever it stands - so castle rights a FEN asserts but
/// the board cannot support let the engine conjure a rook out of nothing.
#[test]
pub fn castling_is_not_generated_without_the_pieces() {
    use rusty_rival::fen::algebraic_move_from_move;
    use rusty_rival::moves::generate_moves;

    let castles = |fen: &str| -> Vec<String> {
        let p = get_position(fen);
        generate_moves(&p)
            .iter()
            .map(|m| algebraic_move_from_move(*m))
            .filter(|m| m == "e1g1" || m == "e1c1" || m == "e8g8" || m == "e8c8")
            .collect()
    };

    // Rights claimed, but no rooks at all
    assert!(castles("4k3/8/8/8/8/8/8/4K3 w KQkq - 0 1").is_empty());
    // Right claimed for a side whose rook is missing; the backed side still works
    assert_eq!(castles("4k3/8/8/8/8/8/8/4K2R w KQ - 0 1"), vec!["e1g1"]);
    // King not on its home square
    assert!(castles("4k3/8/8/8/8/8/8/R2K3R w KQ - 0 1").is_empty());
    // Fully legal position still generates both
    let mut both = castles("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1");
    both.sort();
    assert_eq!(both, vec!["e1c1", "e1g1"]);
}

/// Move counters beyond u16 used to hit parse::<u16>().unwrap().
#[test]
pub fn oversized_move_counters_do_not_panic() {
    let p = get_position("4k3/8/8/8/8/8/8/4K3 w - - 99999999 88888888");
    assert_eq!(p.half_moves, u16::MAX);
    assert_eq!(p.move_number, u16::MAX);
}

/// `bench <non-numeric>` used to be parse().unwrap().
#[test]
pub fn bench_with_a_bad_argument_does_not_panic() {
    let mut uci_state = default_uci_state();
    let mut search_state = default_search_state();
    assert!(matches!(run_command_test(&mut uci_state, &mut search_state, "bench cat"), Left(_)));
    // engine still usable
    assert_eq!(
        run_command_test(&mut uci_state, &mut search_state, "position startpos"),
        Right(None)
    );
}

/// NET-374: `next_power_of_two() >> 1` halves the table whenever the raw entry
/// count is ALREADY a power of two, which with 24-byte entries happens for
/// Hash = 3*2^k MB - including 96, the advertised default. Users asking for
/// 96MB were silently given 48MB.
#[test]
pub fn hash_table_uses_the_memory_it_was_given() {
    use rusty_rival::types::SharedHashTable;

    // 96MB / 24 bytes = 4,194,304 entries = exactly 2^22 - the pathological case
    let t = SharedHashTable::new_with_mb(96);
    assert_eq!(t.len(), 4_194_304, "96MB should give 2^22 entries, not half that");
    assert_eq!(t.size_mb(), 96);

    // Same trap at every 3*2^k
    for mb in [3usize, 6, 12, 24, 48, 192] {
        let t = SharedHashTable::new_with_mb(mb);
        assert_eq!(t.size_mb(), mb, "Hash={}MB should allocate {}MB", mb, mb);
    }

    // Non-power-of-two counts still round DOWN, never up past the request
    for mb in [1usize, 2, 5, 100, 128, 1000] {
        let t = SharedHashTable::new_with_mb(mb);
        assert!(t.len().is_power_of_two(), "Hash={}MB: entry count must stay a power of two", mb);
        assert!(
            t.size_mb() <= mb,
            "Hash={}MB allocated {}MB - must never exceed the request",
            mb,
            t.size_mb()
        );
    }
}
