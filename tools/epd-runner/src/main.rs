//! epd-runner: run EPD test suites against UCI engines, keep every result in
//! a per-binary JSON store, show suites × engines tables, and compare runs.

mod compare;
mod config;
mod epd;
mod host;
mod runner;
mod san;
mod store;
mod table;
mod tui;
mod uci;

use clap::{Args, Parser, Subcommand};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Parser)]
#[command(name = "epd-runner", version, about = "EPD test-suite runner for UCI engines")]
struct Cli {
    /// The epd directory (suites/, results/, engines.toml). Defaults to the
    /// repository's epd directory, or ./epd.
    #[arg(long, global = true)]
    epd_dir: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run one engine over one or more suites at a budget and store the result.
    Run(RunArgs),
    /// Print the suites × engines table for one budget from the store.
    Table(TableArgs),
    /// Which positions flipped between two engines at one budget.
    Diff(DiffArgs),
    /// Compare a candidate binary against a baseline record; exit 1 on a drop.
    Check(CheckArgs),
    /// List the suites and their position counts.
    Suites,
    /// The interactive terminal view over the store.
    Tui,
}

#[derive(Args)]
struct BudgetArgs {
    /// Node limit per position (deterministic; the regression mode).
    #[arg(long, conflicts_with_all = ["time", "depth"])]
    nodes: Option<u64>,
    /// Seconds per position (the comparison mode; needs an idle machine).
    #[arg(long, conflicts_with_all = ["nodes", "depth"])]
    time: Option<f64>,
    /// Depth limit per position.
    #[arg(long, conflicts_with_all = ["nodes", "time"])]
    depth: Option<u32>,
}

impl BudgetArgs {
    fn limit(&self) -> Result<uci::Limit, String> {
        match (self.nodes, self.time, self.depth) {
            (Some(n), None, None) => Ok(uci::Limit::Nodes(n)),
            (None, Some(s), None) => Ok(uci::Limit::MoveTime((s * 1000.0).round() as u64)),
            (None, None, Some(d)) => Ok(uci::Limit::Depth(d)),
            _ => Err("give exactly one of --nodes, --time or --depth".to_string()),
        }
    }
}

/// Engine settings shared by the commands that run an engine.
#[derive(Args, Clone)]
struct EngineSettings {
    #[arg(long, default_value_t = 1)]
    threads: u32,
    /// Hash size in MB.
    #[arg(long, default_value_t = 128)]
    hash: u32,
    /// Positions searched in parallel (default: half the logical CPUs in
    /// node and depth mode, 1 in time mode).
    #[arg(long)]
    concurrency: Option<usize>,
    /// Re-run even when the store already holds this result.
    #[arg(long)]
    force: bool,
    /// Run in time mode even if the machine looks busy.
    #[arg(long)]
    allow_busy: bool,
    /// Time mode: the CPU model stored runs must come from when a side is
    /// read from the store (default: this machine's; `any` to ignore).
    #[arg(long)]
    cpu: Option<String>,
}

impl EngineSettings {
    /// Refuse a time-mode run on a busy machine, before any engine is spawned.
    fn guard_busy(&self, limit: uci::Limit) -> Result<(), String> {
        if matches!(limit, uci::Limit::MoveTime(_)) && !self.allow_busy {
            if let Some(reason) = host::busy_reason(4.0) {
                return Err(format!(
                    "time mode needs an idle machine: {} (use --allow-busy to override)",
                    reason
                ));
            }
        }
        Ok(())
    }

    fn concurrency_for(&self, limit: uci::Limit) -> usize {
        self.concurrency.unwrap_or_else(|| match limit {
            uci::Limit::MoveTime(_) => 1,
            _ => std::thread::available_parallelism().map(|n| (n.get() / 2).max(1)).unwrap_or(1),
        })
    }
}

