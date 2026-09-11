//! Every shipped suite must parse, and every `bm`, `am` and graded move must
//! resolve to exactly one legal move. STS lines carry the UCI form of their
//! graded moves in `c9`, which cross-checks the SAN resolver on about 6,000
//! moves.

use rusty_rival::fen::{algebraic_move_from_move, get_position};
use std::path::Path;

#[path = "../src/epd.rs"]
mod epd;
#[path = "../src/san.rs"]
mod san;

fn suites_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../epd/suites")
}

#[test]
fn every_suite_parses_and_every_target_resolves() {
    let files = epd::list_suite_files(&suites_dir()).unwrap();
    assert!(files.len() >= 5, "expected the shipped suites, found {:?}", files);
    let mut checked = 0;
    for file in files {
        let suite = epd::load_suite(&file).unwrap();
        assert!(!suite.positions.is_empty(), "{} is empty", suite.name);
        for position in &suite.positions {
            let board = get_position(&position.fen);
            assert!(
                !position.bm.is_empty() || !position.am.is_empty(),
                "{} {} has neither bm nor am",
                suite.name,
                position.id
            );
            for san in position
                .bm
                .iter()
                .chain(position.am.iter())
                .chain(position.graded.iter().map(|(m, _)| m))
            {
                san::resolve_san(&board, san).unwrap_or_else(|e| panic!("{} {}: {}", suite.name, position.id, e));
                checked += 1;
            }
        }
    }
    assert!(checked > 2_000, "only {} moves checked", checked);
}

#[test]
fn sts_graded_moves_match_their_uci_form() {
    let suite = epd::load_suite(&suites_dir().join("sts.epd")).unwrap();
    assert!(suite.is_graded());
    let mut compared = 0;
    for position in &suite.positions {
        let Some(c9) = position.ops.get("c9") else { continue };
        let uci: Vec<&str> = c9.split_whitespace().collect();
        let board = get_position(&position.fen);
        for ((san, _), expected) in position.graded.iter().zip(uci) {
            let m = san::resolve_san(&board, san).unwrap_or_else(|e| panic!("{}: {}", position.id, e));
            assert_eq!(algebraic_move_from_move(m), expected, "{} {}", position.id, san);
            compared += 1;
        }
    }
    assert!(compared > 5_000, "only {} moves compared", compared);
}
