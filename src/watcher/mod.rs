//! File watcher orchestration.
//!
//! Provides the implementation for the `watch` command.
//!
//! This module initializes filesystem watchers for all
//! configured entries and dispatches commands when matching
//! file changes occur.
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
use output::print_output;
use shutdown::create_shutdown_handler;

/// Errors produced during watcher initialization and runtime
/// setup.
#[derive(Error, Debug)]
pub enum WatcherError {
    /// Failure originating from the notify/debouncer layer.
    ///
    /// This includes:
    /// - Debouncer creation failures
    /// - Debouncer watch registration failures
    #[error("notify-debouncer error: {0}")]
    Notify(#[from] notify::Error),

    /// Failure when installing the Ctrl+C signal handler.
    #[error("failed to create shutdown handler: {0}")]
    SignalHandler(#[from] ctrlc::Error),
}

/// Events emitted by the watcher system and consumed by the
/// event loop.
#[derive(Debug)]
pub enum WatchEvent {
    /// Execute the associated command.
    ///
    /// `name` is the optional name of the watcher entry that
    /// triggered this event, used to identify which watcher's
    /// output is being shown.
    Command { cmd: String, name: Option<String> },

    /// Terminate the watcher loop gracefully.
    Shutdown,
}

/// Runs the file watching system.
///
/// Initializes:
/// - A shutdown signal handler (Ctrl+C)
/// - Debounced filesystem watchers for all configured entries
/// - The main event loop
///
/// This functions blocks until a shutdown event is received.
///
/// # Arguments
/// * `config` - Application configuration containing watcher
///   entries
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
/// use watchr::entry::WatcherEntry;
///
/// let entry = WatcherEntry {
///     name: "test",
///     dirs: vec![PathBuf::from(".")],
///     ext: None,
///     command: "cargo test".to_string(),
/// }
/// let config = WatcherConfig{
///     debounce_ms: 500,
///     entries: vec![entry]
/// }
///
/// run_watch(config)?
/// ```
pub fn run_watch(
    config: WatcherConfig,
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

    run_event_loop(rx);
    Ok(())
}

/// Runs the main event loop, consuming [`WatchEvent`]s.
///
/// Behavior:
/// - Executes shell commands for [`WatchEvent::Command`]
/// - Terminates cleanly on [`WatchEvent::Shutdown`]
/// - Exits if the channel is closed
///
/// Commands are executed via `sh -c`.
///
/// # Arguments
/// * `rx` - Channel receiver for incoming [`WatchEvent`]s
///
/// # Examples
/// ```no_run
/// let (_tx, rx) = std::sync::mpsc::channel();
/// run_event_loop(rx);
/// ```
fn run_event_loop(rx: Receiver<WatchEvent>) {
    loop {
        match rx.recv() {
            Ok(WatchEvent::Command { cmd, name }) => {
                let output = process::Command::new("sh")
                    .arg("-c")
                    .arg(&cmd)
                    .output();

                print_output(&cmd, name.as_deref(), output);
            }
            Ok(WatchEvent::Shutdown) => {
                println!("Shutting down gracefully...");
                break;
            }
            Err(_) => break,
        }
    }
}
