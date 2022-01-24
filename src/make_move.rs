use crate::bitboards::{A1_BIT, A8_BIT, bit, C1_BIT, C8_BIT, clear_bit, D1_BIT, D8_BIT, E1_BIT, E8_BIT, F1_BIT, F8_BIT, G1_BIT, G8_BIT, H1_BIT, H8_BIT, test_bit};
use crate::move_constants::*;
use crate::{move_mover, opponent};
use crate::types::{Bitboard, BLACK, is_any_black_castle_available, is_any_white_castle_available, Move, Mover, Pieces, Piece, Position, PositionHistory, Square, unset_bk_castle, unset_black_castles, unset_bq_castle, unset_white_castles, unset_wk_castle, unset_wq_castle, WHITE};
use crate::types::Piece::{Bishop, Empty, King, Knight, Pawn, Queen, Rook};
use crate::utils::{from_square_part, to_square_part};

pub fn make_move(position: &mut Position, mv: Move, history: &mut PositionHistory) {

    let from = from_square_part(mv);
    let to = to_square_part(mv);
    let to_mask = bit(to);
    let from_mask = bit(from);

    let piece = moving_piece(&position, from_mask);

    store_history(position, history);
    if (position.pieces[WHITE as usize].all_pieces_bitboard | position.pieces[BLACK as usize].all_pieces_bitboard) & to_mask != 0 ||
        (piece == Pawn && ((from - to) % 8 != 0 || PROMOTION_SQUARES & to_mask != 0)) ||
        (piece == King && KING_START_POSITIONS & from_mask != 0) {
        let promoted_piece = promotion_piece_from_move(mv);
        if promoted_piece != Empty {
            make_move_with_promotion(position, from, to, promoted_piece);
        } else if from == E1_BIT && (to == G1_BIT || to == C1_BIT) && is_any_white_castle_available(position) {
            make_castle_move(position, to);
        } else if from == E8_BIT && (to == G8_BIT || to == C8_BIT) && is_any_black_castle_available(position) {
            make_castle_move(position, to);
        } else {
            make_simple_complex_move(position, from, to)
        }
        position.en_passant_square = EN_PASSANT_NOT_AVAILABLE;
    } else {
        make_simple_move(position, from, to, piece)
    };

    if position.mover == WHITE {
        position.mover = BLACK;
    } else {
        position.move_number += 1;
        position.mover = WHITE;
    }
}

fn make_simple_move(position: &mut Position, from: Square, to: Square, piece: Piece) {
    let switch_bitboard = bit(from) | bit(to);
    let friendly = &mut position.pieces[position.mover as usize];
    friendly.all_pieces_bitboard ^= switch_bitboard;

    if piece == Pawn {
        friendly.pawn_bitboard = clear_bit(friendly.pawn_bitboard, from) | bit(to);

        position.en_passant_square = if position.mover == WHITE {
            if to - from == 16 { from + 8 } else { EN_PASSANT_NOT_AVAILABLE }
        } else {
            if from - to == 16 { from - 8 } else { EN_PASSANT_NOT_AVAILABLE }
        };

        position.half_moves = 0;
    } else {
        position.half_moves += 1;
        position.en_passant_square = EN_PASSANT_NOT_AVAILABLE;
        match piece {
            Knight => friendly.knight_bitboard = clear_bit(friendly.knight_bitboard, from) | bit(to),
            Bishop => friendly.bishop_bitboard = clear_bit(friendly.bishop_bitboard, from) | bit(to),
            Rook => {
                friendly.rook_bitboard = clear_bit(friendly.rook_bitboard, from) | bit(to);
                if from == H1_BIT { unset_wk_castle(position) }
                if from == A1_BIT { unset_wq_castle(position) }
                if from == H8_BIT { unset_bk_castle(position) }
                if from == A8_BIT { unset_bq_castle(position) }
            },
            Queen => friendly.queen_bitboard = clear_bit(friendly.queen_bitboard, from) | bit(to),
            King => friendly.king_square = to,
            _ => panic!("Piece panic")
        }
    }
}

