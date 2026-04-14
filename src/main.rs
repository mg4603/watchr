#![allow(dead_code)]
mod cli;
mod config;
mod entry;
mod resolver;
mod watcher;

use std::path::PathBuf;

use clap::Parser;
use thiserror::Error;

use crate::cli::{Cli, CliError};
use crate::config::{ConfigError, WatchrConfig, read_config};
use crate::resolver::{ResolverError, find_config_file};
use crate::watcher::{WatcherError, run_watch};

#[derive(Error, Debug)]
enum MainError {
    #[error("CliError: {0}")]
    CliError(#[from] CliError),

    #[error("ResolverError: {0}")]
    ResolverError(#[from] ResolverError),

    #[error("ConfigError: {0}")]
    ConfigError(#[from] ConfigError),

    #[error("WatcherError: {0}")]
    WatcherError(#[from] WatcherError),

    #[error("IoError: {0}")]
    Io(#[from] std::io::Error),

    #[error("No watcher entries provided")]
    NoWatcherEntriesProvided,

    #[error("Directory not found: {0}")]
    DirNotFound(PathBuf),
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), MainError> {
    let cli = Cli::parse();

    let cli_entry = cli.command.to_entry()?;
    let mut config_path =
        cli.command.config_path().map(|p| p.to_path_buf());

    if config_path.is_none() {
        config_path =
            find_config_file(&std::env::current_dir()?).ok();
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
        return Err(MainError::DirNotFound(path.to_path_buf()));
    }

    run_watch(config)?;

    Ok(())
}
