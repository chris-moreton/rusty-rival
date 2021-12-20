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
