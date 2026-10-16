// NET-1759 TMNODE arm: node-fraction time management. The soft budget is
// scaled by the share of root-attributed nodes that went to the current best
// move; the hard limit, the NET-339 ceiling and exact (movetime) budgets are
// untouched. These tests pin the factor, the accounting and the clamping.
use rusty_rival::engine_constants::{TM_NODE_BASE, TM_NODE_FACTOR_MAX, TM_NODE_FACTOR_MIN, TM_NODE_SCALE};
use rusty_rival::fen::get_position;
use rusty_rival::search::{best_move_node_fraction, iterative_deepening, node_scaled_soft_limit, node_time_factor, record_root_move_nodes};
use rusty_rival::types::default_search_state;
use std::time::{Duration, Instant};

const START: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

#[test]
fn factor_is_one_without_information_and_decreases_with_concentration() {
    assert!(close(node_time_factor(None), 1.0));
    // No concentration at all: (1.5 - 0) * 1.35 = 2.025, capped at the maximum
    assert!(close(node_time_factor(Some(0.0)), TM_NODE_FACTOR_MAX));
    // Everything on the best move: (1.5 - 1) * 1.35 = 0.675
    assert!(close(node_time_factor(Some(1.0)), (TM_NODE_BASE - 1.0) * TM_NODE_SCALE));
    // Unity close to the step-band centre
    let centre = TM_NODE_BASE - 1.0 / TM_NODE_SCALE;
    assert!(close(node_time_factor(Some(centre)), 1.0));
    let mut previous = f64::INFINITY;
    for step in 0..=20 {
        let f = step as f64 / 20.0;
        let factor = node_time_factor(Some(f));
        assert!(factor <= previous, "factor must not increase with concentration");
        assert!((TM_NODE_FACTOR_MIN..=TM_NODE_FACTOR_MAX).contains(&factor));
        previous = factor;
    }
    // Out-of-range input (impossible in practice) still clamps
    assert!(close(node_time_factor(Some(5.0)), TM_NODE_FACTOR_MIN));
    assert!(close(node_time_factor(Some(-5.0)), TM_NODE_FACTOR_MAX));
}

#[test]
fn accounting_accumulates_per_root_move_and_reports_the_fraction() {
    let mut state = default_search_state();
    assert_eq!(best_move_node_fraction(&state, 0x1234), None, "zero total gives no fraction");
    record_root_move_nodes(&mut state, 0x1234, 300);
    record_root_move_nodes(&mut state, 0x5678, 100);
    record_root_move_nodes(&mut state, 0x1234, 200); // aspiration retry / research of the same move
    assert_eq!(state.root_move_nodes.len(), 2);
    assert!(close(best_move_node_fraction(&state, 0x1234).unwrap(), 500.0 / 600.0));
    assert!(close(best_move_node_fraction(&state, 0x5678).unwrap(), 100.0 / 600.0));
    // A best move with no entry (never searched by this thread) counts as 0
    assert!(close(best_move_node_fraction(&state, 0x9abc).unwrap(), 0.0));
    // Zero-node attributions keep the total at zero
    let mut empty = default_search_state();
    record_root_move_nodes(&mut empty, 0x1234, 0);
    assert_eq!(best_move_node_fraction(&empty, 0x1234), None);
}

#[test]
fn scaled_soft_limit_respects_ceiling_and_hard_limit() {
    let start = Instant::now();
    let soft = start + Duration::from_millis(600);
    let max_soft = start + Duration::from_millis(1200);
    let end = start + Duration::from_millis(2500);
    assert_eq!(node_scaled_soft_limit(start, soft, max_soft, end, 1.0), soft);
    assert_eq!(
        node_scaled_soft_limit(start, soft, max_soft, end, 0.5),
        start + Duration::from_millis(300)
    );
    // 600 * 1.5 = 900 fits under the ceiling
    assert_eq!(
        node_scaled_soft_limit(start, soft, max_soft, end, 1.5),
        start + Duration::from_millis(900)
    );
    // 600 * 2.0 = 1200 equals the ceiling; anything larger is clamped to it
    assert_eq!(node_scaled_soft_limit(start, soft, max_soft, end, 2.0), max_soft);
    assert_eq!(node_scaled_soft_limit(start, soft, max_soft, end, 3.0), max_soft);
    // A ceiling past the hard limit never lets the soft limit exceed the hard limit
    let loose_ceiling = start + Duration::from_millis(5000);
    assert_eq!(node_scaled_soft_limit(start, soft, loose_ceiling, end, 10.0), end);
    // Soft already extended beyond the ceiling is clamped back to it
    let extended = start + Duration::from_millis(1500);
    assert_eq!(node_scaled_soft_limit(start, extended, max_soft, end, 1.0), max_soft);
}

fn fixed_depth(fen: &str, depth: u8) -> (u32, rusty_rival::types::SearchState) {
    let mut position = get_position(fen);
    let mut state = default_search_state();
    state.show_info = false;
    state.end_time = Instant::now() + Duration::from_secs(120);
    let mv = iterative_deepening(&mut position, depth, &mut state, 1);
    (mv, state)
}

#[test]
fn a_search_attributes_nodes_to_every_root_move_and_the_best_move_has_some() {
    let (mv, state) = fixed_depth(START, 6);
    assert_eq!(state.root_move_nodes.len(), 20, "one entry per root move");
    assert_eq!(state.root_moves.len(), state.root_move_nodes.len());
    let total: u64 = state.root_move_nodes.iter().map(|entry| entry.1).sum();
    assert!(
        total > 0 && total <= state.nodes,
        "attributed nodes never exceed the thread's node count"
    );
    let fraction = best_move_node_fraction(&state, mv).expect("nodes were attributed");
    assert!(fraction > 0.0 && fraction <= 1.0);
    for (root, _) in &state.root_moves {
        assert!(
            state.root_move_nodes.iter().any(|entry| entry.0 == *root),
            "root move {root:#x} has no entry"
        );
    }
}

#[test]
fn accounting_is_cleared_between_searches() {
    let mut position = get_position(START);
    let mut state = default_search_state();
    state.show_info = false;
    state.end_time = Instant::now() + Duration::from_secs(120);
    iterative_deepening(&mut position, 5, &mut state, 1);
    assert_eq!(state.root_move_nodes.len(), 20);
    // Fewer legal moves: entries from the first search must not survive
    let mut kings = get_position("4k3/8/8/8/8/8/8/4K3 w - - 0 1");
    iterative_deepening(&mut kings, 5, &mut state, 1);
    assert_eq!(state.root_move_nodes.len(), 5);
    assert_eq!(state.root_moves.len(), 5);
    assert!(state
        .root_move_nodes
        .iter()
        .all(|entry| state.root_moves.iter().any(|(root, _)| *root == entry.0)));
}
