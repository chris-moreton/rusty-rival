use crate::bitboards::{A1_BIT, A8_BIT, bit, C1_BIT, C8_BIT, clear_bit, D1_BIT, D8_BIT, E1_BIT, E8_BIT, F1_BIT, F8_BIT, G1_BIT, G8_BIT, H1_BIT, H8_BIT, test_bit};
use crate::move_constants::*;
use crate::moves::{move_mover_or_remove_captured};
use crate::types::{Bitboard, BLACK, is_any_black_castle_available, is_any_white_castle_available, Move, Mover, Piece, Position, PositionHistory, Square, unset_bk_castle, unset_black_castles, unset_bq_castle, unset_white_castles, unset_wk_castle, unset_wq_castle, WHITE};
use crate::types::Piece::{Bishop, Empty, King, Knight, Pawn, Queen, Rook};
use crate::utils::{from_square_part, to_square_part};

pub fn make_move(position: &mut Position, mv: Move, history: &mut PositionHistory) {
    let from = from_square_part(mv);
    let to = to_square_part(mv);
    let piece = moving_piece(position, from);
    store_history(position, history);
    if is_complex_move(position, from as Square, to, piece) {
        make_complex_move(position, from, to, mv)
    } else {
        make_simple_move(position, from as Square, to, piece)
    };
    position.mover = switch_side(position.mover);
    position.move_number = if position.mover == WHITE { position.move_number + 1 } else { position.move_number }
}

#[inline(always)]
pub fn make_simple_move(position: &mut Position, from: Square, to: Square, piece: Piece) {
    let switch_bitboard = bit(from) | bit(to);
    position.all_pieces_bitboard ^= switch_bitboard;
    if position.mover == WHITE {
        position.white_pieces_bitboard ^= switch_bitboard;
        make_simple_white_move(position, from, to, piece)
    } else {
        position.black_pieces_bitboard ^= switch_bitboard;
        make_simple_black_move(position, from, to, piece)
    }
}

#[inline(always)]
pub fn make_complex_move(position: &mut Position, from: Square, to: Square, mv: Move) {
    let promoted_piece = promotion_piece_from_move(mv);

    if promoted_piece != Empty {
        make_move_with_promotion(position, mv, promoted_piece);
    } else if from == E1_BIT && (to == G1_BIT || to == C1_BIT) && is_any_white_castle_available(position) {
        make_white_castle_move(position, to);
    } else if from == E8_BIT && (to == G8_BIT || to == C8_BIT) && is_any_black_castle_available(position) {
        make_black_castle_move(position, to);
    } else {
        make_simple_complex_move(position, from, to);
    }

    position.en_passant_square = EN_PASSANT_NOT_AVAILABLE;

}

#[inline(always)]
pub fn make_white_castle_move(position: &mut Position, to: Square) {
    let wr= if to == C1_BIT {
        clear_bit(position.white_rook_bitboard, A1_BIT) | bit(D1_BIT)
    } else {
        clear_bit(position.white_rook_bitboard, H1_BIT) | bit(F1_BIT)
    };
    let wk = clear_bit(position.white_king_bitboard, E1_BIT) | bit(to);
    let wpb = wr | wk | position.white_queen_bitboard | position.white_knight_bitboard | position.white_bishop_bitboard | position.white_pawn_bitboard;
    position.white_rook_bitboard = wr;
    position.white_king_bitboard = wk;
    position.all_pieces_bitboard = wpb | position.black_pieces_bitboard;
    position.white_pieces_bitboard = wpb;
    unset_white_castles(position);
    position.half_moves += 1;
}

#[inline(always)]
pub fn make_black_castle_move(position: &mut Position, to: Square) {
    let br= if to == C8_BIT {
        clear_bit(position.black_rook_bitboard, A8_BIT) | bit(D8_BIT)
    } else {
        clear_bit(position.black_rook_bitboard, H8_BIT) | bit(F8_BIT)
    };

    let bk = clear_bit(position.black_king_bitboard, E8_BIT) | bit(to);

    let bpb = br | bk | position.black_queen_bitboard | position.black_knight_bitboard | position.black_bishop_bitboard | position.black_pawn_bitboard;
    position.black_rook_bitboard = br;
    position.black_king_bitboard = bk;
    position.all_pieces_bitboard = bpb | position.white_pieces_bitboard;
    position.black_pieces_bitboard = bpb;
    unset_black_castles(position);
    position.half_moves += 1;

}

