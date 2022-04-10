use crate::engine_constants::{MAX_DEPTH, NUM_HASH_ENTRIES, NUM_KILLER_MOVES};
use crate::move_constants::{BK_CASTLE, BQ_CASTLE, START_POS, WK_CASTLE, WQ_CASTLE};
use std::time::Instant;

pub type Square = i8;
pub type Bitboard = u64;
pub type Move = u32;
pub type MoveList = Vec<Move>;
pub type MagicMovesArray = [[Bitboard; 4096]; 64];
pub type Mover = i8;
pub type Bound = Score;
pub type Window = (Bound, Bound);
pub type Score = i32;
pub type HashLock = u128;
pub type HashIndex = u32;
pub type HashArray = [HashEntry; NUM_HASH_ENTRIES as usize];
pub type PathScore = (Vec<Move>, Score);
pub type MoveScore = (Move, Score);
pub type MoveScoreList = Vec<MoveScore>;
pub type PositionHistory = Vec<HashLock>;
pub type HistoryScore = i64;
pub type ScorePair = (Score, Score);

pub struct UciState {
    pub fen: String,
    pub debug: bool,
    pub registered_name: String,
    pub wtime: u64,
    pub btime: u64,
    pub winc: u64,
    pub binc: u64,
    pub moves_to_go: u64,
    pub depth: u64,
    pub nodes: u64,
    pub mate: bool,
    pub move_time: u64,
    pub infinite: bool,
}

pub fn default_uci_state() -> UciState {
    UciState {
        fen: START_POS.to_string(),
        debug: false,
        registered_name: "Rusty Rival".parse().unwrap(),
        wtime: u64::MAX,
        btime: u64::MAX,
        winc: 0,
        binc: 0,
        moves_to_go: 0,
        depth: u64::MAX,
        nodes: 0,
        mate: false,
        move_time: u64::MAX,
        infinite: false,
    }
}

pub struct SearchState {
    pub current_best: PathScore,
    pub start_time: Instant,
    pub end_time: Instant,
    pub iterative_depth: u8,
    pub hash_table_height: Box<HashArray>,
    pub hash_table_version: u32,
    pub killer_moves: [[Move; NUM_KILLER_MOVES]; MAX_DEPTH as usize],
    pub mate_killer: [Move; MAX_DEPTH as usize],
    pub history_moves: [[[HistoryScore; 64]; 64]; 12],
    pub highest_history_score: HistoryScore,
    pub nodes: u64,
    pub show_info: bool,
    pub hash_hits_exact: u64,
    pub hash_clashes: u64,
    pub history: PositionHistory,
}

pub fn default_search_state() -> SearchState {
    SearchState {
        current_best: (vec![], 0),
        start_time: Instant::now(),
        end_time: Instant::now(),
        iterative_depth: 0,
        hash_table_height: Box::try_from(
            vec![
                HashEntry {
                    score: 0,
                    version: 0,
                    height: 0,
                    mv: 0,
                    bound: BoundType::Exact,
                    lock: 0
                };
                NUM_HASH_ENTRIES as usize
            ]
            .into_boxed_slice(),
        )
        .unwrap(),
        hash_table_version: 1,
        killer_moves: [[0, 0]; MAX_DEPTH as usize],
        mate_killer: [0; MAX_DEPTH as usize],
        history_moves: [[[0; 64]; 64]; 12],
        highest_history_score: 0,
        nodes: 0,
        show_info: true,
        hash_hits_exact: 0,
        hash_clashes: 0,
        history: vec![],
    }
}

pub struct EvaluateCache {
    pub piece_count: u8,
    pub white_pawn_files: Option<u8>,
    pub black_pawn_files: Option<u8>,
    pub white_pawn_attacks: Option<Bitboard>,
    pub black_pawn_attacks: Option<Bitboard>,
    pub white_passed_knights: Option<Bitboard>,
    pub black_passed_knights: Option<Bitboard>,
    pub white_guarded_passed_knights: Option<Bitboard>,
    pub black_guarded_passed_knights: Option<Bitboard>,
}

