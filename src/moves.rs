use crate::bitboards::{
    bit, epsbit, BISHOP_RAYS, DOUBLE_MOVE_RANK_BITS, EMPTY_CASTLE_SQUARES, KING_MOVES_BITBOARDS, KNIGHT_MOVES_BITBOARDS,
    NO_CHECK_CASTLE_SQUARES, PAWN_MOVES_CAPTURE, PAWN_MOVES_FORWARD, ROOK_RAYS,
};
use crate::magic_bitboards::{magic_moves_bishop, magic_moves_rook};
use crate::move_constants::{
    CASTLE_FLAG, CASTLE_MOVE, KING_INDEX, PIECE_MASK_BISHOP, PIECE_MASK_FULL, PIECE_MASK_KING, PIECE_MASK_KNIGHT, PIECE_MASK_QUEEN,
    PIECE_MASK_ROOK, PROMOTION_BISHOP_MOVE_MASK, PROMOTION_KNIGHT_MOVE_MASK, PROMOTION_QUEEN_MOVE_MASK, PROMOTION_ROOK_MOVE_MASK,
    PROMOTION_SQUARES, QUEEN_INDEX,
};
use crate::types::{Bitboard, Move, MoveList, Mover, Position, Square, BLACK, WHITE};
use crate::utils::{from_square_mask, from_square_part, to_square_part};
use crate::{get_and_unset_lsb, opponent, unset_lsb};

#[macro_export]
macro_rules! add_moves {
    ($move_list:expr, $fsm:expr, $to_bitboard:expr) => {
        let mut bb = $to_bitboard;
        while bb != 0 {
            $move_list.push($fsm | get_and_unset_lsb!(bb) as Move);
        }
    };
}

#[inline(always)]
pub fn verify_move(position: &Position, m: Move) -> bool {
    if m == 0 {
        return false;
    }

    let mut move_list = MoveList::new();

    let all_pieces = position.pieces[WHITE as usize].all_pieces_bitboard | position.pieces[BLACK as usize].all_pieces_bitboard;
    let friendly = position.pieces[position.mover as usize];
    let valid_destinations = !friendly.all_pieces_bitboard;

    match m & PIECE_MASK_FULL {
        PIECE_MASK_KING => {
            let from_square = from_square_part(m);
            return if from_square == friendly.king_square {
                let landing_squares = KING_MOVES_BITBOARDS[from_square as usize] & valid_destinations;
                if bit(to_square_part(m)) & landing_squares != 0 {
                    true
                } else {
                    if position.castle_flags != 0 {
                        generate_castle_moves(position, &mut move_list, all_pieces, position.mover as usize)
                    }
                    move_list.contains(&m)
                }
            } else {
                false
            };
        }
        PIECE_MASK_QUEEN => {
            let from_square = from_square_part(m);
            return if bit(from_square) & friendly.queen_bitboard != 0 {
                let tsp = bit(to_square_part(m));
                (tsp & magic_moves_rook(from_square, all_pieces) & valid_destinations != 0)
                    || (tsp & magic_moves_bishop(from_square, all_pieces) & valid_destinations != 0)
            } else {
                false
            };
        }
        PIECE_MASK_ROOK => {
            let from_square = from_square_part(m);
            return if bit(from_square) & friendly.rook_bitboard != 0 {
                bit(to_square_part(m)) & magic_moves_rook(from_square, all_pieces) & valid_destinations != 0
            } else {
                false
            };
        }
        PIECE_MASK_BISHOP => {
            let from_square = from_square_part(m);
            return if bit(from_square) & friendly.bishop_bitboard != 0 {
                bit(to_square_part(m)) & magic_moves_bishop(from_square, all_pieces) & valid_destinations != 0
            } else {
                false
            };
        }
        PIECE_MASK_KNIGHT => {
            let from_square = from_square_part(m);
            let landing_squares = KNIGHT_MOVES_BITBOARDS[from_square as usize] & valid_destinations;
            return bit(from_square) & friendly.knight_bitboard != 0 && bit(to_square_part(m)) & landing_squares != 0;
        }
        _ => {
            generate_pawn_moves(
                position,
                &mut move_list,
                !all_pieces,
                position.mover as usize,
                friendly.pawn_bitboard,
            );
        }
    }

    move_list.contains(&m)
}

