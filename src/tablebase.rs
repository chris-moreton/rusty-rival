//! Syzygy tablebase probing support
//!
//! This module provides integration with Syzygy endgame tablebases for perfect
//! endgame play in positions with 6 or fewer pieces.

use crate::types::{Bitboard, Position, Score, BLACK, WHITE};
use shakmaty::{CastlingMode, Chess, Color, FromSetup, Piece, Role, Setup, Square};
use shakmaty_syzygy::{AmbiguousWdl, Tablebase};
use std::path::Path;
use std::sync::RwLock;

/// Maximum number of pieces for tablebase probe (6-man tables)
pub const TB_MAX_PIECES: u32 = 6;

/// Score to return for tablebase wins (high but below mate and below MAX_WINDOW)
pub const TB_WIN_SCORE: Score = 19000;

/// Score to return for tablebase losses
pub const TB_LOSS_SCORE: Score = -19000;

/// Global tablebase instance (lazy loaded when path is set)
static TABLEBASE: RwLock<Option<Tablebase<Chess>>> = RwLock::new(None);

/// Initialize the tablebase with the given path
pub fn init_tablebase(path: &str) -> Result<usize, String> {
    let path = Path::new(path);
    if !path.exists() {
        return Err(format!("Tablebase path does not exist: {}", path.display()));
    }

    let mut tb = Tablebase::new();
    let count = tb.add_directory(path).map_err(|e| format!("Failed to load tablebases: {}", e))?;

    let mut global_tb = TABLEBASE.write().unwrap();
    *global_tb = Some(tb);

    Ok(count)
}

/// Check if tablebases are available
pub fn tablebase_available() -> bool {
    TABLEBASE.read().unwrap().is_some()
}

/// Convert our square representation to shakmaty's
/// Our engine: h1=0, g1=1, ..., a1=7, h2=8, ..., a8=63
/// Shakmaty:   a1=0, b1=1, ..., h1=7, a2=8, ..., h8=63
#[inline(always)]
fn convert_square(our_sq: i8) -> Square {
    let rank = our_sq / 8;
    let file = 7 - (our_sq % 8);
    // Safety: rank and file are in valid range 0-7
    Square::from_coords(shakmaty::File::new(file as u32), shakmaty::Rank::new(rank as u32))
}

/// Convert a bitboard from our representation to shakmaty's
fn convert_bitboard(mut our_bb: Bitboard) -> shakmaty::Bitboard {
    let mut result = shakmaty::Bitboard::EMPTY;
    while our_bb != 0 {
        let sq = our_bb.trailing_zeros() as i8;
        our_bb &= our_bb - 1;
        result = result.with(convert_square(sq));
    }
    result
}

/// Convert our Position to a shakmaty Chess position
fn position_to_chess(pos: &Position) -> Result<Chess, String> {
    let mut setup = Setup::empty();

    // Set up white pieces
    let white_pawns = convert_bitboard(pos.pieces[WHITE as usize].pawn_bitboard);
    let white_knights = convert_bitboard(pos.pieces[WHITE as usize].knight_bitboard);
    let white_bishops = convert_bitboard(pos.pieces[WHITE as usize].bishop_bitboard);
    let white_rooks = convert_bitboard(pos.pieces[WHITE as usize].rook_bitboard);
    let white_queens = convert_bitboard(pos.pieces[WHITE as usize].queen_bitboard);
    let white_king = convert_square(pos.pieces[WHITE as usize].king_square);

    // Set up black pieces
    let black_pawns = convert_bitboard(pos.pieces[BLACK as usize].pawn_bitboard);
    let black_knights = convert_bitboard(pos.pieces[BLACK as usize].knight_bitboard);
    let black_bishops = convert_bitboard(pos.pieces[BLACK as usize].bishop_bitboard);
    let black_rooks = convert_bitboard(pos.pieces[BLACK as usize].rook_bitboard);
    let black_queens = convert_bitboard(pos.pieces[BLACK as usize].queen_bitboard);
    let black_king = convert_square(pos.pieces[BLACK as usize].king_square);

    // Build the board
    for sq in white_pawns {
        setup.board.set_piece_at(
            sq,
            Piece {
                color: Color::White,
                role: Role::Pawn,
            },
        );
    }
    for sq in white_knights {
        setup.board.set_piece_at(
            sq,
            Piece {
                color: Color::White,
                role: Role::Knight,
            },
        );
    }
    for sq in white_bishops {
        setup.board.set_piece_at(
            sq,
            Piece {
                color: Color::White,
                role: Role::Bishop,
            },
        );
    }
    for sq in white_rooks {
        setup.board.set_piece_at(
            sq,
            Piece {
                color: Color::White,
                role: Role::Rook,
            },
        );
    }
    for sq in white_queens {
        setup.board.set_piece_at(
            sq,
            Piece {
                color: Color::White,
                role: Role::Queen,
            },
        );
    }
    setup.board.set_piece_at(
        white_king,
        Piece {
            color: Color::White,
            role: Role::King,
        },
    );

    for sq in black_pawns {
        setup.board.set_piece_at(
            sq,
            Piece {
                color: Color::Black,
                role: Role::Pawn,
            },
        );
    }
    for sq in black_knights {
        setup.board.set_piece_at(
            sq,
            Piece {
                color: Color::Black,
                role: Role::Knight,
            },
        );
    }
    for sq in black_bishops {
        setup.board.set_piece_at(
            sq,
            Piece {
                color: Color::Black,
                role: Role::Bishop,
            },
        );
    }
    for sq in black_rooks {
        setup.board.set_piece_at(
            sq,
            Piece {
                color: Color::Black,
                role: Role::Rook,
            },
        );
    }
    for sq in black_queens {
        setup.board.set_piece_at(
            sq,
            Piece {
                color: Color::Black,
                role: Role::Queen,
            },
        );
    }
    setup.board.set_piece_at(
        black_king,
        Piece {
            color: Color::Black,
            role: Role::King,
        },
    );

    // Set side to move
    setup.turn = if pos.mover == WHITE { Color::White } else { Color::Black };

    // Set en passant square
    if pos.en_passant_square != -1 {
        setup.ep_square = Some(convert_square(pos.en_passant_square));
    }

    // Set castling rights - for tablebase positions (<=6 pieces), castling is not possible
    // so we can leave castling_rights empty (the default)
    // In positions with more pieces we'd need to set this, but we won't probe those anyway

    // Set halfmove clock and fullmove number
    setup.halfmoves = pos.half_moves as u32;
    setup.fullmoves = std::num::NonZeroU32::new(pos.move_number as u32).unwrap_or(std::num::NonZeroU32::MIN);

    Chess::from_setup(setup, CastlingMode::Standard).map_err(|e| format!("Invalid position: {:?}", e))
}

