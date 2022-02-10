use rusty_rival::fen::{get_fen, get_position, move_from_algebraic_move};
use rusty_rival::hash::zobrist_lock;
use rusty_rival::make_move::{make_move};
use rusty_rival::move_constants::{BLACK_QUEEN_CASTLE_MOVE, PIECE_MASK_KING, PIECE_MASK_PAWN, PIECE_MASK_ROOK, WHITE_KING_CASTLE_MOVE};
use rusty_rival::utils::hydrate_move_from_algebraic_move;

#[test]
pub fn it_calculates_correct_zobrist_after_pawn_promotion() {
    let original_position = &get_position(&"rnbqkbr1/ppp1pp1P/5n2/3p4/3P4/2N5/PPP1P1PP/R1BQKBNR w KQq - 0 1".to_string());
    let mut new_position = *original_position;
    make_move(original_position, hydrate_move_from_algebraic_move(original_position, "h7h8q".to_string()), &mut new_position);
    assert_eq!("rnbqkbrQ/ppp1pp2/5n2/3p4/3P4/2N5/PPP1P1PP/R1BQKBNR b KQq - 0 1", get_fen(&new_position));
    assert_eq!(zobrist_lock(&new_position), new_position.zobrist_lock);

    let original_position = &get_position(&"rnbqkbr1/ppp1pp1P/5n2/3p4/3P4/2N5/PPP1P1PP/R1BQKBNR w KQq - 0 1".to_string());
    let mut new_position = *original_position;
    make_move(original_position, hydrate_move_from_algebraic_move(original_position, "h7g8q".to_string()), &mut new_position);
    assert_eq!("rnbqkbQ1/ppp1pp2/5n2/3p4/3P4/2N5/PPP1P1PP/R1BQKBNR b KQq - 0 1", get_fen(&new_position));
    assert_eq!(zobrist_lock(&new_position), new_position.zobrist_lock);

    let original_position = &get_position(&"8/2p5/3p4/1P5r/7k/2K5/4P1p1/1R6 b - - 1 3".to_string());
    let mut new_position = *original_position;
    make_move(original_position, hydrate_move_from_algebraic_move(original_position, "g2g1n".to_string()), &mut new_position);
    assert_eq!("8/2p5/3p4/1P5r/7k/2K5/4P3/1R4n1 w - - 0 4", get_fen(&new_position));
    assert_eq!(zobrist_lock(&new_position), new_position.zobrist_lock);
}

#[test]
pub fn it_calculates_correct_zobrist_after_simple_pawn_move() {
    let original_position = &get_position(&"rnbqkbrQ/ppp1pp2/5n2/3p4/3P4/2N5/PPP1P1PP/R1BQKBNR w KQq - 0 1".to_string());
    let mut new_position = *original_position;
    make_move(original_position, hydrate_move_from_algebraic_move(original_position, "b2b4".to_string()), &mut new_position);
    assert_eq!("rnbqkbrQ/ppp1pp2/5n2/3p4/1P1P4/2N5/P1P1P1PP/R1BQKBNR b KQq b3 0 1", get_fen(&new_position));
    assert_eq!(zobrist_lock(&new_position), new_position.zobrist_lock);

    let original_position = &get_position(&"rnbqkbrQ/ppp1pp2/5n2/3p4/1P1P4/2N5/P1P1P1PP/R1BQKBNR b KQq b3 0 1".to_string());
    let mut new_position = *original_position;
    make_move(original_position, hydrate_move_from_algebraic_move(original_position, "c7c5".to_string()), &mut new_position);
    assert_eq!("rnbqkbrQ/pp2pp2/5n2/2pp4/1P1P4/2N5/P1P1P1PP/R1BQKBNR w KQq c6 0 2", get_fen(&new_position));
    assert_eq!(zobrist_lock(&new_position), new_position.zobrist_lock);
}

