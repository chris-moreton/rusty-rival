use crate::bitboards::{bit, test_bit, A1_BIT, A8_BIT, H1_BIT, H8_BIT};
use crate::hash::{
    en_passant_zobrist_key_index, ZOBRIST_KEYS_CASTLE, ZOBRIST_KEYS_EN_PASSANT, ZOBRIST_KEYS_PIECES, ZOBRIST_KEY_MOVER_SWITCH,
    ZOBRIST_PIECE_INDEX_BISHOP, ZOBRIST_PIECE_INDEX_KING, ZOBRIST_PIECE_INDEX_KNIGHT, ZOBRIST_PIECE_INDEX_PAWN, ZOBRIST_PIECE_INDEX_QUEEN,
    ZOBRIST_PIECE_INDEX_ROOK,
};
use crate::move_constants::*;
use crate::opponent;
use crate::types::{Bitboard, Move, Position, Square, UnmakeInfo, BLACK, WHITE};
use crate::utils::{from_square_part, to_square_part};

// Captured piece encoding for UnmakeInfo
pub const CAPTURED_NONE: u8 = 0;
pub const CAPTURED_PAWN: u8 = 1;
pub const CAPTURED_KNIGHT: u8 = 2;
pub const CAPTURED_BISHOP: u8 = 3;
pub const CAPTURED_ROOK: u8 = 4;
pub const CAPTURED_QUEEN: u8 = 5;
pub const CAPTURED_EP_PAWN: u8 = 6;  // En passant capture (pawn on different square)

#[inline(always)]
pub fn make_move(position: &Position, mv: Move, new_position: &mut Position) {
    let from = from_square_part(mv);
    let to = to_square_part(mv);

    let piece_mask = mv & PIECE_MASK_FULL;

    new_position.zobrist_lock ^= ZOBRIST_KEYS_CASTLE[new_position.castle_flags as usize];
    if position.en_passant_square != EN_PASSANT_NOT_AVAILABLE {
        new_position.zobrist_lock ^= ZOBRIST_KEYS_EN_PASSANT[en_passant_zobrist_key_index(position.en_passant_square)];
    }
    match piece_mask {
        PIECE_MASK_PAWN => {
            if mv & PROMOTION_FULL_MOVE_MASK != 0 {
                make_move_with_promotion(new_position, from, to, mv & PROMOTION_FULL_MOVE_MASK);
            } else if (from - to) % 8 != 0 {
                make_pawn_capture_move(new_position, from, to);
            } else {
                make_simple_pawn_move(new_position, from, to)
            }
            new_position.move_number += position.mover as u16;
        }
        PIECE_MASK_KING => {
            if mv >= BLACK_QUEEN_CASTLE_MOVE_MASK {
                make_castle_move(new_position, mv)
            } else {
                make_king_move(new_position, from, to);
            }

            new_position.en_passant_square = EN_PASSANT_NOT_AVAILABLE;
        }
        _ => {
            if test_bit(all_pieces(position), to) {
                make_non_pawn_non_king_capture_move(new_position, from, to, piece_mask);
            } else {
                make_non_pawn_non_king_non_capture_move(new_position, from, to, piece_mask);
            };
            new_position.en_passant_square = EN_PASSANT_NOT_AVAILABLE;
            new_position.move_number += position.mover as u16;
        }
    }

    new_position.mover ^= 1;
    new_position.zobrist_lock ^= ZOBRIST_KEYS_CASTLE[new_position.castle_flags as usize];
    if new_position.en_passant_square != EN_PASSANT_NOT_AVAILABLE {
        new_position.zobrist_lock ^= ZOBRIST_KEYS_EN_PASSANT[en_passant_zobrist_key_index(new_position.en_passant_square)];
    }
    new_position.zobrist_lock ^= ZOBRIST_KEY_MOVER_SWITCH;
}

#[inline(always)]
fn make_castle_move(position: &mut Position, mv: Move) {
    match mv {
        WHITE_KING_CASTLE_MOVE => {
            perform_castle(position, CASTLE_INDEX_WHITE_KING);
        }
        WHITE_QUEEN_CASTLE_MOVE => {
            perform_castle(position, CASTLE_INDEX_WHITE_QUEEN);
        }
        BLACK_KING_CASTLE_MOVE => {
            perform_castle(position, CASTLE_INDEX_BLACK_KING);
        }
        BLACK_QUEEN_CASTLE_MOVE => {
            perform_castle(position, CASTLE_INDEX_BLACK_QUEEN);
        }
        _ => {
            panic!("Was expecting a castle move");
        }
    };
}

