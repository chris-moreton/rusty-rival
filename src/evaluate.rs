use crate::bitboards::{
    bit, north_fill, south_fill, BISHOP_RAYS, DARK_SQUARES_BITS, FILE_A_BITS, FILE_H_BITS, KING_MOVES_BITBOARDS, KNIGHT_MOVES_BITBOARDS,
    LIGHT_SQUARES_BITS, RANK_1_BITS, RANK_2_BITS, RANK_3_BITS, RANK_4_BITS, RANK_5_BITS, RANK_6_BITS, RANK_7_BITS, ROOK_RAYS,
};
use crate::engine_constants::{BISHOP_VALUE_AVERAGE, BISHOP_VALUE_PAIR, DOUBLED_PAWN_PENALTY, ISOLATED_PAWN_PENALTY, KING_THREAT_BONUS_BISHOP, KING_THREAT_BONUS_KNIGHT, KING_THREAT_BONUS_QUEEN, KNIGHT_FORK_THREAT_SCORE, KNIGHT_VALUE_AVERAGE, KNIGHT_VALUE_PAIR, PAWN_ADJUST_MAX_MATERIAL, PAWN_VALUE_AVERAGE, PAWN_VALUE_PAIR, QUEEN_VALUE_AVERAGE, QUEEN_VALUE_PAIR, ROOKS_ON_SEVENTH_RANK_BONUS, ROOK_VALUE_AVERAGE, ROOK_VALUE_PAIR, STARTING_MATERIAL, VALUE_BACKWARD_PAWN_PENALTY, VALUE_BISHOP_MOBILITY, VALUE_BISHOP_PAIR, VALUE_BISHOP_PAIR_FEWER_PAWNS_BONUS, VALUE_GUARDED_PASSED_PAWN, VALUE_KING_CANNOT_CATCH_PAWN, VALUE_KING_CANNOT_CATCH_PAWN_PIECES_REMAIN, VALUE_KING_DISTANCE_PASSED_PAWN_MULTIPLIER, VALUE_KNIGHT_OUTPOST, VALUE_PASSED_PAWN_BONUS, VALUE_ROOKS_ON_SAME_FILE, KING_THREAT_BONUS_ROOK};
use crate::magic_bitboards::{magic_moves_bishop, magic_moves_rook};
use crate::piece_square_tables::piece_square_values;
use crate::types::{default_evaluate_cache, Bitboard, EvaluateCache, Mover, Position, Score, Square, BLACK, WHITE};
use crate::utils::linear_scale;
use crate::{get_and_unset_lsb, opponent};
use std::cmp::{max, min};

#[inline(always)]
pub fn evaluate(position: &Position) -> Score {
    let mut cache = default_evaluate_cache();

    cache_piece_count(position, &mut cache);

    if insufficient_material(position, cache.piece_count, true) {
        return 0;
    }

    let score = material_score(position)
        + piece_square_values(position)
        + king_score(position, &cache)
        + king_threat_score(position)
        + rook_eval(position)
        + passed_pawn_score(position, &mut cache)
        + knight_outpost_scores(position, &mut cache)
        + doubled_and_isolated_pawn_score(position, &mut cache)
        + bishop_mobility_score(position)
        + backward_pawn_score(position);

    10 + if position.mover == WHITE { score } else { -score }
}

#[inline(always)]
pub fn insufficient_material(position: &Position, piece_count: u8, include_helpmates: bool) -> bool {

    if piece_count > 4 {
        return false;
    }

    if piece_count == 2 {
        return true;
    }

    let w = position.pieces[WHITE as usize];
    let b = position.pieces[BLACK as usize];

    let non_minor_bitboard = w.pawn_bitboard | b.pawn_bitboard | w.queen_bitboard | w.rook_bitboard | b.queen_bitboard | b.rook_bitboard;

    if non_minor_bitboard != 0 {
        return false;
    }

    let knight_count = (w.knight_bitboard | b.knight_bitboard).count_ones();
    let minor_count = (w.bishop_bitboard | b.bishop_bitboard).count_ones();

    if include_helpmates {
        return minor_count <= 2 || (minor_count == 3 && knight_count == 0);
    }

    if (w.bishop_bitboard | w.knight_bitboard | b.bishop_bitboard | b.knight_bitboard).count_ones() == 1 {
        return true;
    }

    w.knight_bitboard == 0
        && b.knight_bitboard == 0
        && w.bishop_bitboard.count_ones() == 1
        && b.bishop_bitboard.count_ones() == 1
        && single_bishop_square_colour(w.bishop_bitboard) == single_bishop_square_colour(b.bishop_bitboard)
}

