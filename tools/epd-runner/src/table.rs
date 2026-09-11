//! The suites × engines table for one budget, shared by the text command and
//! the terminal view.

use crate::store::{budget_label, ResultsFile};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize)]
pub struct Column {
    pub family: String,
    pub label: String,
    pub version: String,
    pub sha8: String,
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
    pub positions: usize,
    pub cells: Vec<Option<Cell>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Table {
    pub mode: String,
    pub budget: u64,
    pub budget_label: String,
    pub columns: Vec<Column>,
    pub rows: Vec<Row>,
}

/// Does an engine match a selector? `family:label`, or a bare family, label
/// or version.
pub fn engine_matches(selector: &str, column: &Column) -> bool {
    match selector.split_once(':') {
        Some((family, label)) => column.family == family && (column.label == label || column.version == label),
        None => column.family == selector || column.label == selector || column.version == selector,
    }
}

pub fn build(files: &[ResultsFile], mode: &str, budget: u64, engines: Option<&[String]>, suites: Option<&[String]>) -> Table {
    let mut columns: Vec<(Column, &ResultsFile)> = files
        .iter()
        .map(|f| {
            (
                Column {
                    family: f.engine.family.clone(),
                    label: f.engine.label.clone(),
                    version: f.engine.version.clone(),
                    sha8: f.engine.sha8().to_string(),
                },
                f,
            )
        })
        .filter(|(c, f)| {
            f.runs.iter().any(|r| r.mode == mode && r.budget == budget)
                && engines.is_none_or(|sel| sel.iter().any(|s| engine_matches(s, c)))
        })
        .collect();
    columns.sort_by(|a, b| {
        a.0.family
            .cmp(&b.0.family)
            .then_with(|| version_key(&a.0.label).cmp(&version_key(&b.0.label)))
    });

    let mut suite_names: BTreeMap<String, usize> = BTreeMap::new();
    for (_, f) in &columns {
        for r in f.runs.iter().filter(|r| r.mode == mode && r.budget == budget) {
            suite_names.entry(r.suite.name.clone()).or_insert(r.suite.positions);
        }
    }
    let rows = suite_names
        .into_iter()
        .filter(|(name, _)| suites.is_none_or(|sel| sel.iter().any(|s| s == name)))
        .map(|(name, positions)| Row {
            cells: columns
                .iter()
                .map(|(_, f)| {
                    f.runs
                        .iter()
                        .filter(|r| r.mode == mode && r.budget == budget && r.suite.name == name)
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
            suite: name,
            positions,
        })
        .collect();
    Table {
        mode: mode.to_string(),
        budget,
        budget_label: budget_label(mode, budget),
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
        let mut headers: Vec<String> = vec![format!("suite ({})", self.budget_label)];
        let mut sub: Vec<String> = vec![String::new()];
        for c in &self.columns {
            headers.push(format!("{} {}", c.family, c.label));
            sub.push(c.sha8.to_string());
        }
        let mut lines: Vec<Vec<String>> = vec![headers, sub];
        for row in &self.rows {
            let mut line = vec![format!("{} ({})", row.suite, row.positions)];
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
        if self.columns.is_empty() {
            out.push_str("(no results at this budget; run `epd-runner run` first)\n");
        }
        out
    }
}
