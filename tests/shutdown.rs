use std::process::{Command, Stdio};
use std::time::Duration;
use tempfile::TempDir;

mod shared;

#[test]
fn test_graceful_shutdown_on_sigint() {
    let tmp_dir = TempDir::new().unwrap();

    let binary = env!("CARGO_BIN_EXE_watchr");
    let mut child = Command::new(binary)
        .args(["watch", tmp_dir.path().to_str().unwrap()])
        .args(["--cmd", "echo watchr_test_ran"])
        .current_dir(tmp_dir.path())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn watchr");

    let stdout = child.stdout.take().unwrap();
    let rx = shared::spawn_output_reader(stdout);

    std::thread::sleep(Duration::from_millis(500));

    Command::new("kill")
        .args(["-INT", &child.id().to_string()])
        .status()
        .unwrap();

    child.wait().expect("Failed to wait for watchr");

    let found = shared::wait_for_output(
        &rx,
        "Shutting down gracefully",
        Duration::from_secs(5),
    );

    assert!(found, "Shutdown message not found within timeout");
}
