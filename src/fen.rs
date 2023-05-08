use crate::bitboards::bit;
use crate::hash::zobrist_lock;
use crate::move_constants::{
    BK_CASTLE, BQ_CASTLE, PROMOTION_BISHOP_MOVE_MASK, PROMOTION_FULL_MOVE_MASK, PROMOTION_KNIGHT_MOVE_MASK, PROMOTION_QUEEN_MOVE_MASK,
    PROMOTION_ROOK_MOVE_MASK, WK_CASTLE, WQ_CASTLE,
};
use crate::types::{Bitboard, Move, Mover, Pieces, Position, Square, BLACK, WHITE};
use crate::utils::from_square_mask;
use std::collections::HashMap;

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

pub fn algebraic_squareref_from_bitref(bitref: Square) -> String {
    let rank = (bitref / 8) + 1;
    let file = 8 - (bitref % 8);
    let rank_char = (rank + 48) as u8 as char;
    let file_char = (file + 96) as u8 as char;
    file_char.to_string() + &*(rank_char.to_string())
}

pub fn algebraic_move_from_move(m: Move) -> String {
    let from_square = ((m >> 16) as u8 & 63) as Square;
    let to_square = (m & 63) as u8 as Square;
    algebraic_squareref_from_bitref(from_square) + &*algebraic_squareref_from_bitref(to_square) + &*promotion_part(m)
}

pub fn algebraic_path_from_path(p: &[Move]) -> String {
    p.iter()
        .filter(|m| **m != 0)
        .map(|m| algebraic_move_from_move(*m) + " ")
        .collect::<String>()
}

pub fn promotion_mask(piece_char: String) -> Move {
    if piece_char == "q" {
        PROMOTION_QUEEN_MOVE_MASK
    } else if piece_char == "b" {
        PROMOTION_BISHOP_MOVE_MASK
    } else if piece_char == "r" {
        PROMOTION_ROOK_MOVE_MASK
    } else if piece_char == "n" {
        PROMOTION_KNIGHT_MOVE_MASK
    } else {
        0
    }
}

pub fn move_from_algebraic_move(a: String, piece_mask: Move) -> Move {
    let s = if a.len() == 4 { a + " " } else { a };
    from_square_mask(bitref_from_algebraic_squareref(s[0..2].to_string()))
        | (piece_mask + bitref_from_algebraic_squareref(s[2..4].to_string()) as Move + promotion_mask(s[4..5].to_string()))
}

