//! `fetch`: download a rusty-rival release binary for this platform into
//! `engines/<tag>/rusty-rival`, verify it, and register it.

use crate::uci::Engine;
use std::path::{Path, PathBuf};
use std::time::Duration;

const REPO: &str = "chris-moreton/rusty-rival";

/// The release asset suffix for the running platform.
pub fn asset_suffix() -> Result<&'static str, String> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => {
            #[cfg(target_arch = "x86_64")]
            {
                if std::arch::is_x86_feature_detected!("avx2") {
                    return Ok("linux-x86_64-avx2");
                }
            }
            Ok("linux-x86_64")
        }
        ("macos", "aarch64") => Ok("macos-aarch64"),
        ("macos", "x86_64") => Ok("macos-x86_64"),
        ("windows", "x86_64") => {
            #[cfg(target_arch = "x86_64")]
            {
                if std::arch::is_x86_feature_detected!("avx2") {
                    return Ok("windows-x86_64-avx2.exe");
                }
            }
            Ok("windows-x86_64.exe")
        }
        (os, arch) => Err(format!("no release asset for {}/{}", os, arch)),
    }
}

/// Where a fetched release lives: `<repo>/engines/<tag>/rusty-rival`, the
/// convention the local benchmarking already uses.
pub fn engine_path(epd_dir: &Path, tag: &str) -> PathBuf {
    let name = if std::env::consts::OS == "windows" {
        "rusty-rival.exe"
    } else {
        "rusty-rival"
    };
    epd_dir.join("..").join("engines").join(tag).join(name)
}

