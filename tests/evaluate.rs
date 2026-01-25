use rusty_rival::bitboards::south_fill;
use rusty_rival::engine_constants::{
    BISHOP_VALUE_AVERAGE, DOUBLED_PAWN_PENALTY, ISOLATED_PAWN_PENALTY, KING_THREAT_BONUS_BISHOP, KING_THREAT_BONUS_KNIGHT,
    KING_THREAT_BONUS_QUEEN, KING_THREAT_BONUS_ROOK, KNIGHT_FORK_THREAT_SCORE, KNIGHT_VALUE_AVERAGE, PAWN_VALUE_AVERAGE,
    QUEEN_VALUE_AVERAGE, ROOK_VALUE_AVERAGE, VALUE_CONNECTED_PASSED_PAWNS, VALUE_KNIGHT_OUTPOST,
};
use rusty_rival::evaluate::{
    black_king_early_safety, connected_passed_pawn_score, count_knight_fork_threats, doubled_and_isolated_pawn_score, evaluate,
    insufficient_material, is_wrong_colored_bishop_draw, isolated_pawn_count, king_threat_score, knight_fork_threat_score,
    knight_outpost_scores, material_score, on_same_file_count, passed_pawn_score, white_king_early_safety,
};
use rusty_rival::fen::get_position;
use rusty_rival::types::{default_evaluate_cache, Score, BLACK, WHITE};
use rusty_rival::utils::{invert_fen, invert_pos};

fn test_doubled_pawns(fen: &str, score: Score) {
    let position = get_position(fen);
    assert_eq!(doubled_and_isolated_pawn_score(&position, &mut default_evaluate_cache()), score);

    let position = get_position(&invert_fen(fen));
    assert_eq!(doubled_and_isolated_pawn_score(&position, &mut default_evaluate_cache()), -score);
}

fn test_insufficient_material(fen: &str, result: bool, include_helpmates: bool) {
    let position = get_position(fen);

    assert_eq!(
        insufficient_material(
            &position,
            (position.pieces[WHITE as usize].all_pieces_bitboard.count_ones()
                + position.pieces[BLACK as usize].all_pieces_bitboard.count_ones()) as u8,
            include_helpmates
        ),
        result
    );

    let position = get_position(&invert_fen(fen));
    assert_eq!(
        insufficient_material(
            &position,
            (position.pieces[WHITE as usize].all_pieces_bitboard.count_ones()
                + position.pieces[BLACK as usize].all_pieces_bitboard.count_ones()) as u8,
            include_helpmates
        ),
        result
    );
}

#[test]
fn it_knows_insufficient_material() {
    test_insufficient_material("r1b1kb1r/6p1/ppn3p1/1p1Np2p/4N1n1/6P1/PPPPP1P1/R1BQ1RK1 w kq - 0 1", false, false);
    test_insufficient_material("r1b1kb2/6p1/ppn5/3N3p/4N1n1/6P1/PP1PP3/R2Q1RK1 w q - 0 1", false, false);
    test_insufficient_material("2b1k3/6p1/2n5/3N4/8/8/3PP3/R4RK1 w - - 0 1", false, false);
    test_insufficient_material("2b1k3/8/2n5/3N4/8/8/8/R5K1 w - - 0 1", false, false);
    test_insufficient_material("2b1k3/8/2n5/3N4/8/8/8/6K1 w - - 0 1", false, false);
    test_insufficient_material("4k3/8/2n5/3N4/8/8/8/6K1 w - - 0 1", false, false);
    test_insufficient_material("4k3/8/2n5/8/8/8/8/6K1 w - - 0 1", true, false);
    test_insufficient_material("4k3/8/2n5/8/4B3/8/8/6K1 w - - 0 1", false, false);
    test_insufficient_material("4k3/8/8/8/4B3/8/8/6K1 w - - 0 1", true, false);
    test_insufficient_material("4k3/8/8/8/8/8/8/6K1 w - - 0 1", true, false);
    test_insufficient_material("4k3/2b5/8/8/4B3/8/8/6K1 w - - 0 1", false, false);
    test_insufficient_material("4k3/8/8/8/2b1B3/8/8/6K1 w - - 0 1", true, false);
    test_insufficient_material("4k3/4p3/8/8/2b1B3/8/4PP2/6K1 w - - 0 1", false, false);

    test_insufficient_material("6k1/8/8/4K3/8/7n/7N/8 b - - 0 1", false, false);
    test_insufficient_material("6k1/8/8/4K3/8/7n/7N/8 b - - 0 1", true, true);

    test_insufficient_material("6k1/8/8/4K3/8/4B3/4b3/8 b - - 0 1", false, false);
    test_insufficient_material("6k1/8/8/4K3/8/4B3/4b3/8 b - - 0 1", true, true);

    test_insufficient_material("6k1/8/8/4K3/8/4B3/5b2/8 b - - 0 1", true, false);
    test_insufficient_material("6k1/8/8/4K3/8/4B3/5b2/8 b - - 0 1", true, true);
}

