use crate::move_constants::{BK_CASTLE, BQ_CASTLE, PROMOTION_BISHOP_MOVE_MASK, PROMOTION_FULL_MOVE_MASK, PROMOTION_KNIGHT_MOVE_MASK, PROMOTION_QUEEN_MOVE_MASK, PROMOTION_ROOK_MOVE_MASK, WK_CASTLE, WQ_CASTLE};
use crate::types::{Bitboard, Move, Mover, Position, Square};
use crate::types::Mover::{Black, White};
use crate::utils::from_square_mask;

const EN_PASSANT_UNAVAILABLE: i8 = -1;

pub fn bit_array_to_decimal(is: Vec<u8>) -> u64 {
    let mut total: u64 = 0;

    for (x, _) in is.iter().enumerate().take(64) {
        if is[x] == 1 {
            total += 1 << (63 - x);
        }
    }
    total
}

pub fn board_bits(fen_ranks: &[String], piece_char: char) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();
    for item in fen_ranks {
        result.append(&mut rank_bits(item, piece_char))
    }
    result
}

pub fn is_file_number(c: char) -> bool {
    ('0'..='9').contains(&c)
}

pub fn char_as_num(c: char) -> u8 {
    c as u8 - 48
}

pub fn rank_bits(rank: &str, piece: char) -> Vec<u8> {
    return rank_bits(rank, piece, Vec::new());

    fn rank_bits(rank: &str, piece: char, mut result: Vec<u8>) -> Vec<u8> {
        if rank.chars().count() == 0 {
            return result;
        }
        let c = rank.chars().next().unwrap();
        let mut new_rank = rank.to_string();
        new_rank.remove(0);
        if is_file_number(c) {
            for _ in 0..char_as_num(c) {
                result.push(0)
            }
        } else if piece == c {
            result.push(1);
        } else {
            result.push(0);
        };

        rank_bits(&new_rank, piece, result)
    }
}

pub fn algebraic_squareref_from_bitref(bitref: u8) -> String {
    let rank = (bitref / 8) + 1;
    let file = 8 - (bitref % 8);
    let rank_char = (rank + 48) as char;
    let file_char = (file + 96) as char;
    file_char.to_string() + &*(rank_char.to_string())
}

pub fn algebraic_move_from_move(m: Move) -> String {
    let from_square = (m >> 16) as u8;
    let to_square = (m & 63) as u8;
    algebraic_squareref_from_bitref(from_square) + &*algebraic_squareref_from_bitref(to_square) + &*promotion_part(m)
}

pub fn promotion_mask(piece_char: String) -> Move {
    if piece_char == "q" { PROMOTION_QUEEN_MOVE_MASK }
    else if piece_char == "b" { PROMOTION_BISHOP_MOVE_MASK }
    else if piece_char == "r" { PROMOTION_ROOK_MOVE_MASK }
    else if piece_char == "n" { PROMOTION_QUEEN_MOVE_MASK }
    else { 0 }
}

pub fn move_from_algebraic_move(a: String) -> Move {
    let s = if a.len() == 4 { a + " " } else { a };
    from_square_mask(bitref_from_algebraic_squareref(s[0..2].to_string())) +
        bitref_from_algebraic_squareref(s[2..4].to_string()) as Move +
        promotion_mask(s[4..5].to_string())
}

pub fn promotion_part(m: Move) -> String {
    if PROMOTION_FULL_MOVE_MASK & m == PROMOTION_QUEEN_MOVE_MASK {
        "q".to_string()
    }
    else if PROMOTION_FULL_MOVE_MASK & m == PROMOTION_ROOK_MOVE_MASK {
        "r".to_string()
    }
    else if PROMOTION_FULL_MOVE_MASK & m == PROMOTION_BISHOP_MOVE_MASK {
        "b".to_string()
    }
    else if PROMOTION_FULL_MOVE_MASK & m == PROMOTION_KNIGHT_MOVE_MASK {
        "n".to_string()
    } else {
        "".to_string()
    }
}

