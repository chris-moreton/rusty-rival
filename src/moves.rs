use crate::bitboards::{bit, BLACK_PAWN_MOVES_CAPTURE, BLACK_PAWN_MOVES_FORWARD, EMPTY_CASTLE_SQUARES_BLACK_KING, EMPTY_CASTLE_SQUARES_BLACK_QUEEN, EMPTY_CASTLE_SQUARES_WHITE_KING, EMPTY_CASTLE_SQUARES_WHITE_QUEEN, enemy_bitboard, KING_MOVES_BITBOARDS, KNIGHT_MOVES_BITBOARDS, NO_CHECK_CASTLE_SQUARES_BLACK_KING, NO_CHECK_CASTLE_SQUARES_BLACK_QUEEN, NO_CHECK_CASTLE_SQUARES_WHITE_KING, NO_CHECK_CASTLE_SQUARES_WHITE_QUEEN, RANK_3_BITS, RANK_4_BITS, RANK_5_BITS, RANK_6_BITS, test_bit, WHITE_PAWN_MOVES_CAPTURE, WHITE_PAWN_MOVES_FORWARD};
use crate::magic_bitboards::{magic_moves, MAGIC_BOX};
use crate::move_constants::{BLACK_KING_CASTLE_MOVE, BLACK_QUEEN_CASTLE_MOVE, EN_PASSANT_NOT_AVAILABLE, PROMOTION_BISHOP_MOVE_MASK, PROMOTION_KNIGHT_MOVE_MASK, PROMOTION_QUEEN_MOVE_MASK, PROMOTION_ROOK_MOVE_MASK, PROMOTION_SQUARES, WHITE_KING_CASTLE_MOVE, WHITE_QUEEN_CASTLE_MOVE};
use crate::types::{Bitboard, BLACK, is_bk_castle_available, is_bq_castle_available, is_wk_castle_available, is_wq_castle_available, MagicVars, Move, MoveList, Mover, Position, Square, WHITE};
use crate::{opponent, unset_lsb};
use crate::utils::from_square_mask;

#[inline(always)]
pub fn moves(position: &Position) -> MoveList {
    if position.mover == WHITE {
        white_moves(position)
    } else {
        black_moves(position)
    }
}

#[inline(always)]
pub fn white_moves(position: &Position) -> MoveList {
    let mut move_list = Vec::with_capacity(140);
    let valid_destinations = !position.white_pieces_bitboard;
    let all_pieces = position.all_pieces_bitboard;

    generate_white_pawn_moves(position, &mut move_list);

    let from_square = position.white_king_square;
    let fsm = from_square_mask(from_square as Square);
    let mut to_bitboard = KING_MOVES_BITBOARDS[from_square as usize] & valid_destinations;
    while to_bitboard != 0 {
        let sq = to_bitboard.trailing_zeros() as Square;
        move_list.push(fsm | sq as Move);
        unset_lsb!(to_bitboard);
    };

    if is_wk_castle_available(position) && all_pieces & EMPTY_CASTLE_SQUARES_WHITE_KING == 0 &&
        !any_squares_in_bitboard_attacked(position, BLACK, NO_CHECK_CASTLE_SQUARES_WHITE_KING) {
        move_list.push(WHITE_KING_CASTLE_MOVE);
    }
    if is_wq_castle_available(position) && all_pieces & EMPTY_CASTLE_SQUARES_WHITE_QUEEN == 0 &&
        !any_squares_in_bitboard_attacked(position, BLACK, NO_CHECK_CASTLE_SQUARES_WHITE_QUEEN) {
        move_list.push(WHITE_QUEEN_CASTLE_MOVE);
    }

    let mut from_squares_bitboard = position.white_knight_bitboard;
    while from_squares_bitboard != 0 {
        let from_square = from_squares_bitboard.trailing_zeros() as Square;
        let fsm = from_square_mask(from_square);
        let mut to_bitboard = KNIGHT_MOVES_BITBOARDS[from_square as usize] & valid_destinations;
        while to_bitboard != 0 {
            move_list.push(fsm | to_bitboard.trailing_zeros() as Move);
            unset_lsb!(to_bitboard);
        }
        unset_lsb!(from_squares_bitboard);
    }

    generate_slider_moves(position.white_rook_bitboard | position.white_queen_bitboard, position.all_pieces_bitboard, &mut move_list, &MAGIC_BOX.rook, valid_destinations);
    generate_slider_moves(position.white_bishop_bitboard | position.white_queen_bitboard, position.all_pieces_bitboard, &mut move_list, &MAGIC_BOX.bishop, valid_destinations);

    move_list
}

