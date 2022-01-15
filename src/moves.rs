use crate::bitboards::{bit, BLACK_PAWN_MOVES_CAPTURE, BLACK_PAWN_MOVES_FORWARD, EMPTY_CASTLE_SQUARES_BLACK_KING, EMPTY_CASTLE_SQUARES_BLACK_QUEEN, EMPTY_CASTLE_SQUARES_WHITE_KING, EMPTY_CASTLE_SQUARES_WHITE_QUEEN, empty_squares_bitboard, enemy_bitboard, KING_MOVES_BITBOARDS, KNIGHT_MOVES_BITBOARDS, NO_CHECK_CASTLE_SQUARES_BLACK_KING, NO_CHECK_CASTLE_SQUARES_BLACK_QUEEN, NO_CHECK_CASTLE_SQUARES_WHITE_KING, NO_CHECK_CASTLE_SQUARES_WHITE_QUEEN, RANK_3_BITS, RANK_4_BITS, RANK_5_BITS, RANK_6_BITS, test_bit, WHITE_PAWN_MOVES_CAPTURE, WHITE_PAWN_MOVES_FORWARD};
use crate::magic_bitboards::{MAGIC_NUMBER_BISHOP, MAGIC_NUMBER_ROOK, MAGIC_NUMBER_SHIFTS_BISHOP, MAGIC_NUMBER_SHIFTS_ROOK, magic_moves, OCCUPANCY_MASK_BISHOP, OCCUPANCY_MASK_ROOK};
use crate::magic_moves_bishop::MAGIC_MOVES_BISHOP;
use crate::magic_moves_rook::MAGIC_MOVES_ROOK;
use crate::move_constants::{BLACK_KING_CASTLE_MOVE, BLACK_QUEEN_CASTLE_MOVE, EN_PASSANT_NOT_AVAILABLE, PROMOTION_BISHOP_MOVE_MASK, PROMOTION_KNIGHT_MOVE_MASK, PROMOTION_QUEEN_MOVE_MASK, PROMOTION_ROOK_MOVE_MASK, PROMOTION_SQUARES, WHITE_KING_CASTLE_MOVE, WHITE_QUEEN_CASTLE_MOVE};
use crate::types::{Bitboard, BLACK, is_bk_castle_available, is_bq_castle_available, is_wk_castle_available, is_wq_castle_available, MagicBox, MagicVars, Move, MoveList, Mover, Position, Square, WHITE};
use crate::{opponent, unset_lsb};
use crate::utils::from_square_mask;

#[inline(always)]
pub fn moves(position: &Position, magic_box: &MagicBox) -> MoveList {
    if position.mover == WHITE {
        white_moves(position, magic_box)
    } else {
        black_moves(position, magic_box)
    }
}