#[inline(always)]
pub fn promotion_piece_from_move(mv: Move) -> Piece {
    match PROMOTION_FULL_MOVE_MASK & mv {
        PROMOTION_QUEEN_MOVE_MASK => Queen,
        PROMOTION_ROOK_MOVE_MASK => Rook,
        PROMOTION_BISHOP_MOVE_MASK => Bishop,
        PROMOTION_KNIGHT_MOVE_MASK => Knight,
        _ => Empty
    }
}

#[inline(always)]
pub fn is_promotion_square(square: Square) -> bool {
    test_bit(PROMOTION_SQUARES, square)
}

#[inline(always)]
pub fn remove_pawn_if_promotion(bitboard: Bitboard) -> Bitboard {
    bitboard & NON_PROMOTION_SQUARES
}

#[inline(always)]
pub fn make_move_with_promotion(position: &mut Position, mv: Move, promotion_piece: Piece) {
    let from = from_square_part(mv);
    let to = to_square_part(mv);

    let wp = remove_pawn_if_promotion(move_mover_or_remove_captured(from, to, position.white_pawn_bitboard));
    let bp = remove_pawn_if_promotion(move_mover_or_remove_captured(from, to, position.black_pawn_bitboard));

    let piece_bitboard = move_mover_or_remove_captured(from, to, position.white_knight_bitboard);
    let wn = if position.mover == WHITE && promotion_piece == Knight { piece_bitboard | bit(to) } else { piece_bitboard };

    let piece_bitboard = move_mover_or_remove_captured(from, to, position.black_knight_bitboard);
    let bn = if position.mover == BLACK && promotion_piece == Knight { piece_bitboard | bit(to) } else { piece_bitboard };

    let piece_bitboard = move_mover_or_remove_captured(from, to, position.white_bishop_bitboard);
    let wb = if position.mover == WHITE && promotion_piece == Bishop { piece_bitboard | bit(to) } else { piece_bitboard };

    let piece_bitboard = move_mover_or_remove_captured(from, to, position.black_bishop_bitboard);
    let bb = if position.mover == BLACK && promotion_piece == Bishop { piece_bitboard | bit(to) } else { piece_bitboard };

    let piece_bitboard = move_mover_or_remove_captured(from, to, position.white_rook_bitboard);
    let wr = if position.mover == WHITE && promotion_piece == Rook { piece_bitboard | bit(to) } else { piece_bitboard };

    let piece_bitboard = move_mover_or_remove_captured(from, to, position.black_rook_bitboard);
    let br = if position.mover == BLACK && promotion_piece == Rook { piece_bitboard | bit(to) } else { piece_bitboard };

    let piece_bitboard = move_mover_or_remove_captured(from, to, position.white_queen_bitboard);
    let wq = if position.mover == WHITE && promotion_piece == Queen { piece_bitboard | bit(to) } else { piece_bitboard };

    let piece_bitboard = move_mover_or_remove_captured(from, to, position.black_queen_bitboard);
    let bq = if position.mover == BLACK && promotion_piece == Queen { piece_bitboard | bit(to) } else { piece_bitboard };

    let wk = position.white_king_bitboard;
    let bk = position.black_king_bitboard;

    let wpb = wp | wn | wr | wk | wq | wb;
    let bpb = bp | bn | br | bk | bq | bb;

    position.white_pawn_bitboard = wp;
    position.black_pawn_bitboard = bp;
    position.white_knight_bitboard = wn;
    position.black_knight_bitboard = bn;
    position.white_bishop_bitboard = wb;
    position.black_bishop_bitboard = bb;
    position.white_rook_bitboard = wr;
    position.black_rook_bitboard = br;
    position.white_queen_bitboard = wq;
    position.black_queen_bitboard = bq;
    position.white_king_bitboard = wk;
    position.black_king_bitboard = bk;
    position.all_pieces_bitboard = wpb | bpb;
    position.white_pieces_bitboard = wpb;
    position.black_pieces_bitboard = bpb;

    if to == H1_BIT { unset_wk_castle(position) }
    if to == A1_BIT { unset_wq_castle(position) }
    if to == H8_BIT { unset_bk_castle(position) }
    if to == A8_BIT { unset_bq_castle(position) }

    position.half_moves = 0;
}

#[inline(always)]
pub fn en_passant_captured_piece_square(square: Square) -> Square {
    match square {
        16 => 24,
        17 => 25,
        18 => 26,
        19 => 27,
        20 => 28,
        21 => 29,
        22 => 30,
        23 => 31,
        40 => 32,
        41 => 33,
        42 => 34,
        43 => 35,
        44 => 36,
        45 => 37,
        46 => 38,
        47 => 39,
        _ => panic!("{} is not an option", square)
    }
}

