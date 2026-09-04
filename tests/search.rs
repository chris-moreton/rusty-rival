use rusty_rival::engine_constants::NULL_MOVE_REDUCE_DEPTH_BASE;
use rusty_rival::fen::{algebraic_move_from_move, get_position};
use rusty_rival::search::{is_draw, is_passed_pawn_push, iterative_deepening, null_move_reduced_depth, piece_index_12};
use rusty_rival::types::default_search_state;
use rusty_rival::utils::{hydrate_move_from_algebraic_move, pawn_push};
use std::ops::Add;
use std::time::{Duration, Instant};

#[test]
fn it_returns_the_best_move_at_depth_1() {
    let mut search_state = default_search_state();
    search_state.use_nnue = false;
    search_state.end_time = Instant::now().add(Duration::from_millis(250));

    let mut position = get_position("n5k1/1P2P1n1/5q1p/P1pP4/5R2/5B2/1r2N2P/R3K1n1 w Q - 0 1");
    let mv = iterative_deepening(&mut position, 1, &mut search_state, 1);
    assert_eq!(algebraic_move_from_move(mv), "b7a8q");

    let mut position = get_position("6k1/8/7p/P1pP4/4RB2/7P/1r6/4K3 w - - 0 1");
    let mv = iterative_deepening(&mut position, 1, &mut search_state, 1);
    assert_eq!(algebraic_move_from_move(mv), "f4h6");
}

fn assert_move(fen: &str, depth: u8, millis: u64, bestmove: &str) {
    let moves: Vec<String> = bestmove.split(',').map(|m| m.to_string()).collect();

    let mut search_state = default_search_state();
    search_state.use_nnue = false;
    search_state.show_info = false;
    let mut position = get_position(fen);
    search_state.end_time = Instant::now().add(Duration::from_millis(millis));
    let mv = iterative_deepening(&mut position, depth, &mut search_state, 1);
    println!("{}", algebraic_move_from_move(mv));
    assert!(moves.contains(&algebraic_move_from_move(mv)));
}

#[test]
fn it_knows_a_pawn_push() {
    let position = get_position("1k5r/pP3ppp/3p2b1/1BN5/1Q2P1n1/P1B5/KP3P1P/7q w - - 0 1");
    assert!(!pawn_push(
        &position,
        hydrate_move_from_algebraic_move(&position, "e4e5".to_string())
    ));

    let position = get_position("1k5r/pP4pp/2p3b1/1BN3p1/1Q2P1n1/P1B5/KP3P1P/7q w - - 0 1");
    assert!(!pawn_push(
        &position,
        hydrate_move_from_algebraic_move(&position, "e4e5".to_string())
    ));

    let position = get_position("1k5r/p5pp/1Pp3b1/1BN3p1/1Q2P1n1/P1B5/KP3P1P/7q w - - 0 1");
    assert!(pawn_push(
        &position,
        hydrate_move_from_algebraic_move(&position, "b6b7".to_string())
    ));

    let position = get_position("1k5r/p5pp/1Pp3b1/1BN5/1Q2P1n1/P1B1p3/KP3P1P/7q b - - 0 1");
    assert!(pawn_push(
        &position,
        hydrate_move_from_algebraic_move(&position, "e3e2".to_string())
    ));

    let position = get_position("1k5r/p5pp/1Pp3b1/1BN1p3/1Q3Pn1/P1B5/KP3P1P/7q b - - 0 1");
    assert!(!pawn_push(
        &position,
        hydrate_move_from_algebraic_move(&position, "e5e4".to_string())
    ));

    let position = get_position("1k5r/p5pp/1Pp3b1/1BN5/1Q2pPn1/P1B5/KP3P1P/7q b - - 0 1");
    assert!(!pawn_push(
        &position,
        hydrate_move_from_algebraic_move(&position, "e4e3".to_string())
    ));

    let position = get_position("1k5r/p5pp/1Pp3b1/1BN1pP2/1Q4n1/P1B3P1/KP5P/7q b - - 0 1");
    assert!(!pawn_push(
        &position,
        hydrate_move_from_algebraic_move(&position, "e5e4".to_string())
    ));

    let position = get_position("1k5r/p5pp/1Pp3b1/1BN5/1Q2pPn1/P1B3P1/KP5P/7q b - - 0 1");
    assert!(pawn_push(
        &position,
        hydrate_move_from_algebraic_move(&position, "e4e3".to_string())
    ));

    let position = get_position("1k5r/p5pp/1Pp3b1/1BN5/1Q2p1n1/P1B2PP1/KP5P/7q b - - 0 1");
    assert!(pawn_push(
        &position,
        hydrate_move_from_algebraic_move(&position, "e4e3".to_string())
    ));

    let position = get_position("1k5r/p5pp/1Pp3b1/1BN5/1Q2p1n1/P1B3P1/KP3P1P/7q b - - 0 1");
    assert!(!pawn_push(
        &position,
        hydrate_move_from_algebraic_move(&position, "e4e3".to_string())
    ));

    let position = get_position("1k5r/pP4pp/2p3b1/1BN1P1p1/1Q4n1/P1B5/KP3P1P/7q w - - 0 1");
    assert!(pawn_push(
        &position,
        hydrate_move_from_algebraic_move(&position, "e5e6".to_string())
    ));
}

