use crate::move_constants::{BK_CASTLE, BQ_CASTLE, MAX_MOVE_HISTORY, WK_CASTLE, WQ_CASTLE};

pub type Square = i8;
pub type Bitboard = u64;
pub type Move = u32;
pub type MoveList = Vec<Move>;
pub type Path = Vec<Move>;
pub type MagicFunc = fn(Square, u64) -> Bitboard;
pub type MagicMovesArray = [[Bitboard; 4096]; 64];
pub type Mover = i8;

pub const WHITE: i8 = -1;
pub const BLACK: i8 = 1;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Piece { Pawn, King, Queen, Bishop, Knight, Rook, Empty }

#[derive(Debug, PartialEq)]
pub enum Bound { Exact, Lower, Upper }

pub struct MagicVars {
    pub(crate) occupancy_mask: [Bitboard; 64],
    pub(crate) magic_number: [Bitboard; 64],
    pub(crate) magic_number_shifts: [u8; 64],
    pub(crate) magic_moves: [[Bitboard; 4096]; 64]
}

pub struct MagicBox {
    pub(crate) bishop: Box<MagicVars>,
    pub(crate) rook: Box<MagicVars>,
}

#[inline(always)]
pub fn unset_wk_castle(position: &mut Position) { position.castle_flags &= !WK_CASTLE }
#[inline(always)]
pub fn unset_wq_castle(position: &mut Position) { position.castle_flags &= !WQ_CASTLE }
#[inline(always)]
pub fn unset_bk_castle(position: &mut Position) { position.castle_flags &= !BK_CASTLE }
#[inline(always)]
pub fn unset_bq_castle(position: &mut Position) { position.castle_flags &= !BQ_CASTLE }

#[inline(always)]
pub fn unset_white_castles(position: &mut Position) { position.castle_flags &= !(WK_CASTLE | WQ_CASTLE) }
#[inline(always)]
pub fn unset_black_castles(position: &mut Position) { position.castle_flags &= !(BK_CASTLE | BQ_CASTLE) }

#[inline(always)]
pub fn is_wk_castle_available(position: &Position) -> bool { position.castle_flags & WK_CASTLE != 0 }
#[inline(always)]
pub fn is_wq_castle_available(position: &Position) -> bool { position.castle_flags & WQ_CASTLE != 0 }
#[inline(always)]
pub fn is_bk_castle_available(position: &Position) -> bool { position.castle_flags & BK_CASTLE != 0 }
#[inline(always)]
pub fn is_bq_castle_available(position: &Position) -> bool { position.castle_flags & BQ_CASTLE != 0 }

#[inline(always)]
pub fn is_any_white_castle_available(position: &Position) -> bool { position.castle_flags & (WK_CASTLE | WQ_CASTLE) != 0 }

#[inline(always)]
pub fn is_any_black_castle_available(position: &Position) -> bool { position.castle_flags & (BK_CASTLE | BQ_CASTLE) != 0 }

#[derive(Debug, Copy, Clone)]
pub struct Position {
    pub white_pawn_bitboard: Bitboard,
    pub white_knight_bitboard: Bitboard,
    pub white_bishop_bitboard: Bitboard,
    pub white_queen_bitboard: Bitboard,
    pub white_king_bitboard: Bitboard,
    pub white_rook_bitboard: Bitboard,
    pub black_pawn_bitboard: Bitboard,
    pub black_knight_bitboard: Bitboard,
    pub black_bishop_bitboard: Bitboard,
    pub black_queen_bitboard: Bitboard,
    pub black_king_bitboard: Bitboard,
    pub black_rook_bitboard: Bitboard,
    pub all_pieces_bitboard: Bitboard,
    pub white_pieces_bitboard: Bitboard,
    pub black_pieces_bitboard: Bitboard,
    pub mover: Mover,
    pub en_passant_square: Square,
    pub castle_flags: u8,
    pub half_moves: u16,
    pub move_number: u16,
}

#[derive(Debug, Copy, Clone)]
pub struct PositionHistory {
    pub history: [Position; MAX_MOVE_HISTORY as usize]
}

impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        self.white_pawn_bitboard == other.white_pawn_bitboard &&
        self.white_knight_bitboard == other.white_knight_bitboard &&
        self.white_bishop_bitboard == other.white_bishop_bitboard &&
        self.white_queen_bitboard == other.white_queen_bitboard &&
        self.white_king_bitboard == other.white_king_bitboard &&
        self.white_rook_bitboard == other.white_rook_bitboard &&
        self.black_pawn_bitboard == other.black_pawn_bitboard &&
        self.black_knight_bitboard == other.black_knight_bitboard &&
        self.black_bishop_bitboard == other.black_bishop_bitboard &&
        self.black_queen_bitboard == other.black_queen_bitboard &&
        self.black_king_bitboard == other.black_king_bitboard &&
        self.black_rook_bitboard == other.black_rook_bitboard &&
        self.all_pieces_bitboard == other.all_pieces_bitboard &&
        self.white_pieces_bitboard == other.white_pieces_bitboard &&
        self.black_pieces_bitboard == other.black_pieces_bitboard &&
        self.mover == other.mover &&
        self.en_passant_square == other.en_passant_square &&
        self.castle_flags == other.castle_flags &&
        self.half_moves == other.half_moves &&
        self.move_number == other.move_number
    }
}