#[inline(always)]
pub fn perform_castle(position: &mut Position, index: usize) {
    position.zobrist_lock ^= ZOBRIST_KEYS_CASTLE_PIECE_MOVES[index];

    position.castle_flags &= CASTLE_VARS_CLEAR_FLAGS_MASK[index];
    position.pieces[position.mover as usize].rook_bitboard ^= CASTLE_VARS_ROOK_MASK[index];
    position.pieces[position.mover as usize].all_pieces_bitboard ^= CASTLE_VARS_ALL_PIECES_MASK[index];
    position.pieces[position.mover as usize].king_square = CASTLE_VARS_KING_TO[index];
    position.half_moves += 1;
    position.move_number += CASTLE_VARS_FULL_MOVE_INC[index];
}

#[inline(always)]
fn make_simple_pawn_move(position: &mut Position, from: Square, to: Square) {
    let switch = bit(from) | bit(to);
    position.pieces[position.mover as usize].pawn_bitboard ^= switch;
    position.zobrist_lock ^= ZOBRIST_KEYS_PIECES[position.mover as usize][ZOBRIST_PIECE_INDEX_PAWN][from as usize]
        ^ ZOBRIST_KEYS_PIECES[position.mover as usize][ZOBRIST_PIECE_INDEX_PAWN][to as usize];

    position.pieces[position.mover as usize].all_pieces_bitboard ^= switch;

    position.en_passant_square = if from ^ to == 16 {
        from + if position.mover == WHITE { 8 } else { -8 }
    } else {
        EN_PASSANT_NOT_AVAILABLE
    };

    position.half_moves = 0;
}

#[inline(always)]
fn make_non_pawn_non_king_non_capture_move(position: &mut Position, from: Square, to: Square, piece_mask: Move) {
    let switch = bit(from) | bit(to);
    position.pieces[position.mover as usize].all_pieces_bitboard ^= switch;

    position.half_moves += 1;

    let m = position.mover as usize;

    match piece_mask {
        PIECE_MASK_KNIGHT => {
            position.zobrist_lock ^= ZOBRIST_KEYS_PIECES[m][ZOBRIST_PIECE_INDEX_KNIGHT][from as usize]
                ^ ZOBRIST_KEYS_PIECES[m][ZOBRIST_PIECE_INDEX_KNIGHT][to as usize];
            position.pieces[position.mover as usize].knight_bitboard ^= switch
        }
        PIECE_MASK_BISHOP => {
            position.zobrist_lock ^= ZOBRIST_KEYS_PIECES[m][ZOBRIST_PIECE_INDEX_BISHOP][from as usize]
                ^ ZOBRIST_KEYS_PIECES[m][ZOBRIST_PIECE_INDEX_BISHOP][to as usize];
            position.pieces[position.mover as usize].bishop_bitboard ^= switch
        }
        PIECE_MASK_ROOK => {
            position.zobrist_lock ^= ZOBRIST_KEYS_PIECES[m][ZOBRIST_PIECE_INDEX_ROOK][from as usize]
                ^ ZOBRIST_KEYS_PIECES[m][ZOBRIST_PIECE_INDEX_ROOK][to as usize];
            position.pieces[position.mover as usize].rook_bitboard ^= switch;
            update_castle_flags_if_square(position, from);
        }
        PIECE_MASK_QUEEN => {
            position.zobrist_lock ^= ZOBRIST_KEYS_PIECES[m][ZOBRIST_PIECE_INDEX_QUEEN][from as usize]
                ^ ZOBRIST_KEYS_PIECES[m][ZOBRIST_PIECE_INDEX_QUEEN][to as usize];
            position.pieces[position.mover as usize].queen_bitboard ^= switch
        }
        _ => panic!("Piece panic"),
    }
}

