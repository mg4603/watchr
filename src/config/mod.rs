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
fn read_config(path: &Path) -> Result<WatcherConfig, ConfigError> {
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
        resolved_config_path = find_config_file(&start_dir).ok();
    }

    if let Some(entry) = entry {
        Ok(WatcherConfig {
            debounce_ms: default_debounce_ms(),
            entries: vec![entry],
        })
    } else if let Some(resolved_config_path) = resolved_config_path {
        Ok(read_config(resolved_config_path.as_path())?)
    } else {
        Err(ConfigError::NoConfigSource)
    }
}

fn validate_config(config: &WatcherConfig) -> Result<(), ConfigError> {
    if config.entries.is_empty() {
        return Err(ConfigError::NoWatcherEntriesProvided);
    }

    if let Some(path) = config
        .entries
        .iter()
        .flat_map(|e| e.dirs.iter())
        .find(|p| !p.is_dir())
    {
        return Err(ConfigError::DirNotFound(path.to_path_buf()));
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
mod tests;