/// Download the asset with curl (present on every platform we build for),
/// so the tool needs no HTTP client of its own.
fn download(url: &str, to: &Path) -> Result<(), String> {
    if let Some(parent) = to.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("cannot create {}: {}", parent.display(), e))?;
    }
    let status = std::process::Command::new("curl")
        .args(["-fsSL", "--retry", "3", "-o"])
        .arg(to)
        .arg(url)
        .status()
        .map_err(|e| format!("cannot run curl: {}", e))?;
    if !status.success() {
        let _ = std::fs::remove_file(to);
        return Err(format!("download failed for {} (is the tag published?)", url));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(to, std::fs::Permissions::from_mode(0o755)).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// True when the UCI id name carries exactly this version as a token
/// ("Rusty Rival 1.0.64" for 1.0.64, not for 1.0.6).
pub fn reports_version(id_name: &str, version: &str) -> bool {
    id_name
        .split_whitespace()
        .any(|t| t == version || t.trim_start_matches('v') == version)
}

/// Fetch `tag` (for example `v1.0.64`), verify its `id name` carries the
/// version, and return the binary's path and reported name. The download
/// lands in a `.part` file and is renamed only after it verifies, so a
/// wrong or broken asset never blocks the next attempt.
pub fn fetch_rusty(epd_dir: &Path, tag: &str) -> Result<(PathBuf, String), String> {
    let tag = if tag.starts_with('v') {
        tag.to_string()
    } else {
        format!("v{}", tag)
    };
    let version = tag.trim_start_matches('v');
    let suffix = asset_suffix()?;
    let url = format!(
        "https://github.com/{}/releases/download/{}/rusty-rival-{}-{}",
        REPO, tag, tag, suffix
    );
    let to = engine_path(epd_dir, &tag);
    let verify = |path: &Path| -> Result<String, String> {
        let engine = Engine::spawn(path, &[], Duration::from_secs(60))?;
        let id_name = engine.id_name.clone();
        engine.quit();
        if !reports_version(&id_name, version) {
            return Err(format!("{} reports '{}', not version {}", path.display(), id_name, version));
        }
        Ok(id_name)
    };
    if to.is_file() {
        let id_name = verify(&to)?;
        return Ok((to, id_name));
    }
    let part = to.with_extension("part");
    eprintln!("downloading {}", url);
    download(&url, &part)?;
    let id_name = match verify(&part) {
        Ok(name) => name,
        Err(e) => {
            let _ = std::fs::remove_file(&part);
            return Err(e);
        }
    };
    std::fs::rename(&part, &to).map_err(|e| format!("cannot rename {}: {}", part.display(), e))?;
    Ok((to, id_name))
}

/// One `[[engine]]` block of the registry text, kept as lines so comments
/// and spacing survive a rewrite.
struct Block {
    lines: Vec<String>,
}

impl Block {
    fn value_of(&self, key: &str) -> Option<String> {
        self.lines.iter().find_map(|l| {
            let (k, v) = l.split_once('=')?;
            if k.trim() != key {
                return None;
            }
            Some(v.trim().trim_matches('"').to_string())
        })
    }

    fn set_path(&mut self, path: &str) {
        let line = format!("path = \"{}\"", path);
        if let Some(i) = self
            .lines
            .iter()
            .position(|l| l.split_once('=').is_some_and(|(k, _)| k.trim() == "path"))
        {
            self.lines[i] = line;
        } else {
            // Insert after the last key line so a trailing blank line stays last.
            let at = self
                .lines
                .iter()
                .rposition(|l| l.contains('=') && !l.trim_start().starts_with('#'))
                .map_or(self.lines.len(), |i| i + 1);
            self.lines.insert(at, line);
        }
    }
}

/// Add or update the registry entry for a fetched release, editing the TOML
/// as text so comments and the other entries stay as they are. Keys may be
/// in any order and spaced freely; the match is on the rusty-rival family
/// (or no family) and the name.
pub fn upsert_rusty_entry(toml_text: &str, version: &str, relative_path: &str) -> String {
    let mut preamble: Vec<String> = Vec::new();
    let mut blocks: Vec<Block> = Vec::new();
    for line in toml_text.lines() {
        if line.trim() == "[[engine]]" {
            blocks.push(Block {
                lines: vec![line.to_string()],
            });
        } else if let Some(b) = blocks.last_mut() {
            b.lines.push(line.to_string());
        } else {
            preamble.push(line.to_string());
        }
    }
    let target = blocks
        .iter_mut()
        .find(|b| b.value_of("name").as_deref() == Some(version) && b.value_of("family").as_deref().is_none_or(|f| f == "rusty-rival"));
    match target {
        Some(b) => b.set_path(relative_path),
        None => {
            if let Some(last) = blocks.last_mut() {
                if !last.lines.last().is_some_and(|l| l.trim().is_empty()) {
                    last.lines.push(String::new());
                }
            }
            blocks.push(Block {
                lines: vec![
                    "[[engine]]".to_string(),
                    format!("name = \"{}\"", version),
                    "family = \"rusty-rival\"".to_string(),
                    format!("path = \"{}\"", relative_path),
                ],
            });
        }
    }
    let mut out = String::new();
    for l in preamble.iter().chain(blocks.iter().flat_map(|b| b.lines.iter())) {
        out.push_str(l);
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upsert_appends_or_rewrites_the_path() {
        let base = "# registry\n\n[[engine]]\nname = \"1.0.63\"\nfamily = \"rusty-rival\"\npath = \"~/old/rival-v1.0.63\"\n";
        let added = upsert_rusty_entry(base, "1.0.64", "../engines/v1.0.64/rusty-rival");
        assert!(added.contains("name = \"1.0.64\"\nfamily = \"rusty-rival\"\npath = \"../engines/v1.0.64/rusty-rival\""));
        assert!(added.contains("path = \"~/old/rival-v1.0.63\""), "other entries untouched");
        let updated = upsert_rusty_entry(&added, "1.0.63", "../engines/v1.0.63/rusty-rival");
        assert!(updated.contains("path = \"../engines/v1.0.63/rusty-rival\""));
        assert!(!updated.contains("~/old/rival-v1.0.63"));
        assert_eq!(updated.matches("[[engine]]").count(), 2, "no duplicate block");
        assert!(updated.starts_with("# registry"), "comments kept");
    }

    #[test]
    fn upsert_tolerates_key_order_spacing_and_other_families() {
        let text = "[[engine]]\npath=\"~/x/rival\"\nname=\"1.0.64\"\nfamily = \"rusty-rival\"\n\n[[engine]]\nname = \"1.0.64\"\nfamily = \"other-engine\"\npath = \"~/other\"\n";
        let out = upsert_rusty_entry(text, "1.0.64", "../engines/v1.0.64/rusty-rival");
        assert_eq!(
            out.matches("[[engine]]").count(),
            2,
            "the compact entry is rewritten, not duplicated"
        );
        assert!(
            out.contains("path = \"../engines/v1.0.64/rusty-rival\"\nname=\"1.0.64\""),
            "path rewritten in place before name"
        );
        assert!(
            out.contains("path = \"~/other\""),
            "an entry of another family with the same name is untouched"
        );
        let no_path = "[[engine]]\nname = \"1.0.64\"\nfamily = \"rusty-rival\"\n";
        let out = upsert_rusty_entry(no_path, "1.0.64", "p");
        assert!(
            out.contains("family = \"rusty-rival\"\npath = \"p\""),
            "a missing path line is added"
        );
    }

    #[test]
    fn version_must_match_as_a_whole_token() {
        assert!(reports_version("Rusty Rival 1.0.64", "1.0.64"));
        assert!(reports_version("Rusty Rival v1.0.64", "1.0.64"));
        assert!(!reports_version("Rusty Rival 1.0.64", "1.0.6"));
        assert!(!reports_version("Rusty Rival 1.0.6", "1.0.64"));
    }

    #[test]
    fn engine_path_follows_the_engines_convention() {
        let p = engine_path(Path::new("/repo/epd"), "v1.0.64");
        assert!(p.ends_with(Path::new("engines/v1.0.64/rusty-rival")) || p.ends_with(Path::new("engines/v1.0.64/rusty-rival.exe")));
    }
}