pub fn get_fen_ranks(fen_board_part: String) -> Vec<String> {
    fen_board_part.split('/').map(|s| s.to_string()).collect()
}

pub fn fen_part(fen: &str, i: u8) -> String {
    let parts: Vec<&str> = fen.split(' ').collect();
    String::from(parts[i as usize])
}

pub fn fen_board_part(fen: &str) -> String {
    fen_part(fen, 0)
}

pub fn get_mover(fen: &str) -> Mover {
    let m = fen_part(fen, 1);
    if m == "w" { White } else { Black }
}

pub fn piece_bitboard(fen_ranks: &[String], piece: char) -> Bitboard {
    bit_array_to_decimal(board_bits(fen_ranks, piece))
}

pub fn en_passant_fen_part(fen: &str) -> String {
    fen_part(fen, 3)
}

pub fn bitref_from_algebraic_squareref(algebraic: String) -> Square {
    let file_num = algebraic.chars().next().unwrap() as u8 - 97;
    let rank_num = algebraic.chars().nth(1).unwrap() as u8 - 49;
    (rank_num * 8 + (7 - file_num)) as Square
}

fn en_passant_bit_ref(en_passant_fen_part: String) -> i8 {
    if en_passant_fen_part == "-" {
        EN_PASSANT_UNAVAILABLE
    } else {
        bitref_from_algebraic_squareref(en_passant_fen_part)
    }
}

pub fn get_position(fen: &str) -> Position {
    let fen_ranks = get_fen_ranks(fen_board_part(fen));
    let wp = piece_bitboard(&fen_ranks, 'P');
    let bp = piece_bitboard(&fen_ranks, 'p');
    let wk = piece_bitboard(&fen_ranks, 'K');
    let bk = piece_bitboard(&fen_ranks, 'k');
    let wn = piece_bitboard(&fen_ranks, 'N');
    let bn = piece_bitboard(&fen_ranks, 'n');
    let wb = piece_bitboard(&fen_ranks, 'B');
    let bb = piece_bitboard(&fen_ranks, 'b');
    let wr = piece_bitboard(&fen_ranks, 'R');
    let br = piece_bitboard(&fen_ranks, 'r');
    let wq = piece_bitboard(&fen_ranks, 'Q');
    let bq = piece_bitboard(&fen_ranks, 'q');
    let castle_part = fen_part(fen, 2);
    let mut castle_flags = 0;
    if castle_part.contains('K') { castle_flags |= WK_CASTLE };
    if castle_part.contains('Q') { castle_flags |= WQ_CASTLE };
    if castle_part.contains('k') { castle_flags |= BK_CASTLE };
    if castle_part.contains('q') { castle_flags |= BQ_CASTLE };

    Position {
        white_pawn_bitboard: wp,
        black_pawn_bitboard: bp,
        white_knight_bitboard: wn,
        black_knight_bitboard: bn,
        white_bishop_bitboard: wb,
        black_bishop_bitboard: bb,
        white_rook_bitboard: wr,
        black_rook_bitboard: br,
        white_queen_bitboard: wq,
        black_queen_bitboard: bq,
        white_king_bitboard: wk,
        black_king_bitboard: bk,
        all_pieces_bitboard: wp | bp | wn | bn | wb | bb | wr | br | wq | bq | wk | bk,
        white_pieces_bitboard: wp | wn | wb | wr | wq | wk,
        black_pieces_bitboard: bp | bn | bb | br | bq | bk,
        mover: get_mover(fen),
        en_passant_square: en_passant_bit_ref(en_passant_fen_part(fen)) as Square,
        castle_flags,
        half_moves: fen_part(fen, 4).parse::<u16>().unwrap(),
        move_number: fen_part(fen, 5).parse::<u16>().unwrap(),
    }

}
