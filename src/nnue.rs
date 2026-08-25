//! NNUE (Efficiently Updatable Neural Network) evaluation.
//!
//! Architecture: (768 → 512)x2 → 8 with SCReLU activation
//! - 768 inputs: Chess768 piece-square features (64 squares × 6 pieces × 2 colors)
//! - 512 hidden neurons, dual perspective (side-to-move + not-side-to-move)
//! - 8 material-count output buckets
//!
//! The network weights are quantized to i16 and embedded in the binary.
//! Accumulators are incrementally updated using bitboard diffs.

use crate::types::{Mover, Pieces, Position, Score};
use std::fmt;

// Network dimensions
pub const INPUT_SIZE: usize = 768;
/// Hidden layer width. **Must match the embedded net** — the loader's format
/// detection distinguishes single-bucket from bucketed, *not* 256 from 512, so
/// a 512-wide net read with this set to 256 loads without error and evaluates
/// garbage. Change this and `EMBEDDED_NET` together (NET-324).
pub const HIDDEN_SIZE: usize = 512;

/// Number of output buckets, selected by material count (NET-321).
///
/// A single-bucket net is loaded as 8 identical buckets, so the inference path
/// is uniform and a pre-bucket net evaluates bit-identically to before.
pub const NUM_OUTPUT_BUCKETS: usize = 8;

/// Bucket divisor. Must match bullet's `MaterialCount<N>`, which uses
/// `32usize.div_ceil(N)` — verified against pinned bullet rev 7bc395f3 in
/// `crates/bullet_lib/src/game/outputs.rs`.
const OUTPUT_BUCKET_DIVISOR: u32 = 4; // 32.div_ceil(8)

/// Select the output bucket for a position, reproducing bullet's formula:
/// `(occ.count_ones() - 2) / 32.div_ceil(N)`.
///
/// This MUST match the trainer exactly. If it doesn't, the engine reads a
/// different bucket's weights than training wrote, and the net evaluates
/// plausibly but wrongly — it will load fine and produce sane-looking numbers.
#[inline(always)]
pub fn output_bucket(piece_count: u32) -> usize {
    // saturating_sub guards a malformed position with fewer than two pieces;
    // min() guards a 33+ piece position. Neither occurs in legal play, but a
    // panic or out-of-bounds read in the hottest path is not worth the risk.
    ((piece_count.saturating_sub(2) / OUTPUT_BUCKET_DIVISOR) as usize).min(NUM_OUTPUT_BUCKETS - 1)
}

// Quantization constants (must match training config)
const QA: i32 = 255; // L0 weight/bias and accumulator scale
const QB: i32 = 64; // L1 weight scale
const EVAL_SCALE: i32 = 400; // Converts network output to centipawns

/// Embedded network weights (trained with bullet, quantised.bin format).
///
/// `rival-512x2-ob8-corrected-net1095.bin` — 8 output buckets by material
/// count, 600 superbatches over the full Stockfish depth-9 corpus. Every shard
/// was validated before mutation and its white-relative WDL field corrected;
/// measured score/result coherence moved from roughly 4.5% to 95.5%.
///
/// ⚠ **Every other net in `nets/` is incompatible with this build.** They were
/// all trained against the inverted WDL label that the transposed L1 indexing
/// in `evaluate()` used to cancel out. Now that the indexing is correct, those
/// nets evaluate *backwards*: swapping this `include_bytes!` to any of them
/// yields an engine that plays to lose, and it will pass every structural check
/// on the way — `check_net`'s sign checks are the only thing that catches it.
/// They are kept only as a record of the experiments that produced them.
///
/// To train a replacement, the corpus must carry a **white-relative** score and
/// a **white-relative** result. `bulletformat` flips both for black to move; it
/// does not want side-to-move-relative input, despite what the field order
/// suggests. The corpus in S3 is not stored that way and needs its result field
/// flipped first — see NET-400 for the transform and the verification.
const EMBEDDED_NET: &[u8] = include_bytes!("../nets/rival-512x2-ob8-corrected-net1095.bin");

