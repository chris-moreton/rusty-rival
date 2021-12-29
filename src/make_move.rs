pub mod make_move {
    use crate::bitboards::bitboards::test_bit;
    use crate::move_constants::move_constants::PROMOTION_SQUARES;
    use crate::types::types::{Move, Piece, Position, Square};
    use crate::types::types::Piece::{Bishop, King, Knight, Pawn, Queen, Rook};
    use crate::utils::utils::{from_square_mask, to_square_part};

    pub fn make_move(position: &Position, mv: Move) -> Position {
        let from = from_square_mask(mv as Square);
        let to = to_square_part(mv);
        let piece = moving_piece(position, from as Square);
        return if is_simple_move(position, mv, from as Square, to, &piece) {
            make_simple_move(position, mv, from, piece)
        } else {
            make_complex_move(position, mv)
        }
    }

    pub fn moving_piece(position: &Position, from_square: Square) -> Piece {
        if test_bit(position.white_pawn_bitboard | position.black_pawn_bitboard, from_square) { return Pawn }
        if test_bit(position.white_knight_bitboard | position.black_knight_bitboard, from_square) { return Knight }
        if test_bit(position.white_bishop_bitboard | position.black_bishop_bitboard, from_square) { return Bishop }
        if test_bit(position.white_rook_bitboard | position.black_rook_bitboard, from_square) { return Rook }
        if test_bit(position.white_queen_bitboard | position.black_queen_bitboard, from_square) { return Queen }
        return King
    }

    pub fn is_simple_move(position: &Position, mv: Move, from: Square, to: Square, piece: &Piece) -> bool {
        return
            !is_simple_capture(position, to) &&
                !(piece == Pawn && is_complex_pawn_move(position, from, to) &&
                    !(piece == King && is_potential_first_king_move(position, from)));
    }

    pub fn is_simple_capture(position: &Position, square: Square) -> bool {
        return test_bit(position.all_pieces_bitboard, square);
    }

    pub fn is_complex_pawn_move(position: &Position, from: Square, to: Square) -> bool {
        return (from - to).abs() % 8 != 0 || test_bit(PROMOTION_SQUARES, to);
    }

}