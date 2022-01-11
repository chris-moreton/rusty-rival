use rusty_rival::bitboards::{bit, EMPTY_CASTLE_SQUARES_WHITE_QUEEN, empty_squares_bitboard, enemy_bitboard, WHITE_PAWN_MOVES_CAPTURE, WHITE_PAWN_MOVES_FORWARD};
use rusty_rival::fen::{algebraic_move_from_move, bitref_from_algebraic_squareref, get_position};
use rusty_rival::make_move::{default_position_history, make_move, switch_side};
use rusty_rival::move_constants::EN_PASSANT_NOT_AVAILABLE;
use rusty_rival::moves::{all_bits_except_friendly_pieces, allocate_magic_boxes, any_squares_in_bitboard_attacked, generate_castle_moves, generate_king_moves, generate_knight_moves, generate_pawn_moves, generate_pawn_moves_from_to_squares, generate_slider_moves, is_bishop_attacking_square, is_check, is_square_attacked_by, moves, pawn_captures, pawn_forward_and_capture_moves_bitboard, pawn_forward_moves_bitboard, potential_pawn_jump_moves};
use rusty_rival::types::Piece::{Bishop, Rook};
use rusty_rival::types::{BLACK, MoveList, Square, WHITE};

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
fn it_generates_knight_moves_from_a_given_fen_ignoring_checks() {
    let position = get_position(&"n5k1/6n1/1n2q2p/1p5P/1P3RP1/2PK1B2/1r2N3/8 b kQKq g3 5 56".to_string());
    let mut move_list = Vec::new(); generate_knight_moves(&position, &mut move_list);
    let mut algebraic: Vec<String> = move_list.iter().map(|m| { algebraic_move_from_move(*m) }).collect();
    algebraic.sort();
    assert_eq!(algebraic, vec!["a8c7","b6a4","b6c4","b6c8","b6d5","b6d7","g7e8","g7f5","g7h5"]);

    let position = get_position(&"n5k1/6n1/1n2q2p/1p5P/1P3RP1/2PK1B2/1r2N3/8 w kQKq g3 5 56".to_string());
    let mut move_list = Vec::new(); generate_knight_moves(&position, &mut move_list);
    let mut algebraic: Vec<String> = move_list.iter().map(|m| { algebraic_move_from_move(*m) }).collect();
    algebraic.sort();
    assert_eq!(algebraic, vec!["e2c1","e2d4","e2g1","e2g3"]);
}

#[test]
fn it_generates_king_moves_from_a_given_fen_ignoring_checks() {
    let position = get_position(&"n5k1/6n1/1n2q2p/1p5P/1P3RP1/2PK1B2/1r2N3/8 b kQKq g3 5 56".to_string());
    let mut move_list = Vec::new(); generate_king_moves(&position, &mut move_list);
    let mut algebraic: Vec<String> = move_list.iter().map(|m| { algebraic_move_from_move(*m) }).collect();
    algebraic.sort();
    assert_eq!(algebraic, vec!["g8f7","g8f8","g8h7","g8h8"]);

    let position = get_position(&"n5k1/6n1/1n2q2p/1p5P/1P3RP1/2PK1B2/1r2N3/8 w kQKq g3 5 56".to_string());
    let mut move_list = Vec::new(); generate_king_moves(&position, &mut move_list);
    let mut algebraic: Vec<String> = move_list.iter().map(|m| { algebraic_move_from_move(*m) }).collect();
    algebraic.sort();
    assert_eq!(algebraic, vec!["d3c2","d3c4","d3d2","d3d4","d3e3","d3e4"]);
}