#[inline(always)]
pub fn white_moves(position: &Position, magic_box: &MagicBox) -> MoveList {
    let mut move_list = Vec::with_capacity(80);
    let valid_destinations = !position.white_pieces_bitboard;
    let all_pieces = position.all_pieces_bitboard;

    let bitboard = position.white_pawn_bitboard;
    generate_white_pawn_moves(bitboard, position, empty_squares_bitboard(position), &mut move_list);

    let from_square = position.white_king_square;
    let fsm = from_square_mask(from_square as Square);
    let mut to_bitboard = KING_MOVES_BITBOARDS[from_square as usize] & valid_destinations;
    while to_bitboard != 0 {
        let sq = to_bitboard.trailing_zeros() as Square;
        move_list.push(fsm | sq as Move);
        unset_lsb!(to_bitboard);
    };

    if is_wk_castle_available(position) && all_pieces & EMPTY_CASTLE_SQUARES_WHITE_KING == 0 &&
        !any_squares_in_bitboard_attacked(position, BLACK, NO_CHECK_CASTLE_SQUARES_WHITE_KING, magic_box) {
        move_list.push(WHITE_KING_CASTLE_MOVE);
    }
    if is_wq_castle_available(position) && all_pieces & EMPTY_CASTLE_SQUARES_WHITE_QUEEN == 0 &&
        !any_squares_in_bitboard_attacked(position, BLACK, NO_CHECK_CASTLE_SQUARES_WHITE_QUEEN, magic_box) {
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

    generate_slider_moves(position.white_rook_bitboard | position.white_queen_bitboard, position.all_pieces_bitboard, &mut move_list, &magic_box.rook, valid_destinations);
    generate_slider_moves(position.white_bishop_bitboard | position.white_queen_bitboard, position.all_pieces_bitboard, &mut move_list, &magic_box.bishop, valid_destinations);

    move_list
}

#[inline(always)]
pub fn black_moves(position: &Position, magic_box: &MagicBox) -> MoveList {
    let mut move_list = Vec::with_capacity(80);
    let valid_destinations = !position.black_pieces_bitboard;
    let all_pieces = position.all_pieces_bitboard;

    let bitboard = position.black_pawn_bitboard;
    generate_black_pawn_moves(bitboard, position, empty_squares_bitboard(position), &mut move_list);

    let from_square = position.black_king_square;
    let fsm = from_square_mask(from_square as Square);
    let mut to_bitboard = KING_MOVES_BITBOARDS[from_square as usize] & valid_destinations;
    while to_bitboard != 0 {
        let sq = to_bitboard.trailing_zeros() as Square;
        move_list.push(fsm | sq as Move);
        unset_lsb!(to_bitboard);
    };

    if is_bk_castle_available(position) && all_pieces & EMPTY_CASTLE_SQUARES_BLACK_KING == 0 &&
        !any_squares_in_bitboard_attacked(position, WHITE, NO_CHECK_CASTLE_SQUARES_BLACK_KING, magic_box) {
        move_list.push(BLACK_KING_CASTLE_MOVE);
    }
    if is_bq_castle_available(position) && all_pieces & EMPTY_CASTLE_SQUARES_BLACK_QUEEN == 0 &&
        !any_squares_in_bitboard_attacked(position, WHITE, NO_CHECK_CASTLE_SQUARES_BLACK_QUEEN, magic_box) {
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

    generate_slider_moves(position.black_rook_bitboard | position.black_queen_bitboard, position.all_pieces_bitboard, &mut move_list, &magic_box.rook, valid_destinations);
    generate_slider_moves(position.black_bishop_bitboard | position.black_queen_bitboard, position.all_pieces_bitboard, &mut move_list, &magic_box.bishop, valid_destinations);

    move_list
}

#[inline(always)]
pub fn all_bits_except_friendly_pieces(position: &Position) -> Bitboard {
    !if position.mover == WHITE { position.white_pieces_bitboard } else { position.black_pieces_bitboard }
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
pub fn potential_pawn_jump_moves(bb: Bitboard, position: &Position) -> Bitboard {
    if position.mover == WHITE {
        (bb << 8) & RANK_4_BITS
    } else {
        (bb >> 8) & RANK_5_BITS
    }
}

#[inline(always)]
pub fn pawn_forward_moves_bitboard(pawn_moves: Bitboard, position: &Position) -> Bitboard {
    pawn_moves | (potential_pawn_jump_moves(pawn_moves, position) & empty_squares_bitboard(position))
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
pub fn generate_white_pawn_moves(mut from_squares: Bitboard, position: &Position, empty_squares: Bitboard, move_list: &mut MoveList) {

    while from_squares != 0 {
        let from_square = from_squares.trailing_zeros();
        let pawn_forward_and_capture_moves = pawn_forward_and_capture_moves_bitboard(
            from_square as Square,
            WHITE_PAWN_MOVES_CAPTURE,
            pawn_forward_moves_bitboard(WHITE_PAWN_MOVES_FORWARD[from_square as usize] & empty_squares, position),
            position
        );
        generate_pawn_moves_from_to_squares(from_square as Square, pawn_forward_and_capture_moves, move_list);
        unset_lsb!(from_squares);
    };
}

#[inline(always)]
pub fn generate_black_pawn_moves(mut from_squares: Bitboard, position: &Position, empty_squares: Bitboard, move_list: &mut MoveList) {

    while from_squares != 0 {
        let from_square = from_squares.trailing_zeros();
        let pawn_forward_and_capture_moves = pawn_forward_and_capture_moves_bitboard(
            from_square as Square,
            BLACK_PAWN_MOVES_CAPTURE,
            pawn_forward_moves_bitboard(BLACK_PAWN_MOVES_FORWARD[from_square as usize] & empty_squares, position),
            position
        );
        generate_pawn_moves_from_to_squares(from_square as Square, pawn_forward_and_capture_moves, move_list);
        unset_lsb!(from_squares);
    };
}

#[inline(always)]
pub fn any_squares_in_bitboard_attacked(position: &Position, attacker: Mover, mut bitboard: Bitboard, magic_box: &MagicBox) -> bool {
    while bitboard != 0 {
        let square = bitboard.trailing_zeros() as Square;
        if is_square_attacked_by(position, square, attacker, magic_box) { return true }
        unset_lsb!(bitboard);
    }
    return false;
}

#[inline(always)]
pub fn pawn_moves_capture_of_colour(mover: Mover, square: Square) -> Bitboard {
    if mover == WHITE {
        WHITE_PAWN_MOVES_CAPTURE[square as usize]
    } else {
        BLACK_PAWN_MOVES_CAPTURE[square as usize]
    }
}

#[inline(always)]
pub fn is_square_attacked_by(position: &Position, attacked_square: Square, mover: Mover, magic_box: &MagicBox) -> bool {
    let all_pieces = position.all_pieces_bitboard;
    if mover == WHITE {
        is_square_attacked_by_any_pawn(position.white_pawn_bitboard, pawn_moves_capture_of_colour(BLACK, attacked_square)) ||
            is_square_attacked_by_any_knight(position.white_knight_bitboard, attacked_square) ||
            is_square_attacked_by_any_rook(all_pieces, rook_move_pieces_bitboard(position, WHITE), attacked_square, magic_box) ||
            is_square_attacked_by_any_bishop(all_pieces, bishop_move_pieces_bitboard(position, WHITE), attacked_square, magic_box) ||
            is_square_attacked_by_king(position.white_king_square, attacked_square)
    } else {
        is_square_attacked_by_any_pawn(position.black_pawn_bitboard, pawn_moves_capture_of_colour(WHITE, attacked_square)) ||
            is_square_attacked_by_any_knight(position.black_knight_bitboard, attacked_square) ||
            is_square_attacked_by_any_rook(all_pieces, rook_move_pieces_bitboard(position, BLACK), attacked_square, magic_box) ||
            is_square_attacked_by_any_bishop(all_pieces, bishop_move_pieces_bitboard(position, BLACK), attacked_square, magic_box) ||
            is_square_attacked_by_king(position.black_king_square, attacked_square)
    }
}

#[inline(always)]
pub fn rook_move_pieces_bitboard(position: &Position, mover: Mover) -> Bitboard {
    if mover == WHITE {
        position.white_rook_bitboard | position.white_queen_bitboard
    } else {
        position.black_rook_bitboard | position.black_queen_bitboard
    }
}

#[inline(always)]
pub fn bishop_move_pieces_bitboard(position: &Position, mover: Mover) -> Bitboard {
    if mover == WHITE {
        position.white_bishop_bitboard | position.white_queen_bitboard
    } else {
        position.black_bishop_bitboard | position.black_queen_bitboard
    }
}

#[inline(always)]
pub fn is_square_attacked_by_any_knight(knight_bitboard: Bitboard, attacked_square: Square) -> bool {
    knight_bitboard & KNIGHT_MOVES_BITBOARDS[attacked_square as usize] != 0
}

#[inline(always)]
pub fn is_square_attacked_by_king(king_square: Square, attacked_square: Square) -> bool {
    bit(king_square) & KING_MOVES_BITBOARDS[attacked_square as usize] != 0
}

#[inline(always)]
pub fn is_square_attacked_by_any_pawn(pawns: Bitboard, pawn_attacks: Bitboard) -> bool {
    pawns & pawn_attacks != 0
}

#[inline(always)]
pub fn is_square_attacked_by_any_bishop(all_pieces: Bitboard, mut attacking_bishops: Bitboard, attacked_square: Square, magic_box: &MagicBox) -> bool {
    while attacking_bishops != 0 {
        if is_bishop_attacking_square(attacked_square, attacking_bishops.trailing_zeros() as Square, all_pieces, magic_box) { return true };
        unset_lsb!(attacking_bishops)
    }
    return false;
}

#[inline(always)]
pub fn is_square_attacked_by_any_rook(all_pieces: Bitboard, mut attacking_rooks: Bitboard, attacked_square: Square, magic_box: &MagicBox) -> bool {
    while attacking_rooks != 0 {
        if is_rook_attacking_square(attacked_square, attacking_rooks.trailing_zeros() as Square, all_pieces, magic_box) { return true };
        unset_lsb!(attacking_rooks)
    }
    return false;
}

#[inline(always)]
pub fn is_bishop_attacking_square(attacked_square: Square, piece_square: Square, all_pieces_bitboard: Bitboard, magic_box: &MagicBox) -> bool {
    test_bit(magic_moves(piece_square,all_pieces_bitboard, &magic_box.bishop), attacked_square)
}

#[inline(always)]
pub fn is_rook_attacking_square(attacked_square: Square, piece_square: Square, all_pieces_bitboard: Bitboard, magic_box: &MagicBox) -> bool {
    test_bit(magic_moves(piece_square, all_pieces_bitboard, &magic_box.rook), attacked_square)
}

#[inline(always)]
pub fn generate_castle_moves(position: &Position, move_list: &mut MoveList, magic_box: &MagicBox) {
    let all_pieces = position.all_pieces_bitboard;
    if position.mover == WHITE {
        if is_wk_castle_available(position) && all_pieces & EMPTY_CASTLE_SQUARES_WHITE_KING == 0 &&
            !any_squares_in_bitboard_attacked(position, BLACK, NO_CHECK_CASTLE_SQUARES_WHITE_KING, magic_box) {
            move_list.push(WHITE_KING_CASTLE_MOVE);
        }
        if is_wq_castle_available(position) && all_pieces & EMPTY_CASTLE_SQUARES_WHITE_QUEEN == 0 &&
            !any_squares_in_bitboard_attacked(position, BLACK, NO_CHECK_CASTLE_SQUARES_WHITE_QUEEN, magic_box) {
            move_list.push(WHITE_QUEEN_CASTLE_MOVE);
        }
    } else {
        if is_bk_castle_available(position) && all_pieces & EMPTY_CASTLE_SQUARES_BLACK_KING == 0 &&
            !any_squares_in_bitboard_attacked(position, WHITE, NO_CHECK_CASTLE_SQUARES_BLACK_KING, magic_box) {
            move_list.push(BLACK_KING_CASTLE_MOVE);
        }
        if is_bq_castle_available(position) && all_pieces & EMPTY_CASTLE_SQUARES_BLACK_QUEEN == 0 &&
            !any_squares_in_bitboard_attacked(position, WHITE, NO_CHECK_CASTLE_SQUARES_BLACK_QUEEN, magic_box) {
            move_list.push(BLACK_QUEEN_CASTLE_MOVE);
        }
    };
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
pub fn is_check(position: &Position, mover: Mover, magic_box: &MagicBox) -> bool {
    is_square_attacked_by(position, king_square(position, mover), opponent!(mover), magic_box)
}

#[inline(always)]
pub fn allocate_magic_boxes() -> MagicBox {
    MagicBox {
        bishop: Box::new(MagicVars {
            occupancy_mask: OCCUPANCY_MASK_BISHOP,
            magic_number: MAGIC_NUMBER_BISHOP,
            magic_moves: MAGIC_MOVES_BISHOP,
            magic_number_shifts: MAGIC_NUMBER_SHIFTS_BISHOP
        }),
        rook: Box::new(MagicVars {
            occupancy_mask: OCCUPANCY_MASK_ROOK,
            magic_number: MAGIC_NUMBER_ROOK,
            magic_moves: MAGIC_MOVES_ROOK,
            magic_number_shifts: MAGIC_NUMBER_SHIFTS_ROOK
        })
    }
}
