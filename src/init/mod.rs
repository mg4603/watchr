//! `init` command: generate a `.watchr.toml` template.
//!
//! Creates a commented config file with example watcher
//! entries. See [`InitError`] for failure conditions
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

use thiserror::Error;

/// Errors during `init` execution.
#[derive(Error, Debug)]
pub enum InitError {
    /// Config file already exists; `init` refuses to
    /// overwrite existing configs to prevent data loss.
    #[error("config file already exists: {0}")]
    FileAlreadyExists(std::path::PathBuf),

    /// File system failure when writing config file
    /// (e.g. permission denied, disk full, invalid path).
    #[error("failed to write config file: {0}")]
    Io(#[from] std::io::Error),
}

const CONFIG_FILE_NAME: &str = ".watchr.toml";
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
    let config_path = path.join(CONFIG_FILE_NAME);
    let mut file = open_new(&config_path)?;
    write_template(&mut file, DEFAULT_CONFIG_TEMPLATE)
}

fn open_new(config_path: &Path) -> Result<File, InitError> {
    if config_path.exists() && config_path.is_dir() {
        return Err(InitError::Io(io::Error::new(
            io::ErrorKind::InvalidInput,
            "path is a directory",
        )));
    }

    match File::options().write(true).create_new(true).open(config_path) {
        Ok(f) => Ok(f),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            Err(InitError::FileAlreadyExists(config_path.to_owned()))
        }
        Err(e) => Err(InitError::Io(e)),
    }
}

fn write_template<W: Write>(
    writer: &mut W,
    template: &str,
) -> Result<(), InitError> {
    writer.write_all(template.as_bytes())?;
    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests;
