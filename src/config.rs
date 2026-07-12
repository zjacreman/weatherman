//! Configuration persistence module.
//!
//! Loads and saves the last-used location name across app sessions.
//! Check order: the platform config directory first, then `./weatherman.toml`.
//! Write targets the path that was read at startup; if neither existed, defaults to config dir.
//!
//! The platform config directory is resolved with the `dirs` crate so the
//! feature works on Windows (`%APPDATA%\weatherman\`), macOS
//! (`~/Library/Application Support/weatherman/`), and Linux
//! (`$XDG_CONFIG_HOME/weatherman/` or `~/.config/weatherman/`). The old
//! `HOME`-only lookup broke persistence on Windows because `HOME` is rarely
//! set outside Unix-like shells.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const CONFIG_FILE_NAME: &str = "weatherman.toml";

/// The last-used location name and refresh interval.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedConfig {
    pub name: String,
    pub refresh_interval: Option<u64>,
}

/// Return the default config directory path for the current platform.
///
/// Uses the `dirs` crate: `%APPDATA%\weatherman` on Windows,
/// `~/Library/Application Support/weatherman` on macOS, and
/// `$XDG_CONFIG_HOME/weatherman` (or `~/.config/weatherman`) on Linux.
/// Falls back to `./weatherman` if `dirs` cannot resolve a home directory
/// (extremely rare — e.g. running without a user profile).
pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .map(|c| c.join("weatherman"))
        .unwrap_or_else(|| PathBuf::from(".").join("weatherman"))
}

/// Internal helper to load config from a specific config directory path.
///
/// This function is used by both `load_config()` (production) and
/// `load_config_from_dir()` (testing) to avoid code duplication.
fn do_load_config(config_dir: &Path) -> Option<(SavedConfig, PathBuf)> {
    let cfg = config_dir.join(CONFIG_FILE_NAME);
    if let Ok(contents) = std::fs::read_to_string(&cfg) {
        if let Ok(config) = toml::from_str(&contents) {
            return Some((config, cfg));
        }
    }
    let cwd = PathBuf::from(CONFIG_FILE_NAME);
    if let Ok(contents) = std::fs::read_to_string(&cwd) {
        if let Ok(config) = toml::from_str(&contents) {
            return Some((config, cwd));
        }
    }
    None
}

/// Load the saved config from the default platform location.
///
/// Checks the config directory first, then the working directory.
/// Returns the config along with the path it was loaded from.
pub fn load_config() -> Option<(SavedConfig, PathBuf)> {
    let config_dir = config_path();
    do_load_config(&config_dir)
}

/// Load config from a specific directory path.
///
/// This function is **test-safe**: it does _not_ rely on the `HOME`
/// environment variable and will never write to the user's real config
/// directory. Tests should prefer this function over `load_config()`.
///
/// # Arguments
///
/// * `dir` — A directory that should contain (or will contain)
///   `.config/weatherman/weatherman.toml`.
#[allow(dead_code)]
pub fn load_config_from_dir(dir: &Path) -> Option<(SavedConfig, PathBuf)> {
    do_load_config(dir)
}

/// Save the config to disk.
///
/// Uses `last_config_path` if provided (meaning a config was read at startup),
/// otherwise falls back to the given `config_dir`.
///
/// Creates parent directories as needed.
/// Returns the path that was written to.
///
/// # Arguments
///
/// * `config` — The configuration to save.
/// * `last_config_path` — Previously loaded config path, if any.
/// * `config_dir` — The base config directory to use when no previous path exists.
pub fn save_config(
    config: &SavedConfig,
    last_config_path: Option<&Path>,
    config_dir: &Path,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let target = match last_config_path {
        Some(p) => p.to_path_buf(),
        None => config_dir.join(CONFIG_FILE_NAME),
    };

    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let contents = toml::to_string_pretty(config)?;
    std::fs::write(&target, contents)?;

    Ok(target)
}

/// Save config to a specific directory path.
///
/// This function is **test-safe**: it does _not_ rely on the `HOME`
/// environment variable and will never write to the user's real config
/// directory. Tests should prefer this function over `save_config()`
/// with the default config path.
///
/// # Arguments
///
/// * `config` — The configuration to save.
/// * `last_config_path` — Previously loaded config path, if any.
/// * `dir` — The base directory that will hold the config file.
#[allow(dead_code)]
pub fn save_config_to_dir(
    config: &SavedConfig,
    last_config_path: Option<&Path>,
    dir: &Path,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    save_config(config, last_config_path, dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_load_config_from_dir() {
        let temp_dir = std::env::temp_dir().join("weatherman_config_test");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).expect("create temp dir");

        let config_dir = temp_dir.join(".config").join("weatherman");
        fs::create_dir_all(&config_dir).expect("create config dir");
        let config_file = config_dir.join(CONFIG_FILE_NAME);
        let toml = "name = \"Test\"\nrefresh_interval = 3600\n";
        fs::write(&config_file, toml).expect("write config");

        let (config, path) = load_config_from_dir(&config_dir)
            .expect("load config from test dir");
        assert_eq!(config.name, "Test");
        assert_eq!(path, config_file);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_load_config_from_dir_returns_none() {
        let temp_dir = std::env::temp_dir().join("weatherman_config_none_test");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).expect("create temp dir");

        let config_dir = temp_dir.join(".config").join("weatherman");
        fs::create_dir_all(&config_dir).expect("create config dir");

        let result = load_config_from_dir(&config_dir);
        assert!(result.is_none());

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
