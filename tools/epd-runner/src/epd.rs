//! EPD parsing: one record per line, four FEN fields followed by `;`-separated
//! opcodes (`bm`, `am`, `id`, `c0`..`c9`). STS-style graded alternatives are
//! read from `c0` ("Rd4=10, Bd5=8, ...") or from the `c7`/`c8` pair.

use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct EpdRecord {
    /// Six-field FEN for the engine (halfmove and fullmove set to 0 and 1).
    pub fen: String,
    pub id: String,
    /// Best moves in SAN, any of which solves the position.
    pub bm: Vec<String>,
    /// Moves to avoid in SAN.
    pub am: Vec<String>,
    /// Graded alternatives (STS style): SAN and points.
    pub graded: Vec<(String, u32)>,
    /// Every opcode as written; the detail and diff views read `c0` comments
    /// from here.
    #[allow(dead_code)]
    pub ops: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct Suite {
    pub name: String,
    pub sha256: String,
    pub positions: Vec<EpdRecord>,
}

impl Suite {
    /// True when the suite carries graded scores for every position (STS).
    pub fn is_graded(&self) -> bool {
        !self.positions.is_empty() && self.positions.iter().all(|p| !p.graded.is_empty())
    }
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher.finalize().iter().map(|b| format!("{:02x}", b)).collect()
}

pub fn load_suite(path: &Path) -> Result<Suite, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("cannot read {}: {}", path.display(), e))?;
    let text = String::from_utf8_lossy(&bytes);
    let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("suite").to_string();
    let positions = parse_epd(&text, &name).map_err(|e| format!("{}: {}", path.display(), e))?;
    Ok(Suite {
        name,
        sha256: sha256_hex(&bytes),
        positions,
    })
}

/// Every `.epd` file in the directory, sorted by name.
pub fn list_suite_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .map_err(|e| format!("cannot list {}: {}", dir.display(), e))?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("epd"))
        .collect();
    files.sort();
    Ok(files)
}

pub fn parse_epd(text: &str, default_id_prefix: &str) -> Result<Vec<EpdRecord>, String> {
    let mut out = Vec::new();
    for (line_no, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut record = parse_epd_line(line).map_err(|e| format!("line {}: {}", line_no + 1, e))?;
        if record.id.is_empty() {
            record.id = format!("{}.{}", default_id_prefix, out.len() + 1);
        }
        out.push(record);
    }
    Ok(out)
}

pub fn parse_epd_line(line: &str) -> Result<EpdRecord, String> {
    let tokens: Vec<&str> = line.split_whitespace().collect();
    if tokens.len() < 4 {
        return Err("fewer than four FEN fields".to_string());
    }
    let (board, side, castling, ep) = (tokens[0], tokens[1], tokens[2], tokens[3]);
    if side != "w" && side != "b" {
        return Err(format!("bad side to move '{}'", side));
    }
    let mut rest: Vec<&str> = tokens[4..].to_vec();
    // Tolerate a full six-field FEN: drop leading halfmove/fullmove counters.
    while rest.first().is_some_and(|t| t.chars().all(|c| c.is_ascii_digit())) {
        rest.remove(0);
    }
    let fen = format!("{} {} {} {} 0 1", board, side, castling, ep);
    let mut ops = BTreeMap::new();
    for op in rest.join(" ").split(';') {
        let op = op.trim();
        if op.is_empty() {
            continue;
        }
        let (code, operand) = match op.split_once(char::is_whitespace) {
            Some((c, o)) => (c, o.trim()),
            None => (op, ""),
        };
        ops.insert(code.to_string(), operand.trim_matches('"').to_string());
    }
    let bm = san_list(ops.get("bm"));
    let am = san_list(ops.get("am"));
    let id = ops.get("id").cloned().unwrap_or_default();
    let graded = parse_graded(&ops);
    Ok(EpdRecord {
        fen,
        id,
        bm,
        am,
        graded,
        ops,
    })
}

fn san_list(operand: Option<&String>) -> Vec<String> {
    operand
        .map(|s| {
            s.split_whitespace()
                .map(|m| m.trim_end_matches(',').to_string())
                .filter(|m| !m.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

fn parse_graded(ops: &BTreeMap<String, String>) -> Vec<(String, u32)> {
    if let Some(c0) = ops.get("c0") {
        let parts: Vec<&str> = c0.split(',').map(|p| p.trim()).filter(|p| !p.is_empty()).collect();
        let parsed: Vec<Option<(String, u32)>> = parts
            .iter()
            .map(|p| {
                p.split_once('=')
                    .and_then(|(mv, pts)| pts.trim().parse::<u32>().ok().map(|v| (mv.trim().to_string(), v)))
            })
            .collect();
        if !parsed.is_empty() && parsed.iter().all(|p| p.is_some()) {
            return parsed.into_iter().flatten().collect();
        }
    }
    if let (Some(c7), Some(c8)) = (ops.get("c7"), ops.get("c8")) {
        let moves: Vec<&str> = c7.split_whitespace().collect();
        let points: Vec<Option<u32>> = c8.split_whitespace().map(|p| p.parse().ok()).collect();
        if !moves.is_empty() && moves.len() == points.len() && points.iter().all(|p| p.is_some()) {
            return moves.iter().zip(points).map(|(m, p)| (m.to_string(), p.unwrap_or(0))).collect();
        }
    }
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_wac_line() {
        let r = parse_epd_line("2rr3k/pp3pp1/1nnqbN1p/3pN3/2pP4/2P3Q1/PPB4P/R4RK1 w - - bm Qg6; id \"WAC.001\";").unwrap();
        assert_eq!(r.fen, "2rr3k/pp3pp1/1nnqbN1p/3pN3/2pP4/2P3Q1/PPB4P/R4RK1 w - - 0 1");
        assert_eq!(r.bm, vec!["Qg6"]);
        assert_eq!(r.id, "WAC.001");
        assert!(r.graded.is_empty());
    }

    #[test]
    fn parses_sts_grades_and_tolerates_full_fen() {
        let r = parse_epd_line(
            "1kr5/3n4/q3p2p/p2n2p1/PppB1P2/5BP1/1P2Q2P/3R2K1 w - - 0 1 bm f5; id \"STS(v1.0) Undermine.001\"; c0 \"f5=10, Be5+=2, Bf2=3, Bg4=2\"; c7 \"f5 Be5+ Bf2 Bg4\"; c8 \"10 2 3 2\";",
        )
        .unwrap();
        assert_eq!(
            r.graded,
            vec![("f5".into(), 10), ("Be5+".into(), 2), ("Bf2".into(), 3), ("Bg4".into(), 2)]
        );
        assert_eq!(r.fen, "1kr5/3n4/q3p2p/p2n2p1/PppB1P2/5BP1/1P2Q2P/3R2K1 w - - 0 1");
    }

    #[test]
    fn parses_multiple_best_moves_and_avoid_moves() {
        let r = parse_epd_line("8/8/8/8/8/8/8/K6k w - - bm Kb1 Kb2; am Ka2;").unwrap();
        assert_eq!(r.bm, vec!["Kb1", "Kb2"]);
        assert_eq!(r.am, vec!["Ka2"]);
        assert_eq!(r.id, "");
    }
}
