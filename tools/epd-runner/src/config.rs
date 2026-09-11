//! `epd/engines.toml`: the engine registry.

use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
pub struct EngineEntry {
    pub name: String,
    #[serde(default)]
    pub family: Option<String>,
    pub path: String,
    #[serde(default)]
    pub options: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Registry {
    #[serde(default)]
    pub engine: Vec<EngineEntry>,
}

impl Registry {
    pub fn load(epd_dir: &Path) -> Result<Registry, String> {
        let path = epd_dir.join("engines.toml");
        if !path.exists() {
            return Ok(Registry::default());
        }
        let text = std::fs::read_to_string(&path).map_err(|e| format!("cannot read {}: {}", path.display(), e))?;
        toml::from_str(&text).map_err(|e| format!("{}: {}", path.display(), e))
    }

    pub fn find(&self, name: &str) -> Option<&EngineEntry> {
        self.engine.iter().find(|e| e.name == name)
    }
}

impl EngineEntry {
    pub fn resolved_path(&self) -> PathBuf {
        expand_home(&self.path)
    }
}

pub fn expand_home(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home).join(rest);
        }
    }
    PathBuf::from(path)
}