#[inline(always)]
pub fn make_move_with_promotion(position: &mut Position, from: Square, to: Square, promotion_mask: Move) {
    let bit_to = bit(to);
    let bit_from = bit(from);

    match promotion_mask {
        PROMOTION_KNIGHT_MOVE_MASK => {
            position.pieces[position.mover as usize].knight_bitboard |= bit_to;
            position.zobrist_lock ^= ZOBRIST_KEYS_PIECES[position.mover as usize][ZOBRIST_PIECE_INDEX_KNIGHT][to as usize];
        }
        PROMOTION_BISHOP_MOVE_MASK => {
            position.pieces[position.mover as usize].bishop_bitboard |= bit_to;
            position.zobrist_lock ^= ZOBRIST_KEYS_PIECES[position.mover as usize][ZOBRIST_PIECE_INDEX_BISHOP][to as usize];
        }
        PROMOTION_ROOK_MOVE_MASK => {
            position.pieces[position.mover as usize].rook_bitboard |= bit_to;
            position.zobrist_lock ^= ZOBRIST_KEYS_PIECES[position.mover as usize][ZOBRIST_PIECE_INDEX_ROOK][to as usize];
        }
        PROMOTION_QUEEN_MOVE_MASK => {
            position.pieces[position.mover as usize].queen_bitboard |= bit_to;
            position.zobrist_lock ^= ZOBRIST_KEYS_PIECES[position.mover as usize][ZOBRIST_PIECE_INDEX_QUEEN][to as usize];
        }
        _ => panic!("Invalid promotion piece"),
    }

    position.pieces[position.mover as usize].pawn_bitboard ^= bit_from;

    let opponent = opponent!(position.mover) as usize;

    position.zobrist_lock ^= ZOBRIST_KEYS_PIECES[position.mover as usize][ZOBRIST_PIECE_INDEX_PAWN][from as usize];

    position.pieces[position.mover as usize].all_pieces_bitboard ^= bit_from | bit_to;

    if position.pieces[opponent].all_pieces_bitboard & bit_to != 0 {
        let enemy = &mut position.pieces[opponent];

        if enemy.knight_bitboard & bit_to != 0 {
            enemy.knight_bitboard &= !bit_to;
            position.zobrist_lock ^= ZOBRIST_KEYS_PIECES[opponent][ZOBRIST_PIECE_INDEX_KNIGHT][to as usize];
        }
        if enemy.rook_bitboard & bit_to != 0 {
            enemy.rook_bitboard &= !bit_to;
            position.zobrist_lock ^= ZOBRIST_KEYS_PIECES[opponent][ZOBRIST_PIECE_INDEX_ROOK][to as usize];
        }
        if enemy.bishop_bitboard & bit_to != 0 {
            enemy.bishop_bitboard &= !bit_to;
            position.zobrist_lock ^= ZOBRIST_KEYS_PIECES[opponent][ZOBRIST_PIECE_INDEX_BISHOP][to as usize];
        }
        if enemy.queen_bitboard & bit_to != 0 {
            enemy.queen_bitboard &= !bit_to;
            position.zobrist_lock ^= ZOBRIST_KEYS_PIECES[opponent][ZOBRIST_PIECE_INDEX_QUEEN][to as usize];
        }
        enemy.all_pieces_bitboard &= !bit_to;
    }

    update_castle_flags_if_square(position, to);

    position.en_passant_square = EN_PASSANT_NOT_AVAILABLE;

    position.half_moves = 0;
}

#[inline(always)]
fn update_castle_flags_if_square(position: &mut Position, sq: Square) {
    if position.castle_flags != 0 {
        match sq {
            H8_BIT => position.castle_flags &= !BK_CASTLE,
            A8_BIT => position.castle_flags &= !BQ_CASTLE,
            H1_BIT => position.castle_flags &= !WK_CASTLE,
            A1_BIT => position.castle_flags &= !WQ_CASTLE,
            _ => {}
        }
    }
}

