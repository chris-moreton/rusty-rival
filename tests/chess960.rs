//! Chess960 (NET-1762): castling from any start position, the FEN spellings
//! that carry it, and the `UCI_Chess960` king-takes-rook move notation.
//!
//! Every perft figure here was produced by Stockfish 17 with `UCI_Chess960`
//! on (`/tmp/stockfish/stockfish-ubuntu-x86-64-avx2`, 2026-10-17), not by
//! this engine, so the suite is an independent check of move generation,
//! legality, make and unmake together.

use either::Right;
use rusty_rival::bitboards::{A1_BIT, A8_BIT, B1_BIT, B8_BIT, F1_BIT, F8_BIT, G1_BIT, G8_BIT, H1_BIT, H8_BIT};
use rusty_rival::evaluate::evaluate_position;
use rusty_rival::fen::{algebraic_move_from_move, get_fen, get_position, uci_move_from_move};
use rusty_rival::hash::{castle_rook_key, pawn_key_from_scratch, zobrist_lock, ZOBRIST_KEYS_CASTLE_ROOK};
use rusty_rival::make_move::{make_move, make_move_in_place, unmake_move};
use rusty_rival::move_constants::{
    ALL_CASTLE_MOVE_MASKS, BK_CASTLE, BQ_CASTLE, CASTLE_INDEX_BLACK_KING, CASTLE_INDEX_BLACK_QUEEN, CASTLE_INDEX_WHITE_KING,
    CASTLE_INDEX_WHITE_QUEEN, PIECE_MASK_KING, WHITE_KING_CASTLE_MOVE_MASK, WHITE_QUEEN_CASTLE_MOVE_MASK, WK_CASTLE, WQ_CASTLE,
};
use rusty_rival::moves::{generate_moves, is_check, verify_move};
use rusty_rival::nnue::Accumulator;
use rusty_rival::perft::perft;
use rusty_rival::search::{make_move_nnue, unmake_move_nnue};
use rusty_rival::types::{default_search_state, default_uci_state, Move, Position, SearchState};
use rusty_rival::uci::{is_legal_move, run_command_test};
use rusty_rival::utils::{castle_mask, from_square_mask, hydrate_move_from_algebraic_move, invert_fen};

/// Twelve Chess960 positions and their Stockfish perft counts for depths 1-4.
const STOCKFISH_PERFT: [(&str, [u64; 4]); 12] = [
    (
        "bqnb1rkr/pp3ppp/3ppn2/2p5/5P2/P2P4/NPP1P1PP/BQ1BNRKR w HFhf - 2 9",
        [21, 528, 12189, 326672],
    ),
    (
        "2nnrbkr/p1qppppp/8/1ppb4/6PP/3PP3/PPP2P2/BQNNRBKR w HEhe - 1 9",
        [21, 807, 18002, 667366],
    ),
    (
        "b1q1rrkb/pppppppp/3nn3/8/P7/1PPP4/4PPPP/BQNNRKRB w GE - 1 9",
        [20, 479, 10471, 273318],
    ),
    (
        "qbbnnrkr/2pp2pp/p7/1p2pp2/8/P3PP2/1PPP1KPP/QBBNNR1R w hf - 0 9",
        [22, 593, 13440, 382958],
    ),
    (
        "1nbbnrkr/p1p1ppp1/3p4/1p3P1p/3Pq2P/8/PPP1P1P1/QNBBNRKR w HFhf - 0 9",
        [28, 1120, 31058, 1171749],
    ),
    (
        "qnbnr1kr/ppp2pp1/3p4/4p2p/8/P1N1PP2/1PPP2PP/QNB1RBKR w HEhe - 0 9",
        [27, 827, 22889, 696641],
    ),
    (
        "nrbbnkqr/p1pppppp/1p6/8/2P5/8/PP1PPPPP/NRBBNKQR w HBhb - 0 9",
        [23, 460, 11274, 249472],
    ),
    // Black's `f` names an empty square: no right, and none may appear
    // after the FEN round trip either (that was a real bug in development)
    (
        "rbbqk1rn/pp2pppp/2pp4/8/3PP3/5N2/PPP2PPP/RBBQKR1N w FAfa - 0 9",
        [30, 835, 25552, 728443],
    ),
    (
        "qrkbbnnr/pppp2pp/4pp2/8/3P4/2P5/PP2PPPP/QRKBBNNR w HBhb - 0 9",
        [25, 525, 13816, 319329],
    ),
    (
        "rk2r1b1/1pp1pppp/p7/3p4/8/1P1P1P2/PPP1P1PP/RKNNRBBQ w EAea - 0 9",
        [22, 425, 9586, 188721],
    ),
    (
        "1r1kbrq1/p1p1pp1p/1p1p2p1/8/P5n1/1PNP2P1/1BP1PP1P/1R1KBRQN w FBfb - 0 9",
        [23, 710, 17976, 554200],
    ),
    (
        "rk1nbrqb/pp2pppp/2p5/3p4/8/2N2P2/PPPPP1PP/RKN1BRQB w FAfa - 0 9",
        [30, 505, 15774, 301111],
    ),
];

