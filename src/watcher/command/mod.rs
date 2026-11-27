/// Abstracts command execution for testability.
pub(crate) trait CommandRunner {
    fn run(&self, cmd: &str) -> std::io::Result<std::process::Output>;
}

pub(crate) struct ShellRunner;

impl CommandRunner for ShellRunner {
    fn run(&self, cmd: &str) -> std::io::Result<std::process::Output> {
        std::process::Command::new("sh").arg("-c").arg(cmd).output()
    }
}

#[cfg(test)]
pub(crate) mod tests;
