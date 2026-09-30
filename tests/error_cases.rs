use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_no_config_source_provided() {
    let tmp_dir = TempDir::new().unwrap();

    let binary = env!("CARGO_BIN_EXE_watchr");
    let output = Command::new(binary)
        .arg("watch")
        .current_dir(tmp_dir.path())
        .output()
        .expect("Failed to spawn watchr");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("no config source found"));
}

#[test]
fn test_dir_not_found() {
    let binary = env!("CARGO_BIN_EXE_watchr");
    let output = Command::new(binary)
        .args(["watch", "/nonexistent"])
        .args(["--cmd", "echo test"])
        .output()
        .expect("Failed to spawn watchr");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("directory not found"));
}
