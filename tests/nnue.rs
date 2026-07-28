use rusty_rival::fen::get_position;
use rusty_rival::nnue::{Accumulator, NnueNetwork};
use rusty_rival::types::{Position, BLACK, WHITE};

/// Total pieces on the board — selects the NNUE output bucket.
fn piece_count(position: &Position) -> u32 {
    position.pieces[WHITE as usize].all_pieces_bitboard.count_ones() + position.pieces[BLACK as usize].all_pieces_bitboard.count_ones()
}

#[test]
fn nnue_eval_signs_are_correct() {
    let net = NnueNetwork::embedded();

    // Material edge in a REALISTIC position. Bare KQvK is a trap: Stockfish
    // game data barely contains such positions, so every net in this family
    // scores them near zero regardless of health. The 512x2 net scores KQvK at
    // 74 cp and the shipped 256 net at 153 - neither is evidence of anything.
    // With a full middlegame around it, a queen is worth hundreds to any sane
    // net, and a swapped-perspective net inverts the sign.
    let mut acc = Accumulator::new();

    let pos = get_position("r1b1kb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 0 1");
    acc.compute(&net, &pos);
    let white_queen_up = net.evaluate(&acc, WHITE, piece_count(&pos));
    println!("white a queen up, white to move: {}", white_queen_up);
    assert!(
        white_queen_up > 300,
        "White a queen up should be strongly positive, got {}",
        white_queen_up
    );

    let pos = get_position("r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNB1K2R w KQkq - 0 1");
    acc.compute(&net, &pos);
    let black_queen_up = net.evaluate(&acc, WHITE, piece_count(&pos));
    println!("black a queen up, white to move: {}", black_queen_up);
    assert!(
        black_queen_up < -300,
        "Black a queen up should be strongly negative, got {}",
        black_queen_up
    );
}

#[test]
fn nnue_eval_is_symmetric() {
    let net = NnueNetwork::embedded();

    // Position A: white has pawn advantage, white to move
    let pos_a = get_position("4k3/8/8/8/4P3/8/8/4K3 w - - 0 1");
    let mut acc = Accumulator::new();
    acc.compute(&net, &pos_a);
    let eval_a = net.evaluate(&acc, WHITE, piece_count(&pos_a));

    // Position B: exact color mirror - black has pawn advantage, black to move
    // (flip all colors and flip board vertically)
    let pos_b = get_position("4k3/8/8/8/4p3/8/8/4K3 b - - 0 1");
    acc.compute(&net, &pos_b);
    let eval_b = net.evaluate(&acc, 1, piece_count(&pos_b)); // black STM

    println!("White pawn e4, white STM: {}", eval_a);
    println!("Black pawn e4, black STM: {}", eval_b);
    let diff = (eval_a - eval_b).abs();
    println!("Difference: {} (should be small if symmetric)", diff);
    // Allow some tolerance since the net isn't perfectly symmetric
    assert!(diff < 80, "Mirrored positions should have similar eval, diff was {}", diff);
}

#[test]
fn nnue_material_ordering() {
    let net = NnueNetwork::embedded();
    let mut acc = Accumulator::new();

    // Just kings
    let pos = get_position("4k3/8/8/8/8/8/8/4K3 w - - 0 1");
    acc.compute(&net, &pos);
    let kings_only = net.evaluate(&acc, WHITE, piece_count(&pos));

    // White has a pawn
    let pos = get_position("4k3/8/8/8/4P3/8/8/4K3 w - - 0 1");
    acc.compute(&net, &pos);
    let white_pawn = net.evaluate(&acc, WHITE, piece_count(&pos));

    // White has a knight
    let pos = get_position("4k3/8/8/8/4N3/8/8/4K3 w - - 0 1");
    acc.compute(&net, &pos);
    let white_knight = net.evaluate(&acc, WHITE, piece_count(&pos));

    // White has a queen
    let pos = get_position("4k3/8/8/8/4Q3/8/8/4K3 w - - 0 1");
    acc.compute(&net, &pos);
    let white_queen = net.evaluate(&acc, WHITE, piece_count(&pos));

    println!("Kings only: {}", kings_only);
    println!("White pawn: {}", white_pawn);
    println!("White knight: {}", white_knight);
    println!("White queen: {}", white_queen);

    // Basic ordering: queen > pawn > kings_only (knight may be weak in first net)
    assert!(
        white_queen > white_pawn,
        "Queen ({}) should be better than pawn ({})",
        white_queen,
        white_pawn
    );
    assert!(
        white_pawn >= kings_only,
        "Pawn ({}) should be at least as good as bare kings ({})",
        white_pawn,
        kings_only
    );
}

