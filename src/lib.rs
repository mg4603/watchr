pub mod cli;
pub mod config;
pub mod entry;
pub mod init;
pub mod resolver;
pub mod tracing;
pub mod watcher;

use thiserror::Error;

use cli::{Cli, CliError, Commands};
use config::{ConfigError, prepare_config};
use init::{InitError, run_init};
use watcher::shutdown::CtrlCHandler;
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
    writer: &mut dyn std::io::Write,
) -> Result<(), AppError> {
    match cli.command {
        Commands::Init => {
            run_init(&std::env::current_dir()?)?;
            writeln!(writer, ".watchr.toml created").ok();
        }
        Commands::Watch(args) => {
            let config = prepare_config(
                args.to_entry()?,
                args.config_path().map(|p| p.to_path_buf()),
                std::env::current_dir()?,
            )?;
            run_watch(config, writer, &CtrlCHandler)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod test_helpers;
