//! The suites × engines table for one budget, shared by the text command and
//! the terminal view.

use crate::store::{budget_label, ResultsFile, RunRecord};
use serde::Serialize;
use std::collections::BTreeMap;

/// Everything that must agree for two runs to sit in one table: the budget
/// and the engine settings, and in time mode the host and concurrency too.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TableKey {
    pub mode: String,
    pub budget: u64,
    pub threads: u32,
    pub hash_mb: u32,
    /// Time mode only: the concurrency and CPU model the runs must match.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub concurrency: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu: Option<String>,
}

impl TableKey {
    pub fn matches(&self, run: &RunRecord) -> bool {
        run.mode == self.mode
            && run.budget == self.budget
            && run.threads == self.threads
            && run.hash_mb == self.hash_mb
            && (self.mode != "time"
                || (self.concurrency.is_none_or(|c| run.concurrency == c)
                    && self
                        .cpu
                        .as_deref()
                        .is_none_or(|cpu| run.host.as_ref().is_some_and(|h| h.cpu == cpu))))
    }

    pub fn describe(&self) -> String {
        let mut s = format!(
            "{} · threads {} hash {}",
            budget_label(&self.mode, self.budget),
            self.threads,
            self.hash_mb
        );
        if self.mode == "time" {
            if let Some(c) = self.concurrency {
                s.push_str(&format!(" · concurrency {}", c));
            }
            if let Some(cpu) = &self.cpu {
                s.push_str(&format!(" · {}", cpu));
            }
        }
        s
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Column {
    pub family: String,
    pub label: String,
    pub version: String,
    pub sha8: String,
    /// The UCI options the engine ran with; empty for a plain run.
    pub options: BTreeMap<String, String>,
    /// Eight hex digits over the options, when any are set: the same binary
    /// with other options is another column, and this is how it is named.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options_hash8: Option<String>,
}

impl Column {
    /// The identity line under the header: the binary's sha8, plus the
    /// options hash when the engine ran with options.
    pub fn identity(&self) -> String {
        match &self.options_hash8 {
            Some(h) => format!("{}#{}", self.sha8, h),
            None => self.sha8.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Cell {
    pub solved: usize,
    pub total: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub points: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_points: Option<u32>,
    pub percent: f64,
    pub date: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Row {
    pub suite: String,
    /// The suite file's sha256, so two revisions of one suite never share a row.
    pub suite_sha8: String,
    pub positions: usize,
    /// Set when more than one revision of this suite is in the table.
    pub revision_shown: bool,
    pub cells: Vec<Option<Cell>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Table {
    pub key: TableKey,
    pub budget_label: String,
    pub columns: Vec<Column>,
    pub rows: Vec<Row>,
}

/// Does an engine match a selector? `family:label`, or a bare family, label
/// or version; an optional `#hash` suffix pins the binary sha8 or the
/// options hash, which is how two option variants of one binary are told
/// apart (`stockfish:dev-20260726#9e2b0c1d`).
pub fn engine_matches(selector: &str, column: &Column) -> bool {
    let (base, hash) = match selector.split_once('#') {
        Some((b, h)) => (b, Some(h)),
        None => (selector, None),
    };
    let base_ok = match base.split_once(':') {
        Some((family, label)) => column.family == family && (column.label == label || column.version == label),
        None => base.is_empty() || column.family == base || column.label == base || column.version == base,
    };
    base_ok && hash.is_none_or(|h| column.sha8 == h || column.options_hash8.as_deref() == Some(h))
}

/// The column an engine record would occupy.
pub fn column_of(engine: &crate::store::EngineRecord) -> Column {
    Column {
        family: engine.family.clone(),
        label: engine.label.clone(),
        version: engine.version.clone(),
        sha8: engine.sha8().to_string(),
        options: engine.options.clone(),
        options_hash8: engine.options_hash8(),
    }
}

pub fn build(files: &[ResultsFile], key: &TableKey, engines: Option<&[String]>, suites: Option<&[String]>) -> Table {
    let mut columns: Vec<(Column, &ResultsFile)> = files
        .iter()
        .map(|f| (column_of(&f.engine), f))
        .filter(|(c, f)| f.runs.iter().any(|r| key.matches(r)) && engines.is_none_or(|sel| sel.iter().any(|s| engine_matches(s, c))))
        .collect();
    columns.sort_by(|a, b| {
        a.0.family
            .cmp(&b.0.family)
            .then_with(|| version_key(&a.0.label).cmp(&version_key(&b.0.label)))
            .then_with(|| a.0.identity().cmp(&b.0.identity()))
    });

    // Rows are (suite name, suite sha): a suite file that changed between
    // two engines' runs gives two rows, marked with the revision.
    let mut revisions: BTreeMap<(String, String), usize> = BTreeMap::new();
    for (_, f) in &columns {
        for r in f.runs.iter().filter(|r| key.matches(r)) {
            revisions
                .entry((r.suite.name.clone(), r.suite.sha256.clone()))
                .or_insert(r.suite.positions);
        }
    }
    let mut per_name: BTreeMap<String, usize> = BTreeMap::new();
    for (name, _) in revisions.keys() {
        *per_name.entry(name.clone()).or_insert(0) += 1;
    }
    let rows = revisions
        .into_iter()
        .filter(|((name, _), _)| suites.is_none_or(|sel| sel.iter().any(|s| s == name)))
        .map(|((name, sha), positions)| Row {
            cells: columns
                .iter()
                .map(|(_, f)| {
                    f.runs
                        .iter()
                        .filter(|r| key.matches(r) && r.suite.name == name && r.suite.sha256 == sha)
                        .max_by(|a, b| a.date.cmp(&b.date))
                        .map(|r| {
                            let s = &r.summary;
                            let percent = match (s.points, s.max_points) {
                                (Some(p), Some(m)) if m > 0 => 100.0 * p as f64 / m as f64,
                                _ if s.total > 0 => 100.0 * s.solved as f64 / s.total as f64,
                                _ => 0.0,
                            };
                            Cell {
                                solved: s.solved,
                                total: s.total,
                                points: s.points,
                                max_points: s.max_points,
                                percent: (percent * 10.0).round() / 10.0,
                                date: r.date.clone(),
                            }
                        })
                })
                .collect(),
            revision_shown: per_name.get(&name).copied().unwrap_or(0) > 1,
            suite_sha8: sha[..sha.len().min(8)].to_string(),
            suite: name,
            positions,
        })
        .collect();
    Table {
        budget_label: key.describe(),
        key: key.clone(),
        columns: columns.into_iter().map(|(c, _)| c).collect(),
        rows,
    }
}

/// Natural ordering for version-like labels: numeric runs compare as numbers.
pub fn version_key(label: &str) -> Vec<(u64, String)> {
    let mut key = Vec::new();
    let mut digits = String::new();
    let mut text = String::new();
    let flush = |digits: &mut String, text: &mut String, key: &mut Vec<(u64, String)>| {
        if !digits.is_empty() {
            key.push((digits.parse().unwrap_or(u64::MAX), String::new()));
            digits.clear();
        }
        if !text.is_empty() {
            key.push((u64::MAX, text.clone()));
            text.clear();
        }
    };
    for c in label.chars() {
        if c.is_ascii_digit() {
            if !text.is_empty() {
                flush(&mut digits, &mut text, &mut key);
            }
            digits.push(c);
        } else {
            if !digits.is_empty() {
                flush(&mut digits, &mut text, &mut key);
            }
            text.push(c);
        }
    }
    flush(&mut digits, &mut text, &mut key);
    key
}

impl Table {
    pub fn cell_text(cell: &Option<Cell>, percent: bool) -> String {
        match cell {
            None => "—".to_string(),
            Some(c) if percent => format!("{:.1}%", c.percent),
            Some(c) => match (c.points, c.max_points) {
                (Some(p), Some(m)) => format!("{}/{} pts", p, m),
                _ => format!("{}/{}", c.solved, c.total),
            },
        }
    }

    pub fn render(&self, percent: bool) -> String {
        let mut headers: Vec<String> = vec![format!("suite ({})", budget_label(&self.key.mode, self.key.budget))];
        let mut sub: Vec<String> = vec![String::new()];
        for c in &self.columns {
            headers.push(format!("{} {}", c.family, c.label));
            sub.push(c.identity());
        }
        let mut lines: Vec<Vec<String>> = vec![headers, sub];
        for row in &self.rows {
            let revision = if row.revision_shown {
                format!(" [{}]", row.suite_sha8)
            } else {
                String::new()
            };
            let mut line = vec![format!("{} ({}){}", row.suite, row.positions, revision)];
            for cell in &row.cells {
                line.push(Self::cell_text(cell, percent));
            }
            lines.push(line);
        }
        let widths: Vec<usize> = (0..lines[0].len())
            .map(|i| lines.iter().map(|l| l[i].chars().count()).max().unwrap_or(0))
            .collect();
        let mut out = String::new();
        for (n, line) in lines.iter().enumerate() {
            let cells: Vec<String> = line
                .iter()
                .enumerate()
                .map(|(i, s)| {
                    if i == 0 {
                        format!("{:<width$}", s, width = widths[i])
                    } else {
                        format!("{:>width$}", s, width = widths[i])
                    }
                })
                .collect();
            out.push_str(&cells.join("  "));
            out.push('\n');
            if n == 1 {
                out.push_str(&widths.iter().map(|w| "-".repeat(*w)).collect::<Vec<_>>().join("  "));
                out.push('\n');
            }
        }
        out.push_str(&format!("[{}]\n", self.budget_label));
        if self.columns.is_empty() {
            out.push_str("(no results for these settings; run `epd-runner run` first)\n");
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn column(label: &str, sha8: &str, options: &[(&str, &str)]) -> Column {
        let options: BTreeMap<String, String> = options.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
        let engine = crate::store::EngineRecord {
            name: "Stockfish dev".into(),
            family: "stockfish".into(),
            label: label.into(),
            version: "dev".into(),
            sha256: format!("{}{}", sha8, "0".repeat(56)),
            options: options.clone(),
            bench: None,
        };
        Column {
            family: "stockfish".into(),
            label: label.into(),
            version: "dev".into(),
            sha8: sha8.into(),
            options_hash8: engine.options_hash8(),
            options,
        }
    }

    #[test]
    fn option_variants_get_distinct_identities_and_selectors() {
        let plain = column("dev", "0123abcd", &[]);
        let capped = column("dev", "0123abcd", &[("UCI_Elo", "2800"), ("UCI_LimitStrength", "true")]);
        assert_ne!(plain.identity(), capped.identity());
        assert!(capped.identity().starts_with("0123abcd#"));
        assert!(engine_matches("stockfish", &plain) && engine_matches("stockfish", &capped));
        assert!(engine_matches("stockfish:dev", &capped));
        let pinned = format!("stockfish:dev#{}", capped.options_hash8.as_deref().unwrap());
        assert!(engine_matches(&pinned, &capped));
        assert!(!engine_matches(&pinned, &plain));
        assert!(engine_matches("#0123abcd", &plain));
        assert!(!engine_matches("stockfish:dev#ffffffff", &plain));
    }

    #[test]
    fn versions_sort_naturally() {
        let mut labels = vec!["1.0.9", "1.0.10", "1.0.64", "dev-20260726", "1.0.63"];
        labels.sort_by_key(|l| version_key(l));
        assert_eq!(labels, vec!["1.0.9", "1.0.10", "1.0.63", "1.0.64", "dev-20260726"]);
    }
}