#[test]
fn nnue_startpos_is_roughly_equal() {
    let net = NnueNetwork::embedded();
    let pos = get_position("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let mut acc = Accumulator::new();
    acc.compute(&net, &pos);
    let eval = net.evaluate(&acc, WHITE, piece_count(&pos));
    println!("Starting position eval: {}", eval);
    // Should be roughly equal (small white advantage typical)
    assert!(eval.abs() < 100, "Starting position should be near 0, got {}", eval);
}

#[test]
fn debug_knight_eval() {
    let net = NnueNetwork::embedded();

    // Kings only
    let pos = get_position("4k3/8/8/8/8/8/8/4K3 w - - 0 1");
    let mut acc = Accumulator::new();
    acc.compute(&net, &pos);
    let kings_eval = net.evaluate(&acc, WHITE, piece_count(&pos));

    // Add a white rook on e4 instead
    let pos2 = get_position("4k3/8/8/8/4R3/8/8/4K3 w - - 0 1");
    acc.compute(&net, &pos2);
    let rook_eval = net.evaluate(&acc, WHITE, piece_count(&pos2));

    // Add a white knight on e4
    let pos3 = get_position("4k3/8/8/8/4N3/8/8/4K3 w - - 0 1");
    acc.compute(&net, &pos3);
    let knight_eval = net.evaluate(&acc, WHITE, piece_count(&pos3));

    // Knight on a completely different square
    let pos4 = get_position("4k3/8/8/8/8/8/8/N3K3 w - - 0 1");
    acc.compute(&net, &pos4);
    let knight_a1 = net.evaluate(&acc, WHITE, piece_count(&pos4));

    // Two knights
    let pos5 = get_position("4k3/8/8/8/4N3/8/8/N3K3 w - - 0 1");
    acc.compute(&net, &pos5);
    let two_knights = net.evaluate(&acc, WHITE, piece_count(&pos5));

    println!("Kings only: {}", kings_eval);
    println!("Rook e4: {}", rook_eval);
    println!("Knight e4: {}", knight_eval);
    println!("Knight a1: {}", knight_a1);
    println!("Two knights: {}", two_knights);

    // The key question: is the knight contributing ANYTHING?
    println!("Knight e4 delta from kings: {}", knight_eval - kings_eval);
    println!("Rook e4 delta from kings: {}", rook_eval - kings_eval);
}

/// Golden-value test (NET-320): pins the exact centipawn output of the NNUE
/// forward pass for a fixed set of positions against the embedded net.
///
/// These numbers are not "correct" in any absolute sense — they are simply what
/// `nets/rival-256x2-ob8.bin` (8 output buckets, NET-321) produces today.
/// They were regenerated deliberately when that net replaced the single-bucket
/// `rival-256x2.bin`, after `check_net` confirmed it loads with correct signs. The point is that any change to the
/// inference path (SIMD, i64 accumulation, quantisation, weight layout) must be
/// **bit-identical**, so this test failing means the refactor changed the eval.
///
/// If the net itself is retrained, these values must be regenerated deliberately
/// — never "fixed" by pasting in whatever the new code happens to print.
#[test]
fn nnue_golden_values_are_bit_identical() {
    let net = NnueNetwork::embedded();
    let mut acc = Accumulator::new();

    // (expected_cp, fen) — evaluated from the side to move's perspective.
    let golden: &[(i32, &str)] = &[
        (-26, "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"),
        (-223, "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"),
        (-76, "8/2p5/3p4/KP5r/1R3pPk/8/4P3/8 b - g3 0 1"),
        (-13, "n1n5/PPPk4/8/8/8/8/4Kppp/5N1N w - - 0 1"),
        (350, "4r1k1/5bpp/2p5/3pr3/8/1B3pPq/PPR2P2/2R2QK1 b - - 0 1"),
        (143, "4k3/8/8/8/8/8/8/3QK3 w - - 0 1"),
        (9, "8/8/8/4k3/8/8/4K3/8 w - - 0 1"),
        (-81, "r1bqkbnr/pppp1ppp/2n5/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 0 1"),
    ];

    for &(expected, fen) in golden {
        let pos = get_position(fen);
        acc.compute(&net, &pos);
        let actual = net.evaluate(&acc, pos.mover, piece_count(&pos)) as i32;
        assert_eq!(
            actual, expected,
            "NNUE eval changed for {}: expected {}, got {}",
            fen, expected, actual
        );
    }
}

/// Regression test for the incremental accumulator chain (NET-212): walking the
/// tree with make_move_nnue/unmake_move_nnue and evaluating lazily must produce
/// exactly the same score as a from-scratch accumulator computation, including
/// across captures, castling, en passant, and promotions, and including chains
/// that span several plies between evaluations.
#[test]
fn incremental_accumulator_matches_full_recompute() {
    use rusty_rival::evaluate::evaluate_position;
    use rusty_rival::moves::{generate_moves, is_check};
    use rusty_rival::search::{make_move_nnue, unmake_move_nnue};
    use rusty_rival::types::{default_search_state, Position, SearchState};

    fn dfs(pos: &mut Position, ss: &mut SearchState, depth: u8, checked: &mut u32) {
        // Skip evaluation at some nodes (keyed off the zobrist) so the lazy
        // chain has to span multiple plies, not just parent-to-child
        if pos.zobrist_lock % 3 != 0 {
            let incremental = evaluate_position(pos, ss);
            let net = ss.nnue_network.clone().unwrap();
            let mut acc = Accumulator::new();
            acc.compute(&net, pos);
            let full = net.evaluate(&acc, pos.mover, piece_count(&pos));
            assert_eq!(
                incremental, full,
                "incremental {} != full {} at nnue_ply {}",
                incremental, full, ss.nnue_ply
            );
            *checked += 1;
        }
        if depth == 0 {
            return;
        }
        for m in generate_moves(pos) {
            let old_mover = pos.mover;
            let unmake = make_move_nnue(pos, m, ss);
            if !is_check(pos, old_mover) {
                dfs(pos, ss, depth - 1, checked);
            }
            unmake_move_nnue(pos, m, &unmake, ss);
        }
    }

    let fens = [
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        // Kiwipete: castling both sides, en passant potential, heavy tactics
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        // Promotion race with capture-promotions
        "n1n5/PPPk4/8/8/8/8/4Kppp/5N1N w - - 0 1",
        // En passant is immediately available
        "8/2p5/3p4/KP5r/1R3pPk/8/4P3/8 b - g3 0 1",
    ];

    for fen in fens {
        let mut pos = get_position(fen);
        let mut ss = default_search_state();
        // Root initialization, as iterative_deepening does it
        let net = ss.nnue_network.clone().unwrap();
        ss.nnue_ply = 0;
        ss.nnue_accumulators[0].compute(&net, &pos);
        ss.nnue_pieces[0] = pos.pieces;
        ss.nnue_computed[0] = true;

        let mut checked = 0u32;
        dfs(&mut pos, &mut ss, 3, &mut checked);
        assert!(checked > 500, "too few comparisons for {}: {}", fen, checked);
    }
}

// =============================================================================
// Output buckets (NET-321)
// =============================================================================

use rusty_rival::nnue::{output_bucket, HIDDEN_SIZE, INPUT_SIZE, NUM_OUTPUT_BUCKETS};

/// The bucket formula must match bullet's `MaterialCount<N>` exactly:
/// `(occ.count_ones() - 2) / 32usize.div_ceil(N)`. If it drifts, the engine
/// reads a different bucket than training wrote and the eval is silently wrong.
#[test]
fn output_bucket_matches_bullet_material_count() {
    // Reference implementation, transcribed from bullet's outputs.rs.
    fn bullet_bucket(piece_count: u32) -> usize {
        let divisor = 32usize.div_ceil(NUM_OUTPUT_BUCKETS);
        ((piece_count as usize - 2) / divisor).min(NUM_OUTPUT_BUCKETS - 1)
    }

    // 2 pieces (bare kings) through 32 (startpos) is the full legal range.
    for pieces in 2..=32u32 {
        assert_eq!(output_bucket(pieces), bullet_bucket(pieces), "bucket mismatch at {} pieces", pieces);
    }

    // Spot-check the boundaries explicitly so a divisor change is obvious.
    assert_eq!(output_bucket(2), 0, "bare kings -> bucket 0");
    assert_eq!(output_bucket(5), 0, "(5-2)/4 = 0");
    assert_eq!(output_bucket(6), 1, "(6-2)/4 = 1");
    assert_eq!(output_bucket(32), 7, "startpos -> top bucket");

    // Out-of-range inputs must clamp rather than panic or read out of bounds.
    assert_eq!(output_bucket(0), 0);
    assert_eq!(output_bucket(1), 0);
    assert_eq!(output_bucket(64), NUM_OUTPUT_BUCKETS - 1);
}

/// Build a synthetic 8-bucket net whose only non-zero values are the per-bucket
/// biases, chosen so bucket k evaluates to exactly (k+1)*100 centipawns.
///
/// With all L0 weights and biases zero, every accumulator entry is 0, so both
/// SCReLU terms vanish and the output reduces to the bucket's bias alone:
///   eval = bias * EVAL_SCALE / (QA*QB) = bias * 400 / 16320
/// Setting bias_k = 4080*(k+1) gives eval = 100*(k+1).
fn synthetic_bucketed_net() -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    let mut push = |v: i16| out.extend_from_slice(&v.to_le_bytes());

    for _ in 0..(INPUT_SIZE * HIDDEN_SIZE) {
        push(0); // l0 weights
    }
    for _ in 0..HIDDEN_SIZE {
        push(0); // l0 biases
    }
    for _ in 0..(NUM_OUTPUT_BUCKETS * 2 * HIDDEN_SIZE) {
        push(0); // l1 weights, all buckets
    }
    for k in 0..NUM_OUTPUT_BUCKETS {
        push((4080 * (k + 1)) as i16); // l1 bias per bucket
    }
    out
}

