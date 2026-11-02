//! Implements `watch` command.
//!
//! Initializes filesystem watchers for all configured entries
//! and dispatches commands when matching file changes occur.
//!
//! See [`WatcherError`] for failure modes.
mod command;
mod debounce;
mod output;
pub mod shutdown;

use std::sync::mpsc::{Receiver, channel as mpsc_channel};

use notify_debouncer_full::notify;
use thiserror::Error;

use crate::config::WatcherConfig;

use command::{CommandRunner, ShellRunner};
use debounce::create_debouncers;
use output::format_output;
use shutdown::ShutdownHandler;

/// Errors produced during watcher initialization and runtime
/// setup.
#[derive(Error, Debug)]
pub enum WatcherError {
    /// Debouncer creation or watch registration failure from
    /// the notify layer.
    #[error("notify-debouncer error: {0}")]
    Notify(#[from] notify::Error),

    /// Failed to write to the output writer.
    #[error("failed to write to output: {0}")]
    WriteError(#[from] std::io::Error),

    /// Failure installing the Ctrl+C signal handler.
    #[error("failed to create shutdown handler: {0}")]
    SignalHandler(#[from] ctrlc::Error),

    /// Unexpected disconnection of the event channel.
    #[error("channel dropped unexpectedly")]
    ChannelDisconnected,
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
/// use watchr::watcher::shutdown::{CtrlCHandler, ShutdownHandler};
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
/// run_watch(config, &mut stdout, CtrlCHandler)?;
/// # Ok::<(), watchr::watcher::WatcherError>(())
/// ```
pub fn run_watch<S: ShutdownHandler>(
    config: WatcherConfig,
    writer: &mut dyn std::io::Write,
    shutdown_handler: S,
) -> Result<(), WatcherError> {
    let (tx, rx) = mpsc_channel();

    shutdown_handler.install(tx.clone())?;

    // NB: debouncers must stay in scope - dropping them cancels
    // pending events and stops the watchers. Do not "simplify" to
    // `let _ = `.
    let _watch_handles =
        create_debouncers(config.debounce_ms, config.entries, tx)?;

    let runner = ShellRunner;
    run_event_loop(rx, &runner, writer)?;
    Ok(())
}

fn run_event_loop<R: CommandRunner>(
    rx: Receiver<WatchEvent>,
    runner: &R,
    writer: &mut dyn std::io::Write,
) -> Result<(), WatcherError> {
    loop {
        match rx.recv() {
            Ok(WatchEvent::Command { cmd, name }) => {
                let output = runner.run(&cmd);

                let formatted =
                    format_output(&cmd, name.as_deref(), output);
                writeln!(writer, "{}", formatted)
                    .map_err(WatcherError::WriteError)?;
            }
            Ok(WatchEvent::Shutdown) => {
                writeln!(writer, "Shutting down gracefully...")
                    .map_err(WatcherError::WriteError)?;
                return Ok(());
            }
            Err(_) => {
                return Err(WatcherError::ChannelDisconnected);
            }
        }
    }
}

#[cfg(test)]
mod tests;
