use std::time::Instant;
use crate::fen::algebraic_move_from_move;
use crate::make_move::{make_move};
use crate::moves::{is_check, moves};
use crate::types::{Move, Position};
use num_format::{Locale, ToFormattedString};

pub fn perft(position: &Position, depth: u8) -> u64 {

    let start = Instant::now();

    return perft(position, depth, depth, start, 0);

    pub fn perft(position: &Position, depth: u8, start_depth: u8, start_time: Instant, mut total_nodes: u64) -> u64 {
        let mut count = 0;
        let mover = position.mover;

        for m in moves(position) {
            let mut new_position = *position;
            make_move(position, m, &mut new_position);
            if !is_check(&new_position, mover) {
                count += if depth == 0 {
                    1
                } else {
                    let nodes = perft(&new_position, depth - 1, start_depth, start_time, total_nodes);
                    total_nodes += nodes;
                    if depth == start_depth {
                        show_for_move(start_time, total_nodes, m, nodes)
                    }
                    nodes
                }
            }
        };

        count
    }

    #[inline(always)]
    fn show_for_move(start_time: Instant, total_nodes: u64, m: Move, nodes: u64) {
        let duration = start_time.elapsed();
        println!("{}: {}  {} nps",
                 algebraic_move_from_move(m),
                 nodes.to_formatted_string(&Locale::en),
                 (((total_nodes as f64 / (duration.as_millis() as f64)) * 1000.0) as u64).to_formatted_string(&Locale::en))
    }
}