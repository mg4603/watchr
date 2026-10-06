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
pub fn find_config_file(
    path: &Path,
) -> Result<PathBuf, ResolverError> {
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
mod tests {
    use super::*;

    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_config_file_present() {
        let tmp_dir = TempDir::new().unwrap();
        let path = tmp_dir.path();
        let file_path = path.join(".watchr.toml");

        fs::write(&file_path, "").unwrap();

        let result = find_config_file(path).unwrap();

        assert_eq!(result, file_path)
    }

    #[test]
    fn test_config_file_absent() {
        let tmp_dir = TempDir::new().unwrap();
        let path = tmp_dir.path();

        let result = find_config_file(path);
        assert!(matches!(result, Err(ResolverError::NotFound)))
    }

    #[test]
    fn test_config_file_present_in_parent() {
        let tmp_dir = TempDir::new().unwrap();
        let path = tmp_dir.path();
        let conf_path = path.join(".watchr.toml");

        fs::write(&conf_path, "").unwrap();

        let sub_dir = path.join("subdir/subdir1/subdir2");
        fs::create_dir_all(&sub_dir).unwrap();

        let result = find_config_file(&sub_dir).unwrap();
        assert_eq!(result, conf_path);
    }
}
