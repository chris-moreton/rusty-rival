use crate::bitboards::{bit, test_bit, A1_BIT, A8_BIT, H1_BIT, H8_BIT};
use crate::hash::{
    en_passant_zobrist_key_index, ZOBRIST_KEYS_CASTLE, ZOBRIST_KEYS_EN_PASSANT,
    ZOBRIST_KEYS_PIECES, ZOBRIST_KEY_MOVER_SWITCH, ZOBRIST_PIECE_INDEX_BISHOP,
    ZOBRIST_PIECE_INDEX_KING, ZOBRIST_PIECE_INDEX_KNIGHT, ZOBRIST_PIECE_INDEX_PAWN,
    ZOBRIST_PIECE_INDEX_QUEEN, ZOBRIST_PIECE_INDEX_ROOK,
};
use crate::move_constants::*;
use crate::opponent;
use crate::types::{Bitboard, Move, Position, Square, BLACK, WHITE};
use crate::utils::{from_square_part, to_square_part};

#[inline(always)]
pub fn make_move(position: &Position, mv: Move, new_position: &mut Position) {
    // println!("{}", algebraic_move_from_move(mv));

    let from = from_square_part(mv);
    let to = to_square_part(mv);

    let piece_mask = mv & PIECE_MASK_FULL;

    new_position.zobrist_lock ^= ZOBRIST_KEYS_CASTLE[new_position.castle_flags as usize];
    if position.en_passant_square != EN_PASSANT_NOT_AVAILABLE {
        new_position.zobrist_lock ^=
            ZOBRIST_KEYS_EN_PASSANT[en_passant_zobrist_key_index(position.en_passant_square)];
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
        new_position.zobrist_lock ^=
            ZOBRIST_KEYS_EN_PASSANT[en_passant_zobrist_key_index(new_position.en_passant_square)];
    }
    new_position.zobrist_lock ^= ZOBRIST_KEY_MOVER_SWITCH;

    //assert_eq!(new_position.zobrist_lock, zobrist_lock(new_position));
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
    position.pieces[position.mover as usize].all_pieces_bitboard ^=
        CASTLE_VARS_ALL_PIECES_MASK[index];
    position.pieces[position.mover as usize].king_square = CASTLE_VARS_KING_TO[index];
    position.half_moves += 1;
    position.move_number += CASTLE_VARS_FULL_MOVE_INC[index];
}

