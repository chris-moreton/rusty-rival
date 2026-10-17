use crate::bitboards::{bit, BLACK_PASSED_PAWN_MASK, WHITE_PASSED_PAWN_MASK};
use crate::engine_constants::{BISHOP_VALUE_AVERAGE, KNIGHT_VALUE_AVERAGE, PAWN_VALUE_AVERAGE, QUEEN_VALUE_AVERAGE, ROOK_VALUE_AVERAGE};
use crate::fen::{get_fen, get_position, move_from_algebraic_move, uci_path_from_path};
use crate::move_constants::{
    castle_index_of_move, CASTLE_FLAG, CASTLE_INDEX, CASTLE_MOVE_MASKS, CASTLE_VARS_KING_TO, KING_INDEX, PIECE_MASK_BISHOP,
    PIECE_MASK_FULL, PIECE_MASK_KING, PIECE_MASK_KNIGHT, PIECE_MASK_PAWN, PIECE_MASK_QUEEN, PIECE_MASK_ROOK, PROMOTION_BISHOP_MOVE_MASK,
    PROMOTION_FULL_MOVE_MASK, PROMOTION_KNIGHT_MOVE_MASK, PROMOTION_QUEEN_MOVE_MASK, PROMOTION_ROOK_MOVE_MASK, QUEEN_INDEX,
};
use crate::opponent;
use crate::search::{sync_nodes, MATE_SCORE, MATE_START};
use crate::types::{Bitboard, Move, Position, Score, SearchState, Square, BLACK, WHITE};
use std::sync::atomic::Ordering;

#[inline(always)]
pub const fn from_square_mask(square: Square) -> Move {
    (square as Move) << 16
}

#[inline(always)]
pub const fn from_square_part(mv: Move) -> Square {
    ((mv >> 16) & 63_u32) as Square
}

#[inline(always)]
pub const fn piece_part(mv: Move) -> Square {
    (mv & PIECE_MASK_FULL) as Square
}

#[inline(always)]
pub fn to_square_part(mv: Move) -> Square {
    (mv as Square) & 63
}

/// Check if a move is a capture (not including promotions without capture)
#[inline(always)]
pub fn is_capture(position: &Position, mv: Move) -> bool {
    let enemy = &position.pieces[opponent!(position.mover) as usize];
    let tsp = to_square_part(mv);
    (mv & PIECE_MASK_FULL == PIECE_MASK_PAWN && tsp == position.en_passant_square) || (enemy.all_pieces_bitboard & bit(tsp) != 0)
}

#[inline(always)]
pub fn captured_piece_value(position: &Position, mv: Move) -> Score {
    let enemy = &position.pieces[opponent!(position.mover) as usize];
    let tsp = to_square_part(mv);
    let to_bb = bit(tsp);

    let promote_value = match mv & PROMOTION_FULL_MOVE_MASK {
        PROMOTION_QUEEN_MOVE_MASK => QUEEN_VALUE_AVERAGE - PAWN_VALUE_AVERAGE,
        PROMOTION_ROOK_MOVE_MASK => ROOK_VALUE_AVERAGE - PAWN_VALUE_AVERAGE,
        PROMOTION_BISHOP_MOVE_MASK => BISHOP_VALUE_AVERAGE - PAWN_VALUE_AVERAGE,
        PROMOTION_KNIGHT_MOVE_MASK => KNIGHT_VALUE_AVERAGE - PAWN_VALUE_AVERAGE,
        _ => 0,
    };

    promote_value
        + (if (mv & PIECE_MASK_FULL == PIECE_MASK_PAWN && tsp == position.en_passant_square) || enemy.pawn_bitboard & to_bb != 0 {
            PAWN_VALUE_AVERAGE
        } else if enemy.knight_bitboard & to_bb != 0 {
            KNIGHT_VALUE_AVERAGE
        } else if enemy.bishop_bitboard & to_bb != 0 {
            BISHOP_VALUE_AVERAGE
        } else if enemy.rook_bitboard & to_bb != 0 {
            ROOK_VALUE_AVERAGE
        } else if enemy.queen_bitboard & to_bb != 0 {
            QUEEN_VALUE_AVERAGE
        } else {
            0
        })
}

