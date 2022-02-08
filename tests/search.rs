use std::ops::Add;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use rusty_rival::fen::{algebraic_move_from_move, get_position};
use rusty_rival::search::start_search;
use rusty_rival::types::SearchState;

#[test]
fn it_returns_the_best_move_at_depth_1() {
    let mut search_state = SearchState {
        hash_table: Default::default(),
        pv: vec![],
        pv_score: 0
    };
    let (tx, rx) = mpsc::channel();
    let position = get_position(&"n5k1/1P2P1n1/5q1p/P1pP4/5R2/5B2/1r2N2P/R3K1n1 w Q - 0 1".to_string());
    let mv = start_search(&position, 1, Instant::now().add(Duration::from_millis(250)), &mut search_state, tx);
    assert_eq!(algebraic_move_from_move(mv), "b7a8q");

    let (tx, rx) = mpsc::channel();
    let position = get_position(&"6k1/8/7p/P1pP4/4RB2/7P/1r6/4K3 w - - 0 1".to_string());
    let mv = start_search(&position, 1, Instant::now().add(Duration::from_millis(250)), &mut search_state, tx);
    assert_eq!(algebraic_move_from_move(mv), "f4h6");

}

#[test]
#[ignore]
fn it_finds_a_mate_in_3() {
    let mut search_state = SearchState {
        hash_table: Default::default(),
        pv: vec![],
        pv_score: 0
    };
    let (tx, rx) = mpsc::channel();
    let position = get_position(&"1k5r/pP3ppp/3p2b1/1BN1n3/1Q2P3/P1B5/KP3P1P/7q w - - 1 0".to_string());
    let mv = start_search(&position, 5, Instant::now().add(Duration::from_millis(250)), &mut search_state, tx);
    assert_eq!(algebraic_move_from_move(mv), "c5a6");

}