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
pub const HIDDEN_SIZE: usize = 256;

// Quantization constants (must match training config)
const QA: i32 = 255; // L0 weight/bias and accumulator scale
const QB: i32 = 64; // L1 weight scale
const EVAL_SCALE: i32 = 400; // Converts network output to centipawns

/// Embedded network weights (trained with bullet, quantised.bin format)
const EMBEDDED_NET: &[u8] = include_bytes!("../nets/rival-256x2.bin");

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
    /// L1 weights: first 256 for side-to-move, last 256 for not-side-to-move.
    l1_weights: [i16; 2 * HIDDEN_SIZE],
    /// L1 bias (single value, scale QA*QB).
    l1_bias: i16,
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
    /// File layout: l0_weights[256][768] + l0_biases[256] + l1_weights[512] + l1_bias[1]
    /// All values are little-endian i16. We transpose l0_weights to [768][256] during loading
    /// for cache-friendly feature-indexed access during incremental updates.
    pub fn from_bytes(data: &[u8]) -> Self {
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

        // L1 weights (512 values: 256 for STM perspective, 256 for NTM)
        let mut l1_weights = [0i16; 2 * HIDDEN_SIZE];
        for item in l1_weights.iter_mut() {
            *item = read_i16(data, &mut offset);
        }

        // L1 bias
        let l1_bias = read_i16(data, &mut offset);

        NnueNetwork {
            l0_weights,
            l0_biases,
            l1_weights,
            l1_bias,
        }
    }

    /// Evaluate the position using the current accumulator.
    ///
    /// Returns a score in centipawns from the side-to-move's perspective.
    /// Uses SCReLU activation: clamp(x, 0, QA)² with quantized arithmetic.
    #[inline]
    pub fn evaluate(&self, acc: &Accumulator, _stm: Mover) -> Score {
        // After compute(), white=STM perspective, black=NTM perspective
        // (the position was normalized so STM is always "white" in feature space)
        let (stm_acc, ntm_acc) = (&acc.white, &acc.black);

        let mut output: i32 = 0;

        for i in 0..HIDDEN_SIZE {
            let s = (stm_acc[i] as i32).clamp(0, QA);
            let n = (ntm_acc[i] as i32).clamp(0, QA);
            // SCReLU: squared clipped ReLU, divided by QA to prevent overflow
            // L1 layout: NTM first [0..255], STM second [256..511]
            output += s * s / QA * self.l1_weights[HIDDEN_SIZE + i] as i32;
            output += n * n / QA * self.l1_weights[i] as i32;
        }

        // Add bias (already in scale QA*QB) and convert to centipawns
        output += self.l1_bias as i32;
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
    /// When black moves, we flip: swap piece colors and mirror squares vertically.
    /// The `white` accumulator becomes STM perspective, `black` becomes NTM perspective.
    /// Compute the accumulator from scratch for the given position.
    ///
    /// bullet's Chess768 uses STM-relative features:
    /// - STM accumulator (white): friendly pieces in [0,383], enemy in [384,767]
    /// - NTM accumulator (black): friendly pieces in [0,383], enemy in [384,767], squares flipped
    ///
    /// When black is to move, we swap colors so black pieces become "friendly" (color=0).
    /// The NTM perspective automatically handles the vertical square flip via black_feature().
    pub fn compute(&mut self, net: &NnueNetwork, position: &Position) {
        self.white = net.l0_biases;
        self.black = net.l0_biases;

        let stm = position.mover as usize; // 0=white, 1=black

        for color in 0..2usize {
            let pieces = &position.pieces[color];
            // Map absolute color to STM-relative: 0 = friendly (same as STM), 1 = enemy
            let relative_color = if color == stm { 0 } else { 1 };

            // When black is STM, flip squares vertically to normalize the board
            // (matching bullet's ChessBoard from_raw normalization)
            let sq_flip: i8 = if stm == 1 { 56 } else { 0 };

            for &(bb, piece_type) in &bitboard_piece_types(pieces) {
                let mut bitboard = bb;
                while bitboard != 0 {
                    let sq = bitboard.trailing_zeros() as i8;
                    bitboard &= bitboard - 1;
                    add_feature(&mut self.white, &mut self.black, net, relative_color, piece_type, sq ^ sq_flip);
                }
            }

            // King
            add_feature(
                &mut self.white,
                &mut self.black,
                net,
                relative_color,
                5,
                pieces.king_square ^ sq_flip,
            );
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
