use crate::bitboards::{BISHOP_RAYS, bit, DOUBLE_MOVE_RANK_BITS, EMPTY_CASTLE_SQUARES, epsbit, KING_MOVES_BITBOARDS, KNIGHT_MOVES_BITBOARDS, NO_CHECK_CASTLE_SQUARES, PAWN_MOVES_CAPTURE, PAWN_MOVES_FORWARD, ROOK_RAYS};
use crate::magic_bitboards::{magic_moves_rook, magic_moves_bishop};
use crate::move_constants::{CASTLE_FLAG, CASTLE_MOVE, KING_INDEX, PIECE_MASK_BISHOP, PIECE_MASK_KING, PIECE_MASK_KNIGHT, PIECE_MASK_QUEEN, PIECE_MASK_ROOK, PROMOTION_BISHOP_MOVE_MASK, PROMOTION_KNIGHT_MOVE_MASK, PROMOTION_QUEEN_MOVE_MASK, PROMOTION_ROOK_MOVE_MASK, PROMOTION_SQUARES, QUEEN_INDEX};
use crate::types::{Bitboard, BLACK, Move, MoveList, Mover, Position, Square, WHITE};
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

    unsafe {
        let all_pieces = position.pieces.get_unchecked(WHITE as usize).all_pieces_bitboard | position.pieces.get_unchecked(BLACK as usize).all_pieces_bitboard;
        let friendly = position.pieces.get_unchecked(position.mover as usize);
        let valid_destinations = !friendly.all_pieces_bitboard;

        if position.castle_flags != 0 { generate_castle_moves(position, &mut move_list, all_pieces, position.mover as usize) }
        add_moves!(move_list, from_square_mask(friendly.king_square) | PIECE_MASK_KING, KING_MOVES_BITBOARDS.get_unchecked(friendly.king_square as usize) & valid_destinations);

        generate_straight_slider_moves(friendly.rook_bitboard, all_pieces, &mut move_list, valid_destinations, PIECE_MASK_ROOK);
        generate_knight_moves(&mut move_list, valid_destinations, friendly.knight_bitboard);
        generate_diagonal_slider_moves(friendly.bishop_bitboard, all_pieces, &mut move_list, valid_destinations, PIECE_MASK_BISHOP);
        generate_straight_slider_moves(friendly.queen_bitboard, all_pieces, &mut move_list, valid_destinations, PIECE_MASK_QUEEN);
        generate_diagonal_slider_moves(friendly.queen_bitboard, all_pieces, &mut move_list, valid_destinations, PIECE_MASK_QUEEN);
        generate_pawn_moves(position, &mut move_list, !all_pieces, position.mover as usize, friendly.pawn_bitboard);
    }

    move_list
}

fn generate_pawn_moves(position: &Position, move_list: &mut Vec<Move>, empty_squares: Bitboard, colour_index: usize, mut from_squares: Bitboard) {
    unsafe {
        while from_squares != 0 {
            let from_square = get_and_unset_lsb!(from_squares);
            let pawn_moves = PAWN_MOVES_FORWARD.get_unchecked(colour_index).get_unchecked(from_square as usize) & empty_squares;

            // If you can move one square, maybe you can move two
            let shifted = if position.mover == WHITE { pawn_moves << 8 } else { pawn_moves >> 8 } & DOUBLE_MOVE_RANK_BITS.get_unchecked(colour_index) & empty_squares;

            let enemy_pawns_capture_bitboard = position.pieces.get_unchecked(opponent!(position.mover) as usize).all_pieces_bitboard | epsbit(position.en_passant_square);

            let mut to_bitboard = PAWN_MOVES_CAPTURE.get_unchecked(colour_index).get_unchecked(from_square as usize) & enemy_pawns_capture_bitboard | pawn_moves | shifted;

            let fsm = from_square_mask(from_square);
            let is_promotion = to_bitboard & PROMOTION_SQUARES != 0;
            while to_bitboard != 0 {
                let base_move = fsm | get_and_unset_lsb!(to_bitboard) as Move;
                if is_promotion {
                    move_list.push(base_move | PROMOTION_QUEEN_MOVE_MASK);
                    move_list.push(base_move | PROMOTION_ROOK_MOVE_MASK);
                    move_list.push(base_move | PROMOTION_BISHOP_MOVE_MASK);
                    move_list.push(base_move | PROMOTION_KNIGHT_MOVE_MASK);
                } else {
                    move_list.push(base_move);
                }
            };
        };
    }
}

