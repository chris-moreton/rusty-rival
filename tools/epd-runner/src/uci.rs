//! A minimal UCI driver: one engine process, one search at a time, with a
//! reader thread so every wait has a deadline.

use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Limit {
    Nodes(u64),
    /// Milliseconds per position.
    MoveTime(u64),
    Depth(u32),
}

impl Limit {
    pub fn go_command(&self) -> String {
        match self {
            Limit::Nodes(n) => format!("go nodes {}", n),
            Limit::MoveTime(ms) => format!("go movetime {}", ms),
            Limit::Depth(d) => format!("go depth {}", d),
        }
    }

    pub fn mode(&self) -> &'static str {
        match self {
            Limit::Nodes(_) => "nodes",
            Limit::MoveTime(_) => "time",
            Limit::Depth(_) => "depth",
        }
    }

    pub fn budget(&self) -> u64 {
        match self {
            Limit::Nodes(n) => *n,
            Limit::MoveTime(ms) => *ms,
            Limit::Depth(d) => *d as u64,
        }
    }

    /// Wall-clock allowance for one position before the engine is killed.
    pub fn timeout(&self) -> Duration {
        match self {
            Limit::Nodes(_) => Duration::from_secs(300),
            Limit::MoveTime(ms) => Duration::from_millis(ms + 15_000),
            Limit::Depth(_) => Duration::from_secs(900),
        }
    }

    pub fn describe(&self) -> String {
        match self {
            Limit::Nodes(n) => format!("nodes {}", n),
            Limit::MoveTime(ms) => format!("time {}ms", ms),
            Limit::Depth(d) => format!("depth {}", d),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct InfoLine {
    pub depth: u32,
    pub nodes: u64,
    pub time_ms: u64,
    pub score_cp: Option<i32>,
    pub mate: Option<i32>,
    pub pv: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SearchOutput {
    pub bestmove: String,
    pub infos: Vec<InfoLine>,
    pub elapsed_ms: u64,
}

pub struct Engine {
    child: Child,
    stdin: ChildStdin,
    lines: Receiver<String>,
    pub id_name: String,
}

impl Engine {
    pub fn spawn(path: &Path, options: &[(String, String)], timeout: Duration) -> Result<Engine, String> {
        let mut child = Command::new(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("cannot start {}: {}", path.display(), e))?;
        let stdin = child.stdin.take().ok_or("engine has no stdin")?;
        let stdout = child.stdout.take().ok_or("engine has no stdout")?;
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                match line {
                    Ok(l) => {
                        if tx.send(l).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });
        let mut engine = Engine {
            child,
            stdin,
            lines: rx,
            id_name: String::new(),
        };
        engine.send("uci")?;
        let deadline = Instant::now() + timeout;
        loop {
            let line = engine.next_line(deadline)?;
            if let Some(name) = line.strip_prefix("id name ") {
                engine.id_name = name.trim().to_string();
            }
            if line.trim() == "uciok" {
                break;
            }
        }
        for (name, value) in options {
            engine.send(&format!("setoption name {} value {}", name, value))?;
        }
        engine.send("isready")?;
        engine.wait_for("readyok", deadline)?;
        Ok(engine)
    }

    fn send(&mut self, command: &str) -> Result<(), String> {
        writeln!(self.stdin, "{}", command)
            .and_then(|_| self.stdin.flush())
            .map_err(|e| format!("engine stdin: {}", e))
    }

    fn next_line(&self, deadline: Instant) -> Result<String, String> {
        let now = Instant::now();
        if now >= deadline {
            return Err("timeout waiting for the engine".to_string());
        }
        match self.lines.recv_timeout(deadline - now) {
            Ok(line) => Ok(line),
            Err(RecvTimeoutError::Timeout) => Err("timeout waiting for the engine".to_string()),
            Err(RecvTimeoutError::Disconnected) => Err("engine exited".to_string()),
        }
    }

    fn wait_for(&self, token: &str, deadline: Instant) -> Result<(), String> {
        loop {
            if self.next_line(deadline)?.trim() == token {
                return Ok(());
            }
        }
    }

    pub fn search(&mut self, fen: &str, limit: Limit) -> Result<SearchOutput, String> {
        let deadline = Instant::now() + limit.timeout();
        self.send("ucinewgame")?;
        self.send("isready")?;
        self.wait_for("readyok", deadline)?;
        self.send(&format!("position fen {}", fen))?;
        let start = Instant::now();
        self.send(&limit.go_command())?;
        let mut infos = Vec::new();
        loop {
            let line = match self.next_line(deadline) {
                Ok(l) => l,
                Err(e) => {
                    let _ = self.send("stop");
                    return Err(e);
                }
            };
            let line = line.trim();
            if let Some(rest) = line.strip_prefix("bestmove") {
                let bestmove = rest.split_whitespace().next().unwrap_or("").to_string();
                return Ok(SearchOutput {
                    bestmove,
                    infos,
                    elapsed_ms: start.elapsed().as_millis() as u64,
                });
            }
            if line.starts_with("info ") {
                if let Some(info) = parse_info(line) {
                    infos.push(info);
                }
            }
        }
    }

    pub fn quit(mut self) {
        let _ = self.send("quit");
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            match self.child.try_wait() {
                Ok(Some(_)) => return,
                Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(10)),
                _ => {
                    let _ = self.child.kill();
                    let _ = self.child.wait();
                    return;
                }
            }
        }
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        if let Ok(None) = self.child.try_wait() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

/// Parse an `info` line that carries a principal variation; lines without a
/// PV (currmove, string, hashfull-only) and MultiPV lines beyond the first
/// are dropped.
pub fn parse_info(line: &str) -> Option<InfoLine> {
    let tokens: Vec<&str> = line.split_whitespace().collect();
    let mut info = InfoLine::default();
    let mut i = 1;
    while i < tokens.len() {
        match tokens[i] {
            "depth" => {
                info.depth = tokens.get(i + 1)?.parse().ok()?;
                i += 2;
            }
            "nodes" => {
                info.nodes = tokens.get(i + 1)?.parse().ok()?;
                i += 2;
            }
            "time" => {
                info.time_ms = tokens.get(i + 1)?.parse().ok()?;
                i += 2;
            }
            "multipv" => {
                if tokens.get(i + 1).copied() != Some("1") {
                    return None;
                }
                i += 2;
            }
            "score" => {
                match tokens.get(i + 1).copied() {
                    Some("cp") => info.score_cp = tokens.get(i + 2)?.parse().ok(),
                    Some("mate") => info.mate = tokens.get(i + 2)?.parse().ok(),
                    _ => {}
                }
                i += 3;
                if matches!(tokens.get(i).copied(), Some("lowerbound") | Some("upperbound")) {
                    i += 1;
                }
            }
            "pv" => {
                info.pv = tokens[i + 1..].iter().map(|s| s.to_string()).collect();
                return if info.pv.is_empty() { None } else { Some(info) };
            }
            "string" => return None,
            "seldepth" | "hashfull" | "tbhits" | "nps" | "currmovenumber" | "currmove" | "cpuload" | "wdl" => {
                i += 2;
            }
            _ => i += 1,
        }
    }
    None
}

/// rusty-rival's deterministic bench signature, read from a fresh process.
pub fn bench_signature(path: &Path) -> Option<u64> {
    let mut child = Command::new(path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let mut stdin = child.stdin.take()?;
    let stdout = child.stdout.take()?;
    writeln!(stdin, "bench").ok()?;
    let mut signature = None;
    for line in BufReader::new(stdout).lines().map_while(Result::ok) {
        if let Some(rest) = line.strip_prefix("Nodes searched:") {
            signature = rest.trim().replace(',', "").parse().ok();
            break;
        }
    }
    let _ = writeln!(stdin, "quit");
    drop(stdin);
    let _ = child.kill();
    let _ = child.wait();
    signature
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_info_lines() {
        let i = parse_info("info depth 12 seldepth 20 multipv 1 score cp 35 nodes 123456 nps 1500000 time 82 pv e2e4 e7e5").unwrap();
        assert_eq!((i.depth, i.nodes, i.time_ms, i.score_cp, i.mate), (12, 123456, 82, Some(35), None));
        assert_eq!(i.pv, vec!["e2e4", "e7e5"]);
        let i = parse_info("info depth 9 score mate 3 lowerbound nodes 10 time 1 pv g1f3").unwrap();
        assert_eq!((i.mate, i.pv.len()), (Some(3), 1));
        assert!(parse_info("info depth 3 currmove e2e4 currmovenumber 1").is_none());
        assert!(parse_info("info string Loaded tablebases").is_none());
        assert!(parse_info("info depth 5 multipv 2 score cp 1 pv a2a3").is_none());
    }
}