#[test]
fn it_generates_bishop_moves_including_diagonal_queen_moves_from_a_given_fen_ignoring_checks() {
    let magic_box = &allocate_magic_boxes();

    let position = get_position(&"n5k1/6n1/1n2q2p/1p5P/1P3RP1/2PK1B2/1r2N3/7R w kQKq g3 5 56".to_string());
    let mut move_list = Vec::new(); generate_slider_moves(&position, Bishop, &mut move_list, magic_box);
    let mut algebraic: Vec<String> = move_list.iter().map(|m| { algebraic_move_from_move(*m) }).collect();
    algebraic.sort();
    assert_eq!(algebraic, vec!["f3a8","f3b7","f3c6","f3d5","f3e4","f3g2"]);

    let position = get_position(&"n5k1/6n1/1n2q2p/1p5P/1P3RP1/2PK1B2/1r2N3/8 b kQKq g3 5 56".to_string());
    let mut move_list = Vec::new(); generate_slider_moves(&position, Bishop, &mut move_list, magic_box);
    let mut algebraic: Vec<String> = move_list.iter().map(|m| { algebraic_move_from_move(*m) }).collect();
    algebraic.sort();
    assert_eq!(algebraic, vec!["e6a2","e6b3","e6c4","e6c8","e6d5","e6d7","e6f5","e6f7","e6g4"]);
}

#[test]
fn it_generates_rook_moves_including_horizontal_queen_moves_from_a_given_fen_ignoring_checks() {
    let magic_box = &allocate_magic_boxes();

    let position = get_position(&"n5k1/6n1/1n2q2p/1p5P/1P3RP1/2PK1B2/1r2N3/8 w kQKq g3 5 56".to_string());
    let mut move_list = Vec::new(); generate_slider_moves(&position, Rook, &mut move_list, magic_box);
    let algebraic = sort_moves(move_list);
    assert_eq!(algebraic, vec!["f4c4","f4d4","f4e4","f4f5","f4f6","f4f7","f4f8"]);

    let position = get_position(&"n5k1/6n1/1n2q2p/1p5P/1P3RP1/2PK1B2/1r2N3/6r1 b kQKq g3 5 56".to_string());
    let mut move_list = Vec::new(); generate_slider_moves(&position, Rook, &mut move_list, magic_box);
    let mut algebraic: Vec<String> = move_list.iter().map(|m| { algebraic_move_from_move(*m) }).collect();
    algebraic.sort();
    assert_eq!(algebraic, vec!["b2a2","b2b1","b2b3","b2b4","b2c2","b2d2","b2e2","e6c6","e6d6","e6e2","e6e3","e6e4","e6e5","e6e7","e6e8","e6f6","e6g6","g1a1","g1b1","g1c1","g1d1","g1e1","g1f1","g1g2","g1g3","g1g4","g1h1"]);
}

#[test]
fn it_creates_a_list_of_moves_from_a_given_from_square_and_a_list_of_to_squares() {
    let mut move_list = Vec::new(); generate_pawn_moves_from_to_squares(54, bit(63) | bit(62) | bit(61), &mut move_list);
    let mut algebraic: Vec<String> = move_list.iter().map(|m| { algebraic_move_from_move(*m) }).collect();
    algebraic.sort();
    assert_eq!(algebraic, vec!["b7a8b","b7a8n","b7a8q","b7a8r","b7b8b","b7b8n","b7b8q","b7b8r","b7c8b","b7c8n","b7c8q","b7c8r"]);

    let mut move_list = Vec::new(); generate_pawn_moves_from_to_squares(46, bit(55) | bit(54) | bit(53), &mut move_list);
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
    let mut move_list = Vec::new(); generate_pawn_moves(&position, &mut move_list);
    let mut algebraic: Vec<String> = move_list.iter().map(|m| { algebraic_move_from_move(*m) }).collect();
    algebraic.sort();
    assert_eq!(algebraic, vec!["e7e8b","e7e8n","e7e8q","e7e8r"]);

    let position = get_position(&"n5k1/1P2P1n1/1n2q2p/P1pP4/5R2/3K1B2/1r2N2P/6r1 w - c6 0 1".to_string());
    let mut move_list = Vec::new(); generate_pawn_moves(&position, &mut move_list);
    let mut algebraic: Vec<String> = move_list.iter().map(|m| { algebraic_move_from_move(*m) }).collect();
    algebraic.sort();
    assert_eq!(algebraic, vec!["a5a6","a5b6","b7a8b","b7a8n","b7a8q","b7a8r","b7b8b","b7b8n","b7b8q","b7b8r","d5c6","d5d6","d5e6","e7e8b","e7e8n","e7e8q","e7e8r","h2h3","h2h4"]);
}

