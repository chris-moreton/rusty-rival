//! NodeScratch ownership and reuse (g1v2). Pool behaviour is checked at the
//! data-structure level, and search exactness by comparing searches run with
//! fresh and with deliberately dirtied pools.
//!
//! Worst-case retained memory per SearchState (thread): (MAX_DEPTH + 1) plies
//! x 256 entries x (4 + 8 + 12 bytes) = 251 x 256 x 24 bytes, about 1.5 MiB if
//! every ply ever reached a full 256-entry list. Buffers are allocated lazily
//! (a ply's Vecs grow only when search first uses that ply), so actual use is
//! the peak reached per ply.

use rusty_rival::fen::get_position;
use rusty_rival::search::iterative_deepening;
use rusty_rival::types::{default_search_state, set_stop, NodeScratch, SearchState};
use std::time::{Duration, Instant};

/// The take / use / move-back pattern search() follows, including a nested
/// same-ply take while the outer frame holds the buffer.
#[test]
fn same_ply_nested_take_cannot_alias_and_outer_buffer_wins() {
    let mut pool = NodeScratch::new();
    let ply = 5;

    let mut outer = std::mem::take(&mut pool.searched_quiets[ply]);
    outer.clear();
    outer.extend([11, 12, 13]);

    // A nested frame at the same ply finds an empty Vec, not the outer's data
    let mut inner = std::mem::take(&mut pool.searched_quiets[ply]);
    assert!(inner.is_empty());
    assert_eq!(inner.capacity(), 0);
    inner.clear();
    inner.push(99);
    pool.searched_quiets[ply] = inner;

    // The outer frame is unaffected and its move-back replaces the inner buffer
    assert_eq!(outer, [11, 12, 13]);
    let outer_capacity = outer.capacity();
    pool.searched_quiets[ply] = outer;
    assert_eq!(pool.searched_quiets[ply], [11, 12, 13]);
    assert_eq!(pool.searched_quiets[ply].capacity(), outer_capacity);

    // The next take clears stale contents and keeps the capacity
    let mut next = std::mem::take(&mut pool.searched_quiets[ply]);
    next.clear();
    assert!(next.is_empty());
    assert_eq!(next.capacity(), outer_capacity);
}

#[test]
fn distinct_plies_are_independent() {
    let mut pool = NodeScratch::new();
    let mut at_3 = std::mem::take(&mut pool.bad_captures[3]);
    let mut at_4 = std::mem::take(&mut pool.bad_captures[4]);
    at_3.push((1, 2, 3));
    at_4.push((4, 5, 6));
    at_4.push((7, 8, 9));
    pool.bad_captures[4] = at_4;
    pool.bad_captures[3] = at_3;
    assert_eq!(pool.bad_captures[3], [(1, 2, 3)]);
    assert_eq!(pool.bad_captures[4], [(4, 5, 6), (7, 8, 9)]);
    for (ply, list) in pool.searched_captures.iter().enumerate() {
        assert!(list.is_empty(), "ply {ply}");
    }
}

fn dirty(state: &mut SearchState) {
    let pool = &mut state.node_scratch;
    for ply in 0..pool.searched_quiets.len() {
        pool.searched_quiets[ply] = vec![0xdead_beef; 256];
        pool.searched_captures[ply] = vec![(0xdead_beef, -7); 256];
        pool.bad_captures[ply] = vec![(0xdead_beef, -7, 9); 256];
    }
}

#[test]
fn clone_gets_a_fresh_pool_and_leaves_the_original_alone() {
    let mut state = default_search_state();
    dirty(&mut state);
    let copy = state.clone();
    let plies = state.node_scratch.searched_quiets.len();
    assert_eq!(copy.node_scratch.searched_quiets.len(), plies);
    for ply in 0..plies {
        assert!(copy.node_scratch.searched_quiets[ply].is_empty() && copy.node_scratch.searched_quiets[ply].capacity() == 0);
        assert!(copy.node_scratch.searched_captures[ply].is_empty() && copy.node_scratch.searched_captures[ply].capacity() == 0);
        assert!(copy.node_scratch.bad_captures[ply].is_empty() && copy.node_scratch.bad_captures[ply].capacity() == 0);
        assert_eq!(state.node_scratch.bad_captures[ply].len(), 256);
    }
}

fn fixed_depth_search(fen: &str, depth: u8, dirty_pools: bool) -> (u32, u64, SearchState) {
    let mut position = get_position(fen);
    let mut state = default_search_state();
    state.show_info = false;
    state.end_time = Instant::now() + Duration::from_secs(120);
    if dirty_pools {
        dirty(&mut state);
    }
    let mv = iterative_deepening(&mut position, depth, &mut state, 1);
    (mv, state.nodes, state)
}

/// Stale pool contents never reach search: dirty and fresh pools give the
/// same best move and node count, and buffers come back with their capacity.
#[test]
fn dirty_pools_do_not_change_a_fixed_depth_search() {
    for fen in [
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
    ] {
        let (fresh_move, fresh_nodes, fresh_state) = fixed_depth_search(fen, 8, false);
        let (dirty_move, dirty_nodes, _) = fixed_depth_search(fen, 8, true);
        assert_eq!((dirty_move, dirty_nodes), (fresh_move, fresh_nodes), "{fen}");
        // Buffers were moved back after use: some ply retains capacity
        let retained = fresh_state.node_scratch.searched_quiets.iter().filter(|v| v.capacity() > 0).count();
        assert!(retained > 0, "no searched_quiets buffer was returned to the pool for {fen}");
    }
}

/// Searches stopped by the node limit (exercising the stop returns) can be
/// repeated on the same state; buffers stay bounded and are reused.
#[test]
fn stopped_searches_reuse_the_pool() {
    let mut position = get_position("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
    let mut state = default_search_state();
    state.show_info = false;
    for limit in [2_000u64, 20_000, 200_000] {
        state.end_time = Instant::now() + Duration::from_secs(60);
        state.nodes = 0;
        state.nodes_limit = limit;
        // iterative_deepening deliberately does not reset a reused stop flag
        set_stop(&state.stop, false);
        iterative_deepening(&mut position, 30, &mut state, 1);
        for ply in 0..state.node_scratch.bad_captures.len() {
            assert!(state.node_scratch.searched_quiets[ply].len() <= 256);
            assert!(state.node_scratch.searched_captures[ply].len() <= 256);
            assert!(state.node_scratch.bad_captures[ply].len() <= 256);
        }
    }
    assert!(state.node_scratch.searched_quiets.iter().any(|v| v.capacity() > 0));
}