/// Count total pieces on the board
#[inline(always)]
pub fn count_pieces(pos: &Position) -> u32 {
    let white = &pos.pieces[WHITE as usize];
    let black = &pos.pieces[BLACK as usize];

    white.pawn_bitboard.count_ones()
        + white.knight_bitboard.count_ones()
        + white.bishop_bitboard.count_ones()
        + white.rook_bitboard.count_ones()
        + white.queen_bitboard.count_ones()
        + 1 // white king
        + black.pawn_bitboard.count_ones()
        + black.knight_bitboard.count_ones()
        + black.bishop_bitboard.count_ones()
        + black.rook_bitboard.count_ones()
        + black.queen_bitboard.count_ones()
        + 1 // black king
}

/// Probe the tablebase for the current position
/// Returns Some(score) if the position is in the tablebase, None otherwise
pub fn probe_wdl(pos: &Position) -> Option<Score> {
    // Quick check: only probe if we have few enough pieces
    if count_pieces(pos) > TB_MAX_PIECES {
        return None;
    }

    let tb_guard = TABLEBASE.read().unwrap();
    let tb = tb_guard.as_ref()?;

    let chess = position_to_chess(pos).ok()?;

    match tb.probe_wdl(&chess) {
        Ok(wdl) => {
            // AmbiguousWdl considers the 50-move rule
            let score = match wdl {
                AmbiguousWdl::Win => TB_WIN_SCORE,
                AmbiguousWdl::Loss => TB_LOSS_SCORE,
                AmbiguousWdl::Draw
                | AmbiguousWdl::CursedWin
                | AmbiguousWdl::BlessedLoss
                | AmbiguousWdl::MaybeWin
                | AmbiguousWdl::MaybeLoss => 0,
            };
            Some(score)
        }
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fen::get_position;

    #[test]
    fn test_count_pieces_starting_position() {
        let pos = get_position("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        assert_eq!(count_pieces(&pos), 32);
    }

    #[test]
    fn test_count_pieces_kpk() {
        let pos = get_position("8/8/8/4k3/8/8/4P3/4K3 w - - 0 1");
        assert_eq!(count_pieces(&pos), 3);
    }

    #[test]
    fn test_count_pieces_krk() {
        let pos = get_position("8/8/8/4k3/8/8/8/R3K3 w - - 0 1");
        assert_eq!(count_pieces(&pos), 3);
    }

    #[test]
    fn test_convert_square() {
        // h1 in our system is 0, should be h1 (square 7) in shakmaty
        assert_eq!(convert_square(0), Square::H1);
        // a1 in our system is 7, should be a1 (square 0) in shakmaty
        assert_eq!(convert_square(7), Square::A1);
        // h8 in our system is 56, should be h8 (square 63) in shakmaty
        assert_eq!(convert_square(56), Square::H8);
        // a8 in our system is 63, should be a8 (square 56) in shakmaty
        assert_eq!(convert_square(63), Square::A8);
        // e4 in our system: rank=3, file=4 (e), so square = 3*8 + (7-4) = 27
        assert_eq!(convert_square(27), Square::E4);
    }
}
