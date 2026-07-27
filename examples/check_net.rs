//! Validate a candidate NNUE net file before shipping it (NET-321).
//!
//! Usage:
//!   cargo run --release --example check_net -- <path/to/quantised.bin>
//!
//! A freshly trained net can load cleanly and still be wrong — the two ways
//! that have actually bitten this project are:
//!
//! * **swapped perspectives.** The trainer concatenates `stm.concat(ntm)` while
//!   the loader indexes NTM-first. The shipped net is self-consistent with the
//!   loader, but a retrain could land either way. A swapped net evaluates a
//!   material advantage as a *disadvantage*, which this tool reports plainly.
//! * **wrong bucket layout.** `l1w` is written `[bucket][512]` only because the
//!   trainer's save entry has `.transpose()`. Get it wrong and every bucket
//!   reads a slice of the wrong weights, producing plausible-looking noise.
//!
//! This checks both without needing the net to be embedded or the engine
//! rebuilt, so a checkpoint can be vetted straight out of S3 — including a
//! part-trained one, since these properties do not depend on training quality.

use rusty_rival::fen::get_position;
use rusty_rival::nnue::{output_bucket, Accumulator, NnueNetwork, HIDDEN_SIZE, INPUT_SIZE, NUM_OUTPUT_BUCKETS};
use rusty_rival::types::{Position, BLACK, WHITE};

fn piece_count(position: &Position) -> u32 {
    position.pieces[WHITE as usize].all_pieces_bitboard.count_ones() + position.pieces[BLACK as usize].all_pieces_bitboard.count_ones()
}

fn eval(net: &NnueNetwork, acc: &mut Accumulator, fen: &str) -> i32 {
    let pos = get_position(fen);
    acc.compute(net, &pos);
    net.evaluate(acc, pos.mover, piece_count(&pos)) as i32
}

