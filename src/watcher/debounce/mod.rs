//! Creates one debounced watcher per configured entry and
//! forwards matching filesystem events as [`WatchEvent`]s.
use super::{WatchEvent, WatcherError};
use crate::entry::WatcherEntry;
use notify_debouncer_full::notify::event::EventKind;
use notify_debouncer_full::notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_full::{
    DebounceEventResult, DebouncedEvent, Debouncer, NoCache,
    new_debouncer,
};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::time::Duration;

type WatchEventDebouncer = Debouncer<RecommendedWatcher, NoCache>;
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
) -> Result<Vec<WatchEventDebouncer>, WatcherError> {
    entries
        .into_iter()
        .map(|entry| {
            create_debouncer_for_entry(debounce_ms, entry, tx.clone())
        })
        .collect()
}

fn create_debouncer_for_entry(
    debounce_ms: u64,
    entry: WatcherEntry,
    tx: Sender<WatchEvent>,
) -> Result<WatchEventDebouncer, WatcherError> {
    let WatcherEntry { name, dirs, ext, command } = entry;

    let mut debouncer = new_debouncer(
        Duration::from_millis(debounce_ms),
        None,
        debounced_events_result_handler(name, ext, command, tx),
    )?;

    watch_dirs(&mut debouncer, &dirs)?;
    Ok(debouncer)
}

fn watch_dirs(
    debouncer: &mut WatchEventDebouncer,
    dirs: &[PathBuf],
) -> Result<(), WatcherError> {
    for dir in dirs {
        debouncer.watch(dir, RecursiveMode::Recursive)?;
    }
    Ok(())
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

            if has_matching_event(
                &events,
                extensions_to_filter.as_deref(),
            ) {
                send_command(tx, command_on_fs_mutation, watcher_name);
            }
        }

        Err(errors) => log_errors(errors),
    }
}

fn is_relevant_path(path: &Path, exts: Option<&[String]>) -> bool {
    path.is_file() && check_extensions(path, exts)
}

fn has_matching_event(
    events: &[DebouncedEvent],
    exts: Option<&[String]>,
) -> bool {
    events.iter().inspect(|event| log_event(event)).any(|event| {
        is_mutation(&event.event.kind)
            && event.paths.iter().any(|path| is_relevant_path(path, exts))
    })
}

fn send_command(
    tx: Sender<WatchEvent>,
    command: String,
    name: Option<String>,
) {
    tracing::debug!(command = %command, "sending command event");
    let _ = tx.send(WatchEvent::Command { cmd: command, name });
}

fn log_errors<E: std::fmt::Display>(errors: impl IntoIterator<Item = E>) {
    for e in errors {
        tracing::error!(error = %e, "failed to process file watch event");
    }
}

fn log_event(event: &DebouncedEvent) {
    tracing::debug!(paths = ?event.paths, kind = ?event.event.kind, "event detail");
}

fn is_mutation(kind: &EventKind) -> bool {
    matches!(
        kind,
        EventKind::Modify(_)
            | EventKind::Create(_)
            | EventKind::Remove(_)
    )
}

// Returns `true` if no extension filter is configured, or one
// of the event's file paths matches a filtered extension.
fn check_extensions(
    path: &Path,
    extensions_to_filter: Option<&[String]>,
) -> bool {
    match extensions_to_filter {
        None => true,
        Some(extensions) => extensions.iter().any(|ext| {
            path.extension()
                .and_then(|os_str| os_str.to_str())
                .map(|s| s == ext)
                .unwrap_or(false)
        }),
    }
}

#[cfg(test)]
mod tests;