fn cache_piece_count(position: &Position, cache: &mut EvaluateCache) {
    if cache.piece_count == 0 {
        cache.piece_count = (position.pieces[WHITE as usize].all_pieces_bitboard.count_ones()
            + position.pieces[BLACK as usize].all_pieces_bitboard.count_ones()) as u8;
    }
}

#[inline(always)]
pub fn single_bishop_square_colour(bb: Bitboard) -> Mover {
    if bb & DARK_SQUARES_BITS != 0 {
        BLACK
    } else {
        WHITE
    }
}

#[inline(always)]
pub fn is_knight_fork(position: &Position, to_squares: Bitboard, knight_colour: Mover) -> bool {
    let opponent = opponent!(knight_colour) as usize;
    let is_king_attacked = to_squares & bit(position.pieces[opponent].king_square) != 0;
    let is_a_major_piece_attacked = to_squares & (position.pieces[opponent].rook_bitboard | position.pieces[opponent].queen_bitboard) != 0;
    is_king_attacked && is_a_major_piece_attacked
}

#[inline(always)]
pub fn count_knight_fork_threats(position: &Position, knight_colour: Mover) -> i8 {
    let mut threats = 0;

    let mut bb = position.pieces[knight_colour as usize].knight_bitboard;
    while bb != 0 {
        let mut to_squares = KNIGHT_MOVES_BITBOARDS[get_and_unset_lsb!(bb) as usize];
        if is_knight_fork(position, to_squares, knight_colour) {
            threats += 1;
        }
        while to_squares != 0 {
            if is_knight_fork(
                position,
                KNIGHT_MOVES_BITBOARDS[get_and_unset_lsb!(to_squares) as usize],
                knight_colour,
            ) {
                threats += 1;
            }
        }
    }

    threats
}

#[inline(always)]
pub fn knight_fork_threat_score(position: &Position) -> Score {
    (count_knight_fork_threats(position, WHITE) - count_knight_fork_threats(position, BLACK)) as Score * KNIGHT_FORK_THREAT_SCORE
}