#[test]
pub fn it_calculates_correct_zobrist_after_pawn_capture() {
    let original_position = &get_position(&"8/2p5/3p4/1P5r/1R3p2/1K4P1/4P2k/8 b - - 0 3".to_string());
    let mut new_position = *original_position;
    make_move(original_position, hydrate_move_from_algebraic_move(original_position, "f4g3".to_string()), &mut new_position);
    assert_eq!("8/2p5/3p4/1P5r/1R6/1K4p1/4P2k/8 w - - 0 4", get_fen(&new_position));
    assert_eq!(zobrist_lock(&new_position), new_position.zobrist_lock);
}

#[test]
pub fn it_calculates_correct_zobrist_after_en_passant_capture() {
    let original_position = &get_position(&"8/2p5/3p4/1P5r/1R2Pp2/1K4P1/7k/8 b - e3 0 1".to_string());
    let mut new_position = *original_position;
    make_move(original_position, hydrate_move_from_algebraic_move(original_position, "f4e3".to_string()), &mut new_position);
    assert_eq!("8/2p5/3p4/1P5r/1R6/1K2p1P1/7k/8 w - - 0 2", get_fen(&new_position));
    assert_eq!(zobrist_lock(&new_position), new_position.zobrist_lock);
}

#[test]
pub fn it_calculates_correct_zobrist_after_en_passant_option_removed() {
    let original_position = &get_position(&"8/2p5/3p4/1P5r/1R2Pp2/1K4P1/7k/8 b - e3 0 1".to_string());
    let mut new_position = *original_position;
    make_move(original_position, hydrate_move_from_algebraic_move(original_position, "h5g5".to_string()), &mut new_position);
    assert_eq!("8/2p5/3p4/1P4r1/1R2Pp2/1K4P1/7k/8 w - - 1 2", get_fen(&new_position));
    assert_eq!(zobrist_lock(&new_position), new_position.zobrist_lock);
}

#[test]
pub fn it_calculates_correct_zobrist_after_a_castle_move() {
    let original_position = &get_position(&"rnb1kbrQ/pp2pp2/3q1n2/2pp2B1/1P1P4/2NQ4/P1P1P1PP/R3KBNR w KQq - 0 1".to_string());
    let mut new_position = *original_position;
    make_move(original_position, hydrate_move_from_algebraic_move(original_position, "e1c1".to_string()), &mut new_position);
    assert_eq!("rnb1kbrQ/pp2pp2/3q1n2/2pp2B1/1P1P4/2NQ4/P1P1P1PP/2KR1BNR b q - 1 1", get_fen(&new_position));
    assert_eq!(zobrist_lock(&new_position), new_position.zobrist_lock);
}

#[test]
pub fn it_calculates_correct_zobrist_after_a_king_move() {
    let original_position = &get_position(&"rnbqkbQ1/ppp1pp2/5n2/3p4/3P4/2N5/PPP1P1PP/R1BQKBNR w KQq - 0 1".to_string());
    let mut new_position = *original_position;
    make_move(original_position, hydrate_move_from_algebraic_move(original_position, "e1d2".to_string()), &mut new_position);
    assert_eq!("rnbqkbQ1/ppp1pp2/5n2/3p4/3P4/2N5/PPPKP1PP/R1BQ1BNR b q - 1 1", get_fen(&new_position));
    assert_eq!(zobrist_lock(&new_position), new_position.zobrist_lock);
}

#[test]
pub fn it_calculates_correct_zobrist_after_a_king_makes_a_capture() {
    let original_position = &get_position(&"rnbqkbQ1/ppp1pp2/8/3p4/3P4/2N1n3/PPPKP1PP/R1BQ1BNR w q - 0 1".to_string());
    let mut new_position = *original_position;
    make_move(original_position, hydrate_move_from_algebraic_move(original_position, "d2e3".to_string()), &mut new_position);
    assert_eq!("rnbqkbQ1/ppp1pp2/8/3p4/3P4/2N1K3/PPP1P1PP/R1BQ1BNR b q - 0 1", get_fen(&new_position));
    assert_eq!(zobrist_lock(&new_position), new_position.zobrist_lock);
}

