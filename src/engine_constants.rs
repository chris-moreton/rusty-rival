use crate::types::Score;

pub const PAWN_VALUE: Score = 100;
pub const KNIGHT_VALUE: Score = 350;
pub const BISHOP_VALUE: Score = 350;
pub const ROOK_VALUE: Score = 550;
pub const QUEEN_VALUE: Score = 900;

pub const NUM_KILLER_MOVES: usize = 2;

pub const NULL_MOVE_REDUCE_DEPTH: u8 = 2;

pub const MAX_DEPTH: u8 = 250;

pub const MAX_QUIESCE_DEPTH: u8 = 10;
pub const MAX_HASH_ENTRIES: u64 = 1024 * 1024 * 1024;

pub const DOUBLED_PAWN_PENALTY: Score = 25;
pub const PAWN_TRADE_BONUS_MAX: Score = 600;