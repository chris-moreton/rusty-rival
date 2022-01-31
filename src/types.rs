use crate::move_constants::{BK_CASTLE, BQ_CASTLE, WK_CASTLE, WQ_CASTLE};

pub type Square = i8;
pub type Bitboard = u64;
pub type Move = u32;
pub type MoveList = Vec<Move>;
pub type Path = Vec<Move>;
pub type MagicMovesArray = [[Bitboard; 4096]; 64];
pub type Mover = i8;

#[macro_export]
macro_rules! opponent {
    ($a:expr) => { ($a-1).abs() }
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

pub const WHITE: Mover = 0;
pub const BLACK: Mover = 1;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Piece { Pawn, King, Queen, Bishop, Knight, Rook, Empty }

#[derive(Debug, PartialEq)]
pub enum Bound { Exact, Lower, Upper }

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

impl PartialEq for Pieces {
    fn eq(&self, other: &Self) -> bool {
        self.pawn_bitboard == other.pawn_bitboard &&
        self.knight_bitboard == other.knight_bitboard &&
        self.bishop_bitboard == other.bishop_bitboard &&
        self.queen_bitboard == other.queen_bitboard &&
        self.king_square == other.king_square &&
        self.rook_bitboard == other.rook_bitboard &&
        self.all_pieces_bitboard == other.all_pieces_bitboard
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Position {
    pub pieces: [Pieces; 2],
    pub mover: Mover,
    pub en_passant_square: Square,
    pub castle_flags: u8,
    pub half_moves: u16,
    pub move_number: u16,
}

impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        self.pieces[0] == other.pieces[0] &&
        self.pieces[1] == other.pieces[1] &&
        self.mover == other.mover &&
        self.en_passant_square == other.en_passant_square &&
        self.castle_flags == other.castle_flags &&
        self.half_moves == other.half_moves &&
        self.move_number == other.move_number
    }
}
