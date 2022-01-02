pub mod make_move {
    use std::ptr::null;
    use crate::bitboards::bitboards::{A1_BIT, A8_BIT, bit, C1_BIT, C8_BIT, clear_bit, D1_BIT, D8_BIT, E1_BIT, E8_BIT, F1_BIT, F8_BIT, G1_BIT, G8_BIT, H1_BIT, H8_BIT, test_bit};
    use crate::fen::fen::get_position;
    use crate::move_constants::move_constants::*;
    use crate::moves::moves::move_piece_within_bitboard;
    use crate::types::types::{Bitboard, Move, Mover, Piece, Position, PositionHistory, Square};
    use crate::types::types::Mover::{Black, White};
    use crate::types::types::Piece::{Bishop, Empty, King, Knight, Pawn, Queen, Rook};
    use crate::utils::utils::{from_square_mask, from_square_part, to_square_part};

    pub fn make_move(position: &mut Position, mv: Move, history: &mut PositionHistory) {
        let from = from_square_part(mv);
        let to = to_square_part(mv);
        let piece = moving_piece(position, from);
        store_history(position, history);
        return if is_simple_move(position, from as Square, to, piece) {
            make_simple_move(position, mv, from as Square)
        } else {
            make_complex_move(position, mv)
        };
    }

    #[inline(always)]
    pub fn store_history(position: &mut Position, history: &mut PositionHistory) {
        let index = get_move_index(position.move_number, position.mover);

        history.history[index] = Position {
            white_pawn_bitboard: position.white_pawn_bitboard,
            white_knight_bitboard: position.white_knight_bitboard,
            white_bishop_bitboard: position.white_bishop_bitboard,
            white_queen_bitboard: position.white_queen_bitboard,
            white_king_bitboard: position.white_king_bitboard,
            white_rook_bitboard: position.white_rook_bitboard,
            black_pawn_bitboard: position.black_pawn_bitboard,
            black_knight_bitboard: position.black_knight_bitboard,
            black_bishop_bitboard: position.black_bishop_bitboard,
            black_queen_bitboard: position.black_queen_bitboard,
            black_king_bitboard: position.black_king_bitboard,
            black_rook_bitboard: position.black_rook_bitboard,
            all_pieces_bitboard: position.all_pieces_bitboard,
            white_pieces_bitboard: position.white_pieces_bitboard,
            black_pieces_bitboard: position.black_pieces_bitboard,
            mover: position.mover,
            en_passant_square: position.en_passant_square,
            white_king_castle_available: position.white_king_castle_available,
            black_king_castle_available: position.black_king_castle_available,
            white_queen_castle_available: position.white_queen_castle_available,
            black_queen_castle_available: position.black_queen_castle_available,
            half_moves: position.half_moves,
            move_number: position.move_number,
        }
    }

    #[inline(always)]
    pub fn get_move_index(move_number: u16, mover: Mover) -> usize {
        (move_number * 2 - if mover == White { 1 } else { 0 }) as usize
    }

    #[inline(always)]
    pub fn unmake_move(position: &mut Position, history: &PositionHistory) {
        let index = get_move_index(position.move_number, position.mover) - 1;
        let old = history.history[index];

        position.white_pawn_bitboard = old.white_pawn_bitboard;
        position.white_knight_bitboard = old.white_knight_bitboard;
        position.white_bishop_bitboard = old.white_bishop_bitboard;
        position.white_queen_bitboard = old.white_queen_bitboard;
        position.white_king_bitboard = old.white_king_bitboard;
        position.white_rook_bitboard = old.white_rook_bitboard;
        position.black_pawn_bitboard = old.black_pawn_bitboard;
        position.black_knight_bitboard = old.black_knight_bitboard;
        position.black_bishop_bitboard = old.black_bishop_bitboard;
        position.black_queen_bitboard = old.black_queen_bitboard;
        position.black_king_bitboard = old.black_king_bitboard;
        position.black_rook_bitboard = old.black_rook_bitboard;
        position.all_pieces_bitboard = old.all_pieces_bitboard;
        position.white_pieces_bitboard = old.white_pieces_bitboard;
        position.black_pieces_bitboard = old.black_pieces_bitboard;
        position.mover = old.mover;
        position.en_passant_square = old.en_passant_square;
        position.white_king_castle_available = old.white_king_castle_available;
        position.black_king_castle_available = old.black_king_castle_available;
        position.white_queen_castle_available = old.white_queen_castle_available;
        position.black_queen_castle_available = old.black_queen_castle_available;
        position.half_moves = old.half_moves;
        position.move_number = old.move_number;
    }

    #[inline(always)]
    pub fn make_complex_move(position: &mut Position, mv: Move) {
        let promoted_piece = promotion_piece_from_move(mv);
        let from = from_square_part(mv);
        let to = to_square_part(mv);

        if promoted_piece != Empty {
            make_move_with_promotion(position, mv, promoted_piece);
        } else if from == E1_BIT && (to == G1_BIT || to == C1_BIT) && (position.white_king_castle_available || position.white_queen_castle_available) {
            make_white_castle_move(position, to);
        } else if from == E8_BIT && (to == G8_BIT || to == C8_BIT) && (position.black_king_castle_available || position.black_queen_castle_available) {
            make_black_castle_move(position, to);
        } else {
            make_simple_complex_move(position, from, to);
        }
    }

    #[inline(always)]
    pub fn make_white_castle_move(position: &mut Position, to: Square) {
        let wr= if to == C1_BIT {
            clear_bit(position.white_rook_bitboard, A1_BIT) | bit(D1_BIT)
        } else {
            clear_bit(position.white_rook_bitboard, H1_BIT) | bit(F1_BIT)
        };
        let wk = move_piece_within_bitboard(E1_BIT, to, position.white_king_bitboard);
        let wpb = wr | wk | position.white_queen_bitboard | position.white_knight_bitboard | position.white_bishop_bitboard | position.white_pawn_bitboard;
        position.white_rook_bitboard = wr;
        position.white_king_bitboard = wk;
        position.all_pieces_bitboard = wpb | position.black_pieces_bitboard;
        position.white_pieces_bitboard = wpb;
        position.mover = Black;
        position.en_passant_square = EN_PASSANT_NOT_AVAILABLE;
        position.white_king_castle_available = false;
        position.white_queen_castle_available = false;
        position.half_moves += 1;
    }

    #[inline(always)]
    pub fn make_black_castle_move(position: &mut Position, to: Square) {
        let br= if to == C8_BIT {
            move_piece_within_bitboard(A8_BIT, D8_BIT, position.black_rook_bitboard)
        } else {
            move_piece_within_bitboard(H8_BIT, F8_BIT, position.black_rook_bitboard)
        };

        let bk = move_piece_within_bitboard(E8_BIT, to, position.black_king_bitboard);
        let bpb = br | bk | position.black_queen_bitboard | position.black_knight_bitboard | position.black_bishop_bitboard | position.black_pawn_bitboard;
        position.black_rook_bitboard = br;
        position.black_king_bitboard = bk;
        position.all_pieces_bitboard = bpb | position.white_pieces_bitboard;
        position.black_pieces_bitboard = bpb;
        position.mover = White;
        position.en_passant_square = EN_PASSANT_NOT_AVAILABLE;
        position.black_king_castle_available = false;
        position.black_queen_castle_available = false;
        position.half_moves += 1;
        position.move_number += 1;
        
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
    pub fn create_if_promotion(is_promotion_piece: bool, pawn_bitboard: Bitboard, piece_bitboard: Bitboard, from_square: Square, to_square: Square) -> Bitboard {
        if is_promotion_piece && is_promotion_square(to_square) && test_bit(pawn_bitboard, from_square) {
            piece_bitboard | bit(to_square)
        } else {
            piece_bitboard
        }
    }

    #[inline(always)]
    pub fn remove_pawn_if_promotion(bitboard: Bitboard) -> Bitboard {
        bitboard & 0b0000000011111111111111111111111111111111111111111111111100000000
    }

    #[inline(always)]
    pub fn make_move_with_promotion(position: &mut Position, mv: Move, promotion_piece: Piece) {
        let from = from_square_part(mv);
        let to = to_square_part(mv);
        let m = position.mover;
        let current_move_number = position.move_number;
        let wp = remove_pawn_if_promotion(move_piece_within_bitboard(from, to, position.white_pawn_bitboard));
        let bp = remove_pawn_if_promotion(move_piece_within_bitboard(from, to, position.black_pawn_bitboard));
        let wn = create_if_promotion(promotion_piece == Knight, position.white_pawn_bitboard, move_piece_within_bitboard(from, to, position.white_knight_bitboard), from, to);
        let bn = create_if_promotion(promotion_piece == Knight, position.black_pawn_bitboard, move_piece_within_bitboard(from, to, position.black_knight_bitboard), from, to);
        let wb = create_if_promotion(promotion_piece == Bishop, position.white_pawn_bitboard, move_piece_within_bitboard(from, to, position.white_bishop_bitboard), from, to);
        let bb = create_if_promotion(promotion_piece == Bishop, position.black_pawn_bitboard, move_piece_within_bitboard(from, to, position.black_bishop_bitboard), from, to);
        let wr = create_if_promotion(promotion_piece == Rook, position.white_pawn_bitboard, move_piece_within_bitboard(from, to, position.white_rook_bitboard), from, to);
        let br = create_if_promotion(promotion_piece == Rook, position.black_pawn_bitboard, move_piece_within_bitboard(from, to, position.black_rook_bitboard), from, to);
        let wq = create_if_promotion(promotion_piece == Queen, position.white_pawn_bitboard, move_piece_within_bitboard(from, to, position.white_queen_bitboard), from, to);
        let bq = create_if_promotion(promotion_piece == Queen, position.black_pawn_bitboard, move_piece_within_bitboard(from ,to, position.black_queen_bitboard), from, to);
        let wk = move_piece_within_bitboard(from, to, position.white_king_bitboard);
        let bk = move_piece_within_bitboard(from, to, position.black_king_bitboard);
        let wpb = wp | wn | wr | wk | wq | wb;
        let bpb = bp | bn | br | bk | bq | bb;
        position.white_pawn_bitboard = wp;
        position.black_pawn_bitboard = bp;
        position.white_knight_bitboard = wn;
        position.black_knight_bitboard = bn;
        position.white_bishop_bitboard = wb;
        position.black_bishop_bitboard = bb;
        position.white_rook_bitboard = wr;
        position.black_rook_bitboard = br;
        position.white_queen_bitboard = wq;
        position.black_queen_bitboard = bq;
        position.white_king_bitboard = wk;
        position.black_king_bitboard = bk;
        position.all_pieces_bitboard = wpb | bpb;
        position.white_pieces_bitboard = wpb;
        position.black_pieces_bitboard = bpb;
        position.mover = switch_side(m);
        position.en_passant_square = EN_PASSANT_NOT_AVAILABLE;
        position.white_king_castle_available = position.white_king_castle_available && to != H1_BIT;
        position.white_queen_castle_available = position.white_queen_castle_available && to != A1_BIT;
        position.black_king_castle_available = position.black_king_castle_available && to != H8_BIT;
        position.black_queen_castle_available = position.black_queen_castle_available && to != A8_BIT;
        position.half_moves = 0;
        position.move_number = if m == Black { current_move_number + 1 } else { current_move_number }
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
    pub fn remove_piece_from_bitboard(square: Square, bitboard: Bitboard) -> Bitboard {
        !bit(square) & bitboard
    }

    #[inline(always)]
    pub fn move_white_rook_when_castling(from: Square, to: Square, king_board: Bitboard, rook_board: Bitboard) -> Bitboard {
        if from == E1_BIT && to == G1_BIT && test_bit(king_board, E1_BIT) {
            move_piece_within_bitboard(H1_BIT, F1_BIT, rook_board)
        } else if from == E1_BIT && to == C1_BIT && test_bit(king_board, E1_BIT) {
            move_piece_within_bitboard(A1_BIT, D1_BIT, rook_board)
        } else {
            rook_board
        }
    }

    #[inline(always)]
    pub fn move_black_rook_when_castling(from: Square, to: Square, king_board: Bitboard, rook_board: Bitboard) -> Bitboard {
        if from == E8_BIT && to == G8_BIT && test_bit(king_board, E8_BIT) {
            move_piece_within_bitboard(H8_BIT, F8_BIT, rook_board)
        } else if from == E8_BIT && to == C8_BIT && test_bit(king_board, E8_BIT) {
            move_piece_within_bitboard(A8_BIT, D8_BIT, rook_board)
        } else {
            rook_board
        }
    }

    #[inline(always)]
    pub fn make_simple_complex_move(position: &mut Position, from: Square, to: Square) {

        let current_move_number = position.move_number;
        let m = position.mover;
        let is_pawn_move = test_bit(position.white_pawn_bitboard | position.black_pawn_bitboard, from);
        position.white_pawn_bitboard = move_piece_within_bitboard(from, to, position.white_pawn_bitboard);
        position.black_pawn_bitboard = move_piece_within_bitboard(from, to, position.black_pawn_bitboard);

        if position.en_passant_square == to {
            if test_bit(position.black_pawn_bitboard, to) {
                position.white_pawn_bitboard = remove_piece_from_bitboard(en_passant_captured_piece_square(to), position.white_pawn_bitboard);
            } else if test_bit(position.white_pawn_bitboard, to) {
                position.black_pawn_bitboard = remove_piece_from_bitboard(en_passant_captured_piece_square(to), position.black_pawn_bitboard);
            }
        }

        let wn = move_piece_within_bitboard(from, to, position.white_knight_bitboard);
        let bn = move_piece_within_bitboard(from, to, position.black_knight_bitboard);
        let wb = move_piece_within_bitboard(from, to, position.white_bishop_bitboard);
        let bb = move_piece_within_bitboard(from, to, position.black_bishop_bitboard);

        let wr = move_white_rook_when_castling(from, to, position.white_king_bitboard, move_piece_within_bitboard(from, to, position.white_rook_bitboard));
        let br = move_black_rook_when_castling (from, to, position.black_king_bitboard, move_piece_within_bitboard(from, to, position.black_rook_bitboard));

        let wq = move_piece_within_bitboard(from, to, position.white_queen_bitboard);
        let bq = move_piece_within_bitboard(from, to, position.black_queen_bitboard);
        let wk = move_piece_within_bitboard(from, to, position.white_king_bitboard);
        let bk = move_piece_within_bitboard(from, to, position.black_king_bitboard);

        let wpb = position.white_pawn_bitboard | wn | wr | wk | wq | wb;
        let bpb = position.black_pawn_bitboard | bn | br | bk | bq | bb;

        position.half_moves = if test_bit(position.all_pieces_bitboard, to) || is_pawn_move { 0 } else { position.half_moves + 1 };

        position.white_knight_bitboard = wn;
        position.black_knight_bitboard = bn;
        position.white_bishop_bitboard = wb;
        position.black_bishop_bitboard = bb;
        position.white_rook_bitboard = wr;
        position.black_rook_bitboard = br;
        position.white_queen_bitboard = wq;
        position.black_queen_bitboard = bq;
        position.white_king_bitboard = wk;
        position.black_king_bitboard = bk;
        position.all_pieces_bitboard = wpb | bpb;
        position.white_pieces_bitboard = wpb;
        position.black_pieces_bitboard = bpb;
        position.mover = switch_side(m);

        position.en_passant_square = EN_PASSANT_NOT_AVAILABLE;
        position.white_king_castle_available = position.white_king_castle_available && from != E1_BIT && from != H1_BIT && to != H1_BIT;
        position.white_queen_castle_available = position.white_queen_castle_available && from != E1_BIT && from != A1_BIT && to != A1_BIT;
        position.black_king_castle_available = position.black_king_castle_available && from!= E8_BIT && from != H8_BIT && to != H8_BIT;
        position.black_queen_castle_available = position.black_queen_castle_available && from != E8_BIT && from != A8_BIT && to != A8_BIT;
        position.move_number = if m == Black { current_move_number + 1 } else { current_move_number };

    }

    #[inline(always)]
    pub fn switch_side(mover: Mover) -> Mover {
        if mover == White { Black } else { White }
    }

    #[inline(always)]
    pub fn make_simple_move(position: &mut Position, mv: Move, from: Square) {
        let to = to_square_part(mv);
        let switch_bitboard = bit(from) | bit(to);
        let piece = moving_piece(position, from);
        return if position.mover == White {
            make_simple_white_move(position, from, to, switch_bitboard, piece)
        } else {
            make_simple_black_move(position, from, to, switch_bitboard, piece)
        }
    }

    #[inline(always)]
    pub fn make_simple_white_move(position: &mut Position, from: Square, to: Square, switch_bitboard: Bitboard, piece: Piece) {
        position.mover = Black;
        position.all_pieces_bitboard = position.all_pieces_bitboard ^ switch_bitboard;
        position.white_pieces_bitboard = position.white_pieces_bitboard ^ switch_bitboard;
        return if piece == Pawn {
            position.white_pawn_bitboard = move_piece_within_bitboard(from, to, position.white_pawn_bitboard);
            position.en_passant_square = if to - from == 16 { from + 8 } else { EN_PASSANT_NOT_AVAILABLE };
            position.half_moves = 0;
        } else {
            position.half_moves += 1;
            position.en_passant_square = EN_PASSANT_NOT_AVAILABLE;
            if piece == Knight {
                position.white_knight_bitboard = move_piece_within_bitboard(from, to, position.white_knight_bitboard);
            } else if piece == Bishop {
                position.white_bishop_bitboard = move_piece_within_bitboard(from, to, position.white_bishop_bitboard);
            } else if piece == Rook {
                position.white_rook_bitboard = move_piece_within_bitboard(from, to, position.white_rook_bitboard);
                position.white_king_castle_available = position.white_king_castle_available && from != H1_BIT;
                position.white_queen_castle_available = position.white_queen_castle_available && from != A1_BIT;
            } else if piece == Queen {
                position.white_queen_bitboard = move_piece_within_bitboard(from, to, position.white_queen_bitboard);
            } else {
                position.white_king_bitboard = bit(to);
            }
        }
    }

    #[inline(always)]
    pub fn make_simple_black_move(position: &mut Position, from: Square, to: Square, switch_bitboard: Bitboard, piece: Piece) {
        position.move_number += 1;
        position.mover = White;
        position.all_pieces_bitboard = position.all_pieces_bitboard ^ switch_bitboard;
        position.black_pieces_bitboard = position.black_pieces_bitboard ^ switch_bitboard;
        return if piece == Pawn {
            position.black_pawn_bitboard = move_piece_within_bitboard(from, to, position.black_pawn_bitboard);
            position.en_passant_square = if from - to == 16 { from - 8 } else { EN_PASSANT_NOT_AVAILABLE };
            position.half_moves = 0;
        } else {
            position.half_moves += 1;
            position.en_passant_square = EN_PASSANT_NOT_AVAILABLE;
            if piece == Knight {
                position.black_knight_bitboard = move_piece_within_bitboard(from, to, position.black_knight_bitboard);
            } else if piece == Bishop {
                position.black_bishop_bitboard = move_piece_within_bitboard(from, to, position.black_bishop_bitboard);
            } else if piece == Rook {
                position.black_rook_bitboard = move_piece_within_bitboard(from, to, position.black_rook_bitboard);
                position.black_king_castle_available = position.black_king_castle_available && from != H8_BIT;
                position.black_queen_castle_available = position.black_queen_castle_available && from != A8_BIT;
            } else if piece == Queen {
                position.black_queen_bitboard = move_piece_within_bitboard(from, to, position.black_queen_bitboard);
            } else {
                position.black_king_bitboard = bit(to);
            }
        }
    }

    #[inline(always)]
    pub fn moving_piece(position: &Position, from_square: Square) -> Piece {
        if test_bit(position.white_pawn_bitboard | position.black_pawn_bitboard, from_square) { return Pawn }
        if test_bit(position.white_knight_bitboard | position.black_knight_bitboard, from_square) { return Knight }
        if test_bit(position.white_bishop_bitboard | position.black_bishop_bitboard, from_square) { return Bishop }
        if test_bit(position.white_rook_bitboard | position.black_rook_bitboard, from_square) { return Rook }
        if test_bit(position.white_queen_bitboard | position.black_queen_bitboard, from_square) { return Queen }
        return King
    }

    #[inline(always)]
    pub fn is_simple_move(position: &mut Position, from: Square, to: Square, piece: Piece) -> bool {
        return
            !is_simple_capture(position, to) &&
                !(piece == Pawn && is_complex_pawn_move(from, to)) &&
                    !(piece == King && is_potential_first_king_move(from));
    }

    #[inline(always)]
    pub fn is_simple_capture(position: &mut Position, square: Square) -> bool {
        test_bit(position.all_pieces_bitboard, square)
    }

    // todo - remove position param
    #[inline(always)]
    pub fn is_complex_pawn_move(from: Square, to: Square) -> bool {
        return (from - to).abs() % 8 != 0 || test_bit(PROMOTION_SQUARES, to);
    }

    // todo - remove position param
    #[inline(always)]
    pub fn is_potential_first_king_move(from: Square) -> bool {
        return from == E1_BIT as Square || from == E8_BIT as Square
    }

    pub fn default_position_history() -> PositionHistory {
        PositionHistory {
            history: [Position {
                white_pawn_bitboard: 0,
                white_knight_bitboard: 0,
                white_bishop_bitboard: 0,
                white_queen_bitboard: 0,
                white_king_bitboard: 0,
                white_rook_bitboard: 0,
                black_pawn_bitboard: 0,
                black_knight_bitboard: 0,
                black_bishop_bitboard: 0,
                black_queen_bitboard: 0,
                black_king_bitboard: 0,
                black_rook_bitboard: 0,
                all_pieces_bitboard: 0,
                white_pieces_bitboard: 0,
                black_pieces_bitboard: 0,
                mover: Mover::White,
                en_passant_square: 0,
                white_king_castle_available: false,
                black_king_castle_available: false,
                white_queen_castle_available: false,
                black_queen_castle_available: false,
                half_moves: 0,
                move_number: 1
            }; MAX_MOVE_HISTORY as usize]
        }
    }

}