#[inline(always)]
pub fn make_non_pawn_non_king_capture_move(position: &mut Position, from: Square, to: Square, piece: Move) {
    let to_mask = bit(to);

    remove_captured_piece(position, to);

    let switch = bit(from) | to_mask;
    let m = position.mover as usize;
    position.pieces[position.mover as usize].all_pieces_bitboard ^= switch;

    match piece {
        PIECE_MASK_KNIGHT => {
            position.zobrist_lock ^= ZOBRIST_KEYS_PIECES[m][ZOBRIST_PIECE_INDEX_KNIGHT][from as usize]
                ^ ZOBRIST_KEYS_PIECES[m][ZOBRIST_PIECE_INDEX_KNIGHT][to as usize];
            position.pieces[position.mover as usize].knight_bitboard ^= switch
        }
        PIECE_MASK_BISHOP => {
            position.zobrist_lock ^= ZOBRIST_KEYS_PIECES[m][ZOBRIST_PIECE_INDEX_BISHOP][from as usize]
                ^ ZOBRIST_KEYS_PIECES[m][ZOBRIST_PIECE_INDEX_BISHOP][to as usize];
            position.pieces[position.mover as usize].bishop_bitboard ^= switch
        }
        PIECE_MASK_ROOK => {
            position.zobrist_lock ^= ZOBRIST_KEYS_PIECES[m][ZOBRIST_PIECE_INDEX_ROOK][from as usize]
                ^ ZOBRIST_KEYS_PIECES[m][ZOBRIST_PIECE_INDEX_ROOK][to as usize];
            update_castle_flags_if_square(position, from);
            position.pieces[position.mover as usize].rook_bitboard ^= switch
        }
        PIECE_MASK_QUEEN => {
            position.zobrist_lock ^= ZOBRIST_KEYS_PIECES[m][ZOBRIST_PIECE_INDEX_QUEEN][from as usize]
                ^ ZOBRIST_KEYS_PIECES[m][ZOBRIST_PIECE_INDEX_QUEEN][to as usize];
            position.pieces[position.mover as usize].queen_bitboard ^= switch
        }
        _ => panic!("Unexpected piece"),
    }

    update_castle_flags_if_square(position, to);
}

#[inline(always)]
pub fn make_pawn_capture_move(position: &mut Position, from: Square, to: Square) {
    let opponent = opponent!(position.mover) as usize;
    let enemy = &mut position.pieces[opponent];

    position.half_moves = 0;

    let bit_to = bit(to);

    if position.en_passant_square == to {
        let pawn_off = EN_PASSANT_CAPTURE_MASK[to as usize];

        let epcps = en_passant_captured_piece_square(position.en_passant_square) as usize;
        position.zobrist_lock ^= ZOBRIST_KEYS_PIECES[opponent][ZOBRIST_PIECE_INDEX_PAWN][epcps];

        enemy.pawn_bitboard &= pawn_off;
        enemy.all_pieces_bitboard &= pawn_off;
    } else {
        if enemy.pawn_bitboard & bit_to != 0 {
            enemy.pawn_bitboard &= !bit_to;
            position.zobrist_lock ^= ZOBRIST_KEYS_PIECES[opponent][ZOBRIST_PIECE_INDEX_PAWN][to as usize];
        }
        if enemy.knight_bitboard & bit_to != 0 {
            enemy.knight_bitboard &= !bit_to;
            position.zobrist_lock ^= ZOBRIST_KEYS_PIECES[opponent][ZOBRIST_PIECE_INDEX_KNIGHT][to as usize];
        }
        if enemy.rook_bitboard & bit_to != 0 {
            enemy.rook_bitboard &= !bit_to;
            position.zobrist_lock ^= ZOBRIST_KEYS_PIECES[opponent][ZOBRIST_PIECE_INDEX_ROOK][to as usize];
        }
        if enemy.bishop_bitboard & bit_to != 0 {
            enemy.bishop_bitboard &= !bit_to;
            position.zobrist_lock ^= ZOBRIST_KEYS_PIECES[opponent][ZOBRIST_PIECE_INDEX_BISHOP][to as usize];
        }
        if enemy.queen_bitboard & bit_to != 0 {
            enemy.queen_bitboard &= !bit_to;
            position.zobrist_lock ^= ZOBRIST_KEYS_PIECES[opponent][ZOBRIST_PIECE_INDEX_QUEEN][to as usize];
        }
        enemy.all_pieces_bitboard &= !bit_to;
    }

    let switch = bit(from) | bit_to;
    position.pieces[position.mover as usize].all_pieces_bitboard ^= switch;
    position.pieces[position.mover as usize].pawn_bitboard ^= switch;

    position.zobrist_lock ^= ZOBRIST_KEYS_PIECES[position.mover as usize][ZOBRIST_PIECE_INDEX_PAWN][from as usize]
        ^ ZOBRIST_KEYS_PIECES[position.mover as usize][ZOBRIST_PIECE_INDEX_PAWN][to as usize];

    position.en_passant_square = EN_PASSANT_NOT_AVAILABLE;

    update_castle_flags_if_square(position, to);
}

