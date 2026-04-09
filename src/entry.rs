use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct WatchrEntry {
    pub name: Option<String>,
    pub dirs: Vec<PathBuf>,
    pub ext: Option<Vec<String>>,
    pub command: String,
}
