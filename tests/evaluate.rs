use rusty_rival::bitboards::south_fill;
use rusty_rival::engine_constants::{BISHOP_VALUE, DOUBLED_PAWN_PENALTY, ISOLATED_PAWN_PENALTY, KNIGHT_VALUE, PAWN_VALUE, QUEEN_VALUE, ROOK_VALUE};
use rusty_rival::evaluate::{on_same_file_count, material, material_score, pawn_score, isolated_pawn_count};
use rusty_rival::fen::get_position;
use rusty_rival::types::{BLACK, WHITE};

#[test]
fn it_gets_the_pawn_score() {
    let position = get_position("3Nk3/4p3/2p2p2/1bp2p2/3b1Pn1/2N5/1PP3PP/2BQK2R b K - 0 1");
    assert_eq!(on_same_file_count(position.pieces[WHITE as usize].pawn_bitboard, south_fill(position.pieces[WHITE as usize].pawn_bitboard) as u8), 0);
    assert_eq!(on_same_file_count(position.pieces[BLACK as usize].pawn_bitboard, south_fill(position.pieces[BLACK as usize].pawn_bitboard) as u8), 2);

    assert_eq!(isolated_pawn_count(south_fill(position.pieces[WHITE as usize].pawn_bitboard) as u8), 0);
    assert_eq!(isolated_pawn_count(south_fill(position.pieces[BLACK as usize].pawn_bitboard) as u8), 1);

    assert_eq!(pawn_score(&position), DOUBLED_PAWN_PENALTY * 2 + ISOLATED_PAWN_PENALTY);

    let position = get_position("3Nk3/4p3/1p3p2/1bp2p2/3b1Pn1/2NP4/1P4PP/2BQK2R w K - 0 1");
    assert_eq!(on_same_file_count(position.pieces[WHITE as usize].pawn_bitboard, south_fill(position.pieces[WHITE as usize].pawn_bitboard) as u8), 0);
    assert_eq!(on_same_file_count(position.pieces[BLACK as usize].pawn_bitboard, south_fill(position.pieces[BLACK as usize].pawn_bitboard) as u8), 1);

    assert_eq!(isolated_pawn_count(south_fill(position.pieces[WHITE as usize].pawn_bitboard) as u8), 2);
    assert_eq!(isolated_pawn_count(south_fill(position.pieces[BLACK as usize].pawn_bitboard) as u8), 0);

    assert_eq!(pawn_score(&position), -(ISOLATED_PAWN_PENALTY * 2) + DOUBLED_PAWN_PENALTY);
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
fn it_gets_the_number_of_pieces_on_the_same_file_in_a_bitboard() {
    let position = get_position("3Nk3/4p3/2p2p2/1bp2p2/3b1Pn1/2N5/1PP3PP/2BQK2R b K - 0 1");

    assert_eq!(material(&position.pieces[WHITE as usize]), (KNIGHT_VALUE * 2) + (PAWN_VALUE * 5) + BISHOP_VALUE + QUEEN_VALUE + ROOK_VALUE);
    assert_eq!(material(&position.pieces[BLACK as usize]), KNIGHT_VALUE + (PAWN_VALUE * 5) + (BISHOP_VALUE * 2));

    let position = get_position("r1q1k3/1R2n2p/5b2/5r2/p1Pp4/7P/1p2p3/6K1 b q - 0 1");
    assert_eq!(material_score(&position), ROOK_VALUE + (PAWN_VALUE *2) - (ROOK_VALUE * 2) - QUEEN_VALUE - BISHOP_VALUE - KNIGHT_VALUE - (PAWN_VALUE * 5));

}
