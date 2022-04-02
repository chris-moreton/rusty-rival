use crate::bitboards::{bit, BLACK_PASSED_PAWN_MASK, C1_BIT, C8_BIT, E1_BIT, E8_BIT, G1_BIT, G8_BIT, WHITE_PASSED_PAWN_MASK};
use crate::engine_constants::{BISHOP_VALUE, KNIGHT_VALUE, PAWN_VALUE, QUEEN_VALUE, ROOK_VALUE};
use crate::fen::{get_fen, get_position, move_from_algebraic_move};
use crate::move_constants::{
    BLACK_KING_CASTLE_MOVE_MASK, BLACK_QUEEN_CASTLE_MOVE_MASK, PIECE_MASK_BISHOP, PIECE_MASK_FULL, PIECE_MASK_KING, PIECE_MASK_KNIGHT,
    PIECE_MASK_PAWN, PIECE_MASK_QUEEN, PIECE_MASK_ROOK, PROMOTION_BISHOP_MOVE_MASK, PROMOTION_FULL_MOVE_MASK, PROMOTION_KNIGHT_MOVE_MASK,
    PROMOTION_QUEEN_MOVE_MASK, PROMOTION_ROOK_MOVE_MASK, WHITE_KING_CASTLE_MOVE_MASK, WHITE_QUEEN_CASTLE_MOVE_MASK,
};
use crate::opponent;
use crate::types::{Bitboard, Move, Position, Score, Square, BLACK, WHITE};

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
pub fn captured_piece_value(position: &Position, mv: Move) -> Score {
    let enemy = &position.pieces[opponent!(position.mover) as usize];
    let tsp = to_square_part(mv);
    let to_bb = bit(tsp);

    let promote_value = match mv & PROMOTION_FULL_MOVE_MASK {
        PROMOTION_QUEEN_MOVE_MASK => QUEEN_VALUE - PAWN_VALUE,
        PROMOTION_ROOK_MOVE_MASK => ROOK_VALUE - PAWN_VALUE,
        PROMOTION_BISHOP_MOVE_MASK => BISHOP_VALUE - PAWN_VALUE,
        PROMOTION_KNIGHT_MOVE_MASK => KNIGHT_VALUE - PAWN_VALUE,
        _ => 0,
    };

    promote_value
        + (if tsp == position.en_passant_square || enemy.pawn_bitboard & to_bb != 0 {
            PAWN_VALUE
        } else if enemy.knight_bitboard & to_bb != 0 {
            KNIGHT_VALUE
        } else if enemy.bishop_bitboard & to_bb != 0 {
            BISHOP_VALUE
        } else if enemy.rook_bitboard & to_bb != 0 {
            ROOK_VALUE
        } else if enemy.queen_bitboard & to_bb != 0 {
            QUEEN_VALUE
        } else {
            0
        })
}

#[inline(always)]
pub fn moving_piece_mask(position: &Position, mv: Move) -> Move {
    let friendly = &position.pieces[position.mover as usize];
    let from_bb = bit(from_square_part(mv));

    if friendly.pawn_bitboard & from_bb != 0 {
        PIECE_MASK_PAWN
    } else if friendly.knight_bitboard & from_bb != 0 {
        PIECE_MASK_KNIGHT
    } else if friendly.bishop_bitboard & from_bb != 0 {
        PIECE_MASK_BISHOP
    } else if friendly.rook_bitboard & from_bb != 0 {
        PIECE_MASK_ROOK
    } else if friendly.queen_bitboard & from_bb != 0 {
        PIECE_MASK_QUEEN
    } else {
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
            _ => 0,
        }
    } else if from == position.pieces[BLACK as usize].king_square && from == E8_BIT {
        match to {
            G8_BIT => BLACK_KING_CASTLE_MOVE_MASK,
            C8_BIT => BLACK_QUEEN_CASTLE_MOVE_MASK,
            _ => 0,
        }
    } else {
        0
    }
}

pub fn show_bitboard(title: &str, bitboard: Bitboard) {
    println!("{}", title);
    println!("########");
    for i in (0..=63).rev() {
        print!(" {} ", if bitboard & bit(i) != 0 { '*' } else { '-' });
        if i % 8 == 0 {
            println!()
        }
    }
    println!("########");
}

#[inline(always)]
pub fn hydrate_move_from_algebraic_move(position: &Position, algebraic_move: String) -> Move {
    let mv = move_from_algebraic_move(algebraic_move, 0);
    mv | castle_mask(position, mv) | moving_piece_mask(position, mv)
}

#[inline(always)]
pub fn linear_scale(value: i64, domain_min: i64, domain_max: i64, target_min: i64, target_max: i64) -> i64 {
    if value <= domain_min {
        target_min
    } else if value >= domain_max {
        target_max
    } else {
        target_min + (value - domain_min) * (target_max - target_min) / (domain_max - domain_min)
    }
}

pub fn invert_pos(position: &Position) -> Position {
    let fen = get_fen(position);
    get_position(&invert_fen(&fen))
}

pub fn invert_fen(fen: &str) -> String {
    let inverted_fen = fen
        .trim()
        .replace(" b ", " . ")
        .replace(" w ", " ; ")
        .replace('Q', "z")
        .replace('K', "x")
        .replace('N', "c")
        .replace('B', "v")
        .replace('R', "m")
        .replace('P', ",")
        .replace('q', "Q")
        .replace('k', "K")
        .replace('n', "N")
        .replace('b', "B")
        .replace('r', "R")
        .replace('p', "P")
        .replace('z', "q")
        .replace('x', "k")
        .replace('c', "n")
        .replace('v', "b")
        .replace('m', "r")
        .replace(',', "p")
        .replace(" . ", " w ")
        .replace(" ; ", " b ");

    let fen_parts: Vec<&str> = inverted_fen.split(' ').collect();
    let board_parts: Vec<&str> = fen_parts[0].split('/').collect();

    let en_passant_part = fen_parts[3].replace('6', ".").replace('3', "6").replace('.', "3");

    format!(
        "{}/{}/{}/{}/{}/{}/{}/{} {} {} {} {} {}",
        board_parts[7],
        board_parts[6],
        board_parts[5],
        board_parts[4],
        board_parts[3],
        board_parts[2],
        board_parts[1],
        board_parts[0],
        fen_parts[1],
        fen_parts[2],
        en_passant_part,
        fen_parts[4],
        fen_parts[5]
    )
}

#[inline(always)]
pub fn pawn_push(position: &Position, m: Move) -> bool {
    let move_piece = m & PIECE_MASK_FULL;
    if move_piece == PIECE_MASK_PAWN {
        let to_square = to_square_part(m);
        if to_square >= 48 || to_square <= 15 {
            return true;
        }
        if position.mover == WHITE {
            if (40..=47).contains(&to_square) {
                return position.pieces[BLACK as usize].pawn_bitboard & WHITE_PASSED_PAWN_MASK[to_square as usize] == 0;
            }
        } else if (16..=23).contains(&to_square) {
            return position.pieces[WHITE as usize].pawn_bitboard & BLACK_PASSED_PAWN_MASK[to_square as usize] == 0;
        }
    }
    false
}
