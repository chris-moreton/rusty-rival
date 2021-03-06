use rusty_rival::bitboards::{
    bit, bitboard_for_mover, clear_bit, exactly_one_bit_set, north_fill, south_fill, test_bit, A1B1_BITS, A8B8_BITS, B1C1_BITS, B8C8_BITS,
    DARK_SQUARES_BITS, F1G1_BITS, F8G8_BITS, FILE_A_BITS, FILE_H_BITS, G1H1_BITS, G8H8_BITS, LIGHT_SQUARES_BITS, LOW_32_BITS,
    MIDDLE_FILES_8_BIT, NONMID_FILES_8_BIT, RANK_8_BITS,
};
use rusty_rival::fen::rank_bits;
use rusty_rival::move_constants::ALL_CASTLE_FLAGS;
use rusty_rival::types::{Piece, Pieces, Position, BLACK, WHITE};

#[test]
fn it_sets_a_bit() {
    assert_eq!(bit(0), 0b0000000000000000000000000000000000000000000000000000000000000001);
    assert_eq!(bit(63), 0b1000000000000000000000000000000000000000000000000000000000000000);
    assert_eq!(bit(3), 0b0000000000000000000000000000000000000000000000000000000000001000);
}

#[test]
fn it_north_fills() {
    assert_eq!(
        0b0000000100000000000000000000000000000000000000000000000000000000,
        north_fill(0b0000000100000000000000000000000000000000000000000000000000000000)
    );

    assert_eq!(
        0b1111111111111111111111111111111111111111111010111010101010101010,
        north_fill(0b0100000000000000111001000000000001010101110000110000000010101010)
    );
}

#[test]
fn it_south_fills() {
    assert_eq!(
        0b0000000100000001000000010000000100000001000000010000000100000001,
        south_fill(0b0000000100000000000000000000000000000000000000000000000000000000)
    );

    assert_eq!(
        0b0100000001000000111001001110010011110101111101111111011111111111,
        south_fill(0b0100000000000000111001000000000001010101110000110000000010101010)
    );
}

#[test]
fn it_passes_sanity_checks_for_values_expressed_as_functions() {
    assert_eq!(RANK_8_BITS as i64, -72057594037927936);
    assert_eq!(FILE_A_BITS as i64, -9187201950435737472);
    assert_eq!(FILE_H_BITS, 72340172838076673);
    assert_eq!(MIDDLE_FILES_8_BIT, 24);
    assert_eq!(NONMID_FILES_8_BIT, 231);
    assert_eq!(F1G1_BITS, 6);
    assert_eq!(G1H1_BITS, 3);
    assert_eq!(A1B1_BITS, 192);
    assert_eq!(B1C1_BITS, 96);
    assert_eq!(F8G8_BITS, 432345564227567616);
    assert_eq!(G8H8_BITS, 216172782113783808);
    assert_eq!(A8B8_BITS as i64, -4611686018427387904);
    assert_eq!(B8C8_BITS, 6917529027641081856);
    assert_eq!(DARK_SQUARES_BITS, 6172840429334713770);
    assert_eq!(LIGHT_SQUARES_BITS as i64, -6172840429334713771);
    assert_eq!(LOW_32_BITS, 4294967295);
}

#[test]
fn it_knows_when_exactly_one_bit_is_set() {
    assert_eq!(
        exactly_one_bit_set(0b0000000010000000000000000000000000000000000000000000000000000000),
        true
    );
    assert_eq!(
        exactly_one_bit_set(0b0000000010000000000001000000000000000000000000000000000000000000),
        false
    );
    assert_eq!(
        exactly_one_bit_set(0b0000000000000000000000000000000000000000000000000000000000000000),
        false
    );
    assert_eq!(
        exactly_one_bit_set(0b1000000000000000000000000000000000000000000000000000000000000000),
        true
    );
    assert_eq!(
        exactly_one_bit_set(0b0000000000000000000000000000000000000000000000000000000000000001),
        true
    );
    assert_eq!(
        exactly_one_bit_set(0b1000000000000000000000000000000000000000000000000000000000000001),
        false
    );
    assert_eq!(
        exactly_one_bit_set(0b1111111111111111111111111111111111111111111111111111111111111111),
        false
    );
}