#[inline(always)]
pub fn king_threat_score(position: &Position) -> Score {
    let wks = position.pieces[WHITE as usize].king_square;
    let bks = position.pieces[BLACK as usize].king_square;

    let white_king_danger_zone = bit(wks) | KING_MOVES_BITBOARDS[wks as usize] | (KING_MOVES_BITBOARDS[wks as usize] << 8);
    let black_king_danger_zone = bit(bks) | KING_MOVES_BITBOARDS[bks as usize] | (KING_MOVES_BITBOARDS[bks as usize] >> 8);

    let all_pieces = position.pieces[WHITE as usize].all_pieces_bitboard | position.pieces[BLACK as usize].all_pieces_bitboard;

    let mut score: Score = 0;

    let mut bb = position.pieces[BLACK as usize].knight_bitboard;
    while bb != 0 {
        let from_square = get_and_unset_lsb!(bb);
        score -= (KNIGHT_MOVES_BITBOARDS[from_square as usize] & white_king_danger_zone).count_ones() as Score
            * KING_THREAT_BONUS_KNIGHT as Score;
    }

    let mut bb = position.pieces[WHITE as usize].knight_bitboard;
    while bb != 0 {
        let from_square = get_and_unset_lsb!(bb);
        score += (KNIGHT_MOVES_BITBOARDS[from_square as usize] & black_king_danger_zone).count_ones() as Score
            * KING_THREAT_BONUS_KNIGHT as Score;
    }

    let mut bb = position.pieces[BLACK as usize].bishop_bitboard;
    while bb != 0 {
        let from_square = get_and_unset_lsb!(bb);
        if BISHOP_RAYS[from_square as usize] & white_king_danger_zone != 0 {
            score -= (magic_moves_bishop(from_square, all_pieces) & white_king_danger_zone).count_ones() as Score
                * KING_THREAT_BONUS_BISHOP as Score;
        }
    }

    let mut bb = position.pieces[WHITE as usize].bishop_bitboard;
    while bb != 0 {
        let from_square = get_and_unset_lsb!(bb);
        if BISHOP_RAYS[from_square as usize] & black_king_danger_zone != 0 {
            score += (magic_moves_bishop(from_square, all_pieces) & black_king_danger_zone).count_ones() as Score
                * KING_THREAT_BONUS_BISHOP as Score;
        }
    }

    let mut bb = position.pieces[BLACK as usize].rook_bitboard;
    while bb != 0 {
        let from_square = get_and_unset_lsb!(bb);
        if ROOK_RAYS[from_square as usize] & white_king_danger_zone != 0 {
            score -= (magic_moves_rook(from_square, all_pieces) & white_king_danger_zone).count_ones() as Score
                * KING_THREAT_BONUS_ROOK as Score;
        }
    }

    let mut bb = position.pieces[WHITE as usize].rook_bitboard;
    while bb != 0 {
        let from_square = get_and_unset_lsb!(bb);
        if ROOK_RAYS[from_square as usize] & black_king_danger_zone != 0 {
            score += (magic_moves_rook(from_square, all_pieces) & black_king_danger_zone).count_ones() as Score
                * KING_THREAT_BONUS_ROOK as Score;
        }
    }

    let mut bb = position.pieces[BLACK as usize].queen_bitboard;
    while bb != 0 {
        let from_square = get_and_unset_lsb!(bb);
        if ROOK_RAYS[from_square as usize] & white_king_danger_zone != 0 {
            score -= (magic_moves_rook(from_square, all_pieces) & white_king_danger_zone).count_ones() as Score
                * KING_THREAT_BONUS_QUEEN as Score;
        }
        if BISHOP_RAYS[from_square as usize] & white_king_danger_zone != 0 {
            score -= (magic_moves_bishop(from_square, all_pieces) & white_king_danger_zone).count_ones() as Score
                * KING_THREAT_BONUS_QUEEN as Score;
        }
    }

    let mut bb = position.pieces[WHITE as usize].queen_bitboard;
    while bb != 0 {
        let from_square = get_and_unset_lsb!(bb);
        if ROOK_RAYS[from_square as usize] & black_king_danger_zone != 0 {
            score += (magic_moves_rook(from_square, all_pieces) & black_king_danger_zone).count_ones() as Score
                * KING_THREAT_BONUS_QUEEN as Score;
        }
        if BISHOP_RAYS[from_square as usize] & black_king_danger_zone != 0 {
            score += (magic_moves_bishop(from_square, all_pieces) & black_king_danger_zone).count_ones() as Score
                * KING_THREAT_BONUS_QUEEN as Score;
        }
    }

    score
}

#[inline(always)]
pub fn king_score(position: &Position, cache: &EvaluateCache) -> Score {
    let mut score = 0;

    if cache.piece_count > 10 {
        score += king_early_safety(position);
    }

    score
}

#[inline(always)]
pub fn contains_all_bits(bitboard: Bitboard, mask: Bitboard) -> bool {
    bitboard & mask == mask
}

#[inline(always)]
pub fn king_early_safety(position: &Position) -> Score {
    white_king_early_safety(position) - black_king_early_safety(position)
}