#[inline(always)]
pub fn generate_moves(position: &Position) -> MoveList {
    let mut move_list = MoveList::new();

    let all_pieces = position.pieces[WHITE as usize].all_pieces_bitboard | position.pieces[BLACK as usize].all_pieces_bitboard;
    let friendly = position.pieces[position.mover as usize];
    let valid_destinations = !friendly.all_pieces_bitboard;

    if position.castle_flags != 0 {
        generate_castle_moves(position, &mut move_list, all_pieces, position.mover as usize)
    }
    add_moves!(
        move_list,
        from_square_mask(friendly.king_square) | PIECE_MASK_KING,
        KING_MOVES_BITBOARDS[friendly.king_square as usize] & valid_destinations
    );

    generate_straight_slider_moves(
        friendly.rook_bitboard,
        all_pieces,
        &mut move_list,
        valid_destinations,
        PIECE_MASK_ROOK,
    );
    generate_knight_moves(&mut move_list, valid_destinations, friendly.knight_bitboard);
    generate_diagonal_slider_moves(
        friendly.bishop_bitboard,
        all_pieces,
        &mut move_list,
        valid_destinations,
        PIECE_MASK_BISHOP,
    );
    generate_straight_slider_moves(
        friendly.queen_bitboard,
        all_pieces,
        &mut move_list,
        valid_destinations,
        PIECE_MASK_QUEEN,
    );
    generate_diagonal_slider_moves(
        friendly.queen_bitboard,
        all_pieces,
        &mut move_list,
        valid_destinations,
        PIECE_MASK_QUEEN,
    );
    generate_pawn_moves(
        position,
        &mut move_list,
        !all_pieces,
        position.mover as usize,
        friendly.pawn_bitboard,
    );

    move_list
}

/// Generate only capture moves (for staged move generation)
#[inline(always)]
pub fn generate_captures(position: &Position) -> MoveList {
    let mut move_list = MoveList::new();

    let all_pieces = position.pieces[WHITE as usize].all_pieces_bitboard | position.pieces[BLACK as usize].all_pieces_bitboard;
    let friendly = position.pieces[position.mover as usize];
    let enemy = position.pieces[opponent!(position.mover) as usize];
    let capture_targets = enemy.all_pieces_bitboard;

    // King captures
    add_moves!(
        move_list,
        from_square_mask(friendly.king_square) | PIECE_MASK_KING,
        KING_MOVES_BITBOARDS[friendly.king_square as usize] & capture_targets
    );

    // Rook captures
    generate_straight_slider_moves(friendly.rook_bitboard, all_pieces, &mut move_list, capture_targets, PIECE_MASK_ROOK);

    // Knight captures
    generate_knight_moves(&mut move_list, capture_targets, friendly.knight_bitboard);

    // Bishop captures
    generate_diagonal_slider_moves(
        friendly.bishop_bitboard,
        all_pieces,
        &mut move_list,
        capture_targets,
        PIECE_MASK_BISHOP,
    );

    // Queen captures
    generate_straight_slider_moves(
        friendly.queen_bitboard,
        all_pieces,
        &mut move_list,
        capture_targets,
        PIECE_MASK_QUEEN,
    );
    generate_diagonal_slider_moves(
        friendly.queen_bitboard,
        all_pieces,
        &mut move_list,
        capture_targets,
        PIECE_MASK_QUEEN,
    );

    // Pawn captures (including en passant and capture-promotions)
    generate_pawn_captures(position, &mut move_list, position.mover as usize, friendly.pawn_bitboard);

    move_list
}

