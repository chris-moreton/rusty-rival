//! The terminal view: a suites × engines table over the store, a budget
//! switch, an engine picker that keeps several versions of one family side
//! by side, a per-position suite detail, and on-demand runs for missing
//! cells. Shares the table model with the text `table` command.

use crate::config::Registry;
use crate::epd;
use crate::host;
use crate::store::{self, ResultsFile, RunRecord};
use crate::table::{self, Column, Table, TableKey};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table as TableWidget, Wrap};
use ratatui::Frame;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

/// What the view remembers between sessions. `engines` is absent until the
/// picker has been used, so "no preference" and "none selected" differ;
/// selectors for engines not currently in the store are kept as written.
#[derive(Debug, Serialize, Deserialize)]
struct State {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    engines: Option<Vec<String>>,
    /// Percent cells are the default: fourteen columns of `13164/15000 pts`
    /// do not fit, and percent is what the eye compares anyway.
    #[serde(default = "yes")]
    percent: bool,
    #[serde(default)]
    key: Option<String>,
}

fn yes() -> bool {
    true
}

impl Default for State {
    fn default() -> State {
        State {
            engines: None,
            percent: true,
            key: None,
        }
    }
}

fn state_path(epd_dir: &Path) -> PathBuf {
    epd_dir.join("tui-state.toml")
}

fn load_state(epd_dir: &Path) -> State {
    std::fs::read_to_string(state_path(epd_dir))
        .ok()
        .and_then(|t| toml::from_str(&t).ok())
        .unwrap_or_default()
}

fn save_state(epd_dir: &Path, state: &State) {
    if let Ok(text) = toml::to_string(state) {
        let _ = std::fs::write(state_path(epd_dir), text);
    }
}

/// The identity a column is selected by: `family:label#hash`.
fn selector_of(c: &Column) -> String {
    format!(
        "{}:{}#{}",
        c.family,
        c.label,
        c.options_hash8.clone().unwrap_or_else(|| c.sha8.clone())
    )
}

/// Everything that names one column: family, label and the binary/options
/// identity, so two aliases of one binary never share a cell.
fn column_id(c: &Column) -> String {
    format!("{}:{}:{}", c.family, c.label, c.identity())
}

fn key_id(k: &TableKey) -> String {
    format!(
        "{}:{}:{}:{}:{}:{}",
        k.mode,
        k.budget,
        k.threads,
        k.hash_mb,
        k.concurrency.unwrap_or(0),
        k.cpu.clone().unwrap_or_default()
    )
}

fn mode_rank(m: &str) -> u8 {
    match m {
        "nodes" => 0,
        "depth" => 1,
        _ => 2,
    }
}

/// Every distinct key present in the store, node keys first, then depth,
/// then time, each sorted by budget.
fn keys_in_store(files: &[ResultsFile]) -> Vec<TableKey> {
    let mut seen: BTreeMap<String, TableKey> = BTreeMap::new();
    for f in files {
        for r in &f.runs {
            let key = TableKey {
                mode: r.mode.clone(),
                budget: r.budget,
                threads: r.threads,
                hash_mb: r.hash_mb,
                concurrency: if r.mode == "time" { Some(r.concurrency) } else { None },
                cpu: if r.mode == "time" {
                    r.host.as_ref().map(|h| h.cpu.clone())
                } else {
                    None
                },
            };
            seen.entry(key_id(&key)).or_insert(key);
        }
    }
    let mut keys: Vec<TableKey> = seen.into_values().collect();
    keys.sort_by(|a, b| {
        mode_rank(&a.mode)
            .cmp(&mode_rank(&b.mode))
            .then(a.budget.cmp(&b.budget))
            .then(a.threads.cmp(&b.threads))
            .then(a.hash_mb.cmp(&b.hash_mb))
            .then(a.concurrency.cmp(&b.concurrency))
    });
    keys
}

fn sorted_columns(files: &[ResultsFile]) -> Vec<Column> {
    let mut columns: Vec<Column> = files.iter().map(|f| table::column_of(&f.engine)).collect();
    columns.sort_by(|a, b| {
        a.family
            .cmp(&b.family)
            .then_with(|| table::version_key(&a.label).cmp(&table::version_key(&b.label)))
            .then_with(|| a.identity().cmp(&b.identity()))
    });
    columns
}

/// A detail row: whether the engines disagree, the position label, and
/// one (text, style) per column.
type DetailLine = (bool, String, Vec<(String, Style)>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum View {
    Main,
    Picker,
    Detail,
    Help,
    ConfirmRun,
    ConfirmQuit,
}

/// One cell to fill: which engine (by full column id), which registry entry
/// runs it, which suite, at which key.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Target {
    column_id: String,
    registry_name: String,
    suite: String,
    key_id: String,
    key: TableKey,
}

/// The run in flight and the channel its outcome arrives on. Runs are
/// serialised: one subprocess at a time, so a time-mode run never shares
/// the machine with another one started from here.
struct Job {
    target: Target,
    done: Receiver<Result<(), String>>,
}