#[inline(always)]
pub fn white_king_early_safety(position: &Position) -> Score {
    let mut score: Score = 0;
    let white = position.pieces[WHITE as usize];

    if bit(white.king_square) & 0b0000000000000000000000000000000000000000000000000000000000000011 != 0 {
        let white_pawn_files: u8 = (south_fill(white.pawn_bitboard) & RANK_1_BITS) as u8;
        score += (white_pawn_files & 0b00000111).count_ones() as Score * 5;
        if white.rook_bitboard & 0b0000000000000000000000000000000000000000000000000000000000000100 != 0 {
            if contains_all_bits(
                white.pawn_bitboard,
                0b0000000000000000000000000000000000000000000000000000011100000000,
            ) {
                score += 30 // (A)
            } else if contains_all_bits(
                white.pawn_bitboard,
                0b0000000000000000000000000000000000000000000000100000010100000000,
            ) {
                score += if white.bishop_bitboard & 0b0000000000000000000000000000000000000000000000000000001000000000 != 0 {
                    20 // (B)
                } else {
                    0 // (G)
                }
            } else if contains_all_bits(
                white.pawn_bitboard,
                0b0000000000000000000000000000000000000000000000110000010000000000,
            ) {
                score += 5; // (C-)
                if white.bishop_bitboard & 0b0000000000000000000000000000000000000000000000000000001000000000 != 0 {
                    score += 10; // (C+)
                }
            } else if contains_all_bits(
                white.pawn_bitboard,
                0b0000000000000000000000000000000000000000000000010000011000000000,
            ) {
                score += 12; // (D)
            } else if contains_all_bits(
                white.pawn_bitboard,
                0b0000000000000000000000000000000000000100000000000000001100000000,
            ) {
                score += 17; // (E)
            } else if contains_all_bits(
                white.pawn_bitboard,
                0b0000000000000000000000000000000000000000000001000000001100000000,
            ) {
                score += 7; // (F)
            }
        }
    }
    score
}

#[inline(always)]
pub fn black_king_early_safety(position: &Position) -> Score {
    let mut score: Score = 0;
    let black = position.pieces[BLACK as usize];

    if bit(black.king_square) & 0b0000001100000000000000000000000000000000000000000000000000000000 != 0 {
        let black_pawn_files: u8 = (south_fill(black.pawn_bitboard) & RANK_1_BITS) as u8;
        score += (black_pawn_files & 0b00000111).count_ones() as Score * 5;

        if black.rook_bitboard & 0b0000010000000000000000000000000000000000000000000000000000000000 != 0 {
            if contains_all_bits(
                black.pawn_bitboard,
                0b0000000000000111000000000000000000000000000000000000000000000000,
            ) {
                score += 30 // (A)
            } else if contains_all_bits(
                black.pawn_bitboard,
                0b0000000000000101000000100000000000000000000000000000000000000000,
            ) {
                score += if black.bishop_bitboard & 0b0000000000000010000000000000000000000000000000000000000000000000 != 0 {
                    20 // (B)
                } else {
                    0 // (G)
                }
            } else if contains_all_bits(
                black.pawn_bitboard,
                0b0000000000000100000000110000000000000000000000000000000000000000,
            ) {
                score += 5; // (C-)
                if black.bishop_bitboard & 0b0000000000000010000000000000000000000000000000000000000000000000 != 0 {
                    score += 10; // (C+)
                }
            } else if contains_all_bits(
                black.pawn_bitboard,
                0b0000000000000110000000010000000000000000000000000000000000000000,
            ) {
                score += 12; // (D)
            } else if contains_all_bits(
                black.pawn_bitboard,
                0b0000000000000011000000000000010000000000000000000000000000000000,
            ) {
                score += 17; // (E)
            } else if contains_all_bits(
                black.pawn_bitboard,
                0b0000000000000011000001000000000000000000000000000000000000000000,
            ) {
                score += 7; // (F)
            }
        }
    }
    score
}

