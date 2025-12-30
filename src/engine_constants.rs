use crate::types::{HistoryScore, Score, ScorePair};

pub const DEBUG: bool = false;

pub const PAWN_VALUE_PAIR: ScorePair = (100, 200);
pub const KNIGHT_VALUE_PAIR: ScorePair = (620, 680);
pub const BISHOP_VALUE_PAIR: ScorePair = (650, 725);
pub const ROOK_VALUE_PAIR: ScorePair = (1000, 1100);
pub const QUEEN_VALUE_PAIR: ScorePair = (2000, 2300);

pub const PAWN_VALUE_AVERAGE: Score = (PAWN_VALUE_PAIR.0 + PAWN_VALUE_PAIR.1) / 2;
pub const KNIGHT_VALUE_AVERAGE: Score = (KNIGHT_VALUE_PAIR.0 + KNIGHT_VALUE_PAIR.1) / 2;
pub const BISHOP_VALUE_AVERAGE: Score = (BISHOP_VALUE_PAIR.0 + BISHOP_VALUE_PAIR.1) / 2;
pub const ROOK_VALUE_AVERAGE: Score = (ROOK_VALUE_PAIR.0 + ROOK_VALUE_PAIR.1) / 2;
pub const QUEEN_VALUE_AVERAGE: Score = (QUEEN_VALUE_PAIR.0 + QUEEN_VALUE_PAIR.1) / 2;

pub const STARTING_MATERIAL: Score =
    PAWN_VALUE_AVERAGE * 16 + KNIGHT_VALUE_AVERAGE * 4 + BISHOP_VALUE_AVERAGE * 4 + ROOK_VALUE_AVERAGE * 4 + QUEEN_VALUE_AVERAGE * 2;

pub const HISTORY_MAX_SCORE: Score = (HistoryScore::MAX / 2) as Score;

pub const UCI_MILLIS_REDUCTION: u128 = 5;

pub const BETA_PRUNE_MARGIN_PER_DEPTH: Score = 200;
pub const BETA_PRUNE_MAX_DEPTH: u8 = 3;

pub const NUM_KILLER_MOVES: usize = 2;

pub const NULL_MOVE_MIN_DEPTH: u8 = 4;
pub const NULL_MOVE_REDUCE_DEPTH_BASE: u8 = 3;

pub const MAX_DEPTH: u8 = 250;

pub const MAX_QUIESCE_DEPTH: u8 = 100;

pub const HASH_ENTRY_BYTES: u64 = 22;
pub const HASH_SIZE_MB: u64 = 128;
pub const NUM_HASH_ENTRIES: u64 = (1024 * 1024 * HASH_SIZE_MB) / HASH_ENTRY_BYTES;
pub const ALPHA_PRUNE_MARGINS: [Score; 8] = [128, 192, 256, 320, 384, 448, 512, 576];

pub const TICKER_MILLIS: u16 = 500;

pub const IID_MIN_DEPTH: u8 = 3;
pub const IID_SEARCH_DEPTH: u8 = 2;
pub const IID_REDUCE_DEPTH: u8 = 2;

pub const LMR_LEGAL_MOVES_BEFORE_ATTEMPT: u8 = 4;
pub const LMR_MIN_DEPTH: u8 = 3;
pub const LMR_REDUCTION: u8 = 2;

pub const SCOUT_MINIMUM_DISTANCE_FROM_LEAF: u8 = 2;

pub const VALUE_BISHOP_MOBILITY: [Score; 14] = [-15, -10, -6, -2, 1, 3, 5, 6, 8, 9, 10, 11, 12, 12];
pub const VALUE_BISHOP_PAIR_FEWER_PAWNS_BONUS: Score = 3;
pub const VALUE_BISHOP_PAIR: Score = 10;
pub const VALUE_GUARDED_PASSED_PAWN: Score = 30;
pub const VALUE_KNIGHT_OUTPOST: Score = 7;
pub const VALUE_PASSED_PAWN_BONUS: [Score; 6] = [24, 26, 30, 36, 44, 56];
pub const VALUE_BACKWARD_PAWN_PENALTY: Score = 15;
pub const DOUBLED_PAWN_PENALTY: Score = 15;
pub const ISOLATED_PAWN_PENALTY: Score = 10;
pub const VALUE_ROOKS_ON_SAME_FILE: Score = 8;
pub const ROOKS_ON_SEVENTH_RANK_BONUS: Score = 20;
pub const KING_THREAT_BONUS_KNIGHT: Score = 16;
pub const KING_THREAT_BONUS_QUEEN: Score = 12;
pub const KING_THREAT_BONUS_BISHOP: Score = 12;
pub const KING_THREAT_BONUS_ROOK: Score = 10;

pub const PAWN_ADJUST_MAX_MATERIAL: Score = (QUEEN_VALUE_AVERAGE + ROOK_VALUE_AVERAGE) as Score;
pub const VALUE_KING_CANNOT_CATCH_PAWN: Score = 500;
pub const VALUE_KING_CANNOT_CATCH_PAWN_PIECES_REMAIN: Score = 500;

pub const VALUE_KING_DISTANCE_PASSED_PAWN_MULTIPLIER: Score = 4;

pub const KNIGHT_FORK_THREAT_SCORE: Score = 5;

pub const ROOK_OPEN_FILE_BONUS: Score = 25;
pub const ROOK_SEMI_OPEN_FILE_BONUS: Score = 12;

// Rook mobility bonus based on number of squares available (0-14)
pub const VALUE_ROOK_MOBILITY: [Score; 15] = [-10, -6, -3, 0, 2, 4, 5, 6, 7, 8, 9, 10, 11, 12, 12];

// Queen mobility bonus based on number of squares available (0-27)
pub const VALUE_QUEEN_MOBILITY: [Score; 28] = [
    -12, -8, -5, -2, 0, 1, 2, 3, 4, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 10, 11, 11, 11, 12, 12, 12, 12,
];

// Bonus for connected rooks (rooks that can see each other on rank or file)
pub const VALUE_CONNECTED_ROOKS: Score = 15;
