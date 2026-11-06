
use super::*;

#[test]
fn happy_path() {
    let tmp_dir = TempDir::new().unwrap();
    let path = tmp_dir.path();

    let _ = run_init(path);

    let config_path = path.join(".watchr.toml");
    assert!(config_path.exists());
}

#[test]
#[cfg(unix)]
fn permission_denied() {
    use std::os::unix::fs::PermissionsExt;

    let tmp_dir = TempDir::new().unwrap();
    let path = tmp_dir.path();

    let perms = fs::Permissions::from_mode(0o555);
    fs::set_permissions(path, perms).unwrap();

    let res = run_init(path);

    // Restore permissions for cleanup
    let perms = fs::Permissions::from_mode(0o755);
    fs::set_permissions(path, perms).unwrap();

    assert!(matches!(res, Err(InitError::Io(_))));
}
