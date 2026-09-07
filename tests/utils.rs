use rusty_rival::bitboards::{C8_BIT, F8_BIT, G8_BIT, H2_BIT, H4_BIT, H8_BIT};
use rusty_rival::engine_constants::{KNIGHT_VALUE_AVERAGE, PAWN_VALUE_AVERAGE, QUEEN_VALUE_AVERAGE, ROOK_VALUE_AVERAGE};
use rusty_rival::fen::{get_position, move_from_algebraic_move};
use rusty_rival::move_constants::{
    BLACK_KING_CASTLE_MOVE_MASK, BLACK_QUEEN_CASTLE_MOVE_MASK, PIECE_MASK_KING, PIECE_MASK_KNIGHT, PIECE_MASK_PAWN, PIECE_MASK_ROOK,
    START_POS, WHITE_KING_CASTLE_MOVE, WHITE_KING_CASTLE_MOVE_MASK, WHITE_QUEEN_CASTLE_MOVE_MASK,
};
use rusty_rival::search::{MATE_SCORE, MATE_START};
use rusty_rival::types::{BoundType, HashEntry, HashLock, Move, SharedHashTable};
use rusty_rival::utils::{
    captured_piece_value, castle_mask, format_uci_score, from_square_mask, from_square_part, hydrate_move_from_algebraic_move, invert_fen,
    moving_piece_mask, to_square_part,
};

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
    let position = &get_position(START_POS);
    assert_eq!(
        moving_piece_mask(position, move_from_algebraic_move("e2e3".to_string(), 0)),
        PIECE_MASK_PAWN
    );
    assert_eq!(
        moving_piece_mask(position, move_from_algebraic_move("g1f3".to_string(), 0)),
        PIECE_MASK_KNIGHT
    );
}

#[test]
pub fn it_can_figure_out_the_captured_piece() {
    let position = &get_position("r3k3/pppp1ppp/1bnb1n2/4p1q1/3PP3/1BNB1Nr1/PPP1QPPP/R3K2R w KQq - 0 1");
    assert_eq!(
        captured_piece_value(position, hydrate_move_from_algebraic_move(position, "f3g5".to_string())),
        QUEEN_VALUE_AVERAGE
    );
    assert_eq!(
        captured_piece_value(position, hydrate_move_from_algebraic_move(position, "f3e5".to_string())),
        PAWN_VALUE_AVERAGE
    );
    assert_eq!(
        captured_piece_value(position, hydrate_move_from_algebraic_move(position, "f2g3".to_string())),
        ROOK_VALUE_AVERAGE
    );
    assert_eq!(
        captured_piece_value(position, hydrate_move_from_algebraic_move(position, "h2h3".to_string())),
        0
    );

    let position = &get_position("r3k2r/pppp1p1p/1bnb1n2/4p1q1/3PP1pP/1BNB1N2/PPP1QPP1/R3K2R b KQq h3 0 1");
    assert_eq!(
        captured_piece_value(position, hydrate_move_from_algebraic_move(position, "g4h3".to_string())),
        PAWN_VALUE_AVERAGE
    );

    let position = &get_position("r3k2r/p1pp1p1p/1bnb1n2/4p1q1/3PP1pP/1BNB1N2/PpP1QPP1/R3K2R b KQq - 0 1");
    assert_eq!(
        captured_piece_value(position, hydrate_move_from_algebraic_move(position, "b2a1n".to_string())),
        KNIGHT_VALUE_AVERAGE - PAWN_VALUE_AVERAGE + ROOK_VALUE_AVERAGE
    );
}

#[test]
pub fn it_can_figure_out_the_castle_mask() {
    let position = &get_position("r3k2r/pppp1ppp/1bnb1n2/4p1q1/3PP3/1BNB1N2/PPP1QPPP/R3K2R w KQkq - 0 1");
    assert_eq!(
        castle_mask(position, move_from_algebraic_move("e1g1".to_string(), 0)),
        WHITE_KING_CASTLE_MOVE_MASK
    );
    assert_eq!(
        castle_mask(position, move_from_algebraic_move("e1c1".to_string(), 0)),
        WHITE_QUEEN_CASTLE_MOVE_MASK
    );
    assert_eq!(
        castle_mask(position, move_from_algebraic_move("e8g8".to_string(), 0)),
        BLACK_KING_CASTLE_MOVE_MASK
    );
    assert_eq!(
        castle_mask(position, move_from_algebraic_move("e8c8".to_string(), 0)),
        BLACK_QUEEN_CASTLE_MOVE_MASK
    );
    assert_eq!(castle_mask(position, move_from_algebraic_move("e1d1".to_string(), 0)), 0);
}