// =============================================================================
// Network
// =============================================================================

/// Quantised NNUE network weights, loaded once at startup.
pub struct NnueNetwork {
    /// L0 weights indexed as [feature][neuron] for cache-friendly incremental updates.
    /// Each feature's `HIDDEN_SIZE` weights are contiguous in memory.
    l0_weights: Box<[[i16; HIDDEN_SIZE]; INPUT_SIZE]>,
    /// L0 biases: initial accumulator values before any features are added.
    l0_biases: [i16; HIDDEN_SIZE],
    /// L1 weights, one `2 * HIDDEN_SIZE`-wide row per output bucket (bucket-major).
    ///
    /// Within a row: first `HIDDEN_SIZE` are side-to-move, the second half are
    /// not-side-to-move, matching the trainer's `stm.concat(ntm)` layout.
    ///
    /// ## Two layout facts that will bite a retrain if ignored
    ///
    /// 1. **Bucket-major is what bullet writes** *only because* the trainer's
    ///    `l1w` save entry has `.transpose()`. Derived from pinned rev 7bc395f3:
    ///    `new_affine` builds `Shape::new(out, in)` = (8, 1024), and
    ///    `transpose_impl` writes `new_buf[cols*i + j] = weights[rows*j + i]`,
    ///    i.e. `[bucket][1024]`. Drop the `.transpose()` and this silently
    ///    becomes `[1024][bucket]`.
    /// 2. **The perspective halves are STM-first**, matching the trainer's
    ///    `stm.concat(ntm)`: indices `0..HIDDEN_SIZE` are STM and the second
    ///    half are NTM. `nnue_eval_signs_are_correct` guards this convention
    ///    and fails loudly if a retrained net has its halves transposed.
    l1_weights: Box<[[i16; 2 * HIDDEN_SIZE]; NUM_OUTPUT_BUCKETS]>,
    /// L1 bias per bucket (scale QA*QB).
    l1_biases: [i16; NUM_OUTPUT_BUCKETS],
}

impl fmt::Debug for NnueNetwork {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NnueNetwork({}x{})", INPUT_SIZE, HIDDEN_SIZE)
    }
}

impl NnueNetwork {
    /// Load the embedded network.
    pub fn embedded() -> Self {
        Self::from_bytes(EMBEDDED_NET)
    }

