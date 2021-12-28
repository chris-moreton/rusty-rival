pub mod moves {
    use crate::bitboards::bitboards::{bit, bit_list, bitboard_for_mover, BLACK_PAWN_MOVES_CAPTURE, BLACK_PAWN_MOVES_FORWARD, clear_bit, empty_squares_bitboard, enemy_bitboard, KING_MOVES_BITBOARDS, KNIGHT_MOVES_BITBOARDS, RANK_3_BITS, RANK_4_BITS, RANK_5_BITS, RANK_6_BITS, slider_bitboard_for_colour, WHITE_PAWN_MOVES_CAPTURE, WHITE_PAWN_MOVES_FORWARD};
    use crate::magic_bitboards::magic_bitboards::{magic, MAGIC_BISHOP_VARS, MAGIC_ROOK_VARS};
    use crate::move_constants::move_constants::{EN_PASSANT_NOT_AVAILABLE, PROMOTION_BISHOP_MOVE_MASK, PROMOTION_KNIGHT_MOVE_MASK, PROMOTION_QUEEN_MOVE_MASK, PROMOTION_ROOK_MOVE_MASK};
    use crate::types::types::{Bitboard, Move, MoveList, Mover, Piece, Position, Square};
    use crate::types::types::Mover::{Black, White};
    use crate::types::types::Piece::{Bishop, King, Knight, Pawn};
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

    pub fn potential_pawn_jump_moves(bb: Bitboard, position: &Position) -> Bitboard {
        return if position.mover == White {
            (bb << 8) & RANK_4_BITS
        } else {
            (bb >> 8) & RANK_5_BITS
        }
    }

    pub fn pawn_forward_moves_bitboard(pawn_moves: Bitboard, position: &Position) -> Bitboard {
        return pawn_moves | (potential_pawn_jump_moves(pawn_moves, &position) & empty_squares_bitboard(&position));
    }

    pub fn pawn_forward_and_capture_moves_bitboard(from_square: Square, capture_pawn_moves: &[Bitboard], non_captures: Bitboard, position: &Position) -> Bitboard {
        let eps = position.en_passant_square;
        let captures = if eps != EN_PASSANT_NOT_AVAILABLE && bit(eps) & en_passant_capture_rank(&position.mover) != 0 {
            pawn_captures_plus_en_passant_square(capture_pawn_moves, from_square, &position)
        } else {
            pawn_captures(capture_pawn_moves, from_square, enemy_bitboard(&position))
        };
        return non_captures | captures;
    }

    pub fn pawn_captures_plus_en_passant_square(capture_pawn_moves: &[Bitboard], square: Square, position: &Position) -> Bitboard {
        let eps = position.en_passant_square;
        return pawn_captures(capture_pawn_moves, square, enemy_bitboard(&position) | if eps == EN_PASSANT_NOT_AVAILABLE { 0 } else { bit(eps) })
    }

    pub fn en_passant_capture_rank(mover: &Mover) -> Bitboard {
        return if *mover == White { RANK_6_BITS } else { RANK_3_BITS }
    }

    pub fn generate_pawn_moves(position: &Position) -> MoveList {
        let bitboard = bitboard_for_mover(&position, &Pawn);
        return if position.mover == White {
            generate_white_pawn_moves(bitboard, position, empty_squares_bitboard(&position))
        } else {
            generate_black_pawn_moves(bitboard, position, empty_squares_bitboard(&position))
        }
    }

    pub fn generate_white_pawn_moves(from_squares: Bitboard, position: &Position, empty_squares: Bitboard) -> MoveList {
        let mut move_list = Vec::new();

        bit_list(from_squares).iter().for_each(|from_square| {
            let pawn_forward_and_capture_moves = pawn_forward_and_capture_moves_bitboard(
                *from_square as Square,
                WHITE_PAWN_MOVES_CAPTURE,
                pawn_forward_moves_bitboard(WHITE_PAWN_MOVES_FORWARD.iter().nth(*from_square as usize).unwrap() & empty_squares, &position),
                &position
            );
            let mut ms = generate_pawn_moves_from_to_squares(*from_square as Square, pawn_forward_and_capture_moves);
            move_list.append(ms.as_mut());
        });

        return move_list;
    }

    pub fn generate_black_pawn_moves(from_squares: Bitboard, position: &Position, empty_squares: Bitboard) -> MoveList {
        let mut move_list = Vec::new();

        bit_list(from_squares).iter().for_each(|from_square| {
            let pawn_forward_and_capture_moves = pawn_forward_and_capture_moves_bitboard(
                *from_square as Square,
                BLACK_PAWN_MOVES_CAPTURE,
                pawn_forward_moves_bitboard(BLACK_PAWN_MOVES_FORWARD.iter().nth(*from_square as usize).unwrap() & empty_squares, &position),
                &position
            );
            let mut ms = generate_pawn_moves_from_to_squares(*from_square as Square, pawn_forward_and_capture_moves);
            move_list.append(ms.as_mut());
        });

        return move_list;
    }

    pub fn any_squares_in_bitboard_attacked(position: &Position, attacker: Mover, bitboard: Bitboard) -> bool {
        let square = bitboard.trailing_zeros() as Square;
        return is_square_attacked_by(position, square, attacked) || any_squares_in_bitboard_attacked(position, attacker, clear_bit(bitboard, square));
    }

    pub fn pawn_moves_capture_of_colour(mover: Mover, square: Square) -> &Bitboard {
        return if mover == White {
            WHITE_PAWN_MOVES_CAPTURE.iter().nth(square as usize).unwrap()
        } else {
            BLACK_PAWN_MOVES_CAPTURE.iter().nth(square as usize).unwrap()
        }
    }

    pub fn is_square_attacked_by(position: &Position, attackedSquare: Square, mover: Mover) -> bool {
        let all_pieces = position.all_pieces_bitboard;
        return if mover == White {
                is_square_attacked_by_any_pawn(position.white_pawn_bitboard, pawn_moves_capture_of_colour(Black, attackedSquare), attackedSquare) ||
                is_square_attacked_by_any_knight(position.white_knight_bitboard, attackedSquare) ||
                is_square_attacked_by_any_rook(all_pieces, rook_move_pieces_bitboard(position, White), attackedSquare) ||
                is_square_attacked_by_any_bishop(all_pieces, bishop_move_pieces_bitboard(position, White), attackedSquare) ||
                is_square_attacked_by_king(position.white_king_bitboard, attackedSquare)
        } else {
                is_square_attacked_by_any_pawn(position.black_pawn_bitboard, pawn_moves_capture_of_colour(Black, attackedSquare), attackedSquare) ||
                is_square_attacked_by_any_knight(position.black_knight_bitboard, attackedSquare) ||
                is_square_attacked_by_any_rook(all_pieces, rook_move_pieces_bitboard(position, Black), attackedSquare) ||
                is_square_attacked_by_any_bishop(all_pieces, bishop_move_pieces_bitboard(position, Black), attackedSquare) ||
                is_square_attacked_by_king(position.black_king_bitboard, attackedSquare)
        }
    }


    //
    // {-# INLINE isSquareAttackedByAnyKnight #-}
    // isSquareAttackedByAnyKnight :: Bitboard -> Square -> Bool
    // isSquareAttackedByAnyKnight 0 _ = False
    // isSquareAttackedByAnyKnight !knightBitboard !attackedSquare = (.&.) knightBitboard (knightMovesBitboards attackedSquare) /= 0
    //
    // {-# INLINE isSquareAttackedByKing #-}
    // isSquareAttackedByKing :: Bitboard -> Square -> Bool
    // isSquareAttackedByKing !king !attackedSquare = (.&.) king (kingMovesBitboards attackedSquare) /= 0
    //
    // {-# INLINE isSquareAttackedByAnyPawn #-}
    // isSquareAttackedByAnyPawn :: Bitboard -> Bitboard -> Square -> Bool
    // isSquareAttackedByAnyPawn 0 _ _ = False
    // isSquareAttackedByAnyPawn !pawns !pawnAttacks !attackedSquare = (.&.) pawns pawnAttacks /= 0
    //
    // {-# INLINE isSquareAttackedByAnyBishop #-}
    // isSquareAttackedByAnyBishop :: Bitboard -> Bitboard -> Square -> Bool
    // isSquareAttackedByAnyBishop _ 0 _ = False
    // isSquareAttackedByAnyBishop !allPieces !attackingBishops !attackedSquare =
    // isBishopAttackingSquare attackedSquare bishopSquare allPieces ||
    // isSquareAttackedByAnyBishop allPieces (clearBit attackingBishops bishopSquare) attackedSquare
    // where bishopSquare = countTrailingZeros attackingBishops
    //
    // {-# INLINE isSquareAttackedByAnyRook #-}
    // isSquareAttackedByAnyRook :: Bitboard -> Bitboard -> Square -> Bool
    // isSquareAttackedByAnyRook _ 0 _ = False
    // isSquareAttackedByAnyRook !allPieces !attackingRooks !attackedSquare =
    // isRookAttackingSquare attackedSquare rookSquare allPieces ||
    // isSquareAttackedByAnyRook allPieces (clearBit attackingRooks rookSquare) attackedSquare
    // where rookSquare = countTrailingZeros attackingRooks
    //
    //
    // {-# INLINE isBishopAttackingSquare #-}
    // isBishopAttackingSquare :: Square -> Square -> Bitboard -> Bool
    // isBishopAttackingSquare !attackedSquare !pieceSquare !allPieceBitboard =
    // testBit (magicBishop pieceSquare (magicIndexForBishop pieceSquare allPieceBitboard)) attackedSquare
    //
    // {-# INLINE isRookAttackingSquare #-}
    // isRookAttackingSquare :: Square -> Square -> Bitboard -> Bool
    // isRookAttackingSquare !attackedSquare !pieceSquare !allPieceBitboard =
    // testBit (magicRook pieceSquare (magicIndexForRook pieceSquare allPieceBitboard)) attackedSquare

}
