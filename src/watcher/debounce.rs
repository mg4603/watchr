use super::{WatchEvent, WatcherError};
use crate::entry::WatcherEntry;
use notify_debouncer_full::notify::{
    RecommendedWatcher, RecursiveMode,
};
use notify_debouncer_full::{
    DebounceEventResult, Debouncer, NoCache, new_debouncer,
};
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::time::Duration;

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
///
/// For internal reference only - `create_debouncers` is not
/// part of the public API and this example cannot be compiled
/// or run externally.
///
/// ```ignore
/// use watchr::entry::WatcherEntry;
/// use std::path::PathBuf;
///
/// let (tx, _) = std::sync::mpsc::channel();
/// let entry = WatcherEntry{
///     name: None,
///     dirs: vec![PathBuf::from(".")],
///     ext: None,
///     command: "cargo test".to_string(),
/// };
///
/// let _ = create_debouncers(500, vec![entry], tx)?;
/// ```
pub(super) fn create_debouncers(
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
pub(super) fn handle_events(
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

#[cfg(test)]
mod tests {

    use super::*;

    use notify_debouncer_full::DebouncedEvent;
    use notify_debouncer_full::notify;
    use notify_debouncer_full::notify::event::{
        Event, EventKind, ModifyKind,
    };

    use std::sync::mpsc::channel as mpsc_channel;
    use std::time::Instant;

    fn create_debounced_event_result(
        error: bool,
    ) -> DebounceEventResult {
        if error {
            return Err(vec![notify::Error {
                kind: notify::ErrorKind::Generic(
                    "custom".to_string(),
                ),
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
