pub mod types {
    use std::collections::{HashMap, LinkedList};

    type Square = u64;
    type Bitboard = u64;
    type Move = u64;
    type MoveList = LinkedList<Move>;
    type Path = LinkedList<Move>;
    type MagicFunc = fn(Square, u64) -> Bitboard;

    enum Mover { White, Black }

    enum Piece { Pawn, King, Queen, Bishop, Knight, Rook }

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
        white_king_castle_available: bool,
        black_king_castle_available: bool,
        white_queen_castle_available: bool,
        black_queen_castle_available: bool,
        half_moves: u16,
        move_number: u16,
    }

    impl PartialEq for Mover {
        fn eq(&self, other: &Self) -> bool {
            self == other
        }
    }

    impl PartialEq for Position {
        fn eq(&self, other: &Self) -> bool {
            self.white_pawn_bitboard == other.white_pawn_bitboard &&
            self.white_knight_bitboard == other.white_knight_bitboard &&
            self.white_bishop_bitboard == other.white_bishop_bitboard &&
            self.white_queen_bitboard == other.white_queen_bitboard &&
            self.white_king_bitboard == other.white_king_bitboard &&
            self.white_rook_bitboard == other.white_rook_bitboard &&

            self.black_pawn_bitboard == other.black_pawn_bitboard &&
            self.black_knight_bitboard == other.black_knight_bitboard &&
            self.black_bishop_bitboard == other.black_bishop_bitboard &&
            self.black_queen_bitboard == other.black_queen_bitboard &&
            self.black_pawn_bitboard == other.black_pawn_bitboard &&
            self.black_king_bitboard == other.black_king_bitboard &&

            self.black_pieces_bitboard == other.black_pieces_bitboard &&
            self.en_passant_square == other.en_passant_square &&
            self.white_king_castle_available == other.white_king_castle_available &&
            self.white_queen_castle_available == other.white_queen_castle_available &&
            self.black_king_castle_available == other.black_king_castle_available &&
            self.black_queen_castle_available == other.black_queen_castle_available &&
            self.mover == other.mover
        }
    }

    impl Eq for Position {}

    pub fn bitboard_for_mover(position: &Position, piece: &Piece) -> Bitboard {
        bitboard_for_colour(position, &position.mover, piece)
    }

    fn bitboard_for_colour(position: &Position, mover: &Mover, piece: &Piece) -> Bitboard {
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

    fn slider_bitboard_for_colour(position: &Position, mover: &Mover, piece: &Piece) -> Bitboard {
        match (mover, piece) {
            (Mover::White, Piece::Rook) => position.white_rook_bitboard | position.white_queen_bitboard,
            (Mover::White, Piece::Bishop) => position.white_bishop_bitboard | position.white_queen_bitboard,
            (Mover::Black, Piece::Rook) => position.black_rook_bitboard | position.black_queen_bitboard,
            (Mover::Black, Piece::Bishop) => position.black_bishop_bitboard | position.black_queen_bitboard,
            _ => panic!("Can't handle piece")
        }
    }
}