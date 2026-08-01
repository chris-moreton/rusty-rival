//! NNUE (Efficiently Updatable Neural Network) evaluation.
//!
//! Architecture: (768 → 256)x2 → 1 with SCReLU activation
//! - 768 inputs: Chess768 piece-square features (64 squares × 6 pieces × 2 colors)
//! - 256 hidden neurons, dual perspective (side-to-move + not-side-to-move)
//! - 1 output: evaluation score
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
pub const HIDDEN_SIZE: usize = 256;

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
/// `rival-256x2-ob8.bin` — 8 output buckets by material count (NET-321),
/// 600 superbatches on the same 635M Stockfish depth-9 dataset as the previous
/// single-bucket net. The previous net, `rival-256x2.bin`, is kept in the repo:
/// the loader still reads single-bucket nets, so switching back is a one-line
/// change if the A/B match goes against this one.
const EMBEDDED_NET: &[u8] = include_bytes!("../nets/rival-256x2-ob8.bin");

// =============================================================================
// Network
// =============================================================================

/// Quantised NNUE network weights, loaded once at startup.
pub struct NnueNetwork {
    /// L0 weights indexed as [feature][neuron] for cache-friendly incremental updates.
    /// Each feature's 256 weights are contiguous in memory.
    l0_weights: Box<[[i16; HIDDEN_SIZE]; INPUT_SIZE]>,
    /// L0 biases: initial accumulator values before any features are added.
    l0_biases: [i16; HIDDEN_SIZE],
    /// L1 weights, one 512-wide row per output bucket (bucket-major).
    ///
    /// Within a row: first 256 are **not**-side-to-move, last 256 are
    /// side-to-move — `evaluate()` indexes `[HIDDEN_SIZE + i]` for STM.
    ///
    /// ## Two layout facts that will bite a retrain if ignored
    ///
    /// 1. **Bucket-major is what bullet writes** *only because* the trainer's
    ///    `l1w` save entry has `.transpose()`. Derived from pinned rev 7bc395f3:
    ///    `new_affine` builds `Shape::new(out, in)` = (8, 512), and
    ///    `transpose_impl` writes `new_buf[cols*i + j] = weights[rows*j + i]`,
    ///    i.e. `[bucket][512]`. Drop the `.transpose()` and this silently
    ///    becomes `[512][bucket]`.
    /// 2. **The perspective halves are NTM-first here, but the trainer
    ///    concatenates `stm.concat(ntm)`** — i.e. STM-first. The shipped net is
    ///    self-consistent with this loader (a queen-up position evaluates
    ///    strongly positive, which a swapped net could not do), so the two
    ///    conventions currently cancel. **A retrain could land either way.**
    ///    `nnue_eval_signs_are_correct` is the guard: it fails loudly on a
    ///    swapped net. If it fails after a retrain, swap the two indices here
    ///    rather than assuming the net is bad.
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
    /// Accepts **both** net formats and normalises to the bucketed representation:
    ///
    /// * single-bucket: `l0w[768][256] + l0b[256] + l1w[512] + l1b[1]`
    /// * 8-bucket:      `l0w[768][256] + l0b[256] + l1w[8][512] + l1b[8]`
    ///
    /// A single-bucket net is replicated into all 8 buckets, so it evaluates
    /// bit-identically to the pre-bucket implementation. That is what keeps
    /// `nnue_golden_values_are_bit_identical` passing across this change, and it
    /// means the engine stays shippable before the bucketed net is trained.
    ///
    /// All values are little-endian i16. l0_weights is transposed to [768][256]
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

        // L0 weights: stored as [feature][neuron] = 768 features × 256 neurons
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
            // Bucket-major: each bucket's 512 weights are contiguous.
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

        NnueNetwork {
            l0_weights,
            l0_biases,
            l1_weights,
            l1_biases,
        }
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
            // L1 layout: NTM first [0..255], STM second [256..511]
            output += s * s / QA * l1_weights[HIDDEN_SIZE + i] as i32;
            output += n * n / QA * l1_weights[i] as i32;
        }

        // Add bias (already in scale QA*QB) and convert to centipawns
        output += self.l1_biases[bucket] as i32;
        (output * EVAL_SCALE / (QA * QB)) as Score
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
