//! Command-line interface parsing and validation.
//!
//! Defines the CLI structure using [`clap`] and extracts
//! configuration from command-line arguments. Supports two commands:
//!
//! - `init` for generating config files
//! - `watch` for starting the file watcher.

use std::path::{Path, PathBuf};

use clap::{Args, Parser, Subcommand};
use thiserror::Error;

use crate::entry::WatcherEntry;

/// Command-line interface for watcher.
#[derive(Parser)]
#[command(name = "watchr")]
#[command(
    about = "Watch a directory and execute a given command when changes are made to files in it"
)]
pub struct Cli {
    /// The subcommand to run (`init` or `watch`)
    #[command(subcommand)]
    pub command: Commands,

    /// Increase logging verbosity (`-v`, `-vv`, `-vvv`)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

/// Errors that can occur during CLI argument validation.
#[derive(Error, Debug)]
pub enum CliError {
    /// Returned when exactly one of `--dir` or `--cmd` is given.
    #[error("entry must include both cmd and dir")]
    MalformedEntry,
}

/// Available subcommands for `watchr`.
#[derive(Subcommand)]
pub enum Commands {
    /// Generate a `.watchr.toml` template file in the current
    /// directory, with example watcher entries
    ///
    /// Errors if the file already exists.
    Init,

    /// Start watching for file changes.
    ///
    /// Reads from a `.watchr.toml` (either `--config` or
    /// discovered by walking up from the cwd), or defines a
    /// single watcher inline via `--dir` and `--cmd`.
    Watch(WatchArgs),
}

#[derive(Args, Debug)]
pub struct WatchArgs {
    /// Directory to watch (CLI mode only)
    dir: Option<PathBuf>,

    /// Comma-separated file extension to filter (e.g.,
    /// `"rs,toml"`); all files if ommitted (CLI mode only)
    #[arg(long, value_delimiter = ',')]
    ext: Option<Vec<String>>,

    /// Command to run on file changes (CLI mode only)
    #[arg(long)]
    cmd: Option<String>,

    /// Explicit path to config file, overriding discovery
    #[arg(long)]
    config: Option<PathBuf>,
}

impl WatchArgs {
    /// Converts CLI arguments into a [`WatcherEntry`] when
    /// `--dir` and `--cmd` are provided.
    ///
    /// Returns `Ok(None)` in config mode (neither flag provided).
    ///
    /// # Errors
    ///
    /// Returns [`CliError::MalformedEntry`] if only one of `--dir`
    /// or `--cmd` is provided
    ///
    /// # Examples
    ///
    /// ```
    /// use watchr::cli::{Commands, CliError};
    /// use std::path::PathBuf;
    ///
    /// let cmd = Commands::Watch {
    ///     dir: Some(PathBuf::from("src/")),
    ///     ext: Some(vec!["rs".to_string()]),
    ///     cmd: Some("cargo test".to_string()),
    ///     config: None,
    /// };
    ///
    /// let entry = cmd.to_entry()?;
    /// assert!(entry.is_some());
    /// # Ok::<(), CliError>(())
    /// ```
    pub fn to_entry(&self) -> Result<Option<WatcherEntry>, CliError> {
        match (&self.dir, &self.cmd) {
            (Some(_), None) | (None, Some(_)) => {
                Err(CliError::MalformedEntry)
            }
            (Some(dir), Some(cmd)) => Ok(Some(WatcherEntry {
                name: None,
                dirs: vec![dir.to_path_buf()],
                ext: self.ext.clone(),
                command: cmd.to_string(),
            })),
            (None, None) => Ok(None),
        }
    }

    /// Extracts the config file path from `--config` flag.
    ///
    /// Returns `None` when `--config` is not provided.
    pub fn config_path(&self) -> Option<&Path> {
        self.config.as_deref()
    }
}

impl Commands {
    /// Returns `true` if this is the [`Commands::Init`] command.
    ///
    /// # Examples
    ///
    /// ```
    /// use watchr::cli::Commands;
    ///
    /// let cmd = Commands::Init;
    /// assert!(cmd.is_init());
    ///
    /// let cmd = Commands::Watch {
    ///     dir: None,
    ///     ext: None,
    ///     cmd: None,
    ///     config: None,
    /// };
    /// assert!(!cmd.is_init());
    /// ```
    pub fn is_init(&self) -> bool {
        match self {
            Commands::Init => true,
            Commands::Watch { .. } => false,
        }
    }
}

#[cfg(test)]
mod tests;
