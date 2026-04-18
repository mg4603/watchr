//! Command-line interface parsing and validation.
//!
//! Defines the CLI structure using `clap` and provides methods
//! to extract configuration from command-line arguments.
//! It supports two commands:
//! - `init` for generating config files
//! - `watch` for starting the file watcher.

use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use thiserror::Error;

use crate::entry::WatchrEntry;

/// Command-line interface for watcher.
///
/// Main entry point for parsing user commands and flags.
#[derive(Parser)]
#[command(name = "watchr")]
#[command(
    about = "Watch a directory and execute a given command when changes are made to files in it"
)]
pub struct Cli {
    /// The subcommand to execute.
    #[command(subcommand)]
    pub command: Commands,
}

/// Errors that can occur during CLI argument validation.
#[derive(Error, Debug)]
pub enum CliError {
    /// Raised when `--dir` is provided without `--cmd`,
    /// or vice versa.
    ///
    /// In CLI mode, both flags must be provided together.
    #[error("Entry must include both cmd and dir")]
    MalformedEntry,
}

/// Available subcommands for `watchr`.
///
/// Supports `init` for config file generation and `watch`
/// for starting the file watcher with optional CLI mode.
#[derive(Subcommand)]
pub enum Commands {
    /// Generate a `.watchr.toml` template file.
    ///
    /// Creates a new config file in the current directory
    /// with example watcher entries. Errors if the file
    /// already exists.
    Init,

    /// Start watching for file changes.
    ///
    /// Can operate in two modes:
    /// 1. Config mode: reads `.watchr.toml` from current or
    ///    parent directories
    /// 2. CLI mode: uses `--dir` and `--cmd` flags to define a
    ///    single watcher inline
    Watch {
        /// Directory to watch (CLI mode only).
        ///
        /// Must be used together with `--cmd`. If provided
        /// without `--cmd`, returns `CliError::MalformedEntry`.
        dir: Option<PathBuf>,

        /// File extensions to filter (CLI mode only).
        ///
        /// Comma-separated list (e.g., `"rs,toml"`). If omitted,
        /// all file changes trigger the command.
        #[arg(long)]
        ext: Option<String>,

        /// Command to run on file changes (CLI mode only).
        ///
        /// Must be together with `--dir`. If provided without
        /// `--dir`, returns `CliError::MalformedEntry`.
        #[arg(long)]
        cmd: Option<String>,

        /// Explicit path to config file.
        ///
        /// Overrides the default config resolution (walk up
        /// from current directory). Can be combined with CLI
        /// mode flags.
        #[arg(long)]
        config: Option<PathBuf>,
    },
}

impl Commands {
    /// Converts CLI arguments into a [`WatchrEntry`] when
    /// `--dir` and `--cmd` are both provided.
    ///
    /// # Errors
    ///
    /// Returns [`CliError`] if:
    /// - only one of `--dir` or `--cmd` is provided
    ///
    /// # Examples
    ///
    ///   ```no_run
    ///   use watchr::cli::{Commands, CliError};
    ///   use std::path::PathBuf;
    ///
    ///   let cmd = Commands::Watch {
    ///       dir: Some(PathBuf::from("src/")),
    ///       ext: Some("rs".to_string()),
    ///       cmd: Some("cargo test".to_string()),
    ///       config: None,
    ///   };
    ///
    ///   let entry = cmd.to_entry()?;
    ///   assert!(entry.is_some());
    ///   ```
    pub fn to_entry(
        &self,
    ) -> Result<Option<WatchrEntry>, CliError> {
        match self {
            Commands::Init => Ok(None),
            Commands::Watch { dir, ext, cmd, .. } => {
                let ext = ext.as_ref().map(|ext| {
                    ext.split(',')
                        .map(|x| x.to_string())
                        .collect()
                });

                let dir = dir.as_ref();
                let cmd = cmd.as_ref();

                if (dir.is_none() && cmd.is_some())
                    || (cmd.is_none() && dir.is_some())
                {
                    Err(CliError::MalformedEntry)
                } else if dir.is_some() && cmd.is_some() {
                    Ok(Some(WatchrEntry {
                        name: None,
                        dirs: vec![dir.unwrap().to_path_buf()],
                        ext,
                        command: cmd.unwrap().to_string(),
                    }))
                } else {
                    Ok(None)
                }
            }
        }
    }