#[inline(always)]
pub fn make_castle_move(position: &mut Position, to: Square) {
    let offset = if position.mover == WHITE { 0 } else { 56 };
    let friendly = &mut position.pieces[position.mover as usize];
    
    friendly.rook_bitboard = if to == C1_BIT + offset {
        friendly.king_square = C1_BIT + offset;
        clear_bit(friendly.rook_bitboard, A1_BIT + offset) | bit(D1_BIT + offset)
    } else {
        friendly.king_square = G1_BIT + offset;
        clear_bit(friendly.rook_bitboard, H1_BIT + offset) | bit(F1_BIT + offset)
    };

    friendly.all_pieces_bitboard = friendly.rook_bitboard | bit(friendly.king_square) | friendly.queen_bitboard | friendly.knight_bitboard | friendly.bishop_bitboard | friendly.pawn_bitboard;

    if position.mover == WHITE {
        position.castle_flags &= !(WK_CASTLE | WQ_CASTLE)
    } else {
        position.castle_flags &= !(BK_CASTLE | BQ_CASTLE)
    }

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
pub fn make_move_with_promotion(position: &mut Position, from: Square, to: Square, promotion_piece: Piece) {

    let bit_to = bit(to);
    let not_bit_to = !bit_to;
    let not_bit_from = !bit(from);

    let is_capture = position.pieces[opponent!(position.mover) as usize].all_pieces_bitboard & bit_to != 0;
    let friendly = &mut position.pieces[position.mover as usize];

    match promotion_piece {
        Knight => friendly.knight_bitboard |= bit_to,
        Bishop => friendly.bishop_bitboard |= bit_to,
        Rook => friendly.rook_bitboard |= bit_to,
        Queen => friendly.queen_bitboard |= bit_to,
        _ => panic!("Invalid promotion piece")
    }

    friendly.pawn_bitboard &= not_bit_from;
    friendly.all_pieces_bitboard &= not_bit_from;
    friendly.all_pieces_bitboard |= bit_to;

    if is_capture {
        let enemy = &mut position.pieces[opponent!(position.mover) as usize];
        enemy.knight_bitboard &= not_bit_to;
        enemy.rook_bitboard &= not_bit_to;
        enemy.bishop_bitboard &= not_bit_to;
        enemy.queen_bitboard &= not_bit_to;
        enemy.all_pieces_bitboard &= not_bit_to;
    }

    if to == H8_BIT { unset_bk_castle(position) }
    if to == A8_BIT { unset_bq_castle(position) }
    if to == H1_BIT { unset_wk_castle(position) }
    if to == A1_BIT { unset_wq_castle(position) }

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
pub fn make_simple_complex_move(position: &mut Position, from: Square, to: Square) {

    let to_mask = bit(to);
    let from_mask = bit(from);
    let all_pieces = position.pieces[WHITE as usize].all_pieces_bitboard | position.pieces[BLACK as usize].all_pieces_bitboard;

    let friendly = position.pieces[position.mover as usize];
    let enemy = &mut position.pieces[opponent!(position.mover) as usize];

    if all_pieces & to_mask != 0 {
        let to_mask_inverted = !to_mask;
        enemy.pawn_bitboard &= to_mask_inverted;
        enemy.knight_bitboard &= to_mask_inverted;
        enemy.bishop_bitboard &= to_mask_inverted;
        enemy.rook_bitboard &= to_mask_inverted;
        enemy.queen_bitboard &= to_mask_inverted;
        enemy.all_pieces_bitboard &= to_mask_inverted;
    }

    let is_pawn_move = friendly.pawn_bitboard & from_mask != 0;
    if position.en_passant_square == to && is_pawn_move {
        let pawn_off = !bit(en_passant_captured_piece_square(to));
        enemy.pawn_bitboard &= pawn_off;
        enemy.all_pieces_bitboard &= pawn_off;
    }

    let friendly = &mut position.pieces[position.mover as usize];

    move_mover!(friendly.pawn_bitboard, from_mask, to_mask, friendly);
    move_mover!(friendly.knight_bitboard, from_mask, to_mask, friendly);
    move_mover!(friendly.bishop_bitboard, from_mask, to_mask, friendly);
    move_mover!(friendly.rook_bitboard, from_mask, to_mask, friendly);
    move_mover!(friendly.queen_bitboard, from_mask, to_mask, friendly);

    if friendly.king_square == from {
        friendly.king_square = to;
        friendly.all_pieces_bitboard ^= from_mask | to_mask;
    }
    position.half_moves = if all_pieces & to_mask != 0 || is_pawn_move { 0 } else { position.half_moves + 1 };

    if from == E1_BIT || from == H1_BIT { unset_wk_castle(position) }
    if from == E1_BIT || from == A1_BIT { unset_wq_castle(position) }
    if to == H8_BIT { unset_bk_castle(position) }
    if to == A8_BIT { unset_bq_castle(position) }

    if from == E8_BIT || from == H8_BIT { unset_bk_castle(position) }
    if from == E8_BIT || from == A8_BIT { unset_bq_castle(position) }
    if to == H1_BIT { unset_wk_castle(position) }
    if to == A1_BIT { unset_wq_castle(position) }
}

#[inline(always)]
pub fn moving_piece(position: &Position, from_bb: Bitboard) -> Piece {
    if position.mover == WHITE {
        if position.pieces[WHITE as usize].pawn_bitboard & from_bb != 0 { Pawn }
        else if position.pieces[WHITE as usize].knight_bitboard & from_bb != 0 { Knight }
        else if position.pieces[WHITE as usize].bishop_bitboard & from_bb != 0 { Bishop }
        else if position.pieces[WHITE as usize].rook_bitboard & from_bb != 0  { Rook }
        else if position.pieces[WHITE as usize].queen_bitboard & from_bb != 0  { Queen }
        else { King }
    } else if position.pieces[BLACK as usize].pawn_bitboard & from_bb != 0 { Pawn }
        else if position.pieces[BLACK as usize].knight_bitboard & from_bb != 0 { Knight }
        else if position.pieces[BLACK as usize].bishop_bitboard & from_bb != 0 { Bishop }
        else if position.pieces[BLACK as usize].rook_bitboard & from_bb != 0 { Rook }
        else if position.pieces[BLACK as usize].queen_bitboard & from_bb != 0 { Queen }
        else { King }
}

#[inline(always)]
pub fn is_complex_move(position: &mut Position, from: Square, to: Square, piece: Piece) -> bool {
    test_bit(position.pieces[WHITE as usize].all_pieces_bitboard | position.pieces[BLACK as usize].all_pieces_bitboard, to) ||
        (piece == Pawn && is_complex_pawn_move(from, to)) ||
            (piece == King && test_bit(KING_START_POSITIONS, from))
}

#[inline(always)]
pub fn is_complex_pawn_move(from: Square, to: Square) -> bool {
    (from - to).abs() % 8 != 0 || test_bit(PROMOTION_SQUARES, to)
}

pub fn default_position_history() -> PositionHistory {
        [Position {
            pieces: [
                Pieces { pawn_bitboard: 0, knight_bitboard: 0, bishop_bitboard: 0, queen_bitboard: 0, king_square: 0, rook_bitboard: 0, all_pieces_bitboard: 0 },
                Pieces { pawn_bitboard: 0, knight_bitboard: 0, bishop_bitboard: 0, queen_bitboard: 0, king_square: 0, rook_bitboard: 0, all_pieces_bitboard: 0 },
            ],
            mover: WHITE,
            en_passant_square: EN_PASSANT_NOT_AVAILABLE,
            castle_flags: 0,
            half_moves: 0,
            move_number: 1
        }; MAX_MOVE_HISTORY as usize]
}

#[inline(always)]
pub fn get_move_index(move_number: u16, mover: Mover) -> usize {
    (move_number * 2 - if mover == WHITE { 1 } else { 0 }) as usize
}

#[inline(always)]
pub fn store_history(position: &mut Position, history: &mut PositionHistory) {
    history[get_move_index(position.move_number, position.mover)] = *position
}

#[inline(always)]
pub fn unmake_move(position: &mut Position, history: &PositionHistory) {
    *position = history[get_move_index(position.move_number, position.mover) - 1];
}

