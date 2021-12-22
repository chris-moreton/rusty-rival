pub mod fen {
    use std::iter;
    use crate::types::types::{Bitboard, Mover, Position, Square};
    use crate::types::types::Mover::{Black, White};
    const EN_PASSANT_UNAVAILABLE: i8 = -1;

    pub fn bit_array_to_decimal(is: Vec<u8>) -> u64 {
        let mut total: u64 = 0;
        for x in 0..64 {
            if is[x] == 1 {
                total += 1 << (63-x);
            }
        }
        return total;
    }

    pub fn board_bits(fen_ranks: Vec<String>, piece_char: char) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();
        fen_ranks.iter().for_each(|item| {
            result.append(&mut rank_bits(item, piece_char))
        });
        return result;
    }

    fn is_file_number(c: char) -> bool {
        return c >= '0' && c <= '9';
    }

    pub fn char_as_num(c: char) -> u8 {
        return c as u8 - 48;
    }

    pub fn rank_bits(rank: &String, piece: char) -> Vec<u8> {
        return rank_bits(rank, piece, Vec::new());

        fn rank_bits(rank: &String, piece: char, mut result: Vec<u8>) -> Vec<u8> {
            if rank.chars().count() == 0 {
                return result;
            }
            let c = rank.chars().nth(0).unwrap();
            let mut new_rank = rank.clone();
            new_rank.remove(0);
            if is_file_number(c) {
                for x in 0..char_as_num(c) {
                    result.push(0)
                }
            } else if piece == c {
                result.push(1);
            } else {
                result.push(0);
            };

            return rank_bits(&new_rank, piece, result)
        }
    }

    pub fn algebraic_squareref_from_bitref(bitref: u8) -> String {
        let rank = (bitref / 8) + 1;
        let file = 8 - (bitref % 8);
        let rank_char = (rank + 48) as char;
        let file_char = (file + 96) as char;
        return file_char.to_string() + &*(rank_char.to_string());
    }
    // algebraicSquareRefFromBitRef :: Int -> String
    // algebraicSquareRefFromBitRef bitRef = do
    // let rank = quot bitRef 8 + 1
    // let file = 8 - mod bitRef 8
    // let rankChar = chr (rank + 48)
    // let fileChar = chr (file + 96)
    // [fileChar,rankChar]
    //
    // bitRefFromAlgebraicSquareRef :: String -> Int
    // bitRefFromAlgebraicSquareRef algebraic = do
    // let fileNum = ord (head algebraic) - 97
    // let rankNum = ord (head (tail algebraic)) - 49
    // (rankNum * 8) + (7 - fileNum)
    //
    // promotionPart :: Move -> String
    // promotionPart move
    // | (.&.) promotionFullMoveMask move == promotionQueenMoveMask = "q"
    // | (.&.) promotionFullMoveMask move == promotionRookMoveMask = "r"
    // | (.&.) promotionFullMoveMask move == promotionBishopMoveMask = "b"
    // | (.&.) promotionFullMoveMask move == promotionKnightMoveMask = "n"
    // | otherwise = ""
    //
    // promotionMask :: Char -> Int
    // promotionMask pieceChar
    // | pieceChar == 'q' = promotionQueenMoveMask
    // | pieceChar == 'b' = promotionBishopMoveMask
    // | pieceChar == 'r' = promotionRookMoveMask
    // | pieceChar == 'n' = promotionKnightMoveMask
    // | otherwise = 0
    //
    // algebraicMoveFromMove :: Move -> String
    // algebraicMoveFromMove move = do
    // let fromSquare = shiftR move 16
    // let toSquare = (.&.) 63 move
    // algebraicSquareRefFromBitRef fromSquare ++ algebraicSquareRefFromBitRef toSquare ++ promotionPart move
    //
    // moveFromAlgebraicMove :: String -> Move
    // moveFromAlgebraicMove moveString =
    // fromSquareMask (bitRefFromAlgebraicSquareRef (substring moveString 0 2)) + bitRefFromAlgebraicSquareRef (substring moveString 2 4) + promotionMask (last moveString)


    pub fn get_fen_ranks(fenBoardPart: String) -> Vec<String> {
        return fenBoardPart.split("/").map(|s| s.to_string()).collect();
    }

    pub fn fen_part(fen: &String, i: u8) -> String {
        let parts: Vec<&str> = fen.split(" ").collect();
        return String::from(parts[i as usize])
    }

    pub fn fen_board_part(fen: &String) -> String {
        return fen_part(fen, 0)
    }

    pub fn get_mover(fen: &String) -> Mover {
        let m = fen_part(fen, 1);
        return if m == "w" { White } else { Black }
    }

    pub fn piece_bitboard(fen_ranks: &Vec<String>, piece: char) -> Bitboard {
        todo!()
        // pieceBitboard :: [String] -> Char -> Bitboard
        // pieceBitboard fenRanks pieceChar = fromIntegral(bitArrayToDecimal (boardBits fenRanks pieceChar)) :: Bitboard
    }

    fn en_passant_fen_part(fen: &String) -> String {
        return fen_part(fen, 3);
    }

    fn bit_ref_from_algebraic_square_ref(algebraic: &String) -> i8 {
        todo!()
        // let fileNum = ord (head algebraic) - 97
        // let rankNum = ord (head (tail algebraic)) - 49
        //     (rankNum * 8) + (7 - fileNum)
    }

    fn en_passant_bit_ref(en_passant_fen_part: &String) -> i8 {
        return if en_passant_fen_part == "-" {
            EN_PASSANT_UNAVAILABLE
        } else {
            bit_ref_from_algebraic_square_ref(en_passant_fen_part)
        }
    }

    pub fn get_position(fen: &String) -> Position {
        let fen_ranks = get_fen_ranks(fen_board_part(fen));
        let mover = get_mover(fen);
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
        return Position {
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
            en_passant_square: en_passant_bit_ref(&en_passant_fen_part(fen)) as Square,
            white_king_castle_available: castle_part.contains("K"),
            white_queen_castle_available: castle_part.contains("Q"),
            black_king_castle_available: castle_part.contains("k"),
            black_queen_castle_available: castle_part.contains("q"),
            half_moves: fen_part(fen, 4).parse::<u16>().unwrap(),
            move_number: fen_part(fen, 5).parse::<u16>().unwrap(),
        }
    }
}
