use super::*;

use crate::test_helpers::FailingWriter;

#[cfg(unix)]
mod run_event_loop_tests {
    use super::*;

    use crate::watcher::command::tests::FakeRunner;
    use std::sync::mpsc::channel as mpsc_channel;

    #[test]
    fn shutdown_handled() {
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
    fn channel_close_handled() {
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
    fn command_executes_output_written() {
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

    #[test]
    fn writeln_error_command_arm() {
        let (tx, rx) = mpsc_channel();
        let mut writer = FailingWriter;
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

        let result = run_event_loop(rx, &runner, &mut writer);
        assert!(matches!(result, Err(WatcherError::WriteError(_))));
    }

    #[test]
    fn writeln_error_shutdown_arm() {
        let (tx, rx) = mpsc_channel();
        let mut writer = FailingWriter;
        let runner = FakeRunner {
            status: 0,
            stdout: b"hello".to_vec(),
            stderr: Vec::new(),
        };

        tx.send(WatchEvent::Shutdown).unwrap();
        drop(tx);

        let result = run_event_loop(rx, &runner, &mut writer);
        assert!(matches!(result, Err(WatcherError::WriteError(_))));
    }
}
