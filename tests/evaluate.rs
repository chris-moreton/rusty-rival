use rusty_rival::engine_constants::{BISHOP_VALUE, DOUBLED_PAWN_PENALTY, KNIGHT_VALUE, PAWN_VALUE, QUEEN_VALUE, ROOK_VALUE};
use rusty_rival::evaluate::{doubled_pawn_count, material, pawn_score};
use rusty_rival::fen::get_position;
use rusty_rival::types::{BLACK, WHITE};

#[test]
fn it_gets_the_doubled_pawn_count() {
    let position = get_position("3Nk3/4p3/2p2p2/1bp2p2/3b1Pn1/2N5/1PP3PP/2BQK2R b K - 0 1");
    assert_eq!(doubled_pawn_count(position.pieces[WHITE as usize].pawn_bitboard), 0);
    assert_eq!(doubled_pawn_count(position.pieces[BLACK as usize].pawn_bitboard), 2);

    assert_eq!(pawn_score(&position), DOUBLED_PAWN_PENALTY * 2);
}

#[test]
fn it_gets_material_advantage() {
    let position = get_position("3Nk3/4p3/2p2p2/1bp2p2/3b1Pn1/2N5/1PP3PP/2BQK2R b K - 0 1");

    assert_eq!(material(&position.pieces[WHITE as usize]),
               (KNIGHT_VALUE * 2) + (PAWN_VALUE * 5) + BISHOP_VALUE + QUEEN_VALUE + ROOK_VALUE);

    assert_eq!(material(&position.pieces[BLACK as usize]),
               KNIGHT_VALUE + (PAWN_VALUE * 5) + (BISHOP_VALUE * 2));
}
