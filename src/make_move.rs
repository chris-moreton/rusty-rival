pub mod make_move {
    use crate::bitboards::bitboards::{A1_BIT, A8_BIT, bit, E1_BIT, E8_BIT, H1_BIT, H8_BIT, test_bit};
    use crate::fen::fen::get_position;
    use crate::move_constants::move_constants::{EN_PASSANT_NOT_AVAILABLE, PROMOTION_SQUARES};
    use crate::moves::moves::move_piece_within_bitboard;
    use crate::types::types::{Bitboard, Move, Piece, Position, Square};
    use crate::types::types::Mover::{Black, White};
    use crate::types::types::Piece::{Bishop, King, Knight, Pawn, Queen, Rook};
    use crate::utils::utils::{from_square_mask, from_square_part, to_square_part};

    pub fn make_move(position: &mut Position, mv: Move) {
        let from = from_square_part(mv);
        let to = to_square_part(mv);
        let piece = moving_piece(position, from as Square);
        return if is_simple_move(position, mv, from as Square, to, &piece) {
            make_simple_move(position, mv, from as Square, piece)
        } else {
            make_complex_move(position, mv)
        }
    }

    pub fn make_complex_move(position: &Position, mv: Move) {
        todo!()
    }

    pub fn make_simple_move(position: &mut Position, mv: Move, from: Square, piece: Piece) {
        let to = to_square_part(mv);
        let switch_bitboard = bit(from) | bit(to);
        return if position.mover == White {
            make_simple_white_move(position, from, to, switch_bitboard, piece)
        } else {
            make_simple_black_move(position, from, to, switch_bitboard, piece)
        }
    }

    pub fn make_simple_white_move(position: &mut Position, from: Square, to: Square, switch_bitboard: Bitboard, piece: Piece) {
        return if piece == Pawn {
            make_simple_white_pawn_move(position, from, to, switch_bitboard)
        } else if piece == Knight {
            make_simple_white_knight_move(position, from, to, switch_bitboard)
        } else if piece == Bishop {
            make_simple_white_bishop_move(position, from, to, switch_bitboard)
        } else if piece == Rook {
            make_simple_white_rook_move(position, from, to, switch_bitboard)
        } else if piece == Queen {
            make_simple_white_queen_move(position, from, to, switch_bitboard)
        } else {
            make_simple_white_king_move(position, from, to, switch_bitboard)
        }
    }

    pub fn make_simple_black_move(position: &mut Position, from: Square, to: Square, switch_bitboard: Bitboard, piece: Piece) {
        return if piece == Pawn {
            make_simple_black_pawn_move(position, from, to, switch_bitboard)
        } else if piece == Knight {
            make_simple_black_knight_move(position, from, to, switch_bitboard)
        } else if piece == Bishop {
            make_simple_black_bishop_move(position, from, to, switch_bitboard)
        } else if piece == Rook {
            make_simple_black_rook_move(position, from, to, switch_bitboard)
        } else if piece == Queen {
            make_simple_black_queen_move(position, from, to, switch_bitboard)
        } else {
            make_simple_black_king_move(position, from, to, switch_bitboard)
        }
    }

    pub fn make_simple_white_pawn_move(position: &mut Position, from: Square, to: Square, switch_bitboard: Bitboard) {
        position.white_pawn_bitboard = move_piece_within_bitboard(from, to, position.white_pawn_bitboard);
        position.all_pieces_bitboard = position.all_pieces_bitboard ^ switch_bitboard;
        position.white_pieces_bitboard = position.white_pieces_bitboard ^ switch_bitboard;
        position.mover = Black;
        position.en_passant_square = if to - from == 16 { from + 8 } else { EN_PASSANT_NOT_AVAILABLE };
        position.half_moves = 0;
    }

    pub fn make_simple_white_knight_move(position: &mut Position, from: Square, to: Square, switch_bitboard: Bitboard) {
        position.white_knight_bitboard = move_piece_within_bitboard(from, to, position.white_knight_bitboard);
        position.all_pieces_bitboard = position.all_pieces_bitboard ^ switch_bitboard;
        position.white_pieces_bitboard = position.white_pieces_bitboard ^ switch_bitboard;
        position.mover = Black;
        position.en_passant_square = EN_PASSANT_NOT_AVAILABLE;
        position.half_moves += 1;
    }

    pub fn make_simple_white_bishop_move(position: &mut Position, from: Square, to: Square, switch_bitboard: Bitboard) {
        position.white_bishop_bitboard = move_piece_within_bitboard(from, to, position.white_bishop_bitboard);
        position.all_pieces_bitboard = position.all_pieces_bitboard ^ switch_bitboard;
        position.white_pieces_bitboard = position.white_pieces_bitboard ^ switch_bitboard;
        position.mover = Black;
        position.en_passant_square = EN_PASSANT_NOT_AVAILABLE;
        position.half_moves += 1;
    }

    pub fn make_simple_white_rook_move(position: &mut Position, from: Square, to: Square, switch_bitboard: Bitboard) {
        position.white_rook_bitboard = move_piece_within_bitboard(from, to, position.white_rook_bitboard);
        position.all_pieces_bitboard = position.all_pieces_bitboard ^ switch_bitboard;
        position.white_pieces_bitboard = position.white_pieces_bitboard ^ switch_bitboard;
        position.mover = Black;
        position.en_passant_square = EN_PASSANT_NOT_AVAILABLE;
        position.white_king_castle_available = position.white_king_castle_available && from != H1_BIT;
        position.white_queen_castle_available = position.white_queen_castle_available && from != A1_BIT;
        position.half_moves += 1;
    }

    pub fn make_simple_white_queen_move(position: &mut Position, from: Square, to: Square, switch_bitboard: Bitboard) {
        position.white_queen_bitboard = move_piece_within_bitboard(from, to, position.white_queen_bitboard);
        position.all_pieces_bitboard = position.all_pieces_bitboard ^ switch_bitboard;
        position.white_pieces_bitboard = position.white_pieces_bitboard ^ switch_bitboard;
        position.mover = Black;
        position.en_passant_square = EN_PASSANT_NOT_AVAILABLE;
        position.half_moves += 1;
    }

    pub fn make_simple_white_king_move(position: &mut Position, from: Square, to: Square, switch_bitboard: Bitboard) {
        // todo - this line can be sped up - there's only one king 
        position.white_king_bitboard = move_piece_within_bitboard(from, to, position.white_king_bitboard);
        position.all_pieces_bitboard = position.all_pieces_bitboard ^ switch_bitboard;
        position.white_pieces_bitboard = position.white_pieces_bitboard ^ switch_bitboard;
        position.mover = Black;
        position.en_passant_square = EN_PASSANT_NOT_AVAILABLE;
        position.half_moves += 1;
    }

    pub fn make_simple_black_pawn_move(position: &mut Position, from: Square, to: Square, switch_bitboard: Bitboard) {
        position.black_pawn_bitboard = move_piece_within_bitboard(from, to, position.black_pawn_bitboard);
        position.all_pieces_bitboard = position.all_pieces_bitboard ^ switch_bitboard;
        position.black_pieces_bitboard = position.black_pieces_bitboard ^ switch_bitboard;
        position.mover = Black;
        position.en_passant_square = if from - to == 16 { from - 8 } else { EN_PASSANT_NOT_AVAILABLE };
        position.half_moves = 0;
    }

    pub fn make_simple_black_knight_move(position: &mut Position, from: Square, to: Square, switch_bitboard: Bitboard) {
        position.black_knight_bitboard = move_piece_within_bitboard(from, to, position.black_knight_bitboard);
        position.all_pieces_bitboard = position.all_pieces_bitboard ^ switch_bitboard;
        position.black_pieces_bitboard = position.black_pieces_bitboard ^ switch_bitboard;
        position.mover = Black;
        position.en_passant_square = EN_PASSANT_NOT_AVAILABLE;
        position.half_moves += 1;
    }

    pub fn make_simple_black_bishop_move(position: &mut Position, from: Square, to: Square, switch_bitboard: Bitboard) {
        position.black_bishop_bitboard = move_piece_within_bitboard(from, to, position.black_bishop_bitboard);
        position.all_pieces_bitboard = position.all_pieces_bitboard ^ switch_bitboard;
        position.black_pieces_bitboard = position.black_pieces_bitboard ^ switch_bitboard;
        position.mover = Black;
        position.en_passant_square = EN_PASSANT_NOT_AVAILABLE;
        position.half_moves += 1;
    }

    pub fn make_simple_black_rook_move(position: &mut Position, from: Square, to: Square, switch_bitboard: Bitboard) {
        position.black_rook_bitboard = move_piece_within_bitboard(from, to, position.black_rook_bitboard);
        position.all_pieces_bitboard = position.all_pieces_bitboard ^ switch_bitboard;
        position.black_pieces_bitboard = position.black_pieces_bitboard ^ switch_bitboard;
        position.mover = Black;
        position.en_passant_square = EN_PASSANT_NOT_AVAILABLE;
        position.black_king_castle_available = position.black_king_castle_available && from != H8_BIT;
        position.black_queen_castle_available = position.black_queen_castle_available && from != A8_BIT;
        position.half_moves += 1;
    }

    pub fn make_simple_black_queen_move(position: &mut Position, from: Square, to: Square, switch_bitboard: Bitboard) {
        position.black_queen_bitboard = move_piece_within_bitboard(from, to, position.black_queen_bitboard);
        position.all_pieces_bitboard = position.all_pieces_bitboard ^ switch_bitboard;
        position.black_pieces_bitboard = position.black_pieces_bitboard ^ switch_bitboard;
        position.mover = Black;
        position.en_passant_square = EN_PASSANT_NOT_AVAILABLE;
        position.half_moves += 1;
    }

    pub fn make_simple_black_king_move(position: &mut Position, from: Square, to: Square, switch_bitboard: Bitboard) {
        // todo - this line can be sped up - there's only one king 
        position.black_king_bitboard = move_piece_within_bitboard(from, to, position.black_king_bitboard);
        position.all_pieces_bitboard = position.all_pieces_bitboard ^ switch_bitboard;
        position.black_pieces_bitboard = position.black_pieces_bitboard ^ switch_bitboard;
        position.mover = Black;
        position.en_passant_square = EN_PASSANT_NOT_AVAILABLE;
        position.half_moves += 1;
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
                !(*piece == Pawn && is_complex_pawn_move(position, from, to) &&
                    !(*piece == King && is_potential_first_king_move(position, from)));
    }

    pub fn is_simple_capture(position: &Position, square: Square) -> bool {
        return test_bit(position.all_pieces_bitboard, square);
    }

    // todo - remove position param
    pub fn is_complex_pawn_move(position: &Position, from: Square, to: Square) -> bool {
        return (from - to).abs() % 8 != 0 || test_bit(PROMOTION_SQUARES, to);
    }

    // todo - remove position param
    pub fn is_potential_first_king_move(position: &Position, from: Square) -> bool {
        return from == E1_BIT as Square || from == E8_BIT as Square
    }

}