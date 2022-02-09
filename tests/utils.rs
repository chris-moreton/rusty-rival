use rusty_rival::bitboards::{C8_BIT, F8_BIT, G8_BIT, H2_BIT, H4_BIT, H8_BIT};
use rusty_rival::fen::{get_position, move_from_algebraic_move};
use rusty_rival::make_move::make_move;
use rusty_rival::move_constants::{BLACK_KING_CASTLE_MOVE_MASK, BLACK_QUEEN_CASTLE_MOVE_MASK, PIECE_MASK_KING, PIECE_MASK_KNIGHT, PIECE_MASK_PAWN, PIECE_MASK_ROOK, START_POS, WHITE_KING_CASTLE_MOVE, WHITE_KING_CASTLE_MOVE_MASK, WHITE_QUEEN_CASTLE_MOVE_MASK};
use rusty_rival::types::Move;
use rusty_rival::utils::{castle_mask, from_square_mask, from_square_part, hydrate_move_from_algebraic_move, moving_piece_mask, to_square_part};

#[test]
fn it_creates_a_move_with_the_from_part_only() {
    assert_eq!(from_square_mask(21), 0b00000000000101010000000000000000);
}

#[test]
fn it_gets_the_from_part_of_a_move() {
    assert_eq!(from_square_part(0b00000000000101010000000000111100), 21);
}

#[test]
fn it_gets_the_to_part_of_a_move() {
    assert_eq!(to_square_part(0b00000000000101010000000000111100), 60);
}

#[test]
pub fn it_can_figure_out_the_moving_piece() {
    let position = &get_position(&START_POS.to_string());
    assert_eq!(moving_piece_mask(position, move_from_algebraic_move("e2e3".to_string(), 0)), PIECE_MASK_PAWN);
    assert_eq!(moving_piece_mask(position, move_from_algebraic_move("g1f3".to_string(), 0)), PIECE_MASK_KNIGHT);
}

#[test]
pub fn it_can_figure_out_the_castle_mask() {
    let position = &get_position(&"r3k2r/pppp1ppp/1bnb1n2/4p1q1/3PP3/1BNB1N2/PPP1QPPP/R3K2R w KQkq - 0 1".to_string());
    assert_eq!(castle_mask(position, move_from_algebraic_move("e1g1".to_string(), 0)), WHITE_KING_CASTLE_MOVE_MASK);
    assert_eq!(castle_mask(position, move_from_algebraic_move("e1c1".to_string(), 0)), WHITE_QUEEN_CASTLE_MOVE_MASK);
    assert_eq!(castle_mask(position, move_from_algebraic_move("e8g8".to_string(), 0)), BLACK_KING_CASTLE_MOVE_MASK);
    assert_eq!(castle_mask(position, move_from_algebraic_move("e8c8".to_string(), 0)), BLACK_QUEEN_CASTLE_MOVE_MASK);
    assert_eq!(castle_mask(position, move_from_algebraic_move("e1d1".to_string(), 0)), 0);
}

#[test]
pub fn it_can_hydrate_a_move() {
    let position = &get_position(&"r3k2r/pppp1ppp/1bnb1n2/4p1q1/3PP3/1BNB1N2/PPP1QPPP/R3K2R w KQkq - 0 1".to_string());
    assert_eq!(hydrate_move_from_algebraic_move(position, "e1g1".to_string()), WHITE_KING_CASTLE_MOVE);
    assert_eq!(hydrate_move_from_algebraic_move(position, "h2h4".to_string()), from_square_mask(H2_BIT) | H4_BIT as Move | PIECE_MASK_PAWN);

    let position = &get_position(&"5rk1/5p1p/6p1/8/4Q3/2q5/4RPPP/6K1 b - - 0 1".to_string());
    assert_eq!(hydrate_move_from_algebraic_move(position, "f8c8".to_string()), from_square_mask(F8_BIT) | C8_BIT as Move | PIECE_MASK_ROOK);

    let position = &get_position(&"7k/pB3p1p/2p4b/3p2p1/3P3P/P1PNB3/1P3PP1/6K1 b - - 0 1".to_string());
    assert_eq!(hydrate_move_from_algebraic_move(position, "h8g8".to_string()), from_square_mask(H8_BIT) | G8_BIT as Move | PIECE_MASK_KING);

}

