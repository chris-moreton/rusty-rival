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
    let mv = start_search(&position, 0, Instant::now().add(Duration::from_millis(250)), &mut search_state, tx);
    assert_eq!(algebraic_move_from_move(mv), "b7a8q");

}
