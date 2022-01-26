use rusty_rival::bitboards::{bit, EMPTY_CASTLE_SQUARES_WHITE_QUEEN, RANK_4_BITS, WHITE_PAWN_MOVES_CAPTURE, WHITE_PAWN_MOVES_FORWARD};
use rusty_rival::fen::{algebraic_move_from_move, bitref_from_algebraic_squareref, get_position};
use rusty_rival::magic_bitboards::MAGIC_BOX;
use rusty_rival::make_move::{default_position_history, make_move};
use rusty_rival::move_constants::{EN_PASSANT_NOT_AVAILABLE, PIECE_MASK_FULL};
use rusty_rival::moves::{any_squares_in_bitboard_attacked, generate_slider_moves, is_check, is_square_attacked, moves};
use rusty_rival::opponent;
use rusty_rival::types::{BLACK, MoveList, Square, WHITE};

#[test]
fn it_gets_all_pieces_bitboard() {
    let position = get_position(&"n5k1/6n1/1n2q2p/1p5P/1P3RP1/2PK1B2/1r2N3/8 b kQKq g3 5 56".to_string());
    assert_eq!((position.pieces[WHITE as usize].all_pieces_bitboard | position.pieces[BLACK as usize].all_pieces_bitboard), 0b1000001000000010010010010100000101000110001101000100100000000000);
}

#[test]
fn it_generates_bishop_moves_including_diagonal_queen_moves_from_a_given_fen_ignoring_checks() {
    let position = get_position(&"n5k1/6n1/1n2q2p/1p5P/1P3RP1/2PK1B2/1r2N3/7R w kQKq g3 5 56".to_string());
    let mut move_list = Vec::new(); generate_slider_moves(position.pieces[WHITE as usize].queen_bitboard | position.pieces[WHITE as usize].bishop_bitboard, position.pieces[WHITE as usize].all_pieces_bitboard | position.pieces[BLACK as usize].all_pieces_bitboard, &mut move_list, &MAGIC_BOX.bishop, !position.pieces[WHITE as usize].all_pieces_bitboard);
    let mut algebraic: Vec<String> = move_list.iter().map(|m| { algebraic_move_from_move(*m) }).collect();
    algebraic.sort();
    assert_eq!(algebraic, vec!["f3a8","f3b7","f3c6","f3d5","f3e4","f3g2"]);

    let position = get_position(&"n5k1/6n1/1n2q2p/1p5P/1P3RP1/2PK1B2/1r2N3/8 b kQKq g3 5 56".to_string());
    let mut move_list = Vec::new(); generate_slider_moves(position.pieces[BLACK as usize].queen_bitboard | position.pieces[BLACK as usize].bishop_bitboard, position.pieces[WHITE as usize].all_pieces_bitboard | position.pieces[BLACK as usize].all_pieces_bitboard, &mut move_list, &MAGIC_BOX.bishop, !position.pieces[BLACK as usize].all_pieces_bitboard);
    let mut algebraic: Vec<String> = move_list.iter().map(|m| { algebraic_move_from_move(*m) }).collect();
    algebraic.sort();
    assert_eq!(algebraic, vec!["e6a2","e6b3","e6c4","e6c8","e6d5","e6d7","e6f5","e6f7","e6g4"]);
}

#[test]
fn it_generates_rook_moves_including_horizontal_queen_moves_from_a_given_fen_ignoring_checks() {
    let position = get_position(&"n5k1/6n1/1n2q2p/1p5P/1P3RP1/2PK1B2/1r2N3/8 w kQKq g3 5 56".to_string());
    let mut move_list = Vec::new(); generate_slider_moves(position.pieces[WHITE as usize].queen_bitboard | position.pieces[WHITE as usize].rook_bitboard, position.pieces[WHITE as usize].all_pieces_bitboard | position.pieces[BLACK as usize].all_pieces_bitboard, &mut move_list, &MAGIC_BOX.rook, !position.pieces[WHITE as usize].all_pieces_bitboard);
    let algebraic = sort_moves(move_list);
    assert_eq!(algebraic, vec!["f4c4","f4d4","f4e4","f4f5","f4f6","f4f7","f4f8"]);

    let position = get_position(&"n5k1/6n1/1n2q2p/1p5P/1P3RP1/2PK1B2/1r2N3/6r1 b kQKq g3 5 56".to_string());
    let mut move_list = Vec::new(); generate_slider_moves(position.pieces[BLACK as usize].queen_bitboard | position.pieces[BLACK as usize].rook_bitboard, position.pieces[WHITE as usize].all_pieces_bitboard | position.pieces[BLACK as usize].all_pieces_bitboard, &mut move_list, &MAGIC_BOX.rook, !position.pieces[BLACK as usize].all_pieces_bitboard);
    let mut algebraic: Vec<String> = move_list.iter().map(|m| { algebraic_move_from_move(*m) }).collect();
    algebraic.sort();
    assert_eq!(algebraic, vec!["b2a2","b2b1","b2b3","b2b4","b2c2","b2d2","b2e2","e6c6","e6d6","e6e2","e6e3","e6e4","e6e5","e6e7","e6e8","e6f6","e6g6","g1a1","g1b1","g1c1","g1d1","g1e1","g1f1","g1g2","g1g3","g1g4","g1h1"]);
}

