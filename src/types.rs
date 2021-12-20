pub mod types {
    use std::collections::{HashMap, LinkedList};

    pub type Square = u64;
    pub type Bitboard = u64;
    pub type Move = u64;
    pub type MoveList = LinkedList<Move>;
    pub type Path = LinkedList<Move>;
    pub type MagicFunc = fn(Square, u64) -> Bitboard;

    pub enum Mover { White, Black }

    pub enum Piece { Pawn, King, Queen, Bishop, Knight, Rook }

    pub enum Bound { Exact, Lower, Upper }

    pub struct HashEntry {
        score: u16,
        he_path: Path,
        height: u16,
        bound: Bound,
        lock: u64
    }

    pub struct MoveScore {
        ms_score: u16,
        ms_bound: Bound,
        ms_path: Path,
    }

    pub type HashTable = HashMap<u64, HashEntry>;
    pub type MagicHashTable = HashMap<u64, u64>;

    pub struct Position {
        pub white_pawn_bitboard: Bitboard,
        pub white_knight_bitboard: Bitboard,
        pub white_bishop_bitboard: Bitboard,
        pub white_queen_bitboard: Bitboard,
        pub white_king_bitboard: Bitboard,
        pub white_rook_bitboard: Bitboard,
        pub black_pawn_bitboard: Bitboard,
        pub black_knight_bitboard: Bitboard,
        pub black_bishop_bitboard: Bitboard,
        pub black_queen_bitboard: Bitboard,
        pub black_king_bitboard: Bitboard,
        pub black_rook_bitboard: Bitboard,
        pub all_pieces_bitboard: Bitboard,
        pub white_pieces_bitboard: Bitboard,
        pub black_pieces_bitboard: Bitboard,
        pub mover: Mover,
        pub en_passant_square: Square,
        pub white_king_castle_available: bool,
        pub black_king_castle_available: bool,
        pub white_queen_castle_available: bool,
        pub black_queen_castle_available: bool,
        pub half_moves: u16,
        pub move_number: u16,
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
}
