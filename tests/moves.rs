use rusty_rival::bitboards::bitboards::bit;
use rusty_rival::fen::fen::get_position;
use rusty_rival::moves::moves::{all_bits_except_friendly_pieces, moves_from_to_squares_bitboard};

#[test]
fn it_gets_all_bits_except_friendly_pieces() {
    let position = get_position(&"n5k1/6n1/1n2q2p/1p5P/1P3RP1/2PK1B2/1r2N3/8 b kQKq g3 5 56".to_string());
    assert_eq!(all_bits_except_friendly_pieces(&position), 0b0111110111111101101101101011111111111111111111111011111111111111);
}

#[test]
fn it_gets_all_pieces_bitboard() {
    let position = get_position(&"n5k1/6n1/1n2q2p/1p5P/1P3RP1/2PK1B2/1r2N3/8 b kQKq g3 5 56".to_string());
    assert_eq!(position.all_pieces_bitboard, 0b1000001000000010010010010100000101000110001101000100100000000000);
}

#[test]
fn it_creates_a_list_of_moves_from_a_from_square_and_a_list_of_to_squares() {
    let mut moves = moves_from_to_squares_bitboard(11, bit(22) | bit(33) | bit(44));
    moves.sort();
    assert_eq!(moves, vec![720918,720929,720940]);
}

// #[test]
// fn it_generates_knight_moves_from_a_given_fen_ignoring_checks() {
//     sort (map algebraicMoveFromMove (generateKnightMoves (getPosition "n5k1/6n1/1n2q2p/1p5P/1P3RP1/2PK1B2/1r2N3/8 b kQKq g3 5 56")))
//     `shouldBe` ["a8c7","b6a4","b6c4","b6c8","b6d5","b6d7","g7e8","g7f5","g7h5"]
//     sort (map algebraicMoveFromMove (generateKnightMoves (getPosition "n5k1/6n1/1n2q2p/1p5P/1P3RP1/2PK1B2/1r2N3/8 w kQKq g3 5 56")))
//     `shouldBe` ["e2c1","e2d4","e2g1","e2g3"]
// }
//