#[test]
fn it_returns_a_bitboard_showing_target_squares_for_pawn_captures_from_a_given_square_and_an_enemy_piece_bitboard() {
    let position = get_position(&"n5k1/1P4n1/1n2q2p/Pp3P2/3P1R2/3K1B2/1r2N2P/6r1 w - - 0 1".to_string());

    assert_eq!(WHITE_PAWN_MOVES_CAPTURE[29] & position.pieces[BLACK as usize].all_pieces_bitboard, 0b0000000000000000000000000100000000000000000000000000000000000000);

    let position = get_position(&"n5k1/1P2P1n1/1n2q2p/Pp1pP3/3P1R2/3K1B2/1r2N2P/6r1 w - - 0 1".to_string());
    assert_eq!(WHITE_PAWN_MOVES_CAPTURE[51] & position.pieces[BLACK as usize].all_pieces_bitboard, 0b0000000000000000000000000000000000000000000000000000000000000000);

    let position = get_position(&"n5k1/1P2P1n1/1n2q2p/Pp1pP3/3P1R2/3K1B2/1r2N2P/6r1 w - - 0 1".to_string());
    assert_eq!(WHITE_PAWN_MOVES_CAPTURE[54] & position.pieces[BLACK as usize].all_pieces_bitboard, 0b1000000000000000000000000000000000000000000000000000000000000000);

    let position = get_position(&"n5k1/4P1n1/1n2q2p/1p1p4/5R2/3K1B2/1r2N3/6r1 w - - 0 1".to_string());
    assert_eq!(WHITE_PAWN_MOVES_CAPTURE[51] & position.pieces[BLACK as usize].all_pieces_bitboard, 0b0000000000000000000000000000000000000000000000000000000000000000);
}

#[test]
fn it_returns_a_bitboard_showing_target_squares_for_pawn_moves_that_would_land_on_the_two_move_rank_if_moved_one_more_rank() {
    let bb = 0b0101000000000100010000011000000001000000010101010000001100010001;
    assert_eq!((bb << 8) & RANK_4_BITS, 0b0000000000000000000000000000000001010101000000000000000000000000);
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
    let empty_squares = !(position.pieces[WHITE as usize].all_pieces_bitboard | position.pieces[BLACK as usize].all_pieces_bitboard);
    let from_square = 51;
    let forward_moves_for_square = WHITE_PAWN_MOVES_FORWARD.iter().nth(from_square).unwrap();
    assert_eq!(*forward_moves_for_square, 0b0000100000000000000000000000000000000000000000000000000000000000);
    let pawn_moves = forward_moves_for_square & empty_squares;
    let bb = pawn_moves;
    let pfmb = pawn_moves | ((bb << 8) & RANK_4_BITS & !(position.pieces[WHITE as usize].all_pieces_bitboard | position.pieces[BLACK as usize].all_pieces_bitboard));
    assert_eq!(pfmb, 0b0000100000000000000000000000000000000000000000000000000000000000);
}

