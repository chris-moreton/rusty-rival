// NET-1700: the transposition-table block in search() returns a score, or
// narrows alpha/beta, only at null-window (scout) nodes. At a PV node
// (incoming window wider than one) a deep entry must behave exactly like a
// shallow entry carrying the same move and static eval - an entry that can
// never cut - so the comparisons below include score, full PV and node count.
//
// Each fixture seeds one entry for the root position with a score the real
// search would never produce (FAKE). Search is single-threaded and
// deterministic; the seeded and control runs differ only in the root entry's
// height, and at DEPTH 3 nothing else reads that height (singular extension
// needs depth >= 8). Reverting the `scouting` condition makes every
// `pv_*` test below fail: a deep Exact entry returns FAKE immediately, and a
// deep Lower/Upper entry either cuts or narrows the window (changing nodes).
use rusty_rival::fen::get_position;
use rusty_rival::search::{search, MATE_SCORE, MAX_WINDOW};
use rusty_rival::types::{default_search_state, BoundType, HashEntry, Move, Position, Score, SearchState, STATIC_EVAL_NONE};
use rusty_rival::utils::hydrate_move_from_algebraic_move;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

const FEN: &str = "r1bqkbnr/pppp1ppp/2n5/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3";
const DEPTH: u8 = 3;
const DEEP: u8 = DEPTH + 5;
const SHALLOW: u8 = DEPTH - 1;
const PLY: u8 = 1;
const FAKE: Score = 1234;
const WIDE: (Score, Score) = (-MAX_WINDOW, MAX_WINDOW);
const NARROW_PV: (Score, Score) = (-60, 60);
const WIDTH_TWO: (Score, Score) = (0, 2);
const SCOUT: (Score, Score) = (0, 1);
const PV_WINDOWS: [(Score, Score); 3] = [WIDE, NARROW_PV, WIDTH_TWO];

#[derive(Debug, PartialEq)]
struct Outcome {
    pv: Vec<Move>,
    score: Score,
    nodes: u64,
}

fn fresh() -> (Position, SearchState) {
    let position = get_position(FEN);
    let mut search_state = default_search_state();
    search_state.use_nnue = false;
    search_state.show_info = false;
    search_state.end_time = Instant::now() + Duration::from_secs(60);
    // As if inside iterative deepening at this depth, so the ply-based
    // extension guards (ply < 2 * iterative_depth) behave as in a real search
    search_state.iterative_depth = DEPTH;
    (position, search_state)
}

fn mv(name: &str) -> Move {
    hydrate_move_from_algebraic_move(&get_position(FEN), name.to_string())
}

fn seed(position: &Position, search_state: &SearchState, score: Score, height: u8, bound: BoundType, entry_move: Move) {
    let index = (position.zobrist_lock as u64 & search_state.hash_table.mask()) as usize;
    search_state.hash_table.store(
        index,
        HashEntry {
            score,
            version: search_state.hash_table.version(),
            height,
            mv: entry_move,
            bound,
            lock: position.zobrist_lock,
            static_eval: STATIC_EVAL_NONE,
        },
    );
}

fn run(position: &mut Position, search_state: &mut SearchState, ply: u8, window: (Score, Score), excluded: Move) -> Outcome {
    let result = search(position, DEPTH, ply, window, search_state, false, excluded, None);
    Outcome {
        pv: result.0.to_vec(),
        score: result.1,
        nodes: search_state.nodes,
    }
}

/// Search the root with one seeded entry (`entry = (score, height, bound, move)`)
fn seeded(entry: (Score, u8, BoundType, Move), ply: u8, window: (Score, Score), excluded: Move) -> Outcome {
    let (mut position, mut search_state) = fresh();
    let (score, height, bound, entry_move) = entry;
    seed(&position, &search_state, score, height, bound, entry_move);
    run(&mut position, &mut search_state, ply, window, excluded)
}

/// The same entry made too shallow to cut at any node type: the control a PV
/// node must match
fn control(entry: (Score, u8, BoundType, Move), ply: u8, window: (Score, Score), excluded: Move) -> Outcome {
    seeded((entry.0, SHALLOW, entry.2, entry.3), ply, window, excluded)
}

/// The same search from an empty table
fn reference(ply: u8, window: (Score, Score), excluded: Move) -> Outcome {
    let (mut position, mut search_state) = fresh();
    run(&mut position, &mut search_state, ply, window, excluded)
}

#[test]
fn scout_nodes_still_take_deep_exact_lower_and_upper_cutoffs() {
    assert_eq!(seeded((FAKE, DEPTH, BoundType::Exact, 0), PLY, SCOUT, 0).score, FAKE);
    assert_eq!(seeded((FAKE + 50, DEEP, BoundType::Exact, 0), PLY, SCOUT, 0).score, FAKE + 50);
    // Lower bound at or above beta: alpha is raised past beta, returning the bound
    assert_eq!(seeded((FAKE, DEPTH, BoundType::Lower, 0), PLY, SCOUT, 0).score, FAKE);
    // Upper bound at or below alpha: beta drops below alpha, returning the bound
    assert_eq!(seeded((-FAKE, DEPTH, BoundType::Upper, 0), PLY, SCOUT, 0).score, -FAKE);
}

