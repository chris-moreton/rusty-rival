use rusty_rival::fen::get_position;
use rusty_rival::piece_square_tables::{
    black_bishop_piece_square_values, black_knight_piece_square_values, black_pawn_piece_square_values, non_pawn_piece_values, pawn_values,
    white_bishop_piece_square_values, white_knight_piece_square_values, white_pawn_piece_square_values,
};
use rusty_rival::types::{Score, BLACK, WHITE};
use rusty_rival::utils::{invert_fen, invert_pos};

#[test]
fn it_calculates_the_pawn_piece_square_values() {
    let position = get_position(&"nr4k1/1P2P1n1/5q1p/P1pP4/5R2/5B2/1r2N2P/R1Q1K1n1 w Q - 0 1".to_string());
    assert_eq!(
        white_pawn_piece_square_values(&position, non_pawn_piece_values(&position.pieces[BLACK as usize])),
        57
    );
    let position = get_position(&invert_fen("nr4k1/1P2P1n1/5q1p/P1pP4/5R2/5B2/1r2N2P/R1Q1K1n1 w Q - 0 1").to_string());
    assert_eq!(
        black_pawn_piece_square_values(&position, non_pawn_piece_values(&position.pieces[WHITE as usize])),
        57
    );

    let position = get_position(&"6k1/1P2P3/7p/P1pP4/5R2/5B2/1r2N2P/R1Q1K1n1 w Q - 0 1".to_string());
    assert_eq!(
        white_pawn_piece_square_values(&position, non_pawn_piece_values(&position.pieces[BLACK as usize])),
        90
    );
    let position = get_position(&invert_fen("6k1/1P2P3/7p/P1pP4/5R2/5B2/1r2N2P/R1Q1K1n1 w Q - 0 1").to_string());
    assert_eq!(
        black_pawn_piece_square_values(&position, non_pawn_piece_values(&position.pieces[WHITE as usize])),
        90
    );

    let position = get_position(&"6k1/1P2P3/7p/P1pP4/5R2/5B2/1r2N2P/R1Q1K3 w Q - 0 1".to_string());
    assert_eq!(
        white_pawn_piece_square_values(&position, non_pawn_piece_values(&position.pieces[BLACK as usize])),
        95
    );
    let position = get_position(&invert_fen("6k1/1P2P3/7p/P1pP4/5R2/5B2/1r2N2P/R1Q1K3 w Q - 0 1").to_string());
    assert_eq!(
        black_pawn_piece_square_values(&position, non_pawn_piece_values(&position.pieces[WHITE as usize])),
        95
    );
}

#[test]
fn it_calculates_the_knight_piece_square_values() {
    let position = get_position(&"nr4k1/1P2P1n1/5q1p/P1pP4/5R2/5B2/1r2N2P/R1Q1K1n1 w Q - 0 1".to_string());
    assert_eq!(
        black_knight_piece_square_values(&position, non_pawn_piece_values(&position.pieces[BLACK as usize])),
        -120
    );
    let position = get_position(&invert_fen("nr4k1/1P2P1n1/5q1p/P1pP4/5R2/5B2/1r2N2P/R1Q1K1n1 w Q - 0 1").to_string());
    assert_eq!(
        white_knight_piece_square_values(&position, non_pawn_piece_values(&position.pieces[WHITE as usize])),
        -120
    );

    let position = get_position(&"6k1/1P2P3/7p/P1pP4/8/4nB2/1r2N2P/R3K3 w Q - 0 1".to_string());
    assert_eq!(
        black_knight_piece_square_values(
            &position,
            pawn_values(&position.pieces[WHITE as usize]) + non_pawn_piece_values(&position.pieces[WHITE as usize])
        ),
        11
    );
    let position = get_position(&invert_fen("6k1/1P2P3/7p/P1pP4/8/4nB2/1r2N2P/R3K3 w Q - 0 1").to_string());
    assert_eq!(
        white_knight_piece_square_values(
            &position,
            pawn_values(&position.pieces[BLACK as usize]) + non_pawn_piece_values(&position.pieces[BLACK as usize])
        ),
        11
    );
}

fn test_bishop_piece_squares(fen: &str, score: Score) {
    let position = get_position(fen);
    assert_eq!(white_bishop_piece_square_values(&position), score);
    assert_eq!(black_bishop_piece_square_values(&invert_pos(&position)), score);
}

#[test]
fn it_calculates_the_bishop_piece_square_values() {
    test_bishop_piece_squares(&"nr4k1/1P2P1n1/5q1p/P1pP4/5R2/5B2/1r2N2P/R1Q1K1n1 w Q - 0 1".to_string(), 5);
    test_bishop_piece_squares(&"nr4k1/1P2P1n1/5q1p/P1pP4/5R2/8/1r2NB1P/R1Q1K1n1 w Q - 0 1".to_string(), 2);
    test_bishop_piece_squares(&"nr4k1/1P2P1n1/5q1p/P1pP4/5R2/8/1rB1NB1P/R1Q1K1n1 w Q - 0 1".to_string(), 4);
    test_bishop_piece_squares(&"nr4k1/1P2P1n1/5q1p/P1pP4/5R2/8/1r2N2P/R1Q1KBn1 w Q - 0 1".to_string(), 0);
    test_bishop_piece_squares(&"6k1/1P2P3/7p/P1pP4/8/4nB2/1r2N2P/R3K3 w Q - 0 1".to_string(), 5);
    test_bishop_piece_squares(&"6k1/1P2P3/7p/P1p5/8/4n3/1r2N1PP/R3K2B w Q - 0 1".to_string(), 0);
}