    /// Load network from raw quantised.bin bytes.
    ///
    /// Accepts **both output formats at the compiled `HIDDEN_SIZE`** and
    /// normalises to the bucketed representation:
    ///
    /// * single-bucket: `l0w[768][H] + l0b[H] + l1w[2H] + l1b[1]`
    /// * 8-bucket:      `l0w[768][H] + l0b[H] + l1w[8][2H] + l1b[8]`
    ///
    /// A single-bucket net at the current width is replicated into all 8
    /// buckets. Width is a compile-time property: a 256-wide file is
    /// deliberately rejected by this 512-wide build rather than ambiguously
    /// parsed as another format.
    ///
    /// All values are little-endian i16. l0_weights is stored as [768][H]
    /// during loading for cache-friendly feature-indexed access.
    ///
    /// Note the format is detected by size rather than matched exactly: the
    /// shipped net carries 62 trailing bytes beyond what is read (bullet
    /// padding), so an equality check would be brittle.
    pub fn from_bytes(data: &[u8]) -> Self {
        // Reject a net whose width doesn't match HIDDEN_SIZE before reading a
        // single byte. The format detection below distinguishes single-bucket
        // from bucketed, but NOT 256-wide from 512-wide: a mismatched net either
        // runs off the end of the buffer (panic with an opaque index message) or,
        // if the file happens to be larger, is read as plausible garbage and
        // silently evaluates nonsense. Both are far worse than failing here.
        let single_i16 = INPUT_SIZE * HIDDEN_SIZE + HIDDEN_SIZE + 2 * HIDDEN_SIZE + 1;
        assert!(
            data.len() >= single_i16 * 2,
            "NNUE net is too small for HIDDEN_SIZE={}: got {} bytes, need at least {}. \
             The embedded net and HIDDEN_SIZE must be changed together.",
            HIDDEN_SIZE,
            data.len(),
            single_i16 * 2
        );

        let mut offset = 0;

        // L0 weights: stored as [feature][neuron] = 768 features × 512 neurons
        let mut l0_weights = Box::new([[0i16; HIDDEN_SIZE]; INPUT_SIZE]);
        for feature in 0..INPUT_SIZE {
            for neuron in 0..HIDDEN_SIZE {
                l0_weights[feature][neuron] = read_i16(data, &mut offset);
            }
        }

        // L0 biases
        let mut l0_biases = [0i16; HIDDEN_SIZE];
        for item in l0_biases.iter_mut() {
            *item = read_i16(data, &mut offset);
        }

        // Decide the format from what is left after the L0 section.
        let remaining_i16 = (data.len() - offset) / 2;
        let bucketed_i16 = NUM_OUTPUT_BUCKETS * (2 * HIDDEN_SIZE) + NUM_OUTPUT_BUCKETS;
        let is_bucketed = remaining_i16 >= bucketed_i16;

        let mut l1_weights = Box::new([[0i16; 2 * HIDDEN_SIZE]; NUM_OUTPUT_BUCKETS]);
        let mut l1_biases = [0i16; NUM_OUTPUT_BUCKETS];

        if is_bucketed {
            // Bucket-major: each bucket's 1024 weights are contiguous.
            for bucket in l1_weights.iter_mut() {
                for item in bucket.iter_mut() {
                    *item = read_i16(data, &mut offset);
                }
            }
            for item in l1_biases.iter_mut() {
                *item = read_i16(data, &mut offset);
            }
        } else {
            // Legacy single-bucket net: read one row and one bias, then
            // replicate so every bucket resolves to the same weights.
            let mut row = [0i16; 2 * HIDDEN_SIZE];
            for item in row.iter_mut() {
                *item = read_i16(data, &mut offset);
            }
            let bias = read_i16(data, &mut offset);
            for bucket in l1_weights.iter_mut() {
                *bucket = row;
            }
            l1_biases = [bias; NUM_OUTPUT_BUCKETS];
        }

        let net = NnueNetwork {
            l0_weights,
            l0_biases,
            l1_weights,
            l1_biases,
        };
        net.assert_no_output_overflow();
        net
    }

    /// Guard the i32 accumulation in `evaluate()` at LOAD time (NET-374).
    ///
    /// `output` sums 2*HIDDEN_SIZE terms of `(s*s/QA) * w` where `s*s/QA <= QA`,
    /// so the worst case is `2 * HIDDEN_SIZE * QA * max|w|`. With the shipped
    /// net that is nowhere near i32::MAX, but nothing checked it: a retrained
    /// net with larger L1 weights would silently wrap (or panic in debug) and
    /// produce garbage evaluations with no other symptom. Scaling is widened
    /// to i64, and the second bound ensures the final result still fits Score.
    ///
    /// Done at load rather than in the loop because `evaluate()` is the hottest
    /// path in the engine and this costs nothing there.
    fn assert_no_output_overflow(&self) {
        let max_w = self
            .l1_weights
            .iter()
            .flat_map(|bucket| bucket.iter())
            .map(|w| (*w as i64).abs())
            .max()
            .unwrap_or(0);
        let max_bias = self.l1_biases.iter().map(|b| (*b as i64).abs()).max().unwrap_or(0);
        let worst = 2 * HIDDEN_SIZE as i64 * QA as i64 * max_w + max_bias;
        assert!(
            worst <= i32::MAX as i64,
            "NNUE L1 weights can overflow the i32 accumulator in evaluate(): worst case {} > {}. \
             Widen the accumulator to i64 or requantise the net.",
            worst,
            i32::MAX
        );
        assert!(
            worst.saturating_mul(EVAL_SCALE as i64) <= i32::MAX as i64 * QA as i64 * QB as i64,
            "NNUE scaled output can exceed Score: worst case {} with EVAL_SCALE {}.",
            worst,
            EVAL_SCALE
        );
    }

