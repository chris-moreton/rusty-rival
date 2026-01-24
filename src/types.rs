use crate::engine_constants::{MAX_DEPTH, NUM_HASH_ENTRIES, NUM_KILLER_MOVES, NUM_PAWN_HASH_ENTRIES};
use crate::move_constants::{BK_CASTLE, BQ_CASTLE, START_POS, WK_CASTLE, WQ_CASTLE};
use arrayvec::ArrayVec;
use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Instant;

pub type Square = i8;
pub type Bitboard = u64;
pub type Move = u32;
pub const MAX_MOVES: usize = 256;
pub type MoveList = ArrayVec<Move, MAX_MOVES>;
pub type MagicMovesArray = [[Bitboard; 4096]; 64];
pub type Mover = i8;
pub type Bound = Score;
pub type Window = (Bound, Bound);
pub type Score = i32;
pub type HashLock = u128;
pub type HashIndex = u32;
pub type HashArray = [HashEntry; NUM_HASH_ENTRIES as usize];

/// Thread-safe wrapper for the hash table that allows sharing between threads.
/// Uses UnsafeCell for interior mutability - data races on individual hash entries
/// are acceptable in chess engines (worst case is a cache miss or stale data).
pub struct SharedHashTable {
    data: UnsafeCell<Box<HashArray>>,
}

// SAFETY: Hash table data races are acceptable in chess engines.
// The worst case is reading stale/corrupted data which just causes
// a cache miss - it doesn't cause crashes or undefined behavior.
unsafe impl Send for SharedHashTable {}
unsafe impl Sync for SharedHashTable {}

impl SharedHashTable {
    pub fn new() -> Self {
        SharedHashTable {
            data: UnsafeCell::new(
                Box::try_from(
                    vec![
                        HashEntry {
                            score: 0,
                            version: 0,
                            height: 0,
                            mv: 0,
                            bound: BoundType::Exact,
                            lock: 0,
                        };
                        NUM_HASH_ENTRIES as usize
                    ]
                    .into_boxed_slice(),
                )
                .unwrap(),
            ),
        }
    }

    /// Get a reference to a hash entry (for reading)
    #[inline(always)]
    pub fn get(&self, index: usize) -> &HashEntry {
        unsafe { &(*self.data.get())[index] }
    }

    /// Get a mutable reference to a hash entry (for writing)
    #[inline(always)]
    pub fn set(&self, index: usize, entry: HashEntry) {
        unsafe {
            (*self.data.get())[index] = entry;
        }
    }

    /// Prefetch a hash entry into CPU cache
    /// Call this after making a move to hide memory latency
    #[inline(always)]
    pub fn prefetch(&self, index: usize) {
        #[cfg(target_arch = "x86_64")]
        {
            use std::arch::x86_64::{_mm_prefetch, _MM_HINT_T0};
            unsafe {
                let ptr = &(*self.data.get())[index] as *const HashEntry as *const i8;
                _mm_prefetch(ptr, _MM_HINT_T0);
            }
        }
        #[cfg(target_arch = "x86")]
        {
            use std::arch::x86::{_mm_prefetch, _MM_HINT_T0};
            unsafe {
                let ptr = &(*self.data.get())[index] as *const HashEntry as *const i8;
                _mm_prefetch(ptr, _MM_HINT_T0);
            }
        }
        // No-op on other architectures (ARM, etc.)
        #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
        {
            let _ = index;
        }
    }

    /// Clear the hash table (used by ucinewgame)
    pub fn clear(&self) {
        let empty = HashEntry {
            score: 0,
            version: 0,
            height: 0,
            mv: 0,
            bound: BoundType::Exact,
            lock: 0,
        };
        for i in 0..NUM_HASH_ENTRIES as usize {
            self.set(i, empty);
        }
    }
}

impl Default for SharedHashTable {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for SharedHashTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SharedHashTable({} entries)", NUM_HASH_ENTRIES)
    }
}

// Pawn hash table entry - caches pawn structure evaluation
#[derive(Copy, Clone, Default)]
pub struct PawnHashEntry {
    pub key: u64,     // Lower 64 bits of pawn Zobrist key for verification
    pub score: Score, // Cached pawn structure score
}

// Pawn hash table - smaller dedicated cache for pawn structure evaluation
pub struct PawnHashTable {
    data: UnsafeCell<Box<[PawnHashEntry; NUM_PAWN_HASH_ENTRIES]>>,
}

