//! `WatcherEntry`: a single watcher configuration specifying
//! directories to monitor, optional extension filters, and the
//! command to run on changes.
//!
//! Entries comes from `[[watcher]]` sections in `.watchr.toml`
//! files or from CLI arguments.
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
pub struct WatcherEntry {
    /// Optional descriptive name.
    #[allow(dead_code)]
    pub name: Option<String>,

    /// Directories to watch; paths can be absolute or relative
    /// to the working directory.
    pub dirs: Vec<PathBuf>,

    /// Extensions to filter (e.g. `["rs", "toml"]`), without
    /// the leading dot. `None` matches all files.
    pub ext: Option<Vec<String>>,

    /// Shell command to run on changes, executed in the working
    /// directory.
    pub command: String,
}