fn generate_castle_moves(position: &Position, move_list: &mut Vec<Move>, all_pieces: Bitboard, colour_index: usize) {
    for side in [KING_INDEX, QUEEN_INDEX] {
        unsafe {
            if position.castle_flags & CASTLE_FLAG.get_unchecked(side).get_unchecked(colour_index) != 0 && all_pieces & EMPTY_CASTLE_SQUARES.get_unchecked(side).get_unchecked(colour_index) == 0 &&
                !any_squares_in_bitboard_attacked(position, position.mover, *NO_CHECK_CASTLE_SQUARES.get_unchecked(side).get_unchecked(colour_index)) {
                move_list.push(*CASTLE_MOVE.get_unchecked(side).get_unchecked(colour_index));
            }
        }
    }
}

fn generate_knight_moves(move_list: &mut Vec<Move>, valid_destinations: Bitboard, mut from_squares_bitboard: Bitboard) {
    while from_squares_bitboard != 0 {
        let from_square = get_and_unset_lsb!(from_squares_bitboard);
        unsafe {
            add_moves!(move_list, from_square_mask(from_square) | PIECE_MASK_KNIGHT, KNIGHT_MOVES_BITBOARDS.get_unchecked(from_square as usize) & valid_destinations);
        }
    }
}

#[inline(always)]
pub fn generate_diagonal_slider_moves(mut slider_bitboard: Bitboard, all_pieces_bitboard: Bitboard, move_list: &mut MoveList, valid_destinations: Bitboard, piece_mask: Move) {
    while slider_bitboard != 0 {
        let from_square = get_and_unset_lsb!(slider_bitboard) as Square;
        add_moves!(move_list, from_square_mask(from_square) | piece_mask, magic_moves_bishop(from_square, all_pieces_bitboard) & valid_destinations);
    };
}

#[inline(always)]
pub fn generate_straight_slider_moves(mut slider_bitboard: Bitboard, all_pieces_bitboard: Bitboard, move_list: &mut MoveList, valid_destinations: Bitboard, piece_mask: Move) {
    while slider_bitboard != 0 {
        let from_square = get_and_unset_lsb!(slider_bitboard) as Square;
        add_moves!(move_list, from_square_mask(from_square) | piece_mask, magic_moves_rook(from_square, all_pieces_bitboard) & valid_destinations);
    };
}

#[inline(always)]
pub fn any_squares_in_bitboard_attacked(position: &Position, attacked: Mover, mut bitboard: Bitboard) -> bool {
    while bitboard != 0 {
        if is_square_attacked(position, bitboard.trailing_zeros() as Square, attacked) { return true }
        unset_lsb!(bitboard);
    }
    false
}

#[inline(always)]
pub fn is_square_attacked(position: &Position, attacked_square: Square, attacked: Mover) -> bool {
    unsafe {
        let all_pieces = position.pieces.get_unchecked(WHITE as usize).all_pieces_bitboard | position.pieces.get_unchecked(BLACK as usize).all_pieces_bitboard;
        let enemy = if attacked == WHITE { position.pieces.get_unchecked(BLACK as usize) } else { position.pieces.get_unchecked(WHITE as usize) };
        enemy.pawn_bitboard & PAWN_MOVES_CAPTURE.get_unchecked(attacked as usize).get_unchecked(attacked_square as usize) != 0 ||
        enemy.knight_bitboard & KNIGHT_MOVES_BITBOARDS.get_unchecked(attacked_square as usize) != 0 ||
        bit(enemy.king_square) & KING_MOVES_BITBOARDS.get_unchecked(attacked_square as usize) != 0 ||
        is_square_attacked_by_straight_slider(all_pieces, enemy.rook_bitboard | enemy.queen_bitboard, attacked_square) ||
        is_square_attacked_by_diagonal_slider(all_pieces, enemy.bishop_bitboard | enemy.queen_bitboard, attacked_square)
    }
}

#[inline(always)]
pub fn is_square_attacked_by_straight_slider(all_pieces: Bitboard, attacking_sliders: Bitboard, attacked_square: Square) -> bool {
    if attacking_sliders == 0 { return false }
    unsafe {
        // quick check
        if *ROOK_RAYS.get_unchecked(attacked_square as usize) & attacking_sliders == 0 { return false }
    }
    // proper check, considering blockers
    magic_moves_rook(attacked_square, all_pieces) & attacking_sliders != 0
}

#[inline(always)]
pub fn is_square_attacked_by_diagonal_slider(all_pieces: Bitboard, attacking_sliders: Bitboard, attacked_square: Square) -> bool {
    if attacking_sliders == 0 { return false }
    unsafe {
        // quick check
        if *BISHOP_RAYS.get_unchecked(attacked_square as usize) & attacking_sliders == 0 { return false }
    }
    // proper check, considering blockers
    magic_moves_bishop(attacked_square, all_pieces) & attacking_sliders != 0
}

#[inline(always)]
pub fn is_check(position: &Position, mover: Mover) -> bool {
    unsafe {
        is_square_attacked(position, position.pieces.get_unchecked(mover as usize).king_square, mover)
    }
}
