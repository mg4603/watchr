use super::*;

use crate::entry::WatcherEntry;
use std::sync::mpsc::channel as mpsc_channel;

fn create_test_entry(dirs: Vec<PathBuf>) -> WatcherEntry {
    WatcherEntry {
        name: Some("test".to_string()),
        dirs,
        ext: Some(vec!["rs".to_string()]),
        command: "echo test".to_string(),
    }
}

#[test]
fn success() {
    let dir = TempDir::new().unwrap();
    let entry = create_test_entry(vec![dir.path().to_path_buf()]);
    let (tx, _rx) = mpsc_channel();

    let result = create_debouncer_for_entry(100, entry, tx);

    assert!(
        result.is_ok(),
        "expected debouncer creation to succeed, got: {:?}",
        result.err()
    );
}

#[test]
fn multiple_directories() {
    let dir = TempDir::new().unwrap();

    let dir1 = dir.path().join("dir1");
    let dir2 = dir.path().join("dir2");

    fs::create_dir_all(&dir1).unwrap();
    fs::create_dir_all(&dir2).unwrap();

    let entry = create_test_entry(vec![dir1, dir2]);
    let (tx, _rx) = mpsc_channel();

    let result = create_debouncer_for_entry(100, entry, tx);

    assert!(
        result.is_ok(),
        "expected debouncer creation with multiple directories to succeed, got:{:?}",
        result.err()
    );
}

#[test]
fn empty_directories() {
    let entry = create_test_entry(vec![]);
    let (tx, _rx) = mpsc_channel();

    let result = create_debouncer_for_entry(100, entry, tx);

    assert!(
        result.is_ok(),
        "expected debouncer creation with no directories to succeed, got: {:?}",
        result.err()
    );
}

#[test]
fn invalid_directory() {
    let dir = TempDir::new().unwrap();
    let invalid_dir = dir.path().join("nonexistent");

    let entry = create_test_entry(vec![invalid_dir.clone()]);
    let (tx, _rx) = mpsc_channel();

    let err = create_debouncer_for_entry(100, entry, tx).unwrap_err();

    assert!(
        matches!(
            err,
            WatcherError::Watch{ref path, ..} if *path == invalid_dir),
        "expected WatcherError::Watch, got: {:?}",
        err
    );
}
