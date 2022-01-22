use crate::bitboards::{bit, DOUBLE_MOVE_RANK_BITS, EMPTY_CASTLE_SQUARES, EN_PASSANT_CAPTURE_RANK, enemy_bitboard, KING_MOVES_BITBOARDS, KNIGHT_MOVES_BITBOARDS, NO_CHECK_CASTLE_SQUARES, PAWN_MOVES_CAPTURE, PAWN_MOVES_FORWARD, test_bit };
use crate::magic_bitboards::{magic_moves, MAGIC_BOX};
use crate::move_constants::{BLACK_INDEX, CASTLE_FLAG, CASTLE_MOVE, EN_PASSANT_NOT_AVAILABLE, KING_INDEX, PROMOTION_BISHOP_MOVE_MASK, PROMOTION_KNIGHT_MOVE_MASK, PROMOTION_QUEEN_MOVE_MASK, PROMOTION_ROOK_MOVE_MASK, PROMOTION_SQUARES, QUEEN_INDEX, WHITE_INDEX };
use crate::types::{Bitboard, BLACK, MagicVars, Move, MoveList, Mover, Position, Square, WHITE};
use crate::{get_and_unset_lsb, opponent, unset_lsb};
use crate::utils::from_square_mask;

#[macro_export]
macro_rules! add_moves {
    ($move_list:expr, $fsm:expr, $to_bitboard:expr) => {
        let mut bb = $to_bitboard;
        while bb != 0 {
            $move_list.push($fsm | get_and_unset_lsb!(bb) as Move);
        }
    }
}

#[inline(always)]
pub fn moves(position: &Position) -> MoveList {
    let mut move_list = Vec::with_capacity(80);

    let all_pieces = position.white.all_pieces_bitboard | position.black.all_pieces_bitboard;
    let empty_squares = !all_pieces;
    let colour_index = if position.mover == WHITE { WHITE_INDEX } else { BLACK_INDEX };
    let friendly = if position.mover == WHITE { position.white } else { position.black };
    let valid_destinations = !friendly.all_pieces_bitboard;

    for side in [KING_INDEX, QUEEN_INDEX] {
        if position.castle_flags & CASTLE_FLAG[side][colour_index] != 0 && all_pieces & EMPTY_CASTLE_SQUARES[side][colour_index] == 0 &&
            !any_squares_in_bitboard_attacked(position, opponent!(position.mover), NO_CHECK_CASTLE_SQUARES[side][colour_index]) {
            move_list.push(CASTLE_MOVE[side][colour_index]);
        }
    }

    let mut from_squares_bitboard = friendly.knight_bitboard;
    while from_squares_bitboard != 0 {
        let from_square = get_and_unset_lsb!(from_squares_bitboard);
        add_moves!(move_list, from_square_mask(from_square), KNIGHT_MOVES_BITBOARDS[from_square as usize] & valid_destinations);
    }

    add_moves!(move_list, from_square_mask(friendly.king_square), KING_MOVES_BITBOARDS[friendly.king_square as usize] & valid_destinations);

    generate_slider_moves(friendly.rook_bitboard | friendly.queen_bitboard, all_pieces, &mut move_list, &MAGIC_BOX.rook, valid_destinations);
    generate_slider_moves(friendly.bishop_bitboard | friendly.queen_bitboard, all_pieces, &mut move_list, &MAGIC_BOX.bishop, valid_destinations);

    let mut from_squares = friendly.pawn_bitboard;

    while from_squares != 0 {
        let from_square = get_and_unset_lsb!(from_squares);
        let pawn_moves = PAWN_MOVES_FORWARD[colour_index][from_square as usize] & empty_squares;
        let shifted = if colour_index == WHITE_INDEX { pawn_moves << 8 } else { pawn_moves >> 8 };
        let pawn_forward_and_capture_moves = pawn_forward_and_capture_moves_bitboard(
            from_square as Square,
            PAWN_MOVES_CAPTURE[colour_index],
            position
        ) | pawn_moves | (shifted & DOUBLE_MOVE_RANK_BITS[colour_index] & empty_squares);
        generate_pawn_moves_from_to_squares(from_square, pawn_forward_and_capture_moves, &mut move_list);
    };

    move_list
}

#[inline(always)]
pub fn generate_slider_moves(slider_bitboard: Bitboard, all_pieces_bitboard: Bitboard, move_list: &mut MoveList, magic_vars: &Box<MagicVars>, valid_destinations: Bitboard) {
    let mut from_bitboard = slider_bitboard;
    while from_bitboard != 0 {
        let from_square = get_and_unset_lsb!(from_bitboard) as Square;
        add_moves!(move_list, from_square_mask(from_square), magic_moves(from_square,all_pieces_bitboard, magic_vars) & valid_destinations);
    };
}

#[inline(always)]
pub fn generate_pawn_moves_from_to_squares(from_square: Square, mut to_bitboard: Bitboard, move_list: &mut MoveList) {
    let mask = from_square_mask(from_square);
    let is_promotion = to_bitboard & PROMOTION_SQUARES != 0;
    while to_bitboard != 0 {
        let base_move = mask | get_and_unset_lsb!(to_bitboard) as Move;
        if is_promotion {
            move_list.push(base_move | PROMOTION_QUEEN_MOVE_MASK);
            move_list.push(base_move | PROMOTION_ROOK_MOVE_MASK);
            move_list.push(base_move | PROMOTION_BISHOP_MOVE_MASK);
            move_list.push(base_move | PROMOTION_KNIGHT_MOVE_MASK);
        } else {
            move_list.push(base_move);
        }
    };
}

#[inline(always)]
pub fn pawn_forward_and_capture_moves_bitboard(from_square: Square, capture_pawn_moves: [Bitboard; 64], position: &Position) -> Bitboard {
    if position.en_passant_square != EN_PASSANT_NOT_AVAILABLE && bit(position.en_passant_square) & EN_PASSANT_CAPTURE_RANK[if position.mover == WHITE { WHITE_INDEX } else { BLACK_INDEX }] != 0 {
        capture_pawn_moves[from_square as usize] & (enemy_bitboard(position) | bit(position.en_passant_square))
    } else {
        capture_pawn_moves[from_square as usize] & enemy_bitboard(position)
    }
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
    let all_pieces = position.white.all_pieces_bitboard | position.black.all_pieces_bitboard;
    let friendly = if mover == WHITE { position.white } else { position.black };

    friendly.pawn_bitboard & PAWN_MOVES_CAPTURE[if mover == BLACK { WHITE_INDEX } else { BLACK_INDEX }][attacked_square as usize] != 0 ||
        friendly.knight_bitboard & KNIGHT_MOVES_BITBOARDS[attacked_square as usize] != 0 ||
        is_square_attacked_by_slider_of_type(all_pieces, friendly.rook_bitboard | friendly.queen_bitboard, attacked_square, &MAGIC_BOX.rook) ||
        is_square_attacked_by_slider_of_type(all_pieces, friendly.bishop_bitboard | friendly.queen_bitboard, attacked_square, &MAGIC_BOX.bishop) ||
        bit(friendly.king_square) & KING_MOVES_BITBOARDS[attacked_square as usize] != 0
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
pub fn is_check(position: &Position, mover: Mover) -> bool {
    is_square_attacked_by(position, if mover == WHITE { position.white.king_square } else { position.black.king_square }, opponent!(mover))
}