#[test]
pub fn it_calculates_correct_zobrist_after_a_capture() {
    let original_position = &get_position(&"rnbqk1Q1/ppp1pp2/7b/3p4/3P4/2N1K3/PPP1P1PP/R1BQ1BNR w q - 0 1".to_string());
    let mut new_position = *original_position;
    make_move(original_position, hydrate_move_from_algebraic_move(original_position, "g8f7".to_string()), &mut new_position);
    assert_eq!("rnbqk3/ppp1pQ2/7b/3p4/3P4/2N1K3/PPP1P1PP/R1BQ1BNR b q - 0 1", get_fen(&new_position));
    assert_eq!(zobrist_lock(&new_position), new_position.zobrist_lock);
}

#[test]
pub fn it_calculates_correct_zobrist_after_a_non_capture() {
    let original_position = &get_position(&"rnbqkbQ1/ppp1pp2/8/3p4/3P4/2N1K3/PPP1P1PP/R1BQ1BNR b q - 0 1".to_string());
    let mut new_position = *original_position;
    make_move(original_position, hydrate_move_from_algebraic_move(original_position, "f8h6".to_string()), &mut new_position);
    assert_eq!("rnbqk1Q1/ppp1pp2/7b/3p4/3P4/2N1K3/PPP1P1PP/R1BQ1BNR w q - 1 2", get_fen(&new_position));
    assert_eq!(zobrist_lock(&new_position), new_position.zobrist_lock);
}

