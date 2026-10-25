//! Find the configuration file at the specified directory
//! or by walking up the directory tree.
use std::path::{Path, PathBuf};

use thiserror::Error;

/// Error returned when resolving the config file location.
#[derive(Debug, Error)]
pub enum ResolverError {
    /// No config file in the directory or its parents.
    #[error("config file not found")]
    NotFound,

    // Path is neither a file nor a directory
    #[error("invalid path")]
    InvalidPath,
}

/// Resolve the location of the config file.
///
/// - If `path` is a file, it is returned as-is.
/// - If it is a directory, `.watchr.toml` is searched for there and in
///   each parent directory, up to the filesystem root
///
/// Returns a `[ResolverError]` if `path` is neither file nor directory,
/// or if no config file is found.
pub fn resolve_config_path(
    path: &Path,
) -> Result<PathBuf, ResolverError> {
    let path =
        path.canonicalize().map_err(|_| ResolverError::InvalidPath)?;
    if path.is_file() {
        return Ok(path.to_path_buf());
    }
    find_in_tree(&path, ".watchr.toml")
}

fn find_in_tree(
    start: &Path,
    file_name: &str,
) -> Result<PathBuf, ResolverError> {
    let mut current = start.to_path_buf();
    loop {
        let candidate = current.join(file_name);
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