/// End-to-end check of the bucketed loading path, using a synthetic net so it
/// runs before any bucketed net has actually been trained.
///
/// This is the guard that the `[bucket][512]` layout assumption and the bucket
/// selection agree: each bucket is given a distinct signature and the eval must
/// return the one matching the position's piece count.
#[test]
fn bucketed_net_selects_the_right_bucket() {
    let bytes = synthetic_bucketed_net();

    // Size must be exactly what a real bucketed quantised.bin will be.
    let expected_i16 = INPUT_SIZE * HIDDEN_SIZE + HIDDEN_SIZE + NUM_OUTPUT_BUCKETS * 2 * HIDDEN_SIZE + NUM_OUTPUT_BUCKETS;
    assert_eq!(bytes.len(), expected_i16 * 2, "synthetic net is the wrong size");

    let net = NnueNetwork::from_bytes(&bytes);
    let mut acc = Accumulator::new();

    // (fen, expected piece count) spanning several buckets.
    let cases = [
        ("4k3/8/8/8/8/8/8/4K3 w - - 0 1", 2u32),
        ("4k3/8/8/8/8/8/4P3/4K3 w - - 0 1", 3),
        ("4k3/4p3/8/8/8/8/4P3/4K3 w - - 0 1", 4),
        ("4k3/3ppp2/8/8/8/8/3PPP2/4K3 w - - 0 1", 8),
        ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 32),
    ];

    for (fen, expected_pieces) in cases {
        let pos = get_position(fen);
        let pieces = piece_count(&pos);
        assert_eq!(pieces, expected_pieces, "test case piece count wrong for {}", fen);

        acc.compute(&net, &pos);
        let eval = net.evaluate(&acc, pos.mover, pieces) as i32;

        let bucket = output_bucket(pieces);
        let expected = 100 * (bucket as i32 + 1);
        assert_eq!(
            eval, expected,
            "{} ({} pieces) should hit bucket {} and eval {}, got {}",
            fen, pieces, bucket, expected, eval
        );
    }
}

