use rusty_rival::bitboards::bitboards::{bit, EMPTY_CASTLE_SQUARES_WHITE_QUEEN, empty_squares_bitboard, enemy_bitboard, WHITE_PAWN_MOVES_CAPTURE, WHITE_PAWN_MOVES_FORWARD};
use rusty_rival::fen::fen::{algebraic_move_from_move, get_position};
use rusty_rival::move_constants::move_constants::EN_PASSANT_NOT_AVAILABLE;
use rusty_rival::moves::moves::{all_bits_except_friendly_pieces, any_squares_in_bitboard_attacked, generate_king_moves, generate_knight_moves, generate_pawn_moves, generate_pawn_moves_from_to_squares, generate_slider_moves, is_square_attacked_by, moves_from_to_squares_bitboard, pawn_captures, pawn_forward_and_capture_moves_bitboard, pawn_forward_moves_bitboard, potential_pawn_jump_moves};
use rusty_rival::types::types::Piece::{Bishop, Rook};
use rusty_rival::types::types::{Bitboard, Square};
use rusty_rival::types::types::Mover::{Black, White};

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

#[test]
fn it_creates_a_list_of_moves_from_a_given_from_square_and_a_list_of_to_squares() {
    let move_list = generate_pawn_moves_from_to_squares(54, bit(63) | bit(62) | bit(61));
    let mut algebraic: Vec<String> = move_list.iter().map(|m| { algebraic_move_from_move(*m) }).collect();
    algebraic.sort();
    assert_eq!(algebraic, vec!["b7a8b","b7a8n","b7a8q","b7a8r","b7b8b","b7b8n","b7b8q","b7b8r","b7c8b","b7c8n","b7c8q","b7c8r"]);

    let move_list = generate_pawn_moves_from_to_squares(46, bit(55) | bit(54) | bit(53));
    let mut algebraic: Vec<String> = move_list.iter().map(|m| { algebraic_move_from_move(*m) }).collect();
    algebraic.sort();
    assert_eq!(algebraic, vec!["b6a7","b6b7","b6c7"]);
}

#[test]
fn it_returns_a_bitboard_showing_target_squares_for_pawn_captures_from_a_given_square_and_an_enemy_piece_bitboard() {
    let position = get_position(&"n5k1/1P4n1/1n2q2p/Pp3P2/3P1R2/3K1B2/1r2N2P/6r1 w - - 0 1".to_string());
    assert_eq!(pawn_captures(WHITE_PAWN_MOVES_CAPTURE, 29, enemy_bitboard(&position)), 0b0000000000000000000000000100000000000000000000000000000000000000);

    let position = get_position(&"n5k1/1P2P1n1/1n2q2p/Pp1pP3/3P1R2/3K1B2/1r2N2P/6r1 w - - 0 1".to_string());
    assert_eq!(pawn_captures(WHITE_PAWN_MOVES_CAPTURE, 51, enemy_bitboard(&position)), 0b0000000000000000000000000000000000000000000000000000000000000000);

    let position = get_position(&"n5k1/1P2P1n1/1n2q2p/Pp1pP3/3P1R2/3K1B2/1r2N2P/6r1 w - - 0 1".to_string());
    assert_eq!(pawn_captures(WHITE_PAWN_MOVES_CAPTURE, 54, enemy_bitboard(&position)), 0b1000000000000000000000000000000000000000000000000000000000000000);

    let position = get_position(&"n5k1/4P1n1/1n2q2p/1p1p4/5R2/3K1B2/1r2N3/6r1 w - - 0 1".to_string());
    assert_eq!(pawn_captures(WHITE_PAWN_MOVES_CAPTURE, 51, enemy_bitboard(&position)), 0b0000000000000000000000000000000000000000000000000000000000000000);
}

#[test]
fn it_returns_a_bitboard_showing_target_squares_for_pawn_moves_that_would_land_on_the_two_move_rank_if_moved_one_more_rank() {
    let position = get_position(&"n5k1/1P2P1n1/1n2q2p/Pp6/3P1R2/3K1B2/1r2N2P/6r1 w - d5 0 1".to_string());
    assert_eq!(potential_pawn_jump_moves(0b0101000000000100010000011000000001000000010101010000001100010001, &position), 0b0000000000000000000000000000000001010101000000000000000000000000);
}

#[test]
fn it_identifies_the_en_passant_square_from_a_fen() {
    let position = get_position(&"n5k1/4P1n1/4q2p/PpP1n3/3P1R2/3K1B2/1r2N2P/6r1 w - b6 0 1".to_string());
    assert_eq!(position.en_passant_square, 46);

    let position = get_position(&"n5k1/4P1n1/4q2p/PpP1n3/3P1R2/3K1B2/1r2N2P/6r1 w - - 0 1".to_string());
    assert_eq!(position.en_passant_square, EN_PASSANT_NOT_AVAILABLE);
}