/// Hand-built castling geometries with their Stockfish perft counts.
const STOCKFISH_PERFT_GEOMETRY: [(&str, [u64; 4]); 10] = [
    // King on g1 and rook on h1: the king does not move when castling
    ("6kr/8/8/8/8/8/8/6KR w Hh - 0 1", [11, 90, 1283, 18794]),
    // King on e1 and rook on f1: the rook does not move when castling
    ("4k3/8/8/8/8/8/8/4KR2 w F - 0 1", [14, 47, 781, 4432]),
    // King on b1 and rook on a1: the castle is a one-square king move
    ("1k6/8/8/8/8/8/8/RK6 w A - 0 1", [12, 39, 647, 3507]),
    // ...blocked because the rook's path to d1 is occupied
    ("1k6/8/8/8/8/8/8/RK1B4 w A - 0 1", [18, 55, 1135, 6001]),
    // ...and because the king would cross an attacked c1
    ("1kr5/8/8/8/8/8/8/RK6 w A - 0 1", [9, 116, 1773, 26098]),
    // The rook on b1 shields the king from the queen on a1 until it castles
    ("2k5/8/8/8/8/8/8/qRK5 w B - 0 1", [4, 59, 782, 16334]),
    // X-FEN and Shredder-FEN spell the same rights
    ("1rk3r1/8/8/8/8/8/8/1RK3R1 w GBgb - 0 1", [23, 442, 9940, 214105]),
    ("1rk3r1/8/8/8/8/8/8/1RK3R1 w KQkq - 0 1", [23, 442, 9940, 214105]),
    // Long king walk: b1 to g1 with the rook arriving from e1
    ("rk2r3/8/8/8/8/8/8/RK2R3 w EAea - 0 1", [23, 442, 9899, 211156]),
    // The standard start, as a Chess960 game spells it
    ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w HAha - 0 1", [20, 400, 8902, 197281]),
];

fn legal_moves(position: &Position) -> Vec<Move> {
    generate_moves(position)
        .into_iter()
        .filter(|&m| {
            let mut after = *position;
            make_move(position, m, &mut after);
            !is_check(&after, position.mover)
        })
        .collect()
}

fn spelled(position: &Position, chess960: bool) -> Vec<String> {
    let mut moves: Vec<String> = legal_moves(position)
        .iter()
        .map(|&m| uci_move_from_move(m, &position.castle_rook_squares, chess960))
        .collect();
    moves.sort();
    moves
}

fn castles(position: &Position) -> Vec<Move> {
    legal_moves(position)
        .into_iter()
        .filter(|m| m & ALL_CASTLE_MOVE_MASKS != 0)
        .collect()
}

#[test]
fn perft_matches_stockfish_on_chess960_positions() {
    for (fen, counts) in STOCKFISH_PERFT.iter().chain(STOCKFISH_PERFT_GEOMETRY.iter()) {
        for (depth, expected) in counts.iter().enumerate() {
            assert_eq!(
                perft(&mut get_position(fen), depth as u8),
                *expected,
                "{} at depth {}",
                fen,
                depth + 1
            );
        }
    }
}

