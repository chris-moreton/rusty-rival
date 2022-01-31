use crate::move_constants::PIECE_MASK_FULL;
use crate::types::{Move, Square};

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
