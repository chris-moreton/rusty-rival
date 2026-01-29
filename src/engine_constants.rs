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
pub const BETA_PRUNE_MAX_DEPTH: u8 = 4;

pub const NUM_KILLER_MOVES: usize = 2;

pub const NULL_MOVE_MIN_DEPTH: u8 = 4;
pub const NULL_MOVE_REDUCE_DEPTH_BASE: u8 = 2;

// Threat extension: if null move search returns a score this much below alpha,
// the opponent has a significant threat that warrants deeper search
// Using 400 to be more selective - only trigger for major threats (like losing a piece)
pub const THREAT_EXTENSION_MARGIN: Score = 400;

// SEE pruning: skip bad captures at low depths
// At depth N, skip captures with SEE < -SEE_PRUNE_MARGIN * N
// This prunes obviously losing captures like QxP when the pawn is defended
pub const SEE_PRUNE_MAX_DEPTH: u8 = 3;
pub const SEE_PRUNE_MARGIN: Score = 16;

// Probcut: at high depth, do a shallow search with raised beta
// If it fails high, the position is probably winning and can be cut
// Only searches captures with SEE >= 0 to verify tactical soundness
pub const PROBCUT_MIN_DEPTH: u8 = 5;
pub const PROBCUT_MARGIN: Score = 100;
pub const PROBCUT_DEPTH_REDUCTION: u8 = 4;

// Multi-cut: at high depth, if multiple moves fail high at shallow depth,
// assume the position is good and return a beta cutoff
pub const MULTICUT_MIN_DEPTH: u8 = 8;
pub const MULTICUT_DEPTH_REDUCTION: u8 = 4;
pub const MULTICUT_MOVES_TO_TRY: u8 = 6;
pub const MULTICUT_REQUIRED_CUTOFFS: u8 = 3;

// Late Move Pruning (LMP): skip late quiet moves at low depths
// After searching N moves at depth D, skip remaining quiet moves entirely
// Index by depth: [depth 0, depth 1, depth 2, depth 3]
// Conservative thresholds to avoid missing important moves
pub const LMP_MAX_DEPTH: u8 = 3;
pub const LMP_MOVE_THRESHOLDS: [u8; 4] = [0, 5, 11, 8];

// Fractional extensions: use fixed-point arithmetic with 4 units = 1 ply
// This allows multiple factors to combine (e.g., check + pawn push)
pub const FRAC_EXT_CHECK: u8 = 4; // 1.0 ply for check
pub const FRAC_EXT_PAWN_PUSH: u8 = 2; // 0.5 ply for pawn push to 7th
pub const FRAC_EXT_UNIT: u8 = 4; // Units per ply

pub const MAX_DEPTH: u8 = 250;

pub const MAX_QUIESCE_DEPTH: u8 = 100;

pub const HASH_ENTRY_BYTES: u64 = 22;
pub const HASH_SIZE_MB: u64 = 128;
pub const NUM_HASH_ENTRIES: u64 = (1024 * 1024 * HASH_SIZE_MB) / HASH_ENTRY_BYTES;

// Pawn hash table: 16K entries, each entry is 20 bytes (16 byte key + 4 byte score)
pub const NUM_PAWN_HASH_ENTRIES: usize = 16384;
// SPSA tuned: base=130, per_depth=63
pub const ALPHA_PRUNE_MARGINS: [Score; 8] = [130, 193, 256, 319, 382, 445, 508, 571];

pub const TICKER_MILLIS: u16 = 500;

pub const IID_MIN_DEPTH: u8 = 3;
pub const IID_SEARCH_DEPTH: u8 = 2;
pub const IID_REDUCE_DEPTH: u8 = 2;

pub const LMR_LEGAL_MOVES_BEFORE_ATTEMPT: u8 = 4;
pub const LMR_MIN_DEPTH: u8 = 3;

// LMR reduction table: indexed by [depth][move_count]
// Formula: floor(0.75 + ln(depth) * ln(move_count) / 2.5)
// More conservative than Stockfish's formula to avoid over-pruning
// Precomputed for depths 0-63 and move counts 0-63
pub const LMR_TABLE: [[u8; 64]; 64] = generate_lmr_table();

