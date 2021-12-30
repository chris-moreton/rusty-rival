use rusty_rival::fen::fen::{bitref_from_algebraic_squareref, get_position, move_from_algebraic_move};
use rusty_rival::make_move::make_move::{make_move, moving_piece};
use rusty_rival::types::types::Piece::{Bishop, King, Knight, Pawn, Queen, Rook};
use rusty_rival::types::types::{Move, Position};

#[test]
pub fn it_determines_the_moving_piece() {
    let position = get_position(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string());
    assert_eq!(moving_piece(&position, bitref_from_algebraic_squareref("e2".to_string())), Pawn);
    assert_eq!(moving_piece(&position, bitref_from_algebraic_squareref("a1".to_string())), Rook);
    assert_eq!(moving_piece(&position, bitref_from_algebraic_squareref("b1".to_string())), Knight);
    assert_eq!(moving_piece(&position, bitref_from_algebraic_squareref("c1".to_string())), Bishop);
    assert_eq!(moving_piece(&position, bitref_from_algebraic_squareref("d1".to_string())), Queen);
    assert_eq!(moving_piece(&position, bitref_from_algebraic_squareref("e1".to_string())), King);
    assert_eq!(moving_piece(&position, bitref_from_algebraic_squareref("e5".to_string())), King);
}

#[test]
pub fn it_makes_a_move() {
    let mut position = get_position(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string());
    make_move(&mut position, move_from_algebraic_move("e2e3".to_string()));
    assert_eq!(get_position(&"rnbqkbnr/pppppppp/8/8/8/4P3/PPPP1PPP/RNBQKBNR b KQkq - 0 1".to_string()), position);

    let mut position = get_position(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string());
    make_move(&mut position, move_from_algebraic_move("e2e7".to_string()));
    assert_eq!(get_position(&"rnbqkbnr/ppppPppp/8/8/8/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1".to_string()), position);
}


// makeMove (get_position(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQK2R w KQkq - 0 1".to_string()))
// (moveFromAlgebraicMove "e1g1")
// `shouldBe` get_position(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQ1RK1 b kq - 1 1".to_string())
// makeMove (get_position(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQK2R w KQkq - 0 1".to_string()))
// (moveFromAlgebraicMove "h1g1")
// `shouldBe` get_position(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQK1R1 b kqQ - 1 1".to_string())
// makeMove (get_position(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQK2R w KQkq - 0 1".to_string()))
// (moveFromAlgebraicMove "e2e3")
// `shouldBe` get_position(&"rnbqkbnr/pppppppp/8/8/8/4P3/PPPP1PPP/RNBQK2R b KQkq - 0 1".to_string())
// makeMove (get_position(&"r3k2r/pppppppp/2n1b3/2bn1q2/8/4P3/PPPP1PPP/RNBQK2R b KQq - 0 1".to_string()))
// (moveFromAlgebraicMove "e8c8")
// `shouldBe` get_position(&"2kr3r/pppppppp/2n1b3/2bn1q2/8/4P3/PPPP1PPP/RNBQK2R w KQ - 1 2".to_string())
// makeMove (get_position(&"r3k2r/pppppppp/2n1b3/2bn1q2/8/4P3/PPPP1PPP/RNBQK2R b KQq - 0 1".to_string()))
// (moveFromAlgebraicMove "e8d8")
// `shouldBe` get_position(&"r2k3r/pppppppp/2n1b3/2bn1q2/8/4P3/PPPP1PPP/RNBQK2R w KQ - 1 2".to_string())
// makeMove (get_position(&"r3k2r/pppppppp/2n1b3/2bn1q2/8/4P3/PPPP1PPP/RNBQK2R b KQq - 0 1".to_string()))
// (moveFromAlgebraicMove "h8g8")
// `shouldBe` get_position(&"r3k1r1/pppppppp/2n1b3/2bn1q2/8/4P3/PPPP1PPP/RNBQK2R w KQq - 1 2".to_string())
// makeMove (get_position(&"2kr3r/pppppp1p/2n1b3/2bn1q2/4Pp2/8/PPPP1PPP/RNBQK2R b KQ e3 15 1".to_string()))
// (moveFromAlgebraicMove "f4e3")
// `shouldBe` get_position(&"2kr3r/pppppp1p/2n1b3/2bn1q2/8/4p3/PPPP1PPP/RNBQK2R w KQ - 0 2".to_string())
// makeMove (get_position(&"2kr3r/ppppppPp/2n1b3/2bn1q2/8/4p3/PPPP1P1P/RNBQK2R w KQ - 12 1".to_string()))
// (moveFromAlgebraicMove "g7h8r")
// `shouldBe` get_position(&"2kr3R/pppppp1p/2n1b3/2bn1q2/8/4p3/PPPP1P1P/RNBQK2R b KQ - 0 1".to_string())
// makeMove (get_position(&"2kr3R/pppp1p1p/2n1b3/2bn1q2/8/4p3/PPPP1PpP/RNBQK2R b KQ - 0 1".to_string()))
// (moveFromAlgebraicMove "g2g1q")
// `shouldBe` get_position(&"2kr3R/pppp1p1p/2n1b3/2bn1q2/8/4p3/PPPP1P1P/RNBQK1qR w KQ - 0 2".to_string())
// makeMove (get_position(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string()))
// (moveFromAlgebraicMove "e2e4")
// `shouldBe` get_position(&"rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1".to_string())
//