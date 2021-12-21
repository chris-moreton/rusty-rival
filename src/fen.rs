pub mod fen {
    pub fn bit_array_to_decimal(is: Vec<u8>) -> u64 {
        todo!()
        // bitArrayToDecimal :: [Int] -> Int
        // bitArrayToDecimal bits = recurBitArrayToDecimal bits 63 0
        // recurBitArrayToDecimal :: [Int] -> Int -> Int -> Int
        // recurBitArrayToDecimal _ (-1) result = result
        // recurBitArrayToDecimal bits bitnum result = do
        // let thisResult = if head bits == 1 then shiftL 1 bitnum else 0
        // recurBitArrayToDecimal (tail bits) (bitnum - 1) (result + thisResult)
    }

    pub fn board_bits(fen_ranks: Vec<String>, piece_char: char) -> String {
        // fn board_bits(fen_ranks: Vec<String>, piece_char: char, result: String) {
        //
        // }
        todo!()
        // recurBoardBits :: [String] -> Char -> [Int] -> [Int]
        // recurBoardBits [] _ result = result
        // recurBoardBits fenRanks pieceChar result = do
        // let thisResult = rankBits (head fenRanks) pieceChar
        // recurBoardBits (tail fenRanks) pieceChar (result ++ thisResult)
    }

    pub fn rank_bits(rank: String, piece: char) -> Vec<u8> {
        fn rank_bits(rank: String, piece: char, result: Vec<u8>) -> Vec<u8> {
            if (rank.chars().count == 0) {
                return result;
            }

        }

        rank_bits(rank, piece, Vec::new());
    }

    let c = head fenRankChars
    let thisResult
    | isFileNumber c = replicate (ord c - 48) 0
    | pieceChar == c = [1]
    | otherwise = [0]
    recurRankBits (tail fenRankChars) pieceChar (result ++ thisResult)

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


    pub fn get_fen_ranks(fen: String) -> Vec<String> {
        todo!()
        // getFenRanks :: String -> [String]
        // getFenRanks = splitOn "/"
    }

    pub fn fen_part(fen: String, i: u8) -> String {
        return fen.split(" ").collect()[i]
    }

    pub fn fen_board_part(fen: String) -> String {
        return fen_part(fen, 0)
    }

    pub fn get_mover(fen: String) -> Mover {
        let m = fen_part(fen, 1);
        return if m == "w" { White } else { Black }
    }

    pub fn piece_bitboard(fen_ranks: Vec<String>, piece: char) -> Mover {
        todo!()
        // pieceBitboard :: [String] -> Char -> Bitboard
        // pieceBitboard fenRanks pieceChar = fromIntegral(bitArrayToDecimal (boardBits fenRanks pieceChar)) :: Bitboard
    }

    pub fn get_position(fen: String) -> Position {
        let fen_ranks = get_fen_ranks(fen_board_part(fen));
        let mover = get_mover(fen);
        let wp = piece_bitboard(fen_ranks, 'P');
        let bp = piece_bitboard(fen_ranks, 'p');
        let wn = piece_bitboard(fen_ranks, 'N');
        let bn = piece_bitboard(fen_ranks, 'n');
        let wb = piece_bitboard(fen_ranks, 'B');
        let bb = piece_bitboard(fen_ranks, 'b');
        let wr = piece_bitboard(fen_ranks, 'R');
        let br = piece_bitboard(fen_ranks, 'r');
        let wq = piece_bitboard(fen_ranks, 'Q');
        let bq = piece_bitboard(fen_ranks, 'q');
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
            all_pieces_bitboard: wp | bp | wn | bn | wb | bb | wr | br | wq | bq | wk | bk,
            white_pieces_bitboard: wp | wn | wb | wr | wq | wk,
            black_pieces_bitboard: bp | bn | bb | br | bq | bk,
            mover: get_mover(fen),
            en_passant_square: en_passant_bit_ref(en_passant_fen_part(fen)),
            white_king_castle_available: castle_part.contains("K"),
            white_queen_castle_available: castle_part.contains("Q"),
            black_king_castle_available: castle_part.contains("k"),
            black_queen_castle_available: castle_part.contains("q"),
            half_moves: fen_part(fen, 4).parse::<u16>().unwrap(),
            move_number: fen_part(fen, 5).parse::<u16>().unwrap(),
        }
    }
}
