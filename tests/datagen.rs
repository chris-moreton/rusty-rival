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

/// Remove any shards left behind by an earlier run of a test.
///
/// This matters more than it looks: the generator resumes from whatever
/// complete shards it finds, so a stale shard would make a fresh test run
/// decide the work was already done and write nothing at all.
fn clean_shards(base: &str) {
    for path in shard_files(base) {
        let _ = std::fs::remove_file(path);
    }
}

/// Every shard file for an output base, in shard order.
///
/// Sealed shards are `<base>.<n>.zst`; a trailing short shard carries its game
/// count as `<base>.<n>.p<games>.zst`, so both spellings have to be picked up.
fn shard_files(base: &str) -> Vec<std::path::PathBuf> {
    let path = std::path::Path::new(base);
    let dir = match path.parent() {
        Some(p) if !p.as_os_str().is_empty() => p.to_path_buf(),
        _ => std::path::PathBuf::from("."),
    };
    let stem = path.file_name().unwrap().to_string_lossy().to_string();

    let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            let name = p.file_name().unwrap_or_default().to_string_lossy().to_string();
            name.starts_with(&format!("{}.", stem)) && name.ends_with(".zst")
        })
        .collect();
    files.sort();
    files
}

/// Decompress and concatenate every shard the generator wrote, in shard order.
fn read_shards(base: &str) -> Vec<String> {
    let mut out = Vec::new();
    for path in shard_files(base) {
        let bytes = std::fs::read(&path).expect("cannot read shard");
        let plain = zstd::decode_all(&bytes[..]).expect("shard is not valid zstd");
        let text = String::from_utf8(plain).expect("shard is not valid utf-8");
        out.extend(text.lines().filter(|l| !l.trim().is_empty()).map(str::to_string));
    }
    out
}

/// Positions produced, as a multiset.
///
/// Completion order across worker threads is not part of the guarantee - the
/// collector writes games as they finish - so every comparison here is on
/// content. Training data is shuffled before use, so ordering carries nothing.
fn sorted(mut lines: Vec<String>) -> Vec<String> {
    lines.sort();
    lines
}

/// Run the generator and return the positions it produced.
fn run_datagen(base: &str, games: &str, threads: &str) -> Vec<String> {
    run_datagen_at(base, games, threads, "800")
}

fn run_datagen_at(base: &str, games: &str, threads: &str, nodes: &str) -> Vec<String> {
    let mut uci_state = default_uci_state();
    let mut search_state = default_search_state();

    let parts = vec!["datagen", games, nodes, base, "6", threads];
    let result = cmd_datagen(&mut uci_state, &mut search_state, parts);
    assert!(result.is_right(), "datagen returned an error: {:?}", result.left());

    read_shards(base)
}

/// End-to-end smoke test for the multi-threaded generator.
///
/// Guards the things concurrency gets wrong rather than the chess: that the
/// worker/collector channel terminates (a missing `drop(tx)` hangs forever),
/// that the requested game count is honoured, and that no line is torn or
/// interleaved by concurrent writers.
#[test]
fn parallel_datagen_produces_well_formed_output() {
    let path = std::env::temp_dir().join("rusty_rival_datagen_wellformed");
    let base = path.to_string_lossy().to_string();
    clean_shards(&base);

    // 6 games, low node count, 3 workers, 6 random plies.
    let lines = run_datagen(&base, "6", "3");
    assert!(!lines.is_empty(), "datagen produced no positions");
    let lines: Vec<&str> = lines.iter().map(String::as_str).collect();

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

    clean_shards(&base);
}

/// The same arguments must produce byte-for-byte-equivalent records regardless
/// of worker count.
///
/// This is not a nicety — it is the property that makes an interrupted run
/// resumable, because resume replays game indices and trusts them to reproduce
/// the same games. It was genuinely broken until NET-319: every move-ordering
/// table is reset per game except the countermoves, so a game's result depended
/// on which *other* games its worker happened to be handed, which varies with
/// thread scheduling. Single-threaded runs matched and hid it; this test uses
/// several threads precisely so that they cannot.
#[test]
fn datagen_output_is_reproducible_across_threads() {
    let dir = std::env::temp_dir();
    let base_a = dir.join("rusty_rival_datagen_repro_a").to_string_lossy().to_string();
    let base_b = dir.join("rusty_rival_datagen_repro_b").to_string_lossy().to_string();
    clean_shards(&base_a);
    clean_shards(&base_b);

    // Sized to actually catch the leak: it needs enough games per worker for the
    // countermove table to accumulate, and enough nodes for move ordering to
    // change which move comes back. At 8 games of 800 nodes the two runs agreed
    // even with the bug present.
    let a = run_datagen_at(&base_a, "24", "1", "3000");
    let b = run_datagen_at(&base_b, "24", "4", "3000");

    assert!(!a.is_empty(), "datagen produced no positions");
    assert_eq!(a, b, "changing the worker count changed the data or its game-index order");

    clean_shards(&base_a);
    clean_shards(&base_b);
}

/// A finished run must not redo its work when invoked again.
///
/// The resume path keys off shards on disk, so re-running a completed
/// generation is a no-op rather than a duplicate append.
#[test]
fn datagen_does_not_redo_completed_work() {
    let path = std::env::temp_dir().join("rusty_rival_datagen_resume");
    let base = path.to_string_lossy().to_string();
    clean_shards(&base);

    let first = run_datagen(&base, "4", "2");
    assert!(!first.is_empty(), "datagen produced no positions");

    // Re-running with the same arguments finds the finished shard and stops.
    let second = run_datagen(&base, "4", "2");
    assert_eq!(
        sorted(first),
        sorted(second),
        "re-running a completed generation changed the output"
    );

    clean_shards(&base);
}

