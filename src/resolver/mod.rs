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

    // Path is neither a file nor a directory
    #[error("invalid path")]
    InvalidPath,
}

/// Find `.watchr.toml` in `path` or, failing that, in its
/// parent directories, up to the filesystem root.
///
/// `path` should be a directory; a file path is treated as
/// its containing directory.
///
/// Returns a `[ResolverError]` if no config file is found.
pub fn resolve_config_path(
    path: &Path,
) -> Result<PathBuf, ResolverError> {
    if path.is_file() {
        return Ok(path.to_path_buf());
    }

    if !path.is_dir() {
        return Err(ResolverError::InvalidPath);
    }

    let mut current = path.to_path_buf();
    loop {
        let candidate = current.join(".watchr.toml");
        if candidate.is_file() {
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