/// Generate only quiet (non-capture) moves
#[inline(always)]
pub fn generate_quiet_moves(position: &Position) -> MoveList {
    let mut move_list = MoveList::new();

    let all_pieces = position.pieces[WHITE as usize].all_pieces_bitboard | position.pieces[BLACK as usize].all_pieces_bitboard;
    let empty_squares = !all_pieces;
    let friendly = position.pieces[position.mover as usize];

    // Castling
    if position.castle_flags != 0 {
        generate_castle_moves(position, &mut move_list, all_pieces, position.mover as usize)
    }

    // King quiet moves
    add_moves!(
        move_list,
        from_square_mask(friendly.king_square) | PIECE_MASK_KING,
        KING_MOVES_BITBOARDS[friendly.king_square as usize] & empty_squares
    );

    // Rook quiet moves
    generate_straight_slider_moves(friendly.rook_bitboard, all_pieces, &mut move_list, empty_squares, PIECE_MASK_ROOK);

    // Knight quiet moves
    generate_knight_moves(&mut move_list, empty_squares, friendly.knight_bitboard);

    // Bishop quiet moves
    generate_diagonal_slider_moves(
        friendly.bishop_bitboard,
        all_pieces,
        &mut move_list,
        empty_squares,
        PIECE_MASK_BISHOP,
    );

    // Queen quiet moves
    generate_straight_slider_moves(friendly.queen_bitboard, all_pieces, &mut move_list, empty_squares, PIECE_MASK_QUEEN);
    generate_diagonal_slider_moves(friendly.queen_bitboard, all_pieces, &mut move_list, empty_squares, PIECE_MASK_QUEEN);

    // Pawn quiet moves (forward moves and non-capture promotions)
    generate_pawn_quiet_moves(
        position,
        &mut move_list,
        empty_squares,
        position.mover as usize,
        friendly.pawn_bitboard,
    );

    move_list
}

/// Generate only pawn captures (including en passant and capture-promotions)
#[inline(always)]
fn generate_pawn_captures(position: &Position, move_list: &mut MoveList, colour_index: usize, mut from_squares: Bitboard) {
    let capture_targets = position.pieces[opponent!(position.mover) as usize].all_pieces_bitboard | epsbit(position.en_passant_square);

    while from_squares != 0 {
        let from_square = get_and_unset_lsb!(from_squares);
        let mut to_bitboard = PAWN_MOVES_CAPTURE[colour_index][from_square as usize] & capture_targets;

        let fsm = from_square_mask(from_square);
        let is_promotion = to_bitboard & PROMOTION_SQUARES != 0;
        while to_bitboard != 0 {
            let base_move = fsm | get_and_unset_lsb!(to_bitboard) as Move;
            if is_promotion {
                move_list.push(base_move | PROMOTION_QUEEN_MOVE_MASK);
                move_list.push(base_move | PROMOTION_ROOK_MOVE_MASK);
                move_list.push(base_move | PROMOTION_BISHOP_MOVE_MASK);
                move_list.push(base_move | PROMOTION_KNIGHT_MOVE_MASK);
            } else {
                move_list.push(base_move);
            }
        }
    }
}

/// Generate only pawn quiet moves (forward moves and non-capture promotions)
#[inline(always)]
fn generate_pawn_quiet_moves(
    position: &Position,
    move_list: &mut MoveList,
    empty_squares: Bitboard,
    colour_index: usize,
    mut from_squares: Bitboard,
) {
    while from_squares != 0 {
        let from_square = get_and_unset_lsb!(from_squares);
        let pawn_moves = PAWN_MOVES_FORWARD[colour_index][from_square as usize] & empty_squares;

        // If you can move one square, maybe you can move two
        let shifted = if position.mover == WHITE {
            pawn_moves << 8
        } else {
            pawn_moves >> 8
        } & DOUBLE_MOVE_RANK_BITS[colour_index]
            & empty_squares;

        let mut to_bitboard = pawn_moves | shifted;

        let fsm = from_square_mask(from_square);
        let is_promotion = to_bitboard & PROMOTION_SQUARES != 0;
        while to_bitboard != 0 {
            let base_move = fsm | get_and_unset_lsb!(to_bitboard) as Move;
            if is_promotion {
                // Only generate queen promotions for quiet moves (underpromotions are rare)
                move_list.push(base_move | PROMOTION_QUEEN_MOVE_MASK);
            } else {
                move_list.push(base_move);
            }
        }
    }
}

