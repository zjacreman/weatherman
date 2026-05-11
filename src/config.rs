//! Configuration persistence module.
//!
//! Loads and saves the last-used location name across app sessions.
//! Check order: `~/.config/weatherman/weatherman.toml` first, then `./weatherman.toml`.
//! Write targets the path that was read at startup; if neither existed, defaults to config dir.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// The filename used for the persisted config file.
const CONFIG_FILE_NAME: &str = "weatherman.toml";

/// The last-used location name.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedConfig {
    pub name: String,
}

/// Return the path to `~/.config/weatherman/weatherman.toml`.
fn config_path() -> PathBuf {
    let home = std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    home.join(".config")
        .join("weatherman")
        .join(CONFIG_FILE_NAME)
}

/// Return the path to `./weatherman.toml` in the current working directory.
fn cwd_config_path() -> PathBuf {
    PathBuf::from(CONFIG_FILE_NAME)
}

/// Try to load the saved config file.
///
/// Checks the config directory first, then the working directory.
/// Returns the config along with the path it was loaded from.
pub fn load_config() -> Option<(SavedConfig, PathBuf)> {
    // 1. Check ~/.config/weatherman/weatherman.toml
    let cfg = config_path();
    if let Ok(contents) = std::fs::read_to_string(&cfg) {
        if let Ok(config) = toml::from_str(&contents) {
            return Some((config, cfg));
        }
    }

    // 2. Check ./weatherman.toml
    let cwd = cwd_config_path();
    if let Ok(contents) = std::fs::read_to_string(&cwd) {
        if let Ok(config) = toml::from_str(&contents) {
            return Some((config, cwd));
        }
    }

    None
}

/// Save the config to disk.
///
/// Uses `last_config_path` if provided (meaning a config was read at startup),
/// otherwise falls back to the config directory.
///
/// Creates parent directories as needed.
/// Returns the path that was written to.
pub fn save_config(
    config: &SavedConfig,
    last_config_path: Option<&Path>,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let target = last_config_path
        .map(|p| p.to_path_buf())
        .or_else(|| Some(config_path()))
        .unwrap_or_else(|| cwd_config_path());

    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let contents = toml::to_string_pretty(config)?;
    std::fs::write(&target, contents)?;

    Ok(target)
}