    /// Evaluate the position using the current accumulator.
    ///
    /// Returns a score in centipawns from the side-to-move's perspective.
    /// Uses SCReLU activation: clamp(x, 0, QA)² with quantized arithmetic.
    ///
    /// `piece_count` is the total number of pieces on the board and selects the
    /// output bucket. It must be the same count bullet used when training — see
    /// [`output_bucket`]. With a single-bucket net every bucket holds identical
    /// weights, so the value is irrelevant and the result is unchanged.
    ///
    /// The per-element `/QA` looks like it would be slow, but on aarch64 LLVM
    /// already lowers this whole loop to 8-wide i16 NEON with the division as a
    /// multiply-high plus shift, 4x unrolled. Verified by disassembly (NET-320);
    /// do not "optimise" it into explicit SIMD without measuring first.
    #[inline]
    pub fn evaluate(&self, acc: &Accumulator, stm: Mover, piece_count: u32) -> Score {
        let (stm_acc, ntm_acc) = if stm == 0 {
            (&acc.white, &acc.black)
        } else {
            (&acc.black, &acc.white)
        };

        let bucket = output_bucket(piece_count);
        let l1_weights = &self.l1_weights[bucket];

        let mut output: i32 = 0;

        for i in 0..HIDDEN_SIZE {
            let s = (stm_acc[i] as i32).clamp(0, QA);
            let n = (ntm_acc[i] as i32).clamp(0, QA);
            // SCReLU: squared clipped ReLU, divided by QA to prevent overflow
            //
            // L1 layout: STM first [0..512], NTM second [512..1024]. These two
            // indices were transposed until v1.0.53 (NET-400). Swapping the two
            // perspectives of a dual-perspective net *negates* its output, and
            // does so symmetrically - a position and its colour mirror still
            // agreed - so nothing in the test suite caught it. What masked it in
            // play was that the training corpus had its WDL label inverted too,
            // and the two errors cancelled.
            //
            // The cancellation was not free: bullet blends the target 75% result
            // / 25% score, and only the result half was inverted, so a quarter of
            // every training target fought the other three. Correcting both ends
            // together was worth +198 +/-7 Elo over 3000 games.
            output += s * s / QA * l1_weights[i] as i32;
            output += n * n / QA * l1_weights[HIDDEN_SIZE + i] as i32;
        }

        // Add bias (already in scale QA*QB) and convert to centipawns
        output += self.l1_biases[bucket] as i32;
        (output as i64 * EVAL_SCALE as i64 / (QA * QB) as i64) as Score
    }
}

#[inline(always)]
fn read_i16(data: &[u8], offset: &mut usize) -> i16 {
    let val = i16::from_le_bytes([data[*offset], data[*offset + 1]]);
    *offset += 2;
    val
}

// =============================================================================
// Accumulator
// =============================================================================

/// Hidden layer values for both perspectives. Incrementally updated as pieces move.
#[derive(Clone, Debug)]
pub struct Accumulator {
    pub white: [i16; HIDDEN_SIZE],
    pub black: [i16; HIDDEN_SIZE],
}

impl Default for Accumulator {
    fn default() -> Self {
        Self::new()
    }
}

impl Accumulator {
    /// Create a zeroed accumulator (must be initialized before use).
    pub fn new() -> Self {
        Accumulator {
            white: [0; HIDDEN_SIZE],
            black: [0; HIDDEN_SIZE],
        }
    }

