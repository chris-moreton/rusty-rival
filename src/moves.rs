pub mod moves {
    use crate::bitboards::bitboards::{bit, bit_list, bitboard_for_mover, BLACK_PAWN_MOVES_CAPTURE, BLACK_PAWN_MOVES_FORWARD, clear_bit, EMPTY_CASTLE_SQUARES_BLACK_KING, EMPTY_CASTLE_SQUARES_BLACK_QUEEN, EMPTY_CASTLE_SQUARES_WHITE_KING, EMPTY_CASTLE_SQUARES_WHITE_QUEEN, empty_squares_bitboard, enemy_bitboard, KING_MOVES_BITBOARDS, KNIGHT_MOVES_BITBOARDS, NO_CHECK_CASTLE_SQUARES_BLACK_KING, NO_CHECK_CASTLE_SQUARES_BLACK_QUEEN, NO_CHECK_CASTLE_SQUARES_WHITE_KING, NO_CHECK_CASTLE_SQUARES_WHITE_QUEEN, RANK_3_BITS, RANK_4_BITS, RANK_5_BITS, RANK_6_BITS, slider_bitboard_for_colour, test_bit, WHITE_PAWN_MOVES_CAPTURE, WHITE_PAWN_MOVES_FORWARD};
    use crate::magic_bitboards::magic_bitboards::{magic, magic_bishop, MAGIC_BISHOP_VARS, magic_index_for_bishop, magic_index_for_rook, magic_rook, MAGIC_ROOK_VARS};
    use crate::move_constants::move_constants::{EN_PASSANT_NOT_AVAILABLE, PROMOTION_BISHOP_MOVE_MASK, PROMOTION_KNIGHT_MOVE_MASK, PROMOTION_QUEEN_MOVE_MASK, PROMOTION_ROOK_MOVE_MASK};
    use crate::types::types::{Bitboard, Move, MoveList, Mover, Piece, Position, Square};
    use crate::types::types::Mover::{Black, White};
    use crate::types::types::Piece::{Bishop, King, Knight, Pawn, Rook};
    use crate::utils::utils::from_square_mask;

    #[inline(always)]
    pub fn all_bits_except_friendly_pieces(position: &Position) -> Bitboard {
        return !if position.mover == White { position.white_pieces_bitboard } else { position.black_pieces_bitboard }
    }

    #[inline(always)]
    pub fn moves_from_to_squares_bitboard(from: Square, to_bitboard: Bitboard) -> MoveList {
        let from_part_only = from_square_mask(from);
        let to_squares = bit_list(to_bitboard);
        let mut move_list: MoveList = vec![];
        for sq in to_squares {
            let mv = from_part_only | (sq as u32);
            move_list.push(mv);
        }
        return move_list;
    }

    #[inline(always)]
    pub fn generate_knight_moves(position: &Position) -> MoveList {
        let valid_destinations = all_bits_except_friendly_pieces(position);
        let from_squares = bit_list(bitboard_for_mover(position, &Knight));
        let mut move_list = Vec::new();
        for from_square in from_squares {
            let to_squares = bit_list(KNIGHT_MOVES_BITBOARDS[from_square as usize] & valid_destinations);
            for to_square in to_squares {
               move_list.push(from_square_mask(from_square as i8) | to_square as u32);
            };
        };
        return move_list;
    }

    #[inline(always)]
    pub fn generate_king_moves(position: &Position) -> MoveList {
        let valid_destinations = all_bits_except_friendly_pieces(position);
        let from_square = bitboard_for_mover(position, &King).trailing_zeros();
        let mut move_list = Vec::new();
        let to_squares = bit_list(KING_MOVES_BITBOARDS[from_square as usize] & valid_destinations);
        for to_square in to_squares {
            move_list.push(from_square_mask(from_square as i8) | to_square as u32);
        }
        return move_list;
    }

    #[inline(always)]
    pub fn generate_slider_moves(position: &Position, piece: Piece) -> MoveList {
        return generate_slider_moves_with_targets(position, piece, all_bits_except_friendly_pieces(position));
    }

    #[inline(always)]
    pub fn generate_slider_moves_with_targets(position: &Position, piece: Piece, valid_destinations: Bitboard) -> MoveList {
        let from_squares = bit_list(slider_bitboard_for_colour(position, &position.mover, &piece));
        let mut move_list = Vec::new();
        for from_square in from_squares {
            let magic_vars = if piece == Bishop { &MAGIC_BISHOP_VARS } else { &MAGIC_ROOK_VARS };
            let number_magic = magic_vars.magic_number[from_square as usize];
            let shift_magic = magic_vars.magic_number_shifts[from_square as usize];
            let mask_magic = magic_vars.occupancy_mask[from_square as usize];
            let occupancy = position.all_pieces_bitboard & mask_magic;
            let raw_index: u64 = (0b1111111111111111111111111111111111111111111111111111111111111111 & ((occupancy as u128 * number_magic as u128) as u128)) as u64;
            let to_squares_magic_index = raw_index >> shift_magic;
            let to_squares = bit_list(magic(magic_vars, from_square as Square, to_squares_magic_index) & valid_destinations);
            for to_square in to_squares {
                move_list.push(from_square_mask(from_square as i8) | to_square as u32);
            }
        };
        return move_list;
    }

    #[inline(always)]
    pub fn promotion_moves(mv: Move) -> MoveList {
        return vec![mv | PROMOTION_QUEEN_MOVE_MASK,
                    mv | PROMOTION_ROOK_MOVE_MASK,
                    mv | PROMOTION_BISHOP_MOVE_MASK,
                    mv | PROMOTION_KNIGHT_MOVE_MASK];
    }

    #[inline(always)]
    pub fn generate_pawn_moves_from_to_squares(from_square: Square, to_bitboard: Bitboard) -> MoveList {
        let mask = from_square_mask(from_square);
        let to_squares = bit_list(to_bitboard);
        let mut move_list = Vec::new();
        for to_square in to_squares {
            let base_move = mask | to_square as Move;
            if to_square >= 56 || to_square <= 7 {
                for mv in promotion_moves(base_move) {
                    move_list.push(mv);
                }
            } else {
                move_list.push(base_move);
            }
        }
        return move_list;
    }

    #[inline(always)]
    pub fn pawn_captures(lookup: &[Bitboard], square: Square, enemy_bitboard: Bitboard) -> Bitboard {
        return lookup[square as usize] & enemy_bitboard;
    }

    #[inline(always)]
    pub fn potential_pawn_jump_moves(bb: Bitboard, position: &Position) -> Bitboard {
        return if position.mover == White {
            (bb << 8) & RANK_4_BITS
        } else {
            (bb >> 8) & RANK_5_BITS
        }
    }

    #[inline(always)]
    pub fn pawn_forward_moves_bitboard(pawn_moves: Bitboard, position: &Position) -> Bitboard {
        return pawn_moves | (potential_pawn_jump_moves(pawn_moves, &position) & empty_squares_bitboard(&position));
    }

    #[inline(always)]
    pub fn pawn_forward_and_capture_moves_bitboard(from_square: Square, capture_pawn_moves: &[Bitboard], non_captures: Bitboard, position: &Position) -> Bitboard {
        let eps = position.en_passant_square;
        let captures = if eps != EN_PASSANT_NOT_AVAILABLE && bit(eps) & en_passant_capture_rank(&position.mover) != 0 {
            pawn_captures_plus_en_passant_square(capture_pawn_moves, from_square, &position)
        } else {
            pawn_captures(capture_pawn_moves, from_square, enemy_bitboard(&position))
        };
        return non_captures | captures;
    }

    #[inline(always)]
    pub fn pawn_captures_plus_en_passant_square(capture_pawn_moves: &[Bitboard], square: Square, position: &Position) -> Bitboard {
        let eps = position.en_passant_square;
        return pawn_captures(capture_pawn_moves, square, enemy_bitboard(&position) | if eps == EN_PASSANT_NOT_AVAILABLE { 0 } else { bit(eps) })
    }

    #[inline(always)]
    pub fn en_passant_capture_rank(mover: &Mover) -> Bitboard {
        return if *mover == White { RANK_6_BITS } else { RANK_3_BITS }
    }

    #[inline(always)]
    pub fn generate_pawn_moves(position: &Position) -> MoveList {
        let bitboard = bitboard_for_mover(&position, &Pawn);
        return if position.mover == White {
            generate_white_pawn_moves(bitboard, position, empty_squares_bitboard(&position))
        } else {
            generate_black_pawn_moves(bitboard, position, empty_squares_bitboard(&position))
        }
    }

    #[inline(always)]
    pub fn generate_white_pawn_moves(from_squares: Bitboard, position: &Position, empty_squares: Bitboard) -> MoveList {
        let mut move_list = Vec::new();

        for from_square in bit_list(from_squares) {
            let pawn_forward_and_capture_moves = pawn_forward_and_capture_moves_bitboard(
                from_square as Square,
                WHITE_PAWN_MOVES_CAPTURE,
                pawn_forward_moves_bitboard(WHITE_PAWN_MOVES_FORWARD[from_square as usize] & empty_squares, &position),
                &position
            );
            let mut ms = generate_pawn_moves_from_to_squares(from_square as Square, pawn_forward_and_capture_moves);
            move_list.append(ms.as_mut());
        };

        return move_list;
    }

    #[inline(always)]
    pub fn generate_black_pawn_moves(from_squares: Bitboard, position: &Position, empty_squares: Bitboard) -> MoveList {
        let mut move_list = Vec::new();

        for from_square in bit_list(from_squares) {
            let pawn_forward_and_capture_moves = pawn_forward_and_capture_moves_bitboard(
                from_square as Square,
                BLACK_PAWN_MOVES_CAPTURE,
                pawn_forward_moves_bitboard(BLACK_PAWN_MOVES_FORWARD[from_square as usize] & empty_squares, &position),
                &position
            );
            let mut ms = generate_pawn_moves_from_to_squares(from_square as Square, pawn_forward_and_capture_moves);
            move_list.append(ms.as_mut());
        };

        return move_list;
    }

    #[inline(always)]
    pub fn any_squares_in_bitboard_attacked(position: &Position, attacker: &Mover, bitboard: Bitboard) -> bool {
        return if bitboard == 0 {
            false
        } else {
            let square = bitboard.trailing_zeros() as Square;
            is_square_attacked_by(position, square, attacker) ||
                any_squares_in_bitboard_attacked(position, attacker, clear_bit(bitboard, square))
        }
    }

    #[inline(always)]
    pub fn pawn_moves_capture_of_colour(mover: Mover, square: Square) -> Bitboard {
        return if mover == White {
            WHITE_PAWN_MOVES_CAPTURE[square as usize]
        } else {
            BLACK_PAWN_MOVES_CAPTURE[square as usize]
        }
    }

    #[inline(always)]
    pub fn is_square_attacked_by(position: &Position, attacked_square: Square, mover: &Mover) -> bool {
        let all_pieces = position.all_pieces_bitboard;
        return if *mover == White {
                is_square_attacked_by_any_pawn(position.white_pawn_bitboard, pawn_moves_capture_of_colour(Black, attacked_square)) ||
                is_square_attacked_by_any_knight(position.white_knight_bitboard, attacked_square) ||
                is_square_attacked_by_any_rook(all_pieces, rook_move_pieces_bitboard(position, White), attacked_square) ||
                is_square_attacked_by_any_bishop(all_pieces, bishop_move_pieces_bitboard(position, White), attacked_square) ||
                is_square_attacked_by_king(position.white_king_bitboard, attacked_square)
        } else {
                is_square_attacked_by_any_pawn(position.black_pawn_bitboard, pawn_moves_capture_of_colour(White, attacked_square)) ||
                is_square_attacked_by_any_knight(position.black_knight_bitboard, attacked_square) ||
                is_square_attacked_by_any_rook(all_pieces, rook_move_pieces_bitboard(position, Black), attacked_square) ||
                is_square_attacked_by_any_bishop(all_pieces, bishop_move_pieces_bitboard(position, Black), attacked_square) ||
                is_square_attacked_by_king(position.black_king_bitboard, attacked_square)
        }
    }

    #[inline(always)]
    pub fn rook_move_pieces_bitboard(position: &Position, mover: Mover) -> Bitboard {
        return if mover == White {
            position.white_rook_bitboard | position.white_queen_bitboard
        } else {
            position.black_rook_bitboard | position.black_queen_bitboard
        }
    }

    #[inline(always)]
    pub fn bishop_move_pieces_bitboard(position: &Position, mover: Mover) -> Bitboard {
        return if mover == White {
            position.white_bishop_bitboard | position.white_queen_bitboard
        } else {
            position.black_bishop_bitboard | position.black_queen_bitboard
        }
    }

    #[inline(always)]
    pub fn is_square_attacked_by_any_knight(knight_bitboard: Bitboard, attacked_square: Square) -> bool {
        return knight_bitboard & KNIGHT_MOVES_BITBOARDS[attacked_square as usize] != 0;
    }

    #[inline(always)]
    pub fn is_square_attacked_by_king(king_bitboard: Bitboard, attacked_square: Square) -> bool {
        return king_bitboard & KING_MOVES_BITBOARDS[attacked_square as usize] != 0;
    }

    #[inline(always)]
    pub fn is_square_attacked_by_any_pawn(pawns: Bitboard, pawn_attacks: Bitboard) -> bool {
        return pawns & pawn_attacks != 0;
    }

    #[inline(always)]
    pub fn is_square_attacked_by_any_bishop(all_pieces: Bitboard, attacking_bishops: Bitboard, attacked_square: Square) -> bool {
        return if attacking_bishops == 0 {
            false
        } else {
            let bishop_square = attacking_bishops.trailing_zeros();
            is_bishop_attacking_square(attacked_square, bishop_square as Square, all_pieces) ||
            is_square_attacked_by_any_bishop(all_pieces, clear_bit(attacking_bishops, bishop_square as Square), attacked_square)
        }
    }

    #[inline(always)]
    pub fn is_square_attacked_by_any_rook(all_pieces: Bitboard, attacking_rooks: Bitboard, attacked_square: Square) -> bool {
        return if attacking_rooks == 0 {
            false
        } else {
            let rook_square = attacking_rooks.trailing_zeros();
            is_rook_attacking_square(attacked_square, rook_square as Square, all_pieces) ||
            is_square_attacked_by_any_rook(all_pieces, clear_bit(attacking_rooks, rook_square as Square), attacked_square)
        }
    }

    #[inline(always)]
    pub fn is_bishop_attacking_square(attacked_square: Square, piece_square: Square, all_pieces_bitboard: Bitboard) -> bool {
        return test_bit(magic_bishop(piece_square, magic_index_for_bishop(piece_square, all_pieces_bitboard)), attacked_square);
    }

    #[inline(always)]
    pub fn is_rook_attacking_square(attacked_square: Square, piece_square: Square, all_pieces_bitboard: Bitboard) -> bool {
        return test_bit(magic_rook(piece_square, magic_index_for_rook(piece_square, all_pieces_bitboard)), attacked_square);
    }

    #[inline(always)]
    pub fn generate_castle_moves(position: &Position) -> MoveList {
        let all_pieces = position.all_pieces_bitboard;
        let mut move_list = Vec::new();
        if position.mover == White {
            if position.white_king_castle_available && all_pieces & EMPTY_CASTLE_SQUARES_WHITE_KING == 0 && !any_squares_in_bitboard_attacked(position, &Black, NO_CHECK_CASTLE_SQUARES_WHITE_KING) {
                move_list.push(from_square_mask(3) | 1);
            }
            if position.white_queen_castle_available && all_pieces & EMPTY_CASTLE_SQUARES_WHITE_QUEEN == 0 && !any_squares_in_bitboard_attacked(position, &Black, NO_CHECK_CASTLE_SQUARES_WHITE_QUEEN) {
                move_list.push(from_square_mask(3) | 5);
            }
        } else {
            if position.black_king_castle_available && all_pieces & EMPTY_CASTLE_SQUARES_BLACK_KING == 0 && !any_squares_in_bitboard_attacked(position, &White, NO_CHECK_CASTLE_SQUARES_BLACK_KING) {
                move_list.push(from_square_mask(59) | 57);
            }
            if position.black_queen_castle_available && all_pieces & EMPTY_CASTLE_SQUARES_BLACK_QUEEN == 0 && !any_squares_in_bitboard_attacked(position, &White, NO_CHECK_CASTLE_SQUARES_BLACK_QUEEN) {
                move_list.push(from_square_mask(59) | 61);
            }
        }
        return move_list;
    }

    #[inline(always)]
    pub fn moves(position: &Position) -> MoveList {
        let mut move_list = Vec::new();
        move_list.append(generate_pawn_moves(position).as_mut());
        move_list.append(generate_king_moves(position).as_mut());
        move_list.append(generate_castle_moves(position).as_mut());
        move_list.append(generate_knight_moves(position).as_mut());
        move_list.append(generate_slider_moves(position, Rook).as_mut());
        move_list.append(generate_slider_moves(position, Bishop).as_mut());
        return move_list;
    }

    #[inline(always)]
    pub fn king_square(position: &Position, mover: Mover) -> Square {
        return if mover == White {
            position.white_king_bitboard.trailing_zeros() as Square
        } else {
            position.black_king_bitboard.trailing_zeros() as Square
        }
    }

    #[inline(always)]
    pub fn is_check(position: &Position, mover: &Mover) -> bool {
        return if *mover == White {
            is_square_attacked_by(position, king_square(position, White), &Black)
        } else {
            is_square_attacked_by(position, king_square(position, Black), &White)
        }
    }

    #[inline(always)]
    pub fn move_piece_within_bitboard(from: Square, to: Square, bb: Bitboard) -> Bitboard {
        if test_bit(bb, from) {
            clear_bit(bb, from) | bit(to)
        } else {
            clear_bit(bb, to)
        }
    }
}
