//! Creates one debounced watcher per configured entry and
//! forwards matching filesystem events as [`WatchEvent`]s.
use super::{WatchEvent, WatcherError};
use crate::entry::WatcherEntry;
use notify_debouncer_full::notify::event::EventKind;
use notify_debouncer_full::notify::{
    RecommendedWatcher, RecursiveMode,
};
use notify_debouncer_full::{
    DebounceEventResult, DebouncedEvent, Debouncer, NoCache,
    new_debouncer,
};
use std::sync::mpsc::Sender;
use std::time::Duration;

/// Creates and registers filesystem watchers for each entry.
///
/// Returns the active debouncers, which must be kept alive for
/// the watchers to remain active.
///
/// # Errors
///
/// Returns [`WatcherError`] if:
/// - A debouncer cannot be created
/// - A directory cannot be registered for watching
///
/// # Examples
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
    for WatcherEntry {
        name,
        dirs,
        ext,
        command,
    } in entries
    {
        let tx = tx.clone();
        let mut debouncer = new_debouncer(
            Duration::from_millis(debounce_ms),
            None,
            debounced_events_result_handler(
                name, ext, command, tx,
            ),
        )?;

        for dir in &dirs {
            debouncer.watch(dir, RecursiveMode::Recursive)?;
        }
        debouncers.push(debouncer);
    }
    Ok(debouncers)
}

// Returns a closure for [`new_debouncer`] that forwards
// debounced results to [`handle_events`].
fn debounced_events_result_handler(
    watcher_name: Option<String>,
    extensions_to_filter: Option<Vec<String>>,
    command_on_fs_mutation: String,
    tx: Sender<WatchEvent>,
) -> impl FnMut(DebounceEventResult) {
    move |result| {
        handle_events(
            result,
            watcher_name.clone(),
            extensions_to_filter.clone(),
            command_on_fs_mutation.clone(),
            tx.clone(),
        );
    }
}

/// Sends the command for the first event that matches the
/// watcher's extension filter.
///
/// Create, modify, and remove events are considered; if no
/// filter is configured, any such event triggers the command.
pub(super) fn handle_events(
    result: DebounceEventResult,
    watcher_name: Option<String>,
    extensions_to_filter: Option<Vec<String>>,
    command_on_fs_mutation: String,
    tx: Sender<WatchEvent>,
) {
    match result {
        Ok(events) => {
            tracing::debug!(
                count = events.len(),
                "received debounced events"
            );

            for event in &events {
                tracing::debug!(
                    paths = ?event.paths,
                    kind = ?event.event.kind, "event detail"
                );

                if !matches!(
                    event.event.kind,
                    EventKind::Modify(_)
                        | EventKind::Create(_)
                        | EventKind::Remove(_)
                ) {
                    continue;
                }

                if check_extensions(
                    event,
                    extensions_to_filter.as_ref(),
                ) {
                    tracing::debug!(
                        command = %command_on_fs_mutation,
                        "sending command event");

                    let _ = tx.send(WatchEvent::Command {
                        cmd: command_on_fs_mutation.clone(),
                        name: watcher_name.clone(),
                    });

                    return;
                }
            }
        }

        Err(errors) => {
            for e in errors {
                tracing::error!(
                error = %e,
                "failed to process file watch event"
                );
            }
        }
    }
}

// Returns `true` if no extension filter is configured, or one
// of the event's file paths matches a filtered extension.
fn check_extensions(
    event: &DebouncedEvent,
    extensions_to_filter: Option<&Vec<String>>,
) -> bool {
    event.paths.iter().any(|path| {
        if !path.is_file() {
            return false;
        }
        match extensions_to_filter {
            None => true,
            Some(extensions) => extensions.iter().any(|ext| {
                path.extension()
                    .and_then(|os_str| os_str.to_str())
                    .map(|s| s == ext)
                    .unwrap_or(false)
            }),
        }
    })
}

#[cfg(test)]
mod tests {

    use super::*;

