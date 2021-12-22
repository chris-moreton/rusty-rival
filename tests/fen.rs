use rusty_rival::fen::*;
use rusty_rival::fen::fen::{char_as_num, rank_bits};

#[test]
fn it_gets_a_char_as_a_number() {
    assert_eq!(0, char_as_num('0'));
    assert_eq!(4, char_as_num('4'));
}

#[test]
fn it_gets_the_rank_bits_for_a_piece() {
    assert_eq!(vec![0,0,0,0,0,0,0,0], rank_bits(String::from("8"), 'Q'));
    assert_eq!(vec![0,0,0,0,0,0,1,0], rank_bits(String::from("6k1"), 'k'));
    assert_eq!(vec![0,0,0,0,0,0,0,0], rank_bits(String::from("6k1"), 'q'));
    assert_eq!(vec![0,0,0,0,0,0,1,0], rank_bits(String::from("6p1"), 'p'));
    assert_eq!(vec![0,0,0,0,0,0,1,1], rank_bits(String::from("6pp"), 'p'));
    assert_eq!(vec![1,0,0,0,0,0,0,0], rank_bits(String::from("P7"), 'P'));
    assert_eq!(vec![0,1,0,0,0,0,0,1], rank_bits(String::from("1p2q2p"), 'p'));
}

// #[test]
// fn it_gets_the_board_bits() {
//     let fen = "6k1/6p1/1p2q2p/1p5P/1P3RP1/2PK1B2/1r2N3/8 b kQKq g3 5 56";
//
//     assert_eq!(
//         [0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,1,0,0,0,0,0,1,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
//         board_bits(get_fen_ranks(fen_board_part(fen)), 'p')
//     )
// }
//
// #[test]
// fn it_gets_a_piece_bitboard() {
//     let fen = "6k1/6p1/1p2q2p/1p5P/1P3RP1/2PK1B2/1r2N3/8 b Q g3 5 56";
//
//     assert_eq!(
//         634693087133696,
//         piece_bitboard(get_fen_ranks(fen_board_part(fen)), 'p')
//     )
// }

