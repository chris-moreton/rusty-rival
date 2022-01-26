use rusty_rival::bitboards::bit;
use rusty_rival::fen::{bitref_from_algebraic_squareref, get_position, move_from_algebraic_move};
use rusty_rival::make_move::{default_position_history, get_move_index, make_move, moving_piece, unmake_move};
use rusty_rival::types::Piece::{Bishop, King, Knight, Pawn, Queen, Rook};
use rusty_rival::types::{BLACK, WHITE};

#[test]
#[ignore]
pub fn it_determines_the_moving_piece() {
    let position = get_position(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string());
    // assert_eq!(moving_piece(&position.pieces[position.mover as usize], bit(bitref_from_algebraic_squareref("e2".to_string()))), Pawn);
    // assert_eq!(moving_piece(&position.pieces[position.mover as usize], bit(bitref_from_algebraic_squareref("a1".to_string()))), Rook);
    // assert_eq!(moving_piece(&position.pieces[position.mover as usize], bit(bitref_from_algebraic_squareref("b1".to_string()))), Knight);
    // assert_eq!(moving_piece(&position.pieces[position.mover as usize], bit(bitref_from_algebraic_squareref("c1".to_string()))), Bishop);
    // assert_eq!(moving_piece(&position.pieces[position.mover as usize], bit(bitref_from_algebraic_squareref("d1".to_string()))), Queen);
    // assert_eq!(moving_piece(&position.pieces[position.mover as usize], bit(bitref_from_algebraic_squareref("e1".to_string()))), King);
    // assert_eq!(moving_piece(&position.pieces[position.mover as usize], bit(bitref_from_algebraic_squareref("e5".to_string()))), King);
}

#[test]
pub fn it_gets_the_halfmove_index_from_the_move_number() {
    assert_eq!(get_move_index(1, WHITE), 1);
    assert_eq!(get_move_index(1, BLACK), 2);
    assert_eq!(get_move_index(2, WHITE), 3);
    assert_eq!(get_move_index(2, BLACK), 4);
    assert_eq!(get_move_index(3, WHITE), 5);
    assert_eq!(get_move_index(3, BLACK), 6);
}

