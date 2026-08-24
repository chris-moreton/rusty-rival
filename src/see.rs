use crate::bitboards::{bit, BISHOP_RAYS, KING_MOVES_BITBOARDS, KNIGHT_MOVES_BITBOARDS, PAWN_MOVES_CAPTURE, ROOK_RAYS};

use crate::engine_constants::{BISHOP_VALUE_AVERAGE, KNIGHT_VALUE_AVERAGE, PAWN_VALUE_AVERAGE, QUEEN_VALUE_AVERAGE, ROOK_VALUE_AVERAGE};
use crate::types::{Bitboard, Move, MoveList, Mover, Pieces, Position, Score, Square, BLACK, WHITE};
use crate::utils::{from_square_mask, from_square_part, to_square_part};
use std::cmp::min;

use crate::magic_bitboards::{magic_moves_bishop, magic_moves_rook};
use crate::move_constants::{
    EN_PASSANT_CAPTURE_MASK, EN_PASSANT_NOT_AVAILABLE, PIECE_MASK_BISHOP, PIECE_MASK_FULL, PIECE_MASK_KING, PIECE_MASK_KNIGHT,
    PIECE_MASK_PAWN, PIECE_MASK_QUEEN, PIECE_MASK_ROOK, PROMOTION_BISHOP_MOVE_MASK, PROMOTION_FULL_MOVE_MASK, PROMOTION_KNIGHT_MOVE_MASK,
    PROMOTION_QUEEN_MOVE_MASK, PROMOTION_ROOK_MOVE_MASK, PROMOTION_SQUARES,
};
use crate::{get_and_unset_lsb, get_lsb, opponent};

/// The exchange search only reads piece placement, side to move and en
/// passant. Keeping the unrelated clocks, castling flags and Zobrist keys out
/// of its recursive state reduces each by-value copy from the full Position to
/// the exact data SEE consumes, without changing its legality-aware semantics.
#[derive(Copy, Clone)]
struct SeePosition {
    pieces: [Pieces; 2],
    mover: Mover,
    en_passant_square: Square,
}

impl From<&Position> for SeePosition {
    #[inline(always)]
    fn from(position: &Position) -> Self {
        Self {
            pieces: position.pieces,
            mover: position.mover,
            en_passant_square: position.en_passant_square,
        }
    }
}

#[inline(always)]
pub fn static_exchange_evaluation(position: &Position, mv: Move) -> Score {
    let score = captured_piece_value_see(position, mv);
    static_exchange_evaluation_with_value(position, mv, score)
}

#[inline(always)]
pub(crate) fn static_exchange_evaluation_with_value(position: &Position, mv: Move, score: Score) -> Score {
    let mut new_position = SeePosition::from(position);
    make_see_move(mv, &mut new_position);
    see(score, bit(to_square_part(mv)), &mut new_position)
}

#[inline(always)]
fn see(score: Score, capture_square: Bitboard, position: &mut SeePosition) -> Score {
    for m in see_moves(position, capture_square) {
        let captured = captured_piece_value_compact(position, m);
        let mover = position.mover;
        // see_moves deliberately returns at most one candidate. Both branches
        // below immediately return from this exchange level (the illegal case
        // falls through to `score`), so no caller can observe the mutated
        // state and an undo/copy would be dead work.
        make_see_move(m, position);
        if !is_check_see(position, mover) {
            return min(score, score - see(captured, capture_square, position));
        }
    }

    score
}