#[inline(always)]
pub fn black_moves(position: &Position) -> MoveList {
    let mut move_list = Vec::with_capacity(80);
    let valid_destinations = !position.black_pieces_bitboard;
    let all_pieces = position.all_pieces_bitboard;

    generate_black_pawn_moves(position, &mut move_list);

    let from_square = position.black_king_square;
    let fsm = from_square_mask(from_square as Square);
    let mut to_bitboard = KING_MOVES_BITBOARDS[from_square as usize] & valid_destinations;
    while to_bitboard != 0 {
        let sq = to_bitboard.trailing_zeros() as Square;
        move_list.push(fsm | sq as Move);
        unset_lsb!(to_bitboard);
    };

    if is_bk_castle_available(position) && all_pieces & EMPTY_CASTLE_SQUARES_BLACK_KING == 0 &&
        !any_squares_in_bitboard_attacked(position, WHITE, NO_CHECK_CASTLE_SQUARES_BLACK_KING) {
        move_list.push(BLACK_KING_CASTLE_MOVE);
    }
    if is_bq_castle_available(position) && all_pieces & EMPTY_CASTLE_SQUARES_BLACK_QUEEN == 0 &&
        !any_squares_in_bitboard_attacked(position, WHITE, NO_CHECK_CASTLE_SQUARES_BLACK_QUEEN) {
        move_list.push(BLACK_QUEEN_CASTLE_MOVE);
    }

    let mut from_squares_bitboard = position.black_knight_bitboard;
    while from_squares_bitboard != 0 {
        let from_square = from_squares_bitboard.trailing_zeros() as Square;
        let fsm = from_square_mask(from_square);
        let mut to_bitboard = KNIGHT_MOVES_BITBOARDS[from_square as usize] & valid_destinations;
        while to_bitboard != 0 {
            move_list.push(fsm | to_bitboard.trailing_zeros() as Move);
            unset_lsb!(to_bitboard);
        }
        unset_lsb!(from_squares_bitboard);
    }

    generate_slider_moves(position.black_rook_bitboard | position.black_queen_bitboard, position.all_pieces_bitboard, &mut move_list, &MAGIC_BOX.rook, valid_destinations);
    generate_slider_moves(position.black_bishop_bitboard | position.black_queen_bitboard, position.all_pieces_bitboard, &mut move_list, &MAGIC_BOX.bishop, valid_destinations);

    move_list
}

#[inline(always)]
pub fn generate_slider_moves(slider_bitboard: Bitboard, all_pieces_bitboard: Bitboard, move_list: &mut MoveList, magic_vars: &Box<MagicVars>, valid_destinations: Bitboard) {
    let mut from_bitboard = slider_bitboard;
    while from_bitboard != 0 {
        let from_square = from_bitboard.trailing_zeros() as Square;
        let fsm = from_square_mask(from_square);

        let mut to_bitboard = magic_moves(from_square,all_pieces_bitboard, magic_vars) & valid_destinations;

        while to_bitboard != 0 {
            move_list.push(fsm | to_bitboard.trailing_zeros() as Move);
            unset_lsb!(to_bitboard);
        }
        unset_lsb!(from_bitboard);
    };
}

#[inline(always)]
pub fn generate_pawn_moves_from_to_squares(from_square: Square, mut to_bitboard: Bitboard, move_list: &mut MoveList) {
    let mask = from_square_mask(from_square);
    while to_bitboard != 0 {
        let base_move = mask | to_bitboard.trailing_zeros() as Move;
        if to_bitboard & PROMOTION_SQUARES != 0 {
            move_list.push(base_move | PROMOTION_QUEEN_MOVE_MASK);
            move_list.push(base_move | PROMOTION_ROOK_MOVE_MASK);
            move_list.push(base_move | PROMOTION_BISHOP_MOVE_MASK);
            move_list.push(base_move | PROMOTION_KNIGHT_MOVE_MASK);
        } else {
            move_list.push(base_move);
        }
        unset_lsb!(to_bitboard);
    };
}

#[inline(always)]
pub fn pawn_captures(lookup: &[Bitboard], square: Square, enemy_bitboard: Bitboard) -> Bitboard {
    lookup[square as usize] & enemy_bitboard
}

#[inline(always)]
pub fn pawn_forward_and_capture_moves_bitboard(from_square: Square, capture_pawn_moves: &[Bitboard], non_captures: Bitboard, position: &Position) -> Bitboard {
    let eps = position.en_passant_square;
    let captures = if eps != EN_PASSANT_NOT_AVAILABLE && bit(eps) & en_passant_capture_rank(&position.mover) != 0 {
        pawn_captures_plus_en_passant_square(capture_pawn_moves, from_square, position)
    } else {
        pawn_captures(capture_pawn_moves, from_square, enemy_bitboard(position))
    };
    non_captures | captures
}

#[inline(always)]
pub fn pawn_captures_plus_en_passant_square(capture_pawn_moves: &[Bitboard], square: Square, position: &Position) -> Bitboard {
    let eps = position.en_passant_square;
    pawn_captures(capture_pawn_moves, square, enemy_bitboard(position) | if eps == EN_PASSANT_NOT_AVAILABLE { 0 } else { bit(eps) })
}

