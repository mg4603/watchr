use super::*;

use notify_debouncer_full::DebouncedEvent;
use notify_debouncer_full::notify;
use notify_debouncer_full::notify::event::{
    AccessKind, CreateKind, Event, EventKind, ModifyKind, RemoveKind,
};

use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::channel as mpsc_channel;
use std::time::Instant;
use tempfile::TempDir;
use tracing_test::traced_test;

mod check_extensions;
mod debounced_events_result_handler;
mod handle_events;
mod has_matching_event;
mod send_command;

fn event_kind(kind: &str) -> EventKind {
    match kind {
        "access" => EventKind::Access(AccessKind::Any),
        "create" => EventKind::Create(CreateKind::Any),
        "remove" => EventKind::Remove(RemoveKind::Any),
        _ => EventKind::Modify(ModifyKind::Any), //default
    }
}

fn create_debounced_event(kind: &str, path: &str) -> DebouncedEvent {
    DebouncedEvent {
        event: Event {
            kind: event_kind(kind),
            paths: vec![PathBuf::from(path)],
            attrs: Default::default(),
        },
        time: Instant::now(),
    }
}

fn create_debounced_event_result(
    error: bool,
    kind: &str,
) -> DebounceEventResult {
    if error {
        return Err(vec![notify::Error {
            kind: notify::ErrorKind::Generic("custom".to_string()),
            paths: vec![PathBuf::from("./")],
        }]);
    }

    Ok(vec![create_debounced_event(kind, "src/main.rs")])
}

#[test]
fn test_is_mutation() {
    assert!(!is_mutation(&event_kind("access")));
    assert!(is_mutation(&event_kind("create")));
    assert!(is_mutation(&event_kind("remove")));
    assert!(is_mutation(&event_kind("modify")));
}

#[test]
#[traced_test]
fn test_log_event_covers_debug_fields() {
    let event = create_debounced_event("access", "main.rs");
    log_event(&event);
    assert!(logs_contain("event detail"));
}
