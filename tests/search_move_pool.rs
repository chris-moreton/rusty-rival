//! The pooled search move list (NodeScratch::search_moves): stale pooled
//! contents never reach the search, the buffer comes back, and clones start
//! without it.

use rusty_rival::fen::get_position;
use rusty_rival::search::iterative_deepening;
use rusty_rival::types::{default_search_state, set_stop, MoveList, SearchState};
use std::time::{Duration, Instant};

fn junk() -> Box<MoveList> {
    let mut list = Box::new(MoveList::new());
    for i in 0..256u32 {
        list.push(0x00be_e000 | i);
    }
    list
}

fn fixed_depth_search(fen: &str, depth: u8, dirty: bool) -> (u32, u64, SearchState) {
    let mut position = get_position(fen);
    let mut state = default_search_state();
    state.show_info = false;
    state.end_time = Instant::now() + Duration::from_secs(120);
    if dirty {
        state.node_scratch.search_moves = Some(junk());
    }
    let mv = iterative_deepening(&mut position, depth, &mut state, 1);
    (mv, state.nodes, state)
}

/// A fully dirtied pooled list gives the same move and node count as a fresh
/// one, for roots with castling, en passant, promotions and check evasions,
/// and the buffer is back in its slot afterwards.
#[test]
fn dirty_search_move_pool_does_not_change_a_fixed_depth_search() {
    for fen in [
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
        "rnbqkbnr/ppp1pppp/8/8/3pP3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1",
        "n1n5/PPPk4/8/8/8/8/4Kppp/5N1N b - - 0 1",
        "rnbqk1nr/pppp1ppp/8/4p3/1b2P3/3P4/PPP2PPP/RNBQKBNR w KQkq - 1 3",
    ] {
        let (fresh_move, fresh_nodes, fresh_state) = fixed_depth_search(fen, 7, false);
        let (dirty_move, dirty_nodes, _) = fixed_depth_search(fen, 7, true);
        assert_eq!((dirty_move, dirty_nodes), (fresh_move, fresh_nodes), "{fen}");
        assert!(
            fresh_state.node_scratch.search_moves.is_some(),
            "pooled list not returned for {fen}"
        );
    }
}

#[test]
fn clone_gets_no_search_move_pool() {
    let mut state = default_search_state();
    state.node_scratch.search_moves = Some(junk());
    assert!(state.clone().node_scratch.search_moves.is_none());
}

/// Node-limited searches (stop returns mid-tree) repeated on one state.
#[test]
fn stopped_searches_reuse_the_search_move_pool() {
    let mut position = get_position("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
    let mut state = default_search_state();
    state.show_info = false;
    for limit in [2_000u64, 20_000, 200_000] {
        state.end_time = Instant::now() + Duration::from_secs(60);
        state.nodes = 0;
        state.nodes_limit = limit;
        set_stop(&state.stop, false);
        iterative_deepening(&mut position, 30, &mut state, 1);
    }
    assert!(state.node_scratch.search_moves.is_some());
}