pub fn default_evaluate_cache() -> EvaluateCache {
    EvaluateCache {
        piece_count: 0,
        white_pawn_files: None,
        black_pawn_files: None,
        white_pawn_attacks: None,
        black_pawn_attacks: None,
        white_passed_knights: None,
        black_passed_knights: None,
        white_guarded_passed_knights: None,
        black_guarded_passed_knights: None,
    }
}

#[derive(Debug, Copy, Clone)]
pub struct HashEntry {
    pub score: Score,
    pub version: u32,
    pub height: u8,
    pub mv: Move,
    pub bound: BoundType,
    pub lock: HashLock,
}

#[macro_export]
macro_rules! opponent {
    ($a:expr) => {
        $a ^ 1
    };
}

#[macro_export]
macro_rules! unset_lsb {
    ($a:expr) => {
        $a &= $a - 1
    };
}

#[macro_export]
macro_rules! get_and_unset_lsb {
    ($a:expr) => {{
        let lsb = $a.trailing_zeros() as Square;
        $a &= $a - 1;
        lsb
    }};
}

pub const WHITE: Mover = 0;
pub const BLACK: Mover = 1;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Piece {
    Pawn,
    King,
    Queen,
    Bishop,
    Knight,
    Rook,
    Empty,
}

#[derive(Debug, PartialEq)]
pub enum BoundType {
    Exact,
    Lower,
    Upper,
}

impl Clone for BoundType {
    fn clone(&self) -> Self {
        todo!()
    }
}

impl Copy for BoundType {}

#[inline(always)]
pub fn unset_white_castles(position: &mut Position) {
    position.castle_flags &= !(WK_CASTLE | WQ_CASTLE)
}
#[inline(always)]
pub fn unset_black_castles(position: &mut Position) {
    position.castle_flags &= !(BK_CASTLE | BQ_CASTLE)
}

#[inline(always)]
pub fn is_wk_castle_available(position: &Position) -> bool {
    position.castle_flags & WK_CASTLE != 0
}
#[inline(always)]
pub fn is_wq_castle_available(position: &Position) -> bool {
    position.castle_flags & WQ_CASTLE != 0
}
#[inline(always)]
pub fn is_bk_castle_available(position: &Position) -> bool {
    position.castle_flags & BK_CASTLE != 0
}
#[inline(always)]
pub fn is_bq_castle_available(position: &Position) -> bool {
    position.castle_flags & BQ_CASTLE != 0
}

#[derive(Debug, Copy, Clone)]
pub struct Pieces {
    pub pawn_bitboard: Bitboard,
    pub knight_bitboard: Bitboard,
    pub bishop_bitboard: Bitboard,
    pub queen_bitboard: Bitboard,
    pub king_square: Square,
    pub rook_bitboard: Bitboard,
    pub all_pieces_bitboard: Bitboard,
}

impl PartialEq for Pieces {
    fn eq(&self, other: &Self) -> bool {
        self.pawn_bitboard == other.pawn_bitboard
            && self.knight_bitboard == other.knight_bitboard
            && self.bishop_bitboard == other.bishop_bitboard
            && self.queen_bitboard == other.queen_bitboard
            && self.king_square == other.king_square
            && self.rook_bitboard == other.rook_bitboard
            && self.all_pieces_bitboard == other.all_pieces_bitboard
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Position {
    pub pieces: [Pieces; 2],
    pub mover: Mover,
    pub en_passant_square: Square,
    pub castle_flags: u8,
    pub half_moves: u16,
    pub move_number: u16,
    pub zobrist_lock: u128,
}

impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        self.pieces[0] == other.pieces[0]
            && self.pieces[1] == other.pieces[1]
            && self.mover == other.mover
            && self.en_passant_square == other.en_passant_square
            && self.castle_flags == other.castle_flags
            && self.half_moves == other.half_moves
            && self.move_number == other.move_number
    }
}
