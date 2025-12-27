use rusty_rival::fen::{get_position, get_fen};
use rusty_rival::search::iterative_deepening;
use rusty_rival::types::default_search_state;
use std::ops::Add;
use std::time::{Duration, Instant};

fn main() {
    let fen = "rnb1kbnr/pppppppp/8/2q4R/8/8/PPPPPPPP/RNBQKBN1 w Qkq - 0 1";

    let original = get_position(&fen.to_string());

    // First search
    println!("Test (100ms, depth 20):");
    {
        let mut search_state = default_search_state();
        search_state.show_info = false;
        let mut position = get_position(&fen.to_string());

        println!("  Before: {}", get_fen(&position));

        search_state.end_time = Instant::now().add(Duration::from_millis(100));
        let mv = iterative_deepening(&mut position, 20, &mut search_state);

        let after_fen = get_fen(&position);
        println!("  After:  {}", after_fen);
        println!("  Move:   {}", mv);

        if position.pieces[0].all_pieces_bitboard != original.pieces[0].all_pieces_bitboard {
            println!("  ERROR: White all_pieces corrupted!");
            println!("    Original: {:064b}", original.pieces[0].all_pieces_bitboard);
            println!("    After:    {:064b}", position.pieces[0].all_pieces_bitboard);
        } else {
            println!("  OK: Position intact");
        }

        if get_fen(&position) == fen {
            println!("  OK: FEN matches");
        } else {
            println!("  ERROR: FEN differs!");
        }
    }
}
