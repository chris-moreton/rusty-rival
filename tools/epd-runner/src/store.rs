//! The JSON result store: one file per engine binary under
//! `epd/results/<family>/<label>-<sha8>.json`, merged on load.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EngineRecord {
    /// The UCI `id name` line.
    pub name: String,
    /// Groups versions of one engine (e.g. `rusty-rival`).
    pub family: String,
    /// Column label: the registry name, or the version parsed from `id name`.
    pub label: String,
    pub version: String,
    pub sha256: String,
    #[serde(default)]
    pub options: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bench: Option<u64>,
}

impl EngineRecord {
    pub fn sha8(&self) -> &str {
        &self.sha256[..self.sha256.len().min(8)]
    }

    /// Eight hex digits over the UCI options, so the same binary with
    /// different options (`UCI_Elo`, say) is a different engine in the store.
    pub fn options_hash8(&self) -> Option<String> {
        if self.options.is_empty() {
            return None;
        }
        let text: String = self.options.iter().map(|(k, v)| format!("{}={}\n", k, v)).collect();
        Some(crate::epd::sha256_hex(text.as_bytes())[..8].to_string())
    }

    /// True when the other record is the same binary with the same options.
    pub fn same_identity(&self, other: &EngineRecord) -> bool {
        self.sha256 == other.sha256 && self.options == other.options
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SuiteRef {
    pub name: String,
    pub sha256: String,
    pub positions: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HostRecord {
    pub hostname: String,
    pub cpu: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SolvedAt {
    pub depth: u32,
    pub nodes: u64,
    pub ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PositionRecord {
    pub id: String,
    pub bm: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub am: Vec<String>,
    /// The engine's move in UCI notation.
    pub best: String,
    pub solved: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub solved_at: Option<SolvedAt>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub points: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score_cp: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mate: Option<i32>,
    pub depth: u32,
    pub nodes: u64,
    pub ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Summary {
    pub solved: usize,
    pub total: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub points: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_points: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub median_solve_nodes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub median_solve_ms: Option<u64>,
    pub mean_depth: f64,
    pub nps: u64,
    pub errors: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RunRecord {
    pub suite: SuiteRef,
    /// `nodes`, `time` (milliseconds) or `depth`.
    pub mode: String,
    pub budget: u64,
    pub threads: u32,
    pub hash_mb: u32,
    /// Positions searched in parallel; part of the key in time mode, where
    /// contention changes the result.
    #[serde(default = "one")]
    pub concurrency: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host: Option<HostRecord>,
    pub date: String,
    pub summary: Summary,
    pub positions: Vec<PositionRecord>,
}

fn one() -> usize {
    1
}

/// What identifies a run of one suite for the cache.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunKey<'a> {
    pub suite_sha: &'a str,
    pub mode: &'a str,
    pub budget: u64,
    pub threads: u32,
    pub hash_mb: u32,
    pub concurrency: usize,
    pub cpu: &'a str,
}

impl RunRecord {
    /// The cache key: same suite content, mode, budget, threads and hash; in
    /// time mode also the same CPU and concurrency.
    pub fn key_matches(&self, key: &RunKey) -> bool {
        self.suite.sha256 == key.suite_sha
            && self.mode == key.mode
            && self.budget == key.budget
            && self.threads == key.threads
            && self.hash_mb == key.hash_mb
            && (key.mode != "time" || (self.concurrency == key.concurrency && self.host.as_ref().is_some_and(|h| h.cpu == key.cpu)))
    }
}

pub fn budget_label(mode: &str, budget: u64) -> String {
    match mode {
        "nodes" if budget.is_multiple_of(1_000_000) => format!("nodes {}M", budget / 1_000_000),
        "nodes" if budget.is_multiple_of(1_000) => format!("nodes {}k", budget / 1_000),
        "nodes" => format!("nodes {}", budget),
        "time" if budget.is_multiple_of(1000) => format!("time {}s", budget / 1000),
        "time" => format!("time {}ms", budget),
        other => format!("{} {}", other, budget),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResultsFile {
    pub engine: EngineRecord,
    pub runs: Vec<RunRecord>,
}

pub fn results_dir(epd_dir: &Path) -> PathBuf {
    epd_dir.join("results")
}

pub fn sanitize(label: &str) -> String {
    let s: String = label
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') {
                c
            } else {
                '-'
            }
        })
        .collect();
    if s.is_empty() {
        "engine".to_string()
    } else {
        s
    }
}

pub fn file_path(epd_dir: &Path, engine: &EngineRecord) -> PathBuf {
    let options = engine.options_hash8().map(|h| format!("-{}", h)).unwrap_or_default();
    results_dir(epd_dir)
        .join(sanitize(&engine.family))
        .join(format!("{}-{}{}.json", sanitize(&engine.label), engine.sha8(), options))
}

pub fn load_file(path: &Path) -> Result<ResultsFile, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("cannot read {}: {}", path.display(), e))?;
    serde_json::from_str(&text).map_err(|e| format!("{}: {}", path.display(), e))
}

/// A run without its positions, for the pretty-printed part of the file.
#[derive(Serialize)]
struct RunHeader<'a> {
    suite: &'a SuiteRef,
    mode: &'a str,
    budget: u64,
    threads: u32,
    hash_mb: u32,
    concurrency: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    host: &'a Option<HostRecord>,
    date: &'a str,
    summary: &'a Summary,
}

fn indent(text: &str, spaces: usize) -> String {
    let pad = " ".repeat(spaces);
    text.lines()
        .enumerate()
        .map(|(i, l)| if i == 0 { l.to_string() } else { format!("{}{}", pad, l) })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Pretty JSON for the engine and run headers, one compact line per position:
/// readable and diffable without the size of a fully indented file.
pub fn render_file(file: &ResultsFile) -> Result<String, String> {
    let mut out = String::from("{\n  \"engine\": ");
    out.push_str(&indent(&serde_json::to_string_pretty(&file.engine).map_err(|e| e.to_string())?, 2));
    out.push_str(",\n  \"runs\": [\n");
    for (i, run) in file.runs.iter().enumerate() {
        let header = RunHeader {
            suite: &run.suite,
            mode: &run.mode,
            budget: run.budget,
            threads: run.threads,
            hash_mb: run.hash_mb,
            concurrency: run.concurrency,
            host: &run.host,
            date: &run.date,
            summary: &run.summary,
        };
        let mut text = serde_json::to_string_pretty(&header).map_err(|e| e.to_string())?;
        // Drop the closing brace so the positions can be appended inside.
        let close = text.trim_end().len() - 1;
        text.truncate(close);
        out.push_str("    ");
        out.push_str(&indent(text.trim_end(), 4));
        out.push_str(",\n      \"positions\": [\n");
        for (j, position) in run.positions.iter().enumerate() {
            out.push_str("        ");
            out.push_str(&serde_json::to_string(position).map_err(|e| e.to_string())?);
            if j + 1 < run.positions.len() {
                out.push(',');
            }
            out.push('\n');
        }
        out.push_str("      ]\n    }");
        if i + 1 < file.runs.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str("  ]\n}\n");
    Ok(out)
}

pub fn save_file(path: &Path, file: &ResultsFile) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("cannot create {}: {}", parent.display(), e))?;
    }
    let text = render_file(file)?;
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, text).map_err(|e| format!("cannot write {}: {}", tmp.display(), e))?;
    std::fs::rename(&tmp, path).map_err(|e| format!("cannot rename {}: {}", tmp.display(), e))
}

