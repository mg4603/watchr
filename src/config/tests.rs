use super::*;

use crate::entry::WatcherEntry;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn create_config_file(path: &Path, malformed: bool) {
    let injection =
        (if malformed { "[" } else { "" }).to_string();

    let config = format!(
        r####"{}debounce_ms = 500

[[watcher]]
name = "sample_test"
dirs = ["{}"]
command = "pwd"
"####,
        injection,
        path.parent().unwrap().display()
    );

    fs::write(path, config).unwrap()
}

#[test]
fn test_happy_path() {
    let tmp_dir = tempfile::TempDir::new().unwrap();
    let path = tmp_dir.path();
    let file_path = path.join(".watchr.toml");

    create_config_file(&file_path, false);

    let config = read_config(&file_path).unwrap();
    assert_eq!(config.debounce_ms, 500);
    assert_eq!(config.entries.len(), 1);

    let entry = &config.entries[0];

    assert_eq!(entry.name, Some("sample_test".to_string()));
    assert_eq!(entry.dirs.len(), 1);
    assert_eq!(entry.command, "pwd");
    assert!(entry.ext.is_none());
}

#[test]
fn test_path_non_existent() {
    let tmp_dir = tempfile::TempDir::new().unwrap();
    let path = tmp_dir.path();
    let file_path = path.join(".watchr.toml");

    let result = read_config(&file_path);

    assert!(matches!(result, Err(ConfigError::Io(_))))
}

#[test]
fn test_malformed_config() {
    let tmp_dir = tempfile::TempDir::new().unwrap();
    let path = tmp_dir.path();
    let file_path = path.join(".watchr.toml");

    create_config_file(&file_path, true);

    let result = read_config(&file_path);
    assert!(matches!(result, Err(ConfigError::Deserialize(_))));
}

#[test]
fn test_default_debounce_ms() {
    assert_eq!(default_debounce_ms(), 500);
}

#[test]
fn test_validate_config_empty_entries() {
    let config = WatcherConfig {
        debounce_ms: 500,
        entries: vec![],
    };

    assert!(matches!(
        validate_config(&config),
        Err(ConfigError::NoWatcherEntriesProvided)
    ));
}

#[test]
fn test_validate_config_dir_not_found() {
    let config = WatcherConfig {
        debounce_ms: 500,
        entries: vec![WatcherEntry {
            name: None,
            dirs: vec![PathBuf::from("/nonexistent")],
            ext: None,
            command: "echo test".to_string(),
        }],
    };

    assert!(matches!(
        validate_config(&config),
        Err(ConfigError::DirNotFound(_))
    ));
}

#[test]
fn test_resolve_config_from_config_file() {
    let tmp_dir = TempDir::new().unwrap();
    let config_file = tmp_dir.path().join(".watchr.toml");

    let config = format!(
        r#"
[[watcher]]
dirs = ["{}"]
command = "echo test"
"#,
        tmp_dir.path().display()
    );
    fs::write(&config_file, config).unwrap();

    let result = resolve_config(
        None,
        Some(config_file),
        tmp_dir.path().to_path_buf(),
    )
    .unwrap();
    assert_eq!(result.entries.len(), 1);
    assert_eq!(result.entries[0].command, "echo test");
    assert_eq!(result.debounce_ms, 500);
}

#[test]
fn test_resolve_config_no_source() {
    let tmp_dir = TempDir::new().unwrap();

    let original = std::env::current_dir().unwrap();
    std::env::set_current_dir(tmp_dir.path()).unwrap();

    let result = resolve_config(
        None,
        None,
        tmp_dir.path().to_path_buf(),
    );

    std::env::set_current_dir(original).unwrap();

    assert!(matches!(result, Err(ConfigError::NoConfigSource)));
}