#[inline(always)]
pub fn en_passant_capture_rank(mover: &Mover) -> Bitboard {
    if *mover == WHITE { RANK_6_BITS } else { RANK_3_BITS }
}

#[inline(always)]
pub fn generate_white_pawn_moves(position: &Position, move_list: &mut MoveList) {

    let mut from_squares = position.white_pawn_bitboard;
    let empty_squares = !position.all_pieces_bitboard;

    while from_squares != 0 {
        let from_square = from_squares.trailing_zeros();
        let pawn_moves = WHITE_PAWN_MOVES_FORWARD[from_square as usize] & empty_squares;
        let bb = pawn_moves;
        let pawn_forward_and_capture_moves = pawn_forward_and_capture_moves_bitboard(
            from_square as Square,
            WHITE_PAWN_MOVES_CAPTURE,
            pawn_moves | ((bb << 8) & RANK_4_BITS & !position.all_pieces_bitboard),
            position
        );
        generate_pawn_moves_from_to_squares(from_square as Square, pawn_forward_and_capture_moves, move_list);
        unset_lsb!(from_squares);
    };
}

#[inline(always)]
pub fn generate_black_pawn_moves(position: &Position, move_list: &mut MoveList) {

    let mut from_squares = position.black_pawn_bitboard;
    let empty_squares = !position.all_pieces_bitboard;

    while from_squares != 0 {
        let from_square = from_squares.trailing_zeros();
        let pawn_moves = BLACK_PAWN_MOVES_FORWARD[from_square as usize] & empty_squares;
        let bb = pawn_moves;
        let pawn_forward_and_capture_moves = pawn_forward_and_capture_moves_bitboard(
            from_square as Square,
            BLACK_PAWN_MOVES_CAPTURE,
            pawn_moves | ((bb >> 8) & RANK_5_BITS & !position.all_pieces_bitboard),
            position
        );
        generate_pawn_moves_from_to_squares(from_square as Square, pawn_forward_and_capture_moves, move_list);
        unset_lsb!(from_squares);
    };
}

#[inline(always)]
pub fn any_squares_in_bitboard_attacked(position: &Position, attacker: Mover, mut bitboard: Bitboard) -> bool {
    while bitboard != 0 {
        if is_square_attacked_by(position, bitboard.trailing_zeros() as Square, attacker) { return true }
        unset_lsb!(bitboard);
    }
    return false;
}

#[inline(always)]
pub fn is_square_attacked_by(position: &Position, attacked_square: Square, mover: Mover) -> bool {
    let all_pieces = position.all_pieces_bitboard;
    if mover == WHITE {
        position.white_pawn_bitboard & BLACK_PAWN_MOVES_CAPTURE[attacked_square as usize] != 0 ||
            position.white_knight_bitboard & KNIGHT_MOVES_BITBOARDS[attacked_square as usize] != 0 ||
            is_square_attacked_by_slider_of_type(all_pieces, position.white_rook_bitboard | position.white_queen_bitboard, attacked_square, &MAGIC_BOX.rook) ||
            is_square_attacked_by_slider_of_type(all_pieces, position.white_bishop_bitboard | position.white_queen_bitboard, attacked_square, &MAGIC_BOX.bishop) ||
            bit(position.white_king_square) & KING_MOVES_BITBOARDS[attacked_square as usize] != 0
    } else {
        position.black_pawn_bitboard & WHITE_PAWN_MOVES_CAPTURE[attacked_square as usize] != 0 ||
            position.black_knight_bitboard & KNIGHT_MOVES_BITBOARDS[attacked_square as usize] != 0 ||
            is_square_attacked_by_slider_of_type(all_pieces, position.black_rook_bitboard | position.black_queen_bitboard, attacked_square, &MAGIC_BOX.rook) ||
            is_square_attacked_by_slider_of_type(all_pieces, position.black_bishop_bitboard | position.black_queen_bitboard, attacked_square, &MAGIC_BOX.bishop) ||
            bit(position.black_king_square) & KING_MOVES_BITBOARDS[attacked_square as usize] != 0
    }
}

#[inline(always)]
pub fn is_square_attacked_by_slider_of_type(all_pieces: Bitboard, mut attacking_sliders: Bitboard, attacked_square: Square, magic_vars: &Box<MagicVars>) -> bool {
    while attacking_sliders != 0 {
        if test_bit(magic_moves(attacking_sliders.trailing_zeros() as Square,all_pieces, magic_vars), attacked_square) { return true };
        unset_lsb!(attacking_sliders)
    }
    return false;
}

#[inline(always)]
pub fn king_square(position: &Position, mover: Mover) -> Square {
    if mover == WHITE {
        position.white_king_square
    } else {
        position.black_king_square
    }
}

#[inline(always)]
pub fn is_check(position: &Position, mover: Mover) -> bool {
    is_square_attacked_by(position, king_square(position, mover), opponent!(mover))
}
