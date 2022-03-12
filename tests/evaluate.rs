use rusty_rival::bitboards::south_fill;
use rusty_rival::engine_constants::{BISHOP_VALUE, KNIGHT_VALUE, PAWN_VALUE, QUEEN_VALUE, ROOK_VALUE};
use rusty_rival::evaluate::{on_same_file_count, material, material_score, doubled_and_isolated_pawn_score, isolated_pawn_count, white_king_early_safety, black_king_early_safety, bishop_pair_bonus, passed_pawn_score, DOUBLED_PAWN_PENALTY, VALUE_PASSED_PAWN_BONUS, VALUE_GUARDED_PASSED_PAWN, ISOLATED_PAWN_PENALTY, king_threat_score, KING_THREAT_BONUS};
use rusty_rival::fen::get_position;
use rusty_rival::types::{BLACK, Score, WHITE};
use rusty_rival::utils::{invert_fen, invert_pos};

fn test_doubled_pawns(fen: &str, score: Score) {
    let position = get_position(fen);
    assert_eq!(doubled_and_isolated_pawn_score(&position), score);

    let position = get_position(&invert_fen(fen));
    assert_eq!(doubled_and_isolated_pawn_score(&position), -score);
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

#[test]
fn it_gets_the_bishop_pair_bonus() {
    let position = get_position("3Nk3/4p3/2p2p2/1bp2p2/3b1Pn1/2N5/1PP3PP/2BQK2R b K - 0 1");
    assert_eq!(bishop_pair_bonus(position.pieces[WHITE as usize].bishop_bitboard, position.pieces[WHITE as usize].pawn_bitboard), 0);
    assert_eq!(bishop_pair_bonus(position.pieces[BLACK as usize].bishop_bitboard, position.pieces[BLACK as usize].pawn_bitboard), 29);

    let position = get_position("3Nk3/4p3/8/1bp2p2/3b1Pn1/2N5/1PP3PP/1BBQK2R b K - 0 1");
    assert_eq!(bishop_pair_bonus(position.pieces[WHITE as usize].bishop_bitboard, position.pieces[WHITE as usize].pawn_bitboard), 29);
    assert_eq!(bishop_pair_bonus(position.pieces[BLACK as usize].bishop_bitboard, position.pieces[BLACK as usize].pawn_bitboard), 35);
}

fn test_passed_pawns(fen: &str, score: Score) {
    let position = get_position(fen);
    assert_eq!(passed_pawn_score(&position), score);

    let position = get_position(&invert_fen(fen));
    assert_eq!(passed_pawn_score(&position), -score);
}

#[test]
fn it_gets_the_passed_pawn_score() {
    //assert_eq!(passed_pawn_wrapper(START_POS), 0);
    test_passed_pawns("4k3/8/7p/1P2Pp1P/2Pp1PP1/8/8/4K3 w - - 0 1",
               (VALUE_PASSED_PAWN_BONUS[3] * 2 + // white pawns on 5th
                VALUE_GUARDED_PASSED_PAWN * 2 + // two guarded passed pawns
                VALUE_PASSED_PAWN_BONUS[2]) - // white pawn on 4th
                   (VALUE_PASSED_PAWN_BONUS[3]) // black pawn on 5th
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
    test_king_threats("r1bqkb1r/pppppppp/5N2/8/4n2n/4BNP1/PPPPPP1P/R1BQ1RK1 w kq - 0 1", -KING_THREAT_BONUS * 2);
    test_king_threats("rkbq1b1r/pppppppp/5N2/8/4n2n/4BNP1/PPPPPP1P/R1BQ1RK1 w - - 0 1", -(KING_THREAT_BONUS * 2));
    test_king_threats("rkbq1b1r/pppppppp/3n1N2/8/7n/4BNP1/PPPPPP1P/R1BQ1RK1 w - - 0 1", 0);
    test_king_threats("rkb4r/pppppppp/3n1N2/3b1q2/7n/4BNP1/PPPPPP1P/R1BQ1RK1 w - - 0 1", -(KING_THREAT_BONUS * 3));
    test_king_threats("rkb4r/1ppppppp/1p1n1N2/3b1q2/7n/R3BNP1/PPPPPP1P/2BQ1RK1 w - - 0 1", -KING_THREAT_BONUS);
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