#[test]
fn it_determines_if_a_given_square_is_attacked_by_a_given_colour_in_a_given_position() {
    let position = get_position(&"n5k1/1P2P1n1/1n5p/P1pP4/5R2/1q3B2/4Nr1P/R3K2R w Q - 0 1".to_string());
    assert_eq!(any_squares_in_bitboard_attacked(&position, WHITE, bit(2) | bit(3)), true);
    assert_eq!(is_square_attacked(&position, bitref_from_algebraic_squareref("d1".to_string()) as Square, WHITE), true);
    assert_eq!(is_square_attacked(&position, 58, BLACK), true);
    assert_eq!(is_square_attacked(&position, 60, BLACK), true);

    let position = get_position(&"n5k1/1P2P1n1/1n2q2p/P1pP4/5R2/5B2/1r2N2P/R3K1r1 w Q - 0 1".to_string());

    assert_eq!(is_square_attacked(&position, 0, BLACK), true);
    assert_eq!(is_square_attacked(&position, 0, WHITE), true);
    assert_eq!(is_square_attacked(&position, 1, BLACK), true);
    assert_eq!(is_square_attacked(&position, 1, WHITE), false);
    assert_eq!(is_square_attacked(&position, 2, BLACK), true);
    assert_eq!(is_square_attacked(&position, 2, WHITE), true);
    assert_eq!(is_square_attacked(&position, 3, BLACK), true);
    assert_eq!(is_square_attacked(&position, 3, WHITE), true);
    assert_eq!(is_square_attacked(&position, 4, BLACK), true);
    assert_eq!(is_square_attacked(&position, 4, WHITE), false);
    assert_eq!(is_square_attacked(&position, 5, BLACK), true);
    assert_eq!(is_square_attacked(&position, 5, WHITE), false);
    assert_eq!(is_square_attacked(&position, 6, BLACK), true);
    assert_eq!(is_square_attacked(&position, 6, WHITE), true);
    assert_eq!(is_square_attacked(&position, 7, BLACK), false);
    assert_eq!(is_square_attacked(&position, 7, WHITE), false);
    assert_eq!(is_square_attacked(&position, 8, BLACK), false);
    assert_eq!(is_square_attacked(&position, 8, WHITE), false);
    assert_eq!(is_square_attacked(&position, 9, BLACK), true);
    assert_eq!(is_square_attacked(&position, 9, WHITE), true);
    assert_eq!(is_square_attacked(&position, 10, BLACK), true);
    assert_eq!(is_square_attacked(&position, 10, WHITE), false);
    assert_eq!(is_square_attacked(&position, 11, BLACK), true);
    assert_eq!(is_square_attacked(&position, 11, WHITE), true);
    assert_eq!(is_square_attacked(&position, 12, BLACK), true);
    assert_eq!(is_square_attacked(&position, 12, WHITE), true);
    assert_eq!(is_square_attacked(&position, 13, BLACK), false);
    assert_eq!(is_square_attacked(&position, 13, WHITE), true);
    assert_eq!(is_square_attacked(&position, 14, BLACK), false);
    assert_eq!(is_square_attacked(&position, 14, WHITE), false);
    assert_eq!(is_square_attacked(&position, 15, BLACK), true);
    assert_eq!(is_square_attacked(&position, 15, WHITE), true);
    assert_eq!(is_square_attacked(&position, 16, BLACK), false);
    assert_eq!(is_square_attacked(&position, 16, WHITE), true);
    assert_eq!(is_square_attacked(&position, 17, BLACK), true);
    assert_eq!(is_square_attacked(&position, 17, WHITE), true);
    assert_eq!(is_square_attacked(&position, 18, BLACK), true);
    assert_eq!(is_square_attacked(&position, 18, WHITE), false);
    assert_eq!(is_square_attacked(&position, 19, BLACK), false);
    assert_eq!(is_square_attacked(&position, 19, WHITE), true);
    assert_eq!(is_square_attacked(&position, 40, BLACK), false);
    assert_eq!(is_square_attacked(&position, 40, WHITE), true);
    assert_eq!(is_square_attacked(&position, 41, BLACK), false);
    assert_eq!(is_square_attacked(&position, 41, WHITE), true);
    assert_eq!(is_square_attacked(&position, 42, BLACK), true);
    assert_eq!(is_square_attacked(&position, 42, WHITE), true);
    assert_eq!(is_square_attacked(&position, 43, BLACK), true);
    assert_eq!(is_square_attacked(&position, 43, WHITE), true);
    assert_eq!(is_square_attacked(&position, 44, BLACK), false);
    assert_eq!(is_square_attacked(&position, 44, WHITE), true);
    assert_eq!(is_square_attacked(&position, 45, BLACK), true);
    assert_eq!(is_square_attacked(&position, 45, WHITE), true);
    assert_eq!(is_square_attacked(&position, 61, BLACK), true);
    assert_eq!(is_square_attacked(&position, 61, WHITE), true);
    assert_eq!(is_square_attacked(&position, 62, BLACK), false);
    assert_eq!(is_square_attacked(&position, 62, WHITE), false);
    assert_eq!(is_square_attacked(&position, 63, BLACK), true);
    assert_eq!(is_square_attacked(&position, 63, WHITE), true);
}

