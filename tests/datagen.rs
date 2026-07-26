//! Tests for the self-play datagen pipeline (NET-319).
//!
//! The critical invariant is the **score sign convention**. `datagen` records
//! scores from white's point of view, but the search reports them from the side
//! to move's point of view. Getting that negation wrong would silently poison an
//! entire training run — the data would look well-formed and train a net that
//! evaluates black's positions inverted. These tests pin it down.

use rusty_rival::types::{default_search_state, default_uci_state};
use rusty_rival::uci::run_command_sync;

/// Search a FEN at a fixed node count and return the score from the side to
/// move's point of view, mirroring what `datagen` reads out of `current_best`.
fn stm_score(fen: &str, nodes: u64) -> i32 {
    let mut uci_state = default_uci_state();
    let mut search_state = default_search_state();
    search_state.show_info = false;

    run_command_sync(&mut uci_state, &mut search_state, "ucinewgame");
    run_command_sync(&mut uci_state, &mut search_state, &format!("position fen {}", fen));
    run_command_sync(&mut uci_state, &mut search_state, &format!("go nodes {}", nodes));

    search_state.current_best.1 as i32
}

/// The search reports from the side to move's perspective: the same position is
/// winning for the side that is up material, whichever side that is.
#[test]
fn search_score_is_from_side_to_move_perspective() {
    // White is a queen up in both cases; only the side to move differs.
    let white_to_move = stm_score("4k3/8/8/8/8/8/8/3QK3 w - - 0 1", 20_000);
    let black_to_move = stm_score("4k3/8/8/8/8/8/8/3QK3 b - - 0 1", 20_000);

    assert!(
        white_to_move > 100,
        "white is a queen up and to move, so the STM score should be strongly positive, got {}",
        white_to_move
    );
    assert!(
        black_to_move < -100,
        "white is a queen up but black is to move, so the STM score should be strongly negative, got {}",
        black_to_move
    );
}

/// Datagen negates the STM score when black is to move, so the recorded value is
/// always from white's point of view. Both of the above positions are white-up,
/// so both must record as positive.
#[test]
fn datagen_white_pov_conversion_agrees_for_both_sides() {
    // Mirrors the conversion in datagen.rs: `if mover == WHITE { s } else { -s }`.
    let white_pov_w = stm_score("4k3/8/8/8/8/8/8/3QK3 w - - 0 1", 20_000);
    let white_pov_b = -stm_score("4k3/8/8/8/8/8/8/3QK3 b - - 0 1", 20_000);

    assert!(white_pov_w > 100, "white-POV score should be positive, got {}", white_pov_w);
    assert!(white_pov_b > 100, "white-POV score should be positive, got {}", white_pov_b);

    // And the reverse: black a queen up must record negative from white's POV.
    let black_up_w = stm_score("3qk3/8/8/8/8/8/8/4K3 w - - 0 1", 20_000);
    let black_up_b = -stm_score("3qk3/8/8/8/8/8/8/4K3 b - - 0 1", 20_000);

    assert!(black_up_w < -100, "white-POV score should be negative, got {}", black_up_w);
    assert!(black_up_b < -100, "white-POV score should be negative, got {}", black_up_b);
}
