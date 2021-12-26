pub mod moves {
    use crate::bitboards::bitboards::{bit_list, bitboard_for_mover, KNIGHT_MOVES_BITBOARDS};
    use crate::types::types::{Bitboard, MoveList, Position, Square};
    use crate::types::types::Mover::White;
    use crate::types::types::Piece::Knight;
    use crate::utils::utils::from_square_mask;

    pub fn all_bits_except_friendly_pieces(position: &Position) -> Bitboard {
        return !if position.mover == White { position.white_pieces_bitboard } else { position.black_pieces_bitboard }
    }

    pub fn moves_from_to_squares_bitboard(from: Square, to_bitboard: Bitboard) -> MoveList {
        let from_part_only = from_square_mask(from);
        let to_squares = bit_list(to_bitboard);
        let mut move_list: MoveList = vec![];
        to_squares.iter().for_each(|sq| {
            let mv = from_part_only | (*sq as u32);
            move_list.push(mv);
        });
        return move_list;
    }

    pub fn generate_knight_moves(position: &Position) -> MoveList {
        let valid_destinations = all_bits_except_friendly_pieces(position);
        let from_squares = bit_list(bitboard_for_mover(position, &Knight));
        let mut move_list = Vec::new();
        from_squares.iter().for_each(|from_square| {
            let to_squares = bit_list(KNIGHT_MOVES_BITBOARDS[*from_square as usize] & valid_destinations);
            to_squares.iter().for_each(|to_square| {
               move_list.push(from_square_mask(*from_square as i8) | *to_square as u32);
            });
        });
        return move_list;
    }

}

// {-# INLINE generateKnightMoves #-}
// generateKnightMoves :: Position -> MoveList
// generateKnightMoves !position = generateKnightMovesWithTargets position (allBitsExceptFriendlyPieces position)
//
// {-# INLINE generateKnightMovesWithTargets #-}
// generateKnightMovesWithTargets :: Position -> Bitboard -> MoveList
// generateKnightMovesWithTargets !position validLandingSquares =
// [fromSquareMask fromSquare .|. toSquare | fromSquare <- bitList $ bitboardForMover position Knight,
// toSquare   <- bitList $ knightMovesBitboards fromSquare .&. validLandingSquares]