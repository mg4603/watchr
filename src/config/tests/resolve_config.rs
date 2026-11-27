use super::*;

#[test]
fn auto_discovers_config_file() {
    let dir = TempDir::new().unwrap();
    let _ = write_config(&dir, valid_toml());
    let result =
        resolve_config(None, None, dir.path().to_path_buf()).unwrap();

    assert_eq!(result.entries.len(), 1);
    assert_eq!(result.entries[0].command, "echo test");
    assert_eq!(result.debounce_ms, 500);
}

#[test]
fn from_cli_entry() {
    let dir = TempDir::new().unwrap();
    let result = resolve_config(
        Some(entry(vec![dir.path().to_path_buf()])),
        None,
        dir.path().to_path_buf(),
    )
    .unwrap();

    assert_eq!(result.entries.len(), 1);
    assert_eq!(result.entries[0].name, Some("test".to_string()));
    assert_eq!(result.entries[0].command, "echo test");
    assert_eq!(result.debounce_ms, 500);
}

#[test]
fn entry_wins_over_config_path() {
    let dir = TempDir::new().unwrap();
    let config = write_config(&dir, valid_toml());
    let result = resolve_config(
        Some(entry(vec![dir.path().to_path_buf()])),
        Some(config),
        dir.path().to_path_buf(),
    )
    .unwrap();
    assert_eq!(result.entries[0].dirs, vec![dir.path().to_path_buf()]);
}

#[test]
fn from_config_file() {
    let dir = TempDir::new().unwrap();
    let config = write_config(&dir, valid_toml());
    let result =
        resolve_config(None, Some(config), dir.path().to_path_buf())
            .unwrap();
    assert_eq!(result.entries.len(), 1);
    assert_eq!(result.entries[0].command, "echo test");
    assert_eq!(result.debounce_ms, 500);
}

#[test]
fn error_propagates_from_read_config() {
    let tmp = TempDir::new().unwrap();
    let conf_file = tmp.path().join("conf.toml");
    let err =
        resolve_config(None, Some(conf_file), tmp.path().to_path_buf())
            .unwrap_err();
    assert!(matches!(err, ConfigError::Io(_)));
}

#[test]
fn no_source() {
    let tmp = TempDir::new().unwrap();
    let result = resolve_config(None, None, tmp.path().to_path_buf());
    assert!(matches!(result, Err(ConfigError::NoConfigSource)));
}
