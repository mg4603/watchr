use std::fs;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use tempfile::TempDir;

mod shared;

#[test]
fn config_path_arg_is_used() {
    let tmp = TempDir::new().unwrap();
    let test_file = tmp.path().join("test.txt");
    fs::write(&test_file, "hello").unwrap();

    let config = format!(
        r#"
[[watcher]]
dirs = ["{}"]
command = "echo watchr_test_ran"
"#,
        tmp.path().display()
    );
    let config_file = tmp.path().join("config.toml");
    fs::write(&config_file, config).unwrap();

    let binary = env!("CARGO_BIN_EXE_watchr");
    let mut child = Command::new(binary)
        .arg("watch")
        .args(["--config", config_file.to_str().unwrap()])
        .current_dir(tmp.path())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn watchr");

    let stdout = child.stdout.take().unwrap();
    let rx = shared::spawn_output_reader(stdout);

    thread::sleep(Duration::from_millis(500));

    fs::write(&test_file, "world").unwrap();

    let found_command = shared::wait_for_output(
        &rx,
        "$ echo watchr_test_ran",
        Duration::from_secs(5),
    );
    let found_success =
        shared::wait_for_output(&rx, "✓ success", Duration::from_secs(5));
    let found_output = shared::wait_for_output(
        &rx,
        "watchr_test_ran",
        Duration::from_secs(5),
    );

    child.kill().expect("Failed to kill watchr");
    child.wait().expect("Failed to wait for watchr");

    assert!(
        found_command && found_success && found_output,
        "Expected output not found within timeout"
    );
}