#[inline(always)]
pub fn make_king_move(position: &mut Position, from: Square, to: Square) {
    let to_mask = bit(to);

    remove_captured_piece(position, to);

    let friendly = &mut position.pieces[position.mover as usize];

    let switch = bit(from) | to_mask;
    friendly.all_pieces_bitboard ^= switch;

    position.castle_flags &= CLEAR_CASTLE_FLAGS_MASK[position.mover as usize];
    friendly.king_square = to;

    position.move_number += position.mover as u16;

    position.zobrist_lock ^= ZOBRIST_KEYS_PIECES[position.mover as usize][ZOBRIST_PIECE_INDEX_KING][from as usize]
        ^ ZOBRIST_KEYS_PIECES[position.mover as usize][ZOBRIST_PIECE_INDEX_KING][to as usize];
}

#[inline(always)]
fn remove_captured_piece(position: &mut Position, to: Square) {
    let bit_to = bit(to);

    if (position.pieces[WHITE as usize].all_pieces_bitboard | position.pieces[BLACK as usize].all_pieces_bitboard) & bit_to != 0 {
        let opponent = opponent!(position.mover) as usize;
        let enemy = &mut position.pieces[opponent];
        position.half_moves = 0;

        if enemy.pawn_bitboard & bit_to != 0 {
            enemy.pawn_bitboard &= !bit_to;
            position.zobrist_lock ^= ZOBRIST_KEYS_PIECES[opponent][ZOBRIST_PIECE_INDEX_PAWN][to as usize];
        }
        if enemy.knight_bitboard & bit_to != 0 {
            enemy.knight_bitboard &= !bit_to;
            position.zobrist_lock ^= ZOBRIST_KEYS_PIECES[opponent][ZOBRIST_PIECE_INDEX_KNIGHT][to as usize];
        }
        if enemy.rook_bitboard & bit_to != 0 {
            enemy.rook_bitboard &= !bit_to;
            position.zobrist_lock ^= ZOBRIST_KEYS_PIECES[opponent][ZOBRIST_PIECE_INDEX_ROOK][to as usize];
        }
        if enemy.bishop_bitboard & bit_to != 0 {
            enemy.bishop_bitboard &= !bit_to;
            position.zobrist_lock ^= ZOBRIST_KEYS_PIECES[opponent][ZOBRIST_PIECE_INDEX_BISHOP][to as usize];
        }
        if enemy.queen_bitboard & bit_to != 0 {
            enemy.queen_bitboard &= !bit_to;
            position.zobrist_lock ^= ZOBRIST_KEYS_PIECES[opponent][ZOBRIST_PIECE_INDEX_QUEEN][to as usize];
        }

        enemy.all_pieces_bitboard &= !bit_to;
    } else {
        position.half_moves += 1;
    }
}

#[inline(always)]
pub fn en_passant_captured_piece_square(square: Square) -> Square {
    square + if square < 40 { 8 } else { -8 }
}

#[inline(always)]
pub fn all_pieces(position: &Position) -> Bitboard {
    position.pieces[WHITE as usize].all_pieces_bitboard | position.pieces[BLACK as usize].all_pieces_bitboard
}

/// Make a move in-place and return info needed to unmake it
#[inline(always)]
pub fn make_move_in_place(position: &mut Position, mv: Move) -> UnmakeInfo {
    // Save state for unmake
    let unmake = UnmakeInfo {
        castle_flags: position.castle_flags,
        en_passant_square: position.en_passant_square,
        half_moves: position.half_moves,
        zobrist_lock: position.zobrist_lock,
        captured_piece: get_captured_piece(position, mv),
    };

    let from = from_square_part(mv);
    let to = to_square_part(mv);
    let piece_mask = mv & PIECE_MASK_FULL;

    position.zobrist_lock ^= ZOBRIST_KEYS_CASTLE[position.castle_flags as usize];
    if position.en_passant_square != EN_PASSANT_NOT_AVAILABLE {
        position.zobrist_lock ^= ZOBRIST_KEYS_EN_PASSANT[en_passant_zobrist_key_index(position.en_passant_square)];
    }

    match piece_mask {
        PIECE_MASK_PAWN => {
            if mv & PROMOTION_FULL_MOVE_MASK != 0 {
                make_move_with_promotion(position, from, to, mv & PROMOTION_FULL_MOVE_MASK);
            } else if (from - to) % 8 != 0 {
                make_pawn_capture_move(position, from, to);
            } else {
                make_simple_pawn_move(position, from, to)
            }
            position.move_number += position.mover as u16;
        }
        PIECE_MASK_KING => {
            if mv >= BLACK_QUEEN_CASTLE_MOVE_MASK {
                make_castle_move(position, mv)
            } else {
                make_king_move(position, from, to);
            }
            position.en_passant_square = EN_PASSANT_NOT_AVAILABLE;
        }
        _ => {
            if test_bit(all_pieces(position), to) {
                make_non_pawn_non_king_capture_move(position, from, to, piece_mask);
            } else {
                make_non_pawn_non_king_non_capture_move(position, from, to, piece_mask);
            };
            position.en_passant_square = EN_PASSANT_NOT_AVAILABLE;
            position.move_number += position.mover as u16;
        }
    }

    position.mover ^= 1;
    position.zobrist_lock ^= ZOBRIST_KEYS_CASTLE[position.castle_flags as usize];
    if position.en_passant_square != EN_PASSANT_NOT_AVAILABLE {
        position.zobrist_lock ^= ZOBRIST_KEYS_EN_PASSANT[en_passant_zobrist_key_index(position.en_passant_square)];
    }
    position.zobrist_lock ^= ZOBRIST_KEY_MOVER_SWITCH;

    unmake
}

