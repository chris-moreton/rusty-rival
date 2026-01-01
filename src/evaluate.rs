use crate::bitboards::{
    bit, north_fill, south_fill, BISHOP_RAYS, DARK_SQUARES_BITS, FILE_A_BITS, FILE_H_BITS, KING_MOVES_BITBOARDS, KNIGHT_MOVES_BITBOARDS,
    LIGHT_SQUARES_BITS, RANK_1_BITS, RANK_2_BITS, RANK_3_BITS, RANK_4_BITS, RANK_5_BITS, RANK_6_BITS, RANK_7_BITS, ROOK_RAYS,
};
use crate::engine_constants::{
    BISHOP_VALUE_AVERAGE, BISHOP_VALUE_PAIR, DOUBLED_PAWN_PENALTY, ENDGAME_MATERIAL_THRESHOLD, ISOLATED_PAWN_PENALTY,
    KING_THREAT_BONUS_BISHOP, KING_THREAT_BONUS_KNIGHT, KING_THREAT_BONUS_QUEEN, KING_THREAT_BONUS_ROOK, KNIGHT_FORK_THREAT_SCORE,
    KNIGHT_VALUE_AVERAGE, KNIGHT_VALUE_PAIR, PAWN_ADJUST_MAX_MATERIAL, PAWN_VALUE_AVERAGE, PAWN_VALUE_PAIR, QUEEN_VALUE_AVERAGE,
    QUEEN_VALUE_PAIR, ROOKS_ON_SEVENTH_RANK_BONUS, ROOK_OPEN_FILE_BONUS, ROOK_SEMI_OPEN_FILE_BONUS, ROOK_VALUE_AVERAGE, ROOK_VALUE_PAIR,
    STARTING_MATERIAL, VALUE_BACKWARD_PAWN_PENALTY, VALUE_BISHOP_MOBILITY, VALUE_BISHOP_PAIR, VALUE_BISHOP_PAIR_FEWER_PAWNS_BONUS,
    VALUE_CONNECTED_PASSED_PAWNS, VALUE_GUARDED_PASSED_PAWN, VALUE_KING_CANNOT_CATCH_PAWN, VALUE_KING_CANNOT_CATCH_PAWN_PIECES_REMAIN,
    VALUE_KING_DISTANCE_PASSED_PAWN_MULTIPLIER, VALUE_KING_ENDGAME_CENTRALIZATION, VALUE_KNIGHT_OUTPOST, VALUE_PASSED_PAWN_BONUS,
    VALUE_QUEEN_MOBILITY, VALUE_ROOKS_ON_SAME_FILE,
};
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

    if is_wrong_colored_bishop_draw(position, cache.piece_count) {
        return 0;
    }

    if is_kpk_draw(position, cache.piece_count) {
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
        + backward_pawn_score(position)
        + bishop_pair_bonus(
            position.pieces[WHITE as usize].bishop_bitboard,
            position.pieces[WHITE as usize].pawn_bitboard,
        )
        - bishop_pair_bonus(
            position.pieces[BLACK as usize].bishop_bitboard,
            position.pieces[BLACK as usize].pawn_bitboard,
        )
        + knight_fork_threat_score(position)
        + rook_file_score(position)
        + queen_mobility_score(position)
        + endgame_king_centralization_bonus(position);

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

/// Detects drawn positions where one side has a bishop and rook pawn (a or h file)
/// but the bishop cannot control the promotion square.
/// For example: White Ka1, Bc1 (light-squared), Pa2 vs Black Ka3
/// The a-pawn promotes on a8 (light square), but if the bishop is dark-squared,
/// it can never control the promotion square, making it a draw.
#[inline(always)]
pub fn is_wrong_colored_bishop_draw(position: &Position, piece_count: u8) -> bool {
    // Must be exactly 4 pieces: K + K + B + P
    if piece_count != 4 {
        return false;
    }

    let w = position.pieces[WHITE as usize];
    let b = position.pieces[BLACK as usize];

    // Check for White having K + B + P vs Black having only K
    if w.bishop_bitboard.count_ones() == 1
        && w.pawn_bitboard.count_ones() == 1
        && w.knight_bitboard == 0
        && w.rook_bitboard == 0
        && w.queen_bitboard == 0
        && b.pawn_bitboard == 0
        && b.knight_bitboard == 0
        && b.bishop_bitboard == 0
        && b.rook_bitboard == 0
        && b.queen_bitboard == 0
    {
        return is_wrong_color_for_pawn(w.bishop_bitboard, w.pawn_bitboard, true);
    }

    // Check for Black having K + B + P vs White having only K
    if b.bishop_bitboard.count_ones() == 1
        && b.pawn_bitboard.count_ones() == 1
        && b.knight_bitboard == 0
        && b.rook_bitboard == 0
        && b.queen_bitboard == 0
        && w.pawn_bitboard == 0
        && w.knight_bitboard == 0
        && w.bishop_bitboard == 0
        && w.rook_bitboard == 0
        && w.queen_bitboard == 0
    {
        return is_wrong_color_for_pawn(b.bishop_bitboard, b.pawn_bitboard, false);
    }

    false
}

/// Checks if a bishop is the wrong color to help promote a rook pawn.
/// - White's a-pawn promotes on a8 (light square) - needs light-squared bishop
/// - White's h-pawn promotes on h8 (dark square) - needs dark-squared bishop
/// - Black's a-pawn promotes on a1 (dark square) - needs dark-squared bishop
/// - Black's h-pawn promotes on h1 (light square) - needs light-squared bishop
#[inline(always)]
fn is_wrong_color_for_pawn(bishop_bb: Bitboard, pawn_bb: Bitboard, is_white: bool) -> bool {
    let is_a_pawn = pawn_bb & FILE_A_BITS != 0;
    let is_h_pawn = pawn_bb & FILE_H_BITS != 0;

    // Only applies to rook pawns (a-file or h-file)
    if !is_a_pawn && !is_h_pawn {
        return false;
    }

    let bishop_is_light = bishop_bb & LIGHT_SQUARES_BITS != 0;

    if is_white {
        // White's a-pawn promotes on a8 (light) - needs light bishop
        // White's h-pawn promotes on h8 (dark) - needs dark bishop
        if is_a_pawn {
            !bishop_is_light // wrong if bishop is dark
        } else {
            bishop_is_light // wrong if bishop is light
        }
    } else {
        // Black's a-pawn promotes on a1 (dark) - needs dark bishop
        // Black's h-pawn promotes on h1 (light) - needs light bishop
        if is_a_pawn {
            bishop_is_light // wrong if bishop is light
        } else {
            !bishop_is_light // wrong if bishop is dark
        }
    }
}

/// Detects drawn KP vs K positions where the defending king has opposition
/// or can block the pawn's promotion.
///
/// Key concepts:
/// - Rook pawns (a/h file): drawn if defending king can reach the corner
/// - Other pawns: drawn if defending king has opposition and is in front of the pawn
#[inline(always)]
pub fn is_kpk_draw(position: &Position, piece_count: u8) -> bool {
    // Must be exactly 3 pieces: K + K + P
    if piece_count != 3 {
        return false;
    }

    let w = position.pieces[WHITE as usize];
    let b = position.pieces[BLACK as usize];

    // Determine which side has the pawn
    let (pawn_bb, attacking_king_sq, defending_king_sq, pawn_is_white) = if w.pawn_bitboard.count_ones() == 1 && b.pawn_bitboard == 0 {
        // White has K+P, Black has K
        (w.pawn_bitboard, w.king_square, b.king_square, true)
    } else if b.pawn_bitboard.count_ones() == 1 && w.pawn_bitboard == 0 {
        // Black has K+P, White has K
        (b.pawn_bitboard, b.king_square, w.king_square, false)
    } else {
        return false;
    };

    // Ensure no other pieces
    if pawn_is_white {
        if w.knight_bitboard != 0
            || w.bishop_bitboard != 0
            || w.rook_bitboard != 0
            || w.queen_bitboard != 0
            || b.knight_bitboard != 0
            || b.bishop_bitboard != 0
            || b.rook_bitboard != 0
            || b.queen_bitboard != 0
        {
            return false;
        }
    } else if b.knight_bitboard != 0
        || b.bishop_bitboard != 0
        || b.rook_bitboard != 0
        || b.queen_bitboard != 0
        || w.knight_bitboard != 0
        || w.bishop_bitboard != 0
        || w.rook_bitboard != 0
        || w.queen_bitboard != 0
    {
        return false;
    }

    let pawn_sq = pawn_bb.trailing_zeros() as Square;
    let pawn_file = 7 - (pawn_sq % 8); // 0=a, 7=h
    let pawn_rank = pawn_sq / 8; // 0=rank1, 7=rank8

    let def_file = 7 - (defending_king_sq % 8);
    let def_rank = defending_king_sq / 8;

    let atk_file = 7 - (attacking_king_sq % 8);
    let atk_rank = attacking_king_sq / 8;

    // Handle rook pawns (a-file or h-file) specially
    if pawn_file == 0 || pawn_file == 7 {
        return is_rook_pawn_draw(pawn_file, pawn_rank, def_file, def_rank, atk_rank, pawn_is_white, position.mover);
    }

    // For non-rook pawns, check if defending king has opposition and is blocking
    is_opposition_draw(
        pawn_file,
        pawn_rank,
        def_file,
        def_rank,
        atk_file,
        atk_rank,
        pawn_is_white,
        position.mover,
    )
}

/// Check if a rook pawn position is drawn.
/// Rook pawns are drawn if the defending king can reach the corner.
#[inline(always)]
fn is_rook_pawn_draw(
    pawn_file: Square,
    pawn_rank: Square,
    def_file: Square,
    def_rank: Square,
    atk_rank: Square,
    pawn_is_white: bool,
    mover: Mover,
) -> bool {
    // Promotion square corner: a8 for a-pawn (white) or a1 for a-pawn (black)
    // h8 for h-pawn (white) or h1 for h-pawn (black)
    let promo_rank = if pawn_is_white { 7 } else { 0 };

    // If defending king is on the promotion file (same as pawn) and ahead of/at promotion rank
    // or can reach the corner, it's drawn
    if def_file == pawn_file {
        if pawn_is_white {
            // Defending king on promotion file and at or ahead of pawn
            if def_rank >= pawn_rank {
                return true;
            }
        } else {
            // Black pawn: defending king on file and at or below pawn
            if def_rank <= pawn_rank {
                return true;
            }
        }
    }

    // If defending king is in the corner or can reach it before pawn promotes
    // Simple heuristic: if defending king is within reach of corner
    let corner_rank = promo_rank;
    let corner_file = pawn_file;

    let def_dist_to_corner = max((def_file - corner_file).abs(), (def_rank - corner_rank).abs());

    // Distance for pawn to promote
    let pawn_dist_to_promo = if pawn_is_white { 7 - pawn_rank } else { pawn_rank };

    // Account for who moves - defender gets -1 if they move first
    let def_moves = if (pawn_is_white && mover == BLACK) || (!pawn_is_white && mover == WHITE) {
        def_dist_to_corner.saturating_sub(1)
    } else {
        def_dist_to_corner
    };

    // If defender can reach corner in time, it's drawn
    if def_moves <= pawn_dist_to_promo {
        // Also check that attacking king isn't cutting off the path
        // Simple: if attacking king is not between defender and corner, likely draw
        if pawn_is_white {
            if atk_rank <= def_rank || atk_rank < pawn_rank {
                return true;
            }
        } else if atk_rank >= def_rank || atk_rank > pawn_rank {
            return true;
        }
    }

    false
}

/// Check if a non-rook pawn position is drawn due to opposition.
/// The defending king has opposition if it's on the same file as the pawn,
/// in front of the pawn, and the attacker cannot outflank.
#[inline(always)]
#[allow(clippy::too_many_arguments)]
fn is_opposition_draw(
    pawn_file: Square,
    pawn_rank: Square,
    def_file: Square,
    def_rank: Square,
    atk_file: Square,
    atk_rank: Square,
    pawn_is_white: bool,
    mover: Mover,
) -> bool {
    // Defending king must be on the pawn's file or adjacent files
    let file_diff = (def_file - pawn_file).abs();
    if file_diff > 1 {
        return false;
    }

    // Defending king must be in front of the pawn
    if pawn_is_white {
        if def_rank <= pawn_rank {
            return false;
        }
    } else if def_rank >= pawn_rank {
        return false;
    }

    // Check for direct opposition: kings on same file, odd number of ranks apart
    if def_file == atk_file {
        let rank_diff = (def_rank - atk_rank).abs();
        if rank_diff % 2 == 0 {
            // Even distance - side to move has opposition
            // If it's the attacker's move and they have opposition, not drawn
            // If it's the defender's move and they have opposition, drawn
            let attacker_to_move = (pawn_is_white && mover == WHITE) || (!pawn_is_white && mover == BLACK);
            if !attacker_to_move {
                // Defender to move with even distance = defender has opposition = NOT drawn
                // (defender loses opposition by moving)
                return false;
            }
            // Attacker to move with even distance = drawn (defender will get opposition)
            return def_file == pawn_file && rank_diff == 2;
        } else {
            // Odd distance - side NOT to move has opposition
            let attacker_to_move = (pawn_is_white && mover == WHITE) || (!pawn_is_white && mover == BLACK);
            if attacker_to_move {
                // Attacker to move, defender has opposition
                // If defending king is directly in front of pawn, likely drawn
                if def_file == pawn_file {
                    // Check if attacking king is far enough that it can't outflank
                    let atk_file_diff = (atk_file - pawn_file).abs();
                    if atk_file_diff <= 1 {
                        // Attacking king is close but defender has opposition
                        // This is the classic drawn position
                        return true;
                    }
                }
            }
        }
    }

    false
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
    let left: u8 = pawn_files & (pawn_files << 1);
    let right: u8 = pawn_files & (pawn_files >> 1);

    let not_isolated: u8 = (left | right).count_ones() as u8;
    (pawn_files.count_ones() - not_isolated as u32) as Score
}

#[inline(always)]
pub fn doubled_and_isolated_pawn_score(position: &Position, cache: &mut EvaluateCache) -> Score {
    let white_pawns = position.pieces[WHITE as usize].pawn_bitboard;
    let black_pawns = position.pieces[BLACK as usize].pawn_bitboard;

    if cache.white_pawn_files.is_none() {
        cache.white_pawn_files = Option::from((south_fill(white_pawns) & RANK_1_BITS) as u8)
    }
    if cache.black_pawn_files.is_none() {
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

    if cache.white_pawn_attacks.is_none() {
        cache.white_pawn_attacks = Option::from(((white_pawns & !FILE_A_BITS) << 9) | ((white_pawns & !FILE_H_BITS) << 7))
    }
    if cache.black_pawn_attacks.is_none() {
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

    if cache.white_pawn_attacks.is_none() {
        cache.white_pawn_attacks = Option::from(((white_pawns & !FILE_A_BITS) << 9) | ((white_pawns & !FILE_H_BITS) << 7))
    }
    if cache.black_pawn_attacks.is_none() {
        cache.black_pawn_attacks = Option::from(((black_pawns & !FILE_A_BITS) >> 7) | ((black_pawns & !FILE_H_BITS) >> 9))
    }

    let white_pawn_attacks = cache.white_pawn_attacks.unwrap();
    let black_pawn_attacks = cache.black_pawn_attacks.unwrap();

    let white_passed_pawns: Bitboard = white_pawns & !south_fill(black_pawns | black_pawn_attacks | (white_pawns >> 8));
    let black_passed_pawns: Bitboard = black_pawns & !north_fill(white_pawns | white_pawn_attacks | (black_pawns << 8));

    let guarded_score = guarded_passed_pawn_score(white_pawns, black_pawns, white_passed_pawns, black_passed_pawns);
    let connected_score = connected_passed_pawn_score(white_passed_pawns, black_passed_pawns);

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

    guarded_score + connected_score + passed_score + passed_pawn_bonus
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

/// Score for connected passed pawns (passed pawns on adjacent files).
/// Connected passed pawns are extremely dangerous as they support each other toward promotion.
/// The bonus is based on the rank of the more advanced pawn in each connected pair.
#[inline(always)]
pub fn connected_passed_pawn_score(white_passed_pawns: Bitboard, black_passed_pawns: Bitboard) -> Score {
    let mut score: Score = 0;

    // Check each pair of adjacent files for connected white passed pawns
    for file in 0..7 {
        let this_file = FILE_MASKS[file];
        let next_file = FILE_MASKS[file + 1];

        // White connected passed pawns
        let white_this = white_passed_pawns & this_file;
        let white_next = white_passed_pawns & next_file;
        if white_this != 0 && white_next != 0 {
            // Find the most advanced pawn in this connected pair
            // Higher bit position = more advanced for white
            let most_advanced_rank = max((63 - white_this.leading_zeros()) / 8, (63 - white_next.leading_zeros()) / 8) as usize;
            // Rank 2 = index 0, rank 7 = index 5
            let rank_index = most_advanced_rank.saturating_sub(1).min(5);
            score += VALUE_CONNECTED_PASSED_PAWNS[rank_index];
        }

        // Black connected passed pawns
        let black_this = black_passed_pawns & this_file;
        let black_next = black_passed_pawns & next_file;
        if black_this != 0 && black_next != 0 {
            // Find the most advanced pawn in this connected pair
            // Lower bit position = more advanced for black
            let most_advanced_rank = min(black_this.trailing_zeros() / 8, black_next.trailing_zeros() / 8) as usize;
            // Rank 7 = index 0, rank 2 = index 5
            let rank_index = (6 - most_advanced_rank).min(5);
            score -= VALUE_CONNECTED_PASSED_PAWNS[rank_index];
        }
    }

    score
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
    let all_pieces = position.pieces[WHITE as usize].all_pieces_bitboard | position.pieces[BLACK as usize].all_pieces_bitboard;

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
pub fn queen_mobility_score(position: &Position) -> Score {
    let all_pieces = position.pieces[WHITE as usize].all_pieces_bitboard | position.pieces[BLACK as usize].all_pieces_bitboard;

    let mut white_score: Score = 0;
    let mut black_score: Score = 0;

    let mut white_queens = position.pieces[WHITE as usize].queen_bitboard;
    while white_queens != 0 {
        let sq = get_and_unset_lsb!(white_queens);
        let moves = (magic_moves_rook(sq, all_pieces) | magic_moves_bishop(sq, all_pieces)).count_ones() as usize;
        white_score += VALUE_QUEEN_MOBILITY[min(moves, 27)];
    }

    let mut black_queens = position.pieces[BLACK as usize].queen_bitboard;
    while black_queens != 0 {
        let sq = get_and_unset_lsb!(black_queens);
        let moves = (magic_moves_rook(sq, all_pieces) | magic_moves_bishop(sq, all_pieces)).count_ones() as usize;
        black_score += VALUE_QUEEN_MOBILITY[min(moves, 27)];
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

/// File masks for each file (indexed by file number 0-7)
const FILE_MASKS: [Bitboard; 8] = [
    FILE_A_BITS,
    FILE_A_BITS << 1,
    FILE_A_BITS << 2,
    FILE_A_BITS << 3,
    FILE_A_BITS << 4,
    FILE_A_BITS << 5,
    FILE_A_BITS << 6,
    FILE_A_BITS << 7,
];

#[inline(always)]
pub fn rook_file_score(position: &Position) -> Score {
    let white_pawns = position.pieces[WHITE as usize].pawn_bitboard;
    let black_pawns = position.pieces[BLACK as usize].pawn_bitboard;
    let all_pawns = white_pawns | black_pawns;

    let mut score: Score = 0;

    // White rooks
    let mut rooks = position.pieces[WHITE as usize].rook_bitboard;
    while rooks != 0 {
        let sq = get_and_unset_lsb!(rooks);
        let file_mask = FILE_MASKS[(sq % 8) as usize];

        if file_mask & all_pawns == 0 {
            score += ROOK_OPEN_FILE_BONUS;
        } else if file_mask & white_pawns == 0 {
            score += ROOK_SEMI_OPEN_FILE_BONUS;
        }
    }

    // Black rooks
    let mut rooks = position.pieces[BLACK as usize].rook_bitboard;
    while rooks != 0 {
        let sq = get_and_unset_lsb!(rooks);
        let file_mask = FILE_MASKS[(sq % 8) as usize];

        if file_mask & all_pawns == 0 {
            score -= ROOK_OPEN_FILE_BONUS;
        } else if file_mask & black_pawns == 0 {
            score -= ROOK_SEMI_OPEN_FILE_BONUS;
        }
    }

    score
}

/// Extra king centralization bonus for endgames.
/// This bonus kicks in when both sides have low material (no queens, limited pieces).
/// It encourages the king to become an active piece in the endgame.
#[inline(always)]
pub fn endgame_king_centralization_bonus(position: &Position) -> Score {
    let white_piece_value = piece_material(position, WHITE);
    let black_piece_value = piece_material(position, BLACK);

    // Only apply when both sides have low material
    if white_piece_value > ENDGAME_MATERIAL_THRESHOLD || black_piece_value > ENDGAME_MATERIAL_THRESHOLD {
        return 0;
    }

    let white_king_sq = position.pieces[WHITE as usize].king_square as usize;
    let black_king_sq = position.pieces[BLACK as usize].king_square as usize;

    let white_bonus = VALUE_KING_ENDGAME_CENTRALIZATION[white_king_sq];
    let black_bonus = VALUE_KING_ENDGAME_CENTRALIZATION[black_king_sq];

    // Scale bonus: more bonus when material is lower
    let total_piece_value = white_piece_value + black_piece_value;
    let max_material = ENDGAME_MATERIAL_THRESHOLD * 2;

    let raw_bonus = white_bonus - black_bonus;

    // Scale from 100% at 0 material to 0% at max_material
    linear_scale(total_piece_value as i64, 0, max_material as i64, raw_bonus as i64, 0) as Score
}