#[inline(always)]
fn generate_pawn_moves(
    position: &Position,
    move_list: &mut MoveList,
    empty_squares: Bitboard,
    colour_index: usize,
    mut from_squares: Bitboard,
) {
    while from_squares != 0 {
        let from_square = get_and_unset_lsb!(from_squares);
        let pawn_moves = PAWN_MOVES_FORWARD[colour_index][from_square as usize] & empty_squares;

        // If you can move one square, maybe you can move two
        let shifted = if position.mover == WHITE {
            pawn_moves << 8
        } else {
            pawn_moves >> 8
        } & DOUBLE_MOVE_RANK_BITS[colour_index]
            & empty_squares;

        let enemy_pawns_capture_bitboard =
            position.pieces[opponent!(position.mover) as usize].all_pieces_bitboard | epsbit(position.en_passant_square);

        let mut to_bitboard = PAWN_MOVES_CAPTURE[colour_index][from_square as usize] & enemy_pawns_capture_bitboard | pawn_moves | shifted;

        let fsm = from_square_mask(from_square);
        let is_promotion = to_bitboard & PROMOTION_SQUARES != 0;
        while to_bitboard != 0 {
            let base_move = fsm | get_and_unset_lsb!(to_bitboard) as Move;
            if is_promotion {
                move_list.push(base_move | PROMOTION_QUEEN_MOVE_MASK);
                move_list.push(base_move | PROMOTION_ROOK_MOVE_MASK);
                move_list.push(base_move | PROMOTION_BISHOP_MOVE_MASK);
                move_list.push(base_move | PROMOTION_KNIGHT_MOVE_MASK);
            } else {
                move_list.push(base_move);
            }
        }
    }
}

#[inline(always)]
pub fn generate_capture_pawn_moves_with_destinations(
    move_list: &mut MoveList,
    colour_index: usize,
    mut from_squares: Bitboard,
    valid_destinations: Bitboard,
) {
    while from_squares != 0 {
        let from_square = get_and_unset_lsb!(from_squares);

        let mut to_bitboard = PAWN_MOVES_CAPTURE[colour_index][from_square as usize] & valid_destinations;

        let fsm = from_square_mask(from_square);
        let is_promotion = to_bitboard & PROMOTION_SQUARES != 0;
        while to_bitboard != 0 {
            let base_move = fsm | get_and_unset_lsb!(to_bitboard) as Move;
            if is_promotion {
                move_list.push(base_move | PROMOTION_QUEEN_MOVE_MASK);
            } else {
                move_list.push(base_move);
            }
        }
    }
}

#[inline(always)]
fn generate_castle_moves(position: &Position, move_list: &mut MoveList, all_pieces: Bitboard, colour_index: usize) {
    for side in [KING_INDEX, QUEEN_INDEX] {
        if position.castle_flags & CASTLE_FLAG[side][colour_index] != 0
            && all_pieces & EMPTY_CASTLE_SQUARES[side][colour_index] == 0
            && !any_squares_in_bitboard_attacked(position, position.mover, NO_CHECK_CASTLE_SQUARES[side][colour_index])
        {
            move_list.push(CASTLE_MOVE[side][colour_index]);
        }
    }
}

#[inline(always)]
pub fn generate_knight_moves(move_list: &mut MoveList, valid_destinations: Bitboard, mut from_squares_bitboard: Bitboard) {
    while from_squares_bitboard != 0 {
        let from_square = get_and_unset_lsb!(from_squares_bitboard);
        add_moves!(
            move_list,
            from_square_mask(from_square) | PIECE_MASK_KNIGHT,
            KNIGHT_MOVES_BITBOARDS[from_square as usize] & valid_destinations
        );
    }
}

