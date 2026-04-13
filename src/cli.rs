use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use thiserror::Error;

use crate::entry::WatchrEntry;

#[derive(Parser)]
#[command(name = "watchr")]
#[command(
    about = "Watch a directory and execute a given command when changes are made to files in it"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Error, Debug)]
pub enum CliError {
    #[error("Entry must have atleast cmd and dir")]
    MalformedEntry,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate config file
    Init,

    /// Watch specified directories for changes to files with
    /// specified extensions and execute specified command on
    /// change
    Watch {
        /// Directory to watch
        dir: Option<PathBuf>,

        /// Path to .watchr.toml config file
        #[arg(long)]
        config: Option<PathBuf>,

        /// Extensions of files that should be watched
        #[arg(long)]
        ext: Option<String>,

        /// command that should be executed if changes are made
        /// to file being watched
        #[arg(long)]
        cmd: Option<String>,
    },
}

impl Commands {
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

    pub fn config_path(&self) -> Option<&Path> {
        match self {
            Commands::Init => None,
            Commands::Watch { config, .. } => {
                config.as_ref().map(|c| c.as_path())
            }
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
}
