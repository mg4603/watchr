pub mod cli;
pub mod config;
pub mod entry;
pub mod init;
pub mod resolver;
pub mod tracing;
pub mod watcher;

use std::path::PathBuf;
use thiserror::Error;

use cli::{Cli, CliError, Commands};
use config::{ConfigError, prepare_config};
use init::{InitError, run_init};
use watcher::shutdown::ShutdownHandler;
use watcher::{WatcherError, run_watch};

/// Errors that can occur during `watchr` run.
#[derive(Error, Debug)]
pub enum AppError {
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
    #[error("failed to determine current working directory: {0}")]
    Io(#[from] std::io::Error),

    /// Config file initialization failed.
    #[error("InitError: {0}")]
    InitError(#[from] InitError),
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
pub fn run(
    cli: Cli,
    cwd: PathBuf,
    shutdown_handler: &dyn ShutdownHandler,
    writer: &mut dyn std::io::Write,
) -> Result<(), AppError> {
    match cli.command {
        Commands::Init => {
            run_init(&cwd)?;
            writeln!(writer, ".watchr.toml created").ok();
        }
        Commands::Watch(args) => {
            let config = prepare_config(
                args.to_entry()?,
                args.config_path().map(|p| p.to_path_buf()),
                cwd,
            )?;
            run_watch(config, writer, shutdown_handler)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod test_helpers;
#[cfg(test)]
mod tests;
