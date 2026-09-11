//! Comparing two runs of one suite: which positions flipped, and whether a
//! candidate dropped below a baseline by more than a threshold.

use crate::store::{RunRecord, SolvedAt};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Side {
    pub best: String,
    pub solved: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub solved_at: Option<SolvedAt>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub points: Option<u32>,
}

/// A position solved by one side and not the other.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Flip {
    pub id: String,
    pub bm: Vec<String>,
    pub a: Side,
    pub b: Side,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Score {
    pub solved: usize,
    pub total: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub points: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_points: Option<u32>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SuiteDiff {
    pub suite: String,
    pub suite_sha8: String,
    pub a: Score,
    pub b: Score,
    /// Solved by B, not by A.
    pub gained: Vec<Flip>,
    /// Solved by A, not by B.
    pub lost: Vec<Flip>,
    /// B's solved count minus A's.
    pub net: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub points_net: Option<i64>,
    /// Positions that exist on one side only, or errored on either side.
    pub unmatched: usize,
}

fn short(sha: &str) -> &str {
    &sha[..sha.len().min(8)]
}

fn score(run: &RunRecord) -> Score {
    Score {
        solved: run.summary.solved,
        total: run.summary.total,
        points: run.summary.points,
        max_points: run.summary.max_points,
    }
}

/// Compare two runs of the same suite revision (A is the baseline or left
/// side, B the candidate or right side).
pub fn diff_runs(a: &RunRecord, b: &RunRecord) -> Result<SuiteDiff, String> {
    if a.suite.sha256 != b.suite.sha256 {
        return Err(format!(
            "{}: the two runs used different suite revisions ({} vs {})",
            a.suite.name,
            short(&a.suite.sha256),
            short(&b.suite.sha256)
        ));
    }
    let by_id: BTreeMap<&str, &crate::store::PositionRecord> = b.positions.iter().map(|p| (p.id.as_str(), p)).collect();
    let mut gained = Vec::new();
    let mut lost = Vec::new();
    let mut matched = 0usize;
    for pa in &a.positions {
        let Some(pb) = by_id.get(pa.id.as_str()) else { continue };
        if pa.error.is_some() || pb.error.is_some() {
            continue;
        }
        matched += 1;
        if pa.solved != pb.solved {
            let flip = Flip {
                id: pa.id.clone(),
                bm: pa.bm.clone(),
                a: Side {
                    best: pa.best.clone(),
                    solved: pa.solved,
                    solved_at: pa.solved_at.clone(),
                    points: pa.points,
                },
                b: Side {
                    best: pb.best.clone(),
                    solved: pb.solved,
                    solved_at: pb.solved_at.clone(),
                    points: pb.points,
                },
            };
            if pb.solved {
                gained.push(flip);
            } else {
                lost.push(flip);
            }
        }
    }
    let unmatched = a.positions.len().max(b.positions.len()) - matched;
    let points_net = match (a.summary.points, b.summary.points) {
        (Some(pa), Some(pb)) => Some(pb as i64 - pa as i64),
        _ => None,
    };
    Ok(SuiteDiff {
        suite: a.suite.name.clone(),
        suite_sha8: short(&a.suite.sha256).to_string(),
        a: score(a),
        b: score(b),
        net: b.summary.solved as i64 - a.summary.solved as i64,
        points_net,
        gained,
        lost,
        unmatched,
    })
}

fn solved_at_text(s: &Option<SolvedAt>) -> String {
    match s {
        Some(s) => format!("d{} {}n", s.depth, s.nodes),
        None => "-".to_string(),
    }
}

fn score_text(s: &Score) -> String {
    match (s.points, s.max_points) {
        (Some(p), Some(m)) => format!("{}/{} pts ({}/{} solved)", p, m, s.solved, s.total),
        _ => format!("{}/{}", s.solved, s.total),
    }
}

pub fn render_diff(d: &SuiteDiff, a_label: &str, b_label: &str) -> String {
    let mut out = format!(
        "{} ({}): {} {} → {} {} · net {:+}{}\n",
        d.suite,
        d.suite_sha8,
        a_label,
        score_text(&d.a),
        b_label,
        score_text(&d.b),
        d.net,
        d.points_net.map(|p| format!(" · points {:+}", p)).unwrap_or_default()
    );
    for (title, flips) in [("gained", &d.gained), ("lost", &d.lost)] {
        if flips.is_empty() {
            continue;
        }
        out.push_str(&format!("  {} ({}):\n", title, flips.len()));
        for f in flips {
            out.push_str(&format!(
                "    {:<24} bm {:<10} {}: {} {}  |  {}: {} {}\n",
                f.id,
                f.bm.join("/"),
                a_label,
                f.a.best,
                solved_at_text(&f.a.solved_at),
                b_label,
                f.b.best,
                solved_at_text(&f.b.solved_at)
            ));
        }
    }
    if d.unmatched > 0 {
        out.push_str(&format!(
            "  ({} position(s) not compared: missing on one side or errored)\n",
            d.unmatched
        ));
    }
    out
}

/// The verdict of `check` for one suite.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CheckVerdict {
    pub suite: String,
    pub baseline_solved: usize,
    pub candidate_solved: usize,
    pub drop: i64,
    pub flipped: usize,
    pub ok: bool,
    pub reason: Option<String>,
}