#[inline(always)]
pub fn generate_diagonal_slider_moves(
    mut slider_bitboard: Bitboard,
    all_pieces_bitboard: Bitboard,
    move_list: &mut MoveList,
    valid_destinations: Bitboard,
    piece_mask: Move,
) {
    while slider_bitboard != 0 {
        let from_square = get_and_unset_lsb!(slider_bitboard) as Square;
        add_moves!(
            move_list,
            from_square_mask(from_square) | piece_mask,
            magic_moves_bishop(from_square, all_pieces_bitboard) & valid_destinations
        );
    }
}

#[inline(always)]
pub fn generate_straight_slider_moves(
    mut slider_bitboard: Bitboard,
    all_pieces_bitboard: Bitboard,
    move_list: &mut MoveList,
    valid_destinations: Bitboard,
    piece_mask: Move,
) {
    while slider_bitboard != 0 {
        let from_square = get_and_unset_lsb!(slider_bitboard) as Square;
        add_moves!(
            move_list,
            from_square_mask(from_square) | piece_mask,
            magic_moves_rook(from_square, all_pieces_bitboard) & valid_destinations
        );
    }
}

#[inline(always)]
pub fn any_squares_in_bitboard_attacked(position: &Position, attacked: Mover, mut bitboard: Bitboard) -> bool {
    while bitboard != 0 {
        if is_square_attacked(position, bitboard.trailing_zeros() as Square, attacked) {
            return true;
        }
        unset_lsb!(bitboard);
    }
    false
}

#[inline(always)]
pub fn is_square_attacked(position: &Position, attacked_square: Square, attacked: Mover) -> bool {
    let enemy = position.pieces[opponent!(attacked) as usize];

    enemy.pawn_bitboard & PAWN_MOVES_CAPTURE[attacked as usize][attacked_square as usize] != 0
        || (enemy.knight_bitboard > 0 && enemy.knight_bitboard & KNIGHT_MOVES_BITBOARDS[attacked_square as usize] != 0)
        || bit(enemy.king_square) & KING_MOVES_BITBOARDS[attacked_square as usize] != 0
        || {
            let all_pieces = position.pieces[WHITE as usize].all_pieces_bitboard | position.pieces[BLACK as usize].all_pieces_bitboard;
            is_square_attacked_by_straight_slider(enemy.rook_bitboard | enemy.queen_bitboard, attacked_square, all_pieces)
                || is_square_attacked_by_diagonal_slider(enemy.bishop_bitboard | enemy.queen_bitboard, attacked_square, all_pieces)
        }
}

#[inline(always)]
pub fn is_square_attacked_by_straight_slider(attacking_sliders: Bitboard, attacked_square: Square, all_pieces: Bitboard) -> bool {
    attacking_sliders > 0 &&
    ROOK_RAYS[attacked_square as usize] & attacking_sliders > 0 && // any sliders on the rays, worth checking properly?
    magic_moves_rook(attacked_square, all_pieces) & attacking_sliders != 0
}

#[inline(always)]
pub fn is_square_attacked_by_diagonal_slider(attacking_sliders: Bitboard, attacked_square: Square, all_pieces: Bitboard) -> bool {
    attacking_sliders > 0 &&
    BISHOP_RAYS[attacked_square as usize] & attacking_sliders > 0 && // any sliders on the rays, worth checking properly?
    magic_moves_bishop(attacked_square, all_pieces) & attacking_sliders != 0
}

#[inline(always)]
pub fn is_check(position: &Position, mover: Mover) -> bool {
    is_square_attacked(position, position.pieces[mover as usize].king_square, mover)
}

