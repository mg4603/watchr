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
//! All functionality is delegated to command-specific modules.
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
    /// CLI parsing or validation failed.
    #[error("CliError: {0}")]
    CliError(#[from] CliError),

    /// Config resolution, parsing or validation failed.
    #[error("ConfigError: {0}")]
    ConfigError(#[from] ConfigError),

    /// Watcher initialization or shutdown-handler setup failed.
    #[error("WatcherError: {0}")]
    WatcherError(#[from] WatcherError),

    /// Filesystem I/O failed (e.g. permission denied,
    /// invalid path).
    #[error(
        "failed to determine current working directory: {0}"
    )]
    Io(#[from] std::io::Error),

    /// Config file initialization failed.
    #[error("InitError: {0}")]
    InitError(#[from] InitError),
}

fn main() {
    let cli = Cli::parse();
    init_tracing(cli.verbose);
    let mut stdout = std::io::stdout();
    if let Err(e) = run(cli, &mut stdout) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

/// Runs the selected command: `init` creates a config file,
/// `watch` prepares the config and starts the file watcher.
///
/// # Errors
///
/// Returns a [`MainError`] if:
/// - the config file cannot be read
/// - no watcher entries are provided
/// - a watched directory does not exist
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