    /// Compute the accumulator from scratch for the given position.
    ///
    /// bullet's Chess768 normalizes positions so the side-to-move is always "white".
    /// Compute the accumulator from scratch for the given position.
    /// White accumulator = white perspective, black accumulator = black perspective.
    /// At eval time, STM/NTM selection picks the right perspective.
    pub fn compute(&mut self, net: &NnueNetwork, position: &Position) {
        self.white = net.l0_biases;
        self.black = net.l0_biases;

        for color in 0..2usize {
            let pieces = &position.pieces[color];

            for &(bb, piece_type) in &bitboard_piece_types(pieces) {
                let mut bitboard = bb;
                while bitboard != 0 {
                    let sq = bitboard.trailing_zeros() as i8;
                    bitboard &= bitboard - 1;
                    add_feature(&mut self.white, &mut self.black, net, color, piece_type, sq);
                }
            }

            // King
            add_feature(&mut self.white, &mut self.black, net, color, 5, pieces.king_square);
        }
    }
}

// =============================================================================
// Incremental updates via bitboard diff
// =============================================================================

/// Update an accumulator based on the difference between two board states.
///
/// This handles all move types (normal, capture, castling, en passant, promotion)
/// automatically by diffing the piece bitboards. Call with the pieces BEFORE
/// `make_move_in_place` (saved by the caller) and the pieces AFTER.
pub fn update_accumulator(acc: &mut Accumulator, net: &NnueNetwork, pieces_before: &[Pieces; 2], pieces_after: &[Pieces; 2]) {
    for color in 0..2usize {
        // Diff each piece type's bitboard
        let before = bitboard_piece_types(&pieces_before[color]);
        let after = bitboard_piece_types(&pieces_after[color]);

        for i in 0..5 {
            let (bb_before, piece_type) = before[i];
            let (bb_after, _) = after[i];

            // Bits present before but not after = removed pieces
            let removed = bb_before & !bb_after;
            // Bits present after but not before = added pieces
            let added = !bb_before & bb_after;

            apply_bitboard_delta(&mut acc.white, &mut acc.black, net, color, piece_type, removed, added);
        }

        // King (square, not bitboard)
        let king_before = pieces_before[color].king_square;
        let king_after = pieces_after[color].king_square;
        if king_before != king_after {
            remove_feature(&mut acc.white, &mut acc.black, net, color, 5, king_before);
            add_feature(&mut acc.white, &mut acc.black, net, color, 5, king_after);
        }
    }
}

/// Ceiling on feature changes from one move. Castling moves two pieces
/// (king + rook) = 2 removed + 2 added; en passant and capture-promotions are
/// 2 removed + 1 added. Anything beyond this falls back to the generic path.
const MAX_FEATURE_DELTA: usize = 4;

/// Fused accumulator update: compute `dst` directly from `src` in ONE pass.
///
/// `A`/`S` are const so the inner loops unroll and the whole thing vectorises
/// as a single strided i16 add/sub over the hidden layer.
///
/// Wrapping arithmetic is used deliberately: it makes the result independent of
/// the order features are applied in, so this is bit-identical to the previous
/// clone-then-add/sub sequence even in the (practically unreachable) case where
/// an intermediate value would overflow i16.
#[inline(always)]
fn fuse_features<const A: usize, const S: usize>(
    dst: &mut [i16; HIDDEN_SIZE],
    src: &[i16; HIDDEN_SIZE],
    add: [&[i16; HIDDEN_SIZE]; A],
    sub: [&[i16; HIDDEN_SIZE]; S],
) {
    for j in 0..HIDDEN_SIZE {
        let mut v = src[j];
        for a in add.iter() {
            v = v.wrapping_add(a[j]);
        }
        for s in sub.iter() {
            v = v.wrapping_sub(s[j]);
        }
        dst[j] = v;
    }
}