pub struct App {
    epd_dir: PathBuf,
    files: Vec<ResultsFile>,
    registry: Registry,
    keys: Vec<TableKey>,
    key_index: usize,
    all_columns: Vec<Column>,
    /// Selectors as saved, including ones for engines not in the store.
    selected: Option<Vec<String>>,
    percent: bool,
    table: Table,
    view: View,
    cursor_row: usize,
    cursor_col: usize,
    col_offset: usize,
    picker_cursor: usize,
    detail_scroll: usize,
    detail_only_disagreements: bool,
    /// Set while a time-mode run awaits confirmation: run the whole column?
    pending_whole_column: bool,
    queue: VecDeque<Target>,
    active: Option<Job>,
    /// Completed runs and failures, newest last.
    events: Vec<String>,
    message: String,
}

impl App {
    pub fn new(epd_dir: &Path) -> Result<App, String> {
        let files = store::load_all(epd_dir)?;
        let registry = Registry::load(epd_dir)?;
        let state = load_state(epd_dir);
        let keys = keys_in_store(&files);
        // Start on the remembered key, else on the key most engines have
        // runs for (ties: the larger budget), so the first screen is full.
        let key_index = state
            .key
            .as_ref()
            .and_then(|id| keys.iter().position(|k| key_id(k) == *id))
            .unwrap_or_else(|| {
                keys.iter()
                    .enumerate()
                    .max_by_key(|(_, k)| (files.iter().filter(|f| f.runs.iter().any(|r| k.matches(r))).count(), k.budget))
                    .map(|(i, _)| i)
                    .unwrap_or(0)
            });
        let all_columns = sorted_columns(&files);
        let mut app = App {
            epd_dir: epd_dir.to_path_buf(),
            files,
            registry,
            keys,
            key_index,
            all_columns,
            selected: state.engines,
            percent: state.percent,
            table: Table {
                key: TableKey {
                    mode: "nodes".into(),
                    budget: 0,
                    threads: 1,
                    hash_mb: 128,
                    concurrency: None,
                    cpu: None,
                },
                budget_label: String::new(),
                columns: vec![],
                rows: vec![],
            },
            view: View::Main,
            cursor_row: 0,
            cursor_col: 0,
            col_offset: 0,
            picker_cursor: 0,
            detail_scroll: 0,
            detail_only_disagreements: false,
            pending_whole_column: false,
            queue: VecDeque::new(),
            active: None,
            events: Vec::new(),
            message: String::new(),
        };
        app.rebuild();
        Ok(app)
    }

    fn current_key(&self) -> Option<&TableKey> {
        self.keys.get(self.key_index)
    }

    fn is_selected(&self, c: &Column) -> bool {
        match &self.selected {
            None => true,
            Some(list) => list.contains(&selector_of(c)),
        }
    }

    fn rebuild(&mut self) {
        let Some(key) = self.current_key().cloned() else {
            self.table.columns.clear();
            self.table.rows.clear();
            self.message = "the store is empty: run `epd-runner run` first".into();
            return;
        };
        let selectors: Vec<String> = self.all_columns.iter().filter(|c| self.is_selected(c)).map(selector_of).collect();
        self.table = table::build(&self.files, &key, Some(&selectors), None);
        self.clamp_cursors();
    }

    fn clamp_cursors(&mut self) {
        self.cursor_row = self.cursor_row.min(self.table.rows.len().saturating_sub(1));
        self.cursor_col = self.cursor_col.min(self.table.columns.len().saturating_sub(1));
        self.col_offset = self.col_offset.min(self.cursor_col);
        self.picker_cursor = self.picker_cursor.min(self.all_columns.len().saturating_sub(1));
    }

    /// Re-read the store and the registry; keep the same key by identity.
    fn reload(&mut self) -> Result<(), String> {
        let keep = self.current_key().map(key_id);
        self.files = store::load_all(&self.epd_dir)?;
        self.registry = Registry::load(&self.epd_dir)?;
        self.keys = keys_in_store(&self.files);
        self.all_columns = sorted_columns(&self.files);
        self.key_index = keep
            .and_then(|id| self.keys.iter().position(|k| key_id(k) == id))
            .unwrap_or(0)
            .min(self.keys.len().saturating_sub(1));
        self.rebuild();
        Ok(())
    }

    fn persist(&self) {
        save_state(
            &self.epd_dir,
            &State {
                engines: self.selected.clone(),
                percent: self.percent,
                key: self.current_key().map(key_id),
            },
        );
    }

    fn cycle_budget(&mut self, forward: bool) {
        if self.keys.is_empty() {
            return;
        }
        let mode = self.current_key().map(|k| k.mode.clone()).unwrap_or_default();
        let same: Vec<usize> = self
            .keys
            .iter()
            .enumerate()
            .filter(|(_, k)| k.mode == mode)
            .map(|(i, _)| i)
            .collect();
        if let Some(pos) = same.iter().position(|i| *i == self.key_index) {
            let next = if forward {
                (pos + 1) % same.len()
            } else {
                (pos + same.len() - 1) % same.len()
            };
            self.key_index = same[next];
            self.rebuild();
            self.persist();
        }
    }