#[test]
fn it_finds_a_mate_in_3() {
    assert_move("1k5r/pP3ppp/3p2b1/1BN1n3/1Q2P3/P1B5/KP3P1P/7q w - - 1 0", 7, 1000000, "c5a6");
    assert_move("3r4/pR2N3/2pkb3/5p2/8/2B5/qP3PPP/4R1K1 w - - 1 0", 7, 1000000, "c3e5");
    assert_move("R6R/1r3pp1/4p1kp/3pP3/1r2qPP1/7P/1P1Q3K/8 w - - 1 0", 7, 1000000, "f4f5");
    assert_move("4r1k1/5bpp/2p5/3pr3/8/1B3pPq/PPR2P2/2R2QK1 b - - 0 1", 7, 1000000, "e5e1");
}

/// NET-610 regression. Position 4 of the mate-in-3 suite is the one that
/// razoring hides: Re1! Rxe1 Rxe1 Rc1 Qg2# gives up material before it mates,
/// so the static eval and the verifying quiescence both call the node hopeless.
///
/// It is pinned separately from `it_finds_a_mate_in_3` because the failure mode
/// is specific and the diagnosis was not obvious: with root PVS and
/// RAZOR_MAX_DEPTH at its old value of 3, the engine returned h3f1 scoring 188
/// - it did not pick a slower mate, it never saw a mate at all. A suite of four
/// positions failing as one test does not say that.
///
/// Depth 6 rather than 7 so this fails if mate detection slips a further
/// iteration. Baseline found it at depth 5; root PVS costs one iteration here.
#[test]
fn razoring_does_not_hide_the_mate_after_a_sacrifice() {
    assert_move("4r1k1/5bpp/2p5/3pr3/8/1B3pPq/PPR2P2/2R2QK1 b - - 0 1", 6, 1000000, "e5e1");
}

#[test]
fn it_finds_a_mate_in_4() {
    assert_move("7R/r1p1q1pp/3k4/1p1n1Q2/3N4/8/1PP2PPP/2B3K1 w - - 1 0", 9, 1000000, "h8d8");
}

#[test]
fn it_finds_a_mate_in_5() {
    assert_move("6k1/3b3r/1p1p4/p1n2p2/1PPNpP1q/P3Q1p1/1R1RB1P1/5K2 b - - 0 1", 9, 10000000, "h4f4");
}

#[test]
fn it_avoids_thinking_stalemate_is_checkmate() {
    //assert_move("8/8/8/8/4Q3/2P4k/8/5K2 w - - 0 1", 3, 10000000, "f1f2");
    //assert_move("8/8/8/8/4Q3/2P3k1/4K3/8 w - - 0 1", 5, 10000000, "e2f1");
    // All these moves are winning and avoid stalemate - Qf4/Qf3+ are slightly slower but still win
    assert_move("8/8/8/8/4Q3/2PK3k/8/8 w - - 0 1", 11, 10000000, "d3e3,d3e2,e4g6,c3c4,e4f4,e4f3");
}

#[test]
#[ignore]
fn it_finds_a_mate_in_6() {
    assert_move("8/8/8/1K6/4Q3/2P5/5k2/8 w - - 0 1", 13, 10000000, "b5c5,b5c4,e4g4");
}