#[test]
fn it_determines_if_a_given_square_is_attacked_by_a_given_colour_in_a_given_position() {
    let magic_box = &allocate_magic_boxes();

    let position = get_position(&"n5k1/1P2P1n1/1n5p/P1pP4/5R2/1q3B2/4Nr1P/R3K2R w Q - 0 1".to_string());
    assert_eq!(any_squares_in_bitboard_attacked(&position, BLACK, bit(2) | bit(3), magic_box), true);
    assert_eq!(is_bishop_attacking_square(4, 22, position.all_pieces_bitboard, magic_box), true);
    assert_eq!(is_bishop_attacking_square(5, 22, position.all_pieces_bitboard, magic_box), false);
    assert_eq!(is_square_attacked_by(&position, bitref_from_algebraic_squareref("d1".to_string()) as Square, BLACK, magic_box), true);
    assert_eq!(is_square_attacked_by(&position, 58, WHITE, magic_box), true);
    assert_eq!(is_square_attacked_by(&position, 60, WHITE, magic_box), true);

    let position = get_position(&"n5k1/1P2P1n1/1n2q2p/P1pP4/5R2/5B2/1r2N2P/R3K1r1 w Q - 0 1".to_string());

    assert_eq!(is_square_attacked_by(&position, 0, WHITE, magic_box), true);
    assert_eq!(is_square_attacked_by(&position, 0, BLACK, magic_box), true);
    assert_eq!(is_square_attacked_by(&position, 1, WHITE, magic_box), true);
    assert_eq!(is_square_attacked_by(&position, 1, BLACK, magic_box), false);
    assert_eq!(is_square_attacked_by(&position, 2, WHITE, magic_box), true);
    assert_eq!(is_square_attacked_by(&position, 2, BLACK, magic_box), true);
    assert_eq!(is_square_attacked_by(&position, 3, WHITE, magic_box), true);
    assert_eq!(is_square_attacked_by(&position, 3, BLACK, magic_box), true);
    assert_eq!(is_square_attacked_by(&position, 4, WHITE, magic_box), true);
    assert_eq!(is_square_attacked_by(&position, 4, BLACK, magic_box), false);
    assert_eq!(is_square_attacked_by(&position, 5, WHITE, magic_box), true);
    assert_eq!(is_square_attacked_by(&position, 5, BLACK, magic_box), false);
    assert_eq!(is_square_attacked_by(&position, 6, WHITE, magic_box), true);
    assert_eq!(is_square_attacked_by(&position, 6, BLACK, magic_box), true);
    assert_eq!(is_square_attacked_by(&position, 7, WHITE, magic_box), false);
    assert_eq!(is_square_attacked_by(&position, 7, BLACK, magic_box), false);
    assert_eq!(is_square_attacked_by(&position, 8, WHITE, magic_box), false);
    assert_eq!(is_square_attacked_by(&position, 8, BLACK, magic_box), false);
    assert_eq!(is_square_attacked_by(&position, 9, WHITE, magic_box), true);
    assert_eq!(is_square_attacked_by(&position, 9, BLACK, magic_box), true);
    assert_eq!(is_square_attacked_by(&position, 10, WHITE, magic_box), true);
    assert_eq!(is_square_attacked_by(&position, 10, BLACK, magic_box), false);
    assert_eq!(is_square_attacked_by(&position, 11, WHITE, magic_box), true);
    assert_eq!(is_square_attacked_by(&position, 11, BLACK, magic_box), true);
    assert_eq!(is_square_attacked_by(&position, 12, WHITE, magic_box), true);
    assert_eq!(is_square_attacked_by(&position, 12, BLACK, magic_box), true);
    assert_eq!(is_square_attacked_by(&position, 13, WHITE, magic_box), false);
    assert_eq!(is_square_attacked_by(&position, 13, BLACK, magic_box), true);
    assert_eq!(is_square_attacked_by(&position, 14, WHITE, magic_box), false);
    assert_eq!(is_square_attacked_by(&position, 14, BLACK, magic_box), false);
    assert_eq!(is_square_attacked_by(&position, 15, WHITE, magic_box), true);
    assert_eq!(is_square_attacked_by(&position, 15, BLACK, magic_box), true);
    assert_eq!(is_square_attacked_by(&position, 16, WHITE, magic_box), false);
    assert_eq!(is_square_attacked_by(&position, 16, BLACK, magic_box), true);
    assert_eq!(is_square_attacked_by(&position, 17, WHITE, magic_box), true);
    assert_eq!(is_square_attacked_by(&position, 17, BLACK, magic_box), true);
    assert_eq!(is_square_attacked_by(&position, 18, WHITE, magic_box), true);
    assert_eq!(is_square_attacked_by(&position, 18, BLACK, magic_box), false);
    assert_eq!(is_square_attacked_by(&position, 19, WHITE, magic_box), false);
    assert_eq!(is_square_attacked_by(&position, 19, BLACK, magic_box), true);
    assert_eq!(is_square_attacked_by(&position, 40, WHITE, magic_box), false);
    assert_eq!(is_square_attacked_by(&position, 40, BLACK, magic_box), true);
    assert_eq!(is_square_attacked_by(&position, 41, WHITE, magic_box), false);
    assert_eq!(is_square_attacked_by(&position, 41, BLACK, magic_box), true);
    assert_eq!(is_square_attacked_by(&position, 42, WHITE, magic_box), true);
    assert_eq!(is_square_attacked_by(&position, 42, BLACK, magic_box), true);
    assert_eq!(is_square_attacked_by(&position, 43, WHITE, magic_box), true);
    assert_eq!(is_square_attacked_by(&position, 43, BLACK, magic_box), true);
    assert_eq!(is_square_attacked_by(&position, 44, WHITE, magic_box), false);
    assert_eq!(is_square_attacked_by(&position, 44, BLACK, magic_box), true);
    assert_eq!(is_square_attacked_by(&position, 45, WHITE, magic_box), true);
    assert_eq!(is_square_attacked_by(&position, 45, BLACK, magic_box), true);
    assert_eq!(is_square_attacked_by(&position, 61, WHITE, magic_box), true);
    assert_eq!(is_square_attacked_by(&position, 61, BLACK, magic_box), true);
    assert_eq!(is_square_attacked_by(&position, 62, WHITE, magic_box), false);
    assert_eq!(is_square_attacked_by(&position, 62, BLACK, magic_box), false);
    assert_eq!(is_square_attacked_by(&position, 63, WHITE, magic_box), true);
    assert_eq!(is_square_attacked_by(&position, 63, BLACK, magic_box), true);
}