/// Get the captured piece type for a move (0 if no capture)
#[inline(always)]
fn get_captured_piece(position: &Position, mv: Move) -> u8 {
    let to = to_square_part(mv);
    let piece_mask = mv & PIECE_MASK_FULL;

    // Check for en passant capture
    if piece_mask == PIECE_MASK_PAWN && to == position.en_passant_square && position.en_passant_square != EN_PASSANT_NOT_AVAILABLE {
        return CAPTURED_EP_PAWN;
    }

    let bit_to = bit(to);
    let opponent = opponent!(position.mover) as usize;
    let enemy = &position.pieces[opponent];

    if enemy.all_pieces_bitboard & bit_to == 0 {
        return CAPTURED_NONE;
    }

    if enemy.pawn_bitboard & bit_to != 0 { return CAPTURED_PAWN; }
    if enemy.knight_bitboard & bit_to != 0 { return CAPTURED_KNIGHT; }
    if enemy.bishop_bitboard & bit_to != 0 { return CAPTURED_BISHOP; }
    if enemy.rook_bitboard & bit_to != 0 { return CAPTURED_ROOK; }
    if enemy.queen_bitboard & bit_to != 0 { return CAPTURED_QUEEN; }

    CAPTURED_NONE
}

/// Unmake a move, restoring the position to its previous state
#[inline(always)]
pub fn unmake_move(position: &mut Position, mv: Move, unmake: &UnmakeInfo) {
    // Flip mover back (the move was made by the opponent of current mover)
    position.mover ^= 1;

    let from = from_square_part(mv);
    let to = to_square_part(mv);
    let piece_mask = mv & PIECE_MASK_FULL;
    let mover = position.mover as usize;
    let opponent = opponent!(position.mover) as usize;

    match piece_mask {
        PIECE_MASK_PAWN => {
            if mv & PROMOTION_FULL_MOVE_MASK != 0 {
                // Unmake promotion: remove promoted piece, restore pawn
                unmake_promotion(position, from, to, mv & PROMOTION_FULL_MOVE_MASK, mover);
            } else {
                // Move pawn back
                let switch = bit(from) | bit(to);
                position.pieces[mover].pawn_bitboard ^= switch;
                position.pieces[mover].all_pieces_bitboard ^= switch;
            }
            position.move_number -= position.mover as u16;
        }
        PIECE_MASK_KING => {
            if mv >= BLACK_QUEEN_CASTLE_MOVE_MASK {
                unmake_castle(position, mv, mover);
            } else {
                // Move king back
                let switch = bit(from) | bit(to);
                position.pieces[mover].all_pieces_bitboard ^= switch;
                position.pieces[mover].king_square = from;
            }
            position.move_number -= position.mover as u16;
        }
        PIECE_MASK_KNIGHT => {
            let switch = bit(from) | bit(to);
            position.pieces[mover].knight_bitboard ^= switch;
            position.pieces[mover].all_pieces_bitboard ^= switch;
            position.move_number -= position.mover as u16;
        }
        PIECE_MASK_BISHOP => {
            let switch = bit(from) | bit(to);
            position.pieces[mover].bishop_bitboard ^= switch;
            position.pieces[mover].all_pieces_bitboard ^= switch;
            position.move_number -= position.mover as u16;
        }
        PIECE_MASK_ROOK => {
            let switch = bit(from) | bit(to);
            position.pieces[mover].rook_bitboard ^= switch;
            position.pieces[mover].all_pieces_bitboard ^= switch;
            position.move_number -= position.mover as u16;
        }
        PIECE_MASK_QUEEN => {
            let switch = bit(from) | bit(to);
            position.pieces[mover].queen_bitboard ^= switch;
            position.pieces[mover].all_pieces_bitboard ^= switch;
            position.move_number -= position.mover as u16;
        }
        _ => {}
    }

    // Restore captured piece
    restore_captured_piece(position, to, unmake.captured_piece, opponent, unmake.en_passant_square);

    // Restore saved state
    position.castle_flags = unmake.castle_flags;
    position.en_passant_square = unmake.en_passant_square;
    position.half_moves = unmake.half_moves;
    position.zobrist_lock = unmake.zobrist_lock;
}