#[inline(always)]
pub fn moving_piece_mask(position: &Position, mv: Move) -> Move {
    let friendly = &position.pieces[position.mover as usize];
    let from_bb = bit(from_square_part(mv));

    if friendly.pawn_bitboard & from_bb != 0 {
        PIECE_MASK_PAWN
    } else if friendly.knight_bitboard & from_bb != 0 {
        PIECE_MASK_KNIGHT
    } else if friendly.bishop_bitboard & from_bb != 0 {
        PIECE_MASK_BISHOP
    } else if friendly.rook_bitboard & from_bb != 0 {
        PIECE_MASK_ROOK
    } else if friendly.queen_bitboard & from_bb != 0 {
        PIECE_MASK_QUEEN
    } else {
        PIECE_MASK_KING
    }
}

/// The castle flag a UCI move string's from/to squares denote, or 0.
///
/// Both spellings are read (NET-1762): the king taking its own castling rook
/// (`e1h1`), which `UCI_Chess960` prescribes and lichess-bot sends, and the
/// standard king double-step to the g or c file (`e1g1`, `e1c1`). A king
/// stepping one square is never read as a castle, so a GUI that does not
/// speak `UCI_Chess960` can only castle unambiguously - which is why the
/// option exists. The side must still hold the right, as a castle that is
/// not available is not a castle (the move is then rejected as illegal), and
/// the king-takes-rook reading needs the side's own rook on that square: a
/// `K` right kept without a rook behind it (NET-369) must not turn the king's
/// capture of an enemy piece on the empty corner into a castle.
#[inline(always)]
pub fn castle_mask(position: &Position, mv: Move) -> Move {
    let from = from_square_part(mv);
    let to = to_square_part(mv);

    let colour = if from == position.pieces[WHITE as usize].king_square {
        WHITE
    } else if from == position.pieces[BLACK as usize].king_square {
        BLACK
    } else {
        return 0;
    } as usize;
    for side in [KING_INDEX, QUEEN_INDEX] {
        if position.castle_flags & CASTLE_FLAG[side][colour] == 0 {
            continue;
        }
        let index = CASTLE_INDEX[side][colour];
        let rook_square = position.castle_rook_squares[index];
        let takes_own_rook = to == rook_square && position.pieces[colour].rook_bitboard & bit(rook_square) != 0;
        let double_step = to == CASTLE_VARS_KING_TO[index] && from / 8 == to / 8 && (from - to).abs() >= 2;
        if takes_own_rook || double_step {
            return CASTLE_MOVE_MASKS[index];
        }
    }
    0
}

pub fn show_bitboard(title: &str, bitboard: Bitboard) {
    println!("{}", title);
    println!("########");
    for i in (0..=63).rev() {
        print!(" {} ", if bitboard & bit(i) != 0 { '*' } else { '-' });
        if i % 8 == 0 {
            println!()
        }
    }
    println!("########");
}

#[inline(always)]
pub fn hydrate_move_from_algebraic_move(position: &Position, algebraic_move: String) -> Move {
    let mv = move_from_algebraic_move(algebraic_move, 0);
    let castle = castle_mask(position, mv);
    if castle != 0 {
        // The one encoding the generator produces, whichever spelling arrived:
        // king from, king destination, castle flag (so `e1h1` becomes e1->g1)
        return from_square_mask(from_square_part(mv))
            | CASTLE_VARS_KING_TO[castle_index_of_move(castle)] as Move
            | PIECE_MASK_KING
            | castle;
    }
    mv | moving_piece_mask(position, mv)
}

