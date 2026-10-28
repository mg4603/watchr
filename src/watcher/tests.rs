use super::*;

use std::sync::mpsc::channel as mpsc_channel;

use crate::watcher::command::tests::FakeRunner;

#[test]
#[cfg(unix)]
fn test_run_event_loop_handles_shutdown() {
    let (tx, rx) = mpsc_channel();
    let mut output = Vec::new();
    let runner = FakeRunner {
        status: 0,
        stdout: b"test output".to_vec(),
        stderr: Vec::new(),
    };

    tx.send(WatchEvent::Shutdown).unwrap();
    drop(tx);

    let _ = run_event_loop(rx, &runner, &mut output);

    let result = String::from_utf8(output).unwrap();
    assert!(result.contains("Shutting down gracefully..."));
}

#[test]
#[cfg(unix)]
fn test_run_event_loop_handles_channel_close() {
    let (tx, rx) = mpsc_channel();
    let mut output = Vec::new();
    let runner = FakeRunner {
        status: 0,
        stdout: b"test output".to_vec(),
        stderr: Vec::new(),
    };
    drop(tx);

    let _ = run_event_loop(rx, &runner, &mut output);

    assert!(String::from_utf8(output).unwrap().is_empty());
}

#[test]
#[cfg(unix)]
fn test_run_event_loop_handles_command() {
    let (tx, rx) = mpsc_channel();
    let mut output = Vec::new();
    let runner = FakeRunner {
        status: 0,
        stdout: b"hello".to_vec(),
        stderr: Vec::new(),
    };

    tx.send(WatchEvent::Command {
        cmd: "echo hello".to_string(),
        name: None,
    })
    .unwrap();
    tx.send(WatchEvent::Shutdown).unwrap();
    drop(tx);

    let _ = run_event_loop(rx, &runner, &mut output);

    let result = String::from_utf8(output).unwrap();
    assert!(result.contains("$ echo hello"));
    assert!(result.contains("✓ success"));
}
