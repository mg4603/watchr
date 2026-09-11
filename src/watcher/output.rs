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

    if let Some(name) = name {
        result.push_str(&format!("[{}]\n", name));
    }
    result.push_str(&format!("$ {}\n", cmd));

    match output {
        Ok(out) if out.status.success() => {
            result.push_str("✓ success\n");
            match String::from_utf8_lossy(&out.stdout).trim() {
                "" => result.push_str("(no output)\n"),
                out => result.push_str(&format!("{}\n", out)),
            }
        }
        Ok(out) => {
            match out.status.code() {
                Some(code) => result.push_str(&format!(
                    "✗ failed (exit code {})\n",
                    code
                )),
                None => {
                    result.push_str("✗ failed (terminated)\n")
                }
            }

            match String::from_utf8_lossy(&out.stderr).trim() {
                "" => result.push_str("(no output)\n"),
                err => result.push_str(&format!("{}\n", err)),
            }
        }
        Err(e) => {
            result.push_str(&format!(
                "✗ failed to spawn: {}\n",
                e
            ));
        }
    }
    result
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