/// A suite passes when the candidate solved no more than `max_drop` fewer
/// positions than the baseline; in exact mode any flip in either direction
/// fails.
pub fn check_suite(d: &SuiteDiff, max_drop: i64, exact: bool) -> CheckVerdict {
    let drop = d.a.solved as i64 - d.b.solved as i64;
    let flipped = d.gained.len() + d.lost.len();
    let (ok, reason) = if exact && flipped > 0 {
        (false, Some(format!("{} position(s) flipped in exact mode", flipped)))
    } else if drop > max_drop {
        (false, Some(format!("solved fell by {} (allowed {})", drop, max_drop)))
    } else {
        (true, None)
    };
    CheckVerdict {
        suite: d.suite.clone(),
        baseline_solved: d.a.solved,
        candidate_solved: d.b.solved,
        drop,
        flipped,
        ok,
        reason,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::{PositionRecord, SuiteRef, Summary};

    fn run(solved: &[bool]) -> RunRecord {
        let positions: Vec<PositionRecord> = solved
            .iter()
            .enumerate()
            .map(|(i, s)| PositionRecord {
                id: format!("P.{}", i),
                bm: vec!["e4".into()],
                am: vec![],
                best: if *s { "e2e4".into() } else { "d2d4".into() },
                solved: *s,
                solved_at: if *s {
                    Some(SolvedAt {
                        depth: 3,
                        nodes: 100,
                        ms: 1,
                    })
                } else {
                    None
                },
                points: None,
                score_cp: Some(0),
                mate: None,
                depth: 5,
                nodes: 1000,
                ms: 2,
                error: None,
            })
            .collect();
        let n = positions.iter().filter(|p| p.solved).count();
        RunRecord {
            suite: SuiteRef {
                name: "t".into(),
                sha256: "abcdef0123".into(),
                positions: positions.len(),
            },
            mode: "nodes".into(),
            budget: 1000,
            threads: 1,
            hash_mb: 16,
            concurrency: 1,
            host: None,
            date: "2026-09-11T00:00:00Z".into(),
            summary: Summary {
                solved: n,
                total: positions.len(),
                points: None,
                max_points: None,
                median_solve_nodes: None,
                median_solve_ms: None,
                mean_depth: 5.0,
                nps: 500_000,
                errors: 0,
            },
            positions,
        }
    }

    #[test]
    fn diff_lists_flips_in_both_directions() {
        let a = run(&[true, true, false, false]);
        let b = run(&[true, false, true, false]);
        let d = diff_runs(&a, &b).unwrap();
        assert_eq!((d.a.solved, d.b.solved, d.net), (2, 2, 0));
        assert_eq!(d.lost.iter().map(|f| f.id.as_str()).collect::<Vec<_>>(), vec!["P.1"]);
        assert_eq!(d.gained.iter().map(|f| f.id.as_str()).collect::<Vec<_>>(), vec!["P.2"]);
        assert_eq!(d.unmatched, 0);
        let text = render_diff(&d, "base", "cand");
        assert!(text.contains("lost (1)") && text.contains("gained (1)") && text.contains("P.1"));
    }

    #[test]
    fn check_applies_the_threshold_and_exact_mode() {
        let a = run(&[true, true, true, false]);
        let b = run(&[true, false, false, true]);
        let d = diff_runs(&a, &b).unwrap();
        assert_eq!(d.net, -1);
        assert!(check_suite(&d, 1, false).ok);
        assert!(!check_suite(&d, 0, false).ok);
        assert!(!check_suite(&d, 5, true).ok, "exact mode fails on any flip");
        let same = diff_runs(&a, &a).unwrap();
        assert!(check_suite(&same, 0, true).ok);
    }

    #[test]
    fn different_suite_revisions_are_refused() {
        let a = run(&[true]);
        let mut b = run(&[true]);
        b.suite.sha256 = "ffff".into();
        assert!(diff_runs(&a, &b).is_err());
    }
}
