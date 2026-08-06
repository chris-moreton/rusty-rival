//! Colour-mirror consistency check for the NNUE evaluation (NET-400).
//!
//! `check_net` validates a net by calling the engine's own `evaluate()`, so it
//! cannot detect a perspective bug in `evaluate()` itself - a net trained to
//! compensate for one passes cleanly. This example needs no external oracle.
//!
//! The invariant: a position and its exact colour mirror - board flipped
//! vertically, every piece's colour swapped, side to move swapped, castling
//! rights swapped - are the *same position* as far as the side to move is
//! concerned. A dual-perspective net evaluates from the side to move's point of
//! view, so the two must score identically.
//!
//! If they do not, the STM/NTM accumulators are being combined asymmetrically,
//! which is exactly the shape of bug that an inverted training label would mask.
//!
//! Usage: cargo run --release --example mirror_eval [-- <net.bin>]

use rusty_rival::fen::get_position;
use rusty_rival::nnue::{Accumulator, NnueNetwork};
use rusty_rival::types::Position;

fn piece_count(pos: &Position) -> u32 {
    pos.pieces[0].all_pieces_bitboard.count_ones() + pos.pieces[1].all_pieces_bitboard.count_ones()
}

fn eval(net: &NnueNetwork, fen: &str) -> i32 {
    let pos = get_position(fen);
    let mut acc = Accumulator::default();
    acc.compute(net, &pos);
    net.evaluate(&acc, pos.mover, piece_count(&pos)) as i32
}

/// Flip a FEN vertically and swap every piece's colour.
fn mirror_fen(fen: &str) -> String {
    let parts: Vec<&str> = fen.split_whitespace().collect();
    let board: Vec<String> = parts[0]
        .split('/')
        .rev()
        .map(|rank| {
            rank.chars()
                .map(|c| {
                    if c.is_ascii_uppercase() {
                        c.to_ascii_lowercase()
                    } else if c.is_ascii_lowercase() {
                        c.to_ascii_uppercase()
                    } else {
                        c
                    }
                })
                .collect()
        })
        .collect();

    let stm = if parts[1] == "w" { "b" } else { "w" };

    let castle: String = if parts[2] == "-" {
        "-".to_string()
    } else {
        let mut c: String = parts[2]
            .chars()
            .map(|c| {
                if c.is_ascii_uppercase() {
                    c.to_ascii_lowercase()
                } else {
                    c.to_ascii_uppercase()
                }
            })
            .collect();
        // Keep the conventional KQkq ordering so the parser is happy.
        let mut v: Vec<char> = c.drain(..).collect();
        v.sort_by_key(|c| "KQkq".find(*c).unwrap_or(9));
        v.into_iter().collect()
    };

    // En-passant square mirrors rank 3 <-> 6.
    let ep = if parts[3] == "-" {
        "-".to_string()
    } else {
        let f = parts[3].chars().next().unwrap();
        let r = parts[3].chars().nth(1).unwrap();
        let nr = match r {
            '3' => '6',
            '6' => '3',
            other => other,
        };
        format!("{}{}", f, nr)
    };

    format!("{} {} {} {} {} {}", board.join("/"), stm, castle, ep, parts[4], parts[5])
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let net = match args.get(1) {
        Some(path) => {
            let bytes = std::fs::read(path).expect("cannot read net file");
            NnueNetwork::from_bytes(&bytes)
        }
        None => NnueNetwork::embedded(),
    };

    // Deliberately asymmetric positions, so a mismatch cannot hide behind a
    // symmetric board. No castling/en-passant in most, to keep the mirror exact.
    let fens = [
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "r1bqkbnr/pppp1ppp/2n5/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4",
        "4k3/8/8/8/8/8/8/3QK3 w - - 0 1",
        "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        "8/8/8/2k5/2pP4/8/B7/4K3 b - - 0 1",
        "2r3k1/1p3pp1/p1n1b2p/8/2P5/1P2BN2/P4PPP/3R2K1 w - - 0 1",
        "8/5k2/8/8/8/8/2R2K2/8 w - - 0 1",
    ];

    println!("Colour-mirror consistency of evaluate()  (NET-400)\n");
    println!("{:>10}  {:>10}  {:>8}   position", "original", "mirrored", "diff");
    println!("{}", "-".repeat(78));

    let mut worst = 0i32;
    let mut failures = 0;
    for fen in fens {
        let m = mirror_fen(fen);
        let a = eval(&net, fen);
        let b = eval(&net, &m);
        let d = a - b;
        worst = worst.max(d.abs());
        if d != 0 {
            failures += 1;
        }
        println!("{:>10}  {:>10}  {:>8}   {}", a, b, d, &fen[..fen.len().min(44)]);
    }

    println!();
    if failures == 0 {
        println!("PASS - evaluate() is colour-symmetric. Its perspective handling is self-consistent,");
        println!("       so a sign problem lies at the data/trainer end, not in the engine.");
    } else {
        println!(
            "FAIL - {}/{} positions disagree with their mirror (worst {} cp).",
            failures,
            fens.len(),
            worst
        );
        println!("       evaluate() combines the STM/NTM accumulators asymmetrically. An inverted");
        println!("       training label would mask exactly this, which is what NET-400 describes.");
        std::process::exit(1);
    }
}
