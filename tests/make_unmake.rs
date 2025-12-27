use rusty_rival::fen::{get_fen, get_position};
use rusty_rival::make_move::{make_move_in_place, unmake_move};
use rusty_rival::utils::hydrate_move_from_algebraic_move;

/// Test helper: make a move, verify result, unmake, verify restoration
fn test_make_unmake(fen: &str, move_str: &str, expected_fen_after: &str) {
    let original = get_position(&fen.to_string());
    let mut position = get_position(&fen.to_string());

    let mv = hydrate_move_from_algebraic_move(&position, move_str.to_string());
    let unmake_info = make_move_in_place(&mut position, mv);

    let after_fen = get_fen(&position);
    assert_eq!(
        expected_fen_after, after_fen,
        "After move {}: expected {} but got {}", move_str, expected_fen_after, after_fen
    );

    unmake_move(&mut position, mv, &unmake_info);

    let restored_fen = get_fen(&position);
    assert_eq!(
        fen, restored_fen,
        "After unmake {}: expected {} but got {}", move_str, fen, restored_fen
    );

    // Also verify bitboards match
    assert_eq!(original.pieces[0].all_pieces_bitboard, position.pieces[0].all_pieces_bitboard,
        "White all_pieces mismatch after unmake");
    assert_eq!(original.pieces[1].all_pieces_bitboard, position.pieces[1].all_pieces_bitboard,
        "Black all_pieces mismatch after unmake");
    assert_eq!(original.pieces[0].pawn_bitboard, position.pieces[0].pawn_bitboard,
        "White pawn_bitboard mismatch after unmake");
    assert_eq!(original.pieces[1].pawn_bitboard, position.pieces[1].pawn_bitboard,
        "Black pawn_bitboard mismatch after unmake");
}

// ============ PAWN MOVES ============

#[test]
fn test_simple_pawn_push() {
    test_make_unmake(
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "e2e3",
        "rnbqkbnr/pppppppp/8/8/8/4P3/PPPP1PPP/RNBQKBNR b KQkq - 0 1"
    );
}

#[test]
fn test_double_pawn_push_white() {
    test_make_unmake(
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "e2e4",
        "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1"
    );
}

#[test]
fn test_double_pawn_push_black() {
    test_make_unmake(
        "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1",
        "d7d5",
        "rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2"
    );
}

#[test]
fn test_pawn_capture() {
    test_make_unmake(
        "rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2",
        "e4d5",
        "rnbqkbnr/ppp1pppp/8/3P4/8/8/PPPP1PPP/RNBQKBNR b KQkq - 0 2"
    );
}

// ============ EN PASSANT ============

#[test]
fn test_en_passant_capture_white() {
    test_make_unmake(
        "rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3",
        "e5f6",
        "rnbqkbnr/ppp1p1pp/5P2/3p4/8/8/PPPP1PPP/RNBQKBNR b KQkq - 0 3"
    );
}

#[test]
fn test_en_passant_capture_black() {
    test_make_unmake(
        "rnbqkbnr/pppp1ppp/8/8/3Pp3/8/PPP1PPPP/RNBQKBNR b KQkq d3 0 2",
        "e4d3",
        "rnbqkbnr/pppp1ppp/8/8/8/3p4/PPP1PPPP/RNBQKBNR w KQkq - 0 3"
    );
}

// ============ PROMOTIONS ============

#[test]
fn test_promotion_to_queen() {
    test_make_unmake(
        "rnbqkbn1/pppppppP/8/8/8/8/PPPPPPP1/RNBQKBNR w KQq - 0 1",
        "h7h8q",
        "rnbqkbnQ/ppppppp1/8/8/8/8/PPPPPPP1/RNBQKBNR b KQq - 0 1"
    );
}

#[test]
fn test_promotion_to_rook() {
    test_make_unmake(
        "rnbqkbn1/pppppppP/8/8/8/8/PPPPPPP1/RNBQKBNR w KQq - 0 1",
        "h7h8r",
        "rnbqkbnR/ppppppp1/8/8/8/8/PPPPPPP1/RNBQKBNR b KQq - 0 1"
    );
}

#[test]
fn test_promotion_to_bishop() {
    test_make_unmake(
        "rnbqkbn1/pppppppP/8/8/8/8/PPPPPPP1/RNBQKBNR w KQq - 0 1",
        "h7h8b",
        "rnbqkbnB/ppppppp1/8/8/8/8/PPPPPPP1/RNBQKBNR b KQq - 0 1"
    );
}

#[test]
fn test_promotion_to_knight() {
    test_make_unmake(
        "rnbqkbn1/pppppppP/8/8/8/8/PPPPPPP1/RNBQKBNR w KQq - 0 1",
        "h7h8n",
        "rnbqkbnN/ppppppp1/8/8/8/8/PPPPPPP1/RNBQKBNR b KQq - 0 1"
    );
}

#[test]
fn test_promotion_with_capture() {
    test_make_unmake(
        "rnbqkbnr/pppppppP/8/8/8/8/PPPPPPP1/RNBQKBNR w KQkq - 0 1",
        "h7g8q",
        "rnbqkbQr/ppppppp1/8/8/8/8/PPPPPPP1/RNBQKBNR b KQkq - 0 1"
    );
}