#[test]
#[ignore]
pub fn it_makes_a_move() {
    let mut history = default_position_history();
    
    let original_position = get_position(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string());
    let mut position = original_position.clone();
    make_move(&mut position, move_from_algebraic_move("e2e3".to_string()), &mut history);
    assert_eq!(get_position(&"rnbqkbnr/pppppppp/8/8/8/4P3/PPPP1PPP/RNBQKBNR b KQkq - 0 1".to_string()), position);
    unmake_move(&mut position, &mut history);
    assert_eq!(position, original_position);

    let original_position = get_position(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string());
    let mut position = original_position.clone();
    make_move(&mut position, move_from_algebraic_move("e2e7".to_string()), &mut history);
    assert_eq!(get_position(&"rnbqkbnr/ppppPppp/8/8/8/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1".to_string()), position);
    unmake_move(&mut position, &mut history);
    assert_eq!(position, original_position);
    
    let original_position = get_position(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQK2R w KQkq - 0 1".to_string());
    let mut position = original_position.clone();
    make_move(&mut position, move_from_algebraic_move("e1g1".to_string()), &mut history);
    assert_eq!(get_position(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQ1RK1 b kq - 1 1".to_string()), position);
    unmake_move(&mut position, &mut history);
    assert_eq!(position, original_position);

    let original_position = get_position(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQK2R w KQkq - 0 1".to_string());
    let mut position = original_position.clone();
    make_move(&mut position, move_from_algebraic_move("h1g1".to_string()), &mut history);
    assert_eq!(get_position(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQK1R1 b kqQ - 1 1".to_string()), position);
    unmake_move(&mut position, &mut history);
    assert_eq!(position, original_position);

    let original_position = get_position(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQK2R w KQkq - 0 1".to_string());
    let mut position = original_position.clone();
    make_move(&mut position, move_from_algebraic_move("e2e3".to_string()), &mut history);
    assert_eq!(get_position(&"rnbqkbnr/pppppppp/8/8/8/4P3/PPPP1PPP/RNBQK2R b KQkq - 0 1".to_string()), position);
    unmake_move(&mut position, &mut history);
    assert_eq!(position, original_position);

    let original_position = get_position(&"r3k2r/pppppppp/2n1b3/2bn1q2/8/4P3/PPPP1PPP/RNBQK2R b KQq - 0 1".to_string());
    let mut position = original_position.clone();
    make_move(&mut position, move_from_algebraic_move("e8c8".to_string()), &mut history);
    assert_eq!(get_position(&"2kr3r/pppppppp/2n1b3/2bn1q2/8/4P3/PPPP1PPP/RNBQK2R w KQ - 1 2".to_string()), position);
    unmake_move(&mut position, &mut history);
    assert_eq!(position, original_position);

    let original_position = get_position(&"r3k2r/pppppppp/2n1b3/2bn1q2/8/4P3/PPPP1PPP/RNBQK2R b KQq - 0 1".to_string());
    let mut position = original_position.clone();
    make_move(&mut position, move_from_algebraic_move("e8d8".to_string()), &mut history);
    assert_eq!(get_position(&"r2k3r/pppppppp/2n1b3/2bn1q2/8/4P3/PPPP1PPP/RNBQK2R w KQ - 1 2".to_string()), position);
    unmake_move(&mut position, &mut history);
    assert_eq!(position, original_position);

    let original_position = get_position(&"r3k2r/pppppppp/2n1b3/2bn1q2/8/4P3/PPPP1PPP/RNBQK2R b KQq - 0 1".to_string());
    let mut position = original_position.clone();
    make_move(&mut position, move_from_algebraic_move("h8g8".to_string()), &mut history);
    assert_eq!(get_position(&"r3k1r1/pppppppp/2n1b3/2bn1q2/8/4P3/PPPP1PPP/RNBQK2R w KQq - 1 2".to_string()), position);
    unmake_move(&mut position, &mut history);
    assert_eq!(position, original_position);

    let original_position = get_position(&"2kr3r/pppppp1p/2n1b3/2bn1q2/4Pp2/8/PPPP1PPP/RNBQK2R b KQ e3 15 1".to_string());
    let mut position = original_position.clone();
    make_move(&mut position, move_from_algebraic_move("f4e3".to_string()), &mut history);
    assert_eq!(get_position(&"2kr3r/pppppp1p/2n1b3/2bn1q2/8/4p3/PPPP1PPP/RNBQK2R w KQ - 0 2".to_string()), position);
    unmake_move(&mut position, &mut history);
    assert_eq!(position, original_position);

    let original_position = get_position(&"2kr3r/ppppppPp/2n1b3/2bn1q2/8/4p3/PPPP1P1P/RNBQK2R w KQ - 12 1".to_string());
    let mut position = original_position.clone();
    make_move(&mut position, move_from_algebraic_move("g7h8r".to_string()), &mut history);
    assert_eq!(get_position(&"2kr3R/pppppp1p/2n1b3/2bn1q2/8/4p3/PPPP1P1P/RNBQK2R b KQ - 0 1".to_string()), position);
    unmake_move(&mut position, &mut history);
    assert_eq!(position, original_position);

    let original_position = get_position(&"2kr3R/pppp1p1p/2n1b3/2bn1q2/8/4p3/PPPP1PpP/RNBQK2R b KQ - 0 1".to_string());
    let mut position = original_position.clone();
    make_move(&mut position, move_from_algebraic_move("g2g1q".to_string()), &mut history);
    assert_eq!(get_position(&"2kr3R/pppp1p1p/2n1b3/2bn1q2/8/4p3/PPPP1P1P/RNBQK1qR w KQ - 0 2".to_string()), position);
    unmake_move(&mut position, &mut history);
    assert_eq!(position, original_position);

    let original_position = get_position(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string());
    let mut position = original_position.clone();
    make_move(&mut position, move_from_algebraic_move("e2e4".to_string()), &mut history);
    assert_eq!(get_position(&"rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1".to_string()), position);
    unmake_move(&mut position, &mut history);
    assert_eq!(position, original_position);

    let original_position = get_position(&"8/p7/8/1P6/K1k3pP/6P1/8/8 b - h3 0 1".to_string());
    let mut position = original_position.clone();
    make_move(&mut position, move_from_algebraic_move("a7a6".to_string()), &mut history);
    assert_eq!(get_position(&"8/8/p7/1P6/K1k3pP/6P1/8/8 w - - 0 2".to_string()), position);
    unmake_move(&mut position, &mut history);
    assert_eq!(position, original_position);

    let original_position = get_position(&"8/p7/8/1P6/K1k3pP/6P1/8/8 b - h3 0 1".to_string());
    let mut position = original_position.clone();
    make_move(&mut position, move_from_algebraic_move("a7a5".to_string()), &mut history);
    assert_eq!(get_position(&"8/8/8/pP6/K1k3pP/6P1/8/8 w - a6 0 2".to_string()), position);
    unmake_move(&mut position, &mut history);
    assert_eq!(position, original_position);

    let original_position = get_position(&"8/p7/8/1P6/K1k3pP/6P1/8/8 b - h3 0 1".to_string());
    let mut position = original_position.clone();
    make_move(&mut position, move_from_algebraic_move("c4c5".to_string()), &mut history);
    assert_eq!(get_position(&"8/p7/8/1Pk5/K5pP/6P1/8/8 w - - 1 2".to_string()), position);
    unmake_move(&mut position, &mut history);
    assert_eq!(position, original_position);
}