#[inline(always)]
pub fn linear_scale(value: i64, domain_min: i64, domain_max: i64, target_min: i64, target_max: i64) -> i64 {
    if value <= domain_min {
        target_min
    } else if value >= domain_max {
        target_max
    } else {
        target_min + (value - domain_min) * (target_max - target_min) / (domain_max - domain_min)
    }
}

pub fn invert_pos(position: &Position) -> Position {
    let fen = get_fen(position);
    get_position(&invert_fen(&fen))
}

pub fn invert_fen(fen: &str) -> String {
    let inverted_fen = fen
        .trim()
        .replace(" b ", " . ")
        .replace(" w ", " ; ")
        .replace('Q', "z")
        .replace('K', "x")
        .replace('N', "c")
        .replace('B', "v")
        .replace('R', "m")
        .replace('P', ",")
        .replace('q', "Q")
        .replace('k', "K")
        .replace('n', "N")
        .replace('b', "B")
        .replace('r', "R")
        .replace('p', "P")
        .replace('z', "q")
        .replace('x', "k")
        .replace('c', "n")
        .replace('v', "b")
        .replace('m', "r")
        .replace(',', "p")
        .replace(" . ", " w ")
        .replace(" ; ", " b ");

    let fen_parts: Vec<&str> = inverted_fen.split(' ').collect();
    let board_parts: Vec<&str> = fen_parts[0].split('/').collect();

    let en_passant_part = fen_parts[3].replace('6', ".").replace('3', "6").replace('.', "3");
    // The castling field swaps case as a whole: Chess960 rook files (`HAha`)
    // as well as `KQkq` (NET-1762)
    let castle_part: String = fen
        .trim()
        .split(' ')
        .nth(2)
        .unwrap_or("-")
        .chars()
        .map(|c| {
            if c.is_ascii_uppercase() {
                c.to_ascii_lowercase()
            } else {
                c.to_ascii_uppercase()
            }
        })
        .collect();

    format!(
        "{}/{}/{}/{}/{}/{}/{}/{} {} {} {} {} {}",
        board_parts[7],
        board_parts[6],
        board_parts[5],
        board_parts[4],
        board_parts[3],
        board_parts[2],
        board_parts[1],
        board_parts[0],
        fen_parts[1],
        castle_part,
        en_passant_part,
        fen_parts[4],
        fen_parts[5]
    )
}

#[inline(always)]
pub fn penultimate_pawn_push(m: Move) -> bool {
    let move_piece = m & PIECE_MASK_FULL;
    if move_piece == PIECE_MASK_PAWN {
        let to_square = to_square_part(m);
        if to_square >= 48 || to_square <= 15 {
            return true;
        }
    }
    false
}

#[inline(always)]
pub fn pawn_push(position: &Position, m: Move) -> bool {
    let move_piece = m & PIECE_MASK_FULL;
    if move_piece == PIECE_MASK_PAWN {
        let to_square = to_square_part(m);
        if to_square >= 48 || to_square <= 15 {
            return true;
        }
        if position.mover == WHITE {
            if (40..=47).contains(&to_square) {
                return position.pieces[BLACK as usize].pawn_bitboard & WHITE_PASSED_PAWN_MASK[to_square as usize] == 0;
            }
        } else if (16..=23).contains(&to_square) {
            return position.pieces[WHITE as usize].pawn_bitboard & BLACK_PASSED_PAWN_MASK[to_square as usize] == 0;
        }
    }
    false
}

/// Bound annotation on an `info score`, as the UCI protocol defines it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InfoBound {
    Exact,
    Lower,
    Upper,
}

/// Aspiration fail-high/fail-low lines are only worth printing once a search
/// has run long enough for someone to be watching it. Stockfish uses 3 s too.
pub const INFO_BOUND_MIN_MS: u128 = 3000;

/// The `info score` value: `cp N`, or `mate N` in full moves, negative when we
/// are the side being mated. Same arithmetic as Stockfish's `to_score`.
pub fn format_uci_score(score: Score) -> String {
    if score > MATE_START {
        format!("mate {}", (MATE_SCORE - score + 1) / 2)
    } else if score < -MATE_START {
        format!("mate {}", -(MATE_SCORE + score) / 2)
    } else {
        format!("cp {}", score)
    }
}