/// Returns a bitboard of all pieces giving check to the specified side's king
#[inline(always)]
pub fn get_checkers(position: &Position, mover: Mover) -> Bitboard {
    let king_square = position.pieces[mover as usize].king_square;
    let enemy = position.pieces[opponent!(mover) as usize];
    let all_pieces = position.pieces[WHITE as usize].all_pieces_bitboard | position.pieces[BLACK as usize].all_pieces_bitboard;

    let mut checkers: Bitboard = 0;

    // Pawn checkers
    checkers |= enemy.pawn_bitboard & PAWN_MOVES_CAPTURE[mover as usize][king_square as usize];

    // Knight checkers
    checkers |= enemy.knight_bitboard & KNIGHT_MOVES_BITBOARDS[king_square as usize];

    // Rook/Queen checkers (straight sliders)
    let straight_sliders = enemy.rook_bitboard | enemy.queen_bitboard;
    if straight_sliders != 0 {
        checkers |= magic_moves_rook(king_square, all_pieces) & straight_sliders;
    }

    // Bishop/Queen checkers (diagonal sliders)
    let diagonal_sliders = enemy.bishop_bitboard | enemy.queen_bitboard;
    if diagonal_sliders != 0 {
        checkers |= magic_moves_bishop(king_square, all_pieces) & diagonal_sliders;
    }

    checkers
}

/// Returns a bitboard of squares between two squares (exclusive), for slider blocking
#[inline(always)]
fn between_squares(sq1: Square, sq2: Square) -> Bitboard {
    // Pre-compute this in a lookup table would be faster, but for now compute it
    let all_pieces = bit(sq1) | bit(sq2);

    // Check if they're on the same rank, file, or diagonal
    let rook_attacks_from_sq1 = magic_moves_rook(sq1, all_pieces);
    let rook_attacks_from_sq2 = magic_moves_rook(sq2, all_pieces);

    if rook_attacks_from_sq1 & bit(sq2) != 0 {
        // On same rank or file
        return rook_attacks_from_sq1 & rook_attacks_from_sq2;
    }

    let bishop_attacks_from_sq1 = magic_moves_bishop(sq1, all_pieces);
    let bishop_attacks_from_sq2 = magic_moves_bishop(sq2, all_pieces);

    if bishop_attacks_from_sq1 & bit(sq2) != 0 {
        // On same diagonal
        return bishop_attacks_from_sq1 & bishop_attacks_from_sq2;
    }

    0 // Not on same line
}

/// Generate moves when in check (check evasions only)
#[inline(always)]
pub fn generate_check_evasions(position: &Position) -> MoveList {
    let mut move_list = MoveList::new();
    let mover = position.mover;
    let friendly = position.pieces[mover as usize];
    let enemy = position.pieces[opponent!(mover) as usize];
    let all_pieces = friendly.all_pieces_bitboard | enemy.all_pieces_bitboard;
    let king_square = friendly.king_square;

    let checkers = get_checkers(position, mover);
    let num_checkers = checkers.count_ones();

    // Generate king moves to safe squares (always valid in check)
    let king_moves = KING_MOVES_BITBOARDS[king_square as usize] & !friendly.all_pieces_bitboard;
    add_moves!(move_list, from_square_mask(king_square) | PIECE_MASK_KING, king_moves);

    // If double check, only king moves are legal
    if num_checkers > 1 {
        return move_list;
    }

    // Single check - can also block or capture the checker
    let checker_square = checkers.trailing_zeros() as Square;
    let checker_bit = bit(checker_square);

    // Squares that block the check (between king and checker) - only for sliders
    let block_squares = if (enemy.rook_bitboard | enemy.queen_bitboard | enemy.bishop_bitboard) & checker_bit != 0 {
        between_squares(king_square, checker_square)
    } else {
        0 // Non-slider, can't block
    };

    // Valid destination squares: capture checker OR block
    let valid_destinations = checker_bit | block_squares;

    // Generate captures/blocks with each piece type

    // Knights
    generate_knight_moves(&mut move_list, valid_destinations, friendly.knight_bitboard);

    // Rooks
    generate_straight_slider_moves(
        friendly.rook_bitboard,
        all_pieces,
        &mut move_list,
        valid_destinations,
        PIECE_MASK_ROOK,
    );

    // Bishops
    generate_diagonal_slider_moves(
        friendly.bishop_bitboard,
        all_pieces,
        &mut move_list,
        valid_destinations,
        PIECE_MASK_BISHOP,
    );

    // Queens
    generate_straight_slider_moves(
        friendly.queen_bitboard,
        all_pieces,
        &mut move_list,
        valid_destinations,
        PIECE_MASK_QUEEN,
    );
    generate_diagonal_slider_moves(
        friendly.queen_bitboard,
        all_pieces,
        &mut move_list,
        valid_destinations,
        PIECE_MASK_QUEEN,
    );

    // Pawns - captures of checker
    if checker_bit != 0 {
        generate_pawn_evasion_captures(position, &mut move_list, mover as usize, friendly.pawn_bitboard, checker_square);
    }

    // Pawns - blocks (forward moves to block squares)
    if block_squares != 0 {
        generate_pawn_evasion_blocks(
            position,
            &mut move_list,
            !all_pieces,
            mover as usize,
            friendly.pawn_bitboard,
            block_squares,
        );
    }

    move_list
}

