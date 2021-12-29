use rusty_rival::bitboards::bitboards::{bit, EMPTY_CASTLE_SQUARES_WHITE_QUEEN, empty_squares_bitboard, enemy_bitboard, WHITE_PAWN_MOVES_CAPTURE, WHITE_PAWN_MOVES_FORWARD};
use rusty_rival::fen::fen::{algebraic_move_from_move, bitref_from_algebraic_squareref, get_position};
use rusty_rival::move_constants::move_constants::EN_PASSANT_NOT_AVAILABLE;
use rusty_rival::moves::moves::{all_bits_except_friendly_pieces, any_squares_in_bitboard_attacked, generate_castle_moves, generate_king_moves, generate_knight_moves, generate_pawn_moves, generate_pawn_moves_from_to_squares, generate_slider_moves, is_bishop_attacking_square, is_check, is_square_attacked_by, moves, moves_from_to_squares_bitboard, pawn_captures, pawn_forward_and_capture_moves_bitboard, pawn_forward_moves_bitboard, potential_pawn_jump_moves};
use rusty_rival::types::types::Piece::{Bishop, Rook};
use rusty_rival::types::types::{Bitboard, MoveList, Square};
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
    let algebraic = sort_moves(move_list);
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
fn it_determines_if_a_given_square_is_attacked_by_a_given_colour_in_a_given_position() {
    let position = get_position(&"n5k1/1P2P1n1/1n5p/P1pP4/5R2/1q3B2/4Nr1P/R3K2R w Q - 0 1".to_string());
    assert_eq!(any_squares_in_bitboard_attacked(&position, &Black, bit(2) | bit(3)), true);
    assert_eq!(is_bishop_attacking_square(4, 22, position.all_pieces_bitboard), true);
    assert_eq!(is_bishop_attacking_square(5, 22, position.all_pieces_bitboard), false);
    assert_eq!(is_square_attacked_by(&position, bitref_from_algebraic_squareref("d1".to_string()) as Square, &Black), true);
    assert_eq!(is_square_attacked_by(&position, 58, &White), true);
    assert_eq!(is_square_attacked_by(&position, 60, &White), true);

    let position = get_position(&"n5k1/1P2P1n1/1n2q2p/P1pP4/5R2/5B2/1r2N2P/R3K1r1 w Q - 0 1".to_string());

    assert_eq!(is_square_attacked_by(&position, 0, &White), true);
    assert_eq!(is_square_attacked_by(&position, 0, &Black), true);
    assert_eq!(is_square_attacked_by(&position, 1, &White), true);
    assert_eq!(is_square_attacked_by(&position, 1, &Black), false);
    assert_eq!(is_square_attacked_by(&position, 2, &White), true);
    assert_eq!(is_square_attacked_by(&position, 2, &Black), true);
    assert_eq!(is_square_attacked_by(&position, 3, &White), true);
    assert_eq!(is_square_attacked_by(&position, 3, &Black), true);
    assert_eq!(is_square_attacked_by(&position, 4, &White), true);
    assert_eq!(is_square_attacked_by(&position, 4, &Black), false);
    assert_eq!(is_square_attacked_by(&position, 5, &White), true);
    assert_eq!(is_square_attacked_by(&position, 5, &Black), false);
    assert_eq!(is_square_attacked_by(&position, 6, &White), true);
    assert_eq!(is_square_attacked_by(&position, 6, &Black), true);
    assert_eq!(is_square_attacked_by(&position, 7, &White), false);
    assert_eq!(is_square_attacked_by(&position, 7, &Black), false);
    assert_eq!(is_square_attacked_by(&position, 8, &White), false);
    assert_eq!(is_square_attacked_by(&position, 8, &Black), false);
    assert_eq!(is_square_attacked_by(&position, 9, &White), true);
    assert_eq!(is_square_attacked_by(&position, 9, &Black), true);
    assert_eq!(is_square_attacked_by(&position, 10, &White), true);
    assert_eq!(is_square_attacked_by(&position, 10, &Black), false);
    assert_eq!(is_square_attacked_by(&position, 11, &White), true);
    assert_eq!(is_square_attacked_by(&position, 11, &Black), true);
    assert_eq!(is_square_attacked_by(&position, 12, &White), true);
    assert_eq!(is_square_attacked_by(&position, 12, &Black), true);
    assert_eq!(is_square_attacked_by(&position, 13, &White), false);
    assert_eq!(is_square_attacked_by(&position, 13, &Black), true);
    assert_eq!(is_square_attacked_by(&position, 14, &White), false);
    assert_eq!(is_square_attacked_by(&position, 14, &Black), false);
    assert_eq!(is_square_attacked_by(&position, 15, &White), true);
    assert_eq!(is_square_attacked_by(&position, 15, &Black), true);
    assert_eq!(is_square_attacked_by(&position, 16, &White), false);
    assert_eq!(is_square_attacked_by(&position, 16, &Black), true);
    assert_eq!(is_square_attacked_by(&position, 17, &White), true);
    assert_eq!(is_square_attacked_by(&position, 17, &Black), true);
    assert_eq!(is_square_attacked_by(&position, 18, &White), true);
    assert_eq!(is_square_attacked_by(&position, 18, &Black), false);
    assert_eq!(is_square_attacked_by(&position, 19, &White), false);
    assert_eq!(is_square_attacked_by(&position, 19, &Black), true);
    assert_eq!(is_square_attacked_by(&position, 40, &White), false);
    assert_eq!(is_square_attacked_by(&position, 40, &Black), true);
    assert_eq!(is_square_attacked_by(&position, 41, &White), false);
    assert_eq!(is_square_attacked_by(&position, 41, &Black), true);
    assert_eq!(is_square_attacked_by(&position, 42, &White), true);
    assert_eq!(is_square_attacked_by(&position, 42, &Black), true);
    assert_eq!(is_square_attacked_by(&position, 43, &White), true);
    assert_eq!(is_square_attacked_by(&position, 43, &Black), true);
    assert_eq!(is_square_attacked_by(&position, 44, &White), false);
    assert_eq!(is_square_attacked_by(&position, 44, &Black), true);
    assert_eq!(is_square_attacked_by(&position, 45, &White), true);
    assert_eq!(is_square_attacked_by(&position, 45, &Black), true);
    assert_eq!(is_square_attacked_by(&position, 61, &White), true);
    assert_eq!(is_square_attacked_by(&position, 61, &Black), true);
    assert_eq!(is_square_attacked_by(&position, 62, &White), false);
    assert_eq!(is_square_attacked_by(&position, 62, &Black), false);
    assert_eq!(is_square_attacked_by(&position, 63, &White), true);
    assert_eq!(is_square_attacked_by(&position, 63, &Black), true);
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

fn sort_moves(move_list: MoveList) -> Vec<String> {
    let mut algebraic: Vec<String> = move_list.iter().map(|m| { algebraic_move_from_move(*m) }).collect();
    algebraic.sort();
    return algebraic;
}

#[test]
fn it_generates_castle_moves_for_a_given_mover() {
    let position = get_position(&"n5k1/1P2P1n1/1n2q2p/P1pP4/5R2/5B2/1r2N2P/R3K1r1 w Q - 0 1".to_string());
    assert_eq!(sort_moves(generate_castle_moves(&position)).len(), 0);

    let position = get_position(&"n5k1/1P2P1n1/1n2q2p/P1pP4/5R2/5B2/1r2N2P/R3K2R w KQ - 0 1".to_string());
    assert_eq!(sort_moves(generate_castle_moves(&position)), vec!["e1c1", "e1g1"]);

    let position = get_position(&"n5k1/1P2P1n1/1n2q2p/P1pP4/5R2/5B2/3rN2P/R3K2R w KQ - 0 1".to_string());
    assert_eq!(sort_moves(generate_castle_moves(&position)), vec!["e1g1"]);

    let position = get_position(&"n5k1/1P2P1n1/1n2q2p/P1pP4/5R2/5B2/4Nr1P/R3K2R w Q - 0 1".to_string());
    assert_eq!(sort_moves(generate_castle_moves(&position)), vec!["e1c1"]);

    let position = get_position(&"n5k1/1P2P1n1/1n5p/P1pP4/5R2/1q3B2/4Nr1P/R3K2R w Q - 0 1".to_string());
    assert_eq!(sort_moves(generate_castle_moves(&position)).len(), 0);

    let position = get_position(&"r3k1R1/1P2P1n1/1n2q2p/P1pP4/5R2/5B2/1r2N2P/R3K1r1 b Q - 0 1".to_string());
    assert_eq!(sort_moves(generate_castle_moves(&position)).len(), 0);

    let position = get_position(&"r3k2r/1P2P1n1/1n2q2p/P1pP4/5R2/5B2/1r2N2P/R3K2R b Q - 0 1".to_string());
    assert_eq!(sort_moves(generate_castle_moves(&position)).len(), 0);

    let position = get_position(&"r3k2r/1P2PRn1/1n2q2p/P1pP4/8/5B2/1r2N2P/R3K2R b Q - 0 1".to_string());
    assert_eq!(sort_moves(generate_castle_moves(&position)).len(), 0);

    let position = get_position(&"r3k2r/1P3Rn1/1n2q2p/P1pP2P1/8/5B2/1r2N2P/R3K2R b qQ - 0 1".to_string());
    assert_eq!(sort_moves(generate_castle_moves(&position)), vec!["e8c8"]);
}

#[test]
pub fn it_gets_all_moves_for_a_position() {
    assert_eq!(sort_moves(moves(&get_position(&"4k3/8/6N1/4K3/8/8/8/8 b - - 0 1".to_string()))), vec!["e8d7", "e8d8", "e8e7", "e8f7", "e8f8"]);

    // let position = get_position(&"4k3/8/6N1/4K3/8/8/8/8 b - - 0 1".to_string());
    // let no_checks = moves(&position).iter().filter(|&mut m| !is_check(make_move(position,x), &position.mover)).collect();
    // assert_eq!(sort_moves(no_checks), vec!["e8d7","e8d8","e8f7"]);

    assert_eq!(sort_moves(moves(&get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/1R2K2R b Kkq - 0 1".to_string()))), vec!["a7a6","a8b8","a8c8","a8d8","b4b3","c4c3","d3b1","d3c2","d3e2","d3f1","e4e3","e8d7","e8d8","e8e7","e8f7","e8f8","e8g8","h7h5","h7h6","h8f8","h8g8"]);
    assert_eq!(sort_moves(moves(&get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P4K1P/R6R b kq - 0 1".to_string()))), vec!["a7a6","a8b8","a8c8","a8d8","b4b3","c4c3","d3b1","d3c2","d3e2","d3f1","e4e3","e8d7","e8d8","e8e7","e8f7","e8f8","e8g8","h7h5","h7h6","h8f8","h8g8"]);
    assert_eq!(sort_moves(moves(&get_position(&"5k2/7p/p3B1p1/P4pP1/3K1P1P/8/8/8 w - f6 0 1".to_string()))), vec!["d4c3","d4c4","d4c5","d4d3","d4d5","d4e3","d4e4","d4e5","e6a2","e6b3","e6c4","e6c8","e6d5","e6d7","e6f5","e6f7","e6g8","g5f6","h4h5"]);
    assert_eq!(sort_moves(moves(&get_position(&"6k1/5p1p/p3B1p1/P5P1/3K1P1P/8/8/8 w - - 0 1".to_string()))), vec!["d4c3","d4c4","d4c5","d4d3","d4d5","d4e3","d4e4","d4e5","e6a2","e6b3","e6c4","e6c8","e6d5","e6d7","e6f5","e6f7","e6g4","e6h3","f4f5","h4h5"]);

    assert_eq!(sort_moves(moves(&get_position(&"n5k1/1P2P1n1/1n2q2p/P1pP4/5R2/3K1B2/1r2N2P/6r1 w - c6 0 1".to_string()))),
               vec!["a5a6","a5b6","b7a8b","b7a8n","b7a8q","b7a8r","b7b8b","b7b8n","b7b8q","b7b8r",
                    "d3c2","d3c3","d3c4","d3d2","d3d4","d3e3","d3e4",
                    "d5c6","d5d6","d5e6",
                    "e2c1","e2c3","e2d4","e2g1","e2g3",
                    "e7e8b","e7e8n","e7e8q","e7e8r",
                    "f3e4","f3g2","f3g4","f3h1","f3h5",
                    "f4a4","f4b4","f4c4","f4d4","f4e4","f4f5","f4f6","f4f7","f4f8","f4g4","f4h4",
                    "h2h3","h2h4",
               ]);

    assert_eq!(sort_moves(moves(&get_position(&"n5k1/1P2P1n1/1n2q2p/P1pP4/3p1R2/2p2B2/1rPPN2P/R3K1r1 w Q - 0 1".to_string()))),
               vec![
                   "a1a2","a1a3","a1a4","a1b1","a1c1","a1d1"
                 , "a5a6","a5b6"
                 , "b7a8b","b7a8n","b7a8q","b7a8r","b7b8b","b7b8n","b7b8q","b7b8r"
                 , "d2c3","d2d3"
                 , "d5d6","d5e6"
                 , "e1d1","e1f1","e1f2"
                 , "e2c1","e2c3","e2d4","e2g1","e2g3"
                 , "e7e8b","e7e8n","e7e8q","e7e8r"
                 , "f3e4","f3g2","f3g4","f3h1","f3h5"
                 , "f4d4","f4e4","f4f5","f4f6","f4f7","f4f8","f4g4","f4h4"
                 , "h2h3","h2h4"
               ]);

}


//
//
//
// describe "isCheck" $
// it "Determines if the given side's king is attacked by at least one of the other side's pieces" $ do
// let position = get_position(&"n5k1/1P2P1n1/1n5p/P1pP4/5R2/1q3B2/4Nr1P/R3K2R w Q - 0 1".to_string())
// isCheck position White `shouldBe` False
// isCheck position Black `shouldBe` False
// let position = get_position(&"n4Rk1/1P2P1n1/1n5p/P1pP4/8/1q3B2/4Nr1P/R3K2R w Q - 0 1".to_string())
// isCheck position White `shouldBe` False
// isCheck position Black `shouldBe` True
// let position = get_position(&"n4Rk1/1P2P1n1/1n5p/P1pP4/8/2q2B2/4Nr1P/R3K2R w Q - 0 1".to_string())
// isCheck position White `shouldBe` True
// isCheck position Black `shouldBe` True
// let position = get_position(&"n5k1/1P3Pn1/1n5p/P1pP1R2/8/3q1B2/4Nr1P/R3K2R w Q - 0 1".to_string())
// isCheck position White `shouldBe` False
// isCheck position Black `shouldBe` True
// let position = get_position(&"n5k1/1P2P1n1/1n5p/P1pP1R2/8/3q1B2/4N2P/R3Kr1R w Q - 0 1".to_string())
// isCheck position White `shouldBe` True
// isCheck position Black `shouldBe` False
// isCheck (get_position(&"r2k3r/p6p/8/B7/1p2p3/2pb4/P4K1P/R6R w - - 0 1".to_string())) Black `shouldBe` True
//
// describe "captureMoves" $
// it "returns all and only capture moves" $ do
// let position = get_position(&"n5k1/1P2P1n1/1n5p/P1pP4/5R2/1q3B2/4Nr1P/R3K2R w Q - 0 1".to_string()) in captureMoves position `shouldBe` filter (isCapture position) (moves position)
// let position = get_position(&"n4Rk1/1P2P1n1/1n5p/P1pP4/8/1q3B2/4Nr1P/R3K2R w Q - 0 1".to_string()) in captureMoves position `shouldBe` filter (isCapture position) (moves position)
// let position = get_position(&"n5k1/1P3Pn1/1n5p/P1pP1R2/8/3q1B2/4Nr1P/R3K2R w Q - 0 1".to_string()) in captureMoves position `shouldBe` filter (isCapture position) (moves position)
// let position = get_position(&"n5k1/1P2P1n1/1n5p/P1pP1R2/8/3q1B2/4N2P/R3Kr1R w Q - 0 1".to_string()) in captureMoves position `shouldBe` filter (isCapture position) (moves position)
// let position = get_position(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string()) in captureMoves position `shouldBe` filter (isCapture position) (moves position)
// let position = get_position(&"8/2p5/3p4/KP5r/1R3pPk/8/4P3/8 b - g3 0 1".to_string()) in captureMoves position `shouldBe` filter (isCapture position) (moves position)
// -- let position = get_position(&"r2Bk2r/p6p/8/8/1pp1p3/3b4/P6P/R3K2R b KQkq - 0 1".to_string())
// -- captureMoves position `shouldBe` filter (isCapture position) $ moves position
// -- let position = get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P4K1P/R6R b kq - 0 1".to_string())
// -- captureMoves position `shouldBe` filter (isCapture position) $ moves position
// -- let position = get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P2K3P/R6R b kq - 0 1".to_string())
// -- captureMoves position `shouldBe` filter (isCapture position) $ moves position
// -- let position = get_position(&"r3k2r/p1B4p/8/8/1pp1p3/3b4/P6P/R3K2R b KQkq - 0 1".to_string())
// -- captureMoves position `shouldBe` filter (isCapture position) $ moves position
// -- let position = get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R3K1R1 b Qkq - 0 1".to_string())
// -- captureMoves position `shouldBe` filter (isCapture position) $ moves position
// -- let position = get_position(&"r3k2r/p6p/1B6/8/1pp1p3/3b4/P6P/R3K2R b KQkq - 0 1".to_string())
// -- captureMoves position `shouldBe` filter (isCapture position) $ moves position
// -- let position = get_position(&"r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -".to_string())
// -- captureMoves position `shouldBe` filter (isCapture position) $ moves position
// -- let position = get_position(&"8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -".to_string())
// -- captureMoves position `shouldBe` filter (isCapture position) $ moves position
// -- let position = get_position(&"8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -".to_string())
// -- captureMoves position `shouldBe` filter (isCapture position) $ moves position
// -- let position = get_position(&"r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -".to_string())
// -- captureMoves position `shouldBe` filter (isCapture position) $ moves position
// -- let position = get_position(&"8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -".to_string())
// -- captureMoves position `shouldBe` filter (isCapture position) $ moves position
//
// describe "makeAlgebraicMoves" $
// it "Makes a move from a position and returns a new position" $ do
// let expected = get_position(&"rnbqkbnr/ppppppp1/8/8/2PPPP1P/8/PP6/RNBQKqNR w KQkq - 0 7".to_string())
// let position = makeAlgebraicMoves (get_position(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1") ["h2h4","h7h6","g2g4","h6h5","f2f4","h5g4","e2e4","g4g3","d2d4","g3g2","c2c4","g2f1q".to_string())]
// position `shouldBe` expected
//
// describe "moveFromAlgebraicMove" $
// it "Makes a move from a position and returns a new position" $ do
// algebraicMoveFromMove (moveFromAlgebraicMove "a1h8") `shouldBe` "a1h8"
// algebraicMoveFromMove (moveFromAlgebraicMove "h1a8") `shouldBe` "h1a8"
// algebraicMoveFromMove (moveFromAlgebraicMove "h7g8b") `shouldBe` "h7g8b"
//
// describe "fromSquarePart" $
// it "Gets the Square for the from part of a compact move" $
// fromSquarePart (moveFromAlgebraicMove "h1a8") `shouldBe` bitRefFromAlgebraicSquareRef "h1"
//
// describe "toSquarePart" $
// it "Gets the Square for the to part of a compact move" $
// toSquarePart (moveFromAlgebraicMove "h1a8") `shouldBe` bitRefFromAlgebraicSquareRef "a8"
//
// describe "movePieceWithinBitboard" $
// it "Returns a bitboard with the one bit in 'from', if it exists, moved to 'to'" $ do
// movePieceWithinBitboard (bitRefFromAlgebraicSquareRef "a8") (bitRefFromAlgebraicSquareRef "b8") 0b1000100000000000000000000000000000001000010000000000000000000000
// `shouldBe` 0b0100100000000000000000000000000000001000010000000000000000000000
// movePieceWithinBitboard (bitRefFromAlgebraicSquareRef "h1") (bitRefFromAlgebraicSquareRef "b8") 0b1000100000000000000000000000000000001000010000000000000000000001
// `shouldBe` 0b1100100000000000000000000000000000001000010000000000000000000000
// movePieceWithinBitboard (bitRefFromAlgebraicSquareRef "a1") (bitRefFromAlgebraicSquareRef "b8") 0b1000100000000000000000000000000000001000010000000000000010000001
// `shouldBe` 0b1100100000000000000000000000000000001000010000000000000000000001
// movePieceWithinBitboard (bitRefFromAlgebraicSquareRef "a8") (bitRefFromAlgebraicSquareRef "b8") 0b1000100000000000000000000000000000001000010000000000000000000000
// `shouldBe` 0b0100100000000000000000000000000000001000010000000000000000000000
// movePieceWithinBitboard (bitRefFromAlgebraicSquareRef "h1") (bitRefFromAlgebraicSquareRef "a8") 0b0000100000000000000000000000000000001000010000000000000000000001
// `shouldBe` 0b1000100000000000000000000000000000001000010000000000000000000000
// movePieceWithinBitboard (bitRefFromAlgebraicSquareRef "a1") (bitRefFromAlgebraicSquareRef "a8") 0b0000100000000000000000000000000000001000010000000000000010000001
// `shouldBe` 0b1000100000000000000000000000000000001000010000000000000000000001
// movePieceWithinBitboard (bitRefFromAlgebraicSquareRef "b8") (bitRefFromAlgebraicSquareRef "c8") 0b0000100000000000000000000000000000001000010000000000000010000001
// `shouldBe` 0b0000100000000000000000000000000000001000010000000000000010000001
//
//
// describe "enPassantCapturedPieceSquare" $
// it "Returns the square the captured pawn was on before it was captured en passant" $ do
// enPassantCapturedPieceSquare (bitRefFromAlgebraicSquareRef "a3") `shouldBe` bitRefFromAlgebraicSquareRef "a4"
// enPassantCapturedPieceSquare (bitRefFromAlgebraicSquareRef "a6") `shouldBe` bitRefFromAlgebraicSquareRef "a5"
//
// describe "removePawnIfPromotion" $
// it "Removes the bit from the pawn bitboard if it has just moved to a promotion rank" $
// removePawnIfPromotion 0b1000000000000000000000000000000000000000000000000000000000000000 `shouldBe` 0b0000000000000000000000000000000000000000000000000000000000000000
//
// describe "isPromotionSquare" $
// it "Returns True if the given square is on the first or eigth ranks" $ do
// testBit promotionSquares (bitRefFromAlgebraicSquareRef "a8") `shouldBe` True
// testBit promotionSquares (bitRefFromAlgebraicSquareRef "b1") `shouldBe` True
// testBit promotionSquares (bitRefFromAlgebraicSquareRef "a3") `shouldBe` False
//
// describe "promotionPieceFromMove" $
// it "Returns the promotion piece from the move" $ do
// promotionPieceFromMove (moveFromAlgebraicMove "g7h8r") `shouldBe` Rook
// promotionPieceFromMove (moveFromAlgebraicMove "g7h8q") `shouldBe` Queen
// promotionPieceFromMove (moveFromAlgebraicMove "g7h8n") `shouldBe` Knight
// promotionPieceFromMove (moveFromAlgebraicMove "g7h8b") `shouldBe` Bishop
// promotionPieceFromMove (moveFromAlgebraicMove "g6h7") `shouldBe` Pawn
//
// describe "createIfPromotion" $
// it "Adds the promotion piece location to the bitboard" $ do
// createIfPromotion True 0b0000000010000000000000000000000000000000000000000000000000000000 0b0000000000000000000000000000000000000000000000000000000000000000 a7Bit a8Bit
// `shouldBe` 0b1000000000000000000000000000000000000000000000000000000000000000
// createIfPromotion True 0b0000000010000000000000000000000000000000000000000000000000000000 0b0000000000000000000000000000000000000000000000000000000000000000 a7Bit a6Bit
// `shouldBe` 0b0000000000000000000000000000000000000000000000000000000000000000
// createIfPromotion False 0b0000000010000000000000000000000000000000000000000000000000000000 0b0000000000000000000000000000000000000000000000000000000000000000 a7Bit a8Bit
// `shouldBe` 0b0000000000000000000000000000000000000000000000000000000000000000
//
//
//
// describe "onlyKingsRemain" $
// it "Checks if only kings remain" $ do
// onlyKingsRemain (get_position(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string())) `shouldBe` False
// onlyKingsRemain (get_position(&"7k/8/8/8/8/8/8/K7 w - - 0 1".to_string())) `shouldBe` True
// onlyKingsRemain (get_position(&"K7/8/8/8/8/8/8/7k w - - 0 1".to_string())) `shouldBe` True
// onlyKingsRemain (get_position(&"K7/8/8/3p4/8/8/8/7k w - - 0 1".to_string())) `shouldBe` False
//
// describe "whitePieceValues" $
// it "Returns the value of non-pawn white pieces" $ do
// whitePieceValues (get_position(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string())) `shouldBe`
// pieceValue Rook * 2 + pieceValue Bishop * 2 + pieceValue Queen + pieceValue Knight * 2
// whitePieceValues (get_position(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RN-QK2R w KQkq - 0 1".to_string())) `shouldBe`
// pieceValue Rook * 2 + pieceValue Queen + pieceValue Knight
//
// describe "blackPieceValues" $
// it "Returns the value of non-pawn black pieces" $ do
// blackPieceValues (get_position(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string())) `shouldBe`
// pieceValue Rook * 2 + pieceValue Bishop * 2 + pieceValue Queen + pieceValue Knight * 2
// blackPieceValues (get_position(&"-nbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string())) `shouldBe`
// pieceValue Rook + pieceValue Bishop * 2 + pieceValue Queen + pieceValue Knight * 2
//
// describe "friendlyPieceValues" $
// it "Returns the value of non-pawn pieces for the player to move" $ do
// friendlyPieceValues (get_position(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string())) `shouldBe`
// pieceValue Rook * 2 + pieceValue Bishop * 2 + pieceValue Queen + pieceValue Knight * 2
// friendlyPieceValues (get_position(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RN-QK2R w KQkq - 0 1".to_string())) `shouldBe`
// pieceValue Rook * 2 + pieceValue Queen + pieceValue Knight
// friendlyPieceValues (get_position(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1".to_string())) `shouldBe`
// pieceValue Rook * 2 + pieceValue Bishop * 2 + pieceValue Queen + pieceValue Knight * 2
// friendlyPieceValues (get_position(&"-nbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1".to_string())) `shouldBe`
// pieceValue Rook + pieceValue Bishop * 2 + pieceValue Queen + pieceValue Knight * 2
//
// describe "makeMove" $
// it "Makes a move from a position and returns a new position" $ do
// makeMove (get_position(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string()))
// (moveFromAlgebraicMove "e2e3")
// `shouldBe` get_position(&"rnbqkbnr/pppppppp/8/8/8/4P3/PPPP1PPP/RNBQKBNR b KQkq - 0 1".to_string())
// makeMove (get_position(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string()))
// (moveFromAlgebraicMove "e2e7")
// `shouldBe` get_position(&"rnbqkbnr/ppppPppp/8/8/8/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1".to_string())
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
// describe "isCapture" $
// it "returns if a move is a capture" $ do
// let position = get_position(&"rnbqkbnr/p1p2ppp/8/1pPpp3/4PP2/8/PP1P2PP/RNBQKBNR w KQkq d6 0 1".to_string())
// isCapture position (moveFromAlgebraicMove "f4e5") `shouldBe` True
// isCapture position (moveFromAlgebraicMove "c5d6") `shouldBe` True
// isCapture position (moveFromAlgebraicMove "c5c6") `shouldBe` False
//
// describe "quiescePositions" $
// it "returns a list of positions where the move that created them was a capture" $ do
// let position = get_position(&"rnbqkbnr/p1p2ppp/8/1pPpp3/4PP2/8/PP1P2PP/RNBQKBNR w KQkq d6 0 1".to_string())
// let qp = quiescePositions position False
// length qp `shouldBe` 4
// map (\pm -> (fst pm, algebraicMoveFromMove $ snd pm)) qp `shouldBe` [
// (get_position(&"rnbqkbnr/p1p2ppp/3P4/1p2p3/4PP2/8/PP1P2PP/RNBQKBNR b KQkq - 0 1","c5d6".to_string())),
// (get_position(&"rnbqkbnr/p1p2ppp/8/1pPPp3/5P2/8/PP1P2PP/RNBQKBNR b KQkq - 0 1","e4d5".to_string())),
// (get_position(&"rnbqkbnr/p1p2ppp/8/1pPpP3/4P3/8/PP1P2PP/RNBQKBNR b KQkq - 0 1","f4e5".to_string())),
// (get_position(&"rnbqkbnr/p1p2ppp/8/1BPpp3/4PP2/8/PP1P2PP/RNBQK1NR b KQkq - 0 1","f1b5".to_string()))
// ]
// quiescePositions (get_position(&"rnbqkbn1/ppp4r/5p2/3P4/2P4P/8/PP1P1PP1/RNBQK2R w KQq - 0 1".to_string())) False `shouldBe` []
//
// describe "hashtable" $
// it "stores and retrieves hashtable values" $ do
// c <- makeSearchState startStats [] 0
// updateHashTable 1 HashEntry { score=1, hePath=[2], height=3, bound=Exact, lock=0 } c
// updateHashTable 2 HashEntry { score=10, hePath=[20], height=30, bound=Upper, lock=0 } c
// hentry <- H.lookup (hashTable c) 2
// hePath (fromJust hentry) `shouldBe` [20]
// hentry <- H.lookup (hashTable c) 1
// hePath (fromJust hentry) `shouldBe` [2]
//
// describe "quiesce" $
// it "evaluates a position using a quiescence search" $ do
// c <- makeSearchState startStats [] 0
// let position = get_position(&"rnbqkbnr/ppp3pp/5p2/3PB1N1/2P4P/8/PP1P1PP1/RNBQK2R b KQkq - 0 1".to_string())
// q <- quiesce position -100000 100000 0 c
// msScore q `shouldBe` 150
// let position = get_position(&"rnbqkbnr/ppp3pp/5p2/3PB1N1/2P4P/8/PP1P1PP1/RNBQK2R w KQkq - 0 1".to_string())
// q <- quiesce position -100000 100000 0 c
// msScore q `shouldBe` 200
//
//
//
// describe "Perft Test" $
// it "Returns the total number of moves in a full move tree of a given depth with a given position as its head" $ do
// perft (get_position(&"8/p7/8/1P6/K1k3pP/6P1/8/8 b - h3 0 1".to_string())) 0 `shouldBe` 8
// perft (get_position(&"5k2/5p1p/p3B1p1/P5P1/3K1P1P/8/8/8 b - -".to_string())) 0 `shouldBe` 9
// perft (get_position(&"8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -".to_string())) 0 `shouldBe` 14
// perft (get_position(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string())) 0 `shouldBe` 20
// perft (get_position(&"rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1".to_string())) 0 `shouldBe` 20
// perft (get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R3K2R w KQkq -".to_string())) 0 `shouldBe` 17
// perft (get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R3KR2 b Qkq - 0 1".to_string())) 0 `shouldBe` 17
// perft (get_position(&"r3k2r/p6p/8/8/1Bp1p3/3b4/P6P/R3K2R b KQkq - 0 1".to_string())) 0 `shouldBe` 19
// perft (get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R3K1R1 b Qkq - 0 1".to_string())) 0 `shouldBe` 19
// perft (get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/1R2K2R b Kkq - 0 1".to_string())) 0 `shouldBe` 20
// perft (get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/2R1K2R b Kkq - 0 1".to_string())) 0 `shouldBe` 20
// perft (get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/3RK2R b Kkq - 0 1".to_string())) 0 `shouldBe` 20
// perft (get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P4K1P/R6R b kq - 0 1".to_string())) 0 `shouldBe` 20
// perft (get_position(&"r3k2r/p6p/8/B7/1pp1p2P/3b4/P7/R3K2R b KQkq - 0 1".to_string())) 0 `shouldBe` 20
// perft (get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b3P/P7/R3K2R b KQkq - 0 1".to_string())) 0 `shouldBe` 20
// perft (get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P2K3P/R6R b kq - 0 1".to_string())) 0 `shouldBe` 20
// perft (get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R2K3R b kq - 0 1".to_string())) 0 `shouldBe` 20
// perft (get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/2KR3R b kq - 0 1".to_string())) 0 `shouldBe` 20
// perft (get_position(&"r3k2r/p6p/8/B7/Ppp1p3/3b4/7P/R3K2R b KQkq a3 0 1".to_string())) 0 `shouldBe` 21
// perft (get_position(&"r3k2r/p6p/8/B7/1pp1p3/P2b4/7P/R3K2R b KQkq - 0 1".to_string())) 0 `shouldBe` 21
// perft (get_position(&"r3k2r/p1B4p/8/8/1pp1p3/3b4/P6P/R3K2R b KQkq - 0 1".to_string())) 0 `shouldBe` 21
// perft (get_position(&"r2Bk2r/p6p/8/8/1pp1p3/3b4/P6P/R3K2R b KQkq - 0 1".to_string())) 0 `shouldBe` 21
// perft (get_position(&"r3k2r/p6p/1B6/8/1pp1p3/3b4/P6P/R3K2R b KQkq - 0 1".to_string())) 0 `shouldBe` 22
// perft (get_position(&"8/p7/8/1P6/K1k3pP/6P1/8/8 b - h3 0 1".to_string())) 1 `shouldBe` 41
// perft (get_position(&"8/8/8/KP6/5pPk/8/4P3/8 b - g3 0 1".to_string())) 1 `shouldBe` 46
// perft (get_position(&"r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -".to_string())) 0 `shouldBe` 48
// perft (get_position(&"8/2p5/8/KP6/5pPk/8/4P3/8 b - g3 0 1".to_string())) 1 `shouldBe` 57
// perft (get_position(&"8/2p5/3p4/KP6/5pPk/8/4P3/8 b - g3 0 1".to_string())) 1 `shouldBe` 64
// perft (get_position(&"4k3/8/8/8/8/8/PPPPPPPP/RNBQKBNR w KQ - 0 1".to_string())) 1 `shouldBe` 100
// perft (get_position(&"5k2/5p1p/p3B1p1/P5P1/3K1P1P/8/8/8 b - -".to_string())) 1 `shouldBe` 169
// perft (get_position(&"8/2p5/3p4/KP5r/1R2Pp1k/8/6P1/8 b - e3 0 1".to_string())) 1 `shouldBe` 177
// perft (get_position(&"8/8/3p4/KPp4r/1R3p1k/4P3/6P1/8 w - c6 0 1".to_string())) 1 `shouldBe` 190
// perft (get_position(&"8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -".to_string())) 1 `shouldBe` 191
// perft (get_position(&"8/2p5/3p4/KP5r/R4p1k/8/4P1P1/8 b - - 0 1".to_string())) 1 `shouldBe` 202
// perft (get_position(&"8/2p5/3p4/1P5r/KR3p1k/8/4P1P1/8 b - - 0 1".to_string())) 1 `shouldBe` 224
// perft (get_position(&"8/2p5/3p4/KP5r/1R3pPk/8/4P3/8 b - g3 0 1".to_string())) 1 `shouldBe` 226
// perft (get_position(&"8/2p5/K2p4/1P5r/1R3p1k/8/4P1P1/8 b - - 0 1".to_string())) 1 `shouldBe` 240
// perft (get_position(&"8/3K4/2p5/p2b2r1/5k2/8/8/1q6 b - - 1 67".to_string())) 1 `shouldBe` 279
// perft (get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R2K3R b kq - 0 1".to_string())) 1 `shouldBe` 300
// perft (get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/1R2K2R b Kkq - 0 1".to_string())) 1 `shouldBe` 377
// perft (get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/3RK2R b Kkq - 0 1".to_string())) 1 `shouldBe` 365
// perft (get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b3P/P7/R3K2R b KQkq - 0 1".to_string())) 1 `shouldBe` 339
// perft (get_position(&"r3k2r/p6p/8/B7/Ppp1p3/3b4/7P/R3K2R b KQkq a3 0 1".to_string())) 1 `shouldBe` 357
// perft (get_position(&"r3k2r/p6p/8/B7/1pp1p2P/3b4/P7/R3K2R b KQkq - 0 1".to_string())) 1 `shouldBe` 358
// perft (get_position(&"r3k2r/p6p/8/B7/1pp1p3/P2b4/7P/R3K2R b KQkq - 0 1".to_string())) 1 `shouldBe` 376
// perft (get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/2R1K2R b Kkq - 0 1".to_string())) 1 `shouldBe` 380
// perft (get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/2KR3R b kq - 0 1".to_string())) 1 `shouldBe` 385
// perft (get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R3KR2 b Qkq - 0 1".to_string())) 1 `shouldBe` 395
// perft (get_position(&"r3k2r/p6p/8/8/1Bp1p3/3b4/P6P/R3K2R b KQkq - 0 1".to_string())) 1 `shouldBe` 395
// perft (get_position(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string())) 1 `shouldBe` 400
// perft (get_position(&"r2Bk2r/p6p/8/8/1pp1p3/3b4/P6P/R3K2R b KQkq - 0 1".to_string())) 1 `shouldBe` 403
// perft (get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P4K1P/R6R b kq - 0 1".to_string())) 1 `shouldBe` 437
// perft (get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P2K3P/R6R b kq - 0 1".to_string())) 1 `shouldBe` 437
// perft (get_position(&"r3k2r/p1B4p/8/8/1pp1p3/3b4/P6P/R3K2R b KQkq - 0 1".to_string())) 1 `shouldBe` 438
// perft (get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R3K1R1 b Qkq - 0 1".to_string())) 1 `shouldBe` 454
// perft (get_position(&"r3k2r/p6p/1B6/8/1pp1p3/3b4/P6P/R3K2R b KQkq - 0 1".to_string())) 1 `shouldBe` 470
// perft (get_position(&"r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -".to_string())) 1 `shouldBe` 2039
// perft (get_position(&"8/2p5/3p4/KP5r/1R3pPk/8/4P3/8 b - g3 0 1".to_string())) 2 `shouldBe` 3702
// perft (get_position(&"8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -".to_string())) 2 `shouldBe` 2812
// perft (get_position(&"8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -".to_string())) 3 `shouldBe` 43238
// perft (get_position(&"r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -".to_string())) 2 `shouldBe` 97862
// perft (get_position(&"8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -".to_string())) 4 `shouldBe` 674624
//
// perft (get_position(&"r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -".to_string())) 3 `shouldBe` 4085603
// perft (get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/2KR3R b kq - 0 1".to_string())) 4 `shouldBe` 4238116
// perft (get_position(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string())) 4 `shouldBe` 4865609
// perft (get_position(&"8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -".to_string())) 5 `shouldBe` 11030083
// perft (get_position(&"rnbqkb1r/ppppp1pp/7n/4Pp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3".to_string())) 4 `shouldBe` 11139762
// perft (get_position(&"8/7p/p5pb/4k3/P1pPn3/8/P5PP/1rB2RK1 b - d3 0 28".to_string())) 5 `shouldBe` 38633283
// perft (get_position(&"8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -".to_string())) 6 `shouldBe` 178633661
//
// --perft (get_position(&"8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -".to_string())) 7 `shouldBe` 3009794393
// --perft (get_position(&"r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -".to_string())) 4 `shouldBe` 193690690
// --perft (get_position(&"r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -".to_string())) 5 `shouldBe` 8031647685
// --perft (get_position(&"8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -".to_string())) 6 `shouldBe` 178633661
//
// perft (get_position(&"5k2/5p1p/p3B1p1/P5P1/3K1P1P/8/8/8 b - -".to_string())) 3 `shouldBe` 20541
// perft (get_position(&"n1n5/PPPk4/8/8/8/8/4Kppp/5N1N b - - 0 1".to_string())) 3 `shouldBe` 182838
//
// perft (get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/1R2K2R b Kkq - 0 1".to_string())) 3 `shouldBe` 175927
// perft (get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/2R1K2R b Kkq - 0 1".to_string())) 3 `shouldBe` 178248
// perft (get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/3RK2R b Kkq - 0 1".to_string())) 3 `shouldBe` 168357
// perft (get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P4K1P/R6R b kq - 0 1".to_string())) 3 `shouldBe` 221267
// perft (get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P2K3P/R6R b kq - 0 1".to_string())) 3 `shouldBe` 213344
// perft (get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R2K3R b kq - 0 1".to_string())) 3 `shouldBe` 120873
// perft (get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/2KR3R b kq - 0 1".to_string())) 3 `shouldBe` 184127
// perft (get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R3K1R1 b Qkq - 0 1".to_string())) 3 `shouldBe` 240619
// perft (get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R3KR2 b Qkq - 0 1".to_string())) 3 `shouldBe` 189825
// perft (get_position(&"r3k2r/p6p/8/B7/Ppp1p3/3b4/7P/R3K2R b KQkq a3 0 1".to_string())) 3 `shouldBe` 154828
// perft (get_position(&"r3k2r/p6p/8/B7/1pp1p3/P2b4/7P/R3K2R b KQkq - 0 1".to_string())) 3 `shouldBe` 173400
// perft (get_position(&"r3k2r/p6p/8/B7/1pp1p2P/3b4/P7/R3K2R b KQkq - 0 1".to_string())) 3 `shouldBe` 165129
// perft (get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b3P/P7/R3K2R b KQkq - 0 1".to_string())) 3 `shouldBe` 151137
// perft (get_position(&"r3k2r/p6p/1B6/8/1pp1p3/3b4/P6P/R3K2R b KQkq - 0 1".to_string())) 3 `shouldBe` 249845
// perft (get_position(&"r3k2r/p1B4p/8/8/1pp1p3/3b4/P6P/R3K2R b KQkq - 0 1".to_string())) 3 `shouldBe` 227059
// perft (get_position(&"r2Bk2r/p6p/8/8/1pp1p3/3b4/P6P/R3K2R b KQkq - 0 1".to_string())) 3 `shouldBe` 185525
// perft (get_position(&"r3k2r/p6p/8/8/1Bp1p3/3b4/P6P/R3K2R b KQkq - 0 1".to_string())) 3 `shouldBe` 186968
//
// perft (get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R3K2R w KQkq -".to_string())) 1 `shouldBe` 341
// perft (get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R3K2R w KQkq -".to_string())) 2 `shouldBe` 6666
// perft (get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R3K2R w KQkq -".to_string())) 3 `shouldBe` 150072
// perft (get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R3K2R w KQkq -".to_string())) 4 `shouldBe` 3186478
// perft (get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R3K2R w KQkq -".to_string())) 5 `shouldBe` 77054993
//
// perft (get_position(&"8/p7/8/1P6/K1k3pP/6P1/8/8 b - h3 0 1".to_string())) 2 `shouldBe` 325
// perft (get_position(&"8/p7/8/1P6/K1k3pP/6P1/8/8 b - h3 0 1".to_string())) 3 `shouldBe` 2002
// perft (get_position(&"8/p7/8/1P6/K1k3pP/6P1/8/8 b - h3 0 1".to_string())) 4 `shouldBe` 16763
// perft (get_position(&"8/p7/8/1P6/K1k3pP/6P1/8/8 b - h3 0 1".to_string())) 5 `shouldBe` 118853
// perft (get_position(&"8/p7/8/1P6/K1k3pP/6P1/8/8 b - h3 0 1".to_string())) 6 `shouldBe` 986637
//
// perft (get_position(&"8/p7/8/1P6/K1k3p1/6P1/7P/8 w - -".to_string())) 0 `shouldBe` 5
// perft (get_position(&"8/p7/8/1P6/K1k3p1/6P1/7P/8 w - -".to_string())) 1 `shouldBe` 39
// perft (get_position(&"8/p7/8/1P6/K1k3p1/6P1/7P/8 w - -".to_string())) 2 `shouldBe` 237
// perft (get_position(&"8/p7/8/1P6/K1k3p1/6P1/7P/8 w - -".to_string())) 3 `shouldBe` 2002
// perft (get_position(&"8/p7/8/1P6/K1k3p1/6P1/7P/8 w - -".to_string())) 4 `shouldBe` 14062
// perft (get_position(&"8/p7/8/1P6/K1k3p1/6P1/7P/8 w - -".to_string())) 5 `shouldBe` 120995
// perft (get_position(&"8/p7/8/1P6/K1k3p1/6P1/7P/8 w - -".to_string())) 6 `shouldBe` 966152
// perft (get_position(&"8/p7/8/1P6/K1k3p1/6P1/7P/8 w - -".to_string())) 7 `shouldBe` 8103790
//
// describe "Miscellaneous" $
// it "Runs various tests that have been used during the debugging process" $ do
// let position = get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R3K2R b kq - 0 1".to_string())
//
// bitRefFromAlgebraicSquareRef "f8" `shouldBe` 58
// isSquareAttackedBy position (bitRefFromAlgebraicSquareRef "f8") White `shouldBe` False
// anySquaresInBitboardAttacked position White noCheckCastleSquaresBlackKing `shouldBe` False
//
// sort (map algebraicMoveFromMove (generateCastleMoves position)) `shouldBe` ["e8g8"]
// blackKingCastleAvailable position `shouldBe` True
//
// sort (map algebraicMoveFromMove (moves position))
// `shouldBe` ["a7a6","a8b8","a8c8","a8d8","b4b3","c4c3","d3b1","d3c2","d3e2","d3f1","e4e3","e8d7","e8d8","e8e7","e8f7","e8f8","e8g8","h7h5","h7h6","h8f8","h8g8"]
// let newPositions = map (makeMove position) (moves position)
// length newPositions `shouldBe` 21
// perft position 0 `shouldBe` 20
//
// describe "bestMoveFirst" $
// it "returns a list of moves and their resulting positions, with the specified move at the head of the list" $ do
// let position = get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R3K2R b kq - 0 1".to_string())
// let moves = bestMoveFirst position (moveFromAlgebraicMove "e8f8")
// snd (head moves) `shouldBe` moveFromAlgebraicMove "e8f8"
//
// describe "zobrist" $
// it "calculates the zobrist hash of a position" $ do
// let p1 = get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R3K2R b kq - 0 1".to_string())
// let h1 = zobrist p1
// let h2 = zobrist (get_position(&"4k2r/p6p/8/B7/1pp1p3/3b4/P6P/R3K2R b kq - 0 1".to_string()))
// h2 `shouldNotBe` h1
// h2 `shouldBe` xor h1 (blackRookZobristSquares V.! 63)
// let h3 = zobrist (get_position(&"4k3/p6p/8/B7/1pp1p3/3b4/P6P/R3K2R b kq - 0 1".to_string()))
// h3 `shouldNotBe` h2
// h3 `shouldBe` xor h2 (blackRookZobristSquares V.! 56)
// let h4 = zobrist (get_position(&"4k3/7p/8/B7/1pp1p3/3b4/P6P/R3K2R b kq - 0 1".to_string()))
// h4 `shouldNotBe` h3
// h4 `shouldBe` xor h3 (blackPawnZobristSquares V.! 55)
//