#[test]
fn it_returns_true_if_any_squares_set_in_the_bitboard_are_attacked_by_the_given_attacker() {
    let position = get_position(&"n5k1/1P2P1n1/1n2q2p/P1pP4/5R2/5B2/1r2N2P/R3K1r1 w Q - 0 1".to_string());
    let bitboard = 0b0000000000000000000000000000000000000000010110000000000000000000;
    assert_eq!(any_squares_in_bitboard_attacked(&position, BLACK, bitboard), false);

    let bitboard = 0b0000000000000000000000000000000000000000111110000000000000000000;
    assert_eq!(any_squares_in_bitboard_attacked(&position, BLACK, bitboard), true);
    assert_eq!(any_squares_in_bitboard_attacked(&position, BLACK, bit(60) | bit (61)), true);
    assert_eq!(any_squares_in_bitboard_attacked(&position, WHITE, EMPTY_CASTLE_SQUARES_WHITE_QUEEN), true);

    let position = get_position(&"n5k1/1P2P1n1/1n2q2p/P1pP4/5R2/5B2/1r2N2P/R3K2R w Q - 0 1".to_string());
    assert_eq!(any_squares_in_bitboard_attacked(&position, WHITE, bit(3) | bit (2)), false);
    assert_eq!(any_squares_in_bitboard_attacked(&position, WHITE, bit(3) | bit (4)), false);

    let position = get_position(&"n5k1/1P2P1n1/1n5p/P1pP4/5R2/1q3B2/4Nr1P/R3K2R w Q - 0 1".to_string());
    assert_eq!(any_squares_in_bitboard_attacked(&position, WHITE, bit(3) | bit (2)), true);
    assert_eq!(any_squares_in_bitboard_attacked(&position, WHITE, 0b0000000000000000000000000000000000000000000000000000000000011000), true);

    let position = get_position(&"r3k2r/1P2P1n1/1n2q2p/P1pP4/5R2/5B2/1r2N2P/R3K2R b Q - 0 1".to_string());
    assert_eq!(is_square_attacked(&position, 60, BLACK), true);
    assert_eq!(any_squares_in_bitboard_attacked(&position, BLACK, 0b0001100000000000000000000000000000000000000000000000000000000000), true);
    assert_eq!(any_squares_in_bitboard_attacked(&position, BLACK, bit(59) | bit (60)), true);
}

fn sort_moves(move_list: MoveList) -> Vec<String> {
    let mut algebraic: Vec<String> = move_list.iter().map(|m| { algebraic_move_from_move(*m & !PIECE_MASK_FULL) }).collect();
    algebraic.sort();
    return algebraic;
}


