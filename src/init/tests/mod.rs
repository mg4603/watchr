use super::*;

use crate::test_helpers::{FailingWriter, FlushFailingWriter};

use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use tempfile::TempDir;

mod open_new;
mod run_init;
mod write_template;

fn config_path(dir: &TempDir) -> std::path::PathBuf {
    dir.path().join(".watchr.toml")
}

#[test]
fn io_error_from_conversion() {
    let io_err =
        std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
    let err: InitError = io_err.into();
    assert_eq!(err.to_string(), "failed to write config file: not found");
}

#[test]
fn error_display_messages() {
    let path = PathBuf::from("/tmp/config.toml");
    let err = InitError::FileAlreadyExists(path.clone());
    assert_eq!(
        err.to_string(),
        format!("config file already exists: {}", path.display())
    );
    assert!(matches!(err, InitError::FileAlreadyExists(p) if p == path));

    let err = InitError::Io(io::Error::other("denied"));
    assert_eq!(err.to_string(), "failed to write config file: denied");
}
