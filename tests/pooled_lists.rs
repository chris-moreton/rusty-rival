//! Pooled boxed ArrayVec lists (h2 quiescence lists): stale pooled contents must never
//! reach search, buffers must come back to the pool, and clones start fresh.

use rusty_rival::fen::get_position;
use rusty_rival::search::iterative_deepening;
use rusty_rival::types::{default_search_state, set_stop, take_pooled, MoveList, MoveScoreArray, SearchState};
use std::time::{Duration, Instant};

#[allow(unused_variables)]
fn dirty(state: &mut SearchState) {
    let pool = &mut state.node_scratch;
    for slot in pool.qsearch_moves.iter_mut() {
        *slot = Some(junk_moves());
    }
    for slot in pool.qsearch_scores.iter_mut() {
        *slot = Some(junk_scores());
    }
}

#[allow(dead_code)]
fn junk_scores() -> Box<MoveScoreArray> {
    let mut list = Box::new(MoveScoreArray::new());
    for i in 0..256u32 {
        list.push((0xdead_0000 | i, -7));
    }
    list
}

#[allow(dead_code)]
fn junk_moves() -> Box<MoveList> {
    let mut list = Box::new(MoveList::new());
    for i in 0..256u32 {
        list.push(0xbeef_0000 | i);
    }
    list
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

/// Fresh and fully dirtied pools give the same best move and node count, and
/// at least one pooled buffer was moved back.
#[test]
fn dirty_pools_do_not_change_a_fixed_depth_search() {
    for fen in [
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
        "rnbqkbnr/ppp1pppp/8/8/3pP3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1",
    ] {
        let (fresh_move, fresh_nodes, fresh_state) = fixed_depth_search(fen, 8, false);
        let (dirty_move, dirty_nodes, _) = fixed_depth_search(fen, 8, true);
        assert_eq!((dirty_move, dirty_nodes), (fresh_move, fresh_nodes), "{fen}");
        let pool = &fresh_state.node_scratch;
        assert!(
            pool.qsearch_moves.iter().any(|slot| slot.is_some()) && pool.qsearch_scores.iter().any(|slot| slot.is_some()),
            "no pooled buffer was returned for {fen}"
        );
    }
}

#[test]
fn clone_gets_a_fresh_pool() {
    let mut state = default_search_state();
    dirty(&mut state);
    let copy = state.clone();
    let pool = &copy.node_scratch;
    assert!(pool.qsearch_moves.iter().all(|slot| slot.is_none()) && pool.qsearch_scores.iter().all(|slot| slot.is_none()));
}

/// Node-limited searches (exercising stop returns) repeated on one state.
#[test]
fn stopped_searches_reuse_the_pool() {
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
    let pool = &state.node_scratch;
    assert!(pool.qsearch_moves.iter().any(|slot| slot.is_some()) && pool.qsearch_scores.iter().any(|slot| slot.is_some()));
}

/// A nested same-ply take while an outer frame holds the buffer: the inner
/// frame gets a different, empty box; the outer contents are untouched; the
/// outer move-back (last) is the buffer the slot keeps, and the next take
/// reuses that same allocation emptied.
#[test]
fn nested_same_ply_take_isolates_and_keeps_the_outer_buffer() {
    let mut state = default_search_state();
    let ply = 7;
    let mut outer = take_pooled(&mut state.node_scratch.qsearch_scores[ply]);
    assert!(outer.is_empty());
    for i in 0..3u32 {
        outer.push((0xabc0_0000 | i, i as i32));
    }
    let outer_address = &*outer as *const MoveScoreArray;
    assert!(state.node_scratch.qsearch_scores[ply].is_none(), "taking leaves the slot empty");

    let mut inner = take_pooled(&mut state.node_scratch.qsearch_scores[ply]);
    assert!(inner.is_empty());
    assert_ne!(
        &*inner as *const MoveScoreArray, outer_address,
        "nested frame must not alias the outer buffer"
    );
    inner.push((0x1234, 5));
    state.node_scratch.qsearch_scores[ply] = Some(inner);

    assert_eq!(outer.len(), 3);
    assert_eq!(outer[2], (0xabc0_0002, 2));
    state.node_scratch.qsearch_scores[ply] = Some(outer);

    let next = take_pooled(&mut state.node_scratch.qsearch_scores[ply]);
    assert!(next.is_empty(), "stale entries are cleared on take");
    assert_eq!(
        &*next as *const MoveScoreArray, outer_address,
        "the outer frame's buffer is the one retained"
    );
    state.node_scratch.qsearch_scores[ply] = Some(next);

    // Other plies are untouched throughout
    for (index, slot) in state.node_scratch.qsearch_scores.iter().enumerate() {
        assert!(index == ply || slot.is_none(), "ply {index}");
    }
}

/// The same isolation for the quiescence move-list pool.
#[test]
fn nested_same_ply_take_isolates_the_move_list_pool() {
    let mut state = default_search_state();
    let ply = 9;
    let mut outer = take_pooled(&mut state.node_scratch.qsearch_moves[ply]);
    outer.extend([11, 12, 13]);
    let outer_address = &*outer as *const MoveList;
    let mut inner = take_pooled(&mut state.node_scratch.qsearch_moves[ply]);
    assert!(inner.is_empty());
    assert_ne!(&*inner as *const MoveList, outer_address);
    inner.push(99);
    state.node_scratch.qsearch_moves[ply] = Some(inner);
    assert_eq!(outer.as_slice(), &[11, 12, 13]);
    state.node_scratch.qsearch_moves[ply] = Some(outer);
    let next = take_pooled(&mut state.node_scratch.qsearch_moves[ply]);
    assert!(next.is_empty());
    assert_eq!(
        &*next as *const MoveList, outer_address,
        "the outer frame's buffer is the one retained"
    );
}