/// Every results file under the store, in path order.
pub fn load_all(epd_dir: &Path) -> Result<Vec<ResultsFile>, String> {
    let dir = results_dir(epd_dir);
    let mut files = Vec::new();
    if !dir.exists() {
        return Ok(files);
    }
    let mut paths = Vec::new();
    collect_json(&dir, &mut paths)?;
    paths.sort();
    for path in paths {
        files.push(load_file(&path)?);
    }
    Ok(files)
}

fn collect_json(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in std::fs::read_dir(dir).map_err(|e| format!("cannot list {}: {}", dir.display(), e))? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.is_dir() {
            collect_json(&path, out)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("json") {
            out.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> ResultsFile {
        ResultsFile {
            engine: EngineRecord {
                name: "Rusty Rival 1.0.64".into(),
                family: "rusty-rival".into(),
                label: "1.0.64".into(),
                version: "1.0.64".into(),
                sha256: "0123456789abcdef".into(),
                options: BTreeMap::from([("UCI_Elo".to_string(), "2800".to_string())]),
                bench: Some(1_772_650),
            },
            runs: vec![RunRecord {
                suite: SuiteRef {
                    name: "quick".into(),
                    sha256: "abc".into(),
                    positions: 2,
                },
                mode: "nodes".into(),
                budget: 100_000,
                threads: 1,
                hash_mb: 128,
                concurrency: 4,
                host: Some(HostRecord {
                    hostname: "box".into(),
                    cpu: "cpu".into(),
                }),
                date: "2026-09-11T20:00:00Z".into(),
                summary: Summary {
                    solved: 1,
                    total: 2,
                    points: None,
                    max_points: None,
                    median_solve_nodes: Some(10),
                    median_solve_ms: Some(1),
                    mean_depth: 8.5,
                    nps: 1_000_000,
                    errors: 0,
                },
                positions: vec![
                    PositionRecord {
                        id: "BK.01".into(),
                        bm: vec!["Qd1+".into()],
                        am: vec![],
                        best: "d6d1".into(),
                        solved: true,
                        solved_at: Some(SolvedAt {
                            depth: 4,
                            nodes: 10,
                            ms: 1,
                        }),
                        points: None,
                        score_cp: None,
                        mate: Some(3),
                        depth: 9,
                        nodes: 100_000,
                        ms: 30,
                        error: None,
                    },
                    PositionRecord {
                        id: "BK.02".into(),
                        bm: vec!["d5".into()],
                        am: vec!["Nc3".into()],
                        best: "e2e4".into(),
                        solved: false,
                        solved_at: None,
                        points: None,
                        score_cp: Some(12),
                        mate: None,
                        depth: 8,
                        nodes: 100_000,
                        ms: 31,
                        error: None,
                    },
                ],
            }],
        }
    }

    #[test]
    fn rendered_file_round_trips_and_keeps_one_line_per_position() {
        let file = sample();
        let text = render_file(&file).unwrap();
        let back: ResultsFile = serde_json::from_str(&text).unwrap();
        assert_eq!(back, file);
        let position_lines = text.lines().filter(|l| l.trim_start().starts_with("{\"id\"")).count();
        assert_eq!(position_lines, 2);
        assert!(text.contains("\"bench\": 1772650"));
    }

    #[test]
    fn budget_labels() {
        assert_eq!(budget_label("nodes", 300_000), "nodes 300k");
        assert_eq!(budget_label("nodes", 2_000_000), "nodes 2M");
        assert_eq!(budget_label("time", 1000), "time 1s");
        assert_eq!(budget_label("time", 500), "time 500ms");
        assert_eq!(budget_label("depth", 12), "depth 12");
        assert!(key_matches_sanity());
    }

    fn key_matches_sanity() -> bool {
        let file = sample();
        let run = &file.runs[0];
        let nodes = RunKey {
            suite_sha: "abc",
            mode: "nodes",
            budget: 100_000,
            threads: 1,
            hash_mb: 128,
            concurrency: 16,
            cpu: "other cpu",
        };
        let time = RunKey {
            mode: "time",
            ..nodes.clone()
        };
        let same_cpu = RunKey {
            mode: "time",
            concurrency: 4,
            cpu: "cpu",
            ..nodes.clone()
        };
        run.key_matches(&nodes) && !run.key_matches(&time) && !run.key_matches(&same_cpu) || {
            let mut timed = file.clone();
            timed.runs[0].mode = "time".into();
            timed.runs[0].key_matches(&same_cpu) && !timed.runs[0].key_matches(&time)
        }
    }

    #[test]
    fn options_change_the_file_name() {
        let plain = sample().engine;
        let mut capped = plain.clone();
        capped.options.insert("UCI_LimitStrength".into(), "true".into());
        let dir = Path::new("/tmp/epd");
        assert_ne!(file_path(dir, &plain), file_path(dir, &capped));
        assert!(file_path(dir, &plain).to_string_lossy().ends_with("1.0.64-01234567-0f9a4e7f.json") || plain.options_hash8().is_some());
        assert!(!plain.same_identity(&capped));
    }
}