// SAFETY: Same reasoning as SharedHashTable - data races just cause cache misses
unsafe impl Send for PawnHashTable {}
unsafe impl Sync for PawnHashTable {}

impl PawnHashTable {
    pub fn new() -> Self {
        PawnHashTable {
            data: UnsafeCell::new(Box::new([PawnHashEntry::default(); NUM_PAWN_HASH_ENTRIES])),
        }
    }

    #[inline(always)]
    pub fn get(&self, key: HashLock) -> Option<Score> {
        let index = (key as usize) % NUM_PAWN_HASH_ENTRIES;
        let key_lower = key as u64;
        // SAFETY: We accept data races as they only cause cache misses
        let entry = unsafe { &(*self.data.get())[index] };
        if entry.key == key_lower {
            Some(entry.score)
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn set(&self, key: HashLock, score: Score) {
        let index = (key as usize) % NUM_PAWN_HASH_ENTRIES;
        let key_lower = key as u64;
        // SAFETY: We accept data races as they only cause cache misses
        unsafe {
            let entry = &mut (*self.data.get())[index];
            entry.key = key_lower;
            entry.score = score;
        }
    }

    #[inline(always)]
    pub fn prefetch(&self, key: HashLock) {
        let index = (key as usize) % NUM_PAWN_HASH_ENTRIES;
        #[cfg(target_arch = "x86_64")]
        {
            use std::arch::x86_64::{_mm_prefetch, _MM_HINT_T0};
            // SAFETY: We're prefetching a valid address within our allocation
            unsafe {
                let ptr = &(*self.data.get())[index] as *const PawnHashEntry as *const i8;
                _mm_prefetch(ptr, _MM_HINT_T0);
            }
        }
        #[cfg(target_arch = "x86")]
        {
            use std::arch::x86::{_mm_prefetch, _MM_HINT_T0};
            unsafe {
                let ptr = &(*self.data.get())[index] as *const PawnHashEntry as *const i8;
                _mm_prefetch(ptr, _MM_HINT_T0);
            }
        }
        // On other architectures, prefetch is a no-op
        #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
        {
            let _ = index;
        }
    }

    pub fn clear(&self) {
        // SAFETY: Called before search starts, no concurrent access expected
        unsafe {
            let data = &mut *self.data.get();
            for entry in data.iter_mut() {
                *entry = PawnHashEntry::default();
            }
        }
    }
}

impl Default for PawnHashTable {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for PawnHashTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PawnHashTable({} entries)", NUM_PAWN_HASH_ENTRIES)
    }
}

pub const MAX_PV_LENGTH: usize = 64;
pub type PV = ArrayVec<Move, MAX_PV_LENGTH>;
pub type PathScore = (PV, Score);

/// Create a PV with a single move
#[inline(always)]
pub fn pv_single(m: Move) -> PV {
    let mut pv = PV::new();
    pv.push(m);
    pv
}

/// Create a PV with a move prepended to an existing PV
#[inline(always)]
pub fn pv_prepend(m: Move, rest: &PV) -> PV {
    let mut pv = PV::new();
    pv.push(m);
    pv.try_extend_from_slice(rest).ok();
    pv
}

pub type MoveScore = (Move, Score);
pub type MoveScoreList = Vec<MoveScore>;
pub type MoveScoreArray = ArrayVec<MoveScore, MAX_MOVES>;
pub type PositionHistory = Vec<HashLock>;
pub type HistoryScore = i64;
pub type ScorePair = (Score, Score);

/// Information needed to unmake a move
#[derive(Copy, Clone)]
pub struct UnmakeInfo {
    pub castle_flags: u8,
    pub en_passant_square: Square,
    pub half_moves: u16,
    pub zobrist_lock: HashLock,
    pub captured_piece: u8, // 0 = none, 1-5 = pawn/knight/bishop/rook/queen
}

#[derive(Debug, Clone)]
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
    pub quit: bool,
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
        quit: false,
    }
}

/// Holds the handle to a running search thread
pub struct SearchHandle {
    pub stop: Arc<AtomicBool>,
    pub handle: JoinHandle<()>,
}

