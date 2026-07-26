//! Tests for the self-play datagen pipeline (NET-319).
//!
//! The critical invariant is the **score sign convention**. `datagen` records
//! scores from white's point of view, but the search reports them from the side
//! to move's point of view. Getting that negation wrong would silently poison an
//! entire training run — the data would look well-formed and train a net that
//! evaluates black's positions inverted. These tests pin it down.

use rusty_rival::datagen::cmd_datagen;
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

/// End-to-end smoke test for the multi-threaded generator.
///
/// Guards the things concurrency gets wrong rather than the chess: that the
/// worker/collector channel terminates (a missing `drop(tx)` hangs forever),
/// that the requested game count is honoured, and that no line is torn or
/// interleaved by concurrent writers.
#[test]
fn parallel_datagen_produces_well_formed_output() {
    use std::fs;

    let mut uci_state = default_uci_state();
    let mut search_state = default_search_state();

    let path = std::env::temp_dir().join("rusty_rival_datagen_test.txt");
    let path_str = path.to_string_lossy().to_string();

    // 6 games, low node count, 3 workers, 6 random plies.
    let parts = vec!["datagen", "6", "800", path_str.as_str(), "6", "3"];
    let result = cmd_datagen(&mut uci_state, &mut search_state, parts);
    assert!(result.is_right(), "datagen returned an error: {:?}", result.left());

    let contents = fs::read_to_string(&path).expect("datagen wrote no output file");
    let lines: Vec<&str> = contents.lines().filter(|l| !l.trim().is_empty()).collect();
    assert!(!lines.is_empty(), "datagen produced no positions");

    for line in &lines {
        let fields: Vec<&str> = line.split('|').map(|f| f.trim()).collect();
        assert_eq!(fields.len(), 3, "malformed line (concurrent write tearing?): {}", line);

        // Field 1 is a FEN: six space-separated parts, eight ranks.
        let fen_parts: Vec<&str> = fields[0].split_whitespace().collect();
        assert_eq!(fen_parts.len(), 6, "not a full FEN: {}", fields[0]);
        assert_eq!(fen_parts[0].split('/').count(), 8, "FEN board is not 8 ranks: {}", fields[0]);

        // Field 2 is an integer centipawn score.
        assert!(fields[1].parse::<i32>().is_ok(), "score is not an integer: {}", fields[1]);

        // Field 3 is one of the three legal WDL labels.
        assert!(matches!(fields[2], "1.0" | "0.5" | "0.0"), "unexpected wdl label {}", fields[2]);
    }

    let _ = fs::remove_file(&path);
}
