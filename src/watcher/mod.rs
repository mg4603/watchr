//! File watcher orchestration.
//!
//! Provides the implementation for the `watch` command.
//!
//! This module initializes filesystem watchers for all
//! configured entries and dispatches commands when matching
//! file changes occur.
//!
//! See [`WatcherError`] for failure modes.
mod shutdown;

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

use crate::config::WatcherConfig;
use crate::entry::WatcherEntry;
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

/// Prints the output of a command execution, including status
/// and output/error messages.
///
/// # Arguments
/// * `cmd` - The command that was executed
/// * `name` - Optional name of the watcher entry that triggered
///   this command
/// * `output` - Result of running the command
fn print_output(
    cmd: &str,
    name: Option<&str>,
    output: Result<process::Output, std::io::Error>,
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

/// Processes debounced filesystem events and emits commands when
/// matched.
///
/// If no extension filter is configured, any event triggers the
/// command. Otherwise, only file changes matching one of the
/// provided extensions will trigger execution.
///
/// # Arguments
/// * `result` - Debounced event result from the notify layer
/// * `name` - Optional name of watcher entry
/// * `exts` - Optional list of file extensions to filter on
/// * `command` - Command to execute when a match occurs
/// * `tx` - Channel sender used to emit [`WatchEvent`]s
///
///
/// # Examples
/// ```no_run
/// let (tx, _) = std::sync::mpsc::channel();
/// handle_events(Ok(vec![]), None, None, "cargo test".into(), tx);
/// ```
fn handle_events(
    result: DebounceEventResult,
    name: Option<String>,
    exts: Option<Vec<String>>,
    command: String,
    tx: Sender<WatchEvent>,
) {
    match result {
        Ok(events) => {
            if exts.as_ref().is_none() {
                let _ = tx.send(WatchEvent::Command {
                    cmd: command.clone(),
                    name: name.clone(),
                });
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
                    let _ = tx.send(WatchEvent::Command {
                        cmd: command.clone(),
                        name: name.clone(),
                    });
                    return;
                }
            }
        }
        Err(errors) => {
            for e in errors {
                tracing::error!(error = %e, "failed to process file watch event");
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
/// use watchr::entry::WatcherEntry;
/// use std::path::PathBuf;
///
/// let (tx, _) = std::sync::mpsc::channel();
/// let entry = WatcherEntry{
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
    entries: Vec<WatcherEntry>,
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
                    entry.name.clone(),
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
        handle_events(
            result,
            None,
            None,
            "pwd".to_string(),
            tx,
        );

        assert!(matches!(
            rx.try_recv(),
            Ok(WatchEvent::Command { .. })
        ));
    }

    #[test]
    fn test_handle_events_name_in_emitted_watch_event() {
        let result = create_debounced_event_result(false);
        let (tx, rx) = mpsc_channel();
        handle_events(
            result,
            Some("test".to_string()),
            None,
            "pwd".to_string(),
            tx,
        );
        assert!(matches!(
                rx.try_recv(),
                Ok(WatchEvent::Command { name: Some(ref n), ..}) if n == "test"
        ));
    }

    #[test]
    fn test_handle_event_matching_ext() {
        let result = create_debounced_event_result(false);
        let (tx, rx) = mpsc_channel();
        handle_events(
            result,
            None,
            Some(vec!["rs".to_string()]),
            "pwd".to_string(),
            tx,
        );

        assert!(matches!(
            rx.try_recv(),
            Ok(WatchEvent::Command { .. })
        ));
    }

    #[test]
    fn test_handle_event_no_matching_ext() {
        let result = create_debounced_event_result(false);
        let (tx, rx) = mpsc_channel();
        handle_events(
            result,
            None,
            Some(vec!["txt".to_string()]),
            "pwd".to_string(),
            tx,
        );

        // mpsc::TryRecvError::Empty
        assert!(matches!(rx.try_recv(), Err(..)));
    }

    #[test]
    fn test_handle_event_error_result() {
        let result = create_debounced_event_result(true);
        let (tx, rx) = mpsc_channel();
        handle_events(
            result,
            None,
            None,
            "pwd".to_string(),
            tx,
        );

        // mpsc::TryRecvError::Empty
        assert!(matches!(rx.try_recv(), Err(..)))
    }
}
