//! Output formatting for executed watcher commands.
//!
//! Results are rendered as:
//!
//! ```text
//! [name]        ← only when a watcher name is provided
//! $ command
//! ✓ success     or: ✗ failed (exit code N) / ✗ failed (terminated)
//! <trimmed stdout or stderr>
//! ```

use std::io;
use std::process;

/// Formats command execution output as a multi-line string:
/// watcher name (if any), command, status, and trimmed
/// output/error messages.
///
/// Returns a ready-to-print string.
pub(super) fn format_output(
    executed_command: &str,
    watcher_name: Option<&str>,
    command_result: Result<process::Output, io::Error>,
) -> String {
    let mut buffer = String::new();
    push_header(&mut buffer, executed_command, watcher_name);

    match command_result {
        Ok(command_output)
            if command_output.status.success() =>
        {
            push_success(&mut buffer, &command_output)
        }
        Ok(command_output) => {
            push_failure(&mut buffer, &command_output)
        }
        Err(e) => push_spawn_error(&mut buffer, e),
    }
    buffer
}

// Adds header to buffer.
fn push_header(
    buffer: &mut String,
    executed_command: &str,
    watcher_name: Option<&str>,
) {
    if let Some(name) = watcher_name {
        buffer.push_str(&format!("[{}]\n", name));
    }
    buffer.push_str(&format!("$ {}\n", executed_command));
}

// Adds success message to buffer.
fn push_success(
    buffer: &mut String,
    command_output: &process::Output,
) {
    buffer.push_str("✓ success\n");
    push_trimmed(buffer, &command_output.stdout);
}

// Adds failure message to buffer.
fn push_failure(
    buffer: &mut String,
    command_output: &process::Output,
) {
    match command_output.status.code() {
        Some(code) => buffer.push_str(&format!(
            "✗ failed (exit code {})\n",
            code
        )),
        None => buffer.push_str("✗ failed (terminated)\n"),
    }

    push_trimmed(buffer, &command_output.stderr);
}

// Adds spawn error to buffer.
fn push_spawn_error(buffer: &mut String, e: io::Error) {
    buffer.push_str(&format!("✗ failed to spawn: {}\n", e));
}

// Appends trimmed UTF-8 output to buffer.
fn push_trimmed(buffer: &mut String, bytes: &[u8]) {
    match String::from_utf8_lossy(bytes).trim() {
        "" => buffer.push_str("(no output)"),
        s => buffer.push_str(s),
    }
    buffer.push('\n');
}

#[cfg(test)]
mod tests {
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

        let formatted =
            format_output("cargo test", Some("tests"), output);

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

        let formatted =
            format_output("cargo test", None, output);
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

        let formatted =
            format_output("cargo test", None, output);
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

        let formatted =
            format_output("cargo test", None, output);

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

        let formatted =
            format_output("cargo test", None, output);

        assert!(formatted.contains("$ cargo test"));
        assert!(formatted.contains("✗ failed (terminated)"));
        assert!(formatted.contains("(no output)"));
    }

    #[test]
    fn test_format_output_failure_no_spawn() {
        let output = Err(io::Error::new(
            io::ErrorKind::NotFound,
            "command not found",
        ));

        let formatted =
            format_output("nonexistent", None, output);

        println!("{}", formatted);
        assert!(formatted.contains("$ nonexistent"));
        assert!(
            formatted.contains(
                "✗ failed to spawn: command not found"
            )
        );
    }
}
