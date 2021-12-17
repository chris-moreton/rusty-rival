use std::collections::HashMap;

type Square = u64;
type Bitboard = u64;
type Move = u64;
type MoveList = ConsList<Move>;
type Path = ConsList<Move>;
type MagicFunc = fn(Square, Int) -> Bitboard;

enum Mover { White, Black }
enum Piece { Pawn, King, Queen, Bishop, Knight,  Rook }
enum Bound { Exact, Lower, Upper }

struct HashEntry {
    score: u16,
    he_path: Path,
    height: u16,
    bound: Bound,
    lock: u64
}

struct MoveScore {
    ms_score: u16,
    ms_bound: Bound,
    ms_path: Path,
}

type HashTable = HashMap<u64, HashEntry>;
type MagicHashTable = HashMap<u64, u64>;

struct Position {
    white_pawn_bitboard: Bitboard,
    white_knight_bitboard: Bitboard,
    white_bishop_bitboard: Bitboard,
    white_queen_bitboard: Bitboard,
    white_king_bitboard: Bitboard,
    white_rook_bitboard: Bitboard,
    black_pawn_bitboard: Bitboard,
    black_knight_bitboard: Bitboard,
    black_bishop_bitboard: Bitboard,
    black_queen_bitboard: Bitboard,
    black_king_bitboard: Bitboard,
    black_rook_bitboard: Bitboard,
    all_pieces_bitboard: Bitboard,
    white_pieces_bitboard: Bitboard,
    black_pieces_bitboard: Bitboard,
    mover: Mover,
    en_passant_square: Square,
    white_king_castle_available: Bool,
    black_king_castle_available: Bool,
    white_queen_castle_available: Bool,
    black_queen_castle_available: Bool,
    half_moves: Int,
    move_number: Int,
}


instance Eq Position where
a == b = whitePawnBitboard a == whitePawnBitboard b &&
whiteKnightBitboard a == whiteKnightBitboard b &&
whiteBishopBitboard a == whiteBishopBitboard b &&
whiteQueenBitboard a == whiteQueenBitboard b &&
whiteKingBitboard a == whiteKingBitboard b &&
whiteRookBitboard a == whiteRookBitboard b &&
blackPawnBitboard a == blackPawnBitboard b &&
blackKnightBitboard a == blackKnightBitboard b &&
blackBishopBitboard a == blackBishopBitboard b &&
blackQueenBitboard a == blackQueenBitboard b &&
blackKingBitboard a == blackKingBitboard b &&
blackPiecesBitboard a == blackPiecesBitboard b &&
enPassantSquare a == enPassantSquare b &&
whiteKingCastleAvailable a == whiteKingCastleAvailable b &&
whiteQueenCastleAvailable a == whiteQueenCastleAvailable b &&
blackKingCastleAvailable a == blackKingCastleAvailable b &&
blackQueenCastleAvailable a == blackQueenCastleAvailable b &&
mover a == mover b

{-# INLINE bitboardForMover #-}
bitboardForMover :: Position -> Piece -> Bitboard
bitboardForMover !position = bitboardForColour position (mover position)

{-# INLINE bitboardForColour #-}
bitboardForColour :: Position -> Mover -> Piece -> Bitboard
bitboardForColour !pieceBitboards White King = whiteKingBitboard pieceBitboards
bitboardForColour !pieceBitboards White Queen = whiteQueenBitboard pieceBitboards
bitboardForColour !pieceBitboards White Rook = whiteRookBitboard pieceBitboards
bitboardForColour !pieceBitboards White Knight = whiteKnightBitboard pieceBitboards
bitboardForColour !pieceBitboards White Bishop = whiteBishopBitboard pieceBitboards
bitboardForColour !pieceBitboards White Pawn = whitePawnBitboard pieceBitboards
bitboardForColour !pieceBitboards Black King = blackKingBitboard pieceBitboards
bitboardForColour !pieceBitboards Black Queen = blackQueenBitboard pieceBitboards
bitboardForColour !pieceBitboards Black Rook = blackRookBitboard pieceBitboards
bitboardForColour !pieceBitboards Black Knight = blackKnightBitboard pieceBitboards
bitboardForColour !pieceBitboards Black Bishop = blackBishopBitboard pieceBitboards
bitboardForColour !pieceBitboards Black Pawn = blackPawnBitboard pieceBitboards

{-# INLINE sliderBitboardForColour #-}
sliderBitboardForColour :: Position -> Mover -> Piece -> Bitboard
sliderBitboardForColour !pieceBitboards White Rook = whiteRookBitboard pieceBitboards .|. whiteQueenBitboard pieceBitboards
sliderBitboardForColour !pieceBitboards White Bishop = whiteBishopBitboard pieceBitboards .|. whiteQueenBitboard pieceBitboards
sliderBitboardForColour !pieceBitboards Black Rook = blackRookBitboard pieceBitboards .|. blackQueenBitboard pieceBitboards
sliderBitboardForColour !pieceBitboards Black Bishop = blackBishopBitboard pieceBitboards .|. blackQueenBitboard pieceBitboards