#[inline(always)]
fn make_see_move(mv: Move, new_position: &mut SeePosition) {
    let from = from_square_part(mv);
    let to = to_square_part(mv);

    let piece_mask = mv & PIECE_MASK_FULL;
    let bit_to = bit(to);
    let enemy = &mut new_position.pieces[opponent!(new_position.mover) as usize];
    let switch = bit(from) | bit_to;

    if piece_mask == PIECE_MASK_PAWN && new_position.en_passant_square == to {
        let pawn_off = EN_PASSANT_CAPTURE_MASK[to as usize];

        enemy.pawn_bitboard &= pawn_off;
        enemy.all_pieces_bitboard &= pawn_off;
    } else {
        enemy.pawn_bitboard &= !bit_to;
        enemy.knight_bitboard &= !bit_to;
        enemy.rook_bitboard &= !bit_to;
        enemy.bishop_bitboard &= !bit_to;
        enemy.queen_bitboard &= !bit_to;

        enemy.all_pieces_bitboard &= !bit_to;
    }
    new_position.en_passant_square = EN_PASSANT_NOT_AVAILABLE;

    // All four promotion pieces, not just the queen (NET-374). generate_captures
    // emits rook/bishop/knight promo-captures and score_move_with_see feeds them
    // straight to SEE; the queen-only test sent them down the plain-pawn branch
    // below, which XORs a PAWN onto the promotion square - a phantom pawn on
    // rank 8 that then defends and recaptures as a pawn for the rest of the
    // exchange, mis-valuing the whole sequence.
    if mv & PROMOTION_FULL_MOVE_MASK != 0 {
        let mover = new_position.mover as usize;
        match mv & PROMOTION_FULL_MOVE_MASK {
            PROMOTION_QUEEN_MOVE_MASK => new_position.pieces[mover].queen_bitboard |= bit_to,
            PROMOTION_ROOK_MOVE_MASK => new_position.pieces[mover].rook_bitboard |= bit_to,
            PROMOTION_BISHOP_MOVE_MASK => new_position.pieces[mover].bishop_bitboard |= bit_to,
            PROMOTION_KNIGHT_MOVE_MASK => new_position.pieces[mover].knight_bitboard |= bit_to,
            _ => {}
        }
        new_position.pieces[mover].pawn_bitboard &= !bit(from);
    } else {
        match piece_mask {
            PIECE_MASK_PAWN => new_position.pieces[new_position.mover as usize].pawn_bitboard ^= switch,
            PIECE_MASK_KNIGHT => new_position.pieces[new_position.mover as usize].knight_bitboard ^= switch,
            PIECE_MASK_BISHOP => new_position.pieces[new_position.mover as usize].bishop_bitboard ^= switch,
            PIECE_MASK_ROOK => new_position.pieces[new_position.mover as usize].rook_bitboard ^= switch,
            PIECE_MASK_QUEEN => new_position.pieces[new_position.mover as usize].queen_bitboard ^= switch,
            PIECE_MASK_KING => new_position.pieces[new_position.mover as usize].king_square = to,
            _ => panic!("Piece panic"),
        }
    }

    new_position.pieces[new_position.mover as usize].all_pieces_bitboard ^= switch;
    new_position.mover ^= 1;
}

#[inline(always)]
pub fn captured_piece_value_see(position: &Position, mv: Move) -> Score {
    let enemy = &position.pieces[opponent!(position.mover) as usize];
    captured_piece_value(enemy, position.en_passant_square, mv)
}

#[inline(always)]
fn captured_piece_value_compact(position: &SeePosition, mv: Move) -> Score {
    let enemy = &position.pieces[opponent!(position.mover) as usize];
    captured_piece_value(enemy, position.en_passant_square, mv)
}

#[inline(always)]
fn captured_piece_value(enemy: &Pieces, en_passant_square: Square, mv: Move) -> Score {
    let tsp = to_square_part(mv);
    let to_bb = bit(tsp);

    // Mirror utils::captured_piece_value: every promotion gains its piece's
    // value, not only the queen (NET-374)
    let promote_value = match mv & PROMOTION_FULL_MOVE_MASK {
        PROMOTION_QUEEN_MOVE_MASK => QUEEN_VALUE_AVERAGE - PAWN_VALUE_AVERAGE,
        PROMOTION_ROOK_MOVE_MASK => ROOK_VALUE_AVERAGE - PAWN_VALUE_AVERAGE,
        PROMOTION_BISHOP_MOVE_MASK => BISHOP_VALUE_AVERAGE - PAWN_VALUE_AVERAGE,
        PROMOTION_KNIGHT_MOVE_MASK => KNIGHT_VALUE_AVERAGE - PAWN_VALUE_AVERAGE,
        _ => 0,
    };

    promote_value
        + (if (mv & PIECE_MASK_FULL == PIECE_MASK_PAWN && tsp == en_passant_square) || enemy.pawn_bitboard & to_bb != 0 {
            PAWN_VALUE_AVERAGE
        } else if enemy.knight_bitboard & to_bb != 0 {
            KNIGHT_VALUE_AVERAGE
        } else if enemy.bishop_bitboard & to_bb != 0 {
            BISHOP_VALUE_AVERAGE
        } else if enemy.rook_bitboard & to_bb != 0 {
            ROOK_VALUE_AVERAGE
        } else if enemy.queen_bitboard & to_bb != 0 {
            QUEEN_VALUE_AVERAGE
        } else {
            0
        })
}

#[inline(always)]
pub fn generate_capture_pawn_moves_with_destinations_see(
    move_list: &mut MoveList,
    colour_index: usize,
    mut from_squares: Bitboard,
    valid_destinations: Bitboard,
) {
    while from_squares != 0 {
        let from_square = get_and_unset_lsb!(from_squares);

        let to_bitboard = PAWN_MOVES_CAPTURE[colour_index][from_square as usize] & valid_destinations;

        let fsm = from_square_mask(from_square);
        let is_promotion = to_bitboard & PROMOTION_SQUARES != 0;
        if to_bitboard != 0 {
            let base_move = fsm | get_lsb!(to_bitboard) as Move;
            if is_promotion {
                move_list.push(base_move | PROMOTION_QUEEN_MOVE_MASK);
            } else {
                move_list.push(base_move);
            }
            break;
        }
    }
}

