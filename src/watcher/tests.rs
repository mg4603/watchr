use super::*;

use std::sync::mpsc::channel as mpsc_channel;

#[test]
fn test_run_event_loop_handles_command() {
    let (tx, rx) = mpsc_channel();
    let mut output = Vec::new();

    tx.send(WatchEvent::Command {
        cmd: "echo hello".to_string(),
        name: None,
    })
    .unwrap();
    tx.send(WatchEvent::Shutdown).unwrap();
    drop(tx);

    run_event_loop(rx, &mut output);

    let result = String::from_utf8(output).unwrap();
    assert!(result.contains("$ echo hello"));
    assert!(result.contains("✓ success"));
}

#[test]
fn test_run_event_loop_handles_shutdown() {
    let (tx, rx) = mpsc_channel();
    let mut output = Vec::new();

    tx.send(WatchEvent::Shutdown).unwrap();
    drop(tx);

    run_event_loop(rx, &mut output);

    let result = String::from_utf8(output).unwrap();
    assert!(result.contains("Shutting down gracefully..."));
}

#[test]
fn test_run_event_loop_handles_channel_close() {
    let (tx, rx) = mpsc_channel();
    let mut output = Vec::new();

    drop(tx);

    run_event_loop(rx, &mut output);

    assert!(String::from_utf8(output).unwrap().is_empty());
}
