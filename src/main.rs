use std::time::Instant;
use rusty_rival::fen::get_position;
use rusty_rival::perft::perft;

fn main() {
    let start = Instant::now();

    let nodes = perft(&get_position(&"r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".to_string()), 5);
    let duration = start.elapsed();
    println!("Time elapsed in perft is: {:?}", duration);
    println!("{} nodes {} nps", nodes, (nodes as f64 / (duration.as_millis() as f64)) * 1000.0);
}
