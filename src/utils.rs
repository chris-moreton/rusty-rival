use crate::types::{Move, Square};

#[inline(always)]
pub fn from_square_mask(square: Square) -> Move { (square as Move) << 16 }

#[inline(always)]
pub fn from_square_part(mv: Move) -> Square {
    (mv >> 16) as Square
}

#[inline(always)]
pub fn to_square_part(mv: Move) -> Square {
    (mv as Square) & 63
}