#[derive(Args)]
struct RunArgs {
    /// Path to an engine binary (identity is read from its UCI id name).
    #[arg(long, conflicts_with = "name")]
    engine: Option<PathBuf>,
    /// An engine from epd/engines.toml.
    #[arg(long)]
    name: Option<String>,
    /// Comma-separated suite names, or `all`.
    #[arg(long, default_value = "all")]
    suites: String,
    #[command(flatten)]
    budget: BudgetArgs,
    #[command(flatten)]
    settings: EngineSettings,
    /// Print the run records as JSON instead of a summary.
    #[arg(long)]
    json: bool,
    /// Extra UCI option, NAME=VALUE (repeatable).
    #[arg(long = "option", value_name = "NAME=VALUE")]
    options: Vec<String>,
    /// Column label to store (default: the registry name, or the version).
    #[arg(long)]
    label: Option<String>,
    /// Engine family to store under (default: from the registry or id name).
    #[arg(long)]
    family: Option<String>,
}

#[derive(Args)]
struct TableArgs {
    #[command(flatten)]
    budget: BudgetArgs,
    #[arg(long, default_value_t = 1)]
    threads: u32,
    /// Hash size in MB the runs were made with.
    #[arg(long, default_value_t = 128)]
    hash: u32,
    /// Time mode: the concurrency the runs were made with (default 1).
    #[arg(long)]
    concurrency: Option<usize>,
    /// Time mode: the CPU model the runs were made on (default: this machine's; `any` to ignore).
    #[arg(long)]
    cpu: Option<String>,
    /// Comma-separated engine selectors: `family:label[#hash]`, family, label or version.
    #[arg(long)]
    engines: Option<String>,
    /// Comma-separated suite names.
    #[arg(long)]
    suites: Option<String>,
    /// Show percentages instead of counts.
    #[arg(long)]
    percent: bool,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct DiffArgs {
    /// Left side: an engine binary path, `@name` from epd/engines.toml, a
    /// results file, or a store selector (`family:label[#hash]`).
    a: String,
    /// Right side, the same forms.
    b: String,
    /// Comma-separated suite names, or `all`.
    #[arg(long, default_value = "all")]
    suites: String,
    #[command(flatten)]
    budget: BudgetArgs,
    #[command(flatten)]
    settings: EngineSettings,
    /// Extra UCI option for any side that is run, NAME=VALUE (repeatable).
    #[arg(long = "option", value_name = "NAME=VALUE")]
    options: Vec<String>,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct CheckArgs {
    /// The candidate engine binary.
    #[arg(long, conflicts_with = "name")]
    engine: Option<PathBuf>,
    /// The candidate as an entry from epd/engines.toml.
    #[arg(long)]
    name: Option<String>,
    /// The baseline: `@name` from the registry (run or cached), a results
    /// file, or a store selector (`family:label[#hash]`).
    #[arg(long)]
    baseline: String,
    /// Extra UCI option for the candidate (and a `@name` baseline), NAME=VALUE (repeatable).
    #[arg(long = "option", value_name = "NAME=VALUE")]
    options: Vec<String>,
    /// Comma-separated suite names, or `all`.
    #[arg(long, default_value = "all")]
    suites: String,
    #[command(flatten)]
    budget: BudgetArgs,
    #[command(flatten)]
    settings: EngineSettings,
    /// How many fewer solved positions per suite the candidate may have.
    #[arg(long, default_value_t = 0)]
    max_drop: i64,
    /// Fail on any flipped position in either direction (for node-identical claims).
    #[arg(long)]
    exact: bool,
    #[arg(long)]
    json: bool,
}

/// Write to stdout and ignore a closed pipe (`| head`), instead of panicking.
fn out(text: &str) {
    let mut stdout = std::io::stdout().lock();
    let _ = stdout.write_all(text.as_bytes());
    let _ = stdout.flush();
}

fn default_epd_dir() -> PathBuf {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../epd");
    if repo.join("suites").is_dir() {
        return repo.canonicalize().unwrap_or(repo);
    }
    PathBuf::from("epd")
}

fn split_list(s: &str) -> Vec<String> {
    s.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect()
}

fn load_suites(epd_dir: &Path, selection: &str) -> Result<Vec<epd::Suite>, String> {
    let dir = epd_dir.join("suites");
    let files = epd::list_suite_files(&dir)?;
    let wanted: Option<Vec<String>> = if selection == "all" { None } else { Some(split_list(selection)) };
    let mut suites = Vec::new();
    for path in files {
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
        if wanted.as_ref().is_none_or(|w| w.contains(&stem)) {
            suites.push(epd::load_suite(&path)?);
        }
    }
    if let Some(w) = wanted {
        for name in w {
            if !suites.iter().any(|s| s.name == name) {
                return Err(format!("no suite named '{}' in {}", name, dir.display()));
            }
        }
    }
    Ok(suites)
}

/// Family and version from a UCI id name: "Rusty Rival 1.0.64" gives
/// ("rusty-rival", "1.0.64"); "Stockfish dev-20260726-23cf5d82" gives
/// ("stockfish", "dev-20260726-23cf5d82").
fn identity_from_id_name(id_name: &str) -> (String, String) {
    let tokens: Vec<&str> = id_name.split_whitespace().collect();
    let is_version = |t: &str| {
        let t = t.trim_start_matches('v');
        t.starts_with(|c: char| c.is_ascii_digit()) || t.starts_with("dev-")
    };
    let split = tokens.iter().position(|t| is_version(t)).unwrap_or(tokens.len());
    let family: String = tokens[..split.max(1).min(tokens.len())]
        .iter()
        .map(|t| t.to_ascii_lowercase())
        .collect::<Vec<_>>()
        .join("-");
    let version = tokens
        .get(split)
        .map(|t| t.trim_matches(|c| matches!(c, '(' | ')')).to_string())
        .unwrap_or_default();
    (
        store::sanitize(&family),
        if version.is_empty() { "unknown".to_string() } else { version },
    )
}

fn parse_options(list: &[String]) -> Result<BTreeMap<String, String>, String> {
    let mut map = BTreeMap::new();
    for item in list {
        let (k, v) = item.split_once('=').ok_or_else(|| format!("option '{}' is not NAME=VALUE", item))?;
        map.insert(k.trim().to_string(), v.trim().to_string());
    }
    Ok(map)
}

fn describe_engine(path: &Path, options: &[(String, String)]) -> Result<String, String> {
    let engine = uci::Engine::spawn(path, options, Duration::from_secs(60))?;
    let name = engine.id_name.clone();
    engine.quit();
    if name.is_empty() {
        return Err(format!("{} sent no `id name`", path.display()));
    }
    Ok(name)
}

/// An engine binary with its identity, ready to run.
struct Prepared {
    path: PathBuf,
    engine: store::EngineRecord,
}

/// Resolve a binary path or a registry name into an engine record: sha256
/// of the binary, identity from the UCI id name, options from the registry
/// plus any extras, and the bench signature for rusty-rival.
fn prepare_engine(
    epd_dir: &Path,
    engine_path: Option<&Path>,
    registry_name: Option<&str>,
    extra_options: &[String],
    label: Option<String>,
    family: Option<String>,
) -> Result<Prepared, String> {
    let registry = config::Registry::load(epd_dir)?;
    let (path, mut options, reg_label, reg_family) = match (engine_path, registry_name) {
        (Some(p), None) => (p.to_path_buf(), BTreeMap::new(), None, None),
        (None, Some(n)) => {
            let entry = registry.find(n).ok_or_else(|| format!("no engine named '{}' in engines.toml", n))?;
            (
                entry.resolved_path(),
                entry.options.clone(),
                Some(entry.name.clone()),
                entry.family.clone(),
            )
        }
        _ => return Err("give --engine PATH or --name REGISTRY_NAME".to_string()),
    };
    options.extend(parse_options(extra_options)?);
    if !path.is_file() {
        return Err(format!("{} is not a file", path.display()));
    }
    let bytes = std::fs::read(&path).map_err(|e| format!("cannot read {}: {}", path.display(), e))?;
    let sha256 = epd::sha256_hex(&bytes);
    let option_pairs: Vec<(String, String)> = options.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    let id_name = describe_engine(&path, &option_pairs)?;
    let (family_guess, version) = identity_from_id_name(&id_name);
    let family = family.or(reg_family).unwrap_or(family_guess);
    let label = label.or(reg_label).unwrap_or_else(|| version.clone());
    let bench = if id_name.starts_with("Rusty Rival") {
        uci::bench_signature(&path, &option_pairs)
    } else {
        None
    };
    Ok(Prepared {
        path,
        engine: store::EngineRecord {
            name: id_name,
            family,
            label,
            version,
            sha256,
            options,
            bench,
        },
    })
}

/// Run (or fetch from the cache) every suite for a prepared engine,
/// reporting each suite as it completes.
fn run_engine(
    epd_dir: &Path,
    prepared: &Prepared,
    limit: uci::Limit,
    settings: &EngineSettings,
    suites: &[epd::Suite],
    quiet: bool,
    mut on_suite: impl FnMut(&epd::Suite, &runner::RunOutcome),
) -> Result<Vec<runner::RunOutcome>, String> {
    settings.guard_busy(limit)?;
    let concurrency = settings.concurrency_for(limit);
    if !quiet {
        eprintln!(
            "{} [{} {}] sha {} · {} · threads {} hash {} · concurrency {}",
            prepared.engine.name,
            prepared.engine.family,
            prepared.engine.label,
            prepared.engine.sha8(),
            limit.describe(),
            settings.threads,
            settings.hash,
            concurrency
        );
    }
    let spec = runner::RunSpec {
        epd_dir: epd_dir.to_path_buf(),
        engine_path: prepared.path.clone(),
        engine: prepared.engine.clone(),
        limit,
        threads: settings.threads,
        hash_mb: settings.hash,
        concurrency,
        force: settings.force,
        quiet,
    };
    let mut outcomes = Vec::new();
    for suite in suites {
        let outcome = runner::run_suite(&spec, suite)?;
        on_suite(suite, &outcome);
        outcomes.push(outcome);
    }
    Ok(outcomes)
}

fn summary_line(suite: &str, outcome: &runner::RunOutcome) -> String {
    let s = &outcome.record.summary;
    let score = match (s.points, s.max_points) {
        (Some(p), Some(m)) => format!("{}/{} pts ({} of {} solved)", p, m, s.solved, s.total),
        _ => format!("{}/{} solved", s.solved, s.total),
    };
    format!(
        "{:<14} {}{}{}\n",
        suite,
        score,
        s.median_solve_nodes
            .map(|n| format!(" · median solve {} nodes", n))
            .unwrap_or_default(),
        if outcome.cached { " (cached)" } else { "" }
    )
}

fn cmd_run(epd_dir: &Path, args: RunArgs) -> Result<(), String> {
    let limit = args.budget.limit()?;
    args.settings.guard_busy(limit)?;
    let prepared = prepare_engine(
        epd_dir,
        args.engine.as_deref(),
        args.name.as_deref(),
        &args.options,
        args.label.clone(),
        args.family.clone(),
    )?;
    let suites = load_suites(epd_dir, &args.suites)?;
    let json = args.json;
    let outcomes = run_engine(epd_dir, &prepared, limit, &args.settings, &suites, json, |suite, outcome| {
        if !json {
            out(&summary_line(&suite.name, outcome));
        }
    })?;
    if json {
        let records: Vec<&store::RunRecord> = outcomes.iter().map(|o| &o.record).collect();
        out(&format!("{}\n", serde_json::to_string_pretty(&records).map_err(|e| e.to_string())?));
    }
    Ok(())
}

fn table_key(limit: uci::Limit, threads: u32, hash: u32, concurrency: Option<usize>, cpu: Option<&str>) -> table::TableKey {
    let time_mode = limit.mode() == "time";
    table::TableKey {
        mode: limit.mode().to_string(),
        budget: limit.budget(),
        threads,
        hash_mb: hash,
        concurrency: if time_mode { Some(concurrency.unwrap_or(1)) } else { None },
        cpu: if time_mode {
            match cpu {
                Some("any") => None,
                Some(cpu) => Some(cpu.to_string()),
                None => Some(host::cpu_model()),
            }
        } else {
            None
        },
    }
}

fn cmd_table(epd_dir: &Path, args: TableArgs) -> Result<(), String> {
    let limit = args.budget.limit()?;
    let files = store::load_all(epd_dir)?;
    let engines = args.engines.as_deref().map(split_list);
    let suites = args.suites.as_deref().map(split_list);
    let key = table_key(limit, args.threads, args.hash, args.concurrency, args.cpu.as_deref());
    let table = table::build(&files, &key, engines.as_deref(), suites.as_deref());
    if args.json {
        out(&format!("{}\n", serde_json::to_string_pretty(&table).map_err(|e| e.to_string())?));
    } else {
        out(&table.render(args.percent));
    }
    Ok(())
}

/// One side of a comparison: the engine's identity and its runs for the
/// requested suites at the key, from a binary (run or cached), a results
/// file, or the store.
struct Resolved {
    label: String,
    runs: Vec<store::RunRecord>,
}

/// Prepare and run an engine (a binary path or a registry name) over the
/// suites, returning its label and records.
#[allow(clippy::too_many_arguments)]
fn run_side(
    epd_dir: &Path,
    engine_path: Option<&Path>,
    registry_name: Option<&str>,
    options: &[String],
    limit: uci::Limit,
    settings: &EngineSettings,
    suites: &[epd::Suite],
    quiet: bool,
) -> Result<Resolved, String> {
    settings.guard_busy(limit)?;
    let prepared = prepare_engine(epd_dir, engine_path, registry_name, options, None, None)?;
    let outcomes = run_engine(epd_dir, &prepared, limit, settings, suites, quiet, |suite, outcome| {
        if !quiet {
            out(&summary_line(&suite.name, outcome));
        }
    })?;
    Ok(Resolved {
        label: format!("{} {} ({})", prepared.engine.family, prepared.engine.label, prepared.engine.sha8()),
        runs: outcomes.into_iter().map(|o| o.record).collect(),
    })
}

fn resolve_side(
    epd_dir: &Path,
    spec: &str,
    options: &[String],
    limit: uci::Limit,
    settings: &EngineSettings,
    suites: &[epd::Suite],
    quiet: bool,
) -> Result<Resolved, String> {
    let path = Path::new(spec);
    // Stored runs must match this machine's CPU in time mode unless --cpu any.
    let key = table_key(
        limit,
        settings.threads,
        settings.hash,
        settings.concurrency,
        settings.cpu.as_deref(),
    );
    let file = if let Some(name) = spec.strip_prefix('@') {
        return run_side(epd_dir, None, Some(name), options, limit, settings, suites, quiet);
    } else if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("json") {
        store::load_file(path)?
    } else if path.is_file() {
        return run_side(epd_dir, Some(path), None, options, limit, settings, suites, quiet);
    } else {
        let files = store::load_all(epd_dir)?;
        let matches: Vec<store::ResultsFile> = files
            .into_iter()
            .filter(|f| table::engine_matches(spec, &table::column_of(&f.engine)))
            .collect();
        match matches.len() {
            1 => matches.into_iter().next().unwrap_or_else(|| unreachable!()),
            0 => return Err(format!("'{}' is neither a file nor an engine in the store", spec)),
            n => {
                let names: Vec<String> = matches.iter().map(|f| table::column_of(&f.engine).identity()).collect();
                return Err(format!(
                    "'{}' matches {} engines in the store ({}); add #hash",
                    spec,
                    n,
                    names.join(", ")
                ));
            }
        }
    };
    let mut runs = Vec::new();
    for suite in suites {
        let run = file
            .runs
            .iter()
            .filter(|r| key.matches(r) && r.suite.name == suite.name && r.suite.sha256 == suite.sha256)
            .max_by(|a, b| a.date.cmp(&b.date))
            .ok_or_else(|| {
                format!(
                    "{} {} has no run of {} ({}) at {}",
                    file.engine.family,
                    file.engine.label,
                    suite.name,
                    &suite.sha256[..8],
                    limit.describe()
                )
            })?;
        runs.push(run.clone());
    }
    Ok(Resolved {
        label: format!("{} {} ({})", file.engine.family, file.engine.label, file.engine.sha8()),
        runs,
    })
}

fn cmd_diff(epd_dir: &Path, args: DiffArgs) -> Result<(), String> {
    let limit = args.budget.limit()?;
    let suites = load_suites(epd_dir, &args.suites)?;
    let a = resolve_side(epd_dir, &args.a, &args.options, limit, &args.settings, &suites, args.json)?;
    let b = resolve_side(epd_dir, &args.b, &args.options, limit, &args.settings, &suites, args.json)?;
    let mut diffs = Vec::new();
    for (ra, rb) in a.runs.iter().zip(&b.runs) {
        diffs.push(compare::diff_runs(ra, rb)?);
    }
    if args.json {
        out(&format!("{}\n", serde_json::to_string_pretty(&diffs).map_err(|e| e.to_string())?));
    } else {
        out(&format!("A = {}\nB = {}\n{}\n", a.label, b.label, limit.describe()));
        for d in &diffs {
            out(&compare::render_diff(d, "A", "B"));
        }
    }
    Ok(())
}

fn cmd_check(epd_dir: &Path, args: CheckArgs) -> Result<(), String> {
    let limit = args.budget.limit()?;
    let suites = load_suites(epd_dir, &args.suites)?;
    let baseline = resolve_side(epd_dir, &args.baseline, &args.options, limit, &args.settings, &suites, args.json)?;
    // The candidate is always run (or served from the cache), never looked
    // up by name in the store: a binary is a binary whatever it is called.
    let candidate = run_side(
        epd_dir,
        args.engine.as_deref(),
        args.name.as_deref(),
        &args.options,
        limit,
        &args.settings,
        &suites,
        args.json,
    )?;
    let mut diffs = Vec::new();
    let mut verdicts = Vec::new();
    for (rb, rc) in baseline.runs.iter().zip(&candidate.runs) {
        let d = compare::diff_runs(rb, rc)?;
        verdicts.push(compare::check_suite(&d, args.max_drop, args.exact));
        diffs.push(d);
    }
    let all_ok = verdicts.iter().all(|v| v.ok);
    if args.json {
        let report = serde_json::json!({ "baseline": baseline.label, "candidate": candidate.label, "budget": limit.describe(), "ok": all_ok, "verdicts": verdicts, "diffs": diffs });
        out(&format!("{}\n", serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?));
    } else {
        out(&format!(
            "baseline  = {}\ncandidate = {}\n{} · max drop {}{}\n\n",
            baseline.label,
            candidate.label,
            limit.describe(),
            args.max_drop,
            if args.exact { " · exact" } else { "" }
        ));
        for (d, v) in diffs.iter().zip(&verdicts) {
            out(&compare::render_diff(d, "baseline", "candidate"));
            out(&format!(
                "  => {}{}\n",
                if v.ok { "ok" } else { "FAIL" },
                v.reason.as_ref().map(|r| format!(": {}", r)).unwrap_or_default()
            ));
        }
        out(&format!("\n{}\n", if all_ok { "check passed" } else { "check FAILED" }));
    }
    if all_ok {
        Ok(())
    } else {
        std::process::exit(1)
    }
}

fn cmd_suites(epd_dir: &Path) -> Result<(), String> {
    for suite in load_suites(epd_dir, "all")? {
        let graded = if suite.is_graded() { " (graded)" } else { "" };
        out(&format!(
            "{:<14} {:>5} positions  sha {}{}\n",
            suite.name,
            suite.positions.len(),
            &suite.sha256[..8],
            graded
        ));
    }
    Ok(())
}

fn main() {
    let cli = Cli::parse();
    let epd_dir = cli.epd_dir.unwrap_or_else(default_epd_dir);
    let result = match cli.command {
        Command::Run(args) => cmd_run(&epd_dir, args),
        Command::Table(args) => cmd_table(&epd_dir, args),
        Command::Diff(args) => cmd_diff(&epd_dir, args),
        Command::Check(args) => cmd_check(&epd_dir, args),
        Command::Suites => cmd_suites(&epd_dir),
        Command::Tui => tui::run(&epd_dir),
    };
    if let Err(e) = result {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_from_id_names() {
        assert_eq!(identity_from_id_name("Rusty Rival 1.0.64"), ("rusty-rival".into(), "1.0.64".into()));
        assert_eq!(
            identity_from_id_name("Stockfish dev-20260726-23cf5d82"),
            ("stockfish".into(), "dev-20260726-23cf5d82".into())
        );
        assert_eq!(identity_from_id_name("Ethereal 14.40 (PEXT)"), ("ethereal".into(), "14.40".into()));
        assert_eq!(identity_from_id_name("Stash v37.25"), ("stash".into(), "v37.25".into()));
        assert_eq!(identity_from_id_name("Berserk 20260524"), ("berserk".into(), "20260524".into()));
        assert_eq!(identity_from_id_name("Obsidian dev-16.15"), ("obsidian".into(), "dev-16.15".into()));
        assert_eq!(identity_from_id_name("Weird Engine"), ("weird-engine".into(), "unknown".into()));
    }
}
