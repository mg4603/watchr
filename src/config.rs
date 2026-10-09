//! Configuration file parsing, resolution, and validation
//!
//! Reads `.watchr.toml` files, deserializes them into
//! [`WatcherConfig`] structs, resolves the configuration source
//! (CLI args or config file), and validates the result.
//!
//! Resolution order:
/// 1. CLI args (`--dir`, `--cmd`) if provided
/// 2. Explicit config file path (`--config`) if provided
/// 3. `.watchr.toml` discovered by walking up from `start_dir`
/// 4. [`ConfigError::NoConfigSource`] if none of the above
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use thiserror::Error;

use crate::entry::WatcherEntry;
use crate::resolver::{ResolverError, find_config_file};

/// Error reading, parsing, or validating a configuration.
#[derive(Error, Debug)]
pub enum ConfigError {
    /// I/O error while reading the config file (e.g. permission
    /// denied, invalid path).
    #[error("error reading config file: {0}")]
    Io(#[from] std::io::Error),

    /// TOML deserialization error.
    #[error("error deserializing config file: {0}")]
    Deserialize(#[from] toml::de::Error),

    /// Error from resolver module.
    #[error("ResolverError: {0}")]
    ResolverError(#[from] ResolverError),

    /// No watcher entries in the config file or resolvable
    /// from CLI mode.
    /// **Fix**: Add at least one `[[watcher]]` entry or
    /// provide an entry via CLI arguments
    #[error("no watcher entries provided")]
    NoWatcherEntriesProvided,

    /// A watched directory does not exist or is not accessible
    /// **Fix**: Verify the path exists and is accessible.
    #[error(
        "directory not found: {0} (check if path exists and is accessible)"
    )]
    DirNotFound(PathBuf),

    /// No config source found (neither CLI args nor config file).
    #[error("no config source found")]
    NoConfigSource,
}

/// Configuration for watchr, deserialized from `.watchr.toml`.
/// Holds global settings and a list of watcher entries.
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
pub struct WatcherConfig {
    /// Debounce time in milliseconds; groups rapid file changes.
    /// Defaults to 500ms.
    #[serde(default = "default_debounce_ms")]
    pub debounce_ms: u64,

    /// Watcher entries; each defines directories to watch,
    /// optional extension filters, and a command to execute.
    /// Corresponds to `[[watcher]]` section in TOML.
    #[serde(rename = "watcher")]
    pub entries: Vec<WatcherEntry>,
}

// Default `debounce_ms` used when absent from config file.
fn default_debounce_ms() -> u64 {
    500
}

// Reads and parses a watchr configuration file.
fn read_config(
    path: &Path,
) -> Result<WatcherConfig, ConfigError> {
    let config_str = fs::read_to_string(path)?;
    let config: WatcherConfig = toml::from_str(&config_str)?;
    Ok(config)
}

fn resolve_config(
    entry: Option<WatcherEntry>,
    config_path: Option<PathBuf>,
    start_dir: PathBuf,
) -> Result<WatcherConfig, ConfigError> {
    let mut resolved_config_path = config_path;
    if entry.is_none() && resolved_config_path.is_none() {
        resolved_config_path =
            find_config_file(&start_dir).ok();
    }

    if let Some(entry) = entry {
        Ok(WatcherConfig {
            debounce_ms: default_debounce_ms(),
            entries: vec![entry],
        })
    } else if let Some(resolved_config_path) =
        resolved_config_path
    {
        Ok(read_config(resolved_config_path.as_path())?)
    } else {
        Err(ConfigError::NoConfigSource)
    }
}

fn validate_config(
    config: &WatcherConfig,
) -> Result<(), ConfigError> {
    if config.entries.is_empty() {
        return Err(ConfigError::NoWatcherEntriesProvided);
    }

    if let Some(path) = config
        .entries
        .iter()
        .flat_map(|e| e.dirs.iter())
        .find(|p| !p.is_dir())
    {
        return Err(ConfigError::DirNotFound(
            path.to_path_buf(),
        ));
    }
    Ok(())
}

/// Resolves and validates the configuration, returning it
/// ready for use.
///
/// # Errors
///
/// Returns [`ConfigError`] if:
/// - no config source is found (neither CLI args nor config
///   file)
/// - the config file cannot be read or parsed
/// - the config has no watcher entries
/// - a configured directory does not exist or is not accessible
pub fn prepare_config(
    entry: Option<WatcherEntry>,
    config_path: Option<PathBuf>,
    start_dir: PathBuf,
) -> Result<WatcherConfig, ConfigError> {
    let config = resolve_config(entry, config_path, start_dir)?;
    validate_config(&config)?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::entry::WatcherEntry;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

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

    #[test]
    fn test_default_debounce_ms() {
        assert_eq!(default_debounce_ms(), 500);
    }

    #[test]
    fn test_validate_config_empty_entries() {
        let config = WatcherConfig {
            debounce_ms: 500,
            entries: vec![],
        };

        assert!(matches!(
            validate_config(&config),
            Err(ConfigError::NoWatcherEntriesProvided)
        ));
    }

    #[test]
    fn test_validate_config_dir_not_found() {
        let config = WatcherConfig {
            debounce_ms: 500,
            entries: vec![WatcherEntry {
                name: None,
                dirs: vec![PathBuf::from("/nonexistent")],
                ext: None,
                command: "echo test".to_string(),
            }],
        };

        assert!(matches!(
            validate_config(&config),
            Err(ConfigError::DirNotFound(_))
        ));
    }

    #[test]
    fn test_resolve_config_from_config_file() {
        let tmp_dir = TempDir::new().unwrap();
        let config_file = tmp_dir.path().join(".watchr.toml");

        let config = format!(
            r#"
[[watcher]]
dirs = ["{}"]
command = "echo test"
"#,
            tmp_dir.path().display()
        );
        fs::write(&config_file, config).unwrap();

        let result = resolve_config(
            None,
            Some(config_file),
            tmp_dir.path().to_path_buf(),
        )
        .unwrap();
        assert_eq!(result.entries.len(), 1);
        assert_eq!(result.entries[0].command, "echo test");
        assert_eq!(result.debounce_ms, 500);
    }

    #[test]
    fn test_resolve_config_no_source() {
        let tmp_dir = TempDir::new().unwrap();

        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(tmp_dir.path()).unwrap();

        let result = resolve_config(
            None,
            None,
            tmp_dir.path().to_path_buf(),
        );

        std::env::set_current_dir(original).unwrap();

        assert!(matches!(
            result,
            Err(ConfigError::NoConfigSource)
        ));
    }
}
