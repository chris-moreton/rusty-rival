//! Benchmark for the evaluation function
//!
//! Usage: cargo run --release --example bench_eval [epd_file] [iterations]
//!
//! Defaults:
//!   epd_file: epd/sts-v3.epd
//!   iterations: 100

use rusty_rival::evaluate::evaluate;
use rusty_rival::fen::get_position;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::Instant;

fn load_positions(epd_path: &str) -> Vec<rusty_rival::types::Position> {
    let file = File::open(epd_path).expect("Failed to open EPD file");
    let reader = BufReader::new(file);

    reader
        .lines()
        .filter_map(|line| {
            let line = line.ok()?;
            // EPD format: FEN (first 4-6 fields) followed by operations
            // We need to extract just the FEN part
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                // Reconstruct FEN: board, mover, castling, en-passant, [halfmove], [fullmove]
                let fen = if parts.len() >= 6 && parts[4].chars().all(|c| c.is_ascii_digit()) {
                    // Has halfmove and fullmove counters
                    format!("{} {} {} {} {} {}", parts[0], parts[1], parts[2], parts[3], parts[4], parts[5])
                } else {
                    // Standard EPD without move counters - add defaults
                    format!("{} {} {} {} 0 1", parts[0], parts[1], parts[2], parts[3])
                };
                Some(get_position(&fen))
            } else {
                None
            }
        })
        .collect()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let epd_path = args.get(1).map(|s| s.as_str()).unwrap_or("epd/sts-v3.epd");
    let iterations: u32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(100);

    println!("Loading positions from {}...", epd_path);
    let positions = load_positions(epd_path);
    println!("Loaded {} positions", positions.len());

    if positions.is_empty() {
        eprintln!("No positions loaded!");
        return;
    }

    let total_evals = positions.len() as u64 * iterations as u64;
    println!("Running {} iterations ({} total evaluations)...\n", iterations, total_evals);

    // Warmup run (not timed)
    for pos in &positions {
        let _ = evaluate(pos);
    }

    // Timed runs
    let start = Instant::now();

    let mut checksum: i64 = 0; // Prevent optimizer from eliminating evals
    for _ in 0..iterations {
        for pos in &positions {
            checksum = checksum.wrapping_add(evaluate(pos) as i64);
        }
    }

    let elapsed = start.elapsed();
    let elapsed_secs = elapsed.as_secs_f64();
    let evals_per_sec = total_evals as f64 / elapsed_secs;

    println!("Results:");
    println!("  Total time:     {:.3} seconds", elapsed_secs);
    println!("  Evaluations:    {}", total_evals);
    println!("  Evals/second:   {:.0}", evals_per_sec);
    println!(
        "  Time/eval:      {:.3} microseconds",
        elapsed_secs * 1_000_000.0 / total_evals as f64
    );
    println!("  Checksum:       {} (ignore - prevents optimization)", checksum);

    // Also test with pawn hash (more realistic)
    println!("\n--- With Pawn Hash Table ---\n");

    use rusty_rival::evaluate::evaluate_with_pawn_hash;
    use rusty_rival::types::PawnHashTable;

    let pawn_hash = PawnHashTable::new();

    // Warmup
    for pos in &positions {
        let _ = evaluate_with_pawn_hash(pos, &pawn_hash);
    }

    let start = Instant::now();

    let mut checksum: i64 = 0;
    for _ in 0..iterations {
        for pos in &positions {
            checksum = checksum.wrapping_add(evaluate_with_pawn_hash(pos, &pawn_hash) as i64);
        }
    }

    let elapsed = start.elapsed();
    let elapsed_secs = elapsed.as_secs_f64();
    let evals_per_sec = total_evals as f64 / elapsed_secs;

    println!("Results:");
    println!("  Total time:     {:.3} seconds", elapsed_secs);
    println!("  Evaluations:    {}", total_evals);
    println!("  Evals/second:   {:.0}", evals_per_sec);
    println!(
        "  Time/eval:      {:.3} microseconds",
        elapsed_secs * 1_000_000.0 / total_evals as f64
    );
    println!("  Checksum:       {} (ignore - prevents optimization)", checksum);
}
