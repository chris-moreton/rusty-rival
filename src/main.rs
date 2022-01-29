use std::time::Instant;
use rusty_rival::fen::get_position;
use rusty_rival::perft::perft;

fn main() {
    let start = Instant::now();
    unsafe {
        println!("{}", perft(&mut get_position(&"r3k2r/p6p/8/B7/1pp1p3/3b4/P6P/R3K2R w KQkq - 0 1".to_string()), 5));
    }
    let duration = start.elapsed();
    println!("Time elapsed in perft is: {:?}", duration);
}