#[inline(always)]
fn make_simple_pawn_move(position: &mut Position, from: Square, to: Square) {
    let switch = bit(from) | bit(to);
    position.pieces[position.mover as usize].pawn_bitboard ^= switch;
    position.zobrist_lock ^= ZOBRIST_KEYS_PIECES[position.mover as usize][ZOBRIST_PIECE_INDEX_PAWN]
        [from as usize]
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
fn make_non_pawn_non_king_non_capture_move(
    position: &mut Position,
    from: Square,
    to: Square,
    piece_mask: Move,
) {
    let switch = bit(from) | bit(to);
    position.pieces[position.mover as usize].all_pieces_bitboard ^= switch;

    position.half_moves += 1;

    let m = position.mover as usize;

    match piece_mask {
        PIECE_MASK_KNIGHT => {
            position.zobrist_lock ^= ZOBRIST_KEYS_PIECES[m][ZOBRIST_PIECE_INDEX_KNIGHT]
                [from as usize]
                ^ ZOBRIST_KEYS_PIECES[m][ZOBRIST_PIECE_INDEX_KNIGHT][to as usize];
            position.pieces[position.mover as usize].knight_bitboard ^= switch
        }
        PIECE_MASK_BISHOP => {
            position.zobrist_lock ^= ZOBRIST_KEYS_PIECES[m][ZOBRIST_PIECE_INDEX_BISHOP]
                [from as usize]
                ^ ZOBRIST_KEYS_PIECES[m][ZOBRIST_PIECE_INDEX_BISHOP][to as usize];
            position.pieces[position.mover as usize].bishop_bitboard ^= switch
        }
        PIECE_MASK_ROOK => {
            position.zobrist_lock ^= ZOBRIST_KEYS_PIECES[m][ZOBRIST_PIECE_INDEX_ROOK]
                [from as usize]
                ^ ZOBRIST_KEYS_PIECES[m][ZOBRIST_PIECE_INDEX_ROOK][to as usize];
            position.pieces[position.mover as usize].rook_bitboard ^= switch;
            update_castle_flags_if_square(position, from);
        }
        PIECE_MASK_QUEEN => {
            position.zobrist_lock ^= ZOBRIST_KEYS_PIECES[m][ZOBRIST_PIECE_INDEX_QUEEN]
                [from as usize]
                ^ ZOBRIST_KEYS_PIECES[m][ZOBRIST_PIECE_INDEX_QUEEN][to as usize];
            position.pieces[position.mover as usize].queen_bitboard ^= switch
        }
        _ => panic!("Piece panic"),
    }
}

#[inline(always)]
pub fn make_move_with_promotion(
    position: &mut Position,
    from: Square,
    to: Square,
    promotion_mask: Move,
) {
    let bit_to = bit(to);
    let bit_from = bit(from);

    match promotion_mask {
        PROMOTION_KNIGHT_MOVE_MASK => {
            position.pieces[position.mover as usize].knight_bitboard |= bit_to;
            position.zobrist_lock ^= ZOBRIST_KEYS_PIECES[position.mover as usize]
                [ZOBRIST_PIECE_INDEX_KNIGHT][to as usize];
        }
        PROMOTION_BISHOP_MOVE_MASK => {
            position.pieces[position.mover as usize].bishop_bitboard |= bit_to;
            position.zobrist_lock ^= ZOBRIST_KEYS_PIECES[position.mover as usize]
                [ZOBRIST_PIECE_INDEX_BISHOP][to as usize];
        }
        PROMOTION_ROOK_MOVE_MASK => {
            position.pieces[position.mover as usize].rook_bitboard |= bit_to;
            position.zobrist_lock ^=
                ZOBRIST_KEYS_PIECES[position.mover as usize][ZOBRIST_PIECE_INDEX_ROOK][to as usize];
        }
        PROMOTION_QUEEN_MOVE_MASK => {
            position.pieces[position.mover as usize].queen_bitboard |= bit_to;
            position.zobrist_lock ^= ZOBRIST_KEYS_PIECES[position.mover as usize]
                [ZOBRIST_PIECE_INDEX_QUEEN][to as usize];
        }
        _ => panic!("Invalid promotion piece"),
    }

    position.pieces[position.mover as usize].pawn_bitboard ^= bit_from;

    let opponent = opponent!(position.mover) as usize;

    position.zobrist_lock ^=
        ZOBRIST_KEYS_PIECES[position.mover as usize][ZOBRIST_PIECE_INDEX_PAWN][from as usize];

    position.pieces[position.mover as usize].all_pieces_bitboard ^= bit_from | bit_to;

    if position.pieces[opponent].all_pieces_bitboard & bit_to != 0 {
        let enemy = &mut position.pieces[opponent];

        if enemy.knight_bitboard & bit_to != 0 {
            enemy.knight_bitboard &= !bit_to;
            position.zobrist_lock ^=
                ZOBRIST_KEYS_PIECES[opponent][ZOBRIST_PIECE_INDEX_KNIGHT][to as usize];
        }
        if enemy.rook_bitboard & bit_to != 0 {
            enemy.rook_bitboard &= !bit_to;
            position.zobrist_lock ^=
                ZOBRIST_KEYS_PIECES[opponent][ZOBRIST_PIECE_INDEX_ROOK][to as usize];
        }
        if enemy.bishop_bitboard & bit_to != 0 {
            enemy.bishop_bitboard &= !bit_to;
            position.zobrist_lock ^=
                ZOBRIST_KEYS_PIECES[opponent][ZOBRIST_PIECE_INDEX_BISHOP][to as usize];
        }
        if enemy.queen_bitboard & bit_to != 0 {
            enemy.queen_bitboard &= !bit_to;
            position.zobrist_lock ^=
                ZOBRIST_KEYS_PIECES[opponent][ZOBRIST_PIECE_INDEX_QUEEN][to as usize];
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
pub fn make_non_pawn_non_king_capture_move(
    position: &mut Position,
    from: Square,
    to: Square,
    piece: Move,
) {
    let to_mask = bit(to);

    remove_captured_piece(position, to);

    let switch = bit(from) | to_mask;
    let m = position.mover as usize;
    position.pieces[position.mover as usize].all_pieces_bitboard ^= switch;

    match piece {
        PIECE_MASK_KNIGHT => {
            position.zobrist_lock ^= ZOBRIST_KEYS_PIECES[m][ZOBRIST_PIECE_INDEX_KNIGHT]
                [from as usize]
                ^ ZOBRIST_KEYS_PIECES[m][ZOBRIST_PIECE_INDEX_KNIGHT][to as usize];
            position.pieces[position.mover as usize].knight_bitboard ^= switch
        }
        PIECE_MASK_BISHOP => {
            position.zobrist_lock ^= ZOBRIST_KEYS_PIECES[m][ZOBRIST_PIECE_INDEX_BISHOP]
                [from as usize]
                ^ ZOBRIST_KEYS_PIECES[m][ZOBRIST_PIECE_INDEX_BISHOP][to as usize];
            position.pieces[position.mover as usize].bishop_bitboard ^= switch
        }
        PIECE_MASK_ROOK => {
            position.zobrist_lock ^= ZOBRIST_KEYS_PIECES[m][ZOBRIST_PIECE_INDEX_ROOK]
                [from as usize]
                ^ ZOBRIST_KEYS_PIECES[m][ZOBRIST_PIECE_INDEX_ROOK][to as usize];
            update_castle_flags_if_square(position, from);
            position.pieces[position.mover as usize].rook_bitboard ^= switch
        }
        PIECE_MASK_QUEEN => {
            position.zobrist_lock ^= ZOBRIST_KEYS_PIECES[m][ZOBRIST_PIECE_INDEX_QUEEN]
                [from as usize]
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
            position.zobrist_lock ^=
                ZOBRIST_KEYS_PIECES[opponent][ZOBRIST_PIECE_INDEX_PAWN][to as usize];
        }
        if enemy.knight_bitboard & bit_to != 0 {
            enemy.knight_bitboard &= !bit_to;
            position.zobrist_lock ^=
                ZOBRIST_KEYS_PIECES[opponent][ZOBRIST_PIECE_INDEX_KNIGHT][to as usize];
        }
        if enemy.rook_bitboard & bit_to != 0 {
            enemy.rook_bitboard &= !bit_to;
            position.zobrist_lock ^=
                ZOBRIST_KEYS_PIECES[opponent][ZOBRIST_PIECE_INDEX_ROOK][to as usize];
        }
        if enemy.bishop_bitboard & bit_to != 0 {
            enemy.bishop_bitboard &= !bit_to;
            position.zobrist_lock ^=
                ZOBRIST_KEYS_PIECES[opponent][ZOBRIST_PIECE_INDEX_BISHOP][to as usize];
        }
        if enemy.queen_bitboard & bit_to != 0 {
            enemy.queen_bitboard &= !bit_to;
            position.zobrist_lock ^=
                ZOBRIST_KEYS_PIECES[opponent][ZOBRIST_PIECE_INDEX_QUEEN][to as usize];
        }
        enemy.all_pieces_bitboard &= !bit_to;
    }

    let switch = bit(from) | bit_to;
    position.pieces[position.mover as usize].all_pieces_bitboard ^= switch;
    position.pieces[position.mover as usize].pawn_bitboard ^= switch;

    position.zobrist_lock ^= ZOBRIST_KEYS_PIECES[position.mover as usize][ZOBRIST_PIECE_INDEX_PAWN]
        [from as usize]
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

    position.zobrist_lock ^= ZOBRIST_KEYS_PIECES[position.mover as usize][ZOBRIST_PIECE_INDEX_KING]
        [from as usize]
        ^ ZOBRIST_KEYS_PIECES[position.mover as usize][ZOBRIST_PIECE_INDEX_KING][to as usize];
}

#[inline(always)]
fn remove_captured_piece(position: &mut Position, to: Square) {
    let bit_to = bit(to);

    if (position.pieces[WHITE as usize].all_pieces_bitboard
        | position.pieces[BLACK as usize].all_pieces_bitboard)
        & bit_to
        != 0
    {
        let opponent = opponent!(position.mover) as usize;
        let enemy = &mut position.pieces[opponent];
        position.half_moves = 0;

        if enemy.pawn_bitboard & bit_to != 0 {
            enemy.pawn_bitboard &= !bit_to;
            position.zobrist_lock ^=
                ZOBRIST_KEYS_PIECES[opponent][ZOBRIST_PIECE_INDEX_PAWN][to as usize];
        }
        if enemy.knight_bitboard & bit_to != 0 {
            enemy.knight_bitboard &= !bit_to;
            position.zobrist_lock ^=
                ZOBRIST_KEYS_PIECES[opponent][ZOBRIST_PIECE_INDEX_KNIGHT][to as usize];
        }
        if enemy.rook_bitboard & bit_to != 0 {
            enemy.rook_bitboard &= !bit_to;
            position.zobrist_lock ^=
                ZOBRIST_KEYS_PIECES[opponent][ZOBRIST_PIECE_INDEX_ROOK][to as usize];
        }
        if enemy.bishop_bitboard & bit_to != 0 {
            enemy.bishop_bitboard &= !bit_to;
            position.zobrist_lock ^=
                ZOBRIST_KEYS_PIECES[opponent][ZOBRIST_PIECE_INDEX_BISHOP][to as usize];
        }
        if enemy.queen_bitboard & bit_to != 0 {
            enemy.queen_bitboard &= !bit_to;
            position.zobrist_lock ^=
                ZOBRIST_KEYS_PIECES[opponent][ZOBRIST_PIECE_INDEX_QUEEN][to as usize];
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
    position.pieces[WHITE as usize].all_pieces_bitboard
        | position.pieces[BLACK as usize].all_pieces_bitboard
}

#[inline(always)]
pub fn make_see_move(mv: Move, new_position: &mut Position) {
    let from = from_square_part(mv);
    let to = to_square_part(mv);

    let piece_mask = mv & PIECE_MASK_FULL;

    match piece_mask {
        PIECE_MASK_PAWN => {
            if mv & PROMOTION_FULL_MOVE_MASK != 0 {
                make_move_with_promotion(new_position, from, to, mv & PROMOTION_FULL_MOVE_MASK);
            } else if (from - to) % 8 != 0 {
                make_pawn_capture_move(new_position, from, to);
            }
        }
        _ => {
            let bit_to = bit(to);

            if (new_position.pieces[WHITE as usize].all_pieces_bitboard
                | new_position.pieces[BLACK as usize].all_pieces_bitboard)
                & bit_to
                != 0
            {
                let enemy = &mut new_position.pieces[opponent!(new_position.mover) as usize];

                if enemy.pawn_bitboard & bit_to != 0 {
                    enemy.pawn_bitboard &= !bit_to;
                }
                if enemy.knight_bitboard & bit_to != 0 {
                    enemy.knight_bitboard &= !bit_to;
                }
                if enemy.rook_bitboard & bit_to != 0 {
                    enemy.rook_bitboard &= !bit_to;
                }
                if enemy.bishop_bitboard & bit_to != 0 {
                    enemy.bishop_bitboard &= !bit_to;
                }
                if enemy.queen_bitboard & bit_to != 0 {
                    enemy.queen_bitboard &= !bit_to;
                }

                enemy.all_pieces_bitboard &= !bit_to;
            }

            let switch = bit(from) | bit_to;
            new_position.pieces[new_position.mover as usize].all_pieces_bitboard ^= switch;

            match piece_mask {
                PIECE_MASK_KNIGHT => {
                    new_position.pieces[new_position.mover as usize].knight_bitboard ^= switch
                }
                PIECE_MASK_BISHOP => {
                    new_position.pieces[new_position.mover as usize].bishop_bitboard ^= switch
                }
                PIECE_MASK_ROOK => {
                    new_position.pieces[new_position.mover as usize].rook_bitboard ^= switch;
                }
                PIECE_MASK_QUEEN => {
                    new_position.pieces[new_position.mover as usize].queen_bitboard ^= switch
                }
                PIECE_MASK_KING => {
                    new_position.pieces[new_position.mover as usize].king_square = to
                }
                _ => panic!("Piece panic"),
            }
        }
    }

    new_position.mover ^= 1;
}
