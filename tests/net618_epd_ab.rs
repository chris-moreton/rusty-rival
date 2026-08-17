use rusty_rival::fen::{algebraic_move_from_move, get_position};
use rusty_rival::search::iterative_deepening;
use rusty_rival::types::default_search_state;
use std::ops::Add;
use std::time::{Duration, Instant};

/// A/B corpus probe for NET-618. Prints "id|move|score" per position so the
/// same run with razoring enabled and disabled can be diffed. No ground truth
/// needed: the question is only whether razoring changes the verdict.
#[test]
#[ignore]
fn net618_epd_ab() {
    let depth: u8 = std::env::var("NET618_DEPTH").ok().and_then(|d| d.parse().ok()).unwrap_or(9);
    let file = std::env::var("NET618_EPD").unwrap_or_else(|_| "epd/arasan18.epd".to_string());
    let bytes = std::fs::read(&file).expect("epd file");
    let text = String::from_utf8_lossy(&bytes).to_string();

    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let fen: String = line.split(" bm ").next().unwrap_or(line).trim().to_string();
        let id = line
            .split("id \"")
            .nth(1)
            .and_then(|s| s.split('"').next())
            .unwrap_or("?")
            .to_string();

        let mut s = default_search_state();
        s.use_nnue = false;
        s.show_info = false;
        s.end_time = Instant::now().add(Duration::from_millis(600_000));
        let mut p = get_position(&format!("{} 0 1", fen));
        let mv = iterative_deepening(&mut p, depth, &mut s, 1);
        println!("{}|{}|{}", id, algebraic_move_from_move(mv), s.current_best.1);
    }
}