    /// Move to the next mode present in the store (nodes → depth → time → nodes).
    fn toggle_mode(&mut self) {
        let Some(current) = self.current_key().map(|k| k.mode.clone()) else {
            return;
        };
        let mut modes: Vec<String> = self
            .keys
            .iter()
            .map(|k| k.mode.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        modes.sort_by_key(|m| mode_rank(m));
        if modes.len() < 2 {
            self.message = "only one mode in the store".into();
            return;
        }
        let pos = modes.iter().position(|m| *m == current).unwrap_or(0);
        let next = &modes[(pos + 1) % modes.len()];
        if let Some(i) = self.keys.iter().position(|k| k.mode == *next) {
            self.key_index = i;
            self.rebuild();
            self.persist();
        }
    }

    /// The registry entry that reproduces a column exactly (same binary
    /// sha256 and same options), or the reason there is none.
    fn runnable(&self, column: &Column) -> Result<String, String> {
        let file = self
            .files
            .iter()
            .find(|f| column_id(&table::column_of(&f.engine)) == column_id(column))
            .ok_or_else(|| "column has no results file".to_string())?;
        let candidates: Vec<&crate::config::EngineEntry> = self
            .registry
            .engine
            .iter()
            .filter(|e| e.family.as_deref().unwrap_or(&column.family) == column.family && e.options == file.engine.options)
            .collect();
        for entry in candidates {
            let path = entry.resolved_path();
            if let Ok(bytes) = std::fs::read(&path) {
                if epd::sha256_hex(&bytes) == file.engine.sha256 {
                    return Ok(entry.name.clone());
                }
            }
        }
        Err(format!(
            "no engines.toml entry has this binary and options ({} {} {})",
            column.family,
            column.label,
            column.identity()
        ))
    }

    /// Why a cell cannot be reproduced from this machine, if it cannot.
    fn reproducible(&self, key: &TableKey, row: &table::Row) -> Result<(), String> {
        if key.mode == "time" {
            if let Some(cpu) = &key.cpu {
                let here = host::cpu_model();
                if *cpu != here {
                    return Err(format!("this budget's runs are from another CPU ({})", cpu));
                }
            }
        }
        let path = self.epd_dir.join("suites").join(format!("{}.epd", row.suite));
        let bytes = std::fs::read(&path).map_err(|e| format!("cannot read {}: {}", path.display(), e))?;
        if !epd::sha256_hex(&bytes).starts_with(&row.suite_sha8) {
            return Err(format!(
                "{} on disk is another revision than this row ({})",
                row.suite, row.suite_sha8
            ));
        }
        Ok(())
    }

    fn queued_or_active(&self, column_id: &str, suite: &str, key_id: &str) -> bool {
        let same = |t: &Target| t.column_id == column_id && t.suite == suite && t.key_id == key_id;
        self.queue.iter().any(same) || self.active.as_ref().is_some_and(|j| same(&j.target))
    }

    /// Queue `epd-runner run` for the missing cell under the cursor (or
    /// every missing cell in its column when `whole_column`).
    fn queue_runs(&mut self, whole_column: bool) {
        let Some(key) = self.current_key().cloned() else { return };
        let Some(column) = self.table.columns.get(self.cursor_col).cloned() else {
            return;
        };
        let registry_name = match self.runnable(&column) {
            Ok(name) => name,
            Err(e) => {
                self.message = e;
                return;
            }
        };
        let kid = key_id(&key);
        let cid = column_id(&column);
        let mut added = 0;
        let mut skipped = Vec::new();
        for (i, row) in self.table.rows.clone().iter().enumerate() {
            if !(whole_column || i == self.cursor_row) || row.cells[self.cursor_col].is_some() {
                continue;
            }
            if self.queued_or_active(&cid, &row.suite, &kid) {
                continue;
            }
            if let Err(e) = self.reproducible(&key, row) {
                skipped.push(e);
                continue;
            }
            self.queue.push_back(Target {
                column_id: cid.clone(),
                registry_name: registry_name.clone(),
                suite: row.suite.clone(),
                key_id: kid.clone(),
                key: key.clone(),
            });
            added += 1;
        }
        self.message = match (added, skipped.first()) {
            (0, Some(e)) => e.clone(),
            (0, None) => "nothing missing there (or already queued)".into(),
            (n, Some(e)) => format!("queued {} run(s); skipped: {}", n, e),
            (n, None) => format!("queued {} run(s) for {} {}", n, column.family, column.label),
        };
    }

    fn start_next(&mut self) {
        if self.active.is_some() {
            return;
        }
        let Some(target) = self.queue.pop_front() else { return };
        let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("epd-runner"));
        let mut cmd = std::process::Command::new(exe);
        cmd.arg("--epd-dir")
            .arg(&self.epd_dir)
            .arg("run")
            .arg("--name")
            .arg(&target.registry_name)
            .arg("--suites")
            .arg(&target.suite);
        let key = &target.key;
        match key.mode.as_str() {
            "nodes" => {
                cmd.arg("--nodes").arg(key.budget.to_string());
            }
            "time" => {
                cmd.arg("--time").arg(format!("{}", key.budget as f64 / 1000.0));
                if let Some(c) = key.concurrency {
                    cmd.arg("--concurrency").arg(c.to_string());
                }
            }
            _ => {
                cmd.arg("--depth").arg(key.budget.to_string());
            }
        }
        cmd.arg("--threads")
            .arg(key.threads.to_string())
            .arg("--hash")
            .arg(key.hash_mb.to_string())
            .arg("--json")
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped());
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let result = match cmd.output() {
                Ok(out) if out.status.success() => Ok(()),
                Ok(out) => Err(String::from_utf8_lossy(&out.stderr)
                    .trim()
                    .lines()
                    .last()
                    .unwrap_or("failed")
                    .to_string()),
                Err(e) => Err(e.to_string()),
            };
            let _ = tx.send(result);
        });
        self.message = format!("running {} on {} ({})", target.registry_name, target.suite, key.describe());
        self.active = Some(Job { target, done: rx });
    }

    /// Collect the finished run, if any; reload the store when one finished.
    fn poll_jobs(&mut self) {
        let outcome = match &self.active {
            Some(job) => match job.done.try_recv() {
                Ok(result) => Some(result),
                Err(mpsc::TryRecvError::Disconnected) => Some(Err("run thread died".into())),
                Err(mpsc::TryRecvError::Empty) => None,
            },
            None => None,
        };
        if let Some(result) = outcome {
            let job = self.active.take().expect("active job present");
            let line = match result {
                Ok(()) => format!("{} on {} done", job.target.registry_name, job.target.suite),
                Err(e) => format!("{} on {} FAILED: {}", job.target.registry_name, job.target.suite, e),
            };
            self.events.push(line);
            let recent: Vec<&str> = self.events.iter().rev().take(2).map(|s| s.as_str()).collect();
            self.message = recent.into_iter().rev().collect::<Vec<_>>().join(" · ");
            if let Err(e) = self.reload() {
                self.message = e;
            }
        }
        self.start_next();
    }

    fn running_here(&self, row: usize, col: usize) -> Option<&'static str> {
        let (Some(r), Some(c), Some(k)) = (self.table.rows.get(row), self.table.columns.get(col), self.current_key()) else {
            return None;
        };
        let (cid, kid) = (column_id(c), key_id(k));
        let same = |t: &Target| t.column_id == cid && t.suite == r.suite && t.key_id == kid;
        if self.active.as_ref().is_some_and(|j| same(&j.target)) {
            Some("running…")
        } else if self.queue.iter().any(same) {
            Some("queued")
        } else {
            None
        }
    }

    /// The stored run behind a cell, for the detail view.
    fn run_for(&self, column: &Column, suite: &str, sha8: &str) -> Option<&RunRecord> {
        let key = self.current_key()?;
        self.files
            .iter()
            .find(|f| column_id(&table::column_of(&f.engine)) == column_id(column))
            .and_then(|f| {
                f.runs
                    .iter()
                    .filter(|r| key.matches(r) && r.suite.name == suite && r.suite.sha256.starts_with(sha8))
                    .max_by(|a, b| a.date.cmp(&b.date))
            })
    }

    fn detail_rows(&self) -> usize {
        let Some(row) = self.table.rows.get(self.cursor_row) else {
            return 0;
        };
        let first = self.table.columns.iter().find_map(|c| self.run_for(c, &row.suite, &row.suite_sha8));
        match (first, self.detail_only_disagreements) {
            (Some(r), false) => r.positions.len(),
            (Some(_), true) => self.detail_lines().len(),
            (None, _) => 0,
        }
    }

    /// Returns false to quit.
    fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        if code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL) {
            return false;
        }
        match self.view {
            View::Main => match code {
                KeyCode::Char('q') => {
                    if self.active.is_some() || !self.queue.is_empty() {
                        self.message = format!(
                            "{} run(s) active or queued; they finish in the background if you quit. quit? y/n",
                            self.queue.len() + usize::from(self.active.is_some())
                        );
                        self.view = View::ConfirmQuit;
                    } else {
                        return false;
                    }
                }
                KeyCode::Char('?') => self.view = View::Help,
                KeyCode::Char('b') => self.cycle_budget(true),
                KeyCode::Char('B') => self.cycle_budget(false),
                KeyCode::Char('m') => self.toggle_mode(),
                KeyCode::Char('p') => {
                    self.percent = !self.percent;
                    self.persist();
                }
                KeyCode::Char('e') => {
                    self.picker_cursor = 0;
                    self.view = View::Picker;
                }
                KeyCode::Char('r') | KeyCode::Char('R') => {
                    let whole = code == KeyCode::Char('R');
                    if self.current_key().is_some_and(|k| k.mode == "time") {
                        self.message = if whole {
                            "queue every missing cell in this column (time mode, one at a time)? y/n".into()
                        } else {
                            "queue this cell in time mode? y/n".into()
                        };
                        self.pending_whole_column = whole;
                        self.view = View::ConfirmRun;
                    } else {
                        self.queue_runs(whole);
                    }
                }
                KeyCode::Char('l') => match self.reload() {
                    Ok(()) => self.message = "store reloaded".into(),
                    Err(e) => self.message = e,
                },
                KeyCode::Enter => {
                    if !self.table.rows.is_empty() {
                        self.detail_scroll = 0;
                        self.view = View::Detail;
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => self.cursor_row = self.cursor_row.saturating_sub(1),
                KeyCode::Down | KeyCode::Char('j') => self.cursor_row = (self.cursor_row + 1).min(self.table.rows.len().saturating_sub(1)),
                KeyCode::Left | KeyCode::Char('h') => self.cursor_col = self.cursor_col.saturating_sub(1),
                KeyCode::Right => self.cursor_col = (self.cursor_col + 1).min(self.table.columns.len().saturating_sub(1)),
                KeyCode::Home => self.cursor_col = 0,
                KeyCode::End => self.cursor_col = self.table.columns.len().saturating_sub(1),
                _ => {}
            },
            View::ConfirmRun => {
                let whole = self.pending_whole_column;
                self.pending_whole_column = false;
                self.view = View::Main;
                if matches!(code, KeyCode::Char('y') | KeyCode::Char('Y')) {
                    self.queue_runs(whole);
                } else {
                    self.message = "cancelled".into();
                }
            }
            View::ConfirmQuit => {
                if matches!(code, KeyCode::Char('y') | KeyCode::Char('Y')) {
                    return false;
                }
                self.message = "staying".into();
                self.view = View::Main;
            }
            View::Picker => match code {
                KeyCode::Esc | KeyCode::Enter | KeyCode::Char('e') | KeyCode::Char('q') => {
                    self.rebuild();
                    self.persist();
                    self.view = View::Main;
                }
                KeyCode::Up | KeyCode::Char('k') => self.picker_cursor = self.picker_cursor.saturating_sub(1),
                KeyCode::Down | KeyCode::Char('j') => {
                    self.picker_cursor = (self.picker_cursor + 1).min(self.all_columns.len().saturating_sub(1))
                }
                KeyCode::Char(' ') => {
                    if let Some(c) = self.all_columns.get(self.picker_cursor) {
                        let sel = selector_of(c);
                        let mut list = self
                            .selected
                            .clone()
                            .unwrap_or_else(|| self.all_columns.iter().map(selector_of).collect());
                        if let Some(i) = list.iter().position(|s| *s == sel) {
                            list.remove(i);
                        } else {
                            list.push(sel);
                        }
                        self.selected = Some(list);
                    }
                }
                KeyCode::Char('a') => self.selected = Some(self.all_columns.iter().map(selector_of).collect()),
                KeyCode::Char('n') => self.selected = Some(Vec::new()),
                KeyCode::Char('f') => {
                    // Toggle the whole family under the cursor.
                    if let Some(c) = self.all_columns.get(self.picker_cursor) {
                        let family = c.family.clone();
                        let members: Vec<String> = self.all_columns.iter().filter(|x| x.family == family).map(selector_of).collect();
                        let mut list = self
                            .selected
                            .clone()
                            .unwrap_or_else(|| self.all_columns.iter().map(selector_of).collect());
                        let all_on = members.iter().all(|m| list.contains(m));
                        if all_on {
                            list.retain(|s| !members.contains(s));
                        } else {
                            for m in members {
                                if !list.contains(&m) {
                                    list.push(m);
                                }
                            }
                        }
                        self.selected = Some(list);
                    }
                }
                _ => {}
            },
            View::Detail => {
                let max = self.detail_rows().saturating_sub(1);
                match code {
                    KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => self.view = View::Main,
                    KeyCode::Char('d') => {
                        self.detail_only_disagreements = !self.detail_only_disagreements;
                        self.detail_scroll = 0;
                    }
                    KeyCode::Up | KeyCode::Char('k') => self.detail_scroll = self.detail_scroll.saturating_sub(1),
                    KeyCode::Down | KeyCode::Char('j') => self.detail_scroll = (self.detail_scroll + 1).min(max),
                    KeyCode::PageUp => self.detail_scroll = self.detail_scroll.saturating_sub(20),
                    KeyCode::PageDown => self.detail_scroll = (self.detail_scroll + 20).min(max),
                    KeyCode::Home => self.detail_scroll = 0,
                    KeyCode::End => self.detail_scroll = max,
                    KeyCode::Left | KeyCode::Char('h') => self.cursor_col = self.cursor_col.saturating_sub(1),
                    KeyCode::Right => self.cursor_col = (self.cursor_col + 1).min(self.table.columns.len().saturating_sub(1)),
                    _ => {}
                }
            }
            View::Help => self.view = View::Main,
        }
        true
    }

    /// Columns that fit beside the first column at this width, starting at
    /// `col_offset`, which follows the cursor.
    fn visible_columns(&mut self, inner: u16, first: u16) -> (usize, usize, u16) {
        let n = self.table.columns.len();
        if n == 0 {
            return (0, 0, 10);
        }
        let col_width: u16 = 13;
        let fit = ((inner.saturating_sub(first + 1)) / (col_width + 1)).max(1) as usize;
        if self.cursor_col < self.col_offset {
            self.col_offset = self.cursor_col;
        } else if self.cursor_col >= self.col_offset + fit {
            self.col_offset = self.cursor_col + 1 - fit;
        }
        let end = (self.col_offset + fit).min(n);
        // Share any spare width among the shown columns.
        let shown = (end - self.col_offset).max(1) as u16;
        let width = ((inner.saturating_sub(first + shown)) / shown).clamp(col_width, 24);
        (self.col_offset, end, width)
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let [body, footer] = Layout::vertical([Constraint::Min(3), Constraint::Length(2)]).areas(area);
        match self.view {
            View::Main | View::ConfirmRun | View::ConfirmQuit | View::Help => self.draw_main(frame, body),
            View::Picker => self.draw_picker(frame, body),
            View::Detail => self.draw_detail(frame, body),
        }
        let hint = match self.view {
            View::Main => "↑↓←→ move · Enter suite · b/B budget · m mode · p percent · e engines · r run cell · R run column · l reload · ? help · q quit",
            View::ConfirmRun | View::ConfirmQuit => "y to confirm · any other key cancels",
            View::Picker => "space toggle · f family · a all · n none · Enter/Esc back",
            View::Detail => "↑↓ PgUp PgDn scroll · ←→ columns · d disagreements only · Esc back",
            View::Help => "any key to return",
        };
        let footer_text = vec![
            Line::from(Span::styled(hint, Style::default().fg(Color::DarkGray))),
            Line::from(Span::styled(self.message.clone(), Style::default().fg(Color::Yellow))),
        ];
        frame.render_widget(Paragraph::new(footer_text), footer);
        if self.view == View::Help {
            self.draw_help(frame, area);
        }
    }

    fn cell_style(row: &table::Row, col: usize) -> Style {
        let Some(cell) = &row.cells[col] else {
            return Style::default().fg(Color::DarkGray);
        };
        let values: Vec<f64> = row.cells.iter().flatten().map(|c| c.percent).collect();
        let max = values.iter().cloned().fold(f64::MIN, f64::max);
        let min = values.iter().cloned().fold(f64::MAX, f64::min);
        let mut style = Style::default();
        if values.len() > 1 && max > min {
            if cell.percent >= max {
                style = style.fg(Color::Green);
            } else if cell.percent <= min {
                style = style.fg(Color::Red);
            }
        }
        style
    }

    fn draw_main(&mut self, frame: &mut Frame, area: Rect) {
        let Some(key) = self.current_key().cloned() else {
            frame.render_widget(
                Paragraph::new("The store is empty: run `epd-runner run --engine ... --nodes N` first."),
                area,
            );
            return;
        };
        let inner = area.width.saturating_sub(2);
        let suite_width = 22u16.min(inner / 3);
        let (start, end, col_width) = self.visible_columns(inner, suite_width);
        let more = if end < self.table.columns.len() || start > 0 {
            format!(" · columns {}-{} of {} ", start + 1, end, self.table.columns.len())
        } else {
            String::new()
        };
        let title = format!(" epd-runner · {} · {} engines{} ", key.describe(), self.table.columns.len(), more);
        let mut header_cells = vec![Cell::from("suite")];
        for (i, c) in self.table.columns.iter().enumerate().take(end).skip(start) {
            let text = format!("{}\n{}\n{}", c.family, c.label, c.identity());
            let mut style = Style::default().add_modifier(Modifier::BOLD);
            if i == self.cursor_col {
                style = style.fg(Color::Cyan);
            }
            header_cells.push(Cell::from(text).style(style));
        }
        let header = Row::new(header_cells).height(3);
        let rows: Vec<Row> = self
            .table
            .rows
            .iter()
            .enumerate()
            .map(|(ri, row)| {
                let revision = if row.revision_shown {
                    format!(" [{}]", row.suite_sha8)
                } else {
                    String::new()
                };
                let mut cells = vec![Cell::from(format!("{} ({}){}", row.suite, row.positions, revision))];
                for (ci, cell) in row.cells.iter().enumerate().take(end).skip(start) {
                    let text = match self.running_here(ri, ci) {
                        Some(state) => state.to_string(),
                        None => Table::cell_text(cell, self.percent),
                    };
                    let mut style = Self::cell_style(row, ci);
                    if ri == self.cursor_row && ci == self.cursor_col {
                        style = style.add_modifier(Modifier::REVERSED);
                    }
                    cells.push(Cell::from(text).style(style));
                }
                let mut row_widget = Row::new(cells);
                if ri == self.cursor_row {
                    row_widget = row_widget.style(Style::default().add_modifier(Modifier::BOLD));
                }
                row_widget
            })
            .collect();
        let mut widths = vec![Constraint::Length(suite_width)];
        widths.extend((start..end).map(|_| Constraint::Length(col_width)));
        let widget = TableWidget::new(rows, widths)
            .header(header)
            .column_spacing(1)
            .block(Block::default().borders(Borders::ALL).title(title));
        frame.render_widget(widget, area);
    }

    fn draw_picker(&self, frame: &mut Frame, area: Rect) {
        let visible = area.height.saturating_sub(3) as usize;
        let start = self.picker_cursor.saturating_sub(visible.saturating_sub(1));
        let rows: Vec<Row> = self
            .all_columns
            .iter()
            .enumerate()
            .skip(start)
            .take(visible.max(1))
            .map(|(i, c)| {
                let mark = if self.is_selected(c) { "[x]" } else { "[ ]" };
                let mut style = Style::default();
                if i == self.picker_cursor {
                    style = style.add_modifier(Modifier::REVERSED);
                }
                let options = c.options.iter().map(|(k, v)| format!("{}={}", k, v)).collect::<Vec<_>>().join(" ");
                let runnable = match self.runnable(c) {
                    Ok(name) => format!("runs as {}", name),
                    Err(_) => "not in engines.toml".to_string(),
                };
                Row::new(vec![
                    Cell::from(mark),
                    Cell::from(c.family.clone()),
                    Cell::from(c.label.clone()),
                    Cell::from(c.identity()),
                    Cell::from(options),
                    Cell::from(runnable),
                ])
                .style(style)
            })
            .collect();
        let selected = self.all_columns.iter().filter(|c| self.is_selected(c)).count();
        let widget = TableWidget::new(
            rows,
            [
                Constraint::Length(3),
                Constraint::Length(14),
                Constraint::Length(24),
                Constraint::Length(18),
                Constraint::Min(10),
                Constraint::Length(24),
            ],
        )
        .header(Row::new(vec!["", "family", "label", "identity", "options", ""]).style(Style::default().add_modifier(Modifier::BOLD)))
        .block(Block::default().borders(Borders::ALL).title(format!(
            " engines ({} of {} selected) ",
            selected,
            self.all_columns.len()
        )));
        frame.render_widget(widget, area);
    }

    /// One detail line per position: the id with its best moves, and per
    /// column the engine's move, solve point and points, or a marker.
    fn detail_lines(&self) -> Vec<DetailLine> {
        let Some(row) = self.table.rows.get(self.cursor_row) else {
            return Vec::new();
        };
        let runs: Vec<Option<&RunRecord>> = self
            .table
            .columns
            .iter()
            .map(|c| self.run_for(c, &row.suite, &row.suite_sha8))
            .collect();
        let ids: Vec<(String, Vec<String>)> = runs
            .iter()
            .flatten()
            .next()
            .map(|r| r.positions.iter().map(|p| (p.id.clone(), p.bm.clone())).collect())
            .unwrap_or_default();
        let lookup: Vec<BTreeMap<&str, &store::PositionRecord>> = runs
            .iter()
            .map(|r| {
                r.map(|r| r.positions.iter().map(|p| (p.id.as_str(), p)).collect())
                    .unwrap_or_default()
            })
            .collect();
        let mut lines = Vec::new();
        for (id, bm) in &ids {
            let entries: Vec<Option<&store::PositionRecord>> = lookup.iter().map(|m| m.get(id.as_str()).copied()).collect();
            let known: Vec<bool> = entries.iter().flatten().filter(|p| p.error.is_none()).map(|p| p.solved).collect();
            let disagree = known.iter().any(|s| *s) && known.iter().any(|s| !*s);
            if self.detail_only_disagreements && !disagree {
                continue;
            }
            let cells: Vec<(String, Style)> = entries
                .iter()
                .map(|e| match e {
                    None => ("—".to_string(), Style::default().fg(Color::DarkGray)),
                    Some(p) if p.error.is_some() => ("error".to_string(), Style::default().fg(Color::Magenta)),
                    Some(p) => {
                        let at = p
                            .solved_at
                            .as_ref()
                            .map(|s| format!(" d{} {}k", s.depth, s.nodes / 1000))
                            .unwrap_or_default();
                        let pts = p.points.map(|v| format!(" {}pt", v)).unwrap_or_default();
                        (
                            format!("{}{}{}{}", if p.solved { "✓ " } else { "✗ " }, p.best, at, pts),
                            Style::default().fg(if p.solved { Color::Green } else { Color::Red }),
                        )
                    }
                })
                .collect();
            lines.push((disagree, format!("{} {}", id, bm.join("/")), cells));
        }
        lines
    }

    fn draw_detail(&mut self, frame: &mut Frame, area: Rect) {
        let Some(row) = self.table.rows.get(self.cursor_row).cloned() else {
            return;
        };
        let lines = self.detail_lines();
        let total = lines.len();
        let visible = area.height.saturating_sub(4) as usize;
        let start_row = self.detail_scroll.min(total.saturating_sub(visible));
        let inner = area.width.saturating_sub(2);
        let id_width = 30u16.min(inner / 3);
        let (start, end, col_width) = self.visible_columns(inner, id_width);
        let shown: Vec<Row> = lines
            .into_iter()
            .skip(start_row)
            .take(visible)
            .map(|(disagree, label, cells)| {
                let mut widgets = vec![Cell::from(label)];
                for (text, style) in cells.into_iter().take(end).skip(start) {
                    widgets.push(Cell::from(text).style(style));
                }
                let mut r = Row::new(widgets);
                if disagree {
                    r = r.style(Style::default().add_modifier(Modifier::BOLD));
                }
                r
            })
            .collect();
        let mut header_cells = vec![Cell::from("position / bm")];
        for (i, c) in self.table.columns.iter().enumerate().take(end).skip(start) {
            let mut style = Style::default().add_modifier(Modifier::BOLD);
            if i == self.cursor_col {
                style = style.fg(Color::Cyan);
            }
            header_cells.push(Cell::from(format!("{} {}", c.family, c.label)).style(style));
        }
        let mut widths = vec![Constraint::Length(id_width)];
        widths.extend((start..end).map(|_| Constraint::Length(col_width)));
        let more = if end < self.table.columns.len() || start > 0 {
            format!(" · columns {}-{} of {}", start + 1, end, self.table.columns.len())
        } else {
            String::new()
        };
        let title = format!(
            " {} ({}) · {} · {} of {} rows{}{} ",
            row.suite,
            row.suite_sha8,
            self.current_key().map(|k| k.describe()).unwrap_or_default(),
            (start_row + 1).min(total),
            total,
            if self.detail_only_disagreements {
                " · disagreements only"
            } else {
                ""
            },
            more
        );
        let widget = TableWidget::new(shown, widths)
            .header(Row::new(header_cells))
            .column_spacing(1)
            .block(Block::default().borders(Borders::ALL).title(title));
        frame.render_widget(widget, area);
    }

    fn draw_help(&self, frame: &mut Frame, area: Rect) {
        let text = "\
epd-runner terminal view

Main table: rows are suites, columns the selected engines, cells the solved
count (or percent with p) at the current budget; the best and worst cell of
a row are green and red; — means no run in the store for that engine at
this budget. Wide tables scroll sideways with the cursor.

  ↑ ↓ ← →   move the cursor            Enter   open the suite (per position)
  b / B     next / previous budget     m       next mode (nodes, depth, time)
  p         counts ⇄ percent           e       choose engines (versions side by side)
  r         queue a run for the missing cell under the cursor
  R         queue every missing cell in the column
  l         reload the store           q       quit

Runs go one at a time through `epd-runner run`, only for engines whose
engines.toml entry is the same binary and options as the column, on this
CPU for time budgets, and for the suite revision on disk. Time-mode runs
ask first and refuse while the machine is busy. Finished runs land in the
store and the table reloads; runs still going when you quit finish on
their own.";
        let popup = centered(area, 82, 24);
        frame.render_widget(Clear, popup);
        frame.render_widget(
            Paragraph::new(text)
                .wrap(Wrap { trim: false })
                .block(Block::default().borders(Borders::ALL).title(" help ")),
            popup,
        );
    }
}

fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let w = width.min(area.width);
    let h = height.min(area.height);
    Rect {
        x: area.x + (area.width - w) / 2,
        y: area.y + (area.height - h) / 2,
        width: w,
        height: h,
    }
}

pub fn run(epd_dir: &Path) -> Result<(), String> {
    let mut app = App::new(epd_dir)?;
    // Leave the terminal usable if anything panics while it is in raw mode.
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        ratatui::restore();
        previous(info);
    }));
    let mut terminal = ratatui::init();
    let result = (|| -> Result<(), String> {
        loop {
            app.poll_jobs();
            terminal.draw(|frame| app.draw(frame)).map_err(|e| e.to_string())?;
            if event::poll(Duration::from_millis(250)).map_err(|e| e.to_string())? {
                if let Event::Key(key) = event::read().map_err(|e| e.to_string())? {
                    if key.kind == KeyEventKind::Press && !app.handle_key(key.code, key.modifiers) {
                        break;
                    }
                }
            }
        }
        Ok(())
    })();
    ratatui::restore();
    app.persist();
    result
}