#[inline(always)]
fn unmake_promotion(position: &mut Position, from: Square, to: Square, promotion_mask: Move, mover: usize) {
    let bit_to = bit(to);
    let bit_from = bit(from);

    // Remove promoted piece
    match promotion_mask {
        PROMOTION_KNIGHT_MOVE_MASK => position.pieces[mover].knight_bitboard &= !bit_to,
        PROMOTION_BISHOP_MOVE_MASK => position.pieces[mover].bishop_bitboard &= !bit_to,
        PROMOTION_ROOK_MOVE_MASK => position.pieces[mover].rook_bitboard &= !bit_to,
        PROMOTION_QUEEN_MOVE_MASK => position.pieces[mover].queen_bitboard &= !bit_to,
        _ => {}
    }

    // Restore pawn at from square
    position.pieces[mover].pawn_bitboard |= bit_from;
    position.pieces[mover].all_pieces_bitboard ^= bit_from | bit_to;
}

#[inline(always)]
fn unmake_castle(position: &mut Position, mv: Move, mover: usize) {
    match mv {
        WHITE_KING_CASTLE_MOVE => {
            position.pieces[mover].king_square = 4;  // e1
            position.pieces[mover].rook_bitboard ^= bit(5) | bit(7);  // f1, h1
            position.pieces[mover].all_pieces_bitboard ^= bit(4) | bit(5) | bit(6) | bit(7);
        }
        WHITE_QUEEN_CASTLE_MOVE => {
            position.pieces[mover].king_square = 4;  // e1
            position.pieces[mover].rook_bitboard ^= bit(0) | bit(3);  // a1, d1
            position.pieces[mover].all_pieces_bitboard ^= bit(0) | bit(2) | bit(3) | bit(4);
        }
        BLACK_KING_CASTLE_MOVE => {
            position.pieces[mover].king_square = 60;  // e8
            position.pieces[mover].rook_bitboard ^= bit(61) | bit(63);  // f8, h8
            position.pieces[mover].all_pieces_bitboard ^= bit(60) | bit(61) | bit(62) | bit(63);
        }
        BLACK_QUEEN_CASTLE_MOVE => {
            position.pieces[mover].king_square = 60;  // e8
            position.pieces[mover].rook_bitboard ^= bit(56) | bit(59);  // a8, d8
            position.pieces[mover].all_pieces_bitboard ^= bit(56) | bit(58) | bit(59) | bit(60);
        }
        _ => {}
    }
}

#[inline(always)]
fn restore_captured_piece(position: &mut Position, to: Square, captured: u8, opponent: usize, ep_square: Square) {
    if captured == CAPTURED_NONE {
        return;
    }

    let restore_square = if captured == CAPTURED_EP_PAWN {
        en_passant_captured_piece_square(ep_square)
    } else {
        to
    };

    let bit_restore = bit(restore_square);
    position.pieces[opponent].all_pieces_bitboard |= bit_restore;

    match captured {
        CAPTURED_PAWN | CAPTURED_EP_PAWN => position.pieces[opponent].pawn_bitboard |= bit_restore,
        CAPTURED_KNIGHT => position.pieces[opponent].knight_bitboard |= bit_restore,
        CAPTURED_BISHOP => position.pieces[opponent].bishop_bitboard |= bit_restore,
        CAPTURED_ROOK => position.pieces[opponent].rook_bitboard |= bit_restore,
        CAPTURED_QUEEN => position.pieces[opponent].queen_bitboard |= bit_restore,
        _ => {}
    }
}