#[test]
fn it_returns_true_if_any_squares_set_in_the_bitboard_are_attacked_by_the_given_attacker() {
    let magic_box = &allocate_magic_boxes();

    let position = get_position(&"n5k1/1P2P1n1/1n2q2p/P1pP4/5R2/5B2/1r2N2P/R3K1r1 w Q - 0 1".to_string());
    let bitboard = 0b0000000000000000000000000000000000000000010110000000000000000000;
    assert_eq!(any_squares_in_bitboard_attacked(&position, WHITE, bitboard, magic_box), false);

    let bitboard = 0b0000000000000000000000000000000000000000111110000000000000000000;
    assert_eq!(any_squares_in_bitboard_attacked(&position, WHITE, bitboard, magic_box), true);
    assert_eq!(any_squares_in_bitboard_attacked(&position, WHITE, bit(60) | bit (61), magic_box), true);
    assert_eq!(any_squares_in_bitboard_attacked(&position, BLACK, EMPTY_CASTLE_SQUARES_WHITE_QUEEN, magic_box), true);

    let position = get_position(&"n5k1/1P2P1n1/1n2q2p/P1pP4/5R2/5B2/1r2N2P/R3K2R w Q - 0 1".to_string());
    assert_eq!(any_squares_in_bitboard_attacked(&position, BLACK, bit(3) | bit (2), magic_box), false);
    assert_eq!(any_squares_in_bitboard_attacked(&position, BLACK, bit(3) | bit (4), magic_box), false);

    let position = get_position(&"n5k1/1P2P1n1/1n5p/P1pP4/5R2/1q3B2/4Nr1P/R3K2R w Q - 0 1".to_string());
    assert_eq!(any_squares_in_bitboard_attacked(&position, BLACK, bit(3) | bit (2), magic_box), true);
    assert_eq!(any_squares_in_bitboard_attacked(&position, BLACK, 0b0000000000000000000000000000000000000000000000000000000000011000, magic_box), true);

    let position = get_position(&"r3k2r/1P2P1n1/1n2q2p/P1pP4/5R2/5B2/1r2N2P/R3K2R b Q - 0 1".to_string());
    assert_eq!(is_square_attacked_by(&position, 60, WHITE, magic_box), true);
    assert_eq!(any_squares_in_bitboard_attacked(&position, WHITE, 0b0001100000000000000000000000000000000000000000000000000000000000, magic_box), true);
    assert_eq!(any_squares_in_bitboard_attacked(&position, WHITE, bit(59) | bit (60), magic_box), true);
}

