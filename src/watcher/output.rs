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

/// Formats command execution output as a multi-line string.
///
/// Returns a formatted string containing the watcher name
/// (if provided), command, status, and output/error messages
///
/// # Arguments
/// * `cmd` - The command that was executed
/// * `name` - Optional name of the watcher entry that triggered
///   this command
/// * `output` - Result of running the command
///
/// # Returns
/// A formatted string that can be directly written to stdout or
/// any other output destination
pub(super) fn format_output(
    cmd: &str,
    name: Option<&str>,
    output: Result<process::Output, io::Error>,
) -> String {
    let mut result = String::new();
    push_header(&mut result, cmd, name);

    match output {
        Ok(out) if out.status.success() => {
            push_success(&mut result, &out)
        }
        Ok(out) => push_failure(&mut result, &out),
        Err(e) => push_spawn_error(&mut result, e),
    }
    result
}

/// Adds header to mutable string.
/// (For internal reference only)
///
/// # Arguments
/// * `result` - Mutable string to append header to
/// * `cmd` - Command to be triggered by watcher entry
/// * `name` - Name of entry
fn push_header(
    result: &mut String,
    cmd: &str,
    name: Option<&str>,
) {
    if let Some(name) = name {
        result.push_str(&format!("[{}]\n", name));
    }
    result.push_str(&format!("$ {}\n", cmd));
}

/// Adds success message to mutable string
/// (For internal reference only)
///
/// # Arguments
/// * `result` - Mutable string to append success message to
/// * `out` - Output of the executed command
fn push_success(result: &mut String, out: &process::Output) {
    result.push_str("✓ success\n");
    push_trimmed(result, &out.stdout);
}

/// Adds failure message to mutable string
/// (For internal reference only)
///
/// # Arguments
/// * `result` - Mutable string to append failure message to
/// * `out` - Output of the executed command
fn push_failure(result: &mut String, out: &process::Output) {
    match out.status.code() {
        Some(code) => result.push_str(&format!(
            "✗ failed (exit code {})\n",
            code
        )),
        None => result.push_str("✗ failed (terminated)\n"),
    }

    push_trimmed(result, &out.stderr);
}

/// Adds spawn error to mutable string
/// (For internal reference only)
///
/// # Arguments
/// * `result` - Mutable string to append spawn error to
/// * `e` - IO error that occurred while spawning child process
fn push_spawn_error(result: &mut String, e: io::Error) {
    result.push_str(&format!("✗ failed to spawn: {}\n", e));
}

/// Appends trimmed UTF-8 output to mutable string.
/// (For internal reference only)
///
/// # Arguments
/// * `result` - Mutable string to append trimmed output to
/// * `bytes` - Raw bytes to trim and append
fn push_trimmed(result: &mut String, bytes: &[u8]) {
    match String::from_utf8_lossy(bytes).trim() {
        "" => result.push_str("(no output)"),
        s => result.push_str(s),
    }
    result.push('\n');
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
