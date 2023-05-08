use rusty_rival::bitboards::{B2_BIT, C8_BIT, D8_BIT, E1_BIT, E2_BIT, F3_BIT, F4_BIT};
use rusty_rival::fen::{algebraic_move_from_move, algebraic_squareref_from_bitref, bit_array_to_decimal, bitref_from_algebraic_squareref, board_bits, char_as_num, fen_board_part, simple_algebraic_to_pretty_algebraic, get_fen, get_fen_ranks, get_piece_on_square, get_position, move_from_algebraic_move, piece_bitboard, rank_bits};
use rusty_rival::move_constants::{EN_PASSANT_NOT_AVAILABLE, START_POS};
use rusty_rival::types::{is_bk_castle_available, is_bq_castle_available, is_wk_castle_available, is_wq_castle_available, BLACK, WHITE};

#[test]
fn it_gets_the_board_part_from_the_fen() {
    assert_eq!(
        "6k1/6p1/1p2q2p/1p5P/1P3RP1/2PK1B2/1r2N3/8",
        fen_board_part(&"6k1/6p1/1p2q2p/1p5P/1P3RP1/2PK1B2/1r2N3/8 b - g3 5 56".to_string())
    );
}

#[test]
fn it_gets_a_char_as_a_number() {
    assert_eq!(0, char_as_num('0'));
    assert_eq!(4, char_as_num('4'));
}

#[test]
fn it_gets_the_rank_bits_for_a_piece() {
    assert_eq!(vec![0, 0, 0, 0, 0, 0, 0, 0], rank_bits(&String::from("8"), 'Q'));
    assert_eq!(vec![0, 0, 0, 0, 0, 0, 1, 0], rank_bits(&String::from("6k1"), 'k'));
    assert_eq!(vec![0, 0, 0, 0, 0, 0, 0, 0], rank_bits(&String::from("6k1"), 'q'));
    assert_eq!(vec![0, 0, 0, 0, 0, 0, 1, 0], rank_bits(&String::from("6p1"), 'p'));
    assert_eq!(vec![0, 0, 0, 0, 0, 0, 1, 1], rank_bits(&String::from("6pp"), 'p'));
    assert_eq!(vec![1, 0, 0, 0, 0, 0, 0, 0], rank_bits(&String::from("P7"), 'P'));
    assert_eq!(vec![0, 1, 0, 0, 0, 0, 0, 1], rank_bits(&String::from("1p2q2p"), 'p'));
}

#[test]
fn it_gets_the_fen_ranks() {
    assert_eq!(
        vec!["6k1", "6p1", "1p2q2p", "1p5P", "1P3RP1", "2PK1B2", "1r2N3", "8"],
        get_fen_ranks(String::from("6k1/6p1/1p2q2p/1p5P/1P3RP1/2PK1B2/1r2N3/8"))
    );
}

#[test]
fn it_converts_a_bit_array_to_decimal() {
    assert_eq!(
        bit_array_to_decimal(vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]),
        0
    );
    assert_eq!(
        bit_array_to_decimal(vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1
        ]),
        1
    );
    assert_eq!(
        bit_array_to_decimal(vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1
        ]),
        3
    );
    assert_eq!(
        bit_array_to_decimal(vec![
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]),
        9223372036854775808
    );
    assert_eq!(
        bit_array_to_decimal(vec![
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1
        ]),
        18446744073709551615
    );
}

#[test]
fn it_gets_the_board_bits() {
    let fen = String::from("6k1/6p1/1p2q2p/1p5P/1P3RP1/2PK1B2/1r2N3/8 b kQKq g3 5 56");

    assert_eq!(
        vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ],
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

    assert_eq!(634693087133696, piece_bitboard(&get_fen_ranks(fen_board_part(&fen)), 'p'))
}

#[test]
fn it_converts_a_compact_move_to_an_algebraic_move() {
    assert_eq!(algebraic_move_from_move(458808), "a1h8");
    assert_eq!(algebraic_move_from_move(458872), "a1h8r");
    assert_eq!(algebraic_move_from_move(720947), "e2e7");
}

#[test]
fn it_returns_a_char_representing_the_piece_on_a_square() {
    let fen = "2nb2k1/6p1/1p2q2p/1p5P/1P3RP1/2PK1B2/1r2N3/5Q2 b - - 1 56";
    let position = get_position(&fen.to_string());
    assert_eq!(get_piece_on_square(&position, B2_BIT), 'r');
    assert_eq!(get_piece_on_square(&position, F4_BIT), 'R');
    assert_eq!(get_piece_on_square(&position, E2_BIT), 'N');
    assert_eq!(get_piece_on_square(&position, C8_BIT), 'n');
    assert_eq!(get_piece_on_square(&position, D8_BIT), 'b');
    assert_eq!(get_piece_on_square(&position, F3_BIT), 'B');
    assert_eq!(get_piece_on_square(&position, E1_BIT), '-');
}