fn sort_moves(move_list: MoveList) -> Vec<String> {
    let mut algebraic: Vec<String> = move_list.iter().map(|m| { algebraic_move_from_move(*m) }).collect();
    algebraic.sort();
    return algebraic;
}

#[test]
fn it_generates_castle_moves_for_a_given_mover() {
    let magic_box = &allocate_magic_boxes();

    let position = get_position(&"n5k1/1P2P1n1/1n2q2p/P1pP4/5R2/5B2/1r2N2P/R3K1r1 w Q - 0 1".to_string());
    let mut move_list = Vec::new(); generate_castle_moves(&position, &mut move_list, magic_box);
    assert_eq!(sort_moves(move_list).len(), 0);

    let position = get_position(&"n5k1/1P2P1n1/1n2q2p/P1pP4/5R2/5B2/1r2N2P/R3K2R w KQ - 0 1".to_string());
    let mut move_list = Vec::new(); generate_castle_moves(&position, &mut move_list, magic_box);
    assert_eq!(sort_moves(move_list), vec!["e1c1", "e1g1"]);

    let position = get_position(&"n5k1/1P2P1n1/1n2q2p/P1pP4/5R2/5B2/3rN2P/R3K2R w KQ - 0 1".to_string());
    let mut move_list = Vec::new(); generate_castle_moves(&position, &mut move_list, magic_box);
    assert_eq!(sort_moves(move_list), vec!["e1g1"]);

    let position = get_position(&"n5k1/1P2P1n1/1n2q2p/P1pP4/5R2/5B2/4Nr1P/R3K2R w Q - 0 1".to_string());
    let mut move_list = Vec::new(); generate_castle_moves(&position, &mut move_list, magic_box);
    assert_eq!(sort_moves(move_list), vec!["e1c1"]);

    let position = get_position(&"n5k1/1P2P1n1/1n5p/P1pP4/5R2/1q3B2/4Nr1P/R3K2R w Q - 0 1".to_string());
    let mut move_list = Vec::new(); generate_castle_moves(&position, &mut move_list, magic_box);
    assert_eq!(sort_moves(move_list).len(), 0);

    let position = get_position(&"r3k1R1/1P2P1n1/1n2q2p/P1pP4/5R2/5B2/1r2N2P/R3K1r1 b Q - 0 1".to_string());
    let mut move_list = Vec::new(); generate_castle_moves(&position, &mut move_list, magic_box);
    assert_eq!(sort_moves(move_list).len(), 0);

    let position = get_position(&"r3k2r/1P2P1n1/1n2q2p/P1pP4/5R2/5B2/1r2N2P/R3K2R b Q - 0 1".to_string());
    let mut move_list = Vec::new(); generate_castle_moves(&position, &mut move_list, magic_box);
    assert_eq!(sort_moves(move_list).len(), 0);

    let position = get_position(&"r3k2r/1P2PRn1/1n2q2p/P1pP4/8/5B2/1r2N2P/R3K2R b Q - 0 1".to_string());
    let mut move_list = Vec::new(); generate_castle_moves(&position, &mut move_list, magic_box);
    assert_eq!(sort_moves(move_list).len(), 0);

    let position = get_position(&"r3k2r/1P3Rn1/1n2q2p/P1pP2P1/8/5B2/1r2N2P/R3K2R b qQ - 0 1".to_string());
    let mut move_list = Vec::new(); generate_castle_moves(&position, &mut move_list, magic_box);
    assert_eq!(sort_moves(move_list), vec!["e8c8"]);
}

