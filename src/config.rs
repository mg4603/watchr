use std::fs;
use std::path::Path;

use serde::Deserialize;
use thiserror::Error;

use crate::entry::WatchrEntry;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("error reading config file: {0}")]
    Io(#[from] std::io::Error),

    #[error("error deserialzing config file: {0}")]
    Deserialize(#[from] toml::de::Error),
}

#[derive(Debug, Deserialize)]
pub struct WatchrConfig {
    #[serde(default = "default_debounce_ms")]
    pub debounce_ms: u64,

    #[serde(rename = "watcher")]
    pub entries: Vec<WatchrEntry>,
}

fn default_debounce_ms() -> u64 {
    500
}

pub fn read_config(path: &Path) -> Result<WatchrConfig, ConfigError> {
    let config_str = fs::read_to_string(path)?;
    let config: WatchrConfig = toml::from_str(&config_str)?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_config_file(path: &Path, malformed: bool) {
        let injection = (if malformed { "[" } else { "" }).to_string();

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
}
