use crate::bitboards::{bit, G1_BIT, C1_BIT, G8_BIT, C8_BIT, E8_BIT, E1_BIT};
use crate::fen::move_from_algebraic_move;
use crate::move_constants::{BLACK_KING_CASTLE_MOVE_MASK, BLACK_QUEEN_CASTLE_MOVE_MASK, PIECE_MASK_BISHOP, PIECE_MASK_FULL, PIECE_MASK_KING, PIECE_MASK_KNIGHT, PIECE_MASK_PAWN, PIECE_MASK_QUEEN, PIECE_MASK_ROOK, WHITE_KING_CASTLE_MOVE_MASK, WHITE_QUEEN_CASTLE_MOVE_MASK};
use crate::types::{BLACK, Move, Position, Square, WHITE};

#[inline(always)]
pub const fn from_square_mask(square: Square) -> Move {
    (square as Move) << 16
}

#[inline(always)]
pub const fn from_square_part(mv: Move) -> Square {
    ((mv >> 16) & 63_u32) as Square
}

#[inline(always)]
pub const fn piece_part(mv: Move) -> Square {
    (mv & PIECE_MASK_FULL) as Square
}

#[inline(always)]
pub fn to_square_part(mv: Move) -> Square {
    (mv as Square) & 63
}

#[inline(always)]
pub fn moving_piece_mask(position: &Position, mv: Move) -> Move {

    let friendly = &position.pieces[position.mover as usize];
    let from_bb = bit(from_square_part(mv));

    if friendly.pawn_bitboard & from_bb != 0 {
        PIECE_MASK_PAWN
    }
    else if friendly.knight_bitboard & from_bb != 0 {
        PIECE_MASK_KNIGHT
    }
    else if friendly.bishop_bitboard & from_bb != 0 {
        PIECE_MASK_BISHOP
    }
    else if friendly.rook_bitboard & from_bb != 0  {
        PIECE_MASK_ROOK
    }
    else if friendly.queen_bitboard & from_bb != 0  {
        PIECE_MASK_QUEEN
    }
    else {
        PIECE_MASK_KING
    }
}

#[inline(always)]
pub fn castle_mask(position: &Position, mv: Move) -> Move {

    let from = from_square_part(mv);
    let to = to_square_part(mv);

    if from == position.pieces[WHITE as usize].king_square && from == E1_BIT {
        match to {
            G1_BIT => WHITE_KING_CASTLE_MOVE_MASK,
            C1_BIT => WHITE_QUEEN_CASTLE_MOVE_MASK,
            _ => 0
        }
    } else if from == position.pieces[BLACK as usize].king_square && from == E8_BIT {
        match to {
            G8_BIT => BLACK_KING_CASTLE_MOVE_MASK,
            C8_BIT => BLACK_QUEEN_CASTLE_MOVE_MASK,
            _ => 0
        }
    } else {
        0
    }
}

#[inline(always)]
pub fn hydrate_move_from_algebraic_move(position: &Position, algebraic_move: String) -> Move {
    let mv = move_from_algebraic_move(algebraic_move, 0);
    mv | castle_mask(position, mv) | moving_piece_mask(position, mv)
}
