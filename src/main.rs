//! Entry point and orchestration layer for `watchr`
//!
//! # Execution Flow
//! 1. Parse CLI arguments using CLI module
//! 2. Determine command (`is_init` function on cli::Commands)
//!    - `init` -> [`run_init`]
//!    - `watch` -> [`prepare_config`] -> [`run_watch`]
//! 3. Execute the selected command via `run()`
//! 4. Returns `Result<(), MainError>` to the caller
//!
//! The main function invokes `run()` and handles any surfaced
//! errors.
//!
//! This module contains no business logic; all functionality
//! is delegated to command-specific modules

use clap::Parser;
use thiserror::Error;

use watchr::cli::{Cli, CliError};
use watchr::config::{ConfigError, prepare_config};
use watchr::init::{InitError, run_init};

use watchr::tracing::init_tracing;
use watchr::watcher::{WatcherError, run_watch};

/// Errors that can occur during `watchr` run.
#[derive(Error, Debug)]
enum MainError {
    /// Wrapper for errors from CLI module.
    #[error("CliError: {0}")]
    CliError(#[from] CliError),

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
/// used, or prepares configuration and starts the file watcher
/// for the `watch` command.
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
        let config = prepare_config(
            cli.command.to_entry()?,
            cli.command.config_path().map(|p| p.to_path_buf()),
            std::env::current_dir()?,
        )?;
        run_watch(config, writer)?;
    }
    Ok(())
}
