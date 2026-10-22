use super::*;

use std::fs;
use tempfile::TempDir;

#[test]
fn test_resolve_config_path_present() {
    let tmp_dir = TempDir::new().unwrap();
    let path = tmp_dir.path();
    let file_path = path.join(".watchr.toml");

    fs::write(&file_path, "").unwrap();

    let result = resolve_config_path(path).unwrap();

    assert_eq!(result, file_path);
}

#[test]
fn test_resolve_config_path_absent() {
    let tmp_dir = TempDir::new().unwrap();
    let path = tmp_dir.path();

    let result = resolve_config_path(path);
    assert!(matches!(result, Err(ResolverError::NotFound)));
}

#[test]
fn test_resolve_config_path_invalid_path() {
    let tmp_dir = TempDir::new().unwrap();
    let path = tmp_dir.path().join(".watchr.toml");

    let result = resolve_config_path(&path);
    assert!(matches!(result, Err(ResolverError::InvalidPath)));
}

#[test]
fn test_resolve_config_path_file_input() {
    let tmp_dir = TempDir::new().unwrap();
    let path = tmp_dir.path().join("config.toml");
    fs::write(&path, "test").unwrap();

    let result = resolve_config_path(&path).unwrap();
    assert_eq!(result, path);
}

#[test]
fn test_resolve_config_path_in_parent() {
    let tmp_dir = TempDir::new().unwrap();
    let path = tmp_dir.path();
    let conf_path = path.join(".watchr.toml");

    fs::write(&conf_path, "").unwrap();

    let sub_dir = path.join("subdir/subdir1/subdir2");
    fs::create_dir_all(&sub_dir).unwrap();

    let result = resolve_config_path(&sub_dir).unwrap();
    assert_eq!(result, conf_path);
}
