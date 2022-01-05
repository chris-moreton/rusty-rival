use crate::move_constants::{BK_CASTLE, BQ_CASTLE, MAX_MOVE_HISTORY, WK_CASTLE, WQ_CASTLE};

pub type Square = i8;
pub type Bitboard = u64;
pub type Move = u32;
pub type MoveList = Vec<Move>;
pub type Path = Vec<Move>;
pub type MagicFunc = fn(Square, u64) -> Bitboard;
pub type MagicMovesArray = [[Bitboard; 4096]; 64];

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Mover { White, Black }

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
pub fn unset_wk_castle(position: &mut Position) { position.main.castle_flags &= !WK_CASTLE }
#[inline(always)]
pub fn unset_wq_castle(position: &mut Position) { position.main.castle_flags &= !WQ_CASTLE }
#[inline(always)]
pub fn unset_bk_castle(position: &mut Position) { position.main.castle_flags &= !BK_CASTLE }
#[inline(always)]
pub fn unset_bq_castle(position: &mut Position) { position.main.castle_flags &= !BQ_CASTLE }

#[inline(always)]
pub fn unset_white_castles(position: &mut Position) { position.main.castle_flags &= !(WK_CASTLE | WQ_CASTLE) }
#[inline(always)]
pub fn unset_black_castles(position: &mut Position) { position.main.castle_flags &= !(BK_CASTLE | BQ_CASTLE) }

#[inline(always)]
pub fn is_wk_castle_available(position: &Position) -> bool { position.main.castle_flags & WK_CASTLE != 0 }
#[inline(always)]
pub fn is_wq_castle_available(position: &Position) -> bool { position.main.castle_flags & WQ_CASTLE != 0 }
#[inline(always)]
pub fn is_bk_castle_available(position: &Position) -> bool { position.main.castle_flags & BK_CASTLE != 0 }
#[inline(always)]
pub fn is_bq_castle_available(position: &Position) -> bool { position.main.castle_flags & BQ_CASTLE != 0 }

#[inline(always)]
pub fn is_any_white_castle_available(position: &Position) -> bool { position.main.castle_flags & (WK_CASTLE | WQ_CASTLE) != 0 }

#[inline(always)]
pub fn is_any_black_castle_available(position: &Position) -> bool { position.main.castle_flags & (BK_CASTLE | BQ_CASTLE) != 0 }

#[derive(Debug, Copy, Clone)]
pub struct PositionSupplement {
    pub all_pieces_bitboard: Bitboard,
    pub white_pieces_bitboard: Bitboard,
    pub black_pieces_bitboard: Bitboard
}

#[derive(Debug, Copy, Clone)]
pub struct PositionMain {
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
    pub mover: Mover,
    pub en_passant_square: Square,
    pub castle_flags: u8,
    pub half_moves: u16,
    pub move_number: u16
}

#[derive(Debug, Copy, Clone)]
pub struct Position {
    pub main: PositionMain,
    pub supplement: PositionSupplement,
}

#[derive(Debug, Copy, Clone)]
pub struct PositionHistory {
    pub history: [PositionMain; MAX_MOVE_HISTORY as usize]
}

impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        self.main.white_pawn_bitboard == other.main.white_pawn_bitboard &&
            self.main.white_knight_bitboard == other.main.white_knight_bitboard &&
            self.main.white_bishop_bitboard == other.main.white_bishop_bitboard &&
            self.main.white_queen_bitboard == other.main.white_queen_bitboard &&
            self.main.white_king_bitboard == other.main.white_king_bitboard &&
            self.main.white_rook_bitboard == other.main.white_rook_bitboard &&
            self.main.black_pawn_bitboard == other.main.black_pawn_bitboard &&
            self.main.black_knight_bitboard == other.main.black_knight_bitboard &&
            self.main.black_bishop_bitboard == other.main.black_bishop_bitboard &&
            self.main.black_queen_bitboard == other.main.black_queen_bitboard &&
            self.main.black_king_bitboard == other.main.black_king_bitboard &&
            self.main.black_rook_bitboard == other.main.black_rook_bitboard &&
            self.supplement.all_pieces_bitboard == other.supplement.all_pieces_bitboard &&
            self.supplement.white_pieces_bitboard == other.supplement.white_pieces_bitboard &&
            self.supplement.black_pieces_bitboard == other.supplement.black_pieces_bitboard &&
            self.main.mover == other.main.mover &&
            self.main.en_passant_square == other.main.en_passant_square &&
            self.main.castle_flags == other.main.castle_flags &&
            self.main.half_moves == other.main.half_moves &&
            self.main.move_number == other.main.move_number
    }
}