#[test]
fn it_returns_the_best_move_when_time_runs_out() {
    assert_move("rnb1kbnr/pppppppp/8/2q4R/8/8/PPPPPPPP/RNBQKBN1 w Qkq - 0 1", 20, 100, "h5c5");
    assert_move("rnb1kbnr/pppppppp/8/2q4R/8/8/PPPPPPPP/RNBQKBN1 w Qkq - 0 1", 20, 500, "h5c5");
    assert_move("rnb1kbnr/pppppppp/8/2q4R/8/8/PPPPPPPP/RNBQKBN1 w Qkq - 0 1", 20, 1000, "h5c5");
    assert_move("rnb1kbnr/pppppppp/8/2q4R/8/8/PPPPPPPP/RNBQKBN1 w Qkq - 0 1", 20, 5000, "h5c5");
}

#[test]
fn it_gets_the_12_piece_index_for_a_move() {
    let position = get_position("1k5r/pP3ppp/3p2b1/1BN5/1Q2P1n1/P1B5/KP3P1P/7q w - - 0 1");
    assert_eq!(
        1,
        piece_index_12(&position, hydrate_move_from_algebraic_move(&position, "c5d7".to_string()))
    );
    assert_eq!(
        0,
        piece_index_12(&position, hydrate_move_from_algebraic_move(&position, "e4e5".to_string()))
    );

    let position = get_position("1k5r/pP3ppp/3p2b1/1BN5/1Q2P1n1/P1B5/KP3P1P/7q b - - 0 1");
    assert_eq!(
        7,
        piece_index_12(&position, hydrate_move_from_algebraic_move(&position, "g4f2".to_string()))
    );
    assert_eq!(
        6,
        piece_index_12(&position, hydrate_move_from_algebraic_move(&position, "d6d5".to_string()))
    );
}

#[test]
fn it_recognises_a_draw() {
    let position = get_position("1k5r/pP3ppp/3p2b1/1BN5/1Q2P1n1/P1B5/KP3P1P/7q w - - 0 60");
    let mut search_state = default_search_state();
    assert!(!is_draw(&position, &mut search_state, 5));
    let position = get_position("1k5r/pP3ppp/3p2b1/1BN5/1Q2P1n1/P1B5/KP3P1P/7q w - - 1 60");
    search_state.history.push(position.zobrist_lock);
    assert!(!is_draw(&position, &mut search_state, 5));
    let position = get_position("1k5r/pP3ppp/3p2b1/1BN5/1Q2P1n1/P1B5/KP3P1P/7q w - - 2 60");
    search_state.history.push(position.zobrist_lock);
    assert!(is_draw(&position, &mut search_state, 5));

    let position = get_position("1k5r/pP3ppp/3p2b1/1BN5/1Q2P1n1/P1B5/KP3P1P/7q w - - 100 60");
    let mut search_state = default_search_state();
    assert!(is_draw(&position, &mut search_state, 5));

    // Checkmate ends the game before a 50-move draw can be claimed, even when
    // the mating move lands exactly on the 100th reversible halfmove.
    let position = get_position("7k/6Q1/5K2/8/8/8/8/8 b - - 100 1");
    let mut search_state = default_search_state();
    assert!(!is_draw(&position, &mut search_state, 5));

    let position = get_position("6k1/8/8/4K3/8/7n/7P/8 b - - 0 1");
    let mut search_state = default_search_state();
    assert!(!is_draw(&position, &mut search_state, 5));

    let position = get_position("6k1/8/8/4K3/8/7n/8/8 b - - 0 1");
    let mut search_state = default_search_state();
    assert!(!is_draw(&position, &mut search_state, 5)); // don't detect at ply 5
    assert!(is_draw(&position, &mut search_state, 7));

    let position = get_position("6k1/8/8/4K3/8/7n/7N/8 b - - 0 1");
    let mut search_state = default_search_state();
    assert!(is_draw(&position, &mut search_state, 7));
}

#[test]
fn it_calculates_the_null_move_reduced_depth() {
    assert_eq!(NULL_MOVE_REDUCE_DEPTH_BASE, 2);
    // At depth 12 the legacy reduction searches depth 7. Each complete 200cp
    // above beta removes one more ply, capped at two.
    assert_eq!(null_move_reduced_depth(12, -400), 7);
    assert_eq!(null_move_reduced_depth(12, 0), 7);
    assert_eq!(null_move_reduced_depth(12, 199), 7);
    assert_eq!(null_move_reduced_depth(12, 200), 6);
    assert_eq!(null_move_reduced_depth(12, 399), 6);
    assert_eq!(null_move_reduced_depth(12, 400), 5);
    assert_eq!(null_move_reduced_depth(12, 2_000), 5);
    // Shallow searches never underflow or enter quiescence directly.
    assert_eq!(null_move_reduced_depth(0, 2_000), 1);
    assert_eq!(null_move_reduced_depth(4, 2_000), 1);
}

