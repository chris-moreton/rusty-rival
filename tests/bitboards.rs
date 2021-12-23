use rusty_rival::bitboards::*;
use rusty_rival::bitboards::bitboards::{bit_list, bit_string, enemy_bitboard, north_fill, south_fill};
use rusty_rival::fen::fen::rank_bits;

#[test]
fn it_gets_bit_lists() {
    assert_eq!(0, bit_list(0b0000000000000000000000000000000000000000000000000000000000000000).len());
    assert_eq!(vec![55], bit_list(0b0000000010000000000000000000000000000000000000000000000000000000));
    assert_eq!(vec![55,0], bit_list(0b0000000010000000000000000000000000000000000000000000000000000001));
    assert_eq!(vec![63,55,0], bit_list(0b1000000010000000000000000000000000000000000000000000000000000001));
}

#[test]
fn it_gets_a_bit_string() {
    assert_eq!("0000000000000000000000000000000000000000000000000000000000001111", bit_string(15))
}

#[test]
fn it_north_fills() {
    assert_eq!(0b0000000100000000000000000000000000000000000000000000000000000000,
               north_fill(0b0000000100000000000000000000000000000000000000000000000000000000));

    assert_eq!(0b1111111111111111111111111111111111111111111010111010101010101010,
               north_fill(0b0100000000000000111001000000000001010101110000110000000010101010));
}

#[test]
fn it_south_fills() {
    assert_eq!(0b0000000100000001000000010000000100000001000000010000000100000001,
               south_fill(0b0000000100000000000000000000000000000000000000000000000000000000));

    assert_eq!(0b0100000001000000111001001110010011110101111101111111011111111111,
               south_fill(0b0100000000000000111001000000000001010101110000110000000010101010));
}

#[test]
fn it_passes_sanity_checks_for_values_expressed_as_functions() {
    assert_eq!(rank8Bits, -72057594037927936);
    assert_eq!(file_a_bits, -9187201950435737472);
    assert_eq!(fileHBits, 72340172838076673);
    assert_eq!(middleFiles8Bit, 24);
    assert_eq!(nonMidFiles8Bit, 231);
    assert_eq!(f1G1Bits, 6);
    assert_eq!(g1H1Bits, 3);
    assert_eq!(a1B1Bits, 192);
    assert_eq!(b1C1Bits, 96);
    assert_eq!(f8G8Bits, 432345564227567616);
    assert_eq!(g8H8Bits, 216172782113783808);
    assert_eq!(a8B8Bits, -4611686018427387904);
    assert_eq!(b8C8Bits, 6917529027641081856);
    assert_eq!(lightSquaresBits, -6172840429334713771);
    assert_eq!(darkSquaresBits, 6172840429334713770);
    assert_eq!(low32Bits, 4294967295);
}


// #[test]
// fn it_gets_the_enemy_bitboard() {
//     assert_eq!(
//         0b0000000001001000000000001000100000010100000101000000100100000000,
//         enemy_bitboard(get_position("n5k1/1P2P1n1/1n2q2p/Pp1pP3/3P1R2/3K1B2/1r2N2P/6r1 b - - 0 1"))
//     )
// }

#[test]
fn it_gets_the_rank_bits_for_a_piece() {
    assert_eq!(vec![0,0,0,0,0,0,0,0], rank_bits(&"8".to_string(), 'Q'));
    assert_eq!(vec![0,0,0,0,0,0,1,0], rank_bits(&"6k1".to_string(), 'k'));
    assert_eq!(vec![0,0,0,0,0,0,0,0], rank_bits(&"6k1".to_string(), 'q'));
    assert_eq!(vec![0,0,0,0,0,0,1,0], rank_bits(&"6p1".to_string(), 'p'));
    assert_eq!(vec![0,0,0,0,0,0,1,1], rank_bits(&"6pp".to_string(), 'p'));
    assert_eq!(vec![1,0,0,0,0,0,0,0], rank_bits(&"P7".to_string(), 'P'));
    assert_eq!(vec![0,1,0,0,0,0,0,1], rank_bits(&"1p2q2p".to_string(), 'p'));
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


