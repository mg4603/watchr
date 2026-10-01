//! Entry point and orchestration layer for `watchr`
//!
//! # Execution Flow
//! 1. Parse CLI arguments using CLI module
//! 2. Determine command (`is_init` function on cli::Commands)
//!    - `init` -> [`run_init`]
//!    - `watch` -> [`run_watch`]
//! 3. Execute the selected command via `run()`
//! 4. Returns `Result<(), MainError>` to the caller
//!
//! The main function invokes `run()` and handles any surfaced
//! errors.
//!
//! This module contains no business logic; all functionality
//! is delegated to command-specific modules

use std::path::PathBuf;

use clap::Parser;
use thiserror::Error;

use watchr::cli::{Cli, CliError};
use watchr::config::{ConfigError, WatcherConfig, read_config};
use watchr::init::{InitError, run_init};
use watchr::resolver::{ResolverError, find_config_file};
use watchr::tracing::init_tracing;
use watchr::watcher::{WatcherError, run_watch};

/// Errors that can occur during `watchr` run.
#[derive(Error, Debug)]
enum MainError {
    /// Wrapper for errors from CLI module.
    #[error("CliError: {0}")]
    CliError(#[from] CliError),

    /// Wrapper for errors from resolver module.
    #[error("ResolverError: {0}")]
    ResolverError(#[from] ResolverError),

    /// Wrapper for errors from config module.
    #[error("ConfigError: {0}")]
    ConfigError(#[from] ConfigError),

    /// Wrapper for errors from watcher module.
    #[error("WatcherError: {0}")]
    WatcherError(#[from] WatcherError),

    /// Raised when file system operations fail.
    ///
    /// **Common causes**: permission denied, invalid path, or
    /// disk full.
    #[error(
        "failed to determine current working directory: {0}"
    )]
    Io(#[from] std::io::Error),

    /// Raised when no watcher entries exist in config file
    /// (deserialization) or can be resolved from CLI mode.
    ///
    /// **Fix**: Ensure your config file includes at least one
    /// watcher entry, or provide an entry via CLI arguments
    #[error("no watcher entries provided")]
    NoWatcherEntriesProvided,

    /// Raised when directory to watch does not exist
    ///
    /// **Fix**: Verify the path exists and is accessible.
    #[error(
        "directory not found: {0} (check if path exists and is accessible)"
    )]
    DirNotFound(PathBuf),

    /// Wrapper for errors from init module
    #[error("InitError: {0}")]
    InitError(#[from] InitError),

    /// Raised when no config source is found (no CLI args
    /// or config file)
    #[error("no config source found")]
    NoConfigSource,
}

/// Entry point for the `watchr` application.
///
/// # Errors
/// If the application fails during execution (e.g., due to
/// `run()` returning an error), this function prints the error
/// to `stderr` and exits with a non-zero status code (1).
///
/// # Examples
/// ```ignore
/// $ watchr
/// ```
fn main() {
    let cli = Cli::parse();
    init_tracing(cli.verbose);
    let mut stdout = std::io::stdout();
    if let Err(e) = run(cli, &mut stdout) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

/// Main orchestrator for the `watchr` application.
///
/// Initializes a configuration file if the `init` command is
/// used, or resolves configuration, validates directories,
/// and starts the file watcher for the `watch` command.
///
/// # Arguments
///
/// * `cli` - Parsed command-line arguments
/// * `writer` - Mutable writer for output (init confirmation,
///   command results)
///
/// # Errors
///
/// Returns a [`MainError`] if:
/// - no watcher entries are provided (either via CLI or
///   config file).
/// - a directory specified in the config or CLI does not
///   exist.
/// - there is an error reading the config file or parsing
///   CLI arguments.
fn run(
    cli: Cli,
    writer: &mut dyn std::io::Write,
) -> Result<(), MainError> {
    if cli.command.is_init() {
        run_init(&std::env::current_dir()?)?;
        writeln!(writer, ".watchr.toml created").ok();
    } else {
        let config = prepare_config(&cli)?;
        run_watch(config, writer)?;
    }
    Ok(())
}

/// Validates the structure of a resolved watcher
/// configuration.
///
/// # Arguments
/// * `config` - Resolved watcher configuration to validate
///
/// # Errors
///
/// Returns [`MainEror`] if:
/// - the config has no watcher entries
/// - a configured directory does not exist or is not
///   accessible
fn validate_config(
    config: &WatcherConfig,
) -> Result<(), MainError> {
    if config.entries.is_empty() {
        return Err(MainError::NoWatcherEntriesProvided);
    }

    if let Some(path) = config
        .entries
        .iter()
        .flat_map(|e| e.dirs.iter())
        .find(|p| !p.is_dir())
    {
        return Err(MainError::DirNotFound(path.to_path_buf()));
    }
    Ok(())
}

/// Resolves the watcher configuration from CLI args or
/// config file.
///
/// Resolution order:
/// 1. CLI args (`DIR`, `--cmd`) if provided
/// 2. Config file (`--config` flag or `.watchr.toml` found by
///    walking up the directory tree)
/// 3. Error if neither is provided
///
/// # Arguments
/// * `cli` - Parsed command-line arguments
///
/// # Errors
///
/// Returns [`MainError`] if:
/// -  no config source found
/// - the config file cannot be read or parsed
/// - CLI args are malformed (e.g. `DIR` without `--cmd`)
fn resolve_config(
    cli: &Cli,
) -> Result<WatcherConfig, MainError> {
    let cli_entry = cli.command.to_entry()?;
    let mut config_path =
        cli.command.config_path().map(|p| p.to_path_buf());

    if cli_entry.is_none() && config_path.is_none() {
        config_path =
            find_config_file(&std::env::current_dir()?).ok();
    }

    if let Some(entry) = cli_entry {
        Ok(WatcherConfig {
            debounce_ms: 500,
            entries: vec![entry],
        })
    } else if let Some(config_path) = config_path {
        Ok(read_config(config_path.as_path())?)
    } else {
        Err(MainError::NoConfigSource)
    }
}

/// Prepares the watcher configuration for use.
///
/// Resolves the configuration source and validates its
/// content before returning it ready for use.
///
/// # Arguments
/// * `cli` - Parsed command-line arguments
///
/// # Errors
///
/// Returns [`MainError`] if:
/// - no config source is found
/// - the config file cannot be read or parsed
/// - the config validation fails
fn prepare_config(
    cli: &Cli,
) -> Result<WatcherConfig, MainError> {
    let config = resolve_config(cli)?;
    validate_config(&config)?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;
    use watchr::entry::WatcherEntry;

    #[test]
    fn test_validate_config_empty_entries() {
        let config = WatcherConfig {
            debounce_ms: 500,
            entries: vec![],
        };

        assert!(matches!(
            validate_config(&config),
            Err(MainError::NoWatcherEntriesProvided)
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
            Err(MainError::DirNotFound(_))
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

        let cli = Cli::parse_from([
            "watchr",
            "watch",
            "--config",
            config_file.to_str().unwrap(),
        ]);

        let result = resolve_config(&cli).unwrap();
        assert_eq!(result.entries.len(), 1);
        assert_eq!(result.entries[0].command, "echo test");
        assert_eq!(result.debounce_ms, 500);
    }

    #[test]
    fn test_resolve_config_no_source() {
        let tmp_dir = TempDir::new().unwrap();

        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(tmp_dir.path()).unwrap();

        let cli = Cli::parse_from(["watchr", "watch"]);
        let result = resolve_config(&cli);

        std::env::set_current_dir(original).unwrap();

        assert!(matches!(
            result,
            Err(MainError::NoConfigSource)
        ));
    }
}
