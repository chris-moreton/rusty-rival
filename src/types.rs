use crate::move_constants::{BK_CASTLE, BQ_CASTLE, MAX_MOVE_HISTORY, WK_CASTLE, WQ_CASTLE};

pub type Square = i8;
pub type Bitboard = u64;
pub type Move = u32;
pub type MoveList = Vec<Move>;
pub type Path = Vec<Move>;
pub type MagicMovesArray = [[Bitboard; 4096]; 64];
pub type Mover = i8;
pub type PositionHistory = [Position; MAX_MOVE_HISTORY as usize];

#[macro_export]
macro_rules! switch_side {
    ($a:expr) => { $a *= -1 }
}

#[macro_export]
macro_rules! opponent {
    ($a:expr) => { $a * -1 }
}

#[macro_export]
macro_rules! unset_lsb {
    ($a:expr) => { $a &= $a - 1 }
}

#[macro_export]
macro_rules! get_and_unset_lsb {
    ($a:expr) => {
        {
            let lsb = $a.trailing_zeros() as Square;
            $a &= $a - 1;
            lsb
        }
    }
}

#[macro_export]
macro_rules! move_mover_white {
    ($bitboard:expr, $from_mask:expr, $to_mask:expr, $position:expr) => {
        if $bitboard & $from_mask != 0 {
            let switch = $from_mask | $to_mask;
            $bitboard ^= switch;
            $position.white.all_pieces_bitboard ^= switch;
        }
    }
}

#[macro_export]
macro_rules! move_mover_black {
    ($bitboard:expr, $from_mask:expr, $to_mask:expr, $position:expr) => {
        if $bitboard & $from_mask != 0 {
            let switch = $from_mask | $to_mask;
            $bitboard ^= switch;
            $position.black.all_pieces_bitboard ^= switch;
        }
    }
}

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
    pub bishop: Box<MagicVars>,
    pub rook: Box<MagicVars>,
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
pub struct Pieces {
    pub pawn_bitboard: Bitboard,
    pub knight_bitboard: Bitboard,
    pub bishop_bitboard: Bitboard,
    pub queen_bitboard: Bitboard,
    pub king_square: Square,
    pub rook_bitboard: Bitboard,
    pub all_pieces_bitboard: Bitboard
}

#[derive(Debug, Copy, Clone)]
pub struct Position {
    pub white: Pieces,
    pub black: Pieces,
    pub mover: Mover,
    pub en_passant_square: Square,
    pub castle_flags: u8,
    pub half_moves: u16,
    pub move_number: u16,
}

impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        self.white.pawn_bitboard == other.white.pawn_bitboard &&
        self.white.knight_bitboard == other.white.knight_bitboard &&
        self.white.bishop_bitboard == other.white.bishop_bitboard &&
        self.white.queen_bitboard == other.white.queen_bitboard &&
        self.white.king_square == other.white.king_square &&
        self.white.rook_bitboard == other.white.rook_bitboard &&
        self.black.pawn_bitboard == other.black.pawn_bitboard &&
        self.black.knight_bitboard == other.black.knight_bitboard &&
        self.black.bishop_bitboard == other.black.bishop_bitboard &&
        self.black.queen_bitboard == other.black.queen_bitboard &&
        self.black.king_square == other.black.king_square &&
        self.black.rook_bitboard == other.black.rook_bitboard &&
        self.white.all_pieces_bitboard == other.white.all_pieces_bitboard &&
        self.black.all_pieces_bitboard == other.black.all_pieces_bitboard &&
        self.mover == other.mover &&
        self.en_passant_square == other.en_passant_square &&
        self.castle_flags == other.castle_flags &&
        self.half_moves == other.half_moves &&
        self.move_number == other.move_number
    }
}
