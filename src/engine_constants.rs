use crate::types::Score;

pub const DEBUG: bool = false;

pub const PAWN_VALUE: Score = 100;
pub const KNIGHT_VALUE: Score = 350;
pub const BISHOP_VALUE: Score = 350;
pub const ROOK_VALUE: Score = 550;
pub const QUEEN_VALUE: Score = 900;

pub const NUM_KILLER_MOVES: usize = 2;

pub const NULL_MOVE_REDUCE_DEPTH: u8 = 2;

pub const MAX_DEPTH: u8 = 250;

pub const MAX_QUIESCE_DEPTH: u8 = 10;
pub const IID_REDUCE_DEPTH: u8 = 3;
pub const NUM_HASH_ENTRIES: u64 = 1024 * 1024 * 4;

pub const TICKER_MILLIS: u16 = 500;

pub const DEPTH_REMAINING_FOR_NULLMOVE_RD_INCREASE: u8 = 6;

pub const LMR_LEGALMOVES_BEFORE_ATTEMPT: u8 = 3;
pub const LMR_MIN_DEPTH: u8 = 3;

pub const SCOUT_MINIMUM_DISTANCE_FROM_LEAF: u8 = 2;