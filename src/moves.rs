pub mod moves {
    use crate::bitboards::bitboards::bit_list;
    use crate::types::types::{Bitboard, MoveList, Position, Square};
    use crate::types::types::Mover::White;
    use crate::utils::utils::from_square_mask;

    pub fn all_bits_except_friendly_pieces(position: &Position) -> Bitboard {
        return !if position.mover == White { position.white_pieces_bitboard } else { position.black_pieces_bitboard }
    }

    pub fn moves_from_to_squares_bitboard(from: Square, to_bitboard: Bitboard) -> MoveList {
        let from_part_only = from_square_mask(from);
        let to_squares = bit_list(to_bitboard);
        let mut move_list = vec![];
        to_squares.iter().for_each(|sq| {
            move_list.push(from_part_only | sq);
        });
        return move_list;
    }

}