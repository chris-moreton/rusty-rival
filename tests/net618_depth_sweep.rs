use rusty_rival::fen::{algebraic_move_from_move, get_position};
use rusty_rival::search::iterative_deepening;
use rusty_rival::types::default_search_state;
use std::ops::Add;
use std::time::{Duration, Instant};

const CASES: [(&str, &str, &str); 2] = [
    ("arasan18.224", "2r1k2r/pp1bb1pp/6n1/3Q1p2/1B1N4/P7/1q4PP/4RRK1 w k - 0 1", "b4e7"),
    (
        "arasan18.164",
        "r1b3kr/pp1n2Bp/2pb2q1/3p3N/3P4/2P2Q2/P1P3PP/4RRK1 w - - 0 1",
        "e1e5",
    ),
];

#[test]
#[ignore]
fn net618_depth_sweep() {
    for (id, fen, want) in CASES {
        for depth in 9..=13u8 {
            let mut s = default_search_state();
            s.use_nnue = false;
            s.show_info = false;
            s.end_time = Instant::now().add(Duration::from_millis(600_000));
            let mut p = get_position(fen);
            let mv = iterative_deepening(&mut p, depth, &mut s, 1);
            let got = algebraic_move_from_move(mv);
            println!(
                "{}|d{}|{}|{}|{}",
                id,
                depth,
                got,
                s.current_best.1,
                if got == want { "SOLVED" } else { "" }
            );
        }
    }
}