    use notify_debouncer_full::DebouncedEvent;
    use notify_debouncer_full::notify;
    use notify_debouncer_full::notify::event::{
        AccessKind, CreateKind, Event, EventKind, ModifyKind,
        RemoveKind,
    };

    use std::path::PathBuf;
    use std::sync::mpsc::channel as mpsc_channel;
    use std::time::Instant;

    fn create_debounced_event_result(
        error: bool,
        kind: &str,
    ) -> DebounceEventResult {
        if error {
            return Err(vec![notify::Error {
                kind: notify::ErrorKind::Generic(
                    "custom".to_string(),
                ),
                paths: vec![PathBuf::from("./")],
            }]);
        }

        let event_kind = match kind {
            "access" => EventKind::Access(AccessKind::Any),
            "create" => EventKind::Create(CreateKind::Any),
            "remove" => EventKind::Remove(RemoveKind::Any),
            _ => EventKind::Modify(ModifyKind::Any), //default
        };

        Ok(vec![DebouncedEvent {
            event: Event {
                kind: event_kind,
                paths: vec![PathBuf::from("src/main.rs")],
                attrs: Default::default(),
            },
            time: Instant::now(),
        }])
    }

    #[test]
    fn test_handle_events_no_ext() {
        let result =
            create_debounced_event_result(false, "modify");
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
        let result =
            create_debounced_event_result(false, "modify");
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
        let result =
            create_debounced_event_result(false, "modify");
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
        let result =
            create_debounced_event_result(false, "modify");
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
        let result =
            create_debounced_event_result(true, "modify");
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

    #[test]
    fn test_handle_events_filters_access_event() {
        let result =
            create_debounced_event_result(false, "access");
        let (tx, rx) = mpsc_channel();
        handle_events(
            result,
            None,
            None,
            "pwd".to_string(),
            tx,
        );
        assert!(matches!(rx.try_recv(), Err(..)));
    }

    #[test]
    fn test_handle_events_allows_modify_event() {
        let result =
            create_debounced_event_result(false, "modify");
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
    fn test_handle_events_allows_create_event() {
        let result =
            create_debounced_event_result(false, "create");
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
    fn test_handle_events_allows_remove_event() {
        let result =
            create_debounced_event_result(false, "remove");
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

    fn test_check_extensions(
        extensions: Option<Vec<String>>,
        path: &str,
        expected: bool,
    ) -> bool {
        let event = DebouncedEvent {
            event: Event {
                kind: EventKind::Modify(ModifyKind::Any),
                paths: vec![PathBuf::from(path)],
                attrs: Default::default(),
            },
            time: Instant::now(),
        };
        check_extensions(&event, extensions.as_ref())
            == expected
    }

    #[test]
    fn test_check_extensions_with_none() {
        assert!(test_check_extensions(
            None,
            "src/main.rs",
            true
        ));
    }

    #[test]
    fn test_check_extensions_matching() {
        assert!(test_check_extensions(
            Some(vec!["rs".to_string()]),
            "src/main.rs",
            true
        ));
    }

    #[test]
    fn test_check_extensions_no_matching() {
        assert!(test_check_extensions(
            Some(vec!["rs".to_string()]),
            "src/main.py",
            false
        ));
    }

    #[test]
    fn test_check_extensions_directory_path() {
        assert!(test_check_extensions(
            Some(vec!["rs".to_string()]),
            "src",
            false
        ));
    }

    #[test]
    fn test_check_extensions_directory_path_no_none() {
        assert!(test_check_extensions(None, "src", false));
    }

    #[test]
    fn test_debounced_events_result_handler() {
        let (tx, rx) = mpsc_channel();

        let mut handler = debounced_events_result_handler(
            Some("test".to_string()),
            None,
            "echo test".to_string(),
            tx,
        );

        let result =
            create_debounced_event_result(false, "modify");
        handler(result);

        assert!(matches!(
            rx.try_recv(),
            Ok(WatchEvent::Command { .. })
        ));
    }
}
