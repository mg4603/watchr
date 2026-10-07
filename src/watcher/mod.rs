//! Implements `watch` command.
//!
//! Initializes filesystem watchers for all configured entries
//! and dispatches commands when matching file changes occur.
//!
//! See [`WatcherError`] for failure modes.
mod debounce;
mod output;
mod shutdown;

use std::process;
use std::sync::mpsc::{Receiver, channel as mpsc_channel};

use notify_debouncer_full::notify;
use thiserror::Error;

use crate::config::WatcherConfig;

use debounce::create_debouncers;
use output::format_output;
use shutdown::create_shutdown_handler;

/// Errors produced during watcher initialization and runtime
/// setup.
#[derive(Error, Debug)]
pub enum WatcherError {
    /// Debouncer creation or watch registration failure from
    /// the notify layer.
    #[error("notify-debouncer error: {0}")]
    Notify(#[from] notify::Error),

    /// Failure installing the Ctrl+C signal handler.
    #[error("failed to create shutdown handler: {0}")]
    SignalHandler(#[from] ctrlc::Error),
}

/// Events consumed by the event loop.
#[derive(Debug)]
pub enum WatchEvent {
    /// Execute the associated command; `name` optionally
    /// identifies the watcher entry that triggered this event
    /// and is used to label its output.
    Command { cmd: String, name: Option<String> },

    /// Terminate the watcher loop gracefully.
    Shutdown,
}

/// Runs the file watching system and blocks until shutdown.
///
/// Installs a Ctrl+C handler and debounced filesystem watchers
/// for all configured entries, then consumes events in a loop:
/// runs each entry's command via `sh -c` and writes its
/// formatted output to `writer`, until Ctrl+C triggered
/// shutdown (or channel close).
///
/// # Errors
///
/// Returns [`WatcherError`] if:
/// - The debouncer cannot be created
/// - A directory cannot be registered for watching
/// - The signal handler cannot be installed
///
/// # Examples
///
/// ```no_run
/// use watchr::config::WatcherConfig;
/// use watchr::entry::WatcherEntry;
/// use watchr::watcher::run_watch;
/// use std::path::PathBuf;
///
/// let entry = WatcherEntry {
///     name: Some("test".to_string()),
///     dirs: vec![PathBuf::from(".")],
///     ext: None,
///     command: "cargo test".to_string(),
/// };
/// let config = WatcherConfig{
///     debounce_ms: 500,
///     entries: vec![entry]
/// };
/// let mut stdout = std::io::stdout();
/// run_watch(config, &mut stdout)?;
/// # Ok::<(), watchr::watcher::WatcherError>(())
/// ```
pub fn run_watch(
    config: WatcherConfig,
    writer: &mut dyn std::io::Write,
) -> Result<(), WatcherError> {
    let (tx, rx) = mpsc_channel();

    create_shutdown_handler(tx.clone())?;

    let _debouncers = create_debouncers(
        config.debounce_ms,
        config.entries,
        tx.clone(),
    )?;

    // drop initial sender after creating clones
    drop(tx);
    run_event_loop(rx, writer);
    Ok(())
}

fn run_event_loop(
    rx: Receiver<WatchEvent>,
    writer: &mut dyn std::io::Write,
) {
    loop {
        match rx.recv() {
            Ok(WatchEvent::Command { cmd, name }) => {
                let output = process::Command::new("sh")
                    .arg("-c")
                    .arg(&cmd)
                    .output();

                let formatted = format_output(
                    &cmd,
                    name.as_deref(),
                    output,
                );
                let _ = writeln!(writer, "{}", formatted);
            }
            Ok(WatchEvent::Shutdown) => {
                writeln!(writer, "Shutting down gracefully...")
                    .ok();
                break;
            }
            Err(_) => break,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use std::sync::mpsc::channel as mpsc_channel;

    #[test]
    fn test_run_event_loop_handles_command() {
        let (tx, rx) = mpsc_channel();
        let mut output = Vec::new();

        tx.send(WatchEvent::Command {
            cmd: "echo hello".to_string(),
            name: None,
        })
        .unwrap();
        tx.send(WatchEvent::Shutdown).unwrap();
        drop(tx);

        run_event_loop(rx, &mut output);

        let result = String::from_utf8(output).unwrap();
        assert!(result.contains("$ echo hello"));
        assert!(result.contains("✓ success"));
    }

    #[test]
    fn test_run_event_loop_handles_shutdown() {
        let (tx, rx) = mpsc_channel();
        let mut output = Vec::new();

        tx.send(WatchEvent::Shutdown).unwrap();
        drop(tx);

        run_event_loop(rx, &mut output);

        let result = String::from_utf8(output).unwrap();
        assert!(result.contains("Shutting down gracefully..."));
    }

    #[test]
    fn test_run_event_loop_handles_channel_close() {
        let (tx, rx) = mpsc_channel();
        let mut output = Vec::new();

        drop(tx);

        run_event_loop(rx, &mut output);

        assert!(String::from_utf8(output).unwrap().is_empty());
    }
}