#[inline(always)]
pub fn remove_piece_from_bitboard(square: Square, bitboard: Bitboard) -> Bitboard {
    !bit(square) & bitboard
}

#[inline(always)]
pub fn make_simple_complex_move(position: &mut Position, from: Square, to: Square) {

    let is_pawn_move = test_bit(position.white_pawn_bitboard | position.black_pawn_bitboard, from);
    position.white_pawn_bitboard = move_mover_or_remove_captured(from, to, position.white_pawn_bitboard);
    position.black_pawn_bitboard = move_mover_or_remove_captured(from, to, position.black_pawn_bitboard);

    if position.en_passant_square == to {
        if test_bit(position.black_pawn_bitboard, to) {
            position.white_pawn_bitboard = remove_piece_from_bitboard(en_passant_captured_piece_square(to), position.white_pawn_bitboard);
        } else if test_bit(position.white_pawn_bitboard, to) {
            position.black_pawn_bitboard = remove_piece_from_bitboard(en_passant_captured_piece_square(to), position.black_pawn_bitboard);
        }
    }

    let wn = move_mover_or_remove_captured(from, to, position.white_knight_bitboard);
    let bn = move_mover_or_remove_captured(from, to, position.black_knight_bitboard);
    let wb = move_mover_or_remove_captured(from, to, position.white_bishop_bitboard);
    let bb = move_mover_or_remove_captured(from, to, position.black_bishop_bitboard);
    let wr = move_mover_or_remove_captured(from, to, position.white_rook_bitboard);
    let br = move_mover_or_remove_captured(from, to, position.black_rook_bitboard);
    let wq = move_mover_or_remove_captured(from, to, position.white_queen_bitboard);
    let bq = move_mover_or_remove_captured(from, to, position.black_queen_bitboard);
    let wk = move_mover_or_remove_captured(from, to, position.white_king_bitboard);
    let bk = move_mover_or_remove_captured(from, to, position.black_king_bitboard);

    let wpb = position.white_pawn_bitboard | wn | wr | wk | wq | wb;
    let bpb = position.black_pawn_bitboard | bn | br | bk | bq | bb;

    position.half_moves = if test_bit(position.all_pieces_bitboard, to) || is_pawn_move { 0 } else { position.half_moves + 1 };

    position.white_knight_bitboard = wn;
    position.black_knight_bitboard = bn;
    position.white_bishop_bitboard = wb;
    position.black_bishop_bitboard = bb;
    position.white_rook_bitboard = wr;
    position.black_rook_bitboard = br;
    position.white_queen_bitboard = wq;
    position.black_queen_bitboard = bq;
    position.white_king_bitboard = wk;
    position.black_king_bitboard = bk;
    position.all_pieces_bitboard = wpb | bpb;
    position.white_pieces_bitboard = wpb;
    position.black_pieces_bitboard = bpb;

    if from == E1_BIT || from == H1_BIT || to == H1_BIT { unset_wk_castle(position) }
    if from == E1_BIT || from == A1_BIT || to == A1_BIT { unset_wq_castle(position) }
    if from == E8_BIT || from == H8_BIT || to == H8_BIT { unset_bk_castle(position) }
    if from == E8_BIT || from == A8_BIT || to == A8_BIT { unset_bq_castle(position) }


}

#[inline(always)]
pub fn switch_side(mover: Mover) -> Mover {
    //if mover == WHITE { BLACK } else { WHITE }
    mover * -1
}

#[inline(always)]
pub fn make_simple_white_move(position: &mut Position, from: Square, to: Square, piece: Piece) {
    if piece == Pawn {
        position.white_pawn_bitboard = clear_bit(position.white_pawn_bitboard, from) | bit(to);
        position.en_passant_square = if to - from == 16 { from + 8 } else { EN_PASSANT_NOT_AVAILABLE };
        position.half_moves = 0;
    } else {
        position.half_moves += 1;
        position.en_passant_square = EN_PASSANT_NOT_AVAILABLE;
        match piece {
            Knight => {
                position.white_knight_bitboard = clear_bit(position.white_knight_bitboard, from) | bit(to);
            },
            Bishop => {
                position.white_bishop_bitboard = clear_bit(position.white_bishop_bitboard, from) | bit(to);
            },
            Rook => {
                position.white_rook_bitboard = clear_bit(position.white_rook_bitboard, from) | bit(to);
                if from == H1_BIT { unset_wk_castle(position) }
                if from == A1_BIT { unset_wq_castle(position) }
            },
            Queen => {
                position.white_queen_bitboard = clear_bit(position.white_queen_bitboard, from) | bit(to);
            },
            King => {
                position.white_king_bitboard = bit(to);
            },
            _ => {
                panic!("Piece panic")
            }
        }
    }
}

