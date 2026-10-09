//! Command-line interface parsing and validation.
//!
//! Defines the CLI structure using [`clap`] and extracts
//! configuration from command-line arguments. Supports two commands:
//!
//! - `init` for generating config files
//! - `watch` for starting the file watcher.

use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
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
    Watch {
        /// Directory to watch (CLI mode only)
        dir: Option<PathBuf>,

        /// Comma-separated file extension to filter (e.g.,
        /// `"rs,toml"`); all files if ommitted (CLI mode only)
        #[arg(long)]
        ext: Option<String>,

        /// Command to run on file changes (CLI mode only)
        #[arg(long)]
        cmd: Option<String>,

        /// Explicit path to config file, overriding discovery
        #[arg(long)]
        config: Option<PathBuf>,
    },
}

impl Commands {
    /// Converts CLI arguments into a [`WatcherEntry`] when
    /// `--dir` and `--cmd` are provided.
    ///
    /// Returns `Ok(None)` in config mode (neither flag provided) and
    /// for `Init` command.
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
    ///     ext: Some("rs".to_string()),
    ///     cmd: Some("cargo test".to_string()),
    ///     config: None,
    /// };
    ///
    /// let entry = cmd.to_entry()?;
    /// assert!(entry.is_some());
    /// # Ok::<(), CliError>(())
    /// ```
    pub fn to_entry(
        &self,
    ) -> Result<Option<WatcherEntry>, CliError> {
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
                    Ok(Some(WatcherEntry {
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

    /// Extracts the config file path from `--config` flag.
    ///
    /// Returns `None` for `Init` command or when `--config` was not
    /// provided.
    ///
    /// # Examples
    ///
    /// ```
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
    fn test_to_entry_with_extension() {
        let watch = Commands::Watch {
            dir: Some(PathBuf::from("./")),
            ext: Some("rs,toml".to_string()),
            cmd: Some("cargo test".to_string()),
            config: None,
        };

        let entry = watch.to_entry().unwrap().unwrap();
        assert_eq!(entry.command, "cargo test".to_string());
        assert_eq!(
            entry.ext,
            Some(vec!["rs".to_string(), "toml".to_string()])
        );
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
            Some(WatcherEntry { .. })
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
