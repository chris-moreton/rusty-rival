use crate::engine_constants::{
    CORRECTION_HISTORY_SIZE, HASH_ENTRY_BYTES, MAX_DEPTH, MAX_QUIESCE_DEPTH, NUM_KILLER_MOVES, NUM_PAWN_HASH_ENTRIES,
};
use crate::move_constants::{BK_CASTLE, BQ_CASTLE, START_POS, WK_CASTLE, WQ_CASTLE};
use crate::nnue;
use arrayvec::ArrayVec;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
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
/// Lockless shared transposition table.
///
/// Each entry is three relaxed AtomicU64 words: two data words plus a
/// checksum word (`key ^ data1 ^ data2`, where key is the high 64 bits of
/// the 128-bit zobrist lock - the low bits pick the index). A torn read or
/// write makes the checksum mismatch on probe, so racing threads can never
/// smuggle a foreign move, score or bound through: no locks, no UB, and no
/// need to trust unverified entries.
pub struct SharedHashTable {
    data: Box<[[AtomicU64; 3]]>,
    num_entries: usize,
    version: AtomicU32,
}

const HASH_WORD_CHECK: usize = 0;
const HASH_WORD_MOVE_SCORE: usize = 1; // mv in low 32 bits, score in high 32
const HASH_WORD_META: usize = 2; // height 0..8, bound 8..10, version 16..48, static_eval 48..64

/// Sentinel stored in the static-eval field when no static eval is known.
/// `i16::MIN` is unreachable as a real eval, so it doubles as "absent" without
/// costing an extra bit. Entries written before this field existed decode as 0,
/// which is a legal eval - hence the version check on read is *not* enough on
/// its own and every writer must set the field explicitly.
pub const STATIC_EVAL_NONE: Score = i16::MIN as Score;

#[inline(always)]
fn hash_entry_key(lock: HashLock) -> u64 {
    (lock >> 64) as u64
}

impl SharedHashTable {
    /// Create a new hash table with the default size
    pub fn new() -> Self {
        Self::new_with_mb(128) // Default 128 MB
    }

    /// Create a new hash table with specified size in megabytes
    pub fn new_with_mb(mb: usize) -> Self {
        let raw_entries = ((mb * 1024 * 1024) / HASH_ENTRY_BYTES as usize).max(1);
        // Round DOWN to a power of two so the index can use a mask instead of a
        // modulo. `next_power_of_two() >> 1` gets this wrong when raw_entries is
        // already a power of two - it returns the value unchanged, then halves
        // it - throwing away half the requested memory. With 24-byte entries
        // that happens for Hash = 3*2^k MB: 3, 6, 12, 24, 48 and 96, the last
        // of which is the default (NET-374).
        let num_entries = 1usize << raw_entries.ilog2();
        Self::new_with_entries(num_entries)
    }

