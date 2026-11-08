use super::*;

#[test]
fn writes_default_template_to_config_file_in_path() {
    let dir = TempDir::new().unwrap();
    let result = run_init(dir.path());
    assert!(result.is_ok());

    let config_path = dir.path().join(CONFIG_FILE_NAME);
    assert!(config_path.is_file());
    let contents = fs::read_to_string(&config_path).unwrap();
    assert_eq!(contents, DEFAULT_CONFIG_TEMPLATE);
}

#[test]
fn refuses_to_overwrite_existing_config() {
    let dir = TempDir::new().unwrap();
    let config_path = dir.path().join(CONFIG_FILE_NAME);
    let sentinel = "sentinel data\n";
    fs::write(&config_path, sentinel).unwrap();

    let err = run_init(dir.path()).unwrap_err();
    assert!(
        matches!(err, InitError::FileAlreadyExists(p) if p == config_path),
    );
    assert_eq!(fs::read_to_string(&config_path).unwrap(), sentinel);
}

#[test]
fn propagates_creation_failure() {
    let dir = TempDir::new().unwrap();
    let missing_parent = dir.path().join("no/such/dir");

    assert!(run_init(&missing_parent).is_err());
}
