use std::process::{Command, Stdio};
use tempfile::TempDir;

#[test]
fn test_init_creates_config_file() {
    let tmp_dir = TempDir::new().unwrap();

    let binary = env!("CARGO_BIN_EXE_watchr");
    let child = Command::new(binary)
        .arg("init")
        .current_dir(tmp_dir.path())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn watchr");

    let output = child
        .wait_with_output()
        .expect("Failed to wait for output");
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(tmp_dir.path().join(".watchr.toml").exists());
    assert!(stdout.contains(".watchr.toml created"));
}