    /// Create a new hash table with specified number of entries (must be power of 2)
    pub fn new_with_entries(num_entries: usize) -> Self {
        debug_assert!(num_entries.is_power_of_two(), "Hash table size must be power of 2");
        let data: Vec<[AtomicU64; 3]> = std::iter::repeat_with(|| [AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0)])
            .take(num_entries)
            .collect();
        SharedHashTable {
            data: data.into_boxed_slice(),
            num_entries,
            version: AtomicU32::new(1),
        }
    }

    /// Get the number of entries in the hash table
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.num_entries
    }

    /// Get the bitmask for indexing (num_entries - 1, since size is power of 2)
    #[inline(always)]
    pub fn mask(&self) -> u64 {
        (self.num_entries - 1) as u64
    }

    /// Check if the hash table is empty
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.num_entries == 0
    }

    /// Get the size of the hash table in megabytes
    pub fn size_mb(&self) -> usize {
        (self.num_entries * HASH_ENTRY_BYTES as usize) / (1024 * 1024)
    }

    /// Current search generation, shared by all threads (bumped once per `go`)
    #[inline(always)]
    pub fn version(&self) -> u32 {
        self.version.load(Ordering::Relaxed)
    }

    /// Advance the search generation - call once per `go`, from one thread
    pub fn bump_version(&self) {
        self.version.fetch_add(1, Ordering::Relaxed);
    }

    /// Probe for the given lock. Returns the decoded entry only when the
    /// checksum confirms an untorn entry written for this exact lock.
    #[inline(always)]
    pub fn probe(&self, index: usize, lock: HashLock) -> Option<HashEntry> {
        let words = &self.data[index];
        let check = words[HASH_WORD_CHECK].load(Ordering::Relaxed);
        let move_score = words[HASH_WORD_MOVE_SCORE].load(Ordering::Relaxed);
        let meta = words[HASH_WORD_META].load(Ordering::Relaxed);
        if check ^ move_score ^ meta != hash_entry_key(lock) || (check | move_score | meta) == 0 {
            return None;
        }
        let bound = match (meta >> 8) & 0x3 {
            0 => BoundType::Exact,
            1 => BoundType::Lower,
            _ => BoundType::Upper,
        };
        Some(HashEntry {
            score: (move_score >> 32) as u32 as Score,
            version: (meta >> 16) as u32,
            height: meta as u8,
            mv: move_score as u32 as Move,
            bound,
            lock,
            static_eval: (meta >> 48) as u16 as i16 as Score,
        })
    }

    /// Height/version/occupancy of whatever occupies the slot (no key check) -
    /// used for the replacement decision. Torn values only ever influence
    /// which entry gets overwritten, never correctness.
    #[inline(always)]
    pub fn entry_meta(&self, index: usize) -> (u8, u32, bool) {
        let words = &self.data[index];
        let check = words[HASH_WORD_CHECK].load(Ordering::Relaxed);
        let move_score = words[HASH_WORD_MOVE_SCORE].load(Ordering::Relaxed);
        let meta = words[HASH_WORD_META].load(Ordering::Relaxed);
        (meta as u8, (meta >> 16) as u32, (check | move_score | meta) != 0)
    }

    /// Encode and store an entry (entry.lock supplies the checksum key)
    #[inline(always)]
    pub fn store(&self, index: usize, entry: HashEntry) {
        let move_score = entry.mv as u64 | ((entry.score as u32 as u64) << 32);
        let bound_bits = match entry.bound {
            BoundType::Exact => 0u64,
            BoundType::Lower => 1u64,
            BoundType::Upper => 2u64,
        };
        // Clamp rather than wrap: a mate-magnitude eval would otherwise alias to
        // a small value and silently poison every pruning margin that reads it.
        let static_eval = entry.static_eval.clamp(i16::MIN as Score, i16::MAX as Score) as i16;
        let meta = entry.height as u64 | (bound_bits << 8) | ((entry.version as u64) << 16) | ((static_eval as u16 as u64) << 48);
        let words = &self.data[index];
        words[HASH_WORD_CHECK].store(hash_entry_key(entry.lock) ^ move_score ^ meta, Ordering::Relaxed);
        words[HASH_WORD_MOVE_SCORE].store(move_score, Ordering::Relaxed);
        words[HASH_WORD_META].store(meta, Ordering::Relaxed);
    }

    /// Prefetch a hash entry into CPU cache
    /// Call this after making a move to hide memory latency
    #[inline(always)]
    pub fn prefetch(&self, index: usize) {
        #[cfg(target_arch = "x86_64")]
        {
            use std::arch::x86_64::{_mm_prefetch, _MM_HINT_T0};
            unsafe {
                let ptr = &self.data[index] as *const [AtomicU64; 3] as *const i8;
                _mm_prefetch(ptr, _MM_HINT_T0);
            }
        }
        #[cfg(target_arch = "x86")]
        {
            use std::arch::x86::{_mm_prefetch, _MM_HINT_T0};
            unsafe {
                let ptr = &self.data[index] as *const [AtomicU64; 3] as *const i8;
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
        for words in self.data.iter() {
            words[HASH_WORD_CHECK].store(0, Ordering::Relaxed);
            words[HASH_WORD_MOVE_SCORE].store(0, Ordering::Relaxed);
            words[HASH_WORD_META].store(0, Ordering::Relaxed);
        }
        self.version.store(1, Ordering::Relaxed);
    }
}

impl Default for SharedHashTable {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for SharedHashTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SharedHashTable({} entries, {} MB)", self.num_entries, self.size_mb())
    }
}

/// Pawn hash entry: two relaxed atomic words, `key ^ score` and `score`.
///
/// The previous version was a plain `{u64, i32}` written through an UnsafeCell
/// with non-atomic stores, justified by "data races just cause cache misses".
/// That was false, and it was UB (NET-374). Two writers interleaving could
/// leave writer A's key paired with writer B's score, and the reader would
/// then VERIFY that pair and return a wrong pawn score for a key it believes
/// it matched - silent eval corruption under Threads > 1, not a cache miss.
///
/// The XOR check is the same trick SharedHashTable uses: a torn pair fails
/// `key == word0 ^ word1` and simply reads as a miss.
#[derive(Default)]
pub struct PawnHashEntry {
    key_xor_score: AtomicU64,
    score: AtomicU64,
}

