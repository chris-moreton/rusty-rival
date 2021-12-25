pub mod moves {
    use crate::types::types::{Bitboard, Position};
    use crate::types::types::Mover::White;

    pub fn all_bits_except_friendly_pieces(position: &Position) -> Bitboard {
        return !if position.mover == White { position.white_pieces_bitboard } else { position.black_pieces_bitboard }
    }

}