#[test]
pub fn it_checks_for_check() {
    let magic_box = &allocate_magic_boxes();

    assert_eq!(is_check(&get_position(&"5k2/8/6N1/4K3/8/8/8/8 b - - 0 1".to_string()), BLACK, magic_box), true);
    assert_eq!(is_check(&get_position(&"8/4k3/6N1/4K3/8/8/8/8 b - - 0 1".to_string()), BLACK, magic_box), true);
    assert_eq!(is_check(&get_position(&"3k4/8/6N1/4K3/8/8/8/8 b - - 0 1".to_string()), BLACK, magic_box), false);
    assert_eq!(is_check(&get_position(&"8/3k4/6N1/4K3/8/8/8/8 b - - 0 1".to_string()), BLACK, magic_box), false);
    assert_eq!(is_check(&get_position(&"8/5k2/6N1/4K3/8/8/8/8 b - - 0 1".to_string()), BLACK, magic_box), false);

    assert_eq!(is_check(&get_position(&"n5k1/1P2P1n1/1n5p/P1pP4/5R2/1q3B2/4Nr1P/R3K2R w Q - 0 1".to_string()), WHITE, magic_box), false);
    assert_eq!(is_check(&get_position(&"n5k1/1P2P1n1/1n5p/P1pP4/5R2/1q3B2/4Nr1P/R3K2R w Q - 0 1".to_string()), BLACK, magic_box), false);

    assert_eq!(is_check(&get_position(&"n4Rk1/1P2P1n1/1n5p/P1pP4/8/1q3B2/4Nr1P/R3K2R w Q - 0 1".to_string()), WHITE, magic_box), false);
    assert_eq!(is_check(&get_position(&"n4Rk1/1P2P1n1/1n5p/P1pP4/8/1q3B2/4Nr1P/R3K2R w Q - 0 1".to_string()), BLACK, magic_box), true);

    assert_eq!(is_check(&get_position(&"n4Rk1/1P2P1n1/1n5p/P1pP4/8/2q2B2/4Nr1P/R3K2R w Q - 0 1".to_string()), WHITE, magic_box), true);
    assert_eq!(is_check(&get_position(&"n4Rk1/1P2P1n1/1n5p/P1pP4/8/2q2B2/4Nr1P/R3K2R w Q - 0 1".to_string()), BLACK, magic_box), true);

    assert_eq!(is_check(&get_position(&"n5k1/1P3Pn1/1n5p/P1pP1R2/8/3q1B2/4Nr1P/R3K2R w Q - 0 1".to_string()), WHITE, magic_box), false);
    assert_eq!(is_check(&get_position(&"n5k1/1P3Pn1/1n5p/P1pP1R2/8/3q1B2/4Nr1P/R3K2R w Q - 0 1".to_string()), BLACK, magic_box), true);

    assert_eq!(is_check(&get_position(&"n5k1/1P2P1n1/1n5p/P1pP1R2/8/3q1B2/4N2P/R3Kr1R w Q - 0 1".to_string()), WHITE, magic_box), true);
    assert_eq!(is_check(&get_position(&"n5k1/1P2P1n1/1n5p/P1pP1R2/8/3q1B2/4N2P/R3Kr1R w Q - 0 1".to_string()), BLACK, magic_box), false);

    assert_eq!(is_check(&get_position(&"r2k3r/p6p/8/B7/1p2p3/2pb4/P4K1P/R6R w - - 0 1".to_string()), BLACK, magic_box), true);

}