/// A single-bucket (legacy) net must be replicated across all buckets, so its
/// evaluation is independent of piece count and identical to the pre-bucket
/// implementation.
///
/// The legacy net is loaded explicitly rather than via `embedded()`, because
/// the embedded net is now the bucketed one — reading it here would test the
/// opposite property. Keeping this guard matters: the loader still accepts
/// single-bucket nets, which is the escape hatch if the bucketed net loses its
/// A/B match and we need to revert with a one-line change.
#[test]
fn single_bucket_net_is_piece_count_independent() {
    // Synthesised at the current HIDDEN_SIZE rather than read from
    // nets/rival-256x2.bin. That file is 256-wide, so once HIDDEN_SIZE became
    // 512 loading it ran off the end of the buffer - the loader cannot tell
    // 256 from 512. Synthesising keeps this guard meaningful at any width.
    let mut bytes: Vec<u8> = Vec::new();
    let mut push = |v: i16| bytes.extend_from_slice(&v.to_le_bytes());
    for i in 0..(INPUT_SIZE * HIDDEN_SIZE) {
        push((i % 7) as i16 - 3); // arbitrary but non-uniform
    }
    for _ in 0..HIDDEN_SIZE {
        push(1);
    }
    for i in 0..(2 * HIDDEN_SIZE) {
        push((i % 5) as i16 - 2);
    }
    push(64); // single L1 bias
    let net = NnueNetwork::from_bytes(&bytes);
    let mut acc = Accumulator::new();

    // Same position evaluated while claiming wildly different piece counts.
    // With a single-bucket net every bucket holds the same weights, so the
    // result must not move.
    let pos = get_position("4k3/8/8/8/8/8/8/3QK3 w - - 0 1");
    acc.compute(&net, &pos);

    let baseline = net.evaluate(&acc, pos.mover, piece_count(&pos));
    for claimed in [2u32, 7, 15, 23, 32] {
        assert_eq!(
            net.evaluate(&acc, pos.mover, claimed),
            baseline,
            "single-bucket net changed with claimed piece count {}",
            claimed
        );
    }
}
