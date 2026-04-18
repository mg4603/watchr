//! Configuration file parsing and validation.
//!
//! This module handles reading `.watchr.toml` files and
//! deserializing them into `WatchrConfig` structs.
use std::fs;
use std::path::Path;

use serde::Deserialize;
use thiserror::Error;

use crate::entry::WatchrEntry;

/// Errors that can occur while reading or parsing a configuration
/// file.
#[derive(Error, Debug)]
pub enum ConfigError {
    /// File system error while reading the configuration file.
    ///
    /// Common causes: permission denied, or invalid path.
    #[error("error reading config file: {0}")]
    Io(#[from] std::io::Error),

    /// TOML deserialization error while parsing the configuration
    /// file.
    #[error("error deserializing config file: {0}")]
    Deserialize(#[from] toml::de::Error),
}

/// Configuration for watchr.
///
/// Deserialized from `.watchr.toml` files. Contains global
/// settings and a list of watcher entries.
///
/// # Examples
///
/// ```toml
/// debounce_ms = 500
///
/// [[watcher]]
/// name = "tests"
/// dirs = ["src/"]
/// ext = ["rs"]
/// command = "cargo test"
/// ```
#[derive(Debug, Deserialize)]
pub struct WatchrConfig {
    /// Debounce time in milliseconds.
    ///
    /// Groups rapid file changes within this window.
    /// Defaults to 500ms.
    #[serde(default = "default_debounce_ms")]
    pub debounce_ms: u64,

    /// List of watcher entries.
    ///
    /// Each entry defines directories to watch, optional
    /// file extension filters and a command to execute.
    /// Corresponds to `[[watcher]]` section in TOML.
    #[serde(rename = "watcher")]
    pub entries: Vec<WatchrEntry>,
}

/// Default debounce time in milliseconds.
///
/// Used by serde when `debounce_ms` is not specified
/// in the config file.
fn default_debounce_ms() -> u64 {
    500
}

/// Read and parse a watchr configuration file.
///
/// # Arguments
/// * `path` - Path to the `.watchr.toml` file
///
/// # Errors:
///
/// Returns a [`ConfigError`] if:
/// - the file cannot be read or does not exist
/// - the TOML is invalid or does not match the expected schema.
///
/// # Examples
///
/// ```no_run
/// use watchr::config::read_config;
/// use std::path::Path;
///
/// let config = read_config(Path::new(".watchr.toml"))?;
/// println!("Debounce: {}ms", config.debounce_ms);
/// ```
pub fn read_config(
    path: &Path,
) -> Result<WatchrConfig, ConfigError> {
    let config_str = fs::read_to_string(path)?;
    let config: WatchrConfig = toml::from_str(&config_str)?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_config_file(path: &Path, malformed: bool) {
        let injection =
            (if malformed { "[" } else { "" }).to_string();

        let config = format!(
            r####"{}debounce_ms = 500

[[watcher]]
name = "sample_test"
dirs = ["{}"]
command = "pwd"
"####,
            injection,
            path.parent().unwrap().display()
        );

        fs::write(path, config).unwrap()
    }

    #[test]
    fn test_happy_path() {
        let tmp_dir = tempfile::TempDir::new().unwrap();
        let path = tmp_dir.path();
        let file_path = path.join(".watchr.toml");

        create_config_file(&file_path, false);

        let config = read_config(&file_path).unwrap();
        assert_eq!(config.debounce_ms, 500);
        assert_eq!(config.entries.len(), 1);

        let entry = &config.entries[0];

        assert_eq!(entry.name, Some("sample_test".to_string()));
        assert_eq!(entry.dirs.len(), 1);
        assert_eq!(entry.command, "pwd");
        assert!(entry.ext.is_none());
    }

    #[test]
    fn test_path_non_existent() {
        let tmp_dir = tempfile::TempDir::new().unwrap();
        let path = tmp_dir.path();
        let file_path = path.join(".watchr.toml");

        let result = read_config(&file_path);

        assert!(matches!(result, Err(ConfigError::Io(_))))
    }

    #[test]
    fn test_malformed_config() {
        let tmp_dir = tempfile::TempDir::new().unwrap();
        let path = tmp_dir.path();
        let file_path = path.join(".watchr.toml");

        create_config_file(&file_path, true);

        let result = read_config(&file_path);
        assert!(matches!(
            result,
            Err(ConfigError::Deserialize(_))
        ));
    }
}
