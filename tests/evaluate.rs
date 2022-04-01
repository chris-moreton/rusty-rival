use rusty_rival::bitboards::south_fill;
use rusty_rival::engine_constants::{BISHOP_VALUE, KNIGHT_VALUE, PAWN_VALUE, QUEEN_VALUE, ROOK_VALUE, VALUE_KING_CANNOT_CATCH_PAWN, VALUE_KING_DISTANCE_PASSED_PAWN_MULTIPLIER};
use rusty_rival::evaluate::{on_same_file_count, material, material_score, doubled_and_isolated_pawn_score, isolated_pawn_count, white_king_early_safety, black_king_early_safety, passed_pawn_score, DOUBLED_PAWN_PENALTY, VALUE_PASSED_PAWN_BONUS, VALUE_GUARDED_PASSED_PAWN, ISOLATED_PAWN_PENALTY, king_threat_score, KING_THREAT_BONUS, VALUE_KNIGHT_OUTPOST, knight_outpost_scores};
use rusty_rival::fen::get_position;
use rusty_rival::types::{BLACK, default_evaluate_cache, Score, WHITE};
use rusty_rival::utils::{invert_fen, invert_pos};

fn test_doubled_pawns(fen: &str, score: Score) {
    let position = get_position(fen);
    assert_eq!(doubled_and_isolated_pawn_score(&position, &mut default_evaluate_cache()), score);

    let position = get_position(&invert_fen(fen));
    assert_eq!(doubled_and_isolated_pawn_score(&position, &mut default_evaluate_cache()), -score);
}

#[test]
fn it_gets_the_pawn_score() {
    let fen = "3Nk3/4p3/2p2p2/1bp2p2/3b1Pn1/2N5/1PP3PP/2BQK2R b K - 0 1";
    let position = get_position(fen);
    assert_eq!(on_same_file_count(position.pieces[WHITE as usize].pawn_bitboard, south_fill(position.pieces[WHITE as usize].pawn_bitboard) as u8), 0);
    assert_eq!(on_same_file_count(position.pieces[BLACK as usize].pawn_bitboard, south_fill(position.pieces[BLACK as usize].pawn_bitboard) as u8), 2);

    assert_eq!(isolated_pawn_count(south_fill(position.pieces[WHITE as usize].pawn_bitboard) as u8), 0);
    assert_eq!(isolated_pawn_count(south_fill(position.pieces[BLACK as usize].pawn_bitboard) as u8), 1);

    test_doubled_pawns(fen, DOUBLED_PAWN_PENALTY * 2 + ISOLATED_PAWN_PENALTY);

    let fen = "3Nk3/4p3/1p3p2/1bp2p2/3b1Pn1/2NP4/1P4PP/2BQK2R w K - 0 1";
    let position = get_position(fen);
    assert_eq!(on_same_file_count(position.pieces[WHITE as usize].pawn_bitboard, south_fill(position.pieces[WHITE as usize].pawn_bitboard) as u8), 0);
    assert_eq!(on_same_file_count(position.pieces[BLACK as usize].pawn_bitboard, south_fill(position.pieces[BLACK as usize].pawn_bitboard) as u8), 1);

    assert_eq!(isolated_pawn_count(south_fill(position.pieces[WHITE as usize].pawn_bitboard) as u8), 2);
    assert_eq!(isolated_pawn_count(south_fill(position.pieces[BLACK as usize].pawn_bitboard) as u8), 0);

    test_doubled_pawns(fen, DOUBLED_PAWN_PENALTY - ISOLATED_PAWN_PENALTY * 2);
}

#[test]
fn it_gets_material_advantage() {
    let position = get_position("3Nk3/4p3/2p2p2/1bp2p2/3b1Pn1/2N5/1PP3PP/2BQK2R b K - 0 1");

    assert_eq!(material(&position.pieces[WHITE as usize]), (KNIGHT_VALUE * 2) + (PAWN_VALUE * 5) + BISHOP_VALUE + QUEEN_VALUE + ROOK_VALUE);
    assert_eq!(material(&position.pieces[BLACK as usize]), KNIGHT_VALUE + (PAWN_VALUE * 5) + (BISHOP_VALUE * 2));

    let position = get_position("r1q1k3/1R2n2p/5b2/5r2/p1Pp4/7P/1p2p3/6K1 b q - 0 1");
    assert_eq!(material_score(&position), ROOK_VALUE + (PAWN_VALUE *2) - (ROOK_VALUE * 2) - QUEEN_VALUE - BISHOP_VALUE - KNIGHT_VALUE - (PAWN_VALUE * 5));
}

fn test_passed_pawns(fen: &str, score: Score) {
    let position = get_position(fen);
    assert_eq!(passed_pawn_score(&position, &mut default_evaluate_cache()), score);

    let position = get_position(&invert_fen(fen));
    assert_eq!(passed_pawn_score(&position, &mut default_evaluate_cache()), -score);
}