/// Emit one `info` line for `depth` (plus MultiPV lines when the bound is
/// exact).
///
/// Called once per completed iteration from `iterative_deepening`, once more
/// when a hard stop interrupts an iteration (with the last COMPLETED depth, so
/// the line never claims a depth that was not finished; or, when a completed
/// root fail-high from the interrupted iteration is played instead, with that
/// iteration's depth and a `lowerbound` score, NET-1291), and for aspiration
/// bound lines after `INFO_BOUND_MIN_MS`. `pv` and `score` are passed in
/// rather than read from `current_best` because the bound lines report
/// something else: the failing move's line on a fail-high, the previous
/// iteration's line with the new upper bound on a fail-low. `multi_pv_lines`
/// is set only by the completed-iteration call: the runner-up lines come
/// from the `pv` map, which `start_search` keeps overwriting until the moment
/// of a stop, so at stop time only the saved principal line is trustworthy.
///
/// NET-1244 replaced the old scheme, which printed whichever root move topped
/// the `pv` map - a mixture of exact scores, null-window fail-low bounds (equal
/// to alpha because quiesce fails hard) and stale entries from the previous
/// iteration - after every root move that beat the window floor. Whenever the
/// score dropped between iterations a stale bound won the sort, and the GUI was
/// shown a move and score the search did not believe; 22 of 80 completed
/// depths on the bench set opened with the wrong move.
pub fn send_info(search_state: &mut SearchState, depth: u8, score: Score, pv: &[Move], bound: InfoBound, multi_pv_lines: bool) {
    if !search_state.show_info || depth == 0 || pv.is_empty() || pv[0] == 0 {
        return;
    }
    // Flush this thread's nodes so the total covers every thread
    sync_nodes(search_state);
    let total_nodes = search_state.shared_nodes.load(Ordering::Relaxed).max(search_state.nodes);
    let elapsed_ms = search_state.start_time.elapsed().as_millis().max(1);
    let nps = total_nodes as u128 * 1000 / elapsed_ms;
    let hashfull = search_state.hash_table.hashfull();
    let bound_text = match bound {
        InfoBound::Exact => "",
        InfoBound::Lower => " lowerbound",
        InfoBound::Upper => " upperbound",
    };
    println!(
        "info depth {} seldepth {} multipv 1 score {}{} nodes {} nps {} hashfull {} time {} pv {}",
        depth,
        search_state.sel_depth,
        format_uci_score(score),
        bound_text,
        total_nodes,
        nps,
        hashfull,
        elapsed_ms,
        uci_path_from_path(pv, &search_state.castle_rook_squares, search_state.chess960)
    );
    if !multi_pv_lines || bound != InfoBound::Exact || search_state.multi_pv <= 1 {
        return;
    }
    // MultiPV lines 2 and up. The root loop scouts every move after the best
    // with a null window, so what it knows about a runner-up is an UPPER bound
    // (the move scored at most this much) and the line is its refutation. A
    // true multi-PV search that re-searches the runners-up with a full window
    // is phase 2 of NET-1244; until then a bound is reported as a bound rather
    // than dressed up as a score.
    let mut index = 2u8;
    for &(m, _) in search_state.root_moves.iter() {
        if index > search_state.multi_pv {
            break;
        }
        if m == pv[0] {
            continue;
        }
        if let Some((line, line_score)) = search_state.pv.get(&m) {
            println!(
                "info depth {} seldepth {} multipv {} score {} upperbound nodes {} nps {} hashfull {} time {} pv {}",
                depth,
                search_state.sel_depth,
                index,
                format_uci_score(*line_score),
                total_nodes,
                nps,
                hashfull,
                elapsed_ms,
                uci_path_from_path(line, &search_state.castle_rook_squares, search_state.chess960)
            );
            index += 1;
        }
    }
}
