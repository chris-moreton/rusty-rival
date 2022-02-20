use std::collections::HashMap;
use crate::engine_constants::{MAX_DEPTH, NUM_KILLER_MOVES};
use crate::move_constants::{BK_CASTLE, BQ_CASTLE, START_POS, WK_CASTLE, WQ_CASTLE};

pub type Square = i8;
pub type Bitboard = u64;
pub type Move = u32;
pub type MoveList = Vec<Move>;
pub type Path = Vec<Move>;
pub type MagicMovesArray = [[Bitboard; 4096]; 64];
pub type Mover = i8;
pub type Bound = Score;
pub type Window = (Bound, Bound);
pub type Score = i32;
pub type HashIndex = u32;
pub type HashLock = u128;
pub type MoveScore = (Move, Score);
pub type MoveScoreList = Vec<MoveScore>;
pub type PositionHistory = Vec<HashLock>;

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
        registered_name: "Rustival".parse().unwrap(),
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
    pub hash_table: HashMap<HashIndex, HashEntry>,
    pub hash_table_version: u32,
    pub killer_moves: [[Move; NUM_KILLER_MOVES]; MAX_DEPTH as usize],
    pub pv: Path,
    pub pv_score: Score,
    pub nodes: u64,
    pub hash_hits_exact: u64,
    pub history: PositionHistory,
}

pub fn default_search_state() -> SearchState {
    SearchState {
        hash_table: Default::default(),
        hash_table_version: 0,
        killer_moves: [[0,0]; MAX_DEPTH as usize],
        pv: vec![],
        pv_score: 0,
        nodes: 0,
        hash_hits_exact: 0,
        history: vec![],
    }
}

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
    ($a:expr) => { $a ^ 1 }
}

#[macro_export]
macro_rules! unset_lsb {
    ($a:expr) => { $a &= $a - 1 }
}

#[macro_export]
macro_rules! get_and_unset_lsb {
    ($a:expr) => {
        {
            let lsb = $a.trailing_zeros() as Square;
            $a &= $a - 1;
            lsb
        }
    }
}

pub const WHITE: Mover = 0;
pub const BLACK: Mover = 1;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Piece { Pawn, King, Queen, Bishop, Knight, Rook, Empty }

#[derive(Debug, PartialEq)]
pub enum BoundType { Exact, Lower, Upper }

#[inline(always)]
pub fn unset_white_castles(position: &mut Position) { position.castle_flags &= !(WK_CASTLE | WQ_CASTLE) }
#[inline(always)]
pub fn unset_black_castles(position: &mut Position) { position.castle_flags &= !(BK_CASTLE | BQ_CASTLE) }

#[inline(always)]
pub fn is_wk_castle_available(position: &Position) -> bool { position.castle_flags & WK_CASTLE != 0 }
#[inline(always)]
pub fn is_wq_castle_available(position: &Position) -> bool { position.castle_flags & WQ_CASTLE != 0 }
#[inline(always)]
pub fn is_bk_castle_available(position: &Position) -> bool { position.castle_flags & BK_CASTLE != 0 }
#[inline(always)]
pub fn is_bq_castle_available(position: &Position) -> bool { position.castle_flags & BQ_CASTLE != 0 }

#[derive(Debug, Copy, Clone)]
pub struct Pieces {
    pub pawn_bitboard: Bitboard,
    pub knight_bitboard: Bitboard,
    pub bishop_bitboard: Bitboard,
    pub queen_bitboard: Bitboard,
    pub king_square: Square,
    pub rook_bitboard: Bitboard,
    pub all_pieces_bitboard: Bitboard
}

impl PartialEq for Pieces {
    fn eq(&self, other: &Self) -> bool {
        self.pawn_bitboard == other.pawn_bitboard &&
        self.knight_bitboard == other.knight_bitboard &&
        self.bishop_bitboard == other.bishop_bitboard &&
        self.queen_bitboard == other.queen_bitboard &&
        self.king_square == other.king_square &&
        self.rook_bitboard == other.rook_bitboard &&
        self.all_pieces_bitboard == other.all_pieces_bitboard
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
        self.pieces[0] == other.pieces[0] &&
        self.pieces[1] == other.pieces[1] &&
        self.mover == other.mover &&
        self.en_passant_square == other.en_passant_square &&
        self.castle_flags == other.castle_flags &&
        self.half_moves == other.half_moves &&
        self.move_number == other.move_number
    }
}
