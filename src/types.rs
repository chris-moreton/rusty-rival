pub mod types {
    use std::collections::{HashMap, LinkedList};

    pub type Square = i8;
    pub type Bitboard = u64;
    pub type Move = u32;
    pub type MoveList = Vec<Move>;
    pub type Path = Vec<Move>;
    pub type MagicFunc = fn(Square, u64) -> Bitboard;

    #[derive(Debug, PartialEq)]
    pub enum Mover { White, Black }

    #[derive(Debug, PartialEq)]
    pub enum Piece { Pawn, King, Queen, Bishop, Knight, Rook }

    #[derive(Debug, PartialEq)]
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

}