const fn generate_lmr_table() -> [[u8; 64]; 64] {
    // Precomputed ln values * 1000 for integers 1-63 (ln(0) undefined, use 0)
    // ln(1)=0, ln(2)=693, ln(3)=1099, ln(4)=1386, etc.
    const LN_TABLE: [u32; 64] = [
        0, 0, 693, 1099, 1386, 1609, 1792, 1946, 2079, 2197, 2303, 2398, 2485, 2565, 2639, 2708, 2773, 2833, 2890, 2944, 2996, 3045, 3091,
        3135, 3178, 3219, 3258, 3296, 3332, 3367, 3401, 3434, 3466, 3497, 3526, 3555, 3584, 3611, 3638, 3664, 3689, 3714, 3738, 3761, 3784,
        3807, 3829, 3850, 3871, 3892, 3912, 3932, 3951, 3970, 3989, 4007, 4025, 4043, 4060, 4078, 4094, 4111, 4127, 4143,
    ];

    let mut table = [[0u8; 64]; 64];
    let mut depth = 0usize;
    while depth < 64 {
        let mut move_count = 0usize;
        while move_count < 64 {
            if depth >= 3 && move_count >= 4 {
                // reduction = 0.75 + ln(depth) * ln(move_count) / 2.5
                // Using integer math: (750 + ln_d * ln_m / 2500) / 1000
                let ln_d = LN_TABLE[depth];
                let ln_m = LN_TABLE[move_count];
                let reduction = (750 + (ln_d * ln_m) / 2500) / 1000;
                table[depth][move_count] = reduction as u8;
            }
            move_count += 1;
        }
        depth += 1;
    }
    table
}

/// Get LMR reduction for given depth and move count
#[inline(always)]
pub fn lmr_reduction(depth: u8, move_count: u8) -> u8 {
    let d = (depth as usize).min(63);
    let m = (move_count as usize).min(63);
    LMR_TABLE[d][m]
}

pub const SCOUT_MINIMUM_DISTANCE_FROM_LEAF: u8 = 2;

pub const VALUE_BISHOP_MOBILITY: [Score; 14] = [-15, -10, -6, -2, 1, 3, 5, 6, 8, 9, 10, 11, 12, 12];
pub const VALUE_BISHOP_PAIR_FEWER_PAWNS_BONUS: Score = 3;
pub const VALUE_BISHOP_PAIR: Score = 10;
pub const VALUE_GUARDED_PASSED_PAWN: Score = 30;
// Rook behind passed pawn (Tarrasch rule): rooks are strongest supporting passed pawns from behind
// As the pawn advances, the rook's scope increases; and it protects the pawn's advance
pub const VALUE_ROOK_BEHIND_PASSED_PAWN: Score = 20;
pub const VALUE_KNIGHT_OUTPOST: Score = 7;
pub const VALUE_PASSED_PAWN_BONUS: [Score; 6] = [24, 26, 30, 36, 44, 56];
// Bonus for connected passed pawns (two passed pawns on adjacent files)
// They're very dangerous as they support each other toward promotion
pub const VALUE_CONNECTED_PASSED_PAWNS: [Score; 6] = [12, 18, 28, 42, 60, 80];
pub const VALUE_BACKWARD_PAWN_PENALTY: Score = 15;
pub const DOUBLED_PAWN_PENALTY: Score = 15;
pub const ISOLATED_PAWN_PENALTY: Score = 10;

// Queenside pawn majority: bonus per extra pawn on queenside (a-d files)
// Having more pawns on the queenside is strategically valuable because:
// 1. The king typically castles kingside, so queenside pawns are "distant"
// 2. A queenside majority can create a passed pawn while the king defends kingside
// Bonus scales with material - more valuable in endgames
pub const VALUE_QUEENSIDE_PAWN_MAJORITY: Score = 10;
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

// King supporting own passed pawns in endgame
// Bonus for friendly king being close to its own passed pawns
// Formula: (7 - distance) * rank_index * multiplier / 4
// More valuable for advanced pawns (higher rank_index)
pub const VALUE_KING_SUPPORTS_PASSED_PAWN: Score = 3;