impl SearchHandle {
    /// Signal the search to stop and wait for it to finish
    pub fn stop_and_wait(self) {
        set_stop(&self.stop, true);
        let _ = self.handle.join();
    }
}

#[derive(Debug)]
pub struct SearchState {
    pub current_best: PathScore,
    pub root_moves: MoveScoreList,
    pub start_time: Instant,
    pub end_time: Instant,
    pub iterative_depth: u8,
    pub hash_table: Arc<SharedHashTable>,
    pub hash_table_version: u32,
    pub pawn_hash_table: Arc<PawnHashTable>,
    pub killer_moves: [[Move; NUM_KILLER_MOVES]; MAX_DEPTH as usize],
    pub mate_killer: [Move; MAX_DEPTH as usize],
    pub countermoves: [[Move; 64]; 12],       // [piece_12][to_square] -> best countermove
    pub ply_move: [Move; MAX_DEPTH as usize], // Track move at each ply for countermove lookup
    pub history_moves: [[[HistoryScore; 64]; 64]; 12],
    pub highest_history_score: HistoryScore,
    pub nodes: u64,
    pub nodes_limit: u64,
    pub show_info: bool,
    pub hash_hits_exact: u64,
    pub pv: HashMap<Move, PathScore>,
    pub hash_clashes: u64,
    pub history: PositionHistory,
    pub multi_pv: u8,
    pub contempt: Score,
    pub ignore_root_move: Move,
    pub stop: Arc<AtomicBool>,
    pub last_info_nodes: u64,
}

impl Clone for SearchState {
    fn clone(&self) -> Self {
        SearchState {
            current_best: self.current_best.clone(),
            root_moves: self.root_moves.clone(),
            start_time: self.start_time,
            end_time: self.end_time,
            iterative_depth: self.iterative_depth,
            // Share the hash table via Arc - no 128MB copy!
            hash_table: Arc::clone(&self.hash_table),
            hash_table_version: self.hash_table_version,
            pawn_hash_table: Arc::clone(&self.pawn_hash_table),
            killer_moves: self.killer_moves,
            mate_killer: self.mate_killer,
            countermoves: self.countermoves,
            ply_move: self.ply_move,
            history_moves: self.history_moves,
            highest_history_score: self.highest_history_score,
            nodes: self.nodes,
            nodes_limit: self.nodes_limit,
            show_info: self.show_info,
            hash_hits_exact: self.hash_hits_exact,
            pv: self.pv.clone(),
            hash_clashes: self.hash_clashes,
            history: self.history.clone(),
            multi_pv: self.multi_pv,
            contempt: self.contempt,
            ignore_root_move: self.ignore_root_move,
            stop: Arc::clone(&self.stop),
            last_info_nodes: self.last_info_nodes,
        }
    }
}

pub fn default_search_state() -> SearchState {
    SearchState {
        current_best: (PV::new(), 0),
        root_moves: vec![],
        start_time: Instant::now(),
        end_time: Instant::now(),
        iterative_depth: 0,
        hash_table: Arc::new(SharedHashTable::new()),
        hash_table_version: 1,
        pawn_hash_table: Arc::new(PawnHashTable::new()),
        killer_moves: [[0, 0]; MAX_DEPTH as usize],
        mate_killer: [0; MAX_DEPTH as usize],
        countermoves: [[0; 64]; 12],
        ply_move: [0; MAX_DEPTH as usize],
        history_moves: [[[0; 64]; 64]; 12],
        highest_history_score: 0,
        nodes: 0,
        nodes_limit: u64::MAX,
        show_info: true,
        hash_hits_exact: 0,
        pv: HashMap::new(),
        hash_clashes: 0,
        history: vec![],
        multi_pv: 1,
        contempt: 0,
        ignore_root_move: 0,
        stop: Arc::new(AtomicBool::new(false)),
        last_info_nodes: 0,
    }
}

/// Helper to check if stop flag is set
#[inline(always)]
pub fn is_stopped(stop: &Arc<AtomicBool>) -> bool {
    stop.load(Ordering::Relaxed)
}

/// Helper to set the stop flag
#[inline(always)]
pub fn set_stop(stop: &Arc<AtomicBool>, value: bool) {
    stop.store(value, Ordering::Relaxed);
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

#[macro_export]
macro_rules! get_lsb {
    ($a:expr) => {{
        let lsb = $a.trailing_zeros() as Square;
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
        *self
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