#[inline(always)]
pub fn material_score(position: &Position) -> Score {
    let game_stage =
        pawn_material(position, WHITE) + pawn_material(position, BLACK) + piece_material(position, WHITE) + piece_material(position, BLACK);

    let pawn_balance = position.pieces[WHITE as usize].pawn_bitboard.count_ones() as Score
        - position.pieces[BLACK as usize].pawn_bitboard.count_ones() as Score;
    let pawn_score = pawn_balance
        * linear_scale(
            game_stage as i64,
            0,
            STARTING_MATERIAL as i64,
            PAWN_VALUE_PAIR.1 as i64,
            PAWN_VALUE_PAIR.0 as i64,
        ) as Score;

    let knight_balance = position.pieces[WHITE as usize].knight_bitboard.count_ones() as Score
        - position.pieces[BLACK as usize].knight_bitboard.count_ones() as Score;
    let knight_score = knight_balance
        * linear_scale(
            game_stage as i64,
            0,
            STARTING_MATERIAL as i64,
            KNIGHT_VALUE_PAIR.1 as i64,
            KNIGHT_VALUE_PAIR.0 as i64,
        ) as Score;

    let bishop_balance = position.pieces[WHITE as usize].bishop_bitboard.count_ones() as Score
        - position.pieces[BLACK as usize].bishop_bitboard.count_ones() as Score;
    let bishop_score = bishop_balance
        * linear_scale(
            game_stage as i64,
            0,
            STARTING_MATERIAL as i64,
            BISHOP_VALUE_PAIR.1 as i64,
            BISHOP_VALUE_PAIR.0 as i64,
        ) as Score;

    let rook_balance = position.pieces[WHITE as usize].rook_bitboard.count_ones() as Score
        - position.pieces[BLACK as usize].rook_bitboard.count_ones() as Score;
    let rook_score = rook_balance
        * linear_scale(
            game_stage as i64,
            0,
            STARTING_MATERIAL as i64,
            ROOK_VALUE_PAIR.1 as i64,
            ROOK_VALUE_PAIR.0 as i64,
        ) as Score;

    let queen_balance = position.pieces[WHITE as usize].queen_bitboard.count_ones() as Score
        - position.pieces[BLACK as usize].queen_bitboard.count_ones() as Score;
    let queen_score = queen_balance
        * linear_scale(
            game_stage as i64,
            0,
            STARTING_MATERIAL as i64,
            QUEEN_VALUE_PAIR.1 as i64,
            QUEEN_VALUE_PAIR.0 as i64,
        ) as Score;

    pawn_score + knight_score + bishop_score + rook_score + queen_score
}

#[inline(always)]
pub fn piece_material(position: &Position, mover: Mover) -> Score {
    position.pieces[mover as usize].knight_bitboard.count_ones() as Score * KNIGHT_VALUE_AVERAGE
        + position.pieces[mover as usize].rook_bitboard.count_ones() as Score * ROOK_VALUE_AVERAGE
        + position.pieces[mover as usize].bishop_bitboard.count_ones() as Score * BISHOP_VALUE_AVERAGE
        + position.pieces[mover as usize].queen_bitboard.count_ones() as Score * QUEEN_VALUE_AVERAGE
}

#[inline(always)]
pub fn pawn_material(position: &Position, mover: Mover) -> Score {
    position.pieces[mover as usize].pawn_bitboard.count_ones() as Score * PAWN_VALUE_AVERAGE
}

#[inline(always)]
pub fn on_same_file_count(pawn_bitboard: Bitboard, pawn_files: u8) -> Score {
    pawn_bitboard.count_ones() as Score - (pawn_files.count_ones() as Score)
}

#[inline(always)]
pub fn isolated_pawn_count(pawn_files: u8) -> Score {
    let left: u8 = (pawn_files & (pawn_files << 1)) as u8;
    let right: u8 = (pawn_files & (pawn_files >> 1)) as u8;

    let not_isolated: u8 = (left | right).count_ones() as u8;
    (pawn_files.count_ones() - not_isolated as u32) as Score
}

#[inline(always)]
pub fn doubled_and_isolated_pawn_score(position: &Position, cache: &mut EvaluateCache) -> Score {
    let white_pawns = position.pieces[WHITE as usize].pawn_bitboard;
    let black_pawns = position.pieces[BLACK as usize].pawn_bitboard;

    if cache.white_pawn_files == None {
        cache.white_pawn_files = Option::from((south_fill(white_pawns) & RANK_1_BITS) as u8)
    }
    if cache.black_pawn_files == None {
        cache.black_pawn_files = Option::from((south_fill(black_pawns) & RANK_1_BITS) as u8)
    }

    let white_pawn_files = cache.white_pawn_files.unwrap();
    let black_pawn_files = cache.black_pawn_files.unwrap();

    let doubled = ((on_same_file_count(position.pieces[BLACK as usize].pawn_bitboard, black_pawn_files)
        - on_same_file_count(position.pieces[WHITE as usize].pawn_bitboard, white_pawn_files)) as Score
        * DOUBLED_PAWN_PENALTY) as Score;

    let isolated = (isolated_pawn_count(black_pawn_files) - isolated_pawn_count(white_pawn_files)) * ISOLATED_PAWN_PENALTY;

    doubled + isolated
}

