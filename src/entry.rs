//! Watcher entry data structure.
//!
//! This module defines `WatchrEntry`, which represents a
//! single watcher configuration. Each entry specifies
//! which directories to monitor, optional file extension
//! filters, and the command to execute on file changes.
//!
//! Entries are typically deserialzed from `[[watcher]]`
//! sections in `.watchr.toml` files or created from CLI
//! arguments.
use std::path::PathBuf;

use serde::Deserialize;

/// A single watcher entry defining what to watch and what to run.
///
/// Represents one `[[watcher]]` section in the config file.
///
/// # Example
///
/// ```toml
/// [[watcher]]
/// name = "rust-build"
/// dirs = ["src"]
/// ext = ["rs"]
/// command = "cargo build"
/// ```
#[derive(Debug, Deserialize)]
pub struct WatchrEntry {
    /// Optional descriptive name for this watcher
    #[allow(dead_code)]
    pub name: Option<String>,

    /// Directories to watch for changes.
    ///
    /// Paths may be absolute or relative to the working
    /// directory.
    pub dirs: Vec<PathBuf>,

    /// File extensions to filter (e.g., ["rs", "toml"]).
    ///
    /// File extensions should not include the leading dot.
    /// If `None`, all file changes trigger the command.
    pub ext: Option<Vec<String>>,

    /// Shell command to execute when files change.
    ///
    /// Executed in the current working directory.
    pub command: String,
}
