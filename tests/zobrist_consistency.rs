use rusty_rival::fen::get_position;
use rusty_rival::hash::zobrist_lock;
use rusty_rival::make_move::{make_move, make_move_in_place, unmake_move};
use rusty_rival::moves::generate_moves;

/// Verify that make_move (copy) and make_move_in_place produce the same zobrist
#[test]
fn test_zobrist_consistency_between_make_and_make_in_place() {
    let fens = vec![
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R3K1R1 b Qkq - 0 1",
        "rnbqkb1r/pppppppp/5n2/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 1 2",
        "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1",
        "8/2p5/3p4/1P5r/1R2Pp2/1K4P1/7k/8 b - e3 0 1", // En passant available
    ];

    for fen in fens {
        let original = get_position(fen);
        let moves = generate_moves(&original);

        for m in moves {
            // Copy-based make_move
            let mut copy_pos = original;
            make_move(&original, m, &mut copy_pos);
            let zobrist_after_copy_make = copy_pos.zobrist_lock;
            let recalc_copy = zobrist_lock(&copy_pos);

            // In-place make_move_in_place
            let mut inplace_pos = original;
            let unmake = make_move_in_place(&mut inplace_pos, m);
            let zobrist_after_inplace_make = inplace_pos.zobrist_lock;
            let recalc_inplace = zobrist_lock(&inplace_pos);

            // Both should produce the same zobrist
            assert_eq!(
                zobrist_after_copy_make, zobrist_after_inplace_make,
                "FEN: {}, Move: {:x}\ncopy-make zobrist {:x} != inplace-make zobrist {:x}",
                fen, m, zobrist_after_copy_make, zobrist_after_inplace_make
            );

            // Both should match recalculated zobrist
            assert_eq!(
                zobrist_after_copy_make, recalc_copy,
                "FEN: {}, Move: {:x}\ncopy-make zobrist {:x} != recalculated {:x}",
                fen, m, zobrist_after_copy_make, recalc_copy
            );

            assert_eq!(
                zobrist_after_inplace_make, recalc_inplace,
                "FEN: {}, Move: {:x}\ninplace-make zobrist {:x} != recalculated {:x}",
                fen, m, zobrist_after_inplace_make, recalc_inplace
            );

            // After unmake, zobrist should match original
            unmake_move(&mut inplace_pos, m, &unmake);
            assert_eq!(
                original.zobrist_lock, inplace_pos.zobrist_lock,
                "FEN: {}, Move: {:x}\nafter unmake zobrist {:x} != original {:x}",
                fen, m, inplace_pos.zobrist_lock, original.zobrist_lock
            );
        }
    }
}

/// Verify zobrist consistency through a sequence of moves
#[test]
fn test_zobrist_consistency_through_game_sequence() {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let mut position = get_position(fen);
    let original_zobrist = position.zobrist_lock;

    // Make a few moves and unmake them
    let moves = generate_moves(&position);
    if moves.len() < 5 {
        return;
    }

    let mut unmakes = Vec::new();
    let mut made_moves = Vec::new();

    // Make 5 moves
    for i in 0..5 {
        let m = moves[i % moves.len()];
        let unmake = make_move_in_place(&mut position, m);

        // Verify zobrist is correct
        assert_eq!(position.zobrist_lock, zobrist_lock(&position), "After move {}: zobrist mismatch", i);

        unmakes.push(unmake);
        made_moves.push(m);
    }

    // Unmake all moves in reverse
    for i in (0..5).rev() {
        unmake_move(&mut position, made_moves[i], &unmakes[i]);
    }

    // Should be back to original
    assert_eq!(
        original_zobrist, position.zobrist_lock,
        "After unmake sequence: zobrist should match original"
    );
}
