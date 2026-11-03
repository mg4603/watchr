use super::*;
#[test]
fn parses_valid_config() {
    let dir = TempDir::new().unwrap();
    let path = write_config(&dir, &valid_toml());
    let config = read_config(&path).unwrap();

    assert_eq!(config.debounce_ms, 500);
    assert_eq!(config.entries.len(), 1);

    let entry = &config.entries[0];

    assert_eq!(entry.name, Some("sample_test".to_string()));
    assert_eq!(entry.dirs.len(), 1);
    assert_eq!(entry.command, "pwd");
    assert!(entry.ext.is_none());
}

// ----------- IO errors ---------
#[test]
fn missing_file_maps_to_io_error() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("does-not-exist.toml");
    let err = read_config(&file_path).unwrap_err();

    match err {
        ConfigError::Io(io) => {
            assert_eq!(io.kind(), std::io::ErrorKind::NotFound);
        }
        other => panic!("expected Io error, got {other:?}"),
    }
}

#[test]
fn directory_instead_of_file_maps_to_io_error() {
    let dir = TempDir::new().unwrap();
    let err = read_config(dir.path()).unwrap_err();
    assert!(matches!(err, ConfigError::Io(_)));
}

#[cfg(unix)]
#[test]
fn permission_denied_matches_to_io_error() {
    use std::os::unix::fs::PermissionsExt;
    let dir = TempDir::new().unwrap();
    let path = write_config(&dir, &valid_toml());
    fs::set_permissions(&path, fs::Permissions::from_mode(0o000))
        .unwrap();
    let err = read_config(&path).unwrap_err();
    match err {
        ConfigError::Io(io) => {
            assert_eq!(io.kind(), std::io::ErrorKind::PermissionDenied)
        }
        other => {
            panic!("expected Io(PermissionDenied), got {other:?}")
        }
    }
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644))
        .unwrap();
}

// ------------ deserialization errors -------------
#[test]
fn malformed_toml_maps_to_deserialize_error() {
    let dir = TempDir::new().unwrap();
    let path = write_config(&dir, "this is [[ not valid toml");
    let err = read_config(&path).unwrap_err();
    assert!(matches!(err, ConfigError::Deserialize(_)));
}

#[test]
fn wrong_type_maps_to_deserialize_error() {
    let dir = TempDir::new().unwrap();
    let path = write_config(&dir, &wrong_type_toml());
    let err = read_config(&path).unwrap_err();
    assert!(matches!(err, ConfigError::Deserialize(_)));
}
