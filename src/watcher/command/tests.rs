use super::*;
use std::os::unix::process::ExitStatusExt;

pub(crate) struct FakeRunner {
    pub(crate) status: i32,
    pub(crate) stdout: Vec<u8>,
    pub(crate) stderr: Vec<u8>,
}

impl CommandRunner for FakeRunner {
    fn run(&self, _cmd: &str) -> std::io::Result<std::process::Output> {
        Ok(std::process::Output {
            status: std::process::ExitStatus::from_raw(self.status),
            stdout: self.stdout.clone(),
            stderr: self.stderr.clone(),
        })
    }
}

#[test]
fn test_shell_runner_run() {
    let runner = ShellRunner;
    let result = runner.run("echo hello").unwrap();

    assert!(result.status.success());
    assert_eq!(
        String::from_utf8_lossy(&result.stdout).trim(),
        "hello".to_string()
    );
}
