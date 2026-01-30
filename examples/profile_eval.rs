//! Profile evaluation function components
//!
//! Usage: cargo run --release --example profile_eval
//!
//! Shows time breakdown for each evaluation component.

use rusty_rival::evaluate::*;
use rusty_rival::fen::get_position;
use rusty_rival::material_imbalance::material_imbalance_score;
use rusty_rival::piece_square_tables::piece_square_values;
use rusty_rival::types::{Position, Score, BLACK, WHITE};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::{Duration, Instant};

fn load_positions(epd_path: &str) -> Vec<Position> {
    let file = File::open(epd_path).expect("Failed to open EPD file");
    let reader = BufReader::new(file);

    reader
        .lines()
        .filter_map(|line| {
            let line = line.ok()?;
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let fen = if parts.len() >= 6 && parts[4].chars().all(|c| c.is_ascii_digit()) {
                    format!("{} {} {} {} {} {}", parts[0], parts[1], parts[2], parts[3], parts[4], parts[5])
                } else {
                    format!("{} {} {} {} 0 1", parts[0], parts[1], parts[2], parts[3])
                };
                Some(get_position(&fen))
            } else {
                None
            }
        })
        .collect()
}

struct ComponentTimer {
    name: &'static str,
    total_time: Duration,
    call_count: u64,
}

impl ComponentTimer {
    fn new(name: &'static str) -> Self {
        Self {
            name,
            total_time: Duration::ZERO,
            call_count: 0,
        }
    }

    fn record(&mut self, elapsed: Duration) {
        self.total_time += elapsed;
        self.call_count += 1;
    }

    fn avg_nanos(&self) -> f64 {
        if self.call_count == 0 {
            0.0
        } else {
            self.total_time.as_nanos() as f64 / self.call_count as f64
        }
    }
}

macro_rules! time_component {
    ($timer:expr, $expr:expr) => {{
        let start = Instant::now();
        let result = $expr;
        $timer.record(start.elapsed());
        result
    }};
}

