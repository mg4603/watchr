use super::*;

#[test]
fn error_from_resolve_config_propagates_unchanged() {
    let err = prepare_config(None, None, "start".into())
        .expect_err("should propagate resolve error");
    assert!(matches!(err, ConfigError::NoConfigSource));
}

#[test]
fn error_from_validate_config_propagates_unchanged() {
    let dir = TempDir::new().unwrap().path().to_path_buf();
    let err = prepare_config(Some(entry(vec![])), None, dir)
        .expect_err("should propagate validation error");
    assert!(matches!(err, ConfigError::EmptyWatchDirs));
}

#[test]
fn validation_error_not_resolution_error_for_bad_directory() {
    let dir = TempDir::new().unwrap();
    let err = prepare_config(
        Some(entry(vec![dir.path().join("missing")])),
        None,
        dir.path().to_path_buf(),
    )
    .expect_err("expected to resolve and then fail validation");
    assert!(matches!(err, ConfigError::DirNotFound(_)));
}

#[test]
fn happy_path_end_to_end() {
    let dir = TempDir::new().unwrap();
    let config = prepare_config(
        Some(entry(vec![dir.path().to_path_buf()])),
        None,
        dir.path().to_path_buf(),
    )
    .expect("valid input should succeed");
    assert!(!config.entries.is_empty());
}
