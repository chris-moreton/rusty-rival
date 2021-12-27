pub mod moves {
    use crate::bitboards::bitboards::{bit_list, bitboard_for_mover, KING_MOVES_BITBOARDS, KNIGHT_MOVES_BITBOARDS, RANK_4_BITS, RANK_5_BITS, slider_bitboard_for_colour};
    use crate::magic_bitboards::magic_bitboards::{magic, MAGIC_BISHOP_VARS, MAGIC_ROOK_VARS};
    use crate::move_constants::move_constants::{PROMOTION_BISHOP_MOVE_MASK, PROMOTION_KNIGHT_MOVE_MASK, PROMOTION_QUEEN_MOVE_MASK, PROMOTION_ROOK_MOVE_MASK};
    use crate::types::types::{Bitboard, Move, MoveList, Piece, Position, Square};
    use crate::types::types::Mover::White;
    use crate::types::types::Piece::{Bishop, King, Knight};
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

    pub fn generate_king_moves(position: &Position) -> MoveList {
        let valid_destinations = all_bits_except_friendly_pieces(position);
        let from_square = bitboard_for_mover(position, &King).trailing_zeros();
        let mut move_list = Vec::new();
        let to_squares = bit_list(KING_MOVES_BITBOARDS[from_square as usize] & valid_destinations);
        to_squares.iter().for_each(|to_square| {
            move_list.push(from_square_mask(from_square as i8) | *to_square as u32);
        });
        return move_list;
    }

    pub fn generate_slider_moves(position: &Position, piece: Piece) -> MoveList {
        return generate_slider_moves_with_targets(position, piece, all_bits_except_friendly_pieces(position));
    }

    pub fn generate_slider_moves_with_targets(position: &Position, piece: Piece, valid_destinations: Bitboard) -> MoveList {
        let from_squares = bit_list(slider_bitboard_for_colour(position, &position.mover, &piece));
        let mut move_list = Vec::new();
        from_squares.iter().for_each(|from_square| {
            let magic_vars = if piece == Bishop { &MAGIC_BISHOP_VARS } else { &MAGIC_ROOK_VARS };
            let number_magic = magic_vars.magic_number.iter().nth(*from_square as usize).unwrap();
            let shift_magic = magic_vars.magic_number_shifts.iter().nth(*from_square as usize).unwrap();
            let mask_magic = magic_vars.occupancy_mask.iter().nth(*from_square as usize).unwrap();
            let occupancy = position.all_pieces_bitboard & mask_magic;
            let raw_index: u64 = (0b1111111111111111111111111111111111111111111111111111111111111111 & ((occupancy as u128 * *number_magic as u128) as u128)) as u64;
            let to_squares_magic_index = raw_index >> shift_magic;
            let to_squares = bit_list(magic(magic_vars, *from_square as Square, to_squares_magic_index) & valid_destinations);
            to_squares.iter().for_each(|to_square| {
                move_list.push(from_square_mask(*from_square as i8) | *to_square as u32);
            });
        });
        return move_list;
    }

    pub fn promotion_moves(mv: Move) -> MoveList {
        return vec![mv | PROMOTION_QUEEN_MOVE_MASK,
                    mv | PROMOTION_ROOK_MOVE_MASK,
                    mv | PROMOTION_BISHOP_MOVE_MASK,
                    mv | PROMOTION_KNIGHT_MOVE_MASK];
    }

    pub fn generate_pawn_moves_from_to_squares(from_square: Square, to_bitboard: Bitboard) -> MoveList {
        let mask = from_square_mask(from_square);
        let to_squares = bit_list(to_bitboard);
        let mut move_list = Vec::new();
        to_squares.iter().for_each(|to_square| {
            let base_move = mask | *to_square as Move;
            if *to_square >= 56 || *to_square <= 7 {
                promotion_moves(base_move).iter().for_each(|mv| {
                    move_list.push(*mv);
                })
            } else {
                move_list.push(base_move);
            }
        });
        return move_list;
    }

    pub fn pawn_captures(lookup: &[Bitboard], square: Square, enemy_bitboard: Bitboard) -> Bitboard {
        return lookup.iter().nth(square as usize).unwrap() & enemy_bitboard;
    }

    pub fn potential_pawn_jump_moves(bb: Bitboard, position: Position) -> Bitboard {
        return if position.mover == White {
            (bb << 8) & RANK_4_BITS
        } else {
            (bb >> 8) & RANK_5_BITS
        }
    }

}