#[test]
pub fn it_checks_for_check() {
    assert_eq!(is_check(&get_position(&"5k2/8/6N1/4K3/8/8/8/8 b - - 0 1".to_string()), BLACK), true);
    assert_eq!(is_check(&get_position(&"8/4k3/6N1/4K3/8/8/8/8 b - - 0 1".to_string()), BLACK), true);
    assert_eq!(is_check(&get_position(&"3k4/8/6N1/4K3/8/8/8/8 b - - 0 1".to_string()), BLACK), false);
    assert_eq!(is_check(&get_position(&"8/3k4/6N1/4K3/8/8/8/8 b - - 0 1".to_string()), BLACK), false);
    assert_eq!(is_check(&get_position(&"8/5k2/6N1/4K3/8/8/8/8 b - - 0 1".to_string()), BLACK), false);

    assert_eq!(is_check(&get_position(&"n5k1/1P2P1n1/1n5p/P1pP4/5R2/1q3B2/4Nr1P/R3K2R w Q - 0 1".to_string()), WHITE), false);
    assert_eq!(is_check(&get_position(&"n5k1/1P2P1n1/1n5p/P1pP4/5R2/1q3B2/4Nr1P/R3K2R w Q - 0 1".to_string()), BLACK), false);

    assert_eq!(is_check(&get_position(&"n4Rk1/1P2P1n1/1n5p/P1pP4/8/1q3B2/4Nr1P/R3K2R w Q - 0 1".to_string()), WHITE), false);
    assert_eq!(is_check(&get_position(&"n4Rk1/1P2P1n1/1n5p/P1pP4/8/1q3B2/4Nr1P/R3K2R w Q - 0 1".to_string()), BLACK), true);

    assert_eq!(is_check(&get_position(&"n4Rk1/1P2P1n1/1n5p/P1pP4/8/2q2B2/4Nr1P/R3K2R w Q - 0 1".to_string()), WHITE), true);
    assert_eq!(is_check(&get_position(&"n4Rk1/1P2P1n1/1n5p/P1pP4/8/2q2B2/4Nr1P/R3K2R w Q - 0 1".to_string()), BLACK), true);

    assert_eq!(is_check(&get_position(&"n5k1/1P3Pn1/1n5p/P1pP1R2/8/3q1B2/4Nr1P/R3K2R w Q - 0 1".to_string()), WHITE), false);
    assert_eq!(is_check(&get_position(&"n5k1/1P3Pn1/1n5p/P1pP1R2/8/3q1B2/4Nr1P/R3K2R w Q - 0 1".to_string()), BLACK), true);

    assert_eq!(is_check(&get_position(&"n5k1/1P2P1n1/1n5p/P1pP1R2/8/3q1B2/4N2P/R3Kr1R w Q - 0 1".to_string()), WHITE), true);
    assert_eq!(is_check(&get_position(&"n5k1/1P2P1n1/1n5p/P1pP1R2/8/3q1B2/4N2P/R3Kr1R w Q - 0 1".to_string()), BLACK), false);

    assert_eq!(is_check(&get_position(&"r2k3r/p6p/8/B7/1p2p3/2pb4/P4K1P/R6R w - - 0 1".to_string()), BLACK), true);

}

#[test]
pub fn it_gets_all_moves_for_a_position() {
    let mut history = default_position_history();

    assert_eq!(sort_moves(moves(&get_position(&"4k3/8/6N1/4K3/8/8/8/8 b - - 0 1".to_string()))), vec!["e8d7", "e8d8", "e8e7", "e8f7", "e8f8"]);

    let position = get_position(&"4k3/8/6N1/4K3/8/8/8/8 b - - 0 1".to_string());
    let no_checks = moves(&position).into_iter().filter(| m| {
        let mut position = get_position(&"4k3/8/6N1/4K3/8/8/8/8 b - - 0 1".to_string());
        make_move(&mut position, *m, &mut history);
        !is_check(&position, opponent!(position.mover))
    }).collect();
    assert_eq!(sort_moves(no_checks), vec!["e8d7","e8d8","e8f7"]);

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

    let position = get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R3K2R w KQkq - 0 1".to_string());
    let no_checks = moves(&position).into_iter().filter(| m| {
        let mut position = get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R3K2R w KQkq - 0 1".to_string());
        make_move(&mut position, *m, &mut history);
        !is_check(&position, opponent!(position.mover))
    }).collect();
    assert_eq!(sort_moves(no_checks), vec!["a1b1", "a1c1", "a1d1", "a2a3", "a2a4", "a5b4", "a5b6", "a5c7", "a5d8", "e1c1", "e1d1", "e1d2", "e1f2", "h1f1", "h1g1", "h2h3", "h2h4"]);

    let position = get_position(&"8/8/p7/1P6/K1k3pP/6P1/8/8 b - - 0 1".to_string());
    let no_checks = moves(&position).into_iter().filter(| m| {
        let mut position = get_position(&"8/8/p7/1P6/K1k3pP/6P1/8/8 b - - 0 1".to_string());
        make_move(&mut position, *m, &mut history);
        !is_check(&position, opponent!(position.mover))
    }).collect();
    assert_eq!(sort_moves(no_checks), vec!["a6a5", "a6b5", "c4c3", "c4c5", "c4d3", "c4d4", "c4d5"]);


}