fn main() {
    let positions = load_positions("epd/sts-v3.epd");
    println!("Loaded {} positions\n", positions.len());

    let iterations = 100;

    // Initialize timers for each component
    let mut t_material = ComponentTimer::new("material_score");
    let mut t_piece_square = ComponentTimer::new("piece_square_values");
    let mut t_king_score = ComponentTimer::new("king_score");
    let mut t_king_threat = ComponentTimer::new("king_threat_score");
    let mut t_rook_eval = ComponentTimer::new("rook_eval");
    let mut t_passed_pawn = ComponentTimer::new("passed_pawn_score");
    let mut t_knight_outpost = ComponentTimer::new("knight_outpost_scores");
    let mut t_doubled_isolated = ComponentTimer::new("doubled_and_isolated_pawn");
    let mut t_bishop_mobility = ComponentTimer::new("bishop_mobility_score");
    let mut t_backward_pawn = ComponentTimer::new("backward_pawn_score");
    let mut t_bishop_pair = ComponentTimer::new("bishop_pair_bonus");
    let mut t_knight_fork = ComponentTimer::new("knight_fork_threat_score");
    let mut t_rook_file = ComponentTimer::new("rook_file_score");
    let mut t_queen_mobility = ComponentTimer::new("queen_mobility_score");
    let mut t_king_central = ComponentTimer::new("endgame_king_centralization");
    let mut t_king_activity = ComponentTimer::new("king_activity_score");
    let mut t_king_mobility = ComponentTimer::new("king_mobility_score");
    let mut t_trade_bonus = ComponentTimer::new("trade_bonus");
    let mut t_bn_imbalance = ComponentTimer::new("bishop_knight_imbalance");
    let mut t_mat_imbalance = ComponentTimer::new("material_imbalance_score");
    let mut t_trapped = ComponentTimer::new("trapped_piece_penalty");
    let mut t_space = ComponentTimer::new("space_score");

    // We need to create a cache for functions that require it
    // Since cache is private, we'll measure the aggregate for cache-dependent functions
    let mut t_full_eval = ComponentTimer::new("FULL evaluate()");

    println!("Running {} iterations...\n", iterations);

    let mut checksum: i64 = 0;

    for _ in 0..iterations {
        for pos in &positions {
            // Time the full evaluation for reference
            let score: Score = time_component!(t_full_eval, evaluate(pos));
            checksum = checksum.wrapping_add(score as i64);

            // Time individual components
            time_component!(t_material, material_score(pos));
            time_component!(t_piece_square, piece_square_values(pos));

            // king_score needs cache, so we'll skip individual timing
            // Instead we measure it as part of full eval

            time_component!(t_king_threat, king_threat_score(pos));
            time_component!(t_rook_eval, rook_eval(pos));

            // passed_pawn_score needs cache - skip individual

            // knight_outpost_scores needs cache - skip individual

            // doubled_and_isolated needs cache - skip individual

            time_component!(t_bishop_mobility, bishop_mobility_score(pos));
            time_component!(t_backward_pawn, backward_pawn_score(pos));

            // Bishop pair bonus (called twice in eval)
            time_component!(
                t_bishop_pair,
                bishop_pair_bonus(pos.pieces[WHITE as usize].bishop_bitboard, pos.pieces[WHITE as usize].pawn_bitboard,)
            );
            time_component!(
                t_bishop_pair,
                bishop_pair_bonus(pos.pieces[BLACK as usize].bishop_bitboard, pos.pieces[BLACK as usize].pawn_bitboard,)
            );

            time_component!(t_knight_fork, knight_fork_threat_score(pos));
            time_component!(t_rook_file, rook_file_score(pos));
            time_component!(t_queen_mobility, queen_mobility_score(pos));
            time_component!(t_king_central, endgame_king_centralization_bonus(pos));
            time_component!(t_king_activity, king_activity_score(pos));
            time_component!(t_king_mobility, king_mobility_score(pos));
            time_component!(t_trade_bonus, trade_bonus(pos));
            time_component!(t_bn_imbalance, bishop_knight_imbalance_score(pos));
            time_component!(t_mat_imbalance, material_imbalance_score(pos));
            time_component!(t_trapped, trapped_piece_penalty(pos));
            time_component!(t_space, space_score(pos));
        }
    }

    // Collect and sort by time
    let mut timers = vec![
        &t_full_eval,
        &t_material,
        &t_piece_square,
        &t_king_threat,
        &t_rook_eval,
        &t_bishop_mobility,
        &t_backward_pawn,
        &t_bishop_pair,
        &t_knight_fork,
        &t_rook_file,
        &t_queen_mobility,
        &t_king_central,
        &t_king_activity,
        &t_king_mobility,
        &t_trade_bonus,
        &t_bn_imbalance,
        &t_mat_imbalance,
        &t_trapped,
        &t_space,
    ];

    timers.sort_by(|a, b| b.avg_nanos().partial_cmp(&a.avg_nanos()).unwrap());

    println!("Component Timing (sorted by time, descending):");
    println!("{:-<60}", "");
    println!("{:<35} {:>10} {:>12}", "Component", "Avg (ns)", "% of eval");
    println!("{:-<60}", "");

    let full_eval_ns = t_full_eval.avg_nanos();

    for timer in &timers {
        let pct = if timer.name == "FULL evaluate()" {
            100.0
        } else {
            (timer.avg_nanos() / full_eval_ns) * 100.0
        };
        println!("{:<35} {:>10.1} {:>11.1}%", timer.name, timer.avg_nanos(), pct);
    }

    println!("{:-<60}", "");
    println!("\nNote: Some components (king_score, passed_pawn, knight_outpost,");
    println!("doubled_isolated) need internal cache and couldn't be timed separately.");
    println!("\nChecksum: {} (prevents optimization)", checksum);
}
