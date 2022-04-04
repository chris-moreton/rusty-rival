use crate::types::{HistoryScore, Score};

pub const DEBUG: bool = false;

pub const PAWN_VALUE: Score = 100;
pub const KNIGHT_VALUE: Score = 390;
pub const BISHOP_VALUE: Score = 390;
pub const ROOK_VALUE: Score = 595;
pub const QUEEN_VALUE: Score = 1175;

pub const HISTORY_MAX_SCORE: Score = (HistoryScore::MAX / 2) as Score;

pub const UCI_MILLIS_REDUCTION: u128 = 5;

pub const ASPIRATION_RADIUS: Score = 25;
pub const BETA_PRUNE_MARGIN_PER_DEPTH: Score = 300;
pub const BETA_PRUNE_MAX_DEPTH: u8 = 3;

pub const NUM_KILLER_MOVES: usize = 2;

pub const NULL_MOVE_REDUCE_DEPTH: u8 = 2;

pub const PAWN_ADJUST_MAX_MATERIAL: Score = (QUEEN_VALUE + ROOK_VALUE) as Score;
pub const VALUE_KING_CANNOT_CATCH_PAWN: Score = 500;
pub const VALUE_KING_DISTANCE_PASSED_PAWN_MULTIPLIER: Score = 4;

pub const MAX_DEPTH: u8 = 250;

pub const MAX_QUIESCE_DEPTH: u8 = 100;

pub const HASH_ENTRY_BYTES: u64 = 22;
pub const HASH_ENTRY_MB: u64 = 128;
pub const NUM_HASH_ENTRIES: u64 = ((1024 * 1024 * HASH_ENTRY_MB) / HASH_ENTRY_BYTES) as u64;
pub const ALPHA_PRUNE_MARGINS: [Score; 8] = [128, 192, 256, 320, 384, 448, 512, 576];

pub const TICKER_MILLIS: u16 = 500;

pub const DEPTH_REMAINING_FOR_RD_INCREASE: u8 = 6;

pub const IID_MIN_DEPTH: u8 = 5;
pub const IID_SEARCH_DEPTH: u8 = 2;
pub const IID_REDUCE_DEPTH: u8 = 1;

pub const LMR_LEGAL_MOVES_BEFORE_ATTEMPT: u8 = 4;
pub const LMR_MIN_DEPTH: u8 = 3;
pub const LMR_REDUCTION: u8 = 2;

pub const SCOUT_MINIMUM_DISTANCE_FROM_LEAF: u8 = 2;