    /// Extract the expicit config file path from `--config`
    /// flag.
    ///
    /// Returns `None` for `Init` command or if `--config`
    /// flag was not provided.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use watchr::cli::Commands;
    /// use std::path::PathBuf;
    ///
    /// let cmd = Commands::Watch {
    ///     dir: None,
    ///     ext: None,
    ///     cmd: None,
    ///     config: Some(
    ///         PathBuf::from(".watchr.toml")
    ///     ),
    /// };
    ///
    /// assert!(cmd.config_path().is_some());
    /// ```
    pub fn config_path(&self) -> Option<&Path> {
        match self {
            Commands::Init => None,
            Commands::Watch { config, .. } => {
                config.as_ref().map(|c| c.as_path())
            }
        }
    }

    /// Check if the command is `Init`
    ///
    /// Returns `true` if this is `Init`.
    ///
    /// # Examples
    ///
    /// ```no_run
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
mod tests {
    use super::*;

    #[test]
    fn test_to_entry_init() {
        let init = Commands::Init;
        assert!(init.to_entry().unwrap().is_none());
    }

    #[test]
    fn test_to_entry_watch_dir_none() {
        let watch = Commands::Watch {
            dir: None,
            ext: None,
            config: None,
            cmd: Some("cargo test".to_string()),
        };
        assert!(matches!(
            watch.to_entry(),
            Err(CliError::MalformedEntry)
        ));
    }

    #[test]
    fn test_to_entry_cmd_dir_none() {
        let watch = Commands::Watch {
            dir: None,
            ext: None,
            config: None,
            cmd: None,
        };
        assert!(watch.to_entry().unwrap().is_none())
    }

    #[test]
    fn test_to_entry_cmd_none() {
        let watch = Commands::Watch {
            dir: Some(PathBuf::from("./")),
            ext: None,
            config: None,
            cmd: None,
        };
        assert!(matches!(
            watch.to_entry(),
            Err(CliError::MalformedEntry)
        ));
    }

    #[test]
    fn test_to_entry_happy_path() {
        let watch = Commands::Watch {
            dir: Some(PathBuf::from("./")),
            ext: None,
            config: None,
            cmd: Some("cargo test".to_string()),
        };

        assert!(matches!(
            watch.to_entry().unwrap(),
            Some(WatchrEntry { .. })
        ));
        let entry = watch.to_entry().unwrap().unwrap();
        assert_eq!(entry.dirs, vec![PathBuf::from("./")]);
        assert!(entry.ext.is_none());
        assert_eq!(entry.command, "cargo test".to_string());
    }

    #[test]
    fn test_config_path_init() {
        let init = Commands::Init;
        assert!(init.config_path().is_none());
    }

    #[test]
    fn test_config_path_watch_config_none() {
        let watch = Commands::Watch {
            dir: None,
            ext: None,
            cmd: None,
            config: None,
        };
        assert!(watch.config_path().is_none())
    }

    #[test]
    fn test_config_path_watch_config_is_not_none() {
        let watch = Commands::Watch {
            dir: None,
            ext: None,
            cmd: None,
            config: Some(PathBuf::from("./")),
        };
        assert_eq!(
            watch.config_path(),
            Some(PathBuf::from("./").as_path())
        );
    }

    #[test]
    fn test_is_init_true() {
        let init = Commands::Init;
        assert!(init.is_init());
    }

    #[test]
    fn test_is_init_false() {
        let watch = Commands::Watch {
            dir: None,
            ext: None,
            cmd: None,
            config: None,
        };
        assert!(!watch.is_init());
    }
}
