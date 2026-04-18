//! Entry point and orchestration layer for `watchr`
//!
//! # Execution Flow
//! 1. Parse CLI arguments using CLI module
//! 2. Determine command (`is_init` function on cli::Commands)
//!    - `init` -> [`run_init`]
//!    - `watch` -> [`run_watch`]
//! 3. Execute the selected command via `run()`
//! 4. Result `Result<(), MainError>` to the caller
//!
//! The main function invokes `run()` and handles any surfaced
//! errors.
//!
//! This module contains no business logic; all functionality
//! is delegated to command-specific modules
mod cli;
mod config;
mod entry;
mod init;
mod resolver;
mod watcher;

use std::path::PathBuf;

use clap::Parser;
use thiserror::Error;

use crate::cli::{Cli, CliError};
use crate::config::{ConfigError, WatchrConfig, read_config};
use crate::init::{InitError, run_init};
use crate::resolver::{ResolverError, find_config_file};
use crate::watcher::{WatcherError, run_watch};

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
    #[error("IoError: {0}")]
    Io(#[from] std::io::Error),

    /// Raised when no watcher entries exist in config file
    /// (deserialization) or can be resolved from CLI mode.
    ///
    /// **Fix**: Ensure your config file includes at least one
    /// watcher entry, or provide an entry via CLI arguments
    #[error("No watcher entries provided")]
    NoWatcherEntriesProvided,

    /// Raised when directory to watch does not exist
    ///
    /// **Fix**: Verify the path exists and is accessible.
    #[error(
        "Directory not found: {0} (check if path exists and is accessible)"
    )]
    DirNotFound(PathBuf),

    /// Wrapper for errors from init module
    #[error("InitError: {0}")]
    InitError(#[from] InitError),
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
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

/// Main orchestrator for the `watchr` application.
///
/// Parses command-line arguments, initializes configuration
/// file, and starts the file watcher.
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
fn run() -> Result<(), MainError> {
    let cli = Cli::parse();

    if cli.command.is_init() {
        run_init(&std::env::current_dir()?)?;
        println!(".watchr.toml created");
    } else {
        let cli_entry = cli.command.to_entry()?;
        let mut config_path =
            cli.command.config_path().map(|p| p.to_path_buf());

        if config_path.is_none() {
            config_path =
                find_config_file(&std::env::current_dir()?)
                    .ok();
        }

        let config = if let Some(config_path) = config_path {
            read_config(config_path.as_path())?
        } else if let Some(entry) = cli_entry {
            WatchrConfig {
                debounce_ms: 500,
                entries: vec![entry],
            }
        } else {
            return Err(MainError::NoWatcherEntriesProvided);
        };

        if config.entries.is_empty() {
            return Err(MainError::NoWatcherEntriesProvided);
        }

        if let Some(path) = config
            .entries
            .iter()
            .flat_map(|e| e.dirs.iter())
            .find(|p| !p.is_dir())
        {
            return Err(MainError::DirNotFound(
                path.to_path_buf(),
            ));
        }

        run_watch(config)?;
    }
    Ok(())
}