/// Build a child accumulator from its parent without an intermediate copy.
///
/// The previous flow was `child = parent.clone()` followed by one full pass per
/// changed feature inside `update_accumulator` - three or more passes over the
/// 1 KB accumulator (plus the weight rows) for a quiet move. Gathering the
/// changed features first collapses that to a single pass, roughly halving the
/// memory traffic of the hottest non-eval operation in the search.
///
/// Falls back to the clone-then-update path for unusual deltas, so correctness
/// never depends on the gather fitting.
pub fn update_accumulator_from(
    parent: &Accumulator,
    child: &mut Accumulator,
    net: &NnueNetwork,
    pieces_before: &[Pieces; 2],
    pieces_after: &[Pieces; 2],
) {
    // (white_feature, black_feature) pairs
    let mut adds: [(usize, usize); MAX_FEATURE_DELTA] = [(0, 0); MAX_FEATURE_DELTA];
    let mut subs: [(usize, usize); MAX_FEATURE_DELTA] = [(0, 0); MAX_FEATURE_DELTA];
    let mut n_add = 0usize;
    let mut n_sub = 0usize;
    let mut overflowed = false;

    for color in 0..2usize {
        let before = bitboard_piece_types(&pieces_before[color]);
        let after = bitboard_piece_types(&pieces_after[color]);

        for i in 0..5 {
            let (bb_before, piece_type) = before[i];
            let (bb_after, _) = after[i];

            let mut removed = bb_before & !bb_after;
            while removed != 0 {
                let sq = removed.trailing_zeros() as i8;
                removed &= removed - 1;
                if n_sub == MAX_FEATURE_DELTA {
                    overflowed = true;
                    break;
                }
                subs[n_sub] = (white_feature(color, piece_type, sq), black_feature(color, piece_type, sq));
                n_sub += 1;
            }

            let mut added = !bb_before & bb_after;
            while added != 0 {
                let sq = added.trailing_zeros() as i8;
                added &= added - 1;
                if n_add == MAX_FEATURE_DELTA {
                    overflowed = true;
                    break;
                }
                adds[n_add] = (white_feature(color, piece_type, sq), black_feature(color, piece_type, sq));
                n_add += 1;
            }
        }

        let king_before = pieces_before[color].king_square;
        let king_after = pieces_after[color].king_square;
        if king_before != king_after {
            if n_sub == MAX_FEATURE_DELTA || n_add == MAX_FEATURE_DELTA {
                overflowed = true;
            } else {
                subs[n_sub] = (white_feature(color, 5, king_before), black_feature(color, 5, king_before));
                n_sub += 1;
                adds[n_add] = (white_feature(color, 5, king_after), black_feature(color, 5, king_after));
                n_add += 1;
            }
        }
    }

    if overflowed {
        // Generic path: provably correct, just slower. Never hit by legal moves.
        child.clone_from(parent);
        update_accumulator(child, net, pieces_before, pieces_after);
        return;
    }

    let w = &net.l0_weights;
    match (n_add, n_sub) {
        // Quiet move (also promotion: one piece type out, another in)
        (1, 1) => {
            fuse_features(&mut child.white, &parent.white, [&w[adds[0].0]], [&w[subs[0].0]]);
            fuse_features(&mut child.black, &parent.black, [&w[adds[0].1]], [&w[subs[0].1]]);
        }
        // Capture, en passant, capture-promotion
        (1, 2) => {
            fuse_features(&mut child.white, &parent.white, [&w[adds[0].0]], [&w[subs[0].0], &w[subs[1].0]]);
            fuse_features(&mut child.black, &parent.black, [&w[adds[0].1]], [&w[subs[0].1], &w[subs[1].1]]);
        }
        // Castling: king and rook both move
        (2, 2) => {
            fuse_features(
                &mut child.white,
                &parent.white,
                [&w[adds[0].0], &w[adds[1].0]],
                [&w[subs[0].0], &w[subs[1].0]],
            );
            fuse_features(
                &mut child.black,
                &parent.black,
                [&w[adds[0].1], &w[adds[1].1]],
                [&w[subs[0].1], &w[subs[1].1]],
            );
        }
        _ => {
            child.clone_from(parent);
            update_accumulator(child, net, pieces_before, pieces_after);
        }
    }
}

