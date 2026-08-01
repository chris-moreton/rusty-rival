use rusty_rival::fen::{algebraic_move_from_move, get_position};
use rusty_rival::moves::generate_moves;
use rusty_rival::quiesce::score_quiesce_move;
use rusty_rival::types::default_search_state;
use std::collections::HashMap;

/// Score every legal move in `fen` through the qsearch move scorer, keyed by
/// algebraic notation.
fn quiesce_scores(fen: &str) -> HashMap<String, i32> {
    let position = get_position(fen);
    let mut search_state = default_search_state();
    let enemy = position.pieces[position.mover as usize ^ 1];
    generate_moves(&position)
        .iter()
        .map(|&m| {
            (
                algebraic_move_from_move(m),
                score_quiesce_move(&position, m, &enemy, &mut search_state),
            )
        })
        .collect()
}

/// NET-366: qsearch MVV-LVA must try the LEAST valuable attacker first.
/// A 2022 refactor flipped the attacker bonus from `+` to `-`, so for four
/// years the MOST valuable attacker was tried first (QxQ before PxQ).
#[test]
fn qsearch_orders_cheapest_attacker_first() {
    // Black queen on d5, capturable by the c4 pawn and the d2 queen.
    let scores = quiesce_scores("3k4/8/8/3q4/2P5/8/3Q4/3K4 w - - 0 1");
    let pxq = scores["c4d5"];
    let qxq = scores["d2d5"];
    assert!(
        pxq > qxq,
        "PxQ ({}) must sort above QxQ ({}): least valuable attacker first",
        pxq,
        qxq
    );
}

/// NET-366, in-check corollary: capturing the checker with a pawn must sort
/// above a quiet king evasion. Pre-fix, PxP scored 175 - 300 = -125 and sorted
/// below every quiet evasion (score 0).
#[test]
fn qsearch_orders_pawn_captures_checker_above_quiet_evasions() {
    // White Kd2 is in check from the black e3 pawn; f2xe3 captures the checker.
    let scores = quiesce_scores("4k3/8/8/8/8/4p3/3K1P2/8 w - - 0 1");
    let pawn_takes_checker = scores["f2e3"];
    let quiet_evasion = scores["d2c1"];
    assert!(
        pawn_takes_checker > quiet_evasion,
        "PxP capturing the checker ({}) must sort above a quiet evasion ({})",
        pawn_takes_checker,
        quiet_evasion
    );
}