#[test]
fn test_is_passed_pawn_push() {
    // Position with white passed pawn on e5 (can push to e6)
    // FEN: 8/8/8/4P3/8/8/4k3/4K3 w - - 0 1
    let position = get_position("8/8/8/4P3/8/8/4k3/4K3 w - - 0 1");
    // e5-e6 should be a passed pawn push (to 6th rank)
    let mv_e5e6 = hydrate_move_from_algebraic_move(&position, "e5e6".to_string());
    assert!(is_passed_pawn_push(&position, mv_e5e6), "e5e6 should be a passed pawn push");

    // Position with black passed pawn on d4 (can push to d3)
    // FEN: 4k3/4K3/8/8/3p4/8/8/8 b - - 0 1
    let position = get_position("4k3/4K3/8/8/3p4/8/8/8 b - - 0 1");
    // d4-d3 should be a passed pawn push (to 3rd rank)
    let mv_d4d3 = hydrate_move_from_algebraic_move(&position, "d4d3".to_string());
    assert!(is_passed_pawn_push(&position, mv_d4d3), "d4d3 should be a passed pawn push");

    // Position with non-passed pawn (enemy pawn ahead on same file)
    // FEN: 8/4p3/8/4P3/8/8/4k3/4K3 w - - 0 1 (enemy pawn on e7, our pawn on e5)
    let position = get_position("8/4p3/8/4P3/8/8/4k3/4K3 w - - 0 1");
    let mv_e5e6 = hydrate_move_from_algebraic_move(&position, "e5e6".to_string());
    assert!(
        !is_passed_pawn_push(&position, mv_e5e6),
        "e5e6 should NOT be a passed pawn push (blocked by e7)"
    );

    // Position with passed pawn push to 5th rank (now extends)
    // FEN: 8/8/8/8/4P3/8/4k3/4K3 w - - 0 1 (pawn on e4, pushing to e5)
    let position = get_position("8/8/8/8/4P3/8/4k3/4K3 w - - 0 1");
    let mv_e4e5 = hydrate_move_from_algebraic_move(&position, "e4e5".to_string());
    // e4e5 goes to 5th rank - now triggers extension
    assert!(
        is_passed_pawn_push(&position, mv_e4e5),
        "e4e5 SHOULD trigger extension (5th rank passed pawn)"
    );

    // Position with pawn push to 4th rank (not extending - too early)
    // FEN: 8/8/8/8/8/4P3/4k3/4K3 w - - 0 1 (pawn on e3, pushing to e4)
    let position = get_position("8/8/8/8/8/4P3/4k3/4K3 w - - 0 1");
    let mv_e3e4 = hydrate_move_from_algebraic_move(&position, "e3e4".to_string());
    // e3e4 goes to 4th rank - too early for extension
    assert!(
        !is_passed_pawn_push(&position, mv_e3e4),
        "e3e4 should NOT trigger extension (only 4th rank)"
    );
}

/// NET-367: every recursion that descends into a child must push that child's
/// zobrist lock and pop it again. Null-move, probcut and multicut children
/// previously skipped the push entirely; adding it introduces the opposite
/// risk, that some early-return path pops fewer times than it pushed and the
/// history stack grows without bound, silently corrupting draw detection for
/// the rest of the game. Assert the stack is exactly restored.
#[test]
fn search_leaves_the_repetition_history_balanced() {
    use rusty_rival::fen::get_position;
    use rusty_rival::search::iterative_deepening;
    use rusty_rival::types::default_search_state;
    use std::time::{Duration, Instant};

    // Positions chosen to exercise null move, probcut and multicut: deep
    // enough to pass their min-depth gates, with plenty of captures.
    for fen in [
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        "rnbq1rk1/pp2ppbp/2pp1np1/8/2PPP3/2N2N2/PP2BPPP/R1BQK2R w KQ - 0 1",
        "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
    ] {
        let mut position = get_position(fen);
        let mut search_state = default_search_state();
        search_state.show_info = false;
        search_state.end_time = Instant::now() + Duration::from_millis(600);

        search_state.history.push(position.zobrist_lock);
        let before = search_state.history.len();

        iterative_deepening(&mut position, 14, &mut search_state, 1);

        assert_eq!(
            search_state.history.len(),
            before,
            "history stack not restored for {} (before {}, after {})",
            fen,
            before,
            search_state.history.len()
        );
    }
}