#[test]
fn scout_cutoff_returns_a_legal_entry_move_as_its_pv() {
    let d2d4 = mv("d2d4");
    let outcome = seeded((FAKE, DEPTH, BoundType::Exact, d2d4), PLY, SCOUT, 0);
    assert_eq!((outcome.pv, outcome.score), (vec![d2d4], FAKE));
}

#[test]
fn pv_nodes_treat_a_deep_exact_entry_like_a_shallow_one() {
    for window in PV_WINDOWS {
        for entry_move in [0, mv("d2d4")] {
            for height in [DEPTH, DEEP] {
                let entry = (FAKE, height, BoundType::Exact, entry_move);
                let outcome = seeded(entry, PLY, window, 0);
                assert_ne!(outcome.score, FAKE, "{window:?}");
                assert_eq!(outcome, control(entry, PLY, window, 0), "{window:?} {entry_move:#x} {height}");
            }
        }
    }
}

#[test]
fn pv_nodes_neither_cut_nor_narrow_on_lower_or_upper_bounds() {
    for window in PV_WINDOWS {
        let searched = reference(PLY, window, 0).score;
        // Outside the window: before NET-1700 these were cutoffs (score FAKE / -FAKE)
        // Inside the window: before NET-1700 these narrowed alpha or beta, which
        // changes the node count even when the final score happens to agree
        for entry in [
            (FAKE, DEEP, BoundType::Lower, 0),
            (-FAKE, DEEP, BoundType::Upper, 0),
            (window.0 + 1, DEEP, BoundType::Lower, 0),
            (window.1 - 1, DEEP, BoundType::Upper, 0),
            (searched - 30, DEEP, BoundType::Lower, 0),
            (searched + 30, DEEP, BoundType::Upper, 0),
        ] {
            assert_eq!(
                seeded(entry, PLY, window, 0),
                control(entry, PLY, window, 0),
                "{window:?} {entry:?}"
            );
        }
    }
}

#[test]
fn width_two_is_a_pv_node_and_width_one_is_not() {
    let entry = (FAKE, DEEP, BoundType::Exact, 0);
    assert_eq!(seeded(entry, PLY, (0, 1), 0).score, FAKE);
    assert_ne!(seeded(entry, PLY, (0, 2), 0).score, FAKE);
}

#[test]
fn pv_node_orders_the_entry_move_first() {
    // Observable through the tree size: a deep entry at a PV node must still
    // supply its move (here a quiet move the ordering would not put first), so
    // the search differs from one whose entry carries no move. Deterministic
    // for this fixture; the score itself may agree.
    let with_move = seeded((FAKE, DEEP, BoundType::Exact, mv("a2a3")), PLY, WIDE, 0);
    let without_move = seeded((FAKE, DEEP, BoundType::Exact, 0), PLY, WIDE, 0);
    assert_ne!(with_move.nodes, without_move.nodes);
}

#[test]
fn shallow_entries_never_cut_at_either_node_type() {
    for window in [SCOUT, WIDE, NARROW_PV] {
        let expected = reference(PLY, window, 0);
        for bound in [BoundType::Exact, BoundType::Lower, BoundType::Upper] {
            let score = if bound == BoundType::Upper { -FAKE } else { FAKE };
            let outcome = seeded((score, SHALLOW, bound, 0), PLY, window, 0);
            assert_eq!(
                (outcome.pv, outcome.score),
                (expected.pv.clone(), expected.score),
                "{window:?} {bound:?}"
            );
        }
    }
}

#[test]
fn singular_verification_never_takes_a_tt_cutoff() {
    let excluded = mv("d2d4");
    for window in [SCOUT, WIDE] {
        for bound in [BoundType::Exact, BoundType::Lower] {
            let entry = (FAKE, DEEP, bound, 0);
            assert_eq!(
                seeded(entry, PLY, window, excluded),
                control(entry, PLY, window, excluded),
                "{window:?} {bound:?}"
            );
        }
    }
}

#[test]
fn scout_mate_scores_are_normalized_to_the_probing_ply() {
    // Stored mate scores are relative to the storing node; a probe at ply p
    // reports them p plies further from the root
    let stored = MATE_SCORE - 10;
    let ply = 4;
    assert_eq!(
        seeded((stored, DEPTH, BoundType::Exact, 0), ply, SCOUT, 0).score,
        stored - ply as Score
    );
    assert_eq!(
        seeded((-stored, DEPTH, BoundType::Exact, 0), ply, SCOUT, 0).score,
        -stored + ply as Score
    );
    // A PV node at the same ply ignores the mate score entirely
    let entry = (stored, DEPTH, BoundType::Exact, 0);
    assert_eq!(seeded(entry, ply, WIDE, 0), control(entry, ply, WIDE, 0));
}

#[test]
fn a_stopped_search_returns_before_the_tt_at_either_node_type() {
    // The stop check precedes the probe, so neither the seeded score nor its
    // move can surface from a stopped search (callers discard the result)
    for window in [SCOUT, WIDE] {
        let (mut position, mut search_state) = fresh();
        seed(&position, &search_state, FAKE, DEEP, BoundType::Exact, mv("d2d4"));
        search_state.stop.store(true, Ordering::SeqCst);
        let outcome = run(&mut position, &mut search_state, PLY, window, 0);
        assert_eq!((outcome.pv, outcome.score), (vec![0], 0), "{window:?}");
    }
}
