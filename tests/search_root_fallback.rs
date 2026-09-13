use rusty_rival::fen::{algebraic_move_from_move, get_position};
use rusty_rival::search::iterative_deepening;
use rusty_rival::types::default_search_state;
use std::time::{Duration, Instant};

// Independent match-PGN position from NET-1291, outside the EPD suites.
// Kb3 is the unique tablebase win. Its root fail-high completes before the
// node limit, while the wider aspiration retry does not.
const FEN: &str = "8/8/PR5p/4k1p1/2K3r1/8/8/8 w - - 0 61";

#[test]
fn completed_root_bound_survives_an_interrupted_retry_without_replacing_exact_state() {
    let mut state = default_search_state();
    state.show_info = false;
    state.nodes_limit = 1_000_000;
    state.end_time = Instant::now() + Duration::from_secs(120);
    let mv = iterative_deepening(&mut get_position(FEN), 100, &mut state, 1);
    assert_eq!(algebraic_move_from_move(mv), "c4b3");
    let selected = state.interrupted_best.as_ref().expect("completed fail-high must survive");
    assert_eq!(selected.0[0], mv);
    assert!(selected.1 > state.current_best.1);
    assert_eq!(algebraic_move_from_move(state.current_best.0[0]), "c4b5");
    // The exact state is a completed iteration; the retained result belongs to
    // the iteration the node limit interrupted. The depth itself is not pinned:
    // like the bench signature, the node budget ties this test to the current
    // tree, and a search change that moves the boundary updates the budget.
    assert!(state.last_completed_depth < state.iterative_depth);
    assert_eq!(state.selected_result().0[0], mv);

    // Clear the retained result even if the next search exits before iteration
    // setup (a terminal root). A reused state must not supply a stale ponder PV.
    let terminal = "7k/6Q1/5K2/8/8/8/8/8 b - - 0 1";
    assert_eq!(iterative_deepening(&mut get_position(terminal), 1, &mut state, 1), 0);
    assert!(state.interrupted_best.is_none());
}

#[test]
fn completed_iteration_supersedes_earlier_fail_highs() {
    let mut state = default_search_state();
    state.show_info = false;
    state.end_time = Instant::now() + Duration::from_secs(120);
    let mv = iterative_deepening(&mut get_position(FEN), 17, &mut state, 1);
    assert_eq!(algebraic_move_from_move(mv), "c4b3");
    assert_eq!(state.current_best.0[0], mv);
    assert_eq!(state.last_completed_depth, 17);
    assert!(state.interrupted_best.is_none());
}