/// Extending an existing dataset must add exactly the missing games.
///
/// A run that does not end on a shard boundary leaves a short shard. That shard
/// records its game count in its filename precisely so this case works: credit
/// it with a full shard instead and the extension silently generates too few
/// games, leaving a dataset smaller than the one that was asked for while
/// reporting success.
#[test]
fn datagen_extends_a_dataset_to_exactly_the_requested_size() {
    let dir = std::env::temp_dir();
    let grown = dir.join("rusty_rival_datagen_grow").to_string_lossy().to_string();
    let fresh = dir.join("rusty_rival_datagen_fresh").to_string_lossy().to_string();
    clean_shards(&grown);
    clean_shards(&fresh);

    // Four games, then extend the same base to six.
    let partial = run_datagen(&grown, "4", "2");
    let extended = run_datagen(&grown, "6", "2");

    // ...must match six games generated in one go, because a game index always
    // reproduces the same game.
    let one_go = run_datagen(&fresh, "6", "2");

    assert!(extended.len() > partial.len(), "extending the dataset added nothing");
    assert_eq!(
        sorted(extended),
        sorted(one_go),
        "a dataset grown from 4 to 6 games differs from 6 games generated in one run"
    );

    clean_shards(&grown);
    clean_shards(&fresh);
}

/// More than one extension must count every earlier partial shard.
///
/// Before this regression test, the scanner stopped after the first partial
/// shard. A 2 -> 4 -> 6 sequence therefore treated the third invocation as if
/// only two games existed and overwrote the shard written by the second one.
#[test]
fn datagen_survives_repeated_partial_shard_extensions() {
    let dir = std::env::temp_dir();
    let grown = dir.join("rusty_rival_datagen_grow_repeated").to_string_lossy().to_string();
    let fresh = dir.join("rusty_rival_datagen_fresh_repeated").to_string_lossy().to_string();
    clean_shards(&grown);
    clean_shards(&fresh);

    run_datagen(&grown, "2", "2");
    run_datagen(&grown, "4", "2");
    let extended = run_datagen(&grown, "6", "2");
    let one_go = run_datagen(&fresh, "6", "2");

    assert_eq!(
        sorted(extended),
        sorted(one_go),
        "a repeatedly extended dataset differs from six games generated in one run"
    );
    assert_eq!(
        shard_files(&grown).len(),
        3,
        "each two-game invocation should remain an auditable shard"
    );

    clean_shards(&grown);
    clean_shards(&fresh);
}

/// A malformed file in the generator's reserved shard namespace must stop
/// resume rather than being ignored and potentially overwritten.
#[test]
fn datagen_rejects_malformed_partial_shard_names() {
    let base = std::env::temp_dir()
        .join("rusty_rival_datagen_malformed_partial")
        .to_string_lossy()
        .to_string();
    clean_shards(&base);
    let malformed = format!("{}.00000.pinvalid.zst", base);
    std::fs::write(&malformed, b"not a shard").expect("cannot create malformed shard fixture");

    let mut uci_state = default_uci_state();
    let mut search_state = default_search_state();
    let parts = vec!["datagen", "2", "800", &base, "6", "1"];
    let result = cmd_datagen(&mut uci_state, &mut search_state, parts);

    assert!(
        result.left().is_some_and(|e| e.contains("invalid partial shard")),
        "malformed partial shard should fail closed"
    );
    clean_shards(&base);
}

/// A directory cannot stand in for a durable sealed shard.
#[test]
fn datagen_rejects_directory_named_as_sealed_shard() {
    let base = std::env::temp_dir()
        .join("rusty_rival_datagen_sealed_directory")
        .to_string_lossy()
        .to_string();
    clean_shards(&base);
    let sealed = format!("{}.00000.zst", base);
    let _ = std::fs::remove_dir(&sealed);
    std::fs::create_dir(&sealed).expect("cannot create sealed-shard directory fixture");

    let mut uci_state = default_uci_state();
    let mut search_state = default_search_state();
    let parts = vec!["datagen", "2", "800", &base, "6", "1"];
    let result = cmd_datagen(&mut uci_state, &mut search_state, parts);

    assert!(
        result.left().is_some_and(|e| e.contains("not a regular file")),
        "sealed-shard directory should fail closed"
    );
    std::fs::remove_dir(&sealed).expect("cannot remove sealed-shard directory fixture");
}

/// A sealed and partial shard cannot both claim the same game-index range.
#[test]
fn datagen_rejects_ambiguous_sealed_and_partial_shards() {
    let base = std::env::temp_dir()
        .join("rusty_rival_datagen_ambiguous_shards")
        .to_string_lossy()
        .to_string();
    clean_shards(&base);
    std::fs::write(format!("{}.00000.zst", base), b"sealed").expect("cannot create sealed fixture");
    std::fs::write(format!("{}.00000.p2.zst", base), b"partial").expect("cannot create partial fixture");

    let mut uci_state = default_uci_state();
    let mut search_state = default_search_state();
    let parts = vec!["datagen", "2", "800", &base, "6", "1"];
    let result = cmd_datagen(&mut uci_state, &mut search_state, parts);

    assert!(
        result.left().is_some_and(|e| e.contains("ambiguous shard")),
        "conflicting shard files should fail closed"
    );
    clean_shards(&base);
}
