use rusty_rival::bitboards::bitboards::bit;
use rusty_rival::fen::fen::{algebraic_move_from_move, get_position};
use rusty_rival::moves::moves::{all_bits_except_friendly_pieces, generate_king_moves, generate_knight_moves, generate_slider_moves, moves_from_to_squares_bitboard};
use rusty_rival::types::types::Piece::{Bishop, Rook};

#[test]
fn it_gets_all_bits_except_friendly_pieces() {
    let position = get_position(&"n5k1/6n1/1n2q2p/1p5P/1P3RP1/2PK1B2/1r2N3/8 b kQKq g3 5 56".to_string());
    assert_eq!(all_bits_except_friendly_pieces(&position), 0b0111110111111101101101101011111111111111111111111011111111111111);
}

#[test]
fn it_gets_all_pieces_bitboard() {
    let position = get_position(&"n5k1/6n1/1n2q2p/1p5P/1P3RP1/2PK1B2/1r2N3/8 b kQKq g3 5 56".to_string());
    assert_eq!(position.all_pieces_bitboard, 0b1000001000000010010010010100000101000110001101000100100000000000);
}

#[test]
fn it_creates_a_list_of_moves_from_a_from_square_and_a_list_of_to_squares() {
    let mut moves = moves_from_to_squares_bitboard(11, bit(22) | bit(33) | bit(44));
    moves.sort();
    assert_eq!(moves, vec![720918,720929,720940]);
}

#[test]
fn it_generates_knight_moves_from_a_given_fen_ignoring_checks() {
    let position = get_position(&"n5k1/6n1/1n2q2p/1p5P/1P3RP1/2PK1B2/1r2N3/8 b kQKq g3 5 56".to_string());
    let move_list = generate_knight_moves(&position);
    let mut algebraic: Vec<String> = move_list.iter().map(|m| { algebraic_move_from_move(*m) }).collect();
    algebraic.sort();
    assert_eq!(algebraic, vec!["a8c7","b6a4","b6c4","b6c8","b6d5","b6d7","g7e8","g7f5","g7h5"]);

    let position = get_position(&"n5k1/6n1/1n2q2p/1p5P/1P3RP1/2PK1B2/1r2N3/8 w kQKq g3 5 56".to_string());
    let move_list = generate_knight_moves(&position);
    let mut algebraic: Vec<String> = move_list.iter().map(|m| { algebraic_move_from_move(*m) }).collect();
    algebraic.sort();
    assert_eq!(algebraic, vec!["e2c1","e2d4","e2g1","e2g3"]);
}

#[test]
fn it_generates_king_moves_from_a_given_fen_ignoring_checks() {
    let position = get_position(&"n5k1/6n1/1n2q2p/1p5P/1P3RP1/2PK1B2/1r2N3/8 b kQKq g3 5 56".to_string());
    let move_list = generate_king_moves(&position);
    let mut algebraic: Vec<String> = move_list.iter().map(|m| { algebraic_move_from_move(*m) }).collect();
    algebraic.sort();
    assert_eq!(algebraic, vec!["g8f7","g8f8","g8h7","g8h8"]);

    let position = get_position(&"n5k1/6n1/1n2q2p/1p5P/1P3RP1/2PK1B2/1r2N3/8 w kQKq g3 5 56".to_string());
    let move_list = generate_king_moves(&position);
    let mut algebraic: Vec<String> = move_list.iter().map(|m| { algebraic_move_from_move(*m) }).collect();
    algebraic.sort();
    assert_eq!(algebraic, vec!["d3c2","d3c4","d3d2","d3d4","d3e3","d3e4"]);
}

#[test]
fn it_generates_bishop_moves_including_diagonal_queen_moves_from_a_given_fen_ignoring_checks() {
    let position = get_position(&"n5k1/6n1/1n2q2p/1p5P/1P3RP1/2PK1B2/1r2N3/7R w kQKq g3 5 56".to_string());
    let move_list = generate_slider_moves(&position, Bishop);
    let mut algebraic: Vec<String> = move_list.iter().map(|m| { algebraic_move_from_move(*m) }).collect();
    algebraic.sort();
    assert_eq!(algebraic, vec!["f3a8","f3b7","f3c6","f3d5","f3e4","f3g2"]);

    let position = get_position(&"n5k1/6n1/1n2q2p/1p5P/1P3RP1/2PK1B2/1r2N3/8 b kQKq g3 5 56".to_string());
    let move_list = generate_slider_moves(&position, Bishop);
    let mut algebraic: Vec<String> = move_list.iter().map(|m| { algebraic_move_from_move(*m) }).collect();
    algebraic.sort();
    assert_eq!(algebraic, vec!["e6a2","e6b3","e6c4","e6c8","e6d5","e6d7","e6f5","e6f7","e6g4"]);
}

#[test]
fn it_generates_rook_moves_including_horizontal_queen_moves_from_a_given_fen_ignoring_checks() {
    let position = get_position(&"n5k1/6n1/1n2q2p/1p5P/1P3RP1/2PK1B2/1r2N3/8 w kQKq g3 5 56".to_string());
    let move_list = generate_slider_moves(&position, Rook);
    let mut algebraic: Vec<String> = move_list.iter().map(|m| { algebraic_move_from_move(*m) }).collect();
    algebraic.sort();
    assert_eq!(algebraic, vec!["f4c4","f4d4","f4e4","f4f5","f4f6","f4f7","f4f8"]);

    let position = get_position(&"n5k1/6n1/1n2q2p/1p5P/1P3RP1/2PK1B2/1r2N3/6r1 b kQKq g3 5 56".to_string());
    let move_list = generate_slider_moves(&position, Rook);
    let mut algebraic: Vec<String> = move_list.iter().map(|m| { algebraic_move_from_move(*m) }).collect();
    algebraic.sort();
    assert_eq!(algebraic, vec!["b2a2","b2b1","b2b3","b2b4","b2c2","b2d2","b2e2","e6c6","e6d6","e6e2","e6e3","e6e4","e6e5","e6e7","e6e8","e6f6","e6g6","g1a1","g1b1","g1c1","g1d1","g1e1","g1f1","g1g2","g1g3","g1g4","g1h1"]);
}