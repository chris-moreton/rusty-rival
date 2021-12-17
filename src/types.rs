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

impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        self.whitePawnBitboard == other.whitePawnBitboard &&
        self.whiteKnightBitboard == other.whiteKnightBitboard &&
        self.whiteBishopBitboard == other.whiteBishopBitboard &&
        self.whiteQueenBitboard == other.whiteQueenBitboard &&
        self.whiteKingBitboard == other.whiteKingBitboard &&
        self.whiteRookBitboard == other.whiteRookBitboard &&
        self.blackPawnBitboard == other.blackPawnBitboard &&
        self.blackKnightBitboard == other.blackKnightBitboard &&
        self.blackBishopBitboard == other.blackBishopBitboard &&
        self.blackQueenBitboard == other.blackQueenBitboard &&
        self.blackKingBitboard == other.blackKingBitboard &&
        self.blackPiecesBitboard == other.blackPiecesBitboard &&
        self.enPassantSquare == other.enPassantSquare &&
        self.whiteKingCastleAvailable == other.whiteKingCastleAvailable &&
        self.whiteQueenCastleAvailable == other.whiteQueenCastleAvailable &&
        self.blackKingCastleAvailable == other.blackKingCastleAvailable &&
        self.blackQueenCastleAvailable == other.blackQueenCastleAvailable &&
        self.mover == other.mover
    }
}

impl Eq for Position { }

fn bitboard_for_mover(&position: Position, &piece: Piece) {
    bitboardForColour(position, position.mover, piece)
}

fn bitboard_for_colour(&position: Position, &mover: Mover, &piece: Piece) {
    match (mover, piece) {
        (Mover::White, Piece::King) => position.white_king_bitboard,
        (Mover::White, Piece::Queen) => position.white_queen_bitboard,
        (Mover::White, Piece::Rook) => position.white_rook_bitboard,
        (Mover::White, Piece::Knight) => position.white_knight_bitboard,
        (Mover::White, Piece::Bishop) => position.white_bishop_bitboard,
        (Mover::White, Piece::Pawn) => position.white_pawn_bitboard,
        (Mover::Black, Piece::King) => position.black_king_bitboard,
        (Mover::Black, Piece::Queen) => position.black_queen_bitboard,
        (Mover::Black, Piece::Rook) => position.black_rook_bitboard,
        (Mover::Black, Piece::Knight) => position.black_knight_bitboard,
        (Mover::Black, Piece::Bishop) => position.black_bishop_bitboard,
        (Mover::Black, Piece::Pawn) => position.black_pawn_bitboard,
    }
}

fn slider_bitboard_for_colour(&position: Position, &mover: Mover, &piece: Piece) {
    match (mover, piece) {
        (Mover::White, Piece::Rook) => position.white_rook_bitboard | position.white_queen_bitboard,
        (Mover::White, Piece::Bishop) => position.white_bishop_bitboard | position.white_queen_bitboard,
        (Mover::Black, Piece::Rook) => position.black_rook_bitboard | position.black_queen_bitboard,
        (Mover::Black, Piece::Bishop) => position.black_bishop_bitboard | position.black_queen_bitboard,
    }
}
