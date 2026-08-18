//! NET-372: thread 0's learned tables must reach the master SearchState.
//!
//! Ownership is proved by POINTER IDENTITY, not by finding a nonzero entry.
//! The search thread clones the master, so it allocates its own boxed tables;
//! the merge swaps that allocation into the master. The master's pointer must
//! therefore change, and it must change for a reason no amount of ordinary
//! learning could fake. A "history is nonzero now" assertion would pass even
//! if the master had merely updated itself, which is exactly the bug.

use rusty_rival::types::{default_search_state, default_uci_state, SearchHandle, SearchState};
use rusty_rival::uci::run_command;

fn history_ptr(s: &SearchState) -> *const u8 {
    s.history_moves.as_ref() as *const _ as *const u8
}

fn correction_ptr(s: &SearchState) -> *const u8 {
    s.correction_history.as_ref() as *const _ as *const u8
}

/// Drive a full go/join cycle. `joiner` is the command that forces the join.
fn go_then(joiner: &str, threads: usize) -> (SearchState, *const u8, *const u8) {
    let mut uci_state = default_uci_state();
    let mut search_state = default_search_state();
    let mut handle: Option<SearchHandle> = None;

    uci_state.threads = threads;
    run_command(&mut uci_state, &mut search_state, &mut handle, "position startpos");
    let before_hist = history_ptr(&search_state);
    let before_corr = correction_ptr(&search_state);

    run_command(&mut uci_state, &mut search_state, &mut handle, "go depth 6");
    // Wait for thread 0 to actually finish before forcing the join. A fixed
    // sleep would race on a loaded machine: the joiner would stop thread 0
    // before it had learned anything, and learning_is_actually_present_after_a
    // _search would fail for a reason unrelated to the write-back. Poll the
    // real handle instead, with a generous ceiling so a hang fails the test
    // rather than hanging the suite.
    if let Some(ref h) = handle {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        while !h.handles[0].is_finished() && std::time::Instant::now() < deadline {
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(h.handles[0].is_finished(), "thread 0 did not finish `go depth 6` within 30s");
    }
    run_command(&mut uci_state, &mut search_state, &mut handle, joiner);

    (search_state, before_hist, before_corr)
}

#[test]
fn a_new_go_absorbs_the_previous_searchs_tables() {
    // The ordinary path: one move's learning reaches the next move.
    let (state, before_hist, before_corr) = go_then("go depth 1", 1);
    assert_ne!(
        history_ptr(&state),
        before_hist,
        "master still owns its original history_moves allocation - thread state was discarded"
    );
    assert_ne!(correction_ptr(&state), before_corr, "correction_history was not absorbed");
}

#[test]
fn stop_absorbs_the_tables() {
    // A stopped search has still learned; discarding here loses a move of history.
    let (state, before_hist, _) = go_then("stop", 1);
    assert_ne!(history_ptr(&state), before_hist, "stop discarded thread 0's learned tables");
}

#[test]
fn absorbing_happens_at_every_thread_count() {
    for threads in [1usize, 2, 8] {
        let (state, before_hist, _) = go_then("stop", threads);
        assert_ne!(
            history_ptr(&state),
            before_hist,
            "no absorb at Threads={threads} - only thread 0 should merge, but it must always merge"
        );
    }
}

#[test]
fn ucinewgame_merges_then_clears() {
    // Merge-then-clear, in that order: the join absorbs, then the reset wipes.
    // A new game must not inherit the previous game's move ordering.
    let (state, before_hist, _) = go_then("ucinewgame", 1);
    assert_ne!(history_ptr(&state), before_hist, "ucinewgame joined without absorbing");

    assert!(
        state.history_moves.iter().flatten().flatten().all(|h| *h == 0),
        "history_moves not cleared by ucinewgame"
    );
    assert!(
        state.correction_history.iter().flatten().all(|c| *c == 0),
        "correction_history not cleared by ucinewgame"
    );
    assert!(
        state.killer_moves.iter().flatten().all(|m| *m == 0),
        "killer_moves not cleared by ucinewgame - a new game would inherit ordering"
    );
    assert!(state.mate_killer.iter().all(|m| *m == 0), "mate_killer not cleared by ucinewgame");
    assert!(
        state.countermoves.iter().flatten().all(|m| *m == 0),
        "countermoves not cleared by ucinewgame - a new game would inherit ordering"
    );
}

#[test]
fn learning_is_actually_present_after_a_search() {
    // Weaker than the pointer tests on ownership, but it is the thing we
    // ultimately care about: the absorbed tables carry real content, not an
    // empty allocation swapped in.
    let (state, _, _) = go_then("stop", 1);
    let nonzero = state.history_moves.iter().flatten().flatten().filter(|h| **h != 0).count();
    assert!(
        nonzero > 0,
        "absorbed history_moves is entirely zero - nothing was learned or the swap moved an empty table"
    );
}

#[test]
fn absorb_moves_learned_tables_and_leaves_the_rest_alone() {
    // Direct unit test of the merge helper, independent of the UCI plumbing.
    let mut master = default_search_state();
    let mut other = default_search_state();

    other.history_moves[0][0][0] = 4242;
    other.killer_moves[0][0] = 99;
    other.countermoves[0][0] = 77;

    // Fields that must NOT move.
    master.nodes = 12345;
    master.cutoffs = 999;
    let master_hash = master.hash_table.clone();

    let other_hist = history_ptr(&other);
    master.absorb_learned_tables(&mut other);

    assert_eq!(master.history_moves[0][0][0], 4242, "learned history did not move");
    assert_eq!(master.killer_moves[0][0], 99, "killers did not move");
    assert_eq!(master.countermoves[0][0], 77, "countermoves did not move");
    assert_eq!(history_ptr(&master), other_hist, "history allocation was copied, not moved");

    assert_eq!(master.nodes, 12345, "counters must not be absorbed");
    assert_eq!(master.cutoffs, 999, "counters must not be absorbed");
    assert!(
        std::sync::Arc::ptr_eq(&master.hash_table, &master_hash),
        "hash table Arc must not be replaced"
    );
}
