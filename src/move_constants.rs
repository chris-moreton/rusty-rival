use crate::types::{Bitboard, Move};
use crate::utils::from_square_mask;

pub const PROMOTION_QUEEN_MOVE_MASK: Move = 192;
pub const PROMOTION_ROOK_MOVE_MASK: Move = 64;
pub const PROMOTION_BISHOP_MOVE_MASK: Move = 128;
pub const PROMOTION_KNIGHT_MOVE_MASK: Move = 256;
pub const PROMOTION_FULL_MOVE_MASK: Move = 448;
pub const EN_PASSANT_NOT_AVAILABLE: i8 = -1;
pub const PROMOTION_SQUARES: Bitboard = 0b1111111100000000000000000000000000000000000000000000000011111111;
pub const KING_START_POSITIONS: Bitboard = 0b0000100000000000000000000000000000000000000000000000000000001000;
pub const NON_PROMOTION_SQUARES: Bitboard = 0b0000000011111111111111111111111111111111111111111111111100000000;

pub const WK_CASTLE: u8 = 1;
pub const WQ_CASTLE: u8 = 2;
pub const BK_CASTLE: u8 = 4;
pub const BQ_CASTLE: u8 = 8;

pub const ALL_CASTLE_FLAGS: u8 = WK_CASTLE | WQ_CASTLE | BK_CASTLE | BQ_CASTLE;

pub const MAX_MOVE_HISTORY: u16 = 1024;

pub const WHITE_KING_CASTLE_MOVE: Move = from_square_mask(3) | 1;
pub const WHITE_QUEEN_CASTLE_MOVE: Move = from_square_mask(3) | 5;
pub const BLACK_KING_CASTLE_MOVE: Move = from_square_mask(59) | 57;
pub const BLACK_QUEEN_CASTLE_MOVE: Move = from_square_mask(59) | 61;
