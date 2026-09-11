//! Host facts for time-mode records and the idle check.

use std::path::Path;

/// The CPU model, so a time-mode record can be told apart from one made on
/// other hardware.
pub fn cpu_model() -> String {
    if let Ok(text) = std::fs::read_to_string("/proc/cpuinfo") {
        for line in text.lines() {
            if let Some(rest) = line.strip_prefix("model name") {
                if let Some((_, model)) = rest.split_once(':') {
                    return model.trim().to_string();
                }
            }
        }
    }
    if let Ok(out) = std::process::Command::new("sysctl")
        .args(["-n", "machdep.cpu.brand_string"])
        .output()
    {
        let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !s.is_empty() {
            return s;
        }
    }
    "unknown".to_string()
}

pub fn hostname() -> String {
    std::fs::read_to_string("/etc/hostname")
        .map(|s| s.trim().to_string())
        .ok()
        .filter(|s| !s.is_empty())
        .or_else(|| std::env::var("HOSTNAME").ok())
        .unwrap_or_else(|| "unknown".to_string())
}

/// A reason the machine is not idle enough for a time-mode run, if any:
/// a lichess-bot engine process, or a one-minute load average above the
/// threshold.
pub fn busy_reason(load_threshold: f64) -> Option<String> {
    let me = std::process::id();
    if let Ok(entries) = std::fs::read_dir("/proc") {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let Some(pid) = name.to_str().and_then(|s| s.parse::<u32>().ok()) else {
                continue;
            };
            if pid == me {
                continue;
            }
            let cmdline = std::fs::read(Path::new("/proc").join(name).join("cmdline")).unwrap_or_default();
            if cmdline.windows(19).any(|w| w == b"lichess-bot/engines") {
                return Some(format!("a lichess-bot engine is running (pid {})", pid));
            }
        }
    }
    if let Ok(text) = std::fs::read_to_string("/proc/loadavg") {
        if let Some(load) = text.split_whitespace().next().and_then(|s| s.parse::<f64>().ok()) {
            if load > load_threshold {
                return Some(format!("one-minute load average {:.1} is above {:.1}", load, load_threshold));
            }
        }
    }
    None
}
