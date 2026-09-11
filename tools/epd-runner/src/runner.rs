//! Run a suite against one engine binary: resolve the SAN targets, drive one
//! fresh engine process per position across a pool of workers, score, and
//! write the record to the store.

use crate::epd::{EpdRecord, Suite};
use crate::host;
use crate::san::{move_to_uci, resolve_san};
use crate::store::{self, EngineRecord, HostRecord, PositionRecord, ResultsFile, RunRecord, SolvedAt, SuiteRef, Summary};
use crate::uci::{Engine, Limit, SearchOutput};
use rusty_rival::fen::get_position;
use std::io::{IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub struct RunSpec {
    pub epd_dir: PathBuf,
    pub engine_path: PathBuf,
    pub engine: EngineRecord,
    pub limit: Limit,
    pub threads: u32,
    pub hash_mb: u32,
    pub concurrency: usize,
    pub force: bool,
    pub quiet: bool,
}

pub struct RunOutcome {
    pub record: RunRecord,
    pub cached: bool,
}

/// UCI targets of one position, or the reason they could not be resolved.
struct Targets {
    bm: Vec<String>,
    am: Vec<String>,
    graded: Vec<(String, u32)>,
    error: Option<String>,
}

fn resolve_targets(position: &EpdRecord) -> Targets {
    let board = get_position(&position.fen);
    let mut error = None;
    let mut resolve_all = |sans: &[String]| -> Vec<String> {
        sans.iter()
            .filter_map(|san| match resolve_san(&board, san) {
                Ok(m) => Some(move_to_uci(m)),
                Err(e) => {
                    if error.is_none() {
                        error = Some(e);
                    }
                    None
                }
            })
            .collect()
    };
    let bm = resolve_all(&position.bm);
    let am = resolve_all(&position.am);
    let graded: Vec<(String, u32)> = position
        .graded
        .iter()
        .filter_map(|(san, pts)| resolve_san(&board, san).ok().map(|m| (move_to_uci(m), *pts)))
        .collect();
    if bm.is_empty() && am.is_empty() && error.is_none() {
        error = Some("no bm or am operand".to_string());
    }
    Targets { bm, am, graded, error }
}

fn engine_options(spec: &RunSpec) -> Vec<(String, String)> {
    let mut options = vec![
        ("Threads".to_string(), spec.threads.to_string()),
        ("Hash".to_string(), spec.hash_mb.to_string()),
    ];
    for (k, v) in &spec.engine.options {
        options.push((k.clone(), v.clone()));
    }
    options
}

fn build_record(position: &EpdRecord, targets: &Targets, output: Result<SearchOutput, String>) -> PositionRecord {
    let mut record = PositionRecord {
        id: position.id.clone(),
        bm: position.bm.clone(),
        am: position.am.clone(),
        best: String::new(),
        solved: false,
        solved_at: None,
        points: None,
        score_cp: None,
        mate: None,
        depth: 0,
        nodes: 0,
        ms: 0,
        error: targets.error.clone(),
    };
    let output = match output {
        Ok(o) => o,
        Err(e) => {
            record.error = Some(e);
            return record;
        }
    };
    let good = |mv: &str| (targets.bm.is_empty() || targets.bm.iter().any(|b| b == mv)) && !targets.am.iter().any(|a| a == mv);
    record.best = output.bestmove.clone();
    record.ms = output.elapsed_ms;
    if let Some(last) = output.infos.last() {
        record.depth = last.depth;
        record.nodes = last.nodes;
        record.score_cp = last.score_cp;
        record.mate = last.mate;
        if last.time_ms > 0 {
            record.ms = last.time_ms;
        }
    }
    if record.error.is_some() {
        return record;
    }
    record.solved = good(&output.bestmove);
    if !targets.graded.is_empty() {
        record.points = Some(targets.graded.iter().find(|(m, _)| *m == output.bestmove).map_or(0, |(_, p)| *p));
    }
    if record.solved {
        let mut candidate: Option<SolvedAt> = None;
        for info in &output.infos {
            if info.pv.first().is_some_and(|m| good(m)) {
                if candidate.is_none() {
                    candidate = Some(SolvedAt {
                        depth: info.depth,
                        nodes: info.nodes,
                        ms: info.time_ms,
                    });
                }
            } else {
                candidate = None;
            }
        }
        record.solved_at = Some(candidate.unwrap_or(SolvedAt {
            depth: record.depth,
            nodes: record.nodes,
            ms: record.ms,
        }));
    }
    record
}

fn median(mut values: Vec<u64>) -> Option<u64> {
    if values.is_empty() {
        return None;
    }
    values.sort_unstable();
    Some(values[values.len() / 2])
}

pub fn summarise(positions: &[PositionRecord], graded: bool) -> Summary {
    let scored: Vec<&PositionRecord> = positions.iter().filter(|p| p.error.is_none()).collect();
    let solved = scored.iter().filter(|p| p.solved).count();
    let total_nodes: u64 = scored.iter().map(|p| p.nodes).sum();
    let total_ms: u64 = scored.iter().map(|p| p.ms).sum();
    let mean_depth = if scored.is_empty() {
        0.0
    } else {
        scored.iter().map(|p| p.depth as f64).sum::<f64>() / scored.len() as f64
    };
    Summary {
        solved,
        total: scored.len(),
        points: if graded {
            Some(scored.iter().map(|p| p.points.unwrap_or(0)).sum())
        } else {
            None
        },
        max_points: if graded { Some(10 * scored.len() as u32) } else { None },
        median_solve_nodes: median(scored.iter().filter_map(|p| p.solved_at.as_ref().map(|s| s.nodes)).collect()),
        median_solve_ms: median(scored.iter().filter_map(|p| p.solved_at.as_ref().map(|s| s.ms)).collect()),
        mean_depth: (mean_depth * 100.0).round() / 100.0,
        nps: (total_nodes * 1000).checked_div(total_ms).unwrap_or(0),
        errors: positions.len() - scored.len(),
    }
}

fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

fn existing_file(path: &Path, engine: &EngineRecord) -> Result<ResultsFile, String> {
    if path.exists() {
        let mut file = store::load_file(path)?;
        if file.engine.bench.is_none() {
            file.engine.bench = engine.bench;
        }
        Ok(file)
    } else {
        Ok(ResultsFile {
            engine: engine.clone(),
            runs: Vec::new(),
        })
    }
}

/// Run one suite (or return its cached record) and persist the result.
pub fn run_suite(spec: &RunSpec, suite: &Suite) -> Result<RunOutcome, String> {
    let cpu = host::cpu_model();
    let mode = spec.limit.mode();
    let budget = spec.limit.budget();
    let path = store::file_path(&spec.epd_dir, &spec.engine);
    let mut file = existing_file(&path, &spec.engine)?;
    if !spec.force {
        if let Some(run) = file
            .runs
            .iter()
            .find(|r| r.key_matches(&suite.sha256, mode, budget, spec.threads, spec.hash_mb, &cpu))
        {
            return Ok(RunOutcome {
                record: run.clone(),
                cached: true,
            });
        }
    }

    let targets: Vec<Targets> = suite.positions.iter().map(resolve_targets).collect();
    let options = engine_options(spec);
    let total = suite.positions.len();
    let next = Arc::new(Mutex::new(0usize));
    let done = Arc::new(Mutex::new((0usize, 0usize)));
    let results: Arc<Mutex<Vec<Option<PositionRecord>>>> = Arc::new(Mutex::new(vec![None; total]));
    let workers = spec.concurrency.clamp(1, total.max(1));
    let first_error: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    // Live progress only on a terminal; a log gets the summary line alone.
    let show_progress = !spec.quiet && std::io::stderr().is_terminal();

    std::thread::scope(|scope| {
        for _ in 0..workers {
            let next = Arc::clone(&next);
            let done = Arc::clone(&done);
            let results = Arc::clone(&results);
            let first_error = Arc::clone(&first_error);
            let options = &options;
            let targets = &targets;
            scope.spawn(move || loop {
                let index = {
                    let mut n = next.lock().unwrap();
                    if *n >= total {
                        break;
                    }
                    let i = *n;
                    *n += 1;
                    i
                };
                let position = &suite.positions[index];
                let target = &targets[index];
                let output = if target.error.is_some() {
                    Err(target.error.clone().unwrap_or_default())
                } else {
                    match Engine::spawn(&spec.engine_path, options, Duration::from_secs(60)) {
                        Ok(mut engine) => {
                            let out = engine.search(&position.fen, spec.limit);
                            engine.quit();
                            out
                        }
                        Err(e) => Err(e),
                    }
                };
                if let Err(e) = &output {
                    let mut fe = first_error.lock().unwrap();
                    if fe.is_none() {
                        *fe = Some(format!("{}: {}", position.id, e));
                    }
                }
                let record = build_record(position, target, output);
                let solved = record.solved;
                results.lock().unwrap()[index] = Some(record);
                if show_progress {
                    let mut d = done.lock().unwrap();
                    d.0 += 1;
                    d.1 += solved as usize;
                    eprint!("\r  {:<14} {:>5}/{:<5} solved {:>5}", suite.name, d.0, total, d.1);
                    let _ = std::io::stderr().flush();
                }
            });
        }
    });
    if show_progress {
        eprintln!();
    }

    let positions: Vec<PositionRecord> = results
        .lock()
        .unwrap()
        .iter()
        .cloned()
        .map(|r| r.expect("every position produces a record"))
        .collect();
    let graded = suite.is_graded();
    let summary = summarise(&positions, graded);
    let record = RunRecord {
        suite: SuiteRef {
            name: suite.name.clone(),
            sha256: suite.sha256.clone(),
            positions: total,
        },
        mode: mode.to_string(),
        budget,
        threads: spec.threads,
        hash_mb: spec.hash_mb,
        host: Some(HostRecord {
            hostname: host::hostname(),
            cpu,
        }),
        date: now_rfc3339(),
        summary,
        positions,
    };
    let cpu_now = record.host.as_ref().map(|h| h.cpu.clone()).unwrap_or_default();
    file.runs
        .retain(|r| !r.key_matches(&suite.sha256, mode, budget, spec.threads, spec.hash_mb, &cpu_now));
    file.runs.push(record.clone());
    file.engine = spec.engine.clone();
    store::save_file(&path, &file)?;
    if let Some(e) = first_error.lock().unwrap().as_ref() {
        if record.summary.errors == total {
            return Err(format!("every position failed; first error: {}", e));
        }
        if !spec.quiet {
            eprintln!("  warning: {} position(s) failed, first: {}", record.summary.errors, e);
        }
    }
    Ok(RunOutcome { record, cached: false })
}
