//! Find the configuration file at the specified directory
//! or by walking up the directory tree.
use std::path::{Path, PathBuf};

use thiserror::Error;

/// Error when the config file cannot be found.
#[derive(Debug, Error)]
pub enum ResolverError {
    /// No config file in the directory or its parents.
    #[error("config file not found")]
    NotFound,
}

/// Find `.watchr.toml` in `path` or, failing that, in its
/// parent directories, up to the filesystem root.
///
/// `path` should be a directory; a file path is treated as
/// its containing directory.
///
/// Returns a `[ResolverError]` if no config file is found.
pub fn find_config_file(path: &Path) -> Result<PathBuf, ResolverError> {
    let mut current = path.to_path_buf();

    loop {
        let candidate = current.join(".watchr.toml");
        if candidate.exists() {
            return Ok(candidate);
        }
        if !current.pop() {
            break;
        }
    }
    Err(ResolverError::NotFound)
}

#[cfg(test)]
mod tests;