#[test]
fn it_returns_a_bitboard_showing_available_landing_squares_capture_and_non_capture_for_a_pawn_on_a_given_square() {
    let position = get_position(&"n5k1/4P1n1/1n2q2p/1p1p4/5R2/3K1B2/1r2N3/6r1 w - - 0 1".to_string());
    let empty_squares = empty_squares_bitboard(&position);
    let from_square = 51;
    let forward_moves_for_square = WHITE_PAWN_MOVES_FORWARD.iter().nth(from_square).unwrap();
    assert_eq!(*forward_moves_for_square, 0b0000100000000000000000000000000000000000000000000000000000000000);
    let pfmb = pawn_forward_moves_bitboard (forward_moves_for_square & empty_squares, &position);
    assert_eq!(pfmb, 0b0000100000000000000000000000000000000000000000000000000000000000);
    assert_eq!(pawn_forward_and_capture_moves_bitboard(from_square as Square, WHITE_PAWN_MOVES_CAPTURE, pfmb, &position), 0b0000100000000000000000000000000000000000000000000000000000000000);
}

#[test]
fn it_generates_pawn_moves_from_a_given_fen_ignoring_checks() {
    let position = get_position(&"n5k1/4P1n1/1n2q2p/1p1p4/5R2/3K1B2/1r2N3/6r1 w - - 0 1".to_string());
    let move_list = generate_pawn_moves(&position);
    let mut algebraic: Vec<String> = move_list.iter().map(|m| { algebraic_move_from_move(*m) }).collect();
    algebraic.sort();
    assert_eq!(algebraic, vec!["e7e8b","e7e8n","e7e8q","e7e8r"]);

    let position = get_position(&"n5k1/1P2P1n1/1n2q2p/P1pP4/5R2/3K1B2/1r2N2P/6r1 w - c6 0 1".to_string());
    let move_list = generate_pawn_moves(&position);
    let mut algebraic: Vec<String> = move_list.iter().map(|m| { algebraic_move_from_move(*m) }).collect();
    algebraic.sort();
    assert_eq!(algebraic, vec!["a5a6","a5b6","b7a8b","b7a8n","b7a8q","b7a8r","b7b8b","b7b8n","b7b8q","b7b8r","d5c6","d5d6","d5e6","e7e8b","e7e8n","e7e8q","e7e8r","h2h3","h2h4"]);
}

#[test]
fn it_returns_true_if_any_squares_set_in_the_bitboard_are_attacked_by_the_given_attacker() {
    let position = get_position(&"n5k1/1P2P1n1/1n2q2p/P1pP4/5R2/5B2/1r2N2P/R3K1r1 w Q - 0 1".to_string());
    let bitboard = 0b0000000000000000000000000000000000000000010110000000000000000000;
    assert_eq!(any_squares_in_bitboard_attacked(&position, &White, bitboard), false);

    let bitboard = 0b0000000000000000000000000000000000000000111110000000000000000000;
    assert_eq!(any_squares_in_bitboard_attacked(&position, &White, bitboard), true);
    assert_eq!(any_squares_in_bitboard_attacked(&position, &White, bit(60) | bit (61)), true);
    assert_eq!(any_squares_in_bitboard_attacked(&position, &Black, EMPTY_CASTLE_SQUARES_WHITE_QUEEN), true);

    let position = get_position(&"n5k1/1P2P1n1/1n2q2p/P1pP4/5R2/5B2/1r2N2P/R3K2R w Q - 0 1".to_string());
    assert_eq!(any_squares_in_bitboard_attacked(&position, &Black, bit(3) | bit (2)), false);
    assert_eq!(any_squares_in_bitboard_attacked(&position, &Black, bit(3) | bit (4)), false);

    let position = get_position(&"n5k1/1P2P1n1/1n5p/P1pP4/5R2/1q3B2/4Nr1P/R3K2R w Q - 0 1".to_string());
    assert_eq!(any_squares_in_bitboard_attacked(&position, &Black, bit(3) | bit (2)), true);
    assert_eq!(any_squares_in_bitboard_attacked(&position, &Black, 0b0000000000000000000000000000000000000000000000000000000000011000), true);

    let position = get_position(&"r3k2r/1P2P1n1/1n2q2p/P1pP4/5R2/5B2/1r2N2P/R3K2R b Q - 0 1".to_string());
    assert_eq!(is_square_attacked_by(&position, 60, &White), true);
    assert_eq!(any_squares_in_bitboard_attacked(&position, &White, 0b0001100000000000000000000000000000000000000000000000000000000000), true);
    assert_eq!(any_squares_in_bitboard_attacked(&position, &White, bit(59) | bit (60)), true);
}