#[test]
fn it_gets_the_pawn_score() {
    let fen = "3Nk3/4p3/2p2p2/1bp2p2/3b1Pn1/2N5/1PP3PP/2BQK2R b K - 0 1";
    let position = get_position(fen);
    assert_eq!(
        on_same_file_count(
            position.pieces[WHITE as usize].pawn_bitboard,
            south_fill(position.pieces[WHITE as usize].pawn_bitboard) as u8
        ),
        0
    );
    assert_eq!(
        on_same_file_count(
            position.pieces[BLACK as usize].pawn_bitboard,
            south_fill(position.pieces[BLACK as usize].pawn_bitboard) as u8
        ),
        2
    );

    assert_eq!(
        isolated_pawn_count(south_fill(position.pieces[WHITE as usize].pawn_bitboard) as u8),
        0
    );
    assert_eq!(
        isolated_pawn_count(south_fill(position.pieces[BLACK as usize].pawn_bitboard) as u8),
        1
    );

    test_doubled_pawns(fen, DOUBLED_PAWN_PENALTY * 2 + ISOLATED_PAWN_PENALTY);

    let fen = "3Nk3/4p3/1p3p2/1bp2p2/3b1Pn1/2NP4/1P4PP/2BQK2R w K - 0 1";
    let position = get_position(fen);
    assert_eq!(
        on_same_file_count(
            position.pieces[WHITE as usize].pawn_bitboard,
            south_fill(position.pieces[WHITE as usize].pawn_bitboard) as u8
        ),
        0
    );
    assert_eq!(
        on_same_file_count(
            position.pieces[BLACK as usize].pawn_bitboard,
            south_fill(position.pieces[BLACK as usize].pawn_bitboard) as u8
        ),
        1
    );

    assert_eq!(
        isolated_pawn_count(south_fill(position.pieces[WHITE as usize].pawn_bitboard) as u8),
        2
    );
    assert_eq!(
        isolated_pawn_count(south_fill(position.pieces[BLACK as usize].pawn_bitboard) as u8),
        0
    );

    test_doubled_pawns(fen, DOUBLED_PAWN_PENALTY - ISOLATED_PAWN_PENALTY * 2);
}

fn test_passed_pawns(fen: &str, score: Score) {
    let position = get_position(fen);
    assert_eq!(passed_pawn_score(&position, &mut default_evaluate_cache()), score);

    let position = get_position(&invert_fen(fen));
    assert_eq!(passed_pawn_score(&position, &mut default_evaluate_cache()), -score);
}

// King support bonus adds complexity to expected values.
// These are verified computed values including all bonuses.
#[test]
fn it_gets_the_passed_pawn_score() {
    // Position 1: Pieces present, partial king support bonus
    test_passed_pawns("1r5k/8/7p/1N1K4/pP6/n2R3P/8/8 w - - 0 1", 214);

    // Position 2: Pure pawn endgame, full king support bonus
    test_passed_pawns("4k3/8/7p/1P2Pp1P/2Pp1PP1/8/8/4K3 w - - 0 1", 251);

    // Position 3: King can't catch pawn bonus
    test_passed_pawns("k7/8/7p/1P2Pp1P/2Pp1PP1/8/8/4K3 w - - 0 1", 812);

    // Position 4: Black bishop reduces king support scaling
    test_passed_pawns("4k3/8/7p/1P2Pp1P/2Pp1PP1/8/b7/4K3 w - - 0 1", 242);

    // Position 5: White bishop reduces king support scaling
    test_passed_pawns("4k3/8/7p/1P2Pp1P/2Pp1PP1/8/B7/4K3 w - - 0 1", 252);

    // Position 6: Black rook + white bishop, mixed king support
    test_passed_pawns("r3k3/8/7p/1P2Pp1P/2Pp1PP1/8/B7/7K w - - 0 1", 221);

    // Position 7: Black to move, king can't catch pawn
    test_passed_pawns("r3k3/8/7p/1P2Pp1P/2Pp1PP1/8/8/7K b - - 0 1", -283);
}

