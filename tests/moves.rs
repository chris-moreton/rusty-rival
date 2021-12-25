use rusty_rival::fen::fen::get_position;
use rusty_rival::moves::moves::all_bits_except_friendly_pieces;

#[test]
fn it_gets_all_bits_except_friendly_pieces() {
    let position = get_position(&"n5k1/6n1/1n2q2p/1p5P/1P3RP1/2PK1B2/1r2N3/8 b kQKq g3 5 56".to_string());
    assert_eq!(all_bits_except_friendly_pieces(&position), 0b0111110111111101101101101011111111111111111111111011111111111111);
}