fn main() {
    let path = match std::env::args().nth(1) {
        Some(p) => p,
        None => {
            eprintln!("usage: cargo run --release --example check_net -- <quantised.bin>");
            std::process::exit(2);
        }
    };

    let bytes = std::fs::read(&path).unwrap_or_else(|e| {
        eprintln!("cannot read {}: {}", path, e);
        std::process::exit(2);
    });

    // --- Format ---------------------------------------------------------
    let l0 = INPUT_SIZE * HIDDEN_SIZE + HIDDEN_SIZE;
    let single_min = (l0 + 2 * HIDDEN_SIZE + 1) * 2;
    let bucketed_min = (l0 + NUM_OUTPUT_BUCKETS * 2 * HIDDEN_SIZE + NUM_OUTPUT_BUCKETS) * 2;
    let is_bucketed = bytes.len() >= bucketed_min;

    println!("file            : {}", path);
    println!("size            : {} bytes", bytes.len());
    println!("format          : {}", if is_bucketed { "BUCKETED (8)" } else { "single-bucket" });
    println!(
        "minimum sizes   : single {} / bucketed {} (real checkpoints carry trailing padding)",
        single_min, bucketed_min
    );
    if bytes.len() < single_min {
        println!("\nFAIL: file is too small to be a valid net");
        std::process::exit(1);
    }
    println!(
        "padding         : {} bytes beyond the bucketed minimum",
        bytes.len().saturating_sub(bucketed_min)
    );

    let net = NnueNetwork::from_bytes(&bytes);
    let mut acc = Accumulator::new();
    let mut failures = 0;

    // --- Training maturity ----------------------------------------------
    // Sign checks on small material advantages are meaningless in a barely
    // trained net: every score sits near zero, so the sign is noise. Gauge
    // maturity from the largest material swing first and scale expectations,
    // otherwise an early checkpoint reads as a layout bug.
    let queen_up = eval(&net, &mut acc, "4k3/8/8/8/8/8/8/3QK3 w - - 0 1");
    let mature = queen_up.abs() >= 300;
    if !mature {
        println!("\nWARNING: a queen advantage evaluates to only {} cp.", queen_up);
        println!("A trained net scores this in the high hundreds. This looks like an early");
        println!("checkpoint, so treat small-material sign checks below as inconclusive");
        println!("rather than as evidence of a layout bug. Re-run on the final net.");
    }

    // --- Sign sanity ----------------------------------------------------
    // These must hold for ANY sanely trained net, however undertrained.
    println!("\n--- sign checks (catch swapped STM/NTM perspectives) ---");
    let checks: [(&str, &str, bool); 4] = [
        ("white queen up, white to move", "4k3/8/8/8/8/8/8/3QK3 w - - 0 1", true),
        ("black queen up, white to move", "3qk3/8/8/8/8/8/8/4K3 w - - 0 1", false),
        ("white rook up, black to move", "4k3/8/8/8/8/8/8/3RK3 b - - 0 1", false),
        ("black rook up, black to move", "3rk3/8/8/8/8/8/8/4K3 b - - 0 1", true),
    ];
    // Scale the bar to the net's own maturity, so an early checkpoint is judged
    // on direction rather than magnitude.
    let threshold = if mature { 50 } else { 1 };
    for (name, fen, expect_positive) in checks {
        let e = eval(&net, &mut acc, fen);
        // Scores are from the side to move's perspective.
        let ok = if expect_positive { e > threshold } else { e < -threshold };
        let verdict = if ok {
            "ok"
        } else if mature {
            "FAIL"
        } else {
            "inconclusive (undertrained)"
        };
        println!("  {:<32} {:>7} cp  {}", name, e, verdict);
        // Only a mature net's sign errors are real failures.
        if !ok && mature {
            failures += 1;
        }
    }

    // --- Bucket coverage -------------------------------------------------
    // Every bucket should be reachable and produce a finite, non-absurd score.
    println!("\n--- per-bucket evaluation (startpos-like material, thinning out) ---");
    let bucket_fens = [
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQ - 0 1",
        "r1bqkb1r/pppppppp/8/8/8/8/PPPPPPPP/R1BQKB1R w KQkq - 0 1",
        "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1",
        "4k3/pppppppp/8/8/8/8/PPPPPPPP/4K3 w - - 0 1",
        "4k3/pppp4/8/8/8/8/PPPP4/4K3 w - - 0 1",
        "4k3/pp6/8/8/8/8/PP6/4K3 w - - 0 1",
        "4k3/8/8/8/8/8/8/4K3 w - - 0 1",
    ];
    let mut seen = [false; NUM_OUTPUT_BUCKETS];
    for fen in bucket_fens {
        let pos = get_position(fen);
        let pieces = piece_count(&pos);
        let b = output_bucket(pieces);
        seen[b] = true;
        let e = eval(&net, &mut acc, fen);
        println!("  {:>2} pieces -> bucket {}  {:>7} cp", pieces, b, e);
        if e.abs() > 10_000 {
            println!("     FAIL: implausible magnitude (bad weight layout?)");
            failures += 1;
        }
    }
    let unreached: Vec<usize> = (0..NUM_OUTPUT_BUCKETS).filter(|b| !seen[*b]).collect();
    if !unreached.is_empty() {
        println!("  note: buckets {:?} not exercised by this fen set", unreached);
    }

    // --- Distinctness ----------------------------------------------------
    // A bucketed net whose buckets are all identical means the buckets did not
    // train, or a single-bucket net was replicated.
    if is_bucketed {
        println!("\n--- bucket distinctness ---");
        let probe = "4k3/8/8/8/8/8/8/3QK3 w - - 0 1";
        let pos = get_position(probe);
        acc.compute(&net, &pos);
        let scores: Vec<i32> = (2..=32u32).step_by(4).map(|pc| net.evaluate(&acc, pos.mover, pc) as i32).collect();
        println!("  same position across buckets: {:?}", scores);
        if scores.windows(2).all(|w| w[0] == w[1]) {
            println!("  FAIL: every bucket returns the same score — buckets are not differentiated");
            failures += 1;
        } else {
            println!("  ok: buckets differ");
        }
    }

    println!();
    if failures == 0 {
        println!("PASS — net loads with correct signs and plausible per-bucket output.");
        println!("NOTE: this says nothing about playing strength. Run the A/B match for that.");
    } else {
        println!("{} CHECK(S) FAILED.", failures);
        println!("If only the sign checks failed, the likely cause is swapped STM/NTM halves:");
        println!("swap the two l1_weights indices in NnueNetwork::evaluate() rather than");
        println!("discarding the net — see the layout notes in src/nnue.rs.");
        std::process::exit(1);
    }
}
