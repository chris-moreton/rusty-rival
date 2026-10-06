// NET-1700: transposition-table score returns and window narrowing happen only
// at null-window (scout) nodes. A PV node (incoming window wider than one)
// always searches, while still using the entry's move, static eval and
// singular metadata. Each test seeds one entry with a score the real search
// would never produce, then compares against an identical search from an empty
// table: a seeded score that leaks into a PV result shows up as a mismatch.
use rusty_rival::fen::get_position;
use rusty_rival::search::{search, MATE_SCORE, MAX_WINDOW};
use rusty_rival::types::{default_search_state, BoundType, HashEntry, Move, Position, Score, SearchState, STATIC_EVAL_NONE};
use rusty_rival::utils::hydrate_move_from_algebraic_move;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

const FEN: &str = "r1bqkbnr/pppp1ppp/2n5/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3";
const DEPTH: u8 = 3;
const PLY: u8 = 1;
const FAKE: Score = 1234;
const WIDE: (Score, Score) = (-MAX_WINDOW, MAX_WINDOW);
const SCOUT: (Score, Score) = (0, 1);

fn fresh() -> (Position, SearchState) {
    let position = get_position(FEN);
    let mut search_state = default_search_state();
    search_state.use_nnue = false;
    search_state.show_info = false;
    search_state.end_time = Instant::now() + Duration::from_secs(60);
    (position, search_state)
}

fn seed(position: &Position, search_state: &SearchState, score: Score, height: u8, bound: BoundType, mv: Move) {
    let index = (position.zobrist_lock as u64 & search_state.hash_table.mask()) as usize;
    search_state.hash_table.store(
        index,
        HashEntry {
            score,
            version: search_state.hash_table.version(),
            height,
            mv,
            bound,
            lock: position.zobrist_lock,
            static_eval: STATIC_EVAL_NONE,
        },
    );
}

fn run(position: &mut Position, search_state: &mut SearchState, ply: u8, window: (Score, Score), excluded: Move) -> (Move, Score) {
    let result = search(position, DEPTH, ply, window, search_state, false, excluded, None);
    (result.0[0], result.1)
}

/// The same search from an empty table
fn reference(ply: u8, window: (Score, Score), excluded: Move) -> (Move, Score) {
    let (mut position, mut search_state) = fresh();
    run(&mut position, &mut search_state, ply, window, excluded)
}

fn seeded(score: Score, height: u8, bound: BoundType, mv: Move, ply: u8, window: (Score, Score), excluded: Move) -> (Move, Score) {
    let (mut position, mut search_state) = fresh();
    seed(&position, &search_state, score, height, bound, mv);
    run(&mut position, &mut search_state, ply, window, excluded)
}

#[test]
fn scout_nodes_still_take_deep_exact_lower_and_upper_cutoffs() {
    assert_eq!(seeded(FAKE, DEPTH, BoundType::Exact, 0, PLY, SCOUT, 0).1, FAKE);
    assert_eq!(seeded(FAKE + 50, DEPTH + 5, BoundType::Exact, 0, PLY, SCOUT, 0).1, FAKE + 50);
    // Lower bound at or above beta: alpha is raised past beta, returning the bound
    assert_eq!(seeded(FAKE, DEPTH, BoundType::Lower, 0, PLY, SCOUT, 0).1, FAKE);
    // Upper bound at or below alpha: beta drops below alpha, returning the bound
    assert_eq!(seeded(-FAKE, DEPTH, BoundType::Upper, 0, PLY, SCOUT, 0).1, -FAKE);
}

#[test]
fn scout_cutoff_returns_a_legal_entry_move_as_its_pv() {
    let position = get_position(FEN);
    let d2d4 = hydrate_move_from_algebraic_move(&position, "d2d4".to_string());
    let (mv, score) = seeded(FAKE, DEPTH, BoundType::Exact, d2d4, PLY, SCOUT, 0);
    assert_eq!((mv, score), (d2d4, FAKE));
}