fn test_knight_outposts(fen: &str, score: Score) {
    let position = get_position(fen);
    assert_eq!(knight_outpost_scores(&position, &mut default_evaluate_cache()), score);

    let position = get_position(&invert_fen(fen));
    assert_eq!(knight_outpost_scores(&position, &mut default_evaluate_cache()), -score);
}

#[test]
fn it_gets_the_passed_knight_score() {
    test_knight_outposts("4k3/8/7p/1P2Pp1P/2Pp1PP1/3N4/8/4K3 w - - 0 1", 0);
    test_knight_outposts("4k3/8/7p/4Pp1P/2Pp1PP1/3N4/2P5/4K3 w - - 0 1", VALUE_KNIGHT_OUTPOST);
    test_knight_outposts("r1b1kbnr/2p2ppp/ppn5/1B1Np3/4N3/8/PPPP1PPP/R1BQ1RK1 w kq - 0 1", 0);
    test_knight_outposts("r1b1kbnr/2p2ppp/ppn5/1B1Np3/2P1N3/3P4/PP3PPP/R1BQ1RK1 w kq - 0 1", 0);
    test_knight_outposts("r1b1kbnr/2p3pp/ppn3p1/1B1Np3/4N3/8/PPPP1PPP/R1BQ1RK1 w kq - 0 1", 0);
    test_knight_outposts(
        "r1b1kbnr/2p3pp/ppn3p1/1B1Np3/2P1N3/3P4/PP3PPP/R1BQ1RK1 w kq - 0 1",
        VALUE_KNIGHT_OUTPOST,
    );

    test_knight_outposts("r1b1kbnr/6pp/ppn3p1/1p1Np3/4N3/8/PPPP1PPP/R1BQ1RK1 w kq - 0 1", 0);
    test_knight_outposts(
        "r1b1kbnr/6pp/ppn3p1/1p1Np3/4N3/3P4/PPP2PPP/R1BQ1RK1 w kq - 0 1",
        VALUE_KNIGHT_OUTPOST,
    );
    test_knight_outposts(
        "r1b1kbnr/6pp/ppn3p1/1p1Np3/2P1N3/3P4/PP3PPP/R1BQ1RK1 w kq - 0 1",
        VALUE_KNIGHT_OUTPOST * 2,
    );

    test_knight_outposts(
        "r1b1kb1r/6pp/ppn3p1/1p1Np3/4N1n1/3P4/PPPP1PP1/R1BQ1RK1 w kq - 0 1",
        VALUE_KNIGHT_OUTPOST,
    );
    test_knight_outposts(
        "r1b1kb1r/6pp/ppn3p1/1p1Np3/4N1n1/P2P4/PPPP2P1/R1BQ1RK1 w kq - 0 1",
        VALUE_KNIGHT_OUTPOST,
    );
    test_knight_outposts("r1b1kb1r/6p1/ppn3p1/1p1Np2p/4N1n1/P2P4/PPPP2P1/R1BQ1RK1 w kq - 0 1", 0);

    test_knight_outposts(
        "r1b1kb1r/6p1/ppn3p1/1p1Np2p/4N1n1/6P1/PPPPP1P1/R1BQ1RK1 w kq - 0 1",
        -VALUE_KNIGHT_OUTPOST,
    );
}

