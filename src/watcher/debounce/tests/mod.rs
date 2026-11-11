use super::*;

use notify_debouncer_full::DebouncedEvent;
use notify_debouncer_full::notify;
use notify_debouncer_full::notify::event::{
    AccessKind, AccessMode, CreateKind, DataChange, Event, EventKind,
    MetadataKind, ModifyKind, RemoveKind, RenameMode,
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
mod log_errors;
mod log_event;
mod send_command;

fn create_temp_file(filename: &str, content: &str) -> (PathBuf, TempDir) {
    let tmp_dir = TempDir::new().unwrap();
    let file = tmp_dir.path().join(filename);
    fs::write(&file, content).unwrap();

    (file, tmp_dir)
}

fn event_kind(kind: &str) -> EventKind {
    match kind {
        "access" => EventKind::Access(AccessKind::Any),
        "create" => EventKind::Create(CreateKind::Any),
        "remove" => EventKind::Remove(RemoveKind::Any),
        "other" => EventKind::Other,
        _ => EventKind::Modify(ModifyKind::Any), //default
    }
}

fn create_debounced_event(kind: &str, paths: &[&str]) -> DebouncedEvent {
    DebouncedEvent {
        event: Event {
            kind: event_kind(kind),
            paths: paths.iter().map(|p| PathBuf::from(*p)).collect(),
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

    Ok(vec![create_debounced_event(kind, &["src/main.rs"])])
}

#[test]
fn test_is_mutation() {
    let cases = [
        (EventKind::Create(CreateKind::File), true),
        (EventKind::Create(CreateKind::Folder), true),
        (EventKind::Modify(ModifyKind::Data(DataChange::Any)), true),
        (
            EventKind::Modify(ModifyKind::Metadata(MetadataKind::Any)),
            true,
        ),
        (EventKind::Modify(ModifyKind::Name(RenameMode::Any)), true),
        (EventKind::Remove(RemoveKind::File), true),
        (EventKind::Remove(RemoveKind::Folder), true),
        (EventKind::Access(AccessKind::Read), false),
        (EventKind::Access(AccessKind::Close(AccessMode::Write)), false),
        (EventKind::Other, false),
    ];
    for (kind, expected) in cases {
        assert_eq!(is_mutation(&kind), expected, "failed for {kind:?}");
    }
}

#[test]
#[traced_test]
fn test_log_event_covers_debug_fields() {
    let event = create_debounced_event("access", &["main.rs"]);
    log_event(&event);
    assert!(logs_contain("event detail"));
}
