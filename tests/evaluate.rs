use rusty_rival::bitboards::south_fill;
use rusty_rival::engine_constants::{BISHOP_VALUE, DOUBLED_PAWN_PENALTY, KNIGHT_VALUE, PAWN_VALUE, QUEEN_VALUE, ROOK_VALUE};
use rusty_rival::evaluate::{on_same_file_count, material, material_score, pawn_score, isolated_pawn_count, white_king_early_safety, black_king_early_safety, bishop_pair_bonus, passed_pawn_score, VALUE_PASSED_PAWN_BONUS, VALUE_GUARDED_PASSED_PAWN};
use rusty_rival::fen::get_position;
use rusty_rival::move_constants::START_POS;
use rusty_rival::types::{BLACK, Score, WHITE};
use rusty_rival::utils::{invert_fen, invert_pos};

#[test]
fn it_gets_the_pawn_score() {
    let position = get_position("3Nk3/4p3/2p2p2/1bp2p2/3b1Pn1/2N5/1PP3PP/2BQK2R b K - 0 1");
    assert_eq!(on_same_file_count(position.pieces[WHITE as usize].pawn_bitboard, south_fill(position.pieces[WHITE as usize].pawn_bitboard) as u8), 0);
    assert_eq!(on_same_file_count(position.pieces[BLACK as usize].pawn_bitboard, south_fill(position.pieces[BLACK as usize].pawn_bitboard) as u8), 2);

    assert_eq!(isolated_pawn_count(south_fill(position.pieces[WHITE as usize].pawn_bitboard) as u8), 0);
    assert_eq!(isolated_pawn_count(south_fill(position.pieces[BLACK as usize].pawn_bitboard) as u8), 1);

    assert_eq!(pawn_score(&position), DOUBLED_PAWN_PENALTY * 2 );

    let position = get_position("3Nk3/4p3/1p3p2/1bp2p2/3b1Pn1/2NP4/1P4PP/2BQK2R w K - 0 1");
    assert_eq!(on_same_file_count(position.pieces[WHITE as usize].pawn_bitboard, south_fill(position.pieces[WHITE as usize].pawn_bitboard) as u8), 0);
    assert_eq!(on_same_file_count(position.pieces[BLACK as usize].pawn_bitboard, south_fill(position.pieces[BLACK as usize].pawn_bitboard) as u8), 1);

    assert_eq!(isolated_pawn_count(south_fill(position.pieces[WHITE as usize].pawn_bitboard) as u8), 2);
    assert_eq!(isolated_pawn_count(south_fill(position.pieces[BLACK as usize].pawn_bitboard) as u8), 0);

    assert_eq!(pawn_score(&position), DOUBLED_PAWN_PENALTY);
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

fn test_passed_pawns(fen: &str, score: Score) -> Score {
    let position = get_position(fen);
    let white_pawns = position.pieces[WHITE as usize].pawn_bitboard;
    let black_pawns = position.pieces[BLACK as usize].pawn_bitboard;
    passed_pawn_score(white_pawns, black_pawns);

    let position = get_position(&invert_fen(fen));
    let white_pawns = position.pieces[WHITE as usize].pawn_bitboard;
    let black_pawns = position.pieces[BLACK as usize].pawn_bitboard;
    passed_pawn_score(white_pawns, black_pawns)

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

#[test]
fn it_gets_the_number_of_pieces_on_the_same_file_in_a_bitboard() {
    let position = get_position("3Nk3/4p3/2p2p2/1bp2p2/3b1Pn1/2N5/1PP3PP/2BQK2R b K - 0 1");

    assert_eq!(material(&position.pieces[WHITE as usize]), (KNIGHT_VALUE * 2) + (PAWN_VALUE * 5) + BISHOP_VALUE + QUEEN_VALUE + ROOK_VALUE);
    assert_eq!(material(&position.pieces[BLACK as usize]), KNIGHT_VALUE + (PAWN_VALUE * 5) + (BISHOP_VALUE * 2));

    let position = get_position("r1q1k3/1R2n2p/5b2/5r2/p1Pp4/7P/1p2p3/6K1 b q - 0 1");
    assert_eq!(material_score(&position), ROOK_VALUE + (PAWN_VALUE *2) - (ROOK_VALUE * 2) - QUEEN_VALUE - BISHOP_VALUE - KNIGHT_VALUE - (PAWN_VALUE * 5));
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
    test_king_safety("rnbqkbnr/pppppppp/8/8/8/4BNP1/PPPPPP1P/RNBQ1RK1 w kq - 0 1", 25 ,0);
    test_king_safety("rnbqkbnr/pppppppp/8/8/8/5NP1/PPPPPPBP/RNBQ1RK1 w kq - 0 1", 35 ,0);
    test_king_safety("rnbqkbnr/pppppppp/8/8/8/5N2/PPPPPPPP/RNBQBRK1 w kq - 0 1", 45, 0);
    test_king_safety("rnbqkbnr/pppppppp/8/8/8/5NPP/PPPPPPB1/RNBQ1RK1 w kq - 0 1", 30, 0);
    test_king_safety("rnbqkbnr/pppppppp/8/8/8/5NPP/PPPPPP2/RNBQBRK1 w kq - 0 1", 20, 0);
    test_king_safety("rnbqkbnr/pppppppp/8/8/8/3PBN2/PPPPPP1P/RNBQ1RK1 w kq - 0 1", -10, 0);
    test_king_safety("rnbqkbnr/pppppppp/8/8/8/2PPBN2/PPPPPP2/RNBQ1RK1 w kq - 0 1", -15, 0);
    test_king_safety("rnbqkbnr/pppppppp/8/8/2P5/2PPBN2/PPPPP3/RNBQ1RK1 w kq - 0 1", -20, 0);
}