/// Generate pawn captures that capture the checking piece
#[inline(always)]
fn generate_pawn_evasion_captures(
    position: &Position,
    move_list: &mut MoveList,
    colour_index: usize,
    mut from_squares: Bitboard,
    checker_square: Square,
) {
    let checker_bit = bit(checker_square);
    // Also consider en passant if the checker is the pawn that just moved
    let ep_target = if position.en_passant_square != -1 {
        // The pawn that can be captured via en passant is one rank behind the ep square
        let ep_pawn_square = if colour_index == WHITE as usize {
            position.en_passant_square - 8
        } else {
            position.en_passant_square + 8
        };
        if ep_pawn_square == checker_square {
            epsbit(position.en_passant_square)
        } else {
            0
        }
    } else {
        0
    };

    let capture_targets = checker_bit | ep_target;

    while from_squares != 0 {
        let from_square = get_and_unset_lsb!(from_squares);
        let mut to_bitboard = PAWN_MOVES_CAPTURE[colour_index][from_square as usize] & capture_targets;

        let fsm = from_square_mask(from_square);
        let is_promotion = to_bitboard & PROMOTION_SQUARES != 0;
        while to_bitboard != 0 {
            let base_move = fsm | get_and_unset_lsb!(to_bitboard) as Move;
            if is_promotion {
                move_list.push(base_move | PROMOTION_QUEEN_MOVE_MASK);
                move_list.push(base_move | PROMOTION_ROOK_MOVE_MASK);
                move_list.push(base_move | PROMOTION_BISHOP_MOVE_MASK);
                move_list.push(base_move | PROMOTION_KNIGHT_MOVE_MASK);
            } else {
                move_list.push(base_move);
            }
        }
    }
}

/// Generate pawn forward moves that block the check
#[inline(always)]
fn generate_pawn_evasion_blocks(
    position: &Position,
    move_list: &mut MoveList,
    empty_squares: Bitboard,
    colour_index: usize,
    mut from_squares: Bitboard,
    block_squares: Bitboard,
) {
    while from_squares != 0 {
        let from_square = get_and_unset_lsb!(from_squares);
        let pawn_moves = PAWN_MOVES_FORWARD[colour_index][from_square as usize] & empty_squares;

        // If you can move one square, maybe you can move two
        let shifted = if position.mover == WHITE {
            pawn_moves << 8
        } else {
            pawn_moves >> 8
        } & DOUBLE_MOVE_RANK_BITS[colour_index]
            & empty_squares;

        let mut to_bitboard = (pawn_moves | shifted) & block_squares;

        let fsm = from_square_mask(from_square);
        let is_promotion = to_bitboard & PROMOTION_SQUARES != 0;
        while to_bitboard != 0 {
            let base_move = fsm | get_and_unset_lsb!(to_bitboard) as Move;
            if is_promotion {
                move_list.push(base_move | PROMOTION_QUEEN_MOVE_MASK);
                move_list.push(base_move | PROMOTION_ROOK_MOVE_MASK);
                move_list.push(base_move | PROMOTION_BISHOP_MOVE_MASK);
                move_list.push(base_move | PROMOTION_KNIGHT_MOVE_MASK);
            } else {
                move_list.push(base_move);
            }
        }
    }
}
