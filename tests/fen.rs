use rusty_rival::fen::*;
use rusty_rival::fen::fen::{algebraic_squareref_from_bitref, bit_array_to_decimal, bitref_from_algebraic_squareref, board_bits, char_as_num, fen_board_part, get_fen_ranks, piece_bitboard, rank_bits};

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

// describe "boardFromFen" $ do
// it "Converts from FEN to board type (Test 1)" $ do
// let fen = "6k1/6p1/1p2q2p/1p5P/1P3RP1/2PK1B2/1r2N3/8 b q g3 5 56"
// let position = getPosition fen
// let bitboards = position
// whitePawnBitboard bitboards `shouldBe` 5404360704
// whiteKnightBitboard bitboards `shouldBe` 2048
// whiteKingBitboard bitboards `shouldBe` 1048576
// whiteBishopBitboard bitboards `shouldBe` 262144
// whiteQueenBitboard bitboards `shouldBe` 0
// whiteRookBitboard bitboards `shouldBe` 67108864
// blackPawnBitboard bitboards `shouldBe` 634693087133696
// blackKnightBitboard bitboards `shouldBe` 0
// blackKingBitboard bitboards `shouldBe` 144115188075855872
// blackBishopBitboard bitboards `shouldBe` 0
// blackQueenBitboard bitboards `shouldBe` 8796093022208
// blackRookBitboard bitboards `shouldBe` 16384
// mover position `shouldBe` Black
// enPassantSquare position `shouldBe` 17
//
// it "Converts from FEN to board type (Test 2)" $ do
// let fen = "6k1/6p1/1p2q2p/1p5P/1P3RP1/2PK1B2/1r2N3/8 w kQ - 5 56"
// let position = getPosition fen
// let castlePrivs = position
// enPassantSquare position `shouldBe` enPassantNotAvailable
// halfMoves position `shouldBe` 5
// mover position `shouldBe` White
// whiteKingCastleAvailable castlePrivs `shouldBe` False
// whiteQueenCastleAvailable castlePrivs `shouldBe` True
// blackKingCastleAvailable castlePrivs `shouldBe` True
// blackQueenCastleAvailable castlePrivs `shouldBe` False