#[inline(always)]
pub fn knight_outpost_scores(position: &Position, cache: &mut EvaluateCache) -> Score {
    let white_pawns = position.pieces[WHITE as usize].pawn_bitboard;
    let black_pawns = position.pieces[BLACK as usize].pawn_bitboard;

    let white_knights = position.pieces[WHITE as usize].knight_bitboard;
    let black_knights = position.pieces[BLACK as usize].knight_bitboard;

    if cache.white_pawn_attacks == None {
        cache.white_pawn_attacks = Option::from(((white_pawns & !FILE_A_BITS) << 9) | ((white_pawns & !FILE_H_BITS) << 7))
    }
    if cache.black_pawn_attacks == None {
        cache.black_pawn_attacks = Option::from(((black_pawns & !FILE_A_BITS) >> 7) | ((black_pawns & !FILE_H_BITS) >> 9))
    }

    let white_pawn_attacks = cache.white_pawn_attacks.unwrap();
    let black_pawn_attacks = cache.black_pawn_attacks.unwrap();

    let white_passed_knights: Bitboard = white_knights & !south_fill(black_pawn_attacks);
    let black_passed_knights: Bitboard = black_knights & !north_fill(white_pawn_attacks);

    let white_guarded_passed_knights = white_passed_knights & (((white_pawns & !FILE_A_BITS) << 9) | ((white_pawns & !FILE_H_BITS) << 7));
    let black_guarded_passed_knights = black_passed_knights & (((black_pawns & !FILE_A_BITS) >> 7) | ((black_pawns & !FILE_H_BITS) >> 9));

    (white_guarded_passed_knights.count_ones() as Score - black_guarded_passed_knights.count_ones() as Score) * VALUE_KNIGHT_OUTPOST
}