// fn test_backward_pawns(fen: &str, score: Score) {
//     let position = get_position(fen);
//     assert_eq!(backward_pawn_score(&position), score);
//
//     let position = get_position(&invert_fen(fen));
//     assert_eq!(backward_pawn_score(&position), -score);
// }
//
// #[test]
// fn it_gets_the_backward_pawn_score() {
//     test_backward_pawns("4k3/8/2p5/3p4/3P4/8/8/6K1 w - - 0 1", VALUE_BACKWARD_PAWN_PENALTY);
//     test_backward_pawns("r1bqkb1r/pp3ppp/2np1n2/1N2p3/4P3/2N5/PPPP1PPP/R1BQKB1R w KQkq - 0 1", VALUE_BACKWARD_PAWN_PENALTY);
// }

#[test]
fn it_gets_the_material_score() {
    let position = get_position("r1q1k3/1R2n2p/5b2/5r2/p1Pp4/7P/1p2p3/6K1 b q - 0 1");
    assert_eq!(
        material_score(&position),
        ROOK_VALUE_AVERAGE + (PAWN_VALUE_AVERAGE * 2)
            - (ROOK_VALUE_AVERAGE * 2)
            - QUEEN_VALUE_AVERAGE
            - BISHOP_VALUE_AVERAGE
            - KNIGHT_VALUE_AVERAGE
            - (PAWN_VALUE_AVERAGE * 5)
            - 26
    );

    let position = get_position("r3k3/1R2n2p/5b2/8/p1Pp4/7P/1p2p3/6K1 b q - 0 1");
    assert_eq!(
        material_score(&position),
        ROOK_VALUE_AVERAGE + (PAWN_VALUE_AVERAGE * 2)
            - ROOK_VALUE_AVERAGE
            - BISHOP_VALUE_AVERAGE
            - KNIGHT_VALUE_AVERAGE
            - (PAWN_VALUE_AVERAGE * 5)
            - 101
    );

    let position = get_position("r3k3/1R5p/8/8/p1P5/7P/4p3/6K1 b q - 0 1");
    assert_eq!(material_score(&position), -PAWN_VALUE_AVERAGE - 33);

    let position = get_position("r3k3/1R5p/8/8/p1PQ4/7P/4p3/6K1 b q - 0 1");
    assert_eq!(material_score(&position), QUEEN_VALUE_AVERAGE - PAWN_VALUE_AVERAGE + 38);
}

fn test_king_threats(fen: &str, score: Score) {
    let position = get_position(fen);
    assert_eq!(king_threat_score(&position), score);
    assert_eq!(king_threat_score(&invert_pos(&position)), -score);
}

