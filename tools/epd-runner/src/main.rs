//! epd-runner: run EPD test suites against UCI engines, keep every result in
//! a per-binary JSON store, and show suites × engines tables.

mod config;
mod epd;
mod host;
mod runner;
mod san;
mod store;
mod table;
mod uci;

use clap::{Args, Parser, Subcommand};
use std::collections::BTreeMap;
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
    /// List the suites and their position counts.
    Suites,
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
    /// Comma-separated engine selectors: `family:label`, family, label or version.
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

fn cmd_run(epd_dir: &Path, args: RunArgs) -> Result<(), String> {
    let limit = args.budget.limit()?;
    let registry = config::Registry::load(epd_dir)?;
    let (path, mut options, registry_name, registry_family) = match (&args.engine, &args.name) {
        (Some(p), None) => (p.clone(), BTreeMap::new(), None, None),
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
    options.extend(parse_options(&args.options)?);
    if !path.is_file() {
        return Err(format!("{} is not a file", path.display()));
    }
    if matches!(limit, uci::Limit::MoveTime(_)) && !args.allow_busy {
        if let Some(reason) = host::busy_reason(4.0) {
            return Err(format!(
                "time mode needs an idle machine: {} (use --allow-busy to override)",
                reason
            ));
        }
    }
    let bytes = std::fs::read(&path).map_err(|e| format!("cannot read {}: {}", path.display(), e))?;
    let sha256 = epd::sha256_hex(&bytes);
    let option_pairs: Vec<(String, String)> = options.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    let id_name = describe_engine(&path, &option_pairs)?;
    let (family_guess, version) = identity_from_id_name(&id_name);
    let family = args.family.clone().or(registry_family).unwrap_or(family_guess);
    let label = args.label.clone().or(registry_name).unwrap_or_else(|| version.clone());
    let bench = if id_name.starts_with("Rusty Rival") {
        uci::bench_signature(&path, &option_pairs)
    } else {
        None
    };
    let engine = store::EngineRecord {
        name: id_name.clone(),
        family: family.clone(),
        label: label.clone(),
        version,
        sha256,
        options,
        bench,
    };
    let concurrency = args.concurrency.unwrap_or_else(|| match limit {
        uci::Limit::MoveTime(_) => 1,
        _ => std::thread::available_parallelism().map(|n| (n.get() / 2).max(1)).unwrap_or(1),
    });
    let suites = load_suites(epd_dir, &args.suites)?;
    if !args.json {
        eprintln!(
            "{} [{} {}] sha {} · {} · threads {} hash {} · concurrency {}",
            id_name,
            family,
            label,
            engine.sha8(),
            limit.describe(),
            args.threads,
            args.hash,
            concurrency
        );
    }
    let spec = runner::RunSpec {
        epd_dir: epd_dir.to_path_buf(),
        engine_path: path,
        engine,
        limit,
        threads: args.threads,
        hash_mb: args.hash,
        concurrency,
        force: args.force,
        quiet: args.json,
    };
    let mut records = Vec::new();
    for suite in &suites {
        let outcome = runner::run_suite(&spec, suite)?;
        if !args.json {
            let s = &outcome.record.summary;
            let score = match (s.points, s.max_points) {
                (Some(p), Some(m)) => format!("{}/{} pts ({} of {} solved)", p, m, s.solved, s.total),
                _ => format!("{}/{} solved", s.solved, s.total),
            };
            println!(
                "{:<14} {}{}{}",
                suite.name,
                score,
                s.median_solve_nodes
                    .map(|n| format!(" · median solve {} nodes", n))
                    .unwrap_or_default(),
                if outcome.cached { " (cached)" } else { "" }
            );
        }
        records.push(outcome.record);
    }
    if args.json {
        println!("{}", serde_json::to_string_pretty(&records).map_err(|e| e.to_string())?);
    }
    Ok(())
}

fn cmd_table(epd_dir: &Path, args: TableArgs) -> Result<(), String> {
    let limit = args.budget.limit()?;
    let files = store::load_all(epd_dir)?;
    let engines = args.engines.as_deref().map(split_list);
    let suites = args.suites.as_deref().map(split_list);
    let table = table::build(&files, limit.mode(), limit.budget(), engines.as_deref(), suites.as_deref());
    if args.json {
        println!("{}", serde_json::to_string_pretty(&table).map_err(|e| e.to_string())?);
    } else {
        print!("{}", table.render(args.percent));
    }
    Ok(())
}

fn cmd_suites(epd_dir: &Path) -> Result<(), String> {
    for suite in load_suites(epd_dir, "all")? {
        let graded = if suite.is_graded() { " (graded)" } else { "" };
        println!(
            "{:<14} {:>5} positions  sha {}{}",
            suite.name,
            suite.positions.len(),
            &suite.sha256[..8],
            graded
        );
    }
    Ok(())
}

fn main() {
    let cli = Cli::parse();
    let epd_dir = cli.epd_dir.unwrap_or_else(default_epd_dir);
    let result = match cli.command {
        Command::Run(args) => cmd_run(&epd_dir, args),
        Command::Table(args) => cmd_table(&epd_dir, args),
        Command::Suites => cmd_suites(&epd_dir),
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
