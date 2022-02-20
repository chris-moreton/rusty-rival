use std::ops::Add;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use rusty_rival::fen::{algebraic_move_from_move, get_position};
use rusty_rival::search::iterative_deepening;
use rusty_rival::types::{default_search_state};

#[test]
fn it_returns_the_best_move_at_depth_1() {
    let mut search_state = default_search_state();
    let (tx, _rx) = mpsc::channel();
    let position = get_position(&"n5k1/1P2P1n1/5q1p/P1pP4/5R2/5B2/1r2N2P/R3K1n1 w Q - 0 1".to_string());
    let mv = iterative_deepening(&position, 1, Instant::now().add(Duration::from_millis(250)), &mut search_state, &tx);
    assert_eq!(algebraic_move_from_move(mv), "b7a8q");

    let (tx, _rx) = mpsc::channel();
    let position = get_position(&"6k1/8/7p/P1pP4/4RB2/7P/1r6/4K3 w - - 0 1".to_string());
    let mv = iterative_deepening(&position, 1, Instant::now().add(Duration::from_millis(250)), &mut search_state, &tx);
    assert_eq!(algebraic_move_from_move(mv), "f4h6");

}

fn assert_move(fen: &str, depth: u8, millis: u64, bestmove: &str) {
    let moves: Vec<String> = bestmove.split(',').map(|m| { m.to_string() }).collect();

    let mut search_state = default_search_state();
    let (tx, _rx) = mpsc::channel();
    let position = get_position(&fen.to_string());
    let mv = iterative_deepening(&position, depth, Instant::now().add(Duration::from_millis(millis)), &mut search_state, &tx);
    println!("{}", algebraic_move_from_move(mv));
    assert!(moves.contains(&algebraic_move_from_move(mv)));
}

#[test]
fn it_finds_a_mate_in_3() {
    assert_move("1k5r/pP3ppp/3p2b1/1BN1n3/1Q2P3/P1B5/KP3P1P/7q w - - 1 0", 5, 1000000,"c5a6");
    assert_move("3r4/pR2N3/2pkb3/5p2/8/2B5/qP3PPP/4R1K1 w - - 1 0", 5,1000000, "c3e5");
    assert_move("R6R/1r3pp1/4p1kp/3pP3/1r2qPP1/7P/1P1Q3K/8 w - - 1 0", 5, 1000000, "f4f5");
    assert_move("4r1k1/5bpp/2p5/3pr3/8/1B3pPq/PPR2P2/2R2QK1 b - - 0 1", 5, 1000000, "e5e1");
}

#[test]
fn it_finds_a_mate_in_4() {
    assert_move("7R/r1p1q1pp/3k4/1p1n1Q2/3N4/8/1PP2PPP/2B3K1 w - - 1 0", 7, 1000000, "h8d8");
}

#[test]
fn it_finds_a_mate_in_5() {
    assert_move("6k1/3b3r/1p1p4/p1n2p2/1PPNpP1q/P3Q1p1/1R1RB1P1/5K2 b - - 0 1", 9, 10000000, "h4f4");
}

#[test]
fn it_avoids_thinking_stalemate_is_checkmate() {
    assert_move("8/8/8/8/4Q3/2P4k/8/5K2 w - - 0 1", 3, 10000000, "f1f2");
    assert_move("8/8/8/8/4Q3/2P3k1/4K3/8 w - - 0 1", 5, 10000000, "e2f1");
    assert_move("8/8/8/8/4Q3/2PK3k/8/8 w - - 0 1", 7, 10000000, "d3e3,d3e2");
}

#[test]
fn it_finds_a_mate_in_6() {
    assert_move("8/8/8/1K6/4Q3/2P5/5k2/8 w - - 0 1", 11, 10000000, "b5c5,b5c4,e4g4");
}

#[test]
fn it_returns_the_best_move_when_time_runs_out() {
    assert_move("rnb1kbnr/pppppppp/8/2q4R/8/8/PPPPPPPP/RNBQKBN1 w Qkq - 0 1", 20, 100, "h5c5");
    assert_move("rnb1kbnr/pppppppp/8/2q4R/8/8/PPPPPPPP/RNBQKBN1 w Qkq - 0 1", 20, 500, "h5c5");
    assert_move("rnb1kbnr/pppppppp/8/2q4R/8/8/PPPPPPPP/RNBQKBN1 w Qkq - 0 1", 20, 1000, "h5c5");
    assert_move("rnb1kbnr/pppppppp/8/2q4R/8/8/PPPPPPPP/RNBQKBN1 w Qkq - 0 1", 20, 5000, "h5c5");
}
