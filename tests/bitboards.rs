use rusty_rival::bitboards::*;
use rusty_rival::bitboards::bitboards::{bit_list, bit_string, enemy_bitboard};

#[test]
fn it_gets_bit_lists() {
    assert_eq!(0, bit_list(0b0000000000000000000000000000000000000000000000000000000000000000).len());
    assert_eq!(vec![55], bit_list(0b0000000010000000000000000000000000000000000000000000000000000000));
    assert_eq!(vec![0,55], bit_list(0b0000000010000000000000000000000000000000000000000000000000000001));
    assert_eq!(vec![0,55,63], bit_list(0b1000000010000000000000000000000000000000000000000000000000000001));
}

#[test]
fn it_gets_a_bit_string() {
    assert_eq!("0000000000000000000000000000000000000000000000000000000000001111", bit_string(15))
}

#[test]
fn it_gets_the_enemy_bitboard() {
    assert_eq!(
        0b0000000001001000000000001000100000010100000101000000100100000000,
        enemy_bitboard(get_position("n5k1/1P2P1n1/1n2q2p/Pp1pP3/3P1R2/3K1B2/1r2N2P/6r1 b - - 0 1"))
    )
}

#[test]
fn it_gets_the_rank_bits_for_a_piece() {
    assert_eq!([0,0,0,0,0,0,0,0], rank_bits("8", 'Q'))
    assert_eq!([0,0,0,0,0,0,1,0], rank_bits("6k1", 'k'))
    assert_eq!([0,0,0,0,0,0,0,0], rank_bits("6k1", 'q'))
    assert_eq!([0,0,0,0,0,0,1,0], rank_bits("6p1", 'p'))
    assert_eq!([0,0,0,0,0,0,1,1], rank_bits("6pp", 'p'))
    assert_eq!([1,0,0,0,0,0,0,0], rank_bits("P7", 'P'))
    assert_eq!([0,1,0,0,0,0,0,1], rank_bits("1p2q2p", 'p'))
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