#[inline(always)]
pub fn passed_pawn_score(position: &Position, cache: &mut EvaluateCache) -> Score {
    let white_pawns = position.pieces[WHITE as usize].pawn_bitboard;
    let black_pawns = position.pieces[BLACK as usize].pawn_bitboard;

    if cache.white_pawn_attacks == None {
        cache.white_pawn_attacks = Option::from(((white_pawns & !FILE_A_BITS) << 9) | ((white_pawns & !FILE_H_BITS) << 7))
    }
    if cache.black_pawn_attacks == None {
        cache.black_pawn_attacks = Option::from(((black_pawns & !FILE_A_BITS) >> 7) | ((black_pawns & !FILE_H_BITS) >> 9))
    }

    let white_pawn_attacks = cache.white_pawn_attacks.unwrap();
    let black_pawn_attacks = cache.black_pawn_attacks.unwrap();

    let white_passed_pawns: Bitboard = white_pawns & !south_fill(black_pawns | black_pawn_attacks | (white_pawns >> 8));
    let black_passed_pawns: Bitboard = black_pawns & !north_fill(white_pawns | white_pawn_attacks | (black_pawns << 8));

    let guarded_score = guarded_passed_pawn_score(white_pawns, black_pawns, white_passed_pawns, black_passed_pawns);

    let mut passed_score = 0;

    passed_score += (white_passed_pawns & RANK_2_BITS).count_ones() as Score * VALUE_PASSED_PAWN_BONUS[0];
    passed_score += (white_passed_pawns & RANK_3_BITS).count_ones() as Score * VALUE_PASSED_PAWN_BONUS[1];
    passed_score += (white_passed_pawns & RANK_4_BITS).count_ones() as Score * VALUE_PASSED_PAWN_BONUS[2];
    passed_score += (white_passed_pawns & RANK_5_BITS).count_ones() as Score * VALUE_PASSED_PAWN_BONUS[3];
    passed_score += (white_passed_pawns & RANK_6_BITS).count_ones() as Score * VALUE_PASSED_PAWN_BONUS[4];
    passed_score += (white_passed_pawns & RANK_7_BITS).count_ones() as Score * VALUE_PASSED_PAWN_BONUS[5];

    passed_score -= (black_passed_pawns & RANK_2_BITS).count_ones() as Score * VALUE_PASSED_PAWN_BONUS[5];
    passed_score -= (black_passed_pawns & RANK_3_BITS).count_ones() as Score * VALUE_PASSED_PAWN_BONUS[4];
    passed_score -= (black_passed_pawns & RANK_4_BITS).count_ones() as Score * VALUE_PASSED_PAWN_BONUS[3];
    passed_score -= (black_passed_pawns & RANK_5_BITS).count_ones() as Score * VALUE_PASSED_PAWN_BONUS[2];
    passed_score -= (black_passed_pawns & RANK_6_BITS).count_ones() as Score * VALUE_PASSED_PAWN_BONUS[1];
    passed_score -= (black_passed_pawns & RANK_7_BITS).count_ones() as Score * VALUE_PASSED_PAWN_BONUS[0];

    let white_piece_values = piece_material(position, WHITE);
    let black_piece_values = piece_material(position, BLACK);

    let mut passed_pawn_bonus = if black_piece_values < PAWN_ADJUST_MAX_MATERIAL {
        let king_x = position.pieces[BLACK as usize].king_square % 8;
        let king_y = position.pieces[BLACK as usize].king_square / 8;
        let mut bb = white_passed_pawns;
        let mut score: Score = 0;
        while bb != 0 {
            let sq = get_and_unset_lsb!(bb);
            let pawn_distance = min(5, 7 - (sq / 8));
            let king_distance = max((king_x - (sq % 8)).abs(), (king_y - 7).abs());
            score += king_distance as Score * VALUE_KING_DISTANCE_PASSED_PAWN_MULTIPLIER;
            if pawn_distance < (king_distance - position.mover) {
                if black_piece_values == 0 {
                    score += VALUE_KING_CANNOT_CATCH_PAWN
                } else {
                    score += VALUE_KING_CANNOT_CATCH_PAWN_PIECES_REMAIN
                }
            }
        }

        linear_scale(black_piece_values as i64, 0, PAWN_ADJUST_MAX_MATERIAL as i64, score as i64, 0) as Score
    } else {
        0
    };

    passed_pawn_bonus -= if white_piece_values < PAWN_ADJUST_MAX_MATERIAL {
        let king_x = position.pieces[WHITE as usize].king_square % 8;
        let king_y = position.pieces[WHITE as usize].king_square / 8;
        let mut bb = black_passed_pawns;
        let mut score: Score = 0;
        while bb != 0 {
            let sq = get_and_unset_lsb!(bb);
            let pawn_distance = min(5, sq / 8);
            let king_distance = max((king_x - (sq % 8)).abs(), king_y.abs());
            score += king_distance as Score * VALUE_KING_DISTANCE_PASSED_PAWN_MULTIPLIER;

            if pawn_distance < (king_distance - opponent!(position.mover)) {
                if white_piece_values == 0 {
                    score += VALUE_KING_CANNOT_CATCH_PAWN
                } else {
                    score += VALUE_KING_CANNOT_CATCH_PAWN_PIECES_REMAIN
                }
            }
        }
        linear_scale(white_piece_values as i64, 0, PAWN_ADJUST_MAX_MATERIAL as i64, score as i64, 0) as Score
    } else {
        0
    };

    guarded_score + passed_score + passed_pawn_bonus
}