#[test]
fn it_gets_the_rank_bits_for_a_piece() {
    assert_eq!(vec![0, 0, 0, 0, 0, 0, 0, 0], rank_bits(&"8".to_string(), 'Q'));
    assert_eq!(vec![0, 0, 0, 0, 0, 0, 1, 0], rank_bits(&"6k1".to_string(), 'k'));
    assert_eq!(vec![0, 0, 0, 0, 0, 0, 0, 0], rank_bits(&"6k1".to_string(), 'q'));
    assert_eq!(vec![0, 0, 0, 0, 0, 0, 1, 0], rank_bits(&"6p1".to_string(), 'p'));
    assert_eq!(vec![0, 0, 0, 0, 0, 0, 1, 1], rank_bits(&"6pp".to_string(), 'p'));
    assert_eq!(vec![1, 0, 0, 0, 0, 0, 0, 0], rank_bits(&"P7".to_string(), 'P'));
    assert_eq!(vec![0, 1, 0, 0, 0, 0, 0, 1], rank_bits(&"1p2q2p".to_string(), 'p'));
}

#[test]
fn it_returns_the_correct_bitboard_for_mover() {
    let p1 = Position {
        pieces: [
            Pieces {
                pawn_bitboard: 1,
                knight_bitboard: 2,
                bishop_bitboard: 3,
                queen_bitboard: 4,
                king_square: 5,
                rook_bitboard: 6,
                all_pieces_bitboard: 14,
            },
            Pieces {
                pawn_bitboard: 7,
                knight_bitboard: 8,
                bishop_bitboard: 9,
                queen_bitboard: 10,
                king_square: 11,
                rook_bitboard: 12,
                all_pieces_bitboard: 15,
            },
        ],
        mover: WHITE,
        en_passant_square: 1,
        castle_flags: ALL_CASTLE_FLAGS,
        half_moves: 0,
        move_number: 1,
        zobrist_lock: 0,
    };

    assert_eq!(1, bitboard_for_mover(&p1, Piece::Pawn));
    assert_eq!(2, bitboard_for_mover(&p1, Piece::Knight));
    assert_eq!(3, bitboard_for_mover(&p1, Piece::Bishop));
    assert_eq!(4, bitboard_for_mover(&p1, Piece::Queen));
    assert_eq!(bit(5), bitboard_for_mover(&p1, Piece::King));
    assert_eq!(6, bitboard_for_mover(&p1, Piece::Rook));

    let p2 = Position {
        pieces: [
            Pieces {
                pawn_bitboard: 1,
                knight_bitboard: 2,
                bishop_bitboard: 3,
                queen_bitboard: 4,
                king_square: 5,
                rook_bitboard: 6,
                all_pieces_bitboard: 14,
            },
            Pieces {
                pawn_bitboard: 7,
                knight_bitboard: 8,
                bishop_bitboard: 9,
                queen_bitboard: 10,
                king_square: 11,
                rook_bitboard: 12,
                all_pieces_bitboard: 15,
            },
        ],
        mover: BLACK,
        en_passant_square: 1,
        castle_flags: ALL_CASTLE_FLAGS,
        half_moves: 0,
        move_number: 1,
        zobrist_lock: 0,
    };

    assert_eq!(7, bitboard_for_mover(&p2, Piece::Pawn));
    assert_eq!(8, bitboard_for_mover(&p2, Piece::Knight));
    assert_eq!(9, bitboard_for_mover(&p2, Piece::Bishop));
    assert_eq!(10, bitboard_for_mover(&p2, Piece::Queen));
    assert_eq!(bit(11), bitboard_for_mover(&p2, Piece::King));
    assert_eq!(12, bitboard_for_mover(&p2, Piece::Rook));
}

#[test]
fn it_returns_a_bitboard_with_the_given_bit_set_to_zero() {
    assert_eq!(
        clear_bit(0b0000000001001000000000001000100000010100000101000000100100000001, 0),
        0b0000000001001000000000001000100000010100000101000000100100000000
    );
    assert_eq!(
        clear_bit(0b0000000001001000000000001000100000010100000101000000100100000010, 1),
        0b0000000001001000000000001000100000010100000101000000100100000000
    );
    assert_eq!(
        clear_bit(0b1000000001001000000000001000100000010100000101000000100100000000, 63),
        0b0000000001001000000000001000100000010100000101000000100100000000
    );
}

#[test]
fn it_tests_whether_a_bit_is_set() {
    assert_eq!(
        test_bit(0b0000000001001000000000001000100000010100000101000000100100000001, 0),
        true
    );
    assert_eq!(
        test_bit(0b0000000001001000000000001000100000010100000101000000100100000010, 1),
        true
    );
    assert_eq!(
        test_bit(0b1000000001001000000000001000100000010100000101000000100100000000, 63),
        true
    );
    assert_eq!(
        test_bit(0b0000000001001000000000001000100000010100000101000000100100000000, 0),
        false
    );
    assert_eq!(
        test_bit(0b0000000001001000000000001000100000010100000101000000100100000001, 1),
        false
    );
    assert_eq!(
        test_bit(0b0100000001001000000000001000100000010100000101000000100100000000, 63),
        false
    );
}
