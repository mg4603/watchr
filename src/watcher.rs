//! File watcher orchestration.
//!
//! Provides the implementation for the `watch` command.
//!
//! This module initializes filesystem watchers for all
//! configured entries and dispatches commands when matching
//! file changes occur.
//!
//! See [`WatcherError`] for failure modes.
use std::path::PathBuf;
use std::process;
use std::sync::mpsc::{
    Receiver, Sender, channel as mpsc_channel,
};
use std::time::Duration;

use notify_debouncer_full::notify::{
    RecommendedWatcher, RecursiveMode,
};
use notify_debouncer_full::{
    DebounceEventResult, Debouncer, NoCache, new_debouncer,
    notify,
};
use thiserror::Error;

use crate::config::WatchrConfig;
use crate::entry::WatchrEntry;

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
    Command(String),

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
/// use watchr::entry::WatchrEntry;
///
/// let entry = WatchrEntry {
///     name: "test",
///     dirs: vec![PathBuf::from(".")],
///     ext: None,
///     command: "cargo test".to_string(),
/// }
/// let config = WatchrConfig{
///     debounce_ms: 500,
///     entries: vec![entry]
/// }
///
/// run_watch(config)?
/// ```
pub fn run_watch(
    config: WatchrConfig,
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

/// Installs a Ctrl+C handler that triggers a graceful shutdown.
///
/// When SIGINT or SIGTERM is received, [`WatchEvent::Shutdown`]
/// message is sent through the provided channel.
///
/// # Arguments
///
/// * `tx` - Channel sender used to propagate shutdown events
///
/// # Errors
/// Returns [`WatcherError::SignalHandler`] if the handler cannot
/// be registered.
///
/// # Examples
///
/// ```no_run
/// let (tx, _) = std::sync::mpsc::channel();
/// let ctrlc_handler = create_shutdown_handler(tx)?;
/// ```
fn create_shutdown_handler(
    tx: Sender<WatchEvent>,
) -> Result<(), WatcherError> {
    ctrlc::try_set_handler(move || {
        let _ = tx.send(WatchEvent::Shutdown);
    })?;
    Ok(())
}

/// Processes debounced filesystem events and emits commands when
/// matched.
///
/// If no extension filter is configured, any event triggers the
/// command. Otherwise, only file changes matching one of the
/// provided extensions will trigger execution.
///
/// # Arguments
/// * `result` - Debounced event result from the notify layer
/// * `exts` - Optional list of file extensions to filter on
/// * `command` - Command to execute when a match occurs
/// * `tx` - Channel sender used to emit [`WatchEvent`]s
///
///
/// # Examples
/// ```no_run
/// let (tx, _) = std::sync::mpsc::channel();
/// handle_events(Ok(vec![]), None, "cargo test".into(), tx);
/// ```
fn handle_events(
    result: DebounceEventResult,
    exts: Option<Vec<String>>,
    command: String,
    tx: Sender<WatchEvent>,
) {
    match result {
        Ok(events) => {
            if exts.as_ref().is_none() {
                let _ = tx
                    .send(WatchEvent::Command(command.clone()));
                return;
            }

            let paths: Vec<PathBuf> = events
                .into_iter()
                .flat_map(|event| event.paths.clone())
                .collect();

            for path in paths {
                if !path.is_file() {
                    continue;
                }

                if let (Some(ext), Some(exts)) = (
                    path.extension().and_then(|e| e.to_str()),
                    exts.as_deref(),
                ) && exts.iter().any(|e| e == ext)
                {
                    let _ = tx.send(WatchEvent::Command(
                        command.clone(),
                    ));
                    return;
                }
            }
        }
        Err(errors) => {
            for e in errors {
                println!("{:?}", e);
            }
        }
    }
}

/// Creates and registers filesystem watchers for each entry.
///
/// Each entry results in a dedicated debouncer configured with:
/// - The specified debounce duration
/// - A callback that filters events and emits commands
///
/// # Arguments
/// * `debounce_ms` - Debounce window in milliseconds
/// * `entries` - Watch configuration entries
/// * `tx` - Channel sender used to emit [`WatchEvent`]s
///
/// # Returns
///
/// A collection of active debouncers. They must be kept alive
/// for watcher to remain active.
///
/// # Errors
///
/// Returns [`WatcherError`] if:
/// - A debouncer cannot be created
/// - A directory cannot be registered for watching
///
/// # Examples
/// ```no_run
/// use watchr::entry::WatchrEntry;
/// use std::path::PathBuf;
///
/// let (tx, _) = std::sync::mpsc::channel();
/// let entry = WatchrEntry{
///     name: None,
///     dirs: [PathBuf::from(".")]
///     ext: None,
///     command: "cargo test".to_string(),
/// };
///
/// let _ = create_debouncers(500, vec![entry], tx)?;
/// ```
fn create_debouncers(
    debounce_ms: u64,
    entries: Vec<WatchrEntry>,
    tx: Sender<WatchEvent>,
) -> Result<
    Vec<Debouncer<RecommendedWatcher, NoCache>>,
    WatcherError,
> {
    let mut debouncers = Vec::new();
    for entry in entries {
        let tx = tx.clone();

        let mut debouncer = new_debouncer(
            Duration::from_millis(debounce_ms),
            None,
            move |result: DebounceEventResult| {
                handle_events(
                    result,
                    entry.ext.clone(),
                    entry.command.clone(),
                    tx.clone(),
                );
            },
        )?;

        for dir in &entry.dirs {
            debouncer.watch(dir, RecursiveMode::Recursive)?;
        }
        debouncers.push(debouncer);
    }
    Ok(debouncers)
}

/// Runs the main event loop, consumeing [`WatchEvent`]s.
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
            Ok(WatchEvent::Command(cmd)) => {
                let output = process::Command::new("sh")
                    .arg("-c")
                    .arg(&cmd)
                    .output();

                match output {
                    Ok(out) => println!("{:?}", out),
                    Err(e) => println!("{:?}", e),
                }
            }
            Ok(WatchEvent::Shutdown) => {
                println!("Shutting down gracefully...");
                break;
            }
            Err(_) => break,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::time::Instant;

    use notify_debouncer_full::DebouncedEvent;
    use notify_debouncer_full::notify::ErrorKind;
    use notify_debouncer_full::notify::event::{
        Event, EventKind, ModifyKind,
    };

    fn create_debounced_event_result(
        error: bool,
    ) -> DebounceEventResult {
        if error {
            return Err(vec![notify::Error {
                kind: ErrorKind::Generic("custom".to_string()),
                paths: vec![PathBuf::from("./")],
            }]);
        }

        Ok(vec![DebouncedEvent {
            event: Event {
                kind: EventKind::Modify(ModifyKind::Any),
                paths: vec![PathBuf::from("src/main.rs")],
                attrs: Default::default(),
            },
            time: Instant::now(),
        }])
    }

    #[test]
    fn test_handle_events_no_ext() {
        let result = create_debounced_event_result(false);
        let (tx, rx) = mpsc_channel();
        handle_events(result, None, "pwd".to_string(), tx);

        assert!(matches!(
            rx.try_recv(),
            Ok(WatchEvent::Command(_))
        ));
    }

    #[test]
    fn test_handle_event_matching_ext() {
        let result = create_debounced_event_result(false);
        let (tx, rx) = mpsc_channel();
        handle_events(
            result,
            Some(vec!["rs".to_string()]),
            "pwd".to_string(),
            tx,
        );

        assert!(matches!(
            rx.try_recv(),
            Ok(WatchEvent::Command(_))
        ));
    }

    #[test]
    fn test_handle_event_no_matching_ext() {
        let result = create_debounced_event_result(false);
        let (tx, rx) = mpsc_channel();
        handle_events(
            result,
            Some(vec!["txt".to_string()]),
            "pwd".to_string(),
            tx,
        );

        // mpsc::TryRecvError::Empty
        assert!(matches!(rx.try_recv(), Err(..)));
    }

    #[test]
    fn test_hand_event_error_result() {
        let result = create_debounced_event_result(true);
        let (tx, rx) = mpsc_channel();
        handle_events(result, None, "pwd".to_string(), tx);

        // mpsc::TryRecvError::Empty
        assert!(matches!(rx.try_recv(), Err(..)))
    }
}
