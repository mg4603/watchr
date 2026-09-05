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
