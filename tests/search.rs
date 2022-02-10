use std::ops::Add;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use rusty_rival::fen::{algebraic_move_from_move, get_position};
use rusty_rival::search::start_search;
use rusty_rival::types::{default_search_state, SearchState};

#[test]
fn it_returns_the_best_move_at_depth_1() {
    let mut search_state = default_search_state();
    let (tx, rx) = mpsc::channel();
    let position = get_position(&"n5k1/1P2P1n1/5q1p/P1pP4/5R2/5B2/1r2N2P/R3K1n1 w Q - 0 1".to_string());
    let mv = start_search(&position, 1, Instant::now().add(Duration::from_millis(250)), &mut search_state, tx);
    assert_eq!(algebraic_move_from_move(mv), "b7a8q");

    let (tx, rx) = mpsc::channel();
    let position = get_position(&"6k1/8/7p/P1pP4/4RB2/7P/1r6/4K3 w - - 0 1".to_string());
    let mv = start_search(&position, 1, Instant::now().add(Duration::from_millis(250)), &mut search_state, tx);
    assert_eq!(algebraic_move_from_move(mv), "f4h6");

}

fn assert_move(fen: &str, depth: u8, millis: u64, bestmove: &str) {
    let mut search_state = default_search_state();
    let (tx, rx) = mpsc::channel();
    let position = get_position(&fen.to_string());
    let mv = start_search(&position, depth, Instant::now().add(Duration::from_millis(millis)), &mut search_state, tx);
    assert_eq!(algebraic_move_from_move(mv), bestmove);
}

#[test]
fn it_finds_a_mate_in_3() {
    assert_move("1k5r/pP3ppp/3p2b1/1BN1n3/1Q2P3/P1B5/KP3P1P/7q w - - 1 0", 5, 100000,"c5a6");
    assert_move("3r4/pR2N3/2pkb3/5p2/8/2B5/qP3PPP/4R1K1 w - - 1 0", 5,100000, "c3e5");
    assert_move("R6R/1r3pp1/4p1kp/3pP3/1r2qPP1/7P/1P1Q3K/8 w - - 1 0", 5, 100000, "f4f5");
    assert_move("4r1k1/5bpp/2p5/3pr3/8/1B3pPq/PPR2P2/2R2QK1 b - - 0 1", 5, 100000, "e5e1");
}

#[test]
fn it_finds_a_mate_in_4() {
    assert_move("7R/r1p1q1pp/3k4/1p1n1Q2/3N4/8/1PP2PPP/2B3K1 w - - 1 0", 7, 100000, "h8d8");
}

#[test]
fn it_finds_a_mate_in_5() {
//    assert_move("6k1/3b3r/1p1p4/p1n2p2/1PPNpP1q/P3Q1p1/1R1RB1P1/5K2 b - - 0 1", 9, 1000000, "h4f4");
}

#[test]
fn it_returns_the_best_move_when_time_runs_out() {
    assert_move("rnb1kbnr/pppppppp/8/2q4R/8/8/PPPPPPPP/RNBQKBN1 w Qkq - 0 1", 20, 100, "h5c5");
    assert_move("rnb1kbnr/pppppppp/8/2q4R/8/8/PPPPPPPP/RNBQKBN1 w Qkq - 0 1", 20, 500, "h5c5");
    assert_move("rnb1kbnr/pppppppp/8/2q4R/8/8/PPPPPPPP/RNBQKBN1 w Qkq - 0 1", 20, 1000, "h5c5");
    assert_move("rnb1kbnr/pppppppp/8/2q4R/8/8/PPPPPPPP/RNBQKBN1 w Qkq - 0 1", 20, 5000, "h5c5");
}