#[inline(always)]
pub fn make_simple_black_move(position: &mut Position, from: Square, to: Square, piece: Piece) {
    if piece == Pawn {
        position.black_pawn_bitboard = clear_bit(position.black_pawn_bitboard, from) | bit(to);
        position.en_passant_square = if from - to == 16 { from - 8 } else { EN_PASSANT_NOT_AVAILABLE };
        position.half_moves = 0;
    } else {
        position.half_moves += 1;
        position.en_passant_square = EN_PASSANT_NOT_AVAILABLE;
        match piece {
            Knight => {
                position.black_knight_bitboard = clear_bit(position.black_knight_bitboard, from) | bit(to);
            }
            Bishop => {
                let bb = position.black_bishop_bitboard;
                position.black_bishop_bitboard = clear_bit(bb, from) | bit(to);
            }
            Rook => {
                position.black_rook_bitboard = clear_bit(position.black_rook_bitboard, from) | bit(to);
                if from == H8_BIT { unset_bk_castle(position) }
                if from == A8_BIT { unset_bq_castle(position) }
            }
            Queen => {
                position.black_queen_bitboard = clear_bit(position.black_queen_bitboard, from) | bit(to);
            }
            King => {
                position.black_king_bitboard = bit(to);
            },
            _ => {
                panic!("Piece panic")
            }
        }
    }
}

#[inline(always)]
pub fn moving_piece(position: &Position, from_square: Square) -> Piece {
    if position.mover == WHITE {
        if test_bit(position.white_pawn_bitboard, from_square) { Pawn }
        else if test_bit(position.white_knight_bitboard, from_square) { Knight }
        else if test_bit(position.white_bishop_bitboard, from_square) { Bishop }
        else if test_bit(position.white_rook_bitboard, from_square) { Rook }
        else if test_bit(position.white_queen_bitboard, from_square) { Queen }
        else { King }
    } else if test_bit(position.black_pawn_bitboard, from_square) { Pawn }
        else if test_bit(position.black_knight_bitboard, from_square) { Knight }
        else if test_bit(position.black_bishop_bitboard, from_square) { Bishop }
        else if test_bit(position.black_rook_bitboard, from_square) { Rook }
        else if test_bit(position.black_queen_bitboard, from_square) { Queen }
        else { King }
}

#[inline(always)]
pub fn is_complex_move(position: &mut Position, from: Square, to: Square, piece: Piece) -> bool {
    test_bit(position.all_pieces_bitboard, to) ||
        (piece == Pawn && is_complex_pawn_move(from, to)) ||
            (piece == King && test_bit(KING_START_POSITIONS, from))
}

#[inline(always)]
pub fn is_complex_pawn_move(from: Square, to: Square) -> bool {
    (from - to).abs() % 8 != 0 || test_bit(PROMOTION_SQUARES, to)
}

pub fn default_position_history() -> PositionHistory {
    PositionHistory {
        history: [Position {
            white_pawn_bitboard: 0,
            white_knight_bitboard: 0,
            white_bishop_bitboard: 0,
            white_queen_bitboard: 0,
            white_king_bitboard: 0,
            white_rook_bitboard: 0,
            black_pawn_bitboard: 0,
            black_knight_bitboard: 0,
            black_bishop_bitboard: 0,
            black_queen_bitboard: 0,
            black_king_bitboard: 0,
            black_rook_bitboard: 0,
            all_pieces_bitboard: 0,
            white_pieces_bitboard: 0,
            black_pieces_bitboard: 0,
            mover: WHITE,
            en_passant_square: 0,
            castle_flags: 0,
            half_moves: 0,
            move_number: 1
        }; MAX_MOVE_HISTORY as usize]
    }
}

#[inline(always)]
pub fn get_move_index(move_number: u16, mover: Mover) -> usize {
    (move_number * 2 - if mover == WHITE { 1 } else { 0 }) as usize
}

#[inline(always)]
pub fn store_history(position: &mut Position, history: &mut PositionHistory) {
    history.history[get_move_index(position.move_number, position.mover)] = *position
}

#[inline(always)]
pub fn unmake_move(position: &mut Position, history: &PositionHistory) {
    *position = history.history[get_move_index(position.move_number, position.mover) - 1];
}