#[inline(always)]
pub fn generate_diagonal_slider_moves_see(
    slider_bitboard: Bitboard,
    all_pieces_bitboard: Bitboard,
    move_list: &mut MoveList,
    valid_destinations: Bitboard,
    piece_mask: Move,
) {
    let capture_square = valid_destinations.trailing_zeros();
    let bb = magic_moves_bishop(capture_square as Square, all_pieces_bitboard) & slider_bitboard;
    if bb != 0 {
        move_list.push(from_square_mask(bb.trailing_zeros() as Square) | piece_mask | capture_square);
    }
}

#[inline(always)]
pub fn generate_straight_slider_moves_see(
    slider_bitboard: Bitboard,
    all_pieces_bitboard: Bitboard,
    move_list: &mut MoveList,
    valid_destinations: Bitboard,
    piece_mask: Move,
) {
    let capture_square = valid_destinations.trailing_zeros();
    let bb = magic_moves_rook(capture_square as Square, all_pieces_bitboard) & slider_bitboard;
    if bb != 0 {
        move_list.push(from_square_mask(bb.trailing_zeros() as Square) | piece_mask | capture_square);
    }
}

#[inline(always)]
fn see_moves(position: &SeePosition, valid_destinations: Bitboard) -> MoveList {
    let mut move_list = MoveList::new();
    let capture_square = valid_destinations.trailing_zeros() as usize;

    let all_pieces = position.pieces[WHITE as usize].all_pieces_bitboard | position.pieces[BLACK as usize].all_pieces_bitboard;
    let friendly = position.pieces[position.mover as usize];

    generate_capture_pawn_moves_with_destinations_see(&mut move_list, position.mover as usize, friendly.pawn_bitboard, valid_destinations);

    if move_list.is_empty() {
        let knights = KNIGHT_MOVES_BITBOARDS[capture_square] & friendly.knight_bitboard;
        if knights != 0 {
            let fsm = from_square_mask(knights.trailing_zeros() as Square) | PIECE_MASK_KNIGHT;
            move_list.push(fsm | capture_square as Move);
        }
    }

    if move_list.is_empty() && BISHOP_RAYS[capture_square] & friendly.bishop_bitboard != 0 {
        generate_diagonal_slider_moves_see(
            friendly.bishop_bitboard,
            all_pieces,
            &mut move_list,
            valid_destinations,
            PIECE_MASK_BISHOP,
        );
    }
    if move_list.is_empty() && ROOK_RAYS[capture_square] & friendly.rook_bitboard != 0 {
        generate_straight_slider_moves_see(
            friendly.rook_bitboard,
            all_pieces,
            &mut move_list,
            valid_destinations,
            PIECE_MASK_ROOK,
        );
    }
    if move_list.is_empty() && ROOK_RAYS[capture_square] & friendly.queen_bitboard != 0 {
        generate_straight_slider_moves_see(
            friendly.queen_bitboard,
            all_pieces,
            &mut move_list,
            valid_destinations,
            PIECE_MASK_QUEEN,
        );
    }
    if move_list.is_empty() && BISHOP_RAYS[capture_square] & friendly.queen_bitboard != 0 {
        generate_diagonal_slider_moves_see(
            friendly.queen_bitboard,
            all_pieces,
            &mut move_list,
            valid_destinations,
            PIECE_MASK_QUEEN,
        );
    }

    if move_list.is_empty() {
        let bb = KING_MOVES_BITBOARDS[friendly.king_square as usize] & valid_destinations;
        if bb != 0 {
            move_list.push(from_square_mask(friendly.king_square) | PIECE_MASK_KING | bb.trailing_zeros() as Move);
        }
    }

    move_list
}

#[inline(always)]
fn is_check_see(position: &SeePosition, mover: Mover) -> bool {
    let attacked_square = position.pieces[mover as usize].king_square;
    let enemy = position.pieces[opponent!(mover) as usize];

    enemy.pawn_bitboard & PAWN_MOVES_CAPTURE[mover as usize][attacked_square as usize] != 0
        || (enemy.knight_bitboard != 0 && enemy.knight_bitboard & KNIGHT_MOVES_BITBOARDS[attacked_square as usize] != 0)
        || bit(enemy.king_square) & KING_MOVES_BITBOARDS[attacked_square as usize] != 0
        || {
            let all_pieces = position.pieces[WHITE as usize].all_pieces_bitboard | position.pieces[BLACK as usize].all_pieces_bitboard;
            let straight = enemy.rook_bitboard | enemy.queen_bitboard;
            let diagonal = enemy.bishop_bitboard | enemy.queen_bitboard;

            (straight != 0
                && ROOK_RAYS[attacked_square as usize] & straight != 0
                && magic_moves_rook(attacked_square, all_pieces) & straight != 0)
                || (diagonal != 0
                    && BISHOP_RAYS[attacked_square as usize] & diagonal != 0
                    && magic_moves_bishop(attacked_square, all_pieces) & diagonal != 0)
        }
}