#[test]
pub fn it_gets_all_moves_for_a_position() {
    let magic_box = &allocate_magic_boxes();
    let mut history = default_position_history();

    assert_eq!(sort_moves(moves(&get_position(&"4k3/8/6N1/4K3/8/8/8/8 b - - 0 1".to_string()), magic_box)), vec!["e8d7", "e8d8", "e8e7", "e8f7", "e8f8"]);

    // let position = get_position(&"4k3/8/6N1/4K3/8/8/8/8 b - - 0 1".to_string());
    // let no_checks = moves(&position, magic_box).into_iter().filter(| m| {
    //     let mut position = get_position(&"4k3/8/6N1/4K3/8/8/8/8 b - - 0 1".to_string());
    //     make_move(&mut position, *m, &mut history);
    //     !is_check(&position, switch_side(position.mover), magic_box)
    // }).collect();
    // assert_eq!(sort_moves(no_checks), vec!["e8d7","e8d8","e8f7"]);

    assert_eq!(sort_moves(moves(&get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/1R2K2R b Kkq - 0 1".to_string()), magic_box)), vec!["a7a6","a8b8","a8c8","a8d8","b4b3","c4c3","d3b1","d3c2","d3e2","d3f1","e4e3","e8d7","e8d8","e8e7","e8f7","e8f8","e8g8","h7h5","h7h6","h8f8","h8g8"]);
    assert_eq!(sort_moves(moves(&get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P4K1P/R6R b kq - 0 1".to_string()), magic_box)), vec!["a7a6","a8b8","a8c8","a8d8","b4b3","c4c3","d3b1","d3c2","d3e2","d3f1","e4e3","e8d7","e8d8","e8e7","e8f7","e8f8","e8g8","h7h5","h7h6","h8f8","h8g8"]);
    assert_eq!(sort_moves(moves(&get_position(&"5k2/7p/p3B1p1/P4pP1/3K1P1P/8/8/8 w - f6 0 1".to_string()), magic_box)), vec!["d4c3","d4c4","d4c5","d4d3","d4d5","d4e3","d4e4","d4e5","e6a2","e6b3","e6c4","e6c8","e6d5","e6d7","e6f5","e6f7","e6g8","g5f6","h4h5"]);
    assert_eq!(sort_moves(moves(&get_position(&"6k1/5p1p/p3B1p1/P5P1/3K1P1P/8/8/8 w - - 0 1".to_string()), magic_box)), vec!["d4c3","d4c4","d4c5","d4d3","d4d5","d4e3","d4e4","d4e5","e6a2","e6b3","e6c4","e6c8","e6d5","e6d7","e6f5","e6f7","e6g4","e6h3","f4f5","h4h5"]);

    assert_eq!(sort_moves(moves(&get_position(&"n5k1/1P2P1n1/1n2q2p/P1pP4/5R2/3K1B2/1r2N2P/6r1 w - c6 0 1".to_string()), magic_box)),
               vec!["a5a6","a5b6","b7a8b","b7a8n","b7a8q","b7a8r","b7b8b","b7b8n","b7b8q","b7b8r",
                    "d3c2","d3c3","d3c4","d3d2","d3d4","d3e3","d3e4",
                    "d5c6","d5d6","d5e6",
                    "e2c1","e2c3","e2d4","e2g1","e2g3",
                    "e7e8b","e7e8n","e7e8q","e7e8r",
                    "f3e4","f3g2","f3g4","f3h1","f3h5",
                    "f4a4","f4b4","f4c4","f4d4","f4e4","f4f5","f4f6","f4f7","f4f8","f4g4","f4h4",
                    "h2h3","h2h4",
               ]);

    assert_eq!(sort_moves(moves(&get_position(&"n5k1/1P2P1n1/1n2q2p/P1pP4/3p1R2/2p2B2/1rPPN2P/R3K1r1 w Q - 0 1".to_string()), magic_box)),
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

    // let position = get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R3K2R w KQkq - 0 1".to_string());
    // let no_checks = moves(&position, magic_box).into_iter().filter(| m| {
    //     let mut position = get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R3K2R w KQkq - 0 1".to_string());
    //     make_move(&mut position, *m, &mut history);
    //     !is_check(&position, switch_side(position.mover), magic_box)
    // }).collect();
    // assert_eq!(sort_moves(no_checks), vec!["a1b1", "a1c1", "a1d1", "a2a3", "a2a4", "a5b4", "a5b6", "a5c7", "a5d8", "e1c1", "e1d1", "e1d2", "e1f2", "h1f1", "h1g1", "h2h3", "h2h4"]);
    //
    // let position = get_position(&"8/8/p7/1P6/K1k3pP/6P1/8/8 b - - 0 1".to_string());
    // let no_checks = moves(&position, magic_box).into_iter().filter(| m| {
    //     let mut position = get_position(&"8/8/p7/1P6/K1k3pP/6P1/8/8 b - - 0 1".to_string());
    //     make_move(&mut position, *m, &mut history);
    //     !is_check(&position, switch_side(position.mover), magic_box)
    // }).collect();
    // assert_eq!(sort_moves(no_checks), vec!["a6a5", "a6b5", "c4c3", "c4c5", "c4d3", "c4d4", "c4d5"]);


}