#[test]
pub fn it_can_hydrate_a_move() {
    let position = &get_position("r3k2r/pppp1ppp/1bnb1n2/4p1q1/3PP3/1BNB1N2/PPP1QPPP/R3K2R w KQkq - 0 1");
    assert_eq!(
        hydrate_move_from_algebraic_move(position, "e1g1".to_string()),
        WHITE_KING_CASTLE_MOVE
    );
    assert_eq!(
        hydrate_move_from_algebraic_move(position, "h2h4".to_string()),
        from_square_mask(H2_BIT) | H4_BIT as Move | PIECE_MASK_PAWN
    );

    let position = &get_position("5rk1/5p1p/6p1/8/4Q3/2q5/4RPPP/6K1 b - - 0 1");
    assert_eq!(
        hydrate_move_from_algebraic_move(position, "f8c8".to_string()),
        from_square_mask(F8_BIT) | C8_BIT as Move | PIECE_MASK_ROOK
    );

    let position = &get_position("7k/pB3p1p/2p4b/3p2p1/3P3P/P1PNB3/1P3PP1/6K1 b - - 0 1");
    assert_eq!(
        hydrate_move_from_algebraic_move(position, "h8g8".to_string()),
        from_square_mask(H8_BIT) | G8_BIT as Move | PIECE_MASK_KING
    );
}

#[test]
fn it_inverts_a_fen() {
    assert_eq!(
        invert_fen("6k1/1P2P3/7p/P1pP4/5R2/5B2/1r2N2P/R1Q1K3 w Q - 0 1"),
        "r1q1k3/1R2n2p/5b2/5r2/p1Pp4/7P/1p2p3/6K1 b q - 0 1"
    );
}

#[test]
fn it_formats_uci_scores_as_cp_or_mate_in_moves() {
    assert_eq!(format_uci_score(15), "cp 15");
    assert_eq!(format_uci_score(-320), "cp -320");
    assert_eq!(format_uci_score(0), "cp 0");
    // Boundary: MATE_START itself is still a centipawn score
    assert_eq!(format_uci_score(MATE_START), format!("cp {}", MATE_START));
    assert_eq!(format_uci_score(-MATE_START), format!("cp {}", -MATE_START));
    // We mate: found at ply 1 = mate in 1 move, ply 3 = mate in 2, ply 5 = mate in 3
    assert_eq!(format_uci_score(MATE_SCORE - 1), "mate 1");
    assert_eq!(format_uci_score(MATE_SCORE - 3), "mate 2");
    assert_eq!(format_uci_score(MATE_SCORE - 5), "mate 3");
    // We are mated: at ply 2 (our move, their mate) = mate -1, ply 4 = mate -2
    assert_eq!(format_uci_score(-(MATE_SCORE - 2)), "mate -1");
    assert_eq!(format_uci_score(-(MATE_SCORE - 4)), "mate -2");
}

#[test]
fn hashfull_counts_only_entries_of_the_current_generation() {
    let table = SharedHashTable::new_with_entries(4096);
    assert_eq!(table.hashfull(), 0);
    table.bump_version();
    let version = table.version();
    for index in 0..250 {
        table.store(
            index,
            HashEntry {
                score: 0,
                version,
                height: 1,
                mv: 0,
                bound: BoundType::Exact,
                lock: index as HashLock + 1,
                static_eval: 0,
            },
        );
    }
    // 250 of the 1,000 sampled slots hold a current-generation entry
    assert_eq!(table.hashfull(), 250);
    // A new search generation starts from an empty count even though the
    // entries are still physically there
    table.bump_version();
    assert_eq!(table.hashfull(), 0);
}