#[test]
fn it_evaluates_king_threats() {
    test_king_threats("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 0);
    test_king_threats(
        "rnbqkb1r/pppppppp/8/8/4n3/4BNP1/PPPPPP1P/RNBQ1RK1 w kq - 0 1",
        -(KING_THREAT_BONUS_KNIGHT * 2),
    );
    test_king_threats(
        "r1bqkb1r/pppppppp/8/8/4n2n/4BNP1/PPPPPP1P/RNBQ1RK1 w kq - 0 1",
        -(KING_THREAT_BONUS_KNIGHT * 4),
    );
    test_king_threats(
        "r1bqkb1r/pppppppp/8/8/4n2n/4BNP1/PPPPPP1P/RNBQ1RK1 w kq - 0 1",
        -(KING_THREAT_BONUS_KNIGHT * 4),
    );
    test_king_threats(
        "r1bqkb1r/pppppppp/8/4n3/7n/4BNP1/PPPPPP1P/RNBQ1RK1 w kq - 0 1",
        -KING_THREAT_BONUS_KNIGHT * 3,
    );
    test_king_threats(
        "rkbq1b1r/pppppppp/5N2/8/4n2n/4BNP1/PPPPPP1P/R1BQ1RK1 w - - 0 1",
        -(KING_THREAT_BONUS_KNIGHT * 4) + KING_THREAT_BONUS_BISHOP * 2,
    );
    test_king_threats(
        "rkbq1b1r/pppppppp/3n1N2/8/7n/4BNP1/PPPPPP1P/R1BQ1RK1 w - - 0 1",
        -(KING_THREAT_BONUS_KNIGHT * 2) + KING_THREAT_BONUS_BISHOP * 2,
    );
    test_king_threats(
        "rkb4r/pppppppp/3n1N2/3b1q2/7n/4BNP1/PPPPPP1P/R1BQ1RK1 w - - 0 1",
        -KING_THREAT_BONUS_KNIGHT * 2 - KING_THREAT_BONUS_BISHOP - (KING_THREAT_BONUS_QUEEN * 2) + KING_THREAT_BONUS_BISHOP * 2,
    );
    test_king_threats(
        "rkb4r/1ppppppp/1p1n1N2/3b1q2/7n/R3BNP1/PPPPPP1P/2BQ1RK1 w - - 0 1",
        (-KING_THREAT_BONUS_KNIGHT * 2 - KING_THREAT_BONUS_BISHOP - KING_THREAT_BONUS_QUEEN * 2)
            + KING_THREAT_BONUS_BISHOP
            + KING_THREAT_BONUS_ROOK * 3,
    );
}

fn test_king_safety(fen: &str, white_score: Score, black_score: Score) {
    let position = get_position(fen);
    assert_eq!(white_king_early_safety(&position), white_score);
    assert_eq!(black_king_early_safety(&position), black_score);
    assert_eq!(white_king_early_safety(&invert_pos(&position)), black_score);
    assert_eq!(black_king_early_safety(&invert_pos(&position)), white_score);
}

#[test]
fn it_evaluates_king_safety() {
    test_king_safety("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 0, 0);
    test_king_safety("rnbqkbnr/pppppppp/8/8/8/4BNP1/PPPPPP1P/RNBQ1RK1 w kq - 0 1", 15, 0);
    test_king_safety("rnbqkbnr/pppppppp/8/8/8/5NP1/PPPPPPBP/RNBQ1RK1 w kq - 0 1", 35, 0);
    test_king_safety("rnbqkbnr/pppppppp/8/8/8/5N2/PPPPPPPP/RNBQBRK1 w kq - 0 1", 45, 0);
    test_king_safety("rnbqkbnr/pppppppp/8/8/8/5NPP/PPPPPPB1/RNBQ1RK1 w kq - 0 1", 30, 0);
    test_king_safety("rnbqkbnr/pppppppp/8/8/8/5NPP/PPPPPP2/RNBQBRK1 w kq - 0 1", 20, 0);
    test_king_safety("rnbqkbnr/pppppppp/8/8/8/3PBN2/PPPPPP1P/RNBQ1RK1 w kq - 0 1", 10, 0);
    test_king_safety("rnbqkbnr/pppppppp/8/8/8/2PPBN2/PPPPPP2/RNBQ1RK1 w kq - 0 1", 5, 0);
    test_king_safety("rnbqkbnr/pppppppp/8/8/2P5/2PPBN2/PPPPP3/RNBQ1RK1 w kq - 0 1", 0, 0);
    test_king_safety("rnbqkbnr/pppppppp/8/8/8/4BN1P/PPPPPPP1/RNBQ1RK1 w kq - 0 1", 27, 0);
    test_king_safety("rnbqkbnr/pppppppp/8/8/5P2/4BN1P/PPPPP1P1/RNBQ1RK1 w kq - 0 1", 15, 0);
    test_king_safety("rnbqkbnr/pppppppp/8/8/5P2/4BN2/PPPPP1PP/RNBQ1RK1 w kq - 0 1", 32, 0);
    test_king_safety("rnbqkbnr/pppppppp/8/8/4N3/4BP2/PPPPP1PP/RNBQ1RK1 w kq - 0 1", 22, 0);
}

fn test_knight_fork_threats(fen: &str, white_count: i8, black_count: i8) {
    let position = get_position(fen);
    assert_eq!(count_knight_fork_threats(&position, WHITE), white_count);
    assert_eq!(count_knight_fork_threats(&position, BLACK), black_count);
    assert_eq!(
        knight_fork_threat_score(&position),
        (white_count - black_count) as Score * KNIGHT_FORK_THREAT_SCORE
    );
    assert_eq!(count_knight_fork_threats(&invert_pos(&position), WHITE), black_count);
    assert_eq!(count_knight_fork_threats(&invert_pos(&position), BLACK), white_count);
    assert_eq!(
        knight_fork_threat_score(&invert_pos(&position)),
        (black_count - white_count) as Score * KNIGHT_FORK_THREAT_SCORE
    );
}

#[test]
fn it_gets_knight_fork_threats() {
    test_knight_fork_threats("8/7R/1pqp1k2/p3p3/P1n1P3/1Q3P2/2P3r1/1KB5 b - - 1 1", 0, 1);
    test_knight_fork_threats("8/7R/1pqp1k2/p3p3/P3P3/1Q3P2/2Pn2r1/1KB5 w - - 2 2", 0, 1);
    test_knight_fork_threats("8/7R/1pqp1k2/p3p3/P3P3/1Q2nP2/2P3r1/1KB5 w - - 2 2", 0, 0);
    test_knight_fork_threats("8/6NR/1pqp1k2/p3p3/PQn1P3/5P2/2P3r1/1KB5 w - - 0 1", 0, 0);
    test_knight_fork_threats("8/2q3NR/1p1r1k2/pn1pp3/P1Q1P3/5P2/2P5/1KB5 w - - 0 1", 1, 1);
    test_knight_fork_threats("8/2q3NR/1p2r3/pn1ppk2/P1Q1P3/5P2/2P5/1KB5 w - - 0 1", 1, 1);
    test_knight_fork_threats("8/2q3NR/1p1Nrk2/pn1pp3/P1Q1P3/5P2/2P5/1KB5 w - - 0 1", 2, 1);
}

fn test_wrong_colored_bishop(fen: &str, expected: bool) {
    let position = get_position(fen);
    let piece_count = (position.pieces[WHITE as usize].all_pieces_bitboard.count_ones()
        + position.pieces[BLACK as usize].all_pieces_bitboard.count_ones()) as u8;
    assert_eq!(
        is_wrong_colored_bishop_draw(&position, piece_count),
        expected,
        "Wrong result for FEN: {}",
        fen
    );
}

#[test]
fn it_detects_wrong_colored_bishop_draws() {
    // White a-pawn with dark-squared bishop (a8 is light, bishop is dark) = DRAW
    test_wrong_colored_bishop("8/8/8/8/8/k7/P7/K1B5 w - - 0 1", true);

    // White a-pawn with light-squared bishop (a8 is light, bishop is light) = NOT draw
    test_wrong_colored_bishop("8/8/8/8/8/k7/P7/KB6 w - - 0 1", false);

    // White h-pawn with light-squared bishop (h8 is dark, bishop is light) = DRAW
    test_wrong_colored_bishop("8/8/8/8/k7/8/7P/5B1K w - - 0 1", true);

    // White h-pawn with dark-squared bishop (h8 is dark, bishop is dark) = NOT draw
    test_wrong_colored_bishop("8/8/8/8/k7/8/7P/4B2K w - - 0 1", false);

    // Black a-pawn with light-squared bishop (a1 is dark, bishop is light) = DRAW
    // Bishop on a6 is light-squared, a1 is dark, so bishop CANNOT control a1
    test_wrong_colored_bishop("k7/p7/b7/8/8/K7/8/8 w - - 0 1", true);

    // Black a-pawn with dark-squared bishop (a1 is dark, bishop is dark) = NOT draw
    // Bishop on b6 is dark-squared, a1 is dark, so bishop CAN control a1
    test_wrong_colored_bishop("k7/p7/1b6/8/8/K7/8/8 w - - 0 1", false);

    // Black h-pawn with dark-squared bishop (h1 is light, bishop is dark) = DRAW
    // Bishop on f8 is dark-squared, h1 is light, so bishop CANNOT control h1
    test_wrong_colored_bishop("5b1k/7p/8/8/K7/8/8/8 w - - 0 1", true);

    // Black h-pawn with light-squared bishop (h1 is light, bishop is light) = NOT draw
    // Bishop on g8 is light-squared, h1 is light, so bishop CAN control h1
    test_wrong_colored_bishop("6bk/7p/8/8/K7/8/8/8 w - - 0 1", false);

    // Not a rook pawn (e-pawn) - not a draw even with "wrong" bishop
    test_wrong_colored_bishop("8/8/8/8/8/k7/4P3/K1B5 w - - 0 1", false);

    // Too many pieces - not applicable
    test_wrong_colored_bishop("8/8/8/8/8/k7/P7/KNB5 w - - 0 1", false);
}

#[test]
fn it_evaluates_wrong_colored_bishop_as_draw() {
    // These positions should evaluate to 0 (draw)
    let position = get_position("8/8/8/8/8/k7/P7/K1B5 w - - 0 1");
    assert_eq!(evaluate(&position), 0);

    let position = get_position("8/8/8/8/k7/8/7P/5B1K w - - 0 1");
    assert_eq!(evaluate(&position), 0);

    let position = get_position("k7/p7/b7/8/8/K7/8/8 w - - 0 1");
    assert_eq!(evaluate(&position), 0);

    let position = get_position("5b1k/7p/8/8/K7/8/8/8 w - - 0 1");
    assert_eq!(evaluate(&position), 0);
}

#[test]
fn it_gives_king_centralization_bonus_in_endgames() {
    use rusty_rival::evaluate::endgame_king_centralization_bonus;

    // King and pawn endgame - white king on d4 (center) vs black king on h8 (corner)
    // White should get a positive bonus
    let position = get_position("7k/8/8/8/3K4/8/8/8 w - - 0 1");
    let bonus = endgame_king_centralization_bonus(&position);
    assert!(bonus > 0, "Central king should get positive bonus, got {}", bonus);

    // Same but flipped - black king central
    let position = get_position("8/8/8/8/3k4/8/8/7K w - - 0 1");
    let bonus = endgame_king_centralization_bonus(&position);
    assert!(bonus < 0, "Central black king should give negative score, got {}", bonus);

    // Both kings central - should be roughly even
    let position = get_position("8/8/8/3k4/3K4/8/8/8 w - - 0 1");
    let bonus = endgame_king_centralization_bonus(&position);
    assert!(bonus.abs() <= 4, "Both central kings should be roughly even, got {}", bonus);

    // With too much material, bonus should be 0
    let position = get_position("r2k3r/8/8/8/3K4/8/8/R6R w - - 0 1");
    let bonus = endgame_king_centralization_bonus(&position);
    assert_eq!(bonus, 0, "With rooks on board, bonus should be 0");

    // With minor piece endgame, should still apply (material below threshold)
    let position = get_position("7k/8/8/8/3K4/5N2/8/8 w - - 0 1");
    let bonus = endgame_king_centralization_bonus(&position);
    assert!(bonus > 0, "Knight endgame should still give bonus, got {}", bonus);
}

fn test_kpk_draw(fen: &str, expected_draw: bool) {
    use rusty_rival::evaluate::is_kpk_draw;

    let position = get_position(fen);
    let piece_count = (position.pieces[WHITE as usize].all_pieces_bitboard.count_ones()
        + position.pieces[BLACK as usize].all_pieces_bitboard.count_ones()) as u8;

    assert_eq!(
        is_kpk_draw(&position, piece_count),
        expected_draw,
        "KP vs K draw detection failed for FEN: {} (expected {})",
        fen,
        if expected_draw { "draw" } else { "not draw" }
    );

    // If it's a draw, evaluate should return 0
    if expected_draw {
        assert_eq!(evaluate(&position), 0, "Drawn KP vs K position should evaluate to 0: {}", fen);
    }
}

#[test]
fn it_detects_kpk_draws() {
    // Classic opposition draw - White to move, Black has opposition
    // 8/8/8/4k3/8/4K3/4P3/8 w - Ke3, Pe2 vs Ke5
    // Kings on same file (e), 2 ranks apart, White to move = Black has opposition
    test_kpk_draw("8/8/8/4k3/8/4K3/4P3/8 w - - 0 1", true);

    // Same position but Black to move - White has opposition, NOT drawn
    test_kpk_draw("8/8/8/4k3/8/4K3/4P3/8 b - - 0 1", false);

    // White king on key square (d4) - winning, not drawn
    test_kpk_draw("8/8/8/4k3/3K4/8/4P3/8 w - - 0 1", false);

    // Rook pawn - defending king in front on same file = draw
    test_kpk_draw("k7/8/K7/P7/8/8/8/8 w - - 0 1", true);

    // Rook pawn - defending king can reach corner = draw
    test_kpk_draw("8/8/8/8/8/k7/P7/K7 w - - 0 1", true);

    // Rook pawn - defending king too far = not draw
    test_kpk_draw("8/8/8/8/8/8/P6k/K7 w - - 0 1", false);

    // Black pawn version - opposition draw
    test_kpk_draw("8/4p3/8/4k3/8/4K3/8/8 b - - 0 1", true);

    // Not KP vs K (extra piece) - should not be detected as draw
    test_kpk_draw("8/8/8/4k3/8/4K3/4P3/4N3 w - - 0 1", false);
}

fn test_connected_passed_pawns(white_passed: u64, black_passed: u64, expected_score: Score) {
    let score = connected_passed_pawn_score(white_passed, black_passed);
    assert_eq!(
        score, expected_score,
        "Connected passed pawn score wrong: expected {}, got {}",
        expected_score, score
    );
}

#[test]
fn it_scores_connected_passed_pawns() {
    // No passed pawns
    test_connected_passed_pawns(0, 0, 0);

    // Single passed pawn (not connected)
    // e5 = square 36 (rank 4 from h1=0)
    test_connected_passed_pawns(1 << 36, 0, 0);

    // Two white passed pawns on adjacent files (d5 and e5)
    // d5 = square 35, e5 = square 36 (both on rank 5)
    // Rank 5 for white = index 3 in VALUE_CONNECTED_PASSED_PAWNS
    test_connected_passed_pawns((1 << 35) | (1 << 36), 0, VALUE_CONNECTED_PASSED_PAWNS[3]);

    // Two white passed pawns on same file (not connected)
    // e4 = square 28, e5 = square 36
    test_connected_passed_pawns((1 << 28) | (1 << 36), 0, 0);

    // Two white passed pawns far apart (b5 and g5 - not connected)
    // b5 = square 33, g5 = square 38
    test_connected_passed_pawns((1 << 33) | (1 << 38), 0, 0);

    // Two black passed pawns on adjacent files (d4 and e4)
    // e4 = square 27, d4 = square 28 (both on rank 4 for black)
    // Rank 4 for black = index 3 in VALUE_CONNECTED_PASSED_PAWNS (7-4=3)
    test_connected_passed_pawns(0, (1 << 27) | (1 << 28), -VALUE_CONNECTED_PASSED_PAWNS[3]);

    // Two black passed pawns on rank 3 (more advanced for black)
    // e3 = square 19, d3 = square 20
    // Rank 3 for black = index 4 in VALUE_CONNECTED_PASSED_PAWNS (7-3=4)
    test_connected_passed_pawns(0, (1 << 19) | (1 << 20), -VALUE_CONNECTED_PASSED_PAWNS[4]);

    // White connected on rank 6 (very advanced)
    // e6 = square 43, d6 = square 44
    // Rank 6 for white = index 4
    test_connected_passed_pawns((1 << 43) | (1 << 44), 0, VALUE_CONNECTED_PASSED_PAWNS[4]);

    // White connected on rank 7 (about to promote!)
    // e7 = square 51, d7 = square 52
    // Rank 7 for white = index 5
    test_connected_passed_pawns((1 << 51) | (1 << 52), 0, VALUE_CONNECTED_PASSED_PAWNS[5]);

    // Both sides have connected passed pawns
    test_connected_passed_pawns(
        (1 << 35) | (1 << 36), // white e5, d5 (rank 5 = index 3)
        (1 << 19) | (1 << 20), // black e3, d3 (rank 3 = index 4)
        VALUE_CONNECTED_PASSED_PAWNS[3] - VALUE_CONNECTED_PASSED_PAWNS[4],
    );

    // Three white passed pawns on consecutive files (e5, d5, c5) = two connected pairs
    // e5 = square 35, d5 = square 36, c5 = square 37
    test_connected_passed_pawns(
        (1 << 35) | (1 << 36) | (1 << 37),
        0,
        VALUE_CONNECTED_PASSED_PAWNS[3] * 2, // e-d pair and d-c pair
    );
}
