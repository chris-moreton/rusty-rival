pub mod utils {
    use crate::types::types::{Move, Square};

    pub fn from_square_mask(square: Square) -> Move {
        return (square as Move) << 16;
    }

    pub fn from_square_part(mv: Move) -> Square {
        return (mv >> 16) as Square;
    }

    pub fn to_square_part(mv: Move) -> Square {
        return (mv as Square) & 63;
    }
}