// Pawn hash table - smaller dedicated cache for pawn structure evaluation
pub struct PawnHashTable {
    data: Box<[PawnHashEntry]>,
}

impl PawnHashTable {
    pub fn new() -> Self {
        PawnHashTable {
            data: std::iter::repeat_with(PawnHashEntry::default)
                .take(NUM_PAWN_HASH_ENTRIES)
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        }
    }

    #[inline(always)]
    pub fn get(&self, key: HashLock) -> Option<Score> {
        let index = (key as usize) % NUM_PAWN_HASH_ENTRIES;
        let key_lower = key as u64;
        let entry = &self.data[index];
        let check = entry.key_xor_score.load(Ordering::Relaxed);
        let score = entry.score.load(Ordering::Relaxed);
        // A torn write fails this and reads as a miss, which is correct and
        // costs only a recomputation
        if check ^ score == key_lower {
            Some(score as u32 as Score)
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn set(&self, key: HashLock, score: Score) {
        let index = (key as usize) % NUM_PAWN_HASH_ENTRIES;
        let key_lower = key as u64;
        let score_bits = score as u32 as u64;
        let entry = &self.data[index];
        entry.key_xor_score.store(key_lower ^ score_bits, Ordering::Relaxed);
        entry.score.store(score_bits, Ordering::Relaxed);
    }

    #[inline(always)]
    pub fn prefetch(&self, key: HashLock) {
        let index = (key as usize) % NUM_PAWN_HASH_ENTRIES;
        #[cfg(target_arch = "x86_64")]
        {
            use std::arch::x86_64::{_mm_prefetch, _MM_HINT_T0};
            // SAFETY: prefetching a valid address within our allocation
            unsafe {
                let ptr = &self.data[index] as *const PawnHashEntry as *const i8;
                _mm_prefetch(ptr, _MM_HINT_T0);
            }
        }
        #[cfg(target_arch = "x86")]
        {
            use std::arch::x86::{_mm_prefetch, _MM_HINT_T0};
            unsafe {
                let ptr = &self.data[index] as *const PawnHashEntry as *const i8;
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
        for entry in self.data.iter() {
            entry.key_xor_score.store(0, Ordering::Relaxed);
            entry.score.store(0, Ordering::Relaxed);
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
pub type HistoryScore = i16;
pub type ScorePair = (Score, Score);

/// Information needed to unmake a move
#[derive(Copy, Clone)]
pub struct UnmakeInfo {
    pub castle_flags: u8,
    pub en_passant_square: Square,
    pub half_moves: u16,
    pub zobrist_lock: HashLock,
    pub pawn_key: u64,
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
    pub threads: usize,
    pub ponder_enabled: bool,
    pub move_overhead: u64,
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
        threads: 1,
        ponder_enabled: false,
        move_overhead: 10,
    }
}

/// Holds handles to running search threads
pub struct SearchHandle {
    pub stop: Arc<AtomicBool>,
    pub pondering: Arc<AtomicBool>,
    pub ponder_soft_ms: Arc<AtomicU64>,
    pub ponder_hard_ms: Arc<AtomicU64>,
    pub handles: Vec<JoinHandle<()>>,
}

impl SearchHandle {
    /// Signal the search to stop and wait for all threads to finish
    pub fn stop_and_wait(self) {
        set_stop(&self.stop, true);
        for handle in self.handles {
            let _ = handle.join();
        }
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
    pub pawn_hash_table: Arc<PawnHashTable>,
    pub killer_moves: [[Move; NUM_KILLER_MOVES]; MAX_DEPTH as usize],
    pub mate_killer: [Move; MAX_DEPTH as usize],
    pub countermoves: [[Move; 64]; 12],                       // [piece_12][to_square] -> best countermove
    pub countermove_history: Box<[[[[i16; 64]; 6]; 64]; 12]>, // [prev_piece_12][prev_to][curr_piece_6][curr_to]
    pub followup_history: Box<[[[[i16; 64]; 6]; 64]; 6]>,     // [our_prev_piece_6][our_prev_to][curr_piece_6][curr_to]
    pub capture_history: [[[i16; 64]; 6]; 6],                 // [attacker_piece_6][victim_piece_6][to_square]
    pub ply_move: [Move; MAX_DEPTH as usize],                 // Track move at each ply for countermove lookup
    pub eval_stack: [Score; MAX_DEPTH as usize],              // Static eval per ply (-Score::MAX when in check)
    pub correction_history: Box<[[i16; CORRECTION_HISTORY_SIZE]; 2]>, // [side][pawn_key % SIZE] -> eval correction
    pub history_moves: Box<[[[HistoryScore; 64]; 64]; 12]>,
    pub nodes: u64,
    pub nodes_limit: u64,
    pub show_info: bool,
    pub hash_hits_exact: u64,
    // Move-ordering quality (NET-493). Every beta cutoff, and how many of them
    // came from the first move tried at that node. The ratio is the standard
    // measure of ordering quality: a well-ordered search cuts on move 1 the
    // large majority of the time, because a node that needed six tries wasted
    // five subtrees. Counted at the single choke point `cutoff_unmake`.
    pub cutoffs: u64,
    pub cutoffs_first_move: u64,
    // Which ordering heuristic produced the cutting move, and how deep into the
    // move list it sat (NET-493). `cutoffs_first_move` says 11.5% of cutoffs
    // need more than one try; these say *which* heuristic is failing to put the
    // right move first, which is the difference between a TT replacement
    // problem and a history-ranking one.
    // kind:  0=TT  1=capture  2=killer  3=countermove  4=other quiet
    pub cutoff_by_kind: [u64; 5],
    // index: 0=move 1  1=move 2  2=move 3  3=moves 4-6  4=move 7+
    pub cutoff_by_index: [u64; 5],
    // Quiescence nodes only (NET-493). `nodes` counts both, so main-search nodes
    // are `nodes - qnodes`. Splitting them separates the two causes of our
    // oversized tree: a growth-rate problem shows in the effective branching
    // factor, a constant-factor problem shows here.
    pub qnodes: u64,
    pub pv: HashMap<Move, PathScore>,
    pub hash_clashes: u64,
    pub history: PositionHistory,
    pub multi_pv: u8,
    pub contempt: Score,
    pub ignore_root_move: Move,
    pub search_moves: Option<Vec<Move>>,
    pub stop: Arc<AtomicBool>,
    pub last_info_nodes: u64,
    pub shared_nodes: Arc<AtomicU64>,
    pub thread_id: usize,
    pub soft_time_limit: Instant,
    // Ceiling for cumulative soft-limit extensions (NET-339). Lives on SearchState
    // (not a local in iterative_deepening) so the ponderhit conversion in
    // check_time! can rebase it on the REAL budget instead of the 24h ponder
    // placeholder (NET-362).
    pub max_soft_time_limit: Instant,
    pub best_move_stability: u8,
    pub prev_best_move: Move,
    pub prev_score: Score,
    pub time_management_active: bool,
    pub pondering: Arc<AtomicBool>,
    pub ponder_soft_ms: Arc<AtomicU64>,
    pub ponder_hard_ms: Arc<AtomicU64>,
    pub is_ponder_search: bool,
    pub ponder_applied: bool,
    pub eval_noise: Score, // Max noise in centipawns (0 = disabled, e.g. 15 = ±15cp)
    pub use_nnue: bool,
    pub nnue_network: Option<Arc<nnue::NnueNetwork>>,
    pub nnue_accumulators: Vec<nnue::Accumulator>,
    pub nnue_pieces: Vec<[Pieces; 2]>,
    pub nnue_computed: Vec<bool>,
    pub nnue_ply: usize,
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
            pawn_hash_table: Arc::clone(&self.pawn_hash_table),
            killer_moves: self.killer_moves,
            mate_killer: self.mate_killer,
            countermoves: self.countermoves,
            countermove_history: self.countermove_history.clone(),
            followup_history: self.followup_history.clone(),
            capture_history: self.capture_history,
            ply_move: self.ply_move,
            eval_stack: self.eval_stack,
            correction_history: self.correction_history.clone(),
            history_moves: self.history_moves.clone(),
            nodes: self.nodes,
            nodes_limit: self.nodes_limit,
            show_info: self.show_info,
            hash_hits_exact: self.hash_hits_exact,
            // Per-thread counters: a cloned helper thread starts its own tally
            // rather than inheriting the parent's, so a multi-threaded bench
            // does not double-count the main thread's cutoffs.
            cutoffs: 0,
            cutoffs_first_move: 0,
            cutoff_by_kind: [0; 5],
            cutoff_by_index: [0; 5],
            qnodes: 0,
            pv: self.pv.clone(),
            hash_clashes: self.hash_clashes,
            history: self.history.clone(),
            multi_pv: self.multi_pv,
            contempt: self.contempt,
            ignore_root_move: self.ignore_root_move,
            search_moves: self.search_moves.clone(),
            stop: Arc::clone(&self.stop),
            last_info_nodes: self.last_info_nodes,
            shared_nodes: Arc::clone(&self.shared_nodes),
            thread_id: self.thread_id,
            soft_time_limit: self.soft_time_limit,
            max_soft_time_limit: self.max_soft_time_limit,
            best_move_stability: self.best_move_stability,
            prev_best_move: self.prev_best_move,
            prev_score: self.prev_score,
            time_management_active: self.time_management_active,
            pondering: Arc::clone(&self.pondering),
            ponder_soft_ms: Arc::clone(&self.ponder_soft_ms),
            ponder_hard_ms: Arc::clone(&self.ponder_hard_ms),
            is_ponder_search: self.is_ponder_search,
            ponder_applied: self.ponder_applied,
            eval_noise: self.eval_noise,
            use_nnue: self.use_nnue,
            nnue_network: self.nnue_network.clone(),
            nnue_accumulators: self.nnue_accumulators.clone(),
            nnue_pieces: self.nnue_pieces.clone(),
            nnue_computed: self.nnue_computed.clone(),
            nnue_ply: self.nnue_ply,
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
        pawn_hash_table: Arc::new(PawnHashTable::new()),
        killer_moves: [[0, 0]; MAX_DEPTH as usize],
        mate_killer: [0; MAX_DEPTH as usize],
        countermoves: [[0; 64]; 12],
        countermove_history: Box::new([[[[0; 64]; 6]; 64]; 12]),
        followup_history: Box::new([[[[0; 64]; 6]; 64]; 6]),
        capture_history: [[[0; 64]; 6]; 6],
        ply_move: [0; MAX_DEPTH as usize],
        eval_stack: [-Score::MAX; MAX_DEPTH as usize],
        correction_history: Box::new([[0; CORRECTION_HISTORY_SIZE]; 2]),
        history_moves: Box::new([[[0; 64]; 64]; 12]),
        nodes: 0,
        nodes_limit: u64::MAX,
        show_info: true,
        hash_hits_exact: 0,
        cutoffs: 0,
        cutoffs_first_move: 0,
        cutoff_by_kind: [0; 5],
        cutoff_by_index: [0; 5],
        qnodes: 0,
        pv: HashMap::new(),
        hash_clashes: 0,
        history: vec![],
        multi_pv: 1,
        contempt: 0,
        ignore_root_move: 0,
        search_moves: None,
        stop: Arc::new(AtomicBool::new(false)),
        last_info_nodes: 0,
        shared_nodes: Arc::new(AtomicU64::new(0)),
        thread_id: 0,
        soft_time_limit: Instant::now(),
        max_soft_time_limit: Instant::now(),
        best_move_stability: 0,
        prev_best_move: 0,
        prev_score: 0,
        time_management_active: false,
        pondering: Arc::new(AtomicBool::new(false)),
        ponder_soft_ms: Arc::new(AtomicU64::new(0)),
        ponder_hard_ms: Arc::new(AtomicU64::new(0)),
        is_ponder_search: false,
        ponder_applied: false,
        eval_noise: 0,
        use_nnue: true,
        nnue_network: Some(Arc::new(nnue::NnueNetwork::embedded())),
        // Sized for the deepest possible chain: search ply + quiesce recursion
        nnue_accumulators: (0..(MAX_DEPTH as usize + MAX_QUIESCE_DEPTH as usize + 2))
            .map(|_| nnue::Accumulator::new())
            .collect(),
        nnue_pieces: vec![[Pieces::default(); 2]; MAX_DEPTH as usize + MAX_QUIESCE_DEPTH as usize + 2],
        nnue_computed: vec![false; MAX_DEPTH as usize + MAX_QUIESCE_DEPTH as usize + 2],
        nnue_ply: 0,
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
    /// Raw static eval of the position (NNUE output, *before* correction
    /// history), or `STATIC_EVAL_NONE`. Cached so a TT hit can skip the NNUE
    /// forward pass - the dominant per-node cost. Depth-independent, so it is
    /// usable even when the entry's height is too shallow for a score cutoff.
    pub static_eval: Score,
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

#[derive(Debug, Copy, Clone, Default)]
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
    /// Zobrist key over pawn placement only, maintained incrementally
    /// (NET-356). Equals `pawn_zobrist_key(position) as u64` at all times -
    /// built from the low halves of the same tables so correction-history
    /// indices are bit-identical to the old per-node recomputation.
    pub pawn_key: u64,
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