#[test]
fn it_creates_a_fen_from_a_position() {
    let fen = "2nb2k1/6p1/1p2q2p/1p5P/1P3RP1/2PK1B2/1r2N3/5Q2 b - - 1 56";
    let position = get_position(&fen.to_string());
    assert_eq!(get_fen(&position), fen);

    let fen = START_POS;
    let position = get_position(&fen.to_string());
    assert_eq!(get_fen(&position), fen);
}

#[test]
fn it_converts_an_algebraic_move_to_a_move() {
    assert_eq!(algebraic_move_from_move(move_from_algebraic_move("a1h8".to_string(), 0)), "a1h8");
    assert_eq!(algebraic_move_from_move(move_from_algebraic_move("h7g8b".to_string(), 0)), "h7g8b");
    assert_eq!(algebraic_move_from_move(move_from_algebraic_move("h1a8".to_string(), 0)), "h1a8");

    assert_eq!(458808, move_from_algebraic_move("a1h8".to_string(), 0));
    assert_eq!(458872, move_from_algebraic_move("a1h8r".to_string(), 0));
}

#[test]
fn it_creates_a_position_from_a_fen() {
    let fen = "6k1/6p1/1p2q2p/1p5P/1P3RP1/2PK1B2/1r2N3/8 b q g3 5 56";
    let position = get_position(&fen.to_string());
    assert_eq!(position.mover, BLACK);
    assert_eq!(position.pieces[WHITE as usize].pawn_bitboard, 5404360704);
    assert_eq!(position.pieces[WHITE as usize].knight_bitboard, 2048);
    assert_eq!(position.pieces[WHITE as usize].king_square, 20);
    assert_eq!(position.pieces[WHITE as usize].bishop_bitboard, 262144);
    assert_eq!(position.pieces[WHITE as usize].queen_bitboard, 0);
    assert_eq!(position.pieces[WHITE as usize].rook_bitboard, 67108864);
    assert_eq!(position.pieces[BLACK as usize].pawn_bitboard, 634693087133696);
    assert_eq!(position.pieces[BLACK as usize].knight_bitboard, 0);
    assert_eq!(position.pieces[BLACK as usize].king_square, 57);
    assert_eq!(position.pieces[BLACK as usize].bishop_bitboard, 0);
    assert_eq!(position.pieces[BLACK as usize].queen_bitboard, 8796093022208);
    assert_eq!(position.pieces[BLACK as usize].rook_bitboard, 16384);
    assert_eq!(position.en_passant_square, 17);
}

#[test]
fn it_creates_a_position_from_a_fen_2() {
    let fen = "6k1/6p1/1p2q2p/1p5P/1P3RP1/2PK1B2/1r2N3/8 w kQ - 5 56";
    let position = get_position(&fen.to_string());
    assert_eq!(position.en_passant_square, EN_PASSANT_NOT_AVAILABLE);
    assert_eq!(position.mover, WHITE);
    assert_eq!(is_wk_castle_available(&position), false);
    assert_eq!(is_wq_castle_available(&position), true);
    assert_eq!(is_bk_castle_available(&position), true);
    assert_eq!(is_bq_castle_available(&position), false);
}

#[test]
fn it_converts_a_simple_move_to_an_algebraic_move() {
    let fen = "6k1/P5p1/1pq1N2p/2p4P/1P1p1RP1/2PKPB2/1r2N3/7N w - - 0 1";
    assert_eq!(simple_algebraic_to_pretty_algebraic(fen, "f3a8".to_string()), Some(String::from("Ba8")));
    assert_eq!(simple_algebraic_to_pretty_algebraic(fen, "c3c4".to_string()), Some(String::from("c4")));
    assert_eq!(simple_algebraic_to_pretty_algebraic(fen, "a7a8q".to_string()), Some(String::from("a8q")));
    assert_eq!(simple_algebraic_to_pretty_algebraic(fen, "e2d4".to_string()), Some(String::from("N2xd4")));
    assert_eq!(simple_algebraic_to_pretty_algebraic(fen, "h1g3".to_string()), Some(String::from("Nhg3")));
    assert_eq!(simple_algebraic_to_pretty_algebraic(fen, "e6c5".to_string()), Some(String::from("Nxc5")));
    assert_eq!(simple_algebraic_to_pretty_algebraic(fen, "b4c5".to_string()), Some(String::from("bxc5")));
    assert_eq!(simple_algebraic_to_pretty_algebraic(fen, "e3d4".to_string()), Some(String::from("exd4")));
}