#[test]
fn shredder_and_x_fen_castling_fields_parse_to_the_same_rooks() {
    let shredder = get_position("1rk3r1/8/8/8/8/8/8/1RK3R1 w GBgb - 0 1");
    let x_fen = get_position("1rk3r1/8/8/8/8/8/8/1RK3R1 w KQkq - 0 1");
    assert_eq!(shredder.castle_flags, WK_CASTLE | WQ_CASTLE | BK_CASTLE | BQ_CASTLE);
    assert_eq!(shredder.castle_rook_squares, [G1_BIT, B1_BIT, G8_BIT, B8_BIT]);
    assert_eq!(x_fen.castle_rook_squares, shredder.castle_rook_squares);
    assert_eq!(x_fen, shredder);
    assert_eq!(x_fen.zobrist_lock, shredder.zobrist_lock);
    // Written back in a form that reads identically
    assert_eq!(get_fen(&shredder), "1rk3r1/8/8/8/8/8/8/1RK3R1 w GBgb - 0 1");
    assert_eq!(get_fen(&x_fen), "1rk3r1/8/8/8/8/8/8/1RK3R1 w GBgb - 0 1");

    // X-FEN picks the OUTERMOST rook on each side of the king
    let two_rooks_a_side = get_position("k7/8/8/8/8/8/8/RR2K1RR w KQ - 0 1");
    assert_eq!(two_rooks_a_side.castle_rook_squares[CASTLE_INDEX_WHITE_KING], H1_BIT);
    assert_eq!(two_rooks_a_side.castle_rook_squares[CASTLE_INDEX_WHITE_QUEEN], A1_BIT);
    assert_eq!(get_fen(&two_rooks_a_side), "k7/8/8/8/8/8/8/RR2K1RR w KQ - 0 1");
    let inner_rooks = get_position("k7/8/8/8/8/8/8/RR2K1RR w GB - 0 1");
    assert_eq!(inner_rooks.castle_rook_squares[CASTLE_INDEX_WHITE_KING], G1_BIT);
    assert_eq!(inner_rooks.castle_rook_squares[CASTLE_INDEX_WHITE_QUEEN], B1_BIT);
    assert_eq!(get_fen(&inner_rooks), "k7/8/8/8/8/8/8/RR2K1RR w GB - 0 1");

    // The standard start spelled either way is the standard start
    let standard = get_position("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let shredder_standard = get_position("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w HAha - 0 1");
    assert_eq!(standard, shredder_standard);
    assert_eq!(standard.castle_rook_squares, [H1_BIT, A1_BIT, H8_BIT, A8_BIT]);
    assert_eq!(
        get_fen(&shredder_standard),
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
    );
}

#[test]
fn a_rook_file_that_names_no_rook_grants_no_right_and_round_trips() {
    let fen = "rbbqk1rn/pp2pppp/2pp4/8/3PP3/5N2/PPP2PPP/RBBQKR1N w FAfa - 0 9";
    let position = get_position(fen);
    assert_eq!(position.castle_flags, WK_CASTLE | WQ_CASTLE | BQ_CASTLE);
    assert_eq!(position.castle_rook_squares[CASTLE_INDEX_WHITE_KING], F1_BIT);
    assert_eq!(position.castle_rook_squares[CASTLE_INDEX_WHITE_QUEEN], A1_BIT);
    assert_eq!(position.castle_rook_squares[CASTLE_INDEX_BLACK_QUEEN], A8_BIT);
    // The corner rooks print as Q/q, the f1 rook as its file
    let written = get_fen(&position);
    assert_eq!(written, "rbbqk1rn/pp2pppp/2pp4/8/3PP3/5N2/PPP2PPP/RBBQKR1N w FQq - 0 9");
    let reread = get_position(&written);
    assert_eq!(reread, position);
    assert_eq!(reread.castle_rook_squares, position.castle_rook_squares);
    assert_eq!(reread.zobrist_lock, position.zobrist_lock);
    // Black has no castle in either reading
    assert!(castles(&get_position("rbbqk1rn/pp2pppp/2pp4/8/3PP3/5N2/PPP2PPP/RBBQKR1N b FAfa - 0 9")).is_empty());
    assert!(castles(&get_position("rbbqk1rn/pp2pppp/2pp4/8/3PP3/5N2/PPP2PPP/RBBQKR1N b FQq - 0 9")).is_empty());

    // Rights behind a king off its back rank, or on the wrong side of it
    let off_rank = get_position("4k3/8/8/8/8/4K3/8/R6R w AH - 0 1");
    assert_eq!(off_rank.castle_flags, 0);
    let wrong_side = get_position("4k3/8/8/8/8/8/8/R2K4 w A - 0 1");
    assert_eq!(wrong_side.castle_flags, WQ_CASTLE);
    let same_file_as_king = get_position("4k3/8/8/8/8/8/8/R2K4 w D - 0 1");
    assert_eq!(same_file_as_king.castle_flags, 0);

    // `K`/`Q` with nothing behind them are still recorded as written (NET-369)
    let unbacked = get_position("4k3/8/8/8/8/8/8/4K2R w KQ - 0 1");
    assert_eq!(unbacked.castle_flags, WK_CASTLE | WQ_CASTLE);
    assert_eq!(get_fen(&unbacked), "4k3/8/8/8/8/8/8/4K2R w KQ - 0 1");
    assert_eq!(spelled(&unbacked, false).iter().filter(|m| m.as_str() == "e1g1").count(), 1);
    assert!(!spelled(&unbacked, false).contains(&"e1c1".to_string()));
}

#[test]
fn castling_geometry_cases() {
    // King g1, rook h1: king stays, rook crosses to f1
    let position = get_position("6kr/8/8/8/8/8/8/6KR w Hh - 0 1");
    let castle = castles(&position);
    assert_eq!(castle.len(), 1);
    assert_eq!(
        castle[0],
        from_square_mask(G1_BIT) | G1_BIT as Move | PIECE_MASK_KING | WHITE_KING_CASTLE_MOVE_MASK
    );
    assert_eq!(uci_move_from_move(castle[0], &position.castle_rook_squares, true), "g1h1");
    assert_eq!(algebraic_move_from_move(castle[0]), "g1g1");
    let mut after = position;
    make_move(&position, castle[0], &mut after);
    // Black's h8 rook is on its corner, so its right prints as the standard `k`
    assert_eq!(get_fen(&after), "6kr/8/8/8/8/8/8/5RK1 b k - 1 1");

    // King e1, rook f1: rook stays, king crosses to g1
    let position = get_position("4k3/8/8/8/8/8/8/4KR2 w F - 0 1");
    let castle = castles(&position);
    assert_eq!(castle.len(), 1);
    assert_eq!(uci_move_from_move(castle[0], &position.castle_rook_squares, true), "e1f1");
    assert_eq!(algebraic_move_from_move(castle[0]), "e1g1");
    let mut after = position;
    make_move(&position, castle[0], &mut after);
    assert_eq!(get_fen(&after), "4k3/8/8/8/8/8/8/5RK1 b - - 1 1");

    // King b1, rook a1: a castle and a plain step both reach c1, distinctly
    let position = get_position("1k6/8/8/8/8/8/8/RK6 w A - 0 1");
    let castle = castles(&position);
    assert_eq!(castle.len(), 1);
    assert_eq!(
        castle[0],
        from_square_mask(B1_BIT) | 5 | PIECE_MASK_KING | WHITE_QUEEN_CASTLE_MOVE_MASK
    );
    let moves = spelled(&position, true);
    assert!(moves.contains(&"b1a1".to_string()), "{moves:?}");
    assert!(moves.contains(&"b1c1".to_string()), "{moves:?}");
    // With UCI_Chess960 off the one-square castle still prints king-takes-rook,
    // so it cannot be confused with the step to the same square
    let standard_spelling = spelled(&position, false);
    assert_eq!(standard_spelling.iter().filter(|m| m.as_str() == "b1c1").count(), 1);
    assert!(standard_spelling.contains(&"b1a1".to_string()), "{standard_spelling:?}");
    let mut after = position;
    make_move(&position, castle[0], &mut after);
    assert_eq!(get_fen(&after), "1k6/8/8/8/8/8/8/2KR4 b - - 1 1");
    let step = hydrate_move_from_algebraic_move(&position, "b1c1".to_string());
    assert_eq!(step & ALL_CASTLE_MOVE_MASKS, 0);
    let mut after = position;
    make_move(&position, step, &mut after);
    assert_eq!(get_fen(&after), "1k6/8/8/8/8/8/8/R1K5 b - - 1 1");

    // Blocked rook path, attacked king path
    assert!(castles(&get_position("1k6/8/8/8/8/8/8/RK1B4 w A - 0 1")).is_empty());
    assert!(castles(&get_position("1kr5/8/8/8/8/8/8/RK6 w A - 0 1")).is_empty());
    // A king in check cannot castle even when it would not move
    assert!(castles(&get_position("6kr/8/8/8/8/8/8/2r3KR w H - 0 1")).is_empty());

    // The b1 rook shields c1 from the a1 queen: generated, then found illegal
    let position = get_position("2k5/8/8/8/8/8/8/qRK5 w B - 0 1");
    assert!(generate_moves(&position).iter().any(|m| m & ALL_CASTLE_MOVE_MASKS != 0));
    assert!(castles(&position).is_empty());

    // Long walk b1 -> g1 with the rook arriving from e1
    let position = get_position("rk2r3/8/8/8/8/8/8/RK2R3 w EAea - 0 1");
    let mut spelled_castles: Vec<String> = castles(&position)
        .iter()
        .map(|&m| uci_move_from_move(m, &position.castle_rook_squares, true))
        .collect();
    spelled_castles.sort();
    // The e8 rook attacks e1, which the king would cross: only queen side
    assert_eq!(spelled_castles, vec!["b1a1"]);
    let queen_side = hydrate_move_from_algebraic_move(&position, "b1a1".to_string());
    let mut after = position;
    make_move(&position, queen_side, &mut after);
    assert_eq!(get_fen(&after), "rk2r3/8/8/8/8/8/8/2KRR3 b eq - 1 1");
    // Without that rook the walk is on: b1 to g1, e1 to f1
    let position = get_position("1k6/8/8/8/8/8/8/RK2R3 w EA - 0 1");
    let king_side = hydrate_move_from_algebraic_move(&position, "b1e1".to_string());
    assert!(castles(&position).contains(&king_side));
    let mut after = position;
    make_move(&position, king_side, &mut after);
    assert_eq!(get_fen(&after), "1k6/8/8/8/8/8/8/R4RK1 b - - 1 1");
}

#[test]
fn rights_end_when_the_castling_rook_moves_or_is_captured_on_its_square() {
    let position = get_position("1rk3r1/8/8/8/8/8/8/1RK3R1 w GBgb - 0 1");
    let rook_leaves = hydrate_move_from_algebraic_move(&position, "g1g4".to_string());
    let mut after = position;
    make_move(&position, rook_leaves, &mut after);
    assert_eq!(after.castle_flags, WQ_CASTLE | BK_CASTLE | BQ_CASTLE);
    assert_eq!(get_fen(&after), "1rk3r1/8/8/8/6R1/8/8/1RK5 b Bgb - 1 1");
    let captures_rook = hydrate_move_from_algebraic_move(&after, "g8g4".to_string());
    let mut captured = after;
    make_move(&after, captures_rook, &mut captured);
    assert_eq!(captured.castle_flags, WQ_CASTLE | BQ_CASTLE);
    // The corner, no longer a castling rook square, changes nothing
    let position = get_position("1rk3r1/8/8/8/8/8/8/1RK3R1 w GBgb - 0 1");
    let to_corner = hydrate_move_from_algebraic_move(&position, "g1h1".to_string());
    let mut after = position;
    make_move(&position, to_corner, &mut after);
    assert_eq!(after.castle_flags, WQ_CASTLE | BK_CASTLE | BQ_CASTLE);
}

#[test]
fn make_unmake_and_keys_hold_through_chess960_castles() {
    fn walk(position: &mut Position, depth: u8, castles_seen: &mut u32) {
        let before = *position;
        for m in generate_moves(position) {
            let mut copied = before;
            make_move(&before, m, &mut copied);
            let unmake = make_move_in_place(position, m);
            assert_eq!(
                *position,
                copied,
                "copy and in-place make disagree after {m:#x} from {}",
                get_fen(&before)
            );
            assert_eq!(position.zobrist_lock, copied.zobrist_lock);
            assert_eq!(
                position.zobrist_lock,
                zobrist_lock(position),
                "incremental key drifted after {m:#x}"
            );
            assert_eq!(position.pawn_key, pawn_key_from_scratch(position));
            assert_eq!(position.castle_rook_squares, before.castle_rook_squares);
            // The written position reads back as itself
            let reread = get_position(&get_fen(position));
            assert_eq!(reread, *position, "FEN round trip after {m:#x}: {}", get_fen(position));
            assert_eq!(reread.zobrist_lock, position.zobrist_lock);
            if m & ALL_CASTLE_MOVE_MASKS != 0 {
                *castles_seen += 1;
            }
            if depth > 0 && !is_check(position, before.mover) {
                walk(position, depth - 1, castles_seen);
            }
            unmake_move(position, m, &unmake);
            assert_eq!(*position, before, "unmake of {m:#x} did not restore {}", get_fen(&before));
            assert_eq!(position.zobrist_lock, before.zobrist_lock);
            assert_eq!(position.pawn_key, before.pawn_key);
            assert_eq!(position.pieces[0].all_pieces_bitboard, before.pieces[0].all_pieces_bitboard);
            assert_eq!(position.pieces[1].all_pieces_bitboard, before.pieces[1].all_pieces_bitboard);
            assert_eq!(position.pieces[0].rook_bitboard, before.pieces[0].rook_bitboard);
            assert_eq!(position.pieces[1].rook_bitboard, before.pieces[1].rook_bitboard);
        }
    }
    let mut castles_seen = 0;
    for (fen, _) in STOCKFISH_PERFT_GEOMETRY.iter().chain(STOCKFISH_PERFT.iter().take(4)) {
        walk(&mut get_position(fen), 1, &mut castles_seen);
    }
    assert!(castles_seen >= 20, "only {castles_seen} castles exercised");
}

#[test]
fn incremental_nnue_matches_full_recompute_through_chess960_castles() {
    fn piece_count(position: &Position) -> u32 {
        (position.pieces[0].all_pieces_bitboard | position.pieces[1].all_pieces_bitboard).count_ones()
    }
    fn dfs(position: &mut Position, ss: &mut SearchState, depth: u8, checked: &mut u32, castled: &mut u32, under_castle: bool) {
        if !position.zobrist_lock.is_multiple_of(3) || under_castle {
            let incremental = evaluate_position(position, ss);
            let net = ss.nnue_network.clone().unwrap();
            let mut acc = Accumulator::new();
            acc.compute(&net, position);
            let full = net.evaluate(&acc, position.mover, piece_count(position));
            assert_eq!(
                incremental,
                full,
                "incremental {incremental} != full {full} at {}",
                get_fen(position)
            );
            *checked += 1;
            if under_castle {
                *castled += 1;
            }
        }
        if depth == 0 {
            return;
        }
        for m in generate_moves(position) {
            let old_mover = position.mover;
            let unmake = make_move_nnue(position, m, ss);
            if !is_check(position, old_mover) {
                dfs(position, ss, depth - 1, checked, castled, m & ALL_CASTLE_MOVE_MASKS != 0);
            }
            unmake_move_nnue(position, m, &unmake, ss);
        }
    }
    let mut checked = 0;
    let mut castled = 0;
    for (fen, _) in STOCKFISH_PERFT_GEOMETRY.iter().chain(STOCKFISH_PERFT.iter().take(3)) {
        let mut position = get_position(fen);
        let mut ss = default_search_state();
        let net = ss.nnue_network.clone().unwrap();
        ss.nnue_ply = 0;
        ss.nnue_accumulators[0].compute(&net, &position);
        ss.nnue_pieces[0] = position.pieces;
        ss.nnue_computed[0] = true;
        dfs(&mut position, &mut ss, 2, &mut checked, &mut castled, false);
    }
    assert!(checked > 1000 && castled >= 20, "checked {checked}, after castles {castled}");
}

#[test]
fn verify_move_tells_a_one_square_castle_from_a_king_step() {
    let position = get_position("1k6/8/8/8/8/8/8/RK6 w A - 0 1");
    let castle = from_square_mask(B1_BIT) | 5 | PIECE_MASK_KING | WHITE_QUEEN_CASTLE_MOVE_MASK;
    let step = from_square_mask(B1_BIT) | 5 | PIECE_MASK_KING;
    assert!(verify_move(&position, castle));
    assert!(verify_move(&position, step));
    // Rights gone: the flagged move is nothing, the step still a move
    let no_rights = get_position("1k6/8/8/8/8/8/8/RK6 w - - 0 1");
    assert!(!verify_move(&no_rights, castle));
    assert!(verify_move(&no_rights, step));
    // Rook gone from its square, flag still asserted: nothing to castle with
    let no_rook = get_position("1k6/8/8/8/8/8/8/1K6 w Q - 0 1");
    assert!(!verify_move(&no_rook, castle));
    // King g1 castling in place: the flagged g1->g1 move verifies, a g1->g1 step never does
    let in_place = get_position("6kr/8/8/8/8/8/8/6KR w Hh - 0 1");
    let castle = from_square_mask(G1_BIT) | G1_BIT as Move | PIECE_MASK_KING | WHITE_KING_CASTLE_MOVE_MASK;
    assert!(verify_move(&in_place, castle));
    assert!(!verify_move(&in_place, from_square_mask(G1_BIT) | G1_BIT as Move | PIECE_MASK_KING));
}

#[test]
fn both_castle_spellings_hydrate_to_the_generated_move() {
    let position = get_position("1rk3r1/8/8/8/8/8/8/1RK3R1 w GBgb - 0 1");
    let king_side = from_square_mask(5) | G1_BIT as Move | PIECE_MASK_KING | WHITE_KING_CASTLE_MOVE_MASK;
    let queen_side = from_square_mask(5) | 5 | PIECE_MASK_KING | WHITE_QUEEN_CASTLE_MOVE_MASK;
    // The king-side castle is generated but illegal here: once the g1 rook
    // steps to f1 the g8 rook sees the king on g1. The queen-side one is legal.
    assert!(generate_moves(&position).contains(&king_side));
    assert!(!legal_moves(&position).contains(&king_side));
    assert!(legal_moves(&position).contains(&queen_side));
    // King takes rook (UCI_Chess960)
    assert_eq!(hydrate_move_from_algebraic_move(&position, "c1g1".to_string()), king_side);
    assert_eq!(hydrate_move_from_algebraic_move(&position, "c1b1".to_string()), queen_side);
    assert_eq!(
        castle_mask(&position, rusty_rival::fen::move_from_algebraic_move("c1b1".to_string(), 0)),
        WHITE_QUEEN_CASTLE_MOVE_MASK
    );
    // A one-square king step is never a castle
    assert_eq!(
        hydrate_move_from_algebraic_move(&position, "c1d1".to_string()) & ALL_CASTLE_MOVE_MASKS,
        0
    );
    // Standard double-step spelling where it is unambiguous
    let standard = get_position("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1");
    assert_eq!(
        hydrate_move_from_algebraic_move(&standard, "e1g1".to_string()),
        hydrate_move_from_algebraic_move(&standard, "e1h1".to_string())
    );
    assert_eq!(
        hydrate_move_from_algebraic_move(&standard, "e1c1".to_string()),
        hydrate_move_from_algebraic_move(&standard, "e1a1".to_string())
    );
    assert!(is_legal_move(&standard, "e1g1"));
    assert!(is_legal_move(&standard, "e1h1"));
    assert!(is_legal_move(&standard, "e1c1"));
    assert!(is_legal_move(&standard, "e1a1"));
    assert!(!is_legal_move(&standard, "e1b1"));
    // Without the right, e1h1 is a king move onto its own rook: illegal
    let no_rights = get_position("r3k2r/8/8/8/8/8/8/R3K2R w kq - 0 1");
    assert!(!is_legal_move(&no_rights, "e1h1"));
    assert!(!is_legal_move(&no_rights, "e1g1"));
    // A `K` right kept with no rook behind it (NET-369) leaves the rook square
    // at h1; the king capturing an enemy piece there is a capture, not a castle
    // (CodeRabbit on PR #116)
    let phantom_right = get_position("6k1/8/8/8/8/8/8/6Kn w K - 0 1");
    assert_eq!(
        hydrate_move_from_algebraic_move(&phantom_right, "g1h1".to_string()) & ALL_CASTLE_MOVE_MASKS,
        0
    );
    assert!(is_legal_move(&phantom_right, "g1h1"));
    let mut after = phantom_right;
    make_move(
        &phantom_right,
        hydrate_move_from_algebraic_move(&phantom_right, "g1h1".to_string()),
        &mut after,
    );
    assert_eq!(get_fen(&after), "6k1/8/8/8/8/8/8/7K b - - 0 1");
    let mut uci_state = default_uci_state();
    let mut search_state = default_search_state();
    assert_eq!(
        run_command_test(
            &mut uci_state,
            &mut search_state,
            "position fen 6k1/8/8/8/8/8/8/6Kn w K - 0 1 moves g1h1"
        ),
        Right(None)
    );
    assert_eq!(uci_state.fen, "6k1/8/8/8/8/8/8/7K b - - 0 1");
}

#[test]
fn uci_chess960_option_and_notation() {
    let mut uci_state = default_uci_state();
    let mut search_state = default_search_state();

    let uci = run_command_test(&mut uci_state, &mut search_state, "uci");
    assert!(matches!(&uci, Right(Some(text)) if text.contains("option name UCI_Chess960 type check default false")));

    // Shredder-FEN start plus king-takes-rook castles, as lichess-bot sends them
    assert_eq!(
        run_command_test(&mut uci_state, &mut search_state, "setoption name UCI_Chess960 value true"),
        Right(None)
    );
    assert_eq!(
        run_command_test(
            &mut uci_state,
            &mut search_state,
            "position fen 1rk3r1/pppppppp/8/8/8/8/PPPPPPPP/1RK3R1 w GBgb - 0 1 moves c1g1 c8b8"
        ),
        Right(None)
    );
    // Black's c8b8 is the queen-side castle: the king stays on c8, the rook goes to d8
    assert_eq!(uci_state.fen, "2kr2r1/pppppppp/8/8/8/8/PPPPPPPP/1R3RK1 w - - 2 2");

    // In-place castle: the king stays on g1 and the move still reads g1h1
    assert_eq!(
        run_command_test(
            &mut uci_state,
            &mut search_state,
            "position fen 6kr/8/8/8/8/8/8/6KR w Hh - 0 1 moves g1h1"
        ),
        Right(None)
    );
    assert_eq!(uci_state.fen, "6kr/8/8/8/8/8/8/5RK1 b k - 1 1");

    // bestmove and the PV spell a castle as king-takes-rook when the option is on
    assert_eq!(
        run_command_test(&mut uci_state, &mut search_state, "position fen 6kr/8/8/8/8/8/8/6KR w Hh - 0 1"),
        Right(None)
    );
    assert_eq!(
        run_command_test(&mut uci_state, &mut search_state, "go depth 1 searchmoves g1h1"),
        Right(Some("bestmove g1h1".to_string()))
    );
    assert_eq!(
        run_command_test(&mut uci_state, &mut search_state, "setoption name UCI_Chess960 value false"),
        Right(None)
    );
    assert_eq!(
        run_command_test(
            &mut uci_state,
            &mut search_state,
            "position fen r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1"
        ),
        Right(None)
    );
    assert_eq!(
        run_command_test(&mut uci_state, &mut search_state, "go depth 1 searchmoves e1g1"),
        Right(Some("bestmove e1g1".to_string()))
    );
    assert_eq!(
        run_command_test(&mut uci_state, &mut search_state, "setoption name UCI_Chess960 value true"),
        Right(None)
    );
    assert_eq!(
        run_command_test(&mut uci_state, &mut search_state, "go depth 1 searchmoves e1g1"),
        Right(Some("bestmove e1h1".to_string()))
    );

    // A Chess960 start searches and plays a legal move
    assert_eq!(
        run_command_test(
            &mut uci_state,
            &mut search_state,
            "position fen bqnb1rkr/pp3ppp/3ppn2/2p5/5P2/P2P4/NPP1P1PP/BQ1BNRKR w HFhf - 2 9"
        ),
        Right(None)
    );
    let position = get_position("bqnb1rkr/pp3ppp/3ppn2/2p5/5P2/P2P4/NPP1P1PP/BQ1BNRKR w HFhf - 2 9");
    match run_command_test(&mut uci_state, &mut search_state, "go depth 6") {
        Right(Some(text)) => {
            let played = text.split_whitespace().nth(1).unwrap();
            assert!(spelled(&position, true).contains(&played.to_string()), "{text}");
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(run_command_test(&mut uci_state, &mut search_state, "go perft 3"), Right(None));
}

#[test]
fn inverting_a_fen_swaps_the_case_of_rook_files_too() {
    assert_eq!(
        invert_fen("1rk3r1/8/8/8/8/8/8/1RK3R1 w GBgb - 0 1"),
        "1rk3r1/8/8/8/8/8/8/1RK3R1 b gbGB - 0 1"
    );
    assert_eq!(
        invert_fen("rbbqk1rn/pp2pppp/2pp4/8/3PP3/5N2/PPP2PPP/RBBQKR1N w FQq - 0 9"),
        "rbbqkr1n/ppp2ppp/5n2/3pp3/8/2PP4/PP2PPPP/RBBQK1RN b fqQ - 0 9"
    );
    assert_eq!(
        invert_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"),
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b kqKQ - 0 1"
    );
}

#[test]
fn castle_indices_line_up_with_the_right_masks() {
    // The parser, the generator and make_move all key off these four indices
    assert_eq!(
        [
            CASTLE_INDEX_WHITE_KING,
            CASTLE_INDEX_WHITE_QUEEN,
            CASTLE_INDEX_BLACK_KING,
            CASTLE_INDEX_BLACK_QUEEN
        ],
        [0, 1, 2, 3]
    );
    assert_eq!(
        rusty_rival::move_constants::CASTLE_RIGHT_MASKS,
        [WK_CASTLE, WQ_CASTLE, BK_CASTLE, BQ_CASTLE]
    );
    assert_eq!(
        rusty_rival::move_constants::CASTLE_ROOK_START_DEFAULT,
        [H1_BIT, A1_BIT, H8_BIT, A8_BIT]
    );
    assert_eq!(rusty_rival::move_constants::CASTLE_VARS_ROOK_TO, [F1_BIT, 4, F8_BIT, 60]);
}

/// CodeRabbit on PR #116: `KQ` (outer rooks) and `GB` (inner rooks) give the
/// same board, mover and rights bits but different castles, so they must not
/// share a transposition-table lock. Standard-corner rooks contribute nothing,
/// so every orthodox key is the pre-Chess960 key.
#[test]
fn castling_rook_identity_is_part_of_the_lock() {
    let outer = get_position("k7/8/8/8/8/8/8/RR2K1RR w KQ - 0 1");
    let inner = get_position("k7/8/8/8/8/8/8/RR2K1RR w GB - 0 1");
    assert_eq!(outer.pieces, inner.pieces);
    assert_eq!(outer.castle_flags, inner.castle_flags);
    assert_ne!(outer.castle_rook_squares, inner.castle_rook_squares);
    assert_ne!(outer.zobrist_lock, inner.zobrist_lock);
    assert_eq!(inner.zobrist_lock, zobrist_lock(&inner));

    // Orthodox corners: zero salt, so standard keys are unchanged
    for index in 0..4 {
        assert_eq!(
            ZOBRIST_KEYS_CASTLE_ROOK[index][rusty_rival::move_constants::CASTLE_ROOK_START_DEFAULT[index] as usize],
            0
        );
    }
    assert_eq!(castle_rook_key(&outer.castle_rook_squares, outer.castle_flags), 0);
    let start = get_position("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    // Orthodox locks pinned to the pre-salt definition: values computed
    // independently (Python, from the unchanged hash.rs tables) as the XOR of
    // piece, castle-rights, en-passant and mover keys, as in v1.0.78
    for (fen, lock) in [
        (
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            298807166715181971774724577633070736683u128,
        ),
        (
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            287878515169241678237901907072231950823u128,
        ),
        ("r3k2r/8/8/8/8/8/8/R3K2R b Kq - 0 1", 245163171819780492984834757368791268100u128),
    ] {
        assert_eq!(get_position(fen).zobrist_lock, lock, "{fen}");
        assert_eq!(zobrist_lock(&get_position(fen)), lock, "{fen}");
    }
    assert_eq!(castle_rook_key(&start.castle_rook_squares, start.castle_flags), 0);
    assert_eq!(
        get_position("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w HAha - 0 1").zobrist_lock,
        start.zobrist_lock
    );

    // Inactive metadata is ignored: with no rights, the rook squares do not matter
    let no_rights_inner = get_position("k7/8/8/8/8/8/8/RR2K1RR w GB - 0 1");
    let mut stripped = no_rights_inner;
    stripped.castle_flags = 0;
    let mut stripped_outer = outer;
    stripped_outer.castle_flags = 0;
    assert_eq!(zobrist_lock(&stripped), zobrist_lock(&stripped_outer));
    assert_eq!(castle_rook_key(&inner.castle_rook_squares, 0), 0);

    // Incremental keys track the salt through rook moves, rook captures,
    // castles and king moves, on both make paths, and unmake restores them
    for fen in [
        "k7/8/8/8/8/8/8/RR2K1RR w GB - 0 1",
        "k6r/6r1/8/8/8/8/8/RR2K1RR w GB - 0 1",
        "1rk3r1/8/8/8/8/8/8/1RK3R1 w GBgb - 0 1",
        "1rk3r1/8/8/8/8/8/8/1RK3R1 b GBgb - 0 1",
        "rk2r3/8/8/8/8/8/8/RK2R3 w EAea - 0 1",
    ] {
        let mut position = get_position(fen);
        let before = position;
        let mut salted_moves = 0;
        for m in generate_moves(&before) {
            let mut copied = before;
            make_move(&before, m, &mut copied);
            let unmake = make_move_in_place(&mut position, m);
            assert_eq!(position.zobrist_lock, zobrist_lock(&position), "{fen} after {m:#x}");
            assert_eq!(copied.zobrist_lock, position.zobrist_lock, "{fen} copy vs in-place after {m:#x}");
            if position.castle_flags != before.castle_flags {
                salted_moves += 1;
            }
            unmake_move(&mut position, m, &unmake);
            assert_eq!(position.zobrist_lock, before.zobrist_lock);
        }
        assert!(salted_moves > 0, "{fen} exercised no rights change");
    }

    // The same through `position ... moves`, which builds keys by copy-make
    let mut uci_state = default_uci_state();
    let mut search_state = default_search_state();
    assert_eq!(
        run_command_test(
            &mut uci_state,
            &mut search_state,
            "position fen k7/8/8/8/8/8/8/RR2K1RR w GB - 0 1 moves g1g8 a8a7 b1b8"
        ),
        Right(None)
    );
    assert_eq!(*search_state.history.last().unwrap(), zobrist_lock(&get_position(&uci_state.fen)));
}

/// Short-distance castles round-trip in both notation modes: they are written
/// king-takes-rook either way and read back as the same castle (CodeRabbit on
/// PR #116); orthodox castles keep their `e1g1` spelling with the option off.
#[test]
fn short_castles_round_trip_in_both_modes() {
    for (fen, castle_text) in [
        ("1k6/8/8/8/8/8/8/RK6 w A - 0 1", "b1a1"),
        ("6kr/8/8/8/8/8/8/6KR w Hh - 0 1", "g1h1"),
        ("4k3/8/8/8/8/8/8/4KR2 w F - 0 1", "e1f1"),
    ] {
        let position = get_position(fen);
        let castle = castles(&position);
        assert_eq!(castle.len(), 1, "{fen}");
        for chess960 in [false, true] {
            let text = uci_move_from_move(castle[0], &position.castle_rook_squares, chess960);
            let expected = if fen.starts_with("4k3") && !chess960 { "e1g1" } else { castle_text };
            assert_eq!(text, expected, "{fen} chess960={chess960}");
            assert_eq!(hydrate_move_from_algebraic_move(&position, text.clone()), castle[0], "{fen} {text}");
        }
        for chess960 in [false, true] {
            let mut uci_state = default_uci_state();
            let mut search_state = default_search_state();
            let option = format!("setoption name UCI_Chess960 value {chess960}");
            assert_eq!(run_command_test(&mut uci_state, &mut search_state, &option), Right(None));
            assert_eq!(
                run_command_test(&mut uci_state, &mut search_state, &format!("position fen {fen}")),
                Right(None)
            );
            let text = uci_move_from_move(castle[0], &position.castle_rook_squares, chess960);
            assert_eq!(
                run_command_test(&mut uci_state, &mut search_state, &format!("go depth 1 searchmoves {text}")),
                Right(Some(format!("bestmove {text}")))
            );
            let mut after = position;
            make_move(&position, castle[0], &mut after);
            assert_eq!(
                run_command_test(&mut uci_state, &mut search_state, &format!("position fen {fen} moves {text}")),
                Right(None)
            );
            assert_eq!(uci_state.fen, get_fen(&after), "{fen} {text} chess960={chess960}");
        }
    }
    // Orthodox output unchanged with the option off
    let standard = get_position("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1");
    let mut spelled_castles: Vec<String> = castles(&standard)
        .iter()
        .map(|&m| uci_move_from_move(m, &standard.castle_rook_squares, false))
        .collect();
    spelled_castles.sort();
    assert_eq!(spelled_castles, vec!["e1c1", "e1g1"]);
}
