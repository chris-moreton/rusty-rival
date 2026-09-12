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

/// Fetch `tag` (for example `v1.0.64`), verify its `id name` carries the
/// version, and return the binary's path and reported name.
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
    if !to.is_file() {
        eprintln!("downloading {}", url);
        download(&url, &to)?;
    }
    let engine = Engine::spawn(&to, &[], Duration::from_secs(60))?;
    let id_name = engine.id_name.clone();
    engine.quit();
    if !id_name.contains(version) {
        return Err(format!("{} reports '{}', not version {}", to.display(), id_name, version));
    }
    Ok((to, id_name))
}

/// Add or update the registry entry for a fetched release, editing the TOML
/// as text so comments and the other entries stay as they are.
pub fn upsert_rusty_entry(toml_text: &str, version: &str, relative_path: &str) -> String {
    let block = format!(
        "[[engine]]\nname = \"{}\"\nfamily = \"rusty-rival\"\npath = \"{}\"\n",
        version, relative_path
    );
    let name_line = format!("name = \"{}\"", version);
    let mut out = String::new();
    let mut replaced = false;
    let mut in_target = false;
    for line in toml_text.lines() {
        let trimmed = line.trim();
        if trimmed == "[[engine]]" {
            in_target = false;
        }
        if trimmed == name_line && !replaced {
            // Look ahead is not needed: the path line of this block is rewritten below.
            in_target = true;
            replaced = true;
        }
        if in_target && trimmed.starts_with("path =") {
            out.push_str(&format!("path = \"{}\"\n", relative_path));
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    if !replaced {
        if !out.is_empty() && !out.ends_with("\n\n") {
            out.push('\n');
        }
        out.push_str(&block);
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
    fn engine_path_follows_the_engines_convention() {
        let p = engine_path(Path::new("/repo/epd"), "v1.0.64");
        assert!(p.ends_with(Path::new("engines/v1.0.64/rusty-rival")) || p.ends_with(Path::new("engines/v1.0.64/rusty-rival.exe")));
    }
}
