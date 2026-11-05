use super::*;

use crate::entry::WatcherEntry;
use std::fs;
use std::path::PathBuf;
use tempfile::{NamedTempFile, TempDir};

mod prepare_config;
mod read_config;
mod validate_config;

fn write_config(dir: &TempDir, contents: &str) -> std::path::PathBuf {
    let path = dir.path().join(".watchr.toml");
    fs::write(&path, contents).unwrap();
    path
}

fn valid_toml() -> String {
    r####"debounce_ms = 500
[[watcher]]
name = "sample_test"
dirs = ["/tmp"]
command = "pwd"
"####
        .to_string()
}

fn wrong_type_toml() -> String {
    r####"debounce_ms = "500"
[[watcher]]
name = "sample_test"
dirs = ["/tmp"]
command = "pwd"
"####
        .to_string()
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

mod resolve_config_tests {
    use super::*;

    #[test]
    fn from_config_file() {
        let tmp = TempDir::new().unwrap();
        let config_file = tmp.path().join("conf.toml");

        let config = format!(
            r#"
[[watcher]]
dirs = ["{}"]
command = "echo test"
"#,
            tmp.path().display()
        );
        fs::write(&config_file, config).unwrap();

        let result = resolve_config(
            None,
            Some(config_file),
            tmp.path().to_path_buf(),
        )
        .unwrap();
        assert_eq!(result.entries.len(), 1);
        assert_eq!(result.entries[0].command, "echo test");
        assert_eq!(result.debounce_ms, 500);
    }

    #[test]
    fn from_cli_entry() {
        let tmp = TempDir::new().unwrap();
        let entry = WatcherEntry {
            name: Some("cli_test".to_string()),
            dirs: vec![tmp.path().to_path_buf()],
            ext: None,
            command: "echo test".to_string(),
        };

        let result =
            resolve_config(Some(entry), None, tmp.path().to_path_buf())
                .unwrap();

        assert_eq!(result.entries.len(), 1);
        assert_eq!(result.entries[0].name, Some("cli_test".to_string()));
        assert_eq!(result.entries[0].command, "echo test");
        assert_eq!(result.debounce_ms, 500);
    }

    #[test]
    fn no_source() {
        let tmp = TempDir::new().unwrap();
        let result = resolve_config(None, None, tmp.path().to_path_buf());
        assert!(matches!(result, Err(ConfigError::NoConfigSource)));
    }

    #[test]
    fn no_entry_no_config_path() {
        let tmp = TempDir::new().unwrap();
        let conf_file = tmp.path().join(".watchr.toml");
        let conf = format!(
            r#"
[[watcher]]
dirs = ["{}"]
command = "echo test"
"#,
            tmp.path().display()
        );
        fs::write(&conf_file, conf).unwrap();
        let result =
            resolve_config(None, None, tmp.path().to_path_buf()).unwrap();
        assert_eq!(result.entries.len(), 1);
        assert_eq!(result.entries[0].command, "echo test");
        assert_eq!(result.debounce_ms, 500);
    }

    #[test]
    fn wrong_config_path() {
        let tmp = TempDir::new().unwrap();
        let conf_file = tmp.path().join(".watchr.toml");
        let err = resolve_config(
            None,
            Some(conf_file),
            tmp.path().to_path_buf(),
        )
        .unwrap_err();
        assert!(matches!(err, ConfigError::Io(_)));
    }
}
