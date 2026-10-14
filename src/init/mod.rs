//! `init` command: generate a `.watchr.toml` template.
//!
//! Creates a commented config file with example watcher
//! entries. See [`InitError`] for failure conditions
use std::fs::File;
use std::io::Write;
use std::path::Path;

use thiserror::Error;

/// Errors during `init` execution.
#[derive(Error, Debug)]
pub enum InitError {
    /// `.watchr.toml` already exists; `init` refuses to
    /// overwrite existing configs to prevent data loss.
    #[error(".watchr.toml already exists")]
    FileAlreadyExists,

    /// File system failure when writing `.watchr.toml`
    /// (e.g. permission denied, disk full, invalid path).
    #[error("failed to write .watchr.toml: {0}")]
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

/// Create a `.watchr.toml` in `path`, failing if it already
/// exists.
///
/// # Errors
///
/// Returns [`InitError`] if the file exists, directory is
/// missing, path is invalid, or writing fails.
/// # Examples
///
/// ```no_run
/// use watchr::init::run_init;
/// use std::path::Path;
///
/// let path = Path::new(".");
/// run_init(path)?;
/// assert!(path.join(".watchr.toml").exists());
/// # Ok::<(), watchr::init::InitError>(())
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

#[cfg(test)]
mod tests;
