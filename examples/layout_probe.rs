//! Prints the size, alignment and field offsets of the hot search structs
//! (NET-1762). Uses only fields that v1.0.78 also has, so the same file builds
//! against both trees and the two outputs can be diffed:
//!
//!     cargo run --release --example layout_probe
use rusty_rival::types::{Pieces, Position, SearchState};
use std::mem::{align_of, offset_of, size_of};

macro_rules! offsets {
    ($ty:ty, $($field:ident),+) => {
        $(println!("{}.{} {}", stringify!($ty), stringify!($field), offset_of!($ty, $field));)+
    };
}

fn main() {
    println!("Pieces size {} align {}", size_of::<Pieces>(), align_of::<Pieces>());
    println!("Position size {} align {}", size_of::<Position>(), align_of::<Position>());
    println!("SearchState size {} align {}", size_of::<SearchState>(), align_of::<SearchState>());
    offsets!(
        Pieces,
        pawn_bitboard,
        knight_bitboard,
        bishop_bitboard,
        queen_bitboard,
        king_square,
        rook_bitboard,
        all_pieces_bitboard
    );
    offsets!(
        Position,
        pieces,
        mover,
        en_passant_square,
        castle_flags,
        half_moves,
        move_number,
        zobrist_lock,
        pawn_key
    );
    offsets!(
        SearchState,
        hash_table,
        pawn_hash_table,
        killer_moves,
        countermoves,
        countermove_history,
        followup_history,
        capture_history,
        ply_move,
        eval_stack,
        correction_history,
        history_moves,
        nodes,
        nodes_limit,
        stop,
        sel_depth,
        nnue_network,
        nnue_accumulators,
        nnue_pieces,
        nnue_computed,
        node_scratch,
        nnue_ply,
        use_nnue,
        contempt,
        eval_noise
    );
}