#[test]
pub fn it_makes_a_move() {
    let original_position = &get_position(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string());
    let mut new_position = *original_position;
    make_move(original_position, move_from_algebraic_move("e2e3".to_string(), PIECE_MASK_PAWN), &mut new_position);
    assert_eq!(get_position(&"rnbqkbnr/pppppppp/8/8/8/4P3/PPPP1PPP/RNBQKBNR b KQkq - 0 1".to_string()), new_position);

    let original_position = &get_position(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQK2R w KQkq - 0 1".to_string());
    let mut new_position = *original_position;
    make_move(original_position, WHITE_KING_CASTLE_MOVE, &mut new_position);
    assert_eq!(get_position(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQ1RK1 b kq - 1 1".to_string()), new_position);

    let original_position = &get_position(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQK2R w KQkq - 0 1".to_string());
    let mut new_position = *original_position;
    make_move(original_position, move_from_algebraic_move("h1g1".to_string(), PIECE_MASK_ROOK), &mut new_position);
    assert_eq!(get_position(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQK1R1 b kqQ - 1 1".to_string()), new_position);

    let original_position = &get_position(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQK2R w KQkq - 0 1".to_string());
    let mut new_position = *original_position;
    make_move(original_position, move_from_algebraic_move("e2e3".to_string(), PIECE_MASK_PAWN), &mut new_position);
    assert_eq!(get_position(&"rnbqkbnr/pppppppp/8/8/8/4P3/PPPP1PPP/RNBQK2R b KQkq - 0 1".to_string()), new_position);

    let original_position = &get_position(&"r3k2r/pppppppp/2n1b3/2bn1q2/8/4P3/PPPP1PPP/RNBQK2R b KQq - 0 1".to_string());
    let mut new_position = *original_position;
    make_move(original_position, BLACK_QUEEN_CASTLE_MOVE, &mut new_position);
    assert_eq!(get_position(&"2kr3r/pppppppp/2n1b3/2bn1q2/8/4P3/PPPP1PPP/RNBQK2R w KQ - 1 2".to_string()), new_position);

    let original_position = &get_position(&"r3k2r/pppppppp/2n1b3/2bn1q2/8/4P3/PPPP1PPP/RNBQK2R b KQq - 0 1".to_string());
    let mut new_position = *original_position;
    make_move(original_position, move_from_algebraic_move("e8d8".to_string(), PIECE_MASK_KING), &mut new_position);
    assert_eq!(get_position(&"r2k3r/pppppppp/2n1b3/2bn1q2/8/4P3/PPPP1PPP/RNBQK2R w KQ - 1 2".to_string()), new_position);

    let original_position = &get_position(&"r3k2r/pppppppp/2n1b3/2bn1q2/8/4P3/PPPP1PPP/RNBQK2R b KQq - 0 1".to_string());
    let mut new_position = *original_position;
    make_move(original_position, move_from_algebraic_move("h8g8".to_string(), PIECE_MASK_ROOK), &mut new_position);
    assert_eq!(get_position(&"r3k1r1/pppppppp/2n1b3/2bn1q2/8/4P3/PPPP1PPP/RNBQK2R w KQq - 1 2".to_string()), new_position);

    let original_position = &get_position(&"2kr3r/pppppp1p/2n1b3/2bn1q2/4Pp2/8/PPPP1PPP/RNBQK2R b KQ e3 15 1".to_string());
    let mut new_position = *original_position;
    make_move(original_position, move_from_algebraic_move("f4e3".to_string(), PIECE_MASK_PAWN), &mut new_position);
    assert_eq!(get_position(&"2kr3r/pppppp1p/2n1b3/2bn1q2/8/4p3/PPPP1PPP/RNBQK2R w KQ - 0 2".to_string()), new_position);

    let original_position = &get_position(&"2kr3r/ppppppPp/2n1b3/2bn1q2/8/4p3/PPPP1P1P/RNBQK2R w KQ - 12 1".to_string());
    let mut new_position = *original_position;
    make_move(original_position, move_from_algebraic_move("g7h8r".to_string(), PIECE_MASK_PAWN), &mut new_position);
    assert_eq!(get_position(&"2kr3R/pppppp1p/2n1b3/2bn1q2/8/4p3/PPPP1P1P/RNBQK2R b KQ - 0 1".to_string()), new_position);

    let original_position = &get_position(&"2kr3R/pppp1p1p/2n1b3/2bn1q2/8/4p3/PPPP1PpP/RNBQK2R b KQ - 0 1".to_string());
    let mut new_position = *original_position;
    make_move(original_position, move_from_algebraic_move("g2g1q".to_string(), PIECE_MASK_PAWN), &mut new_position);
    assert_eq!(get_position(&"2kr3R/pppp1p1p/2n1b3/2bn1q2/8/4p3/PPPP1P1P/RNBQK1qR w KQ - 0 2".to_string()), new_position);

    let original_position = &get_position(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string());
    let mut new_position = *original_position;
    make_move(original_position, move_from_algebraic_move("e2e4".to_string(), PIECE_MASK_PAWN), &mut new_position);
    assert_eq!(get_position(&"rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1".to_string()), new_position);

    let original_position = &get_position(&"8/p7/8/1P6/K1k3pP/6P1/8/8 b - h3 0 1".to_string());
    let mut new_position = *original_position;
    make_move(original_position, move_from_algebraic_move("a7a6".to_string(), PIECE_MASK_PAWN), &mut new_position);
    assert_eq!(get_position(&"8/8/p7/1P6/K1k3pP/6P1/8/8 w - - 0 2".to_string()), new_position);

    let original_position = &get_position(&"8/p7/8/1P6/K1k3pP/6P1/8/8 b - h3 0 1".to_string());
    let mut new_position = *original_position;
    make_move(original_position, move_from_algebraic_move("a7a5".to_string(), PIECE_MASK_PAWN), &mut new_position);
    assert_eq!(get_position(&"8/8/8/pP6/K1k3pP/6P1/8/8 w - a6 0 2".to_string()), new_position);

    let original_position = &get_position(&"8/p7/8/1P6/K1k3pP/6P1/8/8 b - h3 0 1".to_string());
    let mut new_position = *original_position;
    make_move(original_position, move_from_algebraic_move("c4c5".to_string(), PIECE_MASK_KING), &mut new_position);
    assert_eq!(get_position(&"8/p7/8/1Pk5/K5pP/6P1/8/8 w - - 1 2".to_string()), new_position);
}