pub const KNIGHT_FORK_THREAT_SCORE: Score = 5;

pub const ROOK_OPEN_FILE_BONUS: Score = 25;
pub const ROOK_SEMI_OPEN_FILE_BONUS: Score = 12;

// Queen mobility bonus based on number of squares available (0-27)
pub const VALUE_QUEEN_MOBILITY: [Score; 28] = [
    -12, -8, -5, -2, 0, 1, 2, 3, 4, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 10, 11, 11, 11, 12, 12, 12, 12,
];

// King centralization bonus for endgames - extra bonus beyond PST when material is low
// This encourages the king to actively participate in the endgame
// Indexed by king square (same layout as PST: h1=0, a8=63)
// Values increased to make king activity more impactful
pub const VALUE_KING_ENDGAME_CENTRALIZATION: [Score; 64] = [
    0, 4, 8, 12, 12, 8, 4, 0, // rank 1
    4, 8, 16, 24, 24, 16, 8, 4, // rank 2
    8, 16, 32, 40, 40, 32, 16, 8, // rank 3
    12, 24, 40, 48, 48, 40, 24, 12, // rank 4
    12, 24, 40, 48, 48, 40, 24, 12, // rank 5
    8, 16, 32, 40, 40, 32, 16, 8, // rank 6
    4, 8, 16, 24, 24, 16, 8, 4, // rank 7
    0, 4, 8, 12, 12, 8, 4, 0, // rank 8
];

// Material threshold below which we apply extra king centralization bonus
// This triggers when queens are off or material is significantly reduced
// Roughly no queens + at most one rook per side = ~2100, or queen + minor = ~2800
pub const ENDGAME_MATERIAL_THRESHOLD: Score = QUEEN_VALUE_AVERAGE + KNIGHT_VALUE_AVERAGE;

// King activity: bonus for king attacking enemy pieces in endgames
// Attacking minor pieces (bishops/knights) is valuable as it restricts them
pub const VALUE_KING_ATTACKS_MINOR: Score = 20;
// Attacking rooks is also valuable
pub const VALUE_KING_ATTACKS_ROOK: Score = 15;

// King mobility: bonus per safe square the king can move to in endgames
// A king with more safe squares is more active and flexible
// This helps identify when moving the king improves its activity
pub const VALUE_KING_MOBILITY: Score = 6;

// Bishop vs Knight imbalance
// Bishops are stronger in open positions (fewer pawns), knights in closed positions (more pawns)
// With 16 pawns (max), knights get bonus; with 0 pawns, bishops get bonus
// The bonus is per minor piece imbalance (e.g., having 2 bishops vs 2 knights)
// At 8 pawns (average), imbalance is neutral
pub const BISHOP_KNIGHT_IMBALANCE_BONUS: Score = 15; // Max bonus per imbalance point

// Trapped piece penalties
// Pieces that are trapped (very limited mobility) lose significant value
pub const TRAPPED_BISHOP_PENALTY: Score = 100; // Bishop trapped on a7/h7/a2/h2 by pawns
pub const TRAPPED_ROOK_PENALTY: Score = 50; // Rook trapped in corner by own king

// Space evaluation: bonus per safe square controlled in opponent's territory
// More important in closed positions with many pawns
pub const SPACE_BONUS_PER_SQUARE: Score = 2;

// Blocked passed pawn: penalty when enemy king guards the promotion square
// A passed pawn that can never promote should lose most of its bonus
// This should be larger than the passed pawn bonus for that rank
pub const BLOCKED_PASSED_PAWN_PENALTY: Score = 80;

// Knight blockade: penalty when enemy knight controls the promotion square
// Similar to king blockade but slightly smaller since knight can be driven away
// Applied per blocked passed pawn
pub const KNIGHT_BLOCKADE_PENALTY: Score = 60;

// General knight activity: bonus for knights attacking enemy pawns
// This applies in all positions, not just Q vs N+pawns
pub const KNIGHT_ATTACKS_PAWN_GENERAL_BONUS: Score = 12;