#[test]
fn pv_nodes_ignore_deep_exact_scores() {
    let expected = reference(PLY, WIDE, 0);
    assert_ne!(expected.1, FAKE);
    assert_eq!(seeded(FAKE, DEPTH, BoundType::Exact, 0, PLY, WIDE, 0), expected);
    assert_eq!(seeded(FAKE, DEPTH + 5, BoundType::Exact, 0, PLY, WIDE, 0), expected);
    // A window wider than one but much narrower than the full range is still a PV node
    let narrow_pv = (-60, 60);
    assert_eq!(
        seeded(FAKE, DEPTH, BoundType::Exact, 0, PLY, narrow_pv, 0),
        reference(PLY, narrow_pv, 0)
    );
}

#[test]
fn pv_nodes_do_not_narrow_the_window_from_lower_or_upper_bounds() {
    let expected = reference(PLY, WIDE, 0);
    // Before NET-1700 these raised alpha to +FAKE / lowered beta to -FAKE and
    // searched (or cut) inside the narrowed window
    assert_eq!(seeded(FAKE, DEPTH, BoundType::Lower, 0, PLY, WIDE, 0), expected);
    assert_eq!(seeded(-FAKE, DEPTH, BoundType::Upper, 0, PLY, WIDE, 0), expected);
    // Bounds inside the window would previously have narrowed it without a cutoff
    assert_eq!(seeded(expected.1 - 30, DEPTH, BoundType::Lower, 0, PLY, WIDE, 0), expected);
    assert_eq!(seeded(expected.1 + 30, DEPTH, BoundType::Upper, 0, PLY, WIDE, 0), expected);
}

#[test]
fn pv_search_keeps_using_the_entry_move() {
    // The entry's move still orders first at a PV node; the result must be a
    // full search of this position, not the entry's score
    let position = get_position(FEN);
    let d2d4 = hydrate_move_from_algebraic_move(&position, "d2d4".to_string());
    let (_, score) = seeded(FAKE, DEPTH, BoundType::Exact, d2d4, PLY, WIDE, 0);
    assert_ne!(score, FAKE);
    assert!(score.abs() < MAX_WINDOW);
}

#[test]
fn shallow_entries_never_cut_at_either_node_type() {
    for window in [SCOUT, WIDE] {
        let expected = reference(PLY, window, 0);
        for bound in [BoundType::Exact, BoundType::Lower, BoundType::Upper] {
            let score = if bound == BoundType::Upper { -FAKE } else { FAKE };
            assert_eq!(
                seeded(score, DEPTH - 1, bound, 0, PLY, window, 0),
                expected,
                "{:?} {:?}",
                window,
                bound
            );
        }
    }
}

#[test]
fn singular_verification_never_takes_a_tt_cutoff() {
    let position = get_position(FEN);
    let excluded = hydrate_move_from_algebraic_move(&position, "d2d4".to_string());
    for window in [SCOUT, WIDE] {
        let expected = reference(PLY, window, excluded);
        assert_eq!(seeded(FAKE, DEPTH + 5, BoundType::Exact, 0, PLY, window, excluded), expected);
        assert_eq!(seeded(FAKE, DEPTH + 5, BoundType::Lower, 0, PLY, window, excluded), expected);
    }
}

#[test]
fn scout_mate_scores_are_normalized_to_the_probing_ply() {
    // Stored mate scores are root-relative to the storing node; a probe at ply
    // p reports them p plies further from the root
    let stored = MATE_SCORE - 10;
    let ply = 4;
    assert_eq!(seeded(stored, DEPTH, BoundType::Exact, 0, ply, SCOUT, 0).1, stored - ply as Score);
    assert_eq!(seeded(-stored, DEPTH, BoundType::Exact, 0, ply, SCOUT, 0).1, -stored + ply as Score);
    // A PV node at the same ply ignores the mate entirely
    assert_eq!(seeded(stored, DEPTH, BoundType::Exact, 0, ply, WIDE, 0), reference(ply, WIDE, 0));
}

#[test]
fn a_stopped_search_returns_before_the_tt_at_either_node_type() {
    // The stop check precedes the probe, so neither the seeded score nor its
    // move can surface from a stopped search (callers discard the result)
    for window in [SCOUT, WIDE] {
        let (mut position, mut search_state) = fresh();
        let d2d4 = hydrate_move_from_algebraic_move(&position, "d2d4".to_string());
        seed(&position, &search_state, FAKE, DEPTH + 5, BoundType::Exact, d2d4);
        search_state.stop.store(true, Ordering::SeqCst);
        assert_eq!(run(&mut position, &mut search_state, PLY, window, 0), (0, 0), "{window:?}");
    }
}
