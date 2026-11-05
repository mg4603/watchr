use super::*;

use crate::entry::WatcherEntry;
use std::fs;
use std::path::PathBuf;
use tempfile::{NamedTempFile, TempDir};

mod prepare_config;
mod read_config;
mod resolve_config;
mod validate_config;

fn write_config(dir: &TempDir, contents: &str) -> std::path::PathBuf {
    let path = dir.path().join(".watchr.toml");
    fs::write(&path, contents).unwrap();
    path
}

fn valid_toml() -> &'static str {
    r####"debounce_ms = 500
[[watcher]]
name = "test"
dirs = ["/tmp"]
command = "echo test"
"####
}

fn wrong_type_toml() -> &'static str {
    r####"debounce_ms = "500"
[[watcher]]
name = "test"
dirs = ["/tmp"]
command = "echo test"
"####
}

fn entry(dirs: Vec<PathBuf>) -> WatcherEntry {
    WatcherEntry {
        name: Some("test".to_string()),
        dirs,
        ext: None,
        command: "echo test".to_string(),
    }
}

// ------------ error propagation semantics ---------
#[test]
fn config_error_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ConfigError>();
    assert_send_sync::<Result<WatcherConfig, ConfigError>>();
}

#[test]
fn test_default_debounce_ms() {
    assert_eq!(default_debounce_ms(), 500);
}