/// Process removed and added pieces for a single piece type.
#[inline]
fn apply_bitboard_delta(
    white_acc: &mut [i16; HIDDEN_SIZE],
    black_acc: &mut [i16; HIDDEN_SIZE],
    net: &NnueNetwork,
    color: usize,
    piece_type: usize,
    removed: u64,
    added: u64,
) {
    let mut bb = removed;
    while bb != 0 {
        let sq = bb.trailing_zeros() as i8;
        bb &= bb - 1;
        remove_feature(white_acc, black_acc, net, color, piece_type, sq);
    }

    let mut bb = added;
    while bb != 0 {
        let sq = bb.trailing_zeros() as i8;
        bb &= bb - 1;
        add_feature(white_acc, black_acc, net, color, piece_type, sq);
    }
}

// =============================================================================
// Feature mapping (Chess768)
// =============================================================================

/// Map piece bitboards to (bitboard, chess768_piece_type) pairs.
/// Note: Pieces struct has queen before rook, but Chess768 has rook=3, queen=4.
#[inline(always)]
fn bitboard_piece_types(pieces: &Pieces) -> [(u64, usize); 5] {
    [
        (pieces.pawn_bitboard, 0),
        (pieces.knight_bitboard, 1),
        (pieces.bishop_bitboard, 2),
        (pieces.rook_bitboard, 3),
        (pieces.queen_bitboard, 4),
    ]
}

/// Add a piece feature to both perspective accumulators.
#[inline]
fn add_feature(
    white_acc: &mut [i16; HIDDEN_SIZE],
    black_acc: &mut [i16; HIDDEN_SIZE],
    net: &NnueNetwork,
    color: usize,
    piece_type: usize,
    rival_sq: i8,
) {
    let wf = white_feature(color, piece_type, rival_sq);
    let bf = black_feature(color, piece_type, rival_sq);
    let w_weights = &net.l0_weights[wf];
    let b_weights = &net.l0_weights[bf];

    for j in 0..HIDDEN_SIZE {
        white_acc[j] += w_weights[j];
        black_acc[j] += b_weights[j];
    }
}

/// Remove a piece feature from both perspective accumulators.
#[inline]
fn remove_feature(
    white_acc: &mut [i16; HIDDEN_SIZE],
    black_acc: &mut [i16; HIDDEN_SIZE],
    net: &NnueNetwork,
    color: usize,
    piece_type: usize,
    rival_sq: i8,
) {
    let wf = white_feature(color, piece_type, rival_sq);
    let bf = black_feature(color, piece_type, rival_sq);
    let w_weights = &net.l0_weights[wf];
    let b_weights = &net.l0_weights[bf];

    for j in 0..HIDDEN_SIZE {
        white_acc[j] -= w_weights[j];
        black_acc[j] -= b_weights[j];
    }
}

/// Chess768 feature index for white's perspective.
/// Pieces are seen from white's viewpoint (white = friendly, black = enemy).
#[inline(always)]
fn white_feature(piece_color: usize, piece_type: usize, rival_sq: i8) -> usize {
    // Convert Rival square (h1=0) to standard (a1=0): XOR with 7 flips file
    piece_color * 384 + piece_type * 64 + ((rival_sq ^ 7) as u8 as usize)
}

/// Chess768 feature index for black's perspective.
/// Colors are swapped and the board is flipped vertically.
#[inline(always)]
fn black_feature(piece_color: usize, piece_type: usize, rival_sq: i8) -> usize {
    // Swap color (1-color) and flip board (XOR 56 on standard square)
    // Combined: standard_sq = rival_sq ^ 7, then ^ 56 = rival_sq ^ 63
    (1 - piece_color) * 384 + piece_type * 64 + ((rival_sq ^ 63) as u8 as usize)
}
