use crate::bitboards::{A1_BIT, A8_BIT, E1_BIT, E8_BIT, H1_BIT, H8_BIT};
use crate::types::{Bitboard, Move, Square};
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

pub const KING_INDEX: usize = 0;
pub const QUEEN_INDEX: usize = 1;

pub const CASTLE_FLAG: [[u8; 2]; 2] = [[1,4],[2,8]];

pub const WHITE_CASTLE_FLAGS: u8 = 3;
pub const BLACK_CASTLE_FLAGS: u8 = 12;

pub const ALL_CASTLE_FLAGS: u8 = 15;

pub const WK_CASTLE: u8 = 1;
pub const WQ_CASTLE: u8 = 2;
pub const BK_CASTLE: u8 = 4;
pub const BQ_CASTLE: u8 = 8;

pub const MAX_MOVE_HISTORY: u16 = 1024;

pub const WHITE_KING_CASTLE_MOVE: Move = from_square_mask(3) | 1;
pub const WHITE_QUEEN_CASTLE_MOVE: Move = from_square_mask(3) | 5;
pub const BLACK_KING_CASTLE_MOVE: Move = from_square_mask(59) | 57;
pub const BLACK_QUEEN_CASTLE_MOVE: Move = from_square_mask(59) | 61;

pub const CASTLE_MOVE: [[Move; 2]; 2] = [[WHITE_KING_CASTLE_MOVE, BLACK_KING_CASTLE_MOVE], [WHITE_QUEEN_CASTLE_MOVE, BLACK_QUEEN_CASTLE_MOVE]];
pub const KING_ROOK_START: [Square; 2] = [H1_BIT, H8_BIT];
pub const QUEEN_ROOK_START: [Square; 2] = [A1_BIT, A8_BIT];
pub const KING_START: [Square; 2] = [E1_BIT, E8_BIT];
