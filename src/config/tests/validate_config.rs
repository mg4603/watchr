use super::*;

#[test]
fn valid_config_with_existing_dir_returns_ok() {
    let dir = TempDir::new().unwrap();
    let config = WatcherConfig {
        debounce_ms: 500,
        entries: vec![entry(vec![dir.path().to_path_buf()])],
    };
    assert!(validate_config(&config).is_ok());
}

#[test]
fn empty_entries_returns_no_watcher_entries_provided() {
    let config = WatcherConfig { debounce_ms: 500, entries: vec![] };
    let err = validate_config(&config).unwrap_err();
    assert!(matches!(err, ConfigError::NoWatcherEntriesProvided));
}

#[test]
fn nonexistent_dir_returns_dir_not_found() {
    let fake_path = PathBuf::from("nonexistent");
    let config = WatcherConfig {
        debounce_ms: 500,
        entries: vec![entry(vec![fake_path.clone()])],
    };
    let err = validate_config(&config).unwrap_err();
    assert!(matches!(err, ConfigError::DirNotFound(p) if p == fake_path));
}

#[test]
fn file_instead_of_dir_returns_dir_not_found() {
    let file = NamedTempFile::new().unwrap();
    let config = WatcherConfig {
        debounce_ms: 500,
        entries: vec![entry(vec![file.path().to_path_buf()])],
    };
    let err = validate_config(&config).unwrap_err();
    assert!(matches!(err, ConfigError::DirNotFound(_)));
}

#[test]
fn first_bad_dir_across_entries_is_reported() {
    let dir = TempDir::new().unwrap();
    let bad_a = PathBuf::from("/nonexistent/a");
    let bad_b = PathBuf::from("/nonexistent/b");
    let config = WatcherConfig {
        debounce_ms: 500,
        entries: vec![
            entry(vec![dir.path().to_path_buf()]),
            entry(vec![bad_a.clone(), bad_b]),
        ],
    };
    let err = validate_config(&config).unwrap_err();
    assert!(matches!(err, ConfigError::DirNotFound(p) if p == bad_a));
}

#[test]
fn empty_dir_list_per_entry_is_invalid() {
    let config =
        WatcherConfig { debounce_ms: 500, entries: vec![entry(vec![])] };
    let err = validate_config(&config).unwrap_err();
    assert!(matches!(err, ConfigError::EmptyWatchDirs));
}

#[test]
fn multiple_valid_dirs_return_ok() {
    let d1 = TempDir::new().unwrap();
    let d2 = TempDir::new().unwrap();
    let config = WatcherConfig {
        debounce_ms: 500,
        entries: vec![entry(vec![
            d1.path().to_path_buf(),
            d2.path().to_path_buf(),
        ])],
    };
    assert!(validate_config(&config).is_ok())
}
