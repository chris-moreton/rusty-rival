//! Local decomposition benchmark for the NNUE hot paths.

use rusty_rival::fen::get_position;
use rusty_rival::nnue::{update_accumulator_from, Accumulator, NnueNetwork};
use rusty_rival::types::{Position, BLACK, WHITE};
use std::hint::black_box;
use std::time::Instant;

fn piece_count(position: &Position) -> u32 {
    position.pieces[WHITE as usize].all_pieces_bitboard.count_ones() + position.pieces[BLACK as usize].all_pieces_bitboard.count_ones()
}

fn report(name: &str, iterations: u64, started: Instant) {
    let elapsed = started.elapsed();
    println!(
        "{name:<22} {:>9.2} ns/op  ({:.3}s)",
        elapsed.as_nanos() as f64 / iterations as f64,
        elapsed.as_secs_f64()
    );
}

fn main() {
    let net = NnueNetwork::embedded();
    let position = get_position("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
    let pieces = piece_count(&position);
    let mut accumulator = Accumulator::new();
    accumulator.compute(&net, &position);

    let eval_iterations = 20_000_000u64;
    let started = Instant::now();
    let mut checksum = 0i64;
    for _ in 0..eval_iterations {
        checksum =
            checksum.wrapping_add(black_box(net.evaluate(black_box(&accumulator), black_box(position.mover), black_box(pieces))) as i64);
    }
    report("forward evaluate", eval_iterations, started);
    black_box(checksum);

    let refresh_iterations = 500_000u64;
    let started = Instant::now();
    for _ in 0..refresh_iterations {
        accumulator.compute(black_box(&net), black_box(&position));
        black_box(&accumulator);
    }
    report("full refresh", refresh_iterations, started);

    let quiet_before = get_position("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let quiet_after = get_position("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1");
    let mut parent = Accumulator::new();
    let mut child = Accumulator::new();
    parent.compute(&net, &quiet_before);
    let update_iterations = 5_000_000u64;
    let started = Instant::now();
    for _ in 0..update_iterations {
        update_accumulator_from(
            black_box(&parent),
            black_box(&mut child),
            black_box(&net),
            black_box(&quiet_before.pieces),
            black_box(&quiet_after.pieces),
        );
        black_box(&child);
    }
    report("quiet update", update_iterations, started);

    let capture_before = get_position("4k3/8/8/3p4/4P3/8/8/4K3 w - - 0 1");
    let capture_after = get_position("4k3/8/8/3P4/8/8/8/4K3 b - - 0 1");
    parent.compute(&net, &capture_before);
    let started = Instant::now();
    for _ in 0..update_iterations {
        update_accumulator_from(
            black_box(&parent),
            black_box(&mut child),
            black_box(&net),
            black_box(&capture_before.pieces),
            black_box(&capture_after.pieces),
        );
        black_box(&child);
    }
    report("capture update", update_iterations, started);
}
