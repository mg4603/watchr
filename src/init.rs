use std::fs::File;
use std::io::Write;
use std::path::Path;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum InitError {
    #[error(".watchr.toml already exists")]
    FileAlreadyExists,

    #[error("IoError: {0}")]
    Io(#[from] std::io::Error),
}

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

pub fn run_init(path: &Path) -> Result<(), InitError> {
    let path = path.join(".watchr.toml");
    if path.exists() {
        return Err(InitError::FileAlreadyExists);
    }

    let mut file = File::create(path)?;

    file.write_all(DEFAULT_CONFIG_TEMPLATE.as_bytes())?;
    file.flush()?;
    Ok(())
}
