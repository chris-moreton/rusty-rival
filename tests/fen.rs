use rusty_rival::fen::{algebraic_move_from_move, algebraic_squareref_from_bitref, bit_array_to_decimal, bitref_from_algebraic_squareref, board_bits, char_as_num, fen_board_part, get_fen_ranks, get_position, move_from_algebraic_move, piece_bitboard, rank_bits};
use rusty_rival::move_constants::EN_PASSANT_NOT_AVAILABLE;
use rusty_rival::types::Mover;

#[test]
fn it_gets_the_board_part_from_the_fen() {
    assert_eq!("6k1/6p1/1p2q2p/1p5P/1P3RP1/2PK1B2/1r2N3/8", fen_board_part(&"6k1/6p1/1p2q2p/1p5P/1P3RP1/2PK1B2/1r2N3/8 b - g3 5 56".to_string()));
}

#[test]
fn it_gets_a_char_as_a_number() {
    assert_eq!(0, char_as_num('0'));
    assert_eq!(4, char_as_num('4'));
}

#[test]
fn it_gets_the_rank_bits_for_a_piece() {
    assert_eq!(vec![0,0,0,0,0,0,0,0], rank_bits(&String::from("8"), 'Q'));
    assert_eq!(vec![0,0,0,0,0,0,1,0], rank_bits(&String::from("6k1"), 'k'));
    assert_eq!(vec![0,0,0,0,0,0,0,0], rank_bits(&String::from("6k1"), 'q'));
    assert_eq!(vec![0,0,0,0,0,0,1,0], rank_bits(&String::from("6p1"), 'p'));
    assert_eq!(vec![0,0,0,0,0,0,1,1], rank_bits(&String::from("6pp"), 'p'));
    assert_eq!(vec![1,0,0,0,0,0,0,0], rank_bits(&String::from("P7"), 'P'));
    assert_eq!(vec![0,1,0,0,0,0,0,1], rank_bits(&String::from("1p2q2p"), 'p'));
}

#[test]
fn it_gets_the_fen_ranks() {
    assert_eq!(
        vec!["6k1","6p1","1p2q2p","1p5P","1P3RP1","2PK1B2","1r2N3","8"],
        get_fen_ranks(String::from("6k1/6p1/1p2q2p/1p5P/1P3RP1/2PK1B2/1r2N3/8"))
    );
}

#[test]
fn it_converts_a_bit_array_to_decimal() {
    assert_eq!(
        bit_array_to_decimal(vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]),
        0
    );
    assert_eq!(
        bit_array_to_decimal(vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1]),
        1
    );
    assert_eq!(
        bit_array_to_decimal(vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,1]),
        3
    );
    assert_eq!(
        bit_array_to_decimal(vec![1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]),
        9223372036854775808
    );
    assert_eq!(
        bit_array_to_decimal(vec![1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1]),
        18446744073709551615
    );
}

#[test]
fn it_gets_the_board_bits() {
    let fen = String::from("6k1/6p1/1p2q2p/1p5P/1P3RP1/2PK1B2/1r2N3/8 b kQKq g3 5 56");

    assert_eq!(
        vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,1,0,0,0,0,0,1,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        board_bits(&get_fen_ranks(fen_board_part(&fen)), 'p')
    )
}

#[test]
fn it_converts_a_bitref_to_an_algebraic_square() {
    assert_eq!("a8", algebraic_squareref_from_bitref(63));
    assert_eq!("h1", algebraic_squareref_from_bitref(0));
    assert_eq!("h8", algebraic_squareref_from_bitref(56));
    assert_eq!("a1", algebraic_squareref_from_bitref(7));
}

#[test]
fn it_converts_an_algebraic_square_to_a_bitref() {
    assert_eq!(63, bitref_from_algebraic_squareref("a8".to_string()));
    assert_eq!(0, bitref_from_algebraic_squareref("h1".to_string()));
    assert_eq!(1, bitref_from_algebraic_squareref("g1".to_string()));
}

#[test]
fn it_gets_a_piece_bitboard() {
    let fen = "6k1/6p1/1p2q2p/1p5P/1P3RP1/2PK1B2/1r2N3/8 b Q g3 5 56".to_string();

    assert_eq!(
        634693087133696,
        piece_bitboard(&get_fen_ranks(fen_board_part(&fen)), 'p')
    )
}

#[test]
fn it_converts_a_compact_move_to_an_algebraic_move() {
    assert_eq!(algebraic_move_from_move(458808), "a1h8");
    assert_eq!(algebraic_move_from_move(458872), "a1h8r");
    assert_eq!(algebraic_move_from_move(720947), "e2e7");
}

#[test]
fn it_converts_an_algebraic_move_to_a_move() {
    assert_eq!(algebraic_move_from_move(move_from_algebraic_move("a1h8".to_string())), "a1h8");
    assert_eq!(algebraic_move_from_move(move_from_algebraic_move("h7g8b".to_string())), "h7g8b");
    assert_eq!(algebraic_move_from_move(move_from_algebraic_move("h1a8".to_string())), "h1a8");


    assert_eq!(458808, move_from_algebraic_move("a1h8".to_string()));
    assert_eq!(458872, move_from_algebraic_move("a1h8r".to_string()));
}

#[test]
fn it_creates_a_position_from_a_fen() {
    let fen = "6k1/6p1/1p2q2p/1p5P/1P3RP1/2PK1B2/1r2N3/8 b q g3 5 56";
    let position = get_position(&fen.to_string());
    assert_eq!(position.mover, Mover::Black);
    assert_eq!(position.white_pawn_bitboard, 5404360704);
    assert_eq!(position.white_knight_bitboard, 2048);
    assert_eq!(position.white_king_bitboard, 1048576);
    assert_eq!(position.white_bishop_bitboard, 262144);
    assert_eq!(position.white_queen_bitboard, 0);
    assert_eq!(position.white_rook_bitboard, 67108864);
    assert_eq!(position.black_pawn_bitboard, 634693087133696);
    assert_eq!(position.black_knight_bitboard, 0);
    assert_eq!(position.black_king_bitboard, 144115188075855872);
    assert_eq!(position.black_bishop_bitboard, 0);
    assert_eq!(position.black_queen_bitboard, 8796093022208);
    assert_eq!(position.black_rook_bitboard, 16384);
    assert_eq!(position.en_passant_square, 17);
}

#[test]
fn it_creates_a_position_from_a_fen_2() {
    let fen = "6k1/6p1/1p2q2p/1p5P/1P3RP1/2PK1B2/1r2N3/8 w kQ - 5 56";
    let position = get_position(&fen.to_string());
    assert_eq!(position.en_passant_square, EN_PASSANT_NOT_AVAILABLE);
    assert_eq!(position.mover, Mover::White);
    assert_eq!(position.white_king_castle_available, false);
    assert_eq!(position.white_queen_castle_available, true);
    assert_eq!(position.black_king_castle_available, true);
    assert_eq!(position.black_queen_castle_available, false);
}


