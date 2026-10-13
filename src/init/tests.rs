use super::*;

use std::fs;

fn create_config_file(path: &Path) {
    let config = format!(
        r##"debounce_ms = 500

[[watcher]]
name = "sample_test"
dirs = ["{}"]
command = "pwd"
"##,
        path.parent().unwrap().display()
    );

    fs::write(path, config).unwrap()
}

#[test]
fn test_happy_path() {
    let tmp_dir = tempfile::TempDir::new().unwrap();
    let path = tmp_dir.path();

    let _ = run_init(path);

    let config_path = path.join(".watchr.toml");
    assert!(config_path.exists());
}

#[test]
fn test_already_exists() {
    let tmp_dir = tempfile::TempDir::new().unwrap();
    let path = tmp_dir.path();
    let config_path = path.join(".watchr.toml");

    create_config_file(&config_path);

    let res = run_init(path);

    assert!(matches!(res, Err(InitError::FileAlreadyExists)));
}

#[test]
#[cfg(unix)]
fn test_permission_denied() {
    use std::os::unix::fs::PermissionsExt;

    let tmp_dir = tempfile::TempDir::new().unwrap();
    let path = tmp_dir.path();

    let perms = fs::Permissions::from_mode(0o555);
    fs::set_permissions(path, perms).unwrap();

    let res = run_init(path);

    // Restore permissions for cleanup
    let perms = fs::Permissions::from_mode(0o755);
    fs::set_permissions(path, perms).unwrap();

    assert!(matches!(res, Err(InitError::Io(_))));
}