pub fn promotion_part(m: Move) -> String {
    if PROMOTION_FULL_MOVE_MASK & m == PROMOTION_QUEEN_MOVE_MASK {
        "q".to_string()
    } else if PROMOTION_FULL_MOVE_MASK & m == PROMOTION_ROOK_MOVE_MASK {
        "r".to_string()
    } else if PROMOTION_FULL_MOVE_MASK & m == PROMOTION_BISHOP_MOVE_MASK {
        "b".to_string()
    } else if PROMOTION_FULL_MOVE_MASK & m == PROMOTION_KNIGHT_MOVE_MASK {
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
    if m == "w" {
        WHITE
    } else {
        BLACK
    }
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
    if castle_part.contains('K') {
        castle_flags |= WK_CASTLE
    };
    if castle_part.contains('Q') {
        castle_flags |= WQ_CASTLE
    };
    if castle_part.contains('k') {
        castle_flags |= BK_CASTLE
    };
    if castle_part.contains('q') {
        castle_flags |= BQ_CASTLE
    };

    let mut position = Position {
        pieces: [
            Pieces {
                pawn_bitboard: wp,
                knight_bitboard: wn,
                bishop_bitboard: wb,
                rook_bitboard: wr,
                queen_bitboard: wq,
                king_square: wk.trailing_zeros() as Square,
                all_pieces_bitboard: wp | wn | wb | wr | wq | wk,
            },
            Pieces {
                pawn_bitboard: bp,
                knight_bitboard: bn,
                bishop_bitboard: bb,
                rook_bitboard: br,
                queen_bitboard: bq,
                king_square: bk.trailing_zeros() as Square,
                all_pieces_bitboard: bp | bn | bb | br | bq | bk,
            },
        ],
        mover: get_mover(fen),
        en_passant_square: en_passant_bit_ref(en_passant_fen_part(fen)) as Square,
        castle_flags,
        half_moves: fen_part(fen, 4).parse::<u16>().unwrap(),
        move_number: fen_part(fen, 5).parse::<u16>().unwrap(),
        zobrist_lock: 0,
    };

    let lock = zobrist_lock(&position);

    position.zobrist_lock = lock;
    position
}

pub fn get_piece_on_square(position: &Position, sq: Square) -> char {
    let bb = bit(sq);

    if position.pieces[WHITE as usize].pawn_bitboard & bb != 0 {
        'P'
    } else if position.pieces[WHITE as usize].knight_bitboard & bb != 0 {
        'N'
    } else if position.pieces[WHITE as usize].queen_bitboard & bb != 0 {
        'Q'
    } else if position.pieces[WHITE as usize].rook_bitboard & bb != 0 {
        'R'
    } else if position.pieces[WHITE as usize].bishop_bitboard & bb != 0 {
        'B'
    } else if position.pieces[WHITE as usize].king_square == sq {
        'K'
    } else if position.pieces[BLACK as usize].pawn_bitboard & bb != 0 {
        'p'
    } else if position.pieces[BLACK as usize].knight_bitboard & bb != 0 {
        'n'
    } else if position.pieces[BLACK as usize].queen_bitboard & bb != 0 {
        'q'
    } else if position.pieces[BLACK as usize].rook_bitboard & bb != 0 {
        'r'
    } else if position.pieces[BLACK as usize].bishop_bitboard & bb != 0 {
        'b'
    } else if position.pieces[BLACK as usize].king_square == sq {
        'k'
    } else {
        '-'
    }
}

pub fn get_fen(position: &Position) -> String {
    let mut fen: String = "".to_string();
    let mut blanks = 0;
    for rank in (0..=7).rev() {
        for file in (0..=7).rev() {
            let sq = (rank * 8) + file;
            assert!((0..=63).contains(&sq));
            let c = get_piece_on_square(position, sq);
            if c == '-' {
                blanks += 1;
            } else {
                if blanks > 0 {
                    fen += &blanks.to_string()
                }
                blanks = 0;
                fen += &c.to_string();
            }
        }
        if blanks > 0 {
            fen += &blanks.to_string()
        }
        if rank > 0 {
            fen += "/";
        }
        blanks = 0;
    }
    fen += " ";
    fen += if position.mover == WHITE { "w" } else { "b" };
    fen += " ";
    if position.castle_flags == 0 {
        fen += "-";
    } else {
        if position.castle_flags & WK_CASTLE != 0 {
            fen += "K"
        }
        if position.castle_flags & WQ_CASTLE != 0 {
            fen += "Q"
        }
        if position.castle_flags & BK_CASTLE != 0 {
            fen += "k"
        }
        if position.castle_flags & BQ_CASTLE != 0 {
            fen += "q"
        }
    }
    fen += " ";
    if position.en_passant_square == EN_PASSANT_UNAVAILABLE {
        fen += "-"
    } else {
        fen += &algebraic_squareref_from_bitref(position.en_passant_square);
    }
    fen += " ";
    fen += &position.half_moves.to_string();
    fen += " ";
    fen += &position.move_number.to_string();
    fen
}

pub fn simple_algebraic_to_pretty_algebraic(fen: &str, move_str: String) -> Option<String> {
    let pieces = "RNBQKPrnbqkp";
    let mut piece_to_algebraic: HashMap<char, String> = HashMap::new();
    for ch in pieces.chars() {
        piece_to_algebraic.insert(ch, ch.to_string().to_uppercase());
    }

    let fen_parts: Vec<&str> = fen.split_whitespace().collect();
    let board = fen_parts[0];

    let from = &move_str[0..2];
    let to = &move_str[2..4];

    let mut board_str = String::new();
    for ch in board.chars() {
        if ch.is_digit(10) {
            let empty_squares = ch.to_digit(10).unwrap() as usize;
            board_str.push_str(&".".repeat(empty_squares));
        } else if ch == '/' {
            continue;
        } else {
            board_str.push(ch);
        }

        if board_str.len() >= 64 {
            break;
        }
    }

    let from_sq_idx = square_to_index(from)?;

    let to_sq_idx = square_to_index(to)?;

    let moving_piece = board_str.chars().nth(from_sq_idx)?;
    let captured_piece = board_str.chars().nth(to_sq_idx)?;

    let mut alg_notation = String::new();
    if moving_piece != 'P' && moving_piece != 'p' {
        alg_notation.push_str(&piece_to_algebraic[&moving_piece]);
    }

    let disambiguation_rank = find_disambiguation_rank(&board_str, moving_piece, from_sq_idx, to_sq_idx);
    if let Some(rank) = disambiguation_rank {
        alg_notation.push(rank);
    } else {
        if (moving_piece == 'P' || moving_piece == 'p') && captured_piece != '.' {
            alg_notation.push_str(&from[0..1]);
        }

    }

    if captured_piece != '.' {
        alg_notation.push_str("x");
    }

    if move_str.len() == 5 {
        let promotion = move_str.chars().nth(4)?;
        alg_notation.push_str(&format!("{}{}", to, piece_to_algebraic[&promotion].to_lowercase()));
    } else {
        alg_notation.push_str(to);
    }

    Some(alg_notation)
}

fn square_to_index(square: &str) -> Option<usize> {
    let file = square.chars().nth(0)?;
    let rank = square.chars().nth(1)?;

    if file < 'a' || file > 'h' || rank < '1' || rank > '8' {
        return None;
    }

    let file_idx = (file as u8 - 'a' as u8) as usize;
    let rank_idx = (rank as u8 - '1' as u8) as usize;

    Some((7 - rank_idx) * 8 + file_idx)
}

fn find_disambiguation_rank(board: &str, piece: char, from_idx: usize, to_idx: usize) -> Option<char> {
    let (from_file, from_rank) = index_to_file_rank(from_idx);

    let mut disambiguate_on_file = false;
    let mut disambiguate_on_rank = false;

    for idx in 0..64 {
        if idx == from_idx {
            continue;
        }

        if board.chars().nth(idx)? == piece && can_move_to(board, idx, to_idx) {
            let (other_file, _) = index_to_file_rank(idx);

            if other_file == from_file {
                disambiguate_on_rank = true;
            } else {
                disambiguate_on_file = true;
            }
        }
    }

    if disambiguate_on_file && disambiguate_on_rank {
        return Some((from_rank as u8 + '1' as u8) as char);
    } else if disambiguate_on_file {
        return Some((from_file as u8 + 'a' as u8) as char);
    } else if disambiguate_on_rank {
        return Some((from_rank as u8 + '1' as u8) as char);
    }

    None
}

fn can_move_to(board: &str, from_idx: usize, to_idx: usize) -> bool {
    let piece = board.chars().nth(from_idx).unwrap();

    match piece.to_ascii_uppercase() {
        'R' => can_rook_move(from_idx, to_idx),
        'N' => can_knight_move(from_idx, to_idx),
        'B' => can_bishop_move(from_idx, to_idx),
        'Q' => can_rook_move(from_idx, to_idx) || can_bishop_move(from_idx, to_idx),
        'K' => can_king_move(from_idx, to_idx),
        'P' => can_pawn_move(board, from_idx, to_idx, piece),
        _ => false,
    }
}

fn can_rook_move(from_idx: usize, to_idx: usize) -> bool {
    let (from_file, from_rank) = index_to_file_rank(from_idx);
    let (to_file, to_rank) = index_to_file_rank(to_idx);

    from_file == to_file || from_rank == to_rank
}

fn can_knight_move(from_idx: usize, to_idx: usize) -> bool {
    let (from_file, from_rank) = index_to_file_rank(from_idx);
    let (to_file, to_rank) = index_to_file_rank(to_idx);

    let file_diff = (from_file as isize - to_file as isize).abs();
    let rank_diff = (from_rank as isize - to_rank as isize).abs();

    let response = (file_diff == 2 && rank_diff == 1) || (file_diff == 1 && rank_diff == 2);
    response

}

fn can_bishop_move(from_idx: usize, to_idx: usize) -> bool {
    let (from_file, from_rank) = index_to_file_rank(from_idx);
    let (to_file, to_rank) = index_to_file_rank(to_idx);

    (from_file as isize - to_file as isize).abs() == (from_rank as isize - to_rank as isize).abs()
}

fn can_king_move(from_idx: usize, to_idx: usize) -> bool {
    let (from_file, from_rank) = index_to_file_rank(from_idx);
    let (to_file, to_rank) = index_to_file_rank(to_idx);

    let file_diff = (from_file as isize - to_file as isize).abs();
    let rank_diff = (from_rank as isize - to_rank as isize).abs();

    file_diff <= 1 && rank_diff <= 1
}

fn can_pawn_move(board: &str, from_idx: usize, to_idx: usize, piece: char) -> bool {
    let (from_file, from_rank) = index_to_file_rank(from_idx);
    let (to_file, to_rank) = index_to_file_rank(to_idx);

    let file_diff = (from_file as isize - to_file as isize).abs();
    let rank_diff = if piece == 'P' {
        to_rank as isize - from_rank as isize
    } else {
        from_rank as isize - to_rank as isize
    };

    if file_diff == 1 && rank_diff == 1 {
        let capture_piece = board.chars().nth(to_idx).unwrap();
        return capture_piece != '.' && piece.is_uppercase() != capture_piece.is_uppercase();
    }

    if file_diff == 0 && rank_diff == 1 {
        return board.chars().nth(to_idx).unwrap() == '.';
    }

    false
}

fn index_to_file_rank(idx: usize) -> (usize, usize) {
    let file = idx % 8;
    let rank = idx / 8;
    (file, 7-rank)
}
