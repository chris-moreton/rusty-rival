use rusty_rival::fen::{get_fen, get_position};
use rusty_rival::piece_square_tables::{black_pawn_piece_square_values, non_pawn_piece_values, piece_square_values, white_pawn_piece_square_values};
use rusty_rival::types::{BLACK, WHITE};
use rusty_rival::utils::invert_fen;

#[test]
fn it_calculates_the_pawn_piece_square_values() {
    let position = get_position(&"nr4k1/1P2P1n1/5q1p/P1pP4/5R2/5B2/1r2N2P/R1Q1K1n1 w Q - 0 1".to_string());
    assert_eq!(white_pawn_piece_square_values(&position, non_pawn_piece_values(&position.pieces[BLACK as usize])), 57);
    let position = get_position(&invert_fen("nr4k1/1P2P1n1/5q1p/P1pP4/5R2/5B2/1r2N2P/R1Q1K1n1 w Q - 0 1").to_string());
    assert_eq!(black_pawn_piece_square_values(&position, non_pawn_piece_values(&position.pieces[WHITE as usize])), 57);

    let position = get_position(&"6k1/1P2P3/7p/P1pP4/5R2/5B2/1r2N2P/R1Q1K1n1 w Q - 0 1".to_string());
    assert_eq!(white_pawn_piece_square_values(&position, non_pawn_piece_values(&position.pieces[BLACK as usize])), 91);
    let position = get_position(&invert_fen("6k1/1P2P3/7p/P1pP4/5R2/5B2/1r2N2P/R1Q1K1n1 w Q - 0 1").to_string());
    assert_eq!(black_pawn_piece_square_values(&position, non_pawn_piece_values(&position.pieces[WHITE as usize])), 91);

    let position = get_position(&"6k1/1P2P3/7p/P1pP4/5R2/5B2/1r2N2P/R1Q1K3 w Q - 0 1".to_string());
    assert_eq!(white_pawn_piece_square_values(&position, non_pawn_piece_values(&position.pieces[BLACK as usize])), 95);
    let position = get_position(&invert_fen("6k1/1P2P3/7p/P1pP4/5R2/5B2/1r2N2P/R1Q1K3 w Q - 0 1").to_string());
    assert_eq!(black_pawn_piece_square_values(&position, non_pawn_piece_values(&position.pieces[WHITE as usize])), 95);
}
