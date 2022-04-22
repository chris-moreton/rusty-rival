use rusty_rival::bitboards::south_fill;
use rusty_rival::engine_constants::{
    BISHOP_VALUE_AVERAGE, DOUBLED_PAWN_PENALTY, ISOLATED_PAWN_PENALTY, KING_THREAT_BONUS_BISHOP, KING_THREAT_BONUS_KNIGHT,
    KING_THREAT_BONUS_QUEEN, KNIGHT_FORK_THREAT_SCORE, KNIGHT_VALUE_AVERAGE, PAWN_VALUE_AVERAGE, QUEEN_VALUE_AVERAGE, ROOK_VALUE_AVERAGE,
    VALUE_GUARDED_PASSED_PAWN, VALUE_KING_CANNOT_CATCH_PAWN, VALUE_KING_DISTANCE_PASSED_PAWN_MULTIPLIER, VALUE_KNIGHT_OUTPOST,
    VALUE_PASSED_PAWN_BONUS,
};
use rusty_rival::evaluate::{
    black_king_early_safety, count_knight_fork_threats, doubled_and_isolated_pawn_score, insufficient_material, isolated_pawn_count,
    king_threat_score, knight_fork_threat_score, knight_outpost_scores, material_score, on_same_file_count, passed_pawn_score,
    white_king_early_safety,
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

fn test_insufficient_material(fen: &str, result: bool) {
    let mut cache = default_evaluate_cache();

    let position = get_position(fen);

    assert_eq!(insufficient_material(&position, (position.pieces[WHITE as usize].all_pieces_bitboard.count_ones()
        + position.pieces[BLACK as usize].all_pieces_bitboard.count_ones()) as u8, false), result);

    let position = get_position(&invert_fen(fen));
    cache.piece_count = (position.pieces[WHITE as usize].all_pieces_bitboard.count_ones()
        + position.pieces[BLACK as usize].all_pieces_bitboard.count_ones()) as u8;
    assert_eq!(insufficient_material(&position, (position.pieces[WHITE as usize].all_pieces_bitboard.count_ones()
        + position.pieces[BLACK as usize].all_pieces_bitboard.count_ones()) as u8, false), result);
}

#[test]
fn it_knows_insufficient_material() {
    test_insufficient_material("r1b1kb1r/6p1/ppn3p1/1p1Np2p/4N1n1/6P1/PPPPP1P1/R1BQ1RK1 w kq - 0 1", false);
    test_insufficient_material("r1b1kb2/6p1/ppn5/3N3p/4N1n1/6P1/PP1PP3/R2Q1RK1 w q - 0 1", false);
    test_insufficient_material("2b1k3/6p1/2n5/3N4/8/8/3PP3/R4RK1 w - - 0 1", false);
    test_insufficient_material("2b1k3/8/2n5/3N4/8/8/8/R5K1 w - - 0 1", false);
    test_insufficient_material("2b1k3/8/2n5/3N4/8/8/8/6K1 w - - 0 1", false);
    test_insufficient_material("4k3/8/2n5/3N4/8/8/8/6K1 w - - 0 1", false);
    test_insufficient_material("4k3/8/2n5/8/8/8/8/6K1 w - - 0 1", true);
    test_insufficient_material("4k3/8/2n5/8/4B3/8/8/6K1 w - - 0 1", false);
    test_insufficient_material("4k3/8/8/8/4B3/8/8/6K1 w - - 0 1", true);
    test_insufficient_material("4k3/8/8/8/8/8/8/6K1 w - - 0 1", true);
    test_insufficient_material("4k3/2b5/8/8/4B3/8/8/6K1 w - - 0 1", false);
    test_insufficient_material("4k3/8/8/8/2b1B3/8/8/6K1 w - - 0 1", true);
    test_insufficient_material("4k3/4p3/8/8/2b1B3/8/4PP2/6K1 w - - 0 1", false);
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

#[test]
fn it_gets_the_passed_pawn_score() {
    test_passed_pawns(
        "1r5k/8/7p/1N1K4/pP6/n2R3P/8/8 w - - 0 1",
        242 + // bonus adjustment based on game stage
            (5 * VALUE_KING_DISTANCE_PASSED_PAWN_MULTIPLIER) -
            (6 * VALUE_KING_DISTANCE_PASSED_PAWN_MULTIPLIER) +
            VALUE_PASSED_PAWN_BONUS[2] - // white pawn on 4th
            VALUE_PASSED_PAWN_BONUS[3], // black pawn on 5th
    );

    test_passed_pawns(
        "4k3/8/7p/1P2Pp1P/2Pp1PP1/8/8/4K3 w - - 0 1",
        (5 * VALUE_KING_DISTANCE_PASSED_PAWN_MULTIPLIER) -
                          VALUE_KING_DISTANCE_PASSED_PAWN_MULTIPLIER +
               (VALUE_PASSED_PAWN_BONUS[3] * 2 + // white pawns on 5th
                VALUE_GUARDED_PASSED_PAWN * 2 + // two guarded passed pawns
                VALUE_PASSED_PAWN_BONUS[2]) - // white pawn on 4th
                   (VALUE_PASSED_PAWN_BONUS[3]), // black pawn on 5th
    );
    test_passed_pawns(
        "k7/8/7p/1P2Pp1P/2Pp1PP1/8/8/4K3 w - - 0 1",
        VALUE_KING_CANNOT_CATCH_PAWN + (2 * VALUE_KING_DISTANCE_PASSED_PAWN_MULTIPLIER) +
                          (4 * VALUE_KING_DISTANCE_PASSED_PAWN_MULTIPLIER) + VALUE_KING_DISTANCE_PASSED_PAWN_MULTIPLIER -
                          VALUE_KING_DISTANCE_PASSED_PAWN_MULTIPLIER +
                      (VALUE_PASSED_PAWN_BONUS[3] * 2 + // white pawns on 5th
                          VALUE_GUARDED_PASSED_PAWN * 2 + // two guarded passed pawns
                          VALUE_PASSED_PAWN_BONUS[2]) - // white pawn on 4th
                          (VALUE_PASSED_PAWN_BONUS[3]), // black pawn on 5th
    );
    test_passed_pawns(
        "4k3/8/7p/1P2Pp1P/2Pp1PP1/8/b7/4K3 w - - 0 1",
        -4 + // bonus adjustment based on game stage
                      (5 * VALUE_KING_DISTANCE_PASSED_PAWN_MULTIPLIER) -
                          VALUE_KING_DISTANCE_PASSED_PAWN_MULTIPLIER +
                          (VALUE_PASSED_PAWN_BONUS[3] * 2 + // white pawns on 5th
                              VALUE_GUARDED_PASSED_PAWN * 2 + // two guarded passed pawns
                              VALUE_PASSED_PAWN_BONUS[2]) - // white pawn on 4th
                          (VALUE_PASSED_PAWN_BONUS[3]), // black pawn on 5th
    );
    test_passed_pawns(
        "4k3/8/7p/1P2Pp1P/2Pp1PP1/8/B7/4K3 w - - 0 1",
        (5 * VALUE_KING_DISTANCE_PASSED_PAWN_MULTIPLIER) -
                          VALUE_KING_DISTANCE_PASSED_PAWN_MULTIPLIER +
                          (VALUE_PASSED_PAWN_BONUS[3] * 2 + // white pawns on 5th
                              VALUE_GUARDED_PASSED_PAWN * 2 + // two guarded passed pawns
                              VALUE_PASSED_PAWN_BONUS[2]) - // white pawn on 4th
                          (VALUE_PASSED_PAWN_BONUS[3]), // black pawn on 5th
    );
    test_passed_pawns(
        "r3k3/8/7p/1P2Pp1P/2Pp1PP1/8/B7/7K w - - 0 1",
        -3 + // bonus adjustment based on game stage
                      (5 * VALUE_KING_DISTANCE_PASSED_PAWN_MULTIPLIER) -
                          (4 * VALUE_KING_DISTANCE_PASSED_PAWN_MULTIPLIER) +
                          (VALUE_PASSED_PAWN_BONUS[3] * 2 + // white pawns on 5th
                              VALUE_GUARDED_PASSED_PAWN * 2 + // two guarded passed pawns
                              VALUE_PASSED_PAWN_BONUS[2]) - // white pawn on 4th
                          (VALUE_PASSED_PAWN_BONUS[3]), // black pawn on 5th
    );
    test_passed_pawns(
        "r3k3/8/7p/1P2Pp1P/2Pp1PP1/8/8/7K b - - 0 1",
        -6 + // bonus adjustment based on game stage
                      -VALUE_KING_CANNOT_CATCH_PAWN +
                      (5 * VALUE_KING_DISTANCE_PASSED_PAWN_MULTIPLIER) -
                          (4 * VALUE_KING_DISTANCE_PASSED_PAWN_MULTIPLIER) +
                          (VALUE_PASSED_PAWN_BONUS[3] * 2 + // white pawns on 5th
                              VALUE_GUARDED_PASSED_PAWN * 2 + // two guarded passed pawns
                              VALUE_PASSED_PAWN_BONUS[2]) - // white pawn on 4th
                          (VALUE_PASSED_PAWN_BONUS[3]), // black pawn on 5th
    );
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
        -(KING_THREAT_BONUS_KNIGHT * 2 + KING_THREAT_BONUS_BISHOP * 2),
    );
    test_king_threats(
        "rkbq1b1r/pppppppp/3n1N2/8/7n/4BNP1/PPPPPP1P/R1BQ1RK1 w - - 0 1",
        -KING_THREAT_BONUS_BISHOP * 2,
    );
    test_king_threats(
        "rkb4r/pppppppp/3n1N2/3b1q2/7n/4BNP1/PPPPPP1P/R1BQ1RK1 w - - 0 1",
        -KING_THREAT_BONUS_KNIGHT * 3,
    );
    test_king_threats(
        "rkb4r/1ppppppp/1p1n1N2/3b1q2/7n/R3BNP1/PPPPPP1P/2BQ1RK1 w - - 0 1",
        (-KING_THREAT_BONUS_KNIGHT * 2 - KING_THREAT_BONUS_BISHOP - KING_THREAT_BONUS_QUEEN * 2) + KING_THREAT_BONUS_BISHOP,
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