#[test]
fn test_black_promotion_to_queen() {
    // Note: landing on h1 clears white's K castle flag
    test_make_unmake(
        "rnbqkbnr/ppppppp1/8/8/8/8/PPPPPPPp/RNBQKBN1 b KQkq - 0 1",
        "h2h1q",
        "rnbqkbnr/ppppppp1/8/8/8/8/PPPPPPP1/RNBQKBNq w Qkq - 0 2"
    );
}

#[test]
fn test_black_promotion_to_knight() {
    // Note: landing on h1 clears white's K castle flag
    test_make_unmake(
        "rnbqkbnr/ppppppp1/8/8/8/8/PPPPPPPp/RNBQKBN1 b KQkq - 0 1",
        "h2h1n",
        "rnbqkbnr/ppppppp1/8/8/8/8/PPPPPPP1/RNBQKBNn w Qkq - 0 2"
    );
}

// ============ CASTLING ============

#[test]
fn test_white_kingside_castle() {
    test_make_unmake(
        "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1",
        "e1g1",
        "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R4RK1 b kq - 1 1"
    );
}

#[test]
fn test_white_queenside_castle() {
    test_make_unmake(
        "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1",
        "e1c1",
        "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/2KR3R b kq - 1 1"
    );
}

#[test]
fn test_black_kingside_castle() {
    test_make_unmake(
        "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R b KQkq - 0 1",
        "e8g8",
        "r4rk1/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQ - 1 2"
    );
}

#[test]
fn test_black_queenside_castle() {
    test_make_unmake(
        "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R b KQkq - 0 1",
        "e8c8",
        "2kr3r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQ - 1 2"
    );
}

// ============ KING MOVES ============

#[test]
fn test_king_move() {
    // King move from a position where e2 is empty
    test_make_unmake(
        "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1",
        "e1e2",
        "rnbqkbnr/pppppppp/8/8/4P3/8/PPPPKPPP/RNBQ1BNR b kq - 1 1"
    );
}

#[test]
fn test_king_capture() {
    test_make_unmake(
        "rnbqkbnr/pppp1ppp/8/4p3/4K3/8/PPPP1PPP/RNBQ1BNR w kq - 0 2",
        "e4e5",
        "rnbqkbnr/pppp1ppp/8/4K3/8/8/PPPP1PPP/RNBQ1BNR b kq - 0 2"
    );
}

// ============ PIECE MOVES ============

#[test]
fn test_knight_move() {
    test_make_unmake(
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "g1f3",
        "rnbqkbnr/pppppppp/8/8/8/5N2/PPPPPPPP/RNBQKB1R b KQkq - 1 1"
    );
}

#[test]
fn test_knight_capture() {
    test_make_unmake(
        "rnbqkbnr/pppp1ppp/8/4p3/8/5N2/PPPPPPPP/RNBQKB1R w KQkq - 0 2",
        "f3e5",
        "rnbqkbnr/pppp1ppp/8/4N3/8/8/PPPPPPPP/RNBQKB1R b KQkq - 0 2"
    );
}

#[test]
fn test_bishop_move() {
    test_make_unmake(
        "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2",
        "f1c4",
        "rnbqkbnr/pppp1ppp/8/4p3/2B1P3/8/PPPP1PPP/RNBQK1NR b KQkq - 1 2"
    );
}

#[test]
fn test_rook_move() {
    test_make_unmake(
        "r3kbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1",
        "a8a4",
        "4kbnr/pppppppp/8/8/r7/8/PPPPPPPP/RNBQKBNR w KQk - 1 2"
    );
}

#[test]
fn test_queen_move() {
    test_make_unmake(
        "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2",
        "d1h5",
        "rnbqkbnr/pppp1ppp/8/4p2Q/4P3/8/PPPP1PPP/RNB1KBNR b KQkq - 1 2"
    );
}

#[test]
fn test_queen_capture() {
    test_make_unmake(
        "rnbqkbnr/pppp1ppp/8/4p2Q/4P3/8/PPPP1PPP/RNB1KBNR w KQkq - 0 2",
        "h5e5",
        "rnbqkbnr/pppp1ppp/8/4Q3/4P3/8/PPPP1PPP/RNB1KBNR b KQkq - 0 2"
    );
}

// ============ CASTLE FLAG CHANGES ============

#[test]
fn test_rook_move_clears_castle_flag() {
    test_make_unmake(
        "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1",
        "h1g1",
        "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K1R1 b Qkq - 1 1"
    );
}

#[test]
fn test_king_move_clears_both_castle_flags() {
    test_make_unmake(
        "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1",
        "e1d1",
        "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R2K3R b kq - 1 1"
    );
}

// ============ HALF MOVE CLOCK ============

#[test]
fn test_pawn_move_resets_halfmove() {
    test_make_unmake(
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 5 3",
        "e2e4",
        "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 3"
    );
}

#[test]
fn test_capture_resets_halfmove() {
    test_make_unmake(
        "rnbqkbnr/pppp1ppp/8/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 5 3",
        "f3e5",
        "rnbqkbnr/pppp1ppp/8/4N3/4P3/8/PPPP1PPP/RNBQKB1R b KQkq - 0 3"
    );
}

#[test]
fn test_piece_move_increments_halfmove() {
    test_make_unmake(
        "rnbqkbnr/pppppppp/8/8/8/5N2/PPPPPPPP/RNBQKB1R w KQkq - 5 3",
        "f3g5",
        "rnbqkbnr/pppppppp/8/6N1/8/8/PPPPPPPP/RNBQKB1R b KQkq - 6 3"
    );
}
