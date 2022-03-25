use crate::types::Score;

pub const DEBUG: bool = false;

pub const PAWN_VALUE: Score = 100;
pub const KNIGHT_VALUE: Score = 390;
pub const BISHOP_VALUE: Score = 390;
pub const ROOK_VALUE: Score = 595;
pub const QUEEN_VALUE: Score = 1175;

pub const UCI_MILLIS_REDUCTION: u128 = 5;

pub const NUM_KILLER_MOVES: usize = 2;

pub const NULL_MOVE_REDUCE_DEPTH: u8 = 2;

pub const PAWN_ADJUST_MAX_MATERIAL: Score = (QUEEN_VALUE + ROOK_VALUE) as Score;
pub const VALUE_KING_CANNOT_CATCH_PAWN: Score = 500;
pub const VALUE_KING_DISTANCE_PASSED_PAWN_MULTIPLIER: Score = 4;

pub const MAX_DEPTH: u8 = 250;

pub const MAX_QUIESCE_DEPTH: u8 = 10;
pub const IID_REDUCE_DEPTH: u8 = 3;

pub const HASH_ENTRY_BYTES: u64 = 22;
pub const HASH_TABLE_MB: u64 = 128;
pub const NUM_HASH_ENTRIES: u64 = ((1024 * 1024 * HASH_TABLE_MB) / HASH_ENTRY_BYTES) as u64;

pub const TICKER_MILLIS: u16 = 500;

pub const DEPTH_REMAINING_FOR_RD_INCREASE: u8 = 6;

pub const LMR_LEGALMOVES_BEFORE_ATTEMPT: u8 = 4;
pub const LMR_MIN_DEPTH: u8 = 4;
pub const LMR_REDUCTION: u8 = 3;

pub const SCOUT_MINIMUM_DISTANCE_FROM_LEAF: u8 = 2;