#[inline(always)]
pub fn guarded_passed_pawn_score(
    white_pawns: Bitboard,
    black_pawns: Bitboard,
    white_passed_pawns: Bitboard,
    black_passed_pawns: Bitboard,
) -> Score {
    let white_guarded_passed_pawns = white_passed_pawns & (((white_pawns & !FILE_A_BITS) << 9) | ((white_pawns & !FILE_H_BITS) << 7));
    let black_guarded_passed_pawns = black_passed_pawns & (((black_pawns & !FILE_A_BITS) >> 7) | ((black_pawns & !FILE_H_BITS) >> 9));

    (white_guarded_passed_pawns.count_ones() as Score - black_guarded_passed_pawns.count_ones() as Score) * VALUE_GUARDED_PASSED_PAWN
}

#[inline(always)]
pub fn bishop_pair_bonus(bishops: Bitboard, pawns: Bitboard) -> Score {
    if bishops & DARK_SQUARES_BITS != 0 && bishops & LIGHT_SQUARES_BITS != 0 {
        VALUE_BISHOP_PAIR + (8 - pawns.count_ones()) as Score * VALUE_BISHOP_PAIR_FEWER_PAWNS_BONUS
    } else {
        0
    }
}

#[inline(always)]
pub fn rook_eval(position: &Position) -> Score {
    let white_rook_files: u8 = (south_fill(position.pieces[WHITE as usize].rook_bitboard) & RANK_1_BITS) as u8;
    let black_rook_files: u8 = (south_fill(position.pieces[BLACK as usize].rook_bitboard) & RANK_1_BITS) as u8;

    let mut score = (on_same_file_count(position.pieces[WHITE as usize].rook_bitboard, white_rook_files)
        - on_same_file_count(position.pieces[BLACK as usize].rook_bitboard, black_rook_files))
        * VALUE_ROOKS_ON_SAME_FILE;

    score += (position.pieces[WHITE as usize].rook_bitboard & 0b0000000011111111000000000000000000000000000000000000000000000000)
        .count_ones() as Score
        - (position.pieces[BLACK as usize].rook_bitboard & 0b0000000000000000000000000000000000000000000000001111111100000000).count_ones()
            as Score
            * ROOKS_ON_SEVENTH_RANK_BONUS;

    score
}

#[inline(always)]
pub fn bishop_mobility_score(position: &Position) -> Score {
    let all_pieces = position.pieces[WHITE as usize].all_pieces_bitboard
        | position.pieces[BLACK as usize].all_pieces_bitboard;

    let mut white_score: Score = 0;
    let mut black_score: Score = 0;

    let mut white_bishops = position.pieces[WHITE as usize].bishop_bitboard;
    while white_bishops != 0 {
        let sq = get_and_unset_lsb!(white_bishops);
        let moves = magic_moves_bishop(sq, all_pieces).count_ones() as usize;
        white_score += VALUE_BISHOP_MOBILITY[min(moves, 13)];
    }

    let mut black_bishops = position.pieces[BLACK as usize].bishop_bitboard;
    while black_bishops != 0 {
        let sq = get_and_unset_lsb!(black_bishops);
        let moves = magic_moves_bishop(sq, all_pieces).count_ones() as usize;
        black_score += VALUE_BISHOP_MOBILITY[min(moves, 13)];
    }

    white_score - black_score
}

#[inline(always)]
pub fn backward_pawn_score(position: &Position) -> Score {
    let white_pawns = position.pieces[WHITE as usize].pawn_bitboard;
    let black_pawns = position.pieces[BLACK as usize].pawn_bitboard;

    // White backward pawns: pawns that cannot be supported by adjacent pawns
    // and are behind all friendly pawns on adjacent files
    let white_pawn_attacks = ((white_pawns & !FILE_A_BITS) << 9) | ((white_pawns & !FILE_H_BITS) << 7);
    let black_pawn_attacks = ((black_pawns & !FILE_A_BITS) >> 7) | ((black_pawns & !FILE_H_BITS) >> 9);

    // A pawn is backward if it can't advance without being captured and has no pawn support
    let white_backward = white_pawns & !south_fill(white_pawn_attacks) & (black_pawn_attacks >> 8);
    let black_backward = black_pawns & !north_fill(black_pawn_attacks) & (white_pawn_attacks << 8);

    (black_backward.count_ones() as Score - white_backward.count_ones() as Score) * VALUE_BACKWARD_PAWN_PENALTY
}
