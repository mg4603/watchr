use super::*;

use std::io;
use std::os::unix::process::ExitStatusExt;
use std::process;

#[test]
fn test_format_output_success_output_name() {
    let output = Ok(process::Output {
        status: process::ExitStatus::from_raw(0),
        stdout: b"test output".to_vec(),
        stderr: Vec::new(),
    });

    let formatted = format_output("cargo test", Some("tests"), output);

    assert!(formatted.contains("[tests]"));
    assert!(formatted.contains("$ cargo test"));
    assert!(formatted.contains("✓ success"));
    assert!(formatted.contains("test output"));
}

#[test]
fn test_format_output_success_output_no_name() {
    let output = Ok(process::Output {
        status: process::ExitStatus::from_raw(0),
        stdout: b"test output".to_vec(),
        stderr: Vec::new(),
    });

    let formatted = format_output("cargo test", None, output);
    assert!(formatted.contains("$ cargo test"));
    assert!(formatted.contains("✓ success"));
    assert!(formatted.contains("test output"));
}

#[test]
fn test_format_output_success_no_output() {
    let output = Ok(process::Output {
        status: process::ExitStatus::from_raw(0),
        stdout: Vec::new(),
        stderr: Vec::new(),
    });

    let formatted = format_output("cargo test", None, output);
    assert!(formatted.contains("$ cargo test"));
    assert!(formatted.contains("✓ success"));
    assert!(formatted.contains("(no output)"));
}

#[test]
fn test_format_output_failure_exit_code_stderr() {
    let output = Ok(process::Output {
        status: process::ExitStatus::from_raw(1 << 8),
        stdout: Vec::new(),
        stderr: b"simulated error".to_vec(),
    });

    let formatted = format_output("cargo test", None, output);

    assert!(formatted.contains("$ cargo test"));
    assert!(formatted.contains("✗ failed (exit code 1)"));
    assert!(formatted.contains("simulated error"));
}

#[test]
fn test_format_output_failure_terminated() {
    let output = Ok(process::Output {
        status: process::ExitStatus::from_raw(1),
        stdout: Vec::new(),
        stderr: Vec::new(),
    });

    let formatted = format_output("cargo test", None, output);

    assert!(formatted.contains("$ cargo test"));
    assert!(formatted.contains("✗ failed (terminated)"));
    assert!(formatted.contains("(no output)"));
}

#[test]
fn test_format_output_failure_no_spawn() {
    let output =
        Err(io::Error::new(io::ErrorKind::NotFound, "command not found"));

    let formatted = format_output("nonexistent", None, output);

    println!("{}", formatted);
    assert!(formatted.contains("$ nonexistent"));
    assert!(formatted.contains("✗ failed to spawn: command not found"));
}