#[test]
fn it_gets_the_passed_pawn_score() {
    //assert_eq!(passed_pawn_wrapper(START_POS), 0);
    test_passed_pawns("4k3/8/7p/1P2Pp1P/2Pp1PP1/8/8/4K3 w - - 0 1",
                      (5 * VALUE_KING_DISTANCE_PASSED_PAWN_MULTIPLIER) -
                          VALUE_KING_DISTANCE_PASSED_PAWN_MULTIPLIER +
               (VALUE_PASSED_PAWN_BONUS[3] * 2 + // white pawns on 5th
                VALUE_GUARDED_PASSED_PAWN * 2 + // two guarded passed pawns
                VALUE_PASSED_PAWN_BONUS[2]) - // white pawn on 4th
                   (VALUE_PASSED_PAWN_BONUS[3]) // black pawn on 5th
    );
    test_passed_pawns("k7/8/7p/1P2Pp1P/2Pp1PP1/8/8/4K3 w - - 0 1",
                      VALUE_KING_CANNOT_CATCH_PAWN + (2 * VALUE_KING_DISTANCE_PASSED_PAWN_MULTIPLIER) +
                          (4 * VALUE_KING_DISTANCE_PASSED_PAWN_MULTIPLIER) + VALUE_KING_DISTANCE_PASSED_PAWN_MULTIPLIER -
                          VALUE_KING_DISTANCE_PASSED_PAWN_MULTIPLIER +
                      (VALUE_PASSED_PAWN_BONUS[3] * 2 + // white pawns on 5th
                          VALUE_GUARDED_PASSED_PAWN * 2 + // two guarded passed pawns
                          VALUE_PASSED_PAWN_BONUS[2]) - // white pawn on 4th
                          (VALUE_PASSED_PAWN_BONUS[3]) // black pawn on 5th
    );
    test_passed_pawns("4k3/8/7p/1P2Pp1P/2Pp1PP1/8/b7/4K3 w - - 0 1", -4 + // bonus adjustment based on game stage
                      (5 * VALUE_KING_DISTANCE_PASSED_PAWN_MULTIPLIER) -
                          VALUE_KING_DISTANCE_PASSED_PAWN_MULTIPLIER +
                          (VALUE_PASSED_PAWN_BONUS[3] * 2 + // white pawns on 5th
                              VALUE_GUARDED_PASSED_PAWN * 2 + // two guarded passed pawns
                              VALUE_PASSED_PAWN_BONUS[2]) - // white pawn on 4th
                          (VALUE_PASSED_PAWN_BONUS[3]) // black pawn on 5th
    );
    test_passed_pawns("4k3/8/7p/1P2Pp1P/2Pp1PP1/8/B7/4K3 w - - 0 1",
                      (5 * VALUE_KING_DISTANCE_PASSED_PAWN_MULTIPLIER) -
                          VALUE_KING_DISTANCE_PASSED_PAWN_MULTIPLIER +
                          (VALUE_PASSED_PAWN_BONUS[3] * 2 + // white pawns on 5th
                              VALUE_GUARDED_PASSED_PAWN * 2 + // two guarded passed pawns
                              VALUE_PASSED_PAWN_BONUS[2]) - // white pawn on 4th
                          (VALUE_PASSED_PAWN_BONUS[3]) // black pawn on 5th
    );
    test_passed_pawns("r3k3/8/7p/1P2Pp1P/2Pp1PP1/8/B7/7K w - - 0 1", -3 + // bonus adjustment based on game stage
                      (5 * VALUE_KING_DISTANCE_PASSED_PAWN_MULTIPLIER) -
                          (4 * VALUE_KING_DISTANCE_PASSED_PAWN_MULTIPLIER) +
                          (VALUE_PASSED_PAWN_BONUS[3] * 2 + // white pawns on 5th
                              VALUE_GUARDED_PASSED_PAWN * 2 + // two guarded passed pawns
                              VALUE_PASSED_PAWN_BONUS[2]) - // white pawn on 4th
                          (VALUE_PASSED_PAWN_BONUS[3]) // black pawn on 5th
    );
    test_passed_pawns("r3k3/8/7p/1P2Pp1P/2Pp1PP1/8/8/7K b - - 0 1", -6 + // bonus adjustment based on game stage
                      -VALUE_KING_CANNOT_CATCH_PAWN +
                      (5 * VALUE_KING_DISTANCE_PASSED_PAWN_MULTIPLIER) -
                          (4 * VALUE_KING_DISTANCE_PASSED_PAWN_MULTIPLIER) +
                          (VALUE_PASSED_PAWN_BONUS[3] * 2 + // white pawns on 5th
                              VALUE_GUARDED_PASSED_PAWN * 2 + // two guarded passed pawns
                              VALUE_PASSED_PAWN_BONUS[2]) - // white pawn on 4th
                          (VALUE_PASSED_PAWN_BONUS[3]) // black pawn on 5th
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
    test_knight_outposts("r1b1kbnr/2p3pp/ppn3p1/1B1Np3/4N3/8/PPPP1PPP/R1BQ1RK1 w kq - 0 1",  0);
    test_knight_outposts("r1b1kbnr/2p3pp/ppn3p1/1B1Np3/2P1N3/3P4/PP3PPP/R1BQ1RK1 w kq - 0 1", VALUE_KNIGHT_OUTPOST);

    test_knight_outposts("r1b1kbnr/6pp/ppn3p1/1p1Np3/4N3/8/PPPP1PPP/R1BQ1RK1 w kq - 0 1", 0);
    test_knight_outposts("r1b1kbnr/6pp/ppn3p1/1p1Np3/4N3/3P4/PPP2PPP/R1BQ1RK1 w kq - 0 1", VALUE_KNIGHT_OUTPOST);
    test_knight_outposts("r1b1kbnr/6pp/ppn3p1/1p1Np3/2P1N3/3P4/PP3PPP/R1BQ1RK1 w kq - 0 1", VALUE_KNIGHT_OUTPOST * 2);

    test_knight_outposts("r1b1kb1r/6pp/ppn3p1/1p1Np3/4N1n1/3P4/PPPP1PP1/R1BQ1RK1 w kq - 0 1", VALUE_KNIGHT_OUTPOST);
    test_knight_outposts("r1b1kb1r/6pp/ppn3p1/1p1Np3/4N1n1/P2P4/PPPP2P1/R1BQ1RK1 w kq - 0 1", VALUE_KNIGHT_OUTPOST);
    test_knight_outposts("r1b1kb1r/6p1/ppn3p1/1p1Np2p/4N1n1/P2P4/PPPP2P1/R1BQ1RK1 w kq - 0 1", 0);

    test_knight_outposts("r1b1kb1r/6p1/ppn3p1/1p1Np2p/4N1n1/6P1/PPPPP1P1/R1BQ1RK1 w kq - 0 1", -VALUE_KNIGHT_OUTPOST);
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
fn it_gets_the_number_of_pieces_on_the_same_file_in_a_bitboard() {
    let position = get_position("3Nk3/4p3/2p2p2/1bp2p2/3b1Pn1/2N5/1PP3PP/2BQK2R b K - 0 1");

    assert_eq!(material(&position.pieces[WHITE as usize]), (KNIGHT_VALUE * 2) + (PAWN_VALUE * 5) + BISHOP_VALUE + QUEEN_VALUE + ROOK_VALUE);
    assert_eq!(material(&position.pieces[BLACK as usize]), KNIGHT_VALUE + (PAWN_VALUE * 5) + (BISHOP_VALUE * 2));

    let position = get_position("r1q1k3/1R2n2p/5b2/5r2/p1Pp4/7P/1p2p3/6K1 b q - 0 1");
    assert_eq!(material_score(&position), ROOK_VALUE + (PAWN_VALUE *2) - (ROOK_VALUE * 2) - QUEEN_VALUE - BISHOP_VALUE - KNIGHT_VALUE - (PAWN_VALUE * 5));
}

fn test_king_threats(fen: &str, score: Score) {
    let position = get_position(fen);
    assert_eq!(king_threat_score(&position), score);
    assert_eq!(king_threat_score(&invert_pos(&position)), -score);
}

#[test]
fn it_evaluates_king_threats() {
    test_king_threats("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 0);
    test_king_threats("rnbqkb1r/pppppppp/8/8/4n3/4BNP1/PPPPPP1P/RNBQ1RK1 w kq - 0 1", -(KING_THREAT_BONUS * 2));
    test_king_threats("r1bqkb1r/pppppppp/8/8/4n2n/4BNP1/PPPPPP1P/RNBQ1RK1 w kq - 0 1", -(KING_THREAT_BONUS * 4));
    test_king_threats("r1bqkb1r/pppppppp/8/8/4n2n/4BNP1/PPPPPP1P/RNBQ1RK1 w kq - 0 1", -(KING_THREAT_BONUS * 4));
    test_king_threats("r1bqkb1r/pppppppp/8/4n3/7n/4BNP1/PPPPPP1P/RNBQ1RK1 w kq - 0 1", -KING_THREAT_BONUS * 3);
    test_king_threats("rkbq1b1r/pppppppp/5N2/8/4n2n/4BNP1/PPPPPP1P/R1BQ1RK1 w - - 0 1", -(KING_THREAT_BONUS * 2));
    test_king_threats("rkbq1b1r/pppppppp/3n1N2/8/7n/4BNP1/PPPPPP1P/R1BQ1RK1 w - - 0 1", 0);
    test_king_threats("rkb4r/pppppppp/3n1N2/3b1q2/7n/4BNP1/PPPPPP1P/R1BQ1RK1 w - - 0 1", -KING_THREAT_BONUS * 3);
    test_king_threats("rkb4r/1ppppppp/1p1n1N2/3b1q2/7n/R3BNP1/PPPPPP1P/2BQ1RK1 w - - 0 1", -KING_THREAT_BONUS * 4);
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
    test_king_safety("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 0 ,0);
    test_king_safety("rnbqkbnr/pppppppp/8/8/8/4BNP1/PPPPPP1P/RNBQ1RK1 w kq - 0 1", 15 ,0);
    test_king_safety("rnbqkbnr/pppppppp/8/8/8/5NP1/PPPPPPBP/RNBQ1RK1 w kq - 0 1", 35 ,0);
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
