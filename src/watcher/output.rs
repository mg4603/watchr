use std::io;
use std::process;
/// Prints the output of a command execution, including status
/// and output/error messages.
///
/// # Arguments
/// * `cmd` - The command that was executed
/// * `name` - Optional name of the watcher entry that triggered
///   this command
/// * `output` - Result of running the command
pub(super) fn print_output(
    cmd: &str,
    name: Option<&str>,
    output: Result<process::Output, io::Error>,
) {
    if let Some(name) = name {
        println!("[{}]", name);
    }
    println!("$ {}", cmd);

    match output {
        Ok(out) if out.status.success() => {
            println!("✓ success");
            match String::from_utf8_lossy(&out.stdout).trim() {
                "" => println!("(no output)"),
                out => println!("{}", out),
            }
        }
        Ok(out) => {
            match out.status.code() {
                Some(code) => {
                    println!("✗ failed (exit code {})", code)
                }
                None => println!("✗ failed (terminated)"),
            }

            match String::from_utf8_lossy(&out.stderr).trim() {
                "" => eprintln!("(no output)"),
                err => eprintln!("{}", err),
            }
        }
        Err(e) => {
            println!("✗ failed to spawn: {}", e);
        }
    }
}
