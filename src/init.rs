//! Configuration file initialization.
//!
//! Handles the `init` command.
//!
//! Generates a `.watchr.toml` template file in the target
//! directory.
//! The template includes comments and example watcher entries
//! to help users get started quickly.
//!
//! See [`InitError`] for failure conditions.
use std::fs::File;
use std::io::Write;
use std::path::Path;

use thiserror::Error;

/// Errors that occur during `init` command execution.
#[derive(Error, Debug)]
pub enum InitError {
    /// Raised if `.watchr.toml` file already exists in
    /// the target directory.
    ///
    /// The `init` command refuses to overwrite existing
    /// config files to prevent accidental data loss.
    #[error(".watchr.toml already exists")]
    FileAlreadyExists,

    /// Raised when file system operations fail.
    ///
    /// Common causes: permission denied, disk full, or
    /// invalid path.
    #[error("IoError: {0}")]
    Io(#[from] std::io::Error),
}

const DEFAULT_CONFIG_TEMPLATE: &str = r#"
# watchr configuration file
# Debounce time in milliseconds (default: 500)
debounce_ms = 500

# Example watcher entry
# [[watcher]]
# name = "tests"
# dirs = ["src/", "tests/"]
# ext = ["rs", "toml"]
# command = "cargo test"

# Multiple watchers can be defined
# [[watcher]]
# name = "lint"
# dirs = ["src/"]
# command = "cargo clippy"
"#;

/// Initialize a `.watchr.toml` in the given directory.
///
/// Fails if the file already exists. Does not create parent
/// directories.
///
/// # Arguments
/// * `path` - Directory where the `.watchr.toml` file will be
///   created
///
/// # Errors
///
/// Returns [`InitError`] if:
/// - `.watchr.toml` already exists
/// - the directory does not exist, the path is invalid path,
///   or permission is denied
///
/// # Examples
///
/// ```no_run
/// use watchr::init::run_init;
/// use std::path::Path;
///
/// let path = Path::new(".");
/// run_init(path)?;
/// assert!(path.join(".watchr.toml").exists());
/// ```
pub fn run_init(path: &Path) -> Result<(), InitError> {
    let path = path.join(".watchr.toml");
    if path.exists() {
        return Err(InitError::FileAlreadyExists);
    }

    let mut file = File::create(path)?;

    file.write_all(DEFAULT_CONFIG_TEMPLATE.as_bytes())?;
    file.flush()?;
    Ok(())
}
