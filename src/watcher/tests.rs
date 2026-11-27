use super::*;

use crate::test_helpers::FailingWriter;

#[cfg(unix)]
mod run_event_loop {
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

        let result = run_event_loop(rx, &runner, &mut output);

        assert!(result.is_ok());

        let output = String::from_utf8(output).unwrap();
        assert_eq!(output, "Shutting down gracefully...\n");
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

        let err = run_event_loop(rx, &runner, &mut output).unwrap_err();

        assert!(matches!(err, WatcherError::ChannelDisconnected));
        assert!(output.is_empty());
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

        let result = run_event_loop(rx, &runner, &mut output);
        assert!(result.is_ok());

        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("$ echo hello"));
        assert!(output.contains("✓ success"));
        assert!(output.contains("Shutting down gracefully..."));
    }

    #[test]
    fn writeln_error_command_arm() {
        let (tx, rx) = mpsc_channel();
        let mut writer = FailingWriter { remaining: 4 };
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

        let err = run_event_loop(rx, &runner, &mut writer).unwrap_err();
        assert!(matches!(err, WatcherError::WriteError(_)));
    }

    #[test]
    fn writeln_error_shutdown_arm() {
        let (tx, rx) = mpsc_channel();
        let mut writer = FailingWriter { remaining: 4 };
        let runner = FakeRunner {
            status: 0,
            stdout: b"hello".to_vec(),
            stderr: Vec::new(),
        };

        tx.send(WatchEvent::Shutdown).unwrap();
        drop(tx);

        let err = run_event_loop(rx, &runner, &mut writer).unwrap_err();
        assert!(matches!(err, WatcherError::WriteError(_)));
    }
}
