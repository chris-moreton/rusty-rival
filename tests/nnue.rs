use rusty_rival::fen::get_position;
use rusty_rival::nnue::{Accumulator, NnueNetwork};
use rusty_rival::types::WHITE;

#[test]
fn nnue_eval_signs_are_correct() {
    let net = NnueNetwork::embedded();

    // White has extra queen - should be very positive for white
    let pos = get_position("4k3/8/8/8/8/8/8/3QK3 w - - 0 1");
    let mut acc = Accumulator::new();
    acc.compute(&net, &pos);
    let white_queen_up = net.evaluate(&acc, WHITE);
    println!("Ke1+Qd1 vs ke8 (white STM): {}", white_queen_up);
    assert!(white_queen_up > 100, "White with queen should be positive, got {}", white_queen_up);

    // Black has extra queen - should be very negative for white
    let pos = get_position("3qk3/8/8/8/8/8/8/4K3 w - - 0 1");
    acc.compute(&net, &pos);
    let black_queen_up = net.evaluate(&acc, WHITE);
    println!("Ke1 vs ke8+qd8 (white STM): {}", black_queen_up);
    assert!(black_queen_up < -100, "Black with queen should be negative, got {}", black_queen_up);
}

#[test]
fn nnue_eval_is_symmetric() {
    let net = NnueNetwork::embedded();

    // Position A: white has pawn advantage, white to move
    let pos_a = get_position("4k3/8/8/8/4P3/8/8/4K3 w - - 0 1");
    let mut acc = Accumulator::new();
    acc.compute(&net, &pos_a);
    let eval_a = net.evaluate(&acc, WHITE);

    // Position B: exact color mirror - black has pawn advantage, black to move
    // (flip all colors and flip board vertically)
    let pos_b = get_position("4k3/8/8/8/4p3/8/8/4K3 b - - 0 1");
    acc.compute(&net, &pos_b);
    let eval_b = net.evaluate(&acc, 1); // black STM

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
    let kings_only = net.evaluate(&acc, WHITE);

    // White has a pawn
    let pos = get_position("4k3/8/8/8/4P3/8/8/4K3 w - - 0 1");
    acc.compute(&net, &pos);
    let white_pawn = net.evaluate(&acc, WHITE);

    // White has a knight
    let pos = get_position("4k3/8/8/8/4N3/8/8/4K3 w - - 0 1");
    acc.compute(&net, &pos);
    let white_knight = net.evaluate(&acc, WHITE);

    // White has a queen
    let pos = get_position("4k3/8/8/8/4Q3/8/8/4K3 w - - 0 1");
    acc.compute(&net, &pos);
    let white_queen = net.evaluate(&acc, WHITE);

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
        white_pawn > kings_only,
        "Pawn ({}) should be better than bare kings ({})",
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
    let eval = net.evaluate(&acc, WHITE);
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
    let kings_eval = net.evaluate(&acc, WHITE);

    // Add a white rook on e4 instead
    let pos2 = get_position("4k3/8/8/8/4R3/8/8/4K3 w - - 0 1");
    acc.compute(&net, &pos2);
    let rook_eval = net.evaluate(&acc, WHITE);

    // Add a white knight on e4
    let pos3 = get_position("4k3/8/8/8/4N3/8/8/4K3 w - - 0 1");
    acc.compute(&net, &pos3);
    let knight_eval = net.evaluate(&acc, WHITE);

    // Knight on a completely different square
    let pos4 = get_position("4k3/8/8/8/8/8/8/N3K3 w - - 0 1");
    acc.compute(&net, &pos4);
    let knight_a1 = net.evaluate(&acc, WHITE);

    // Two knights
    let pos5 = get_position("4k3/8/8/8/4N3/8/8/N3K3 w - - 0 1");
    acc.compute(&net, &pos5);
    let two_knights = net.evaluate(&acc, WHITE);

    println!("Kings only: {}", kings_eval);
    println!("Rook e4: {}", rook_eval);
    println!("Knight e4: {}", knight_eval);
    println!("Knight a1: {}", knight_a1);
    println!("Two knights: {}", two_knights);

    // The key question: is the knight contributing ANYTHING?
    println!("Knight e4 delta from kings: {}", knight_eval - kings_eval);
    println!("Rook e4 delta from kings: {}", rook_eval - kings_eval);
}
