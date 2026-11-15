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
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tempfile::TempDir;
use tracing_subscriber::layer::SubscriberExt;
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

fn create_debounced_event(
    kind: &str,
    paths: Vec<PathBuf>,
) -> DebouncedEvent {
    DebouncedEvent {
        event: Event {
            kind: event_kind(kind),
            paths: paths,
            attrs: Default::default(),
        },
        time: Instant::now(),
    }
}

fn create_error_debounce_event_result() -> DebounceEventResult {
    Err(vec![notify::Error {
        kind: notify::ErrorKind::Generic("custom".to_string()),
        paths: vec![PathBuf::from("./")],
    }])
}

#[derive(Clone)]
struct LevelRecorder(Arc<Mutex<Vec<tracing::Level>>>);

impl<S: tracing::Subscriber> tracing_subscriber::Layer<S>
    for LevelRecorder
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        self.0.lock().unwrap().push(*event.metadata().level());
    }
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
fn test_is_relevant_path() {
    let tmp = TempDir::new().unwrap();

    let matching_file = tmp.path().join("main.rs");
    let non_matching_file = tmp.path().join("test.txt");
    let dir = tmp.path().join("directory");

    fs::write(&matching_file, "fn main() {}").unwrap();
    fs::write(&non_matching_file, "test").unwrap();
    fs::create_dir(&dir).unwrap();

    let exts = vec!["rs".to_string()];

    assert!(!is_relevant_path(&dir, Some(&exts)));
    assert!(!is_relevant_path(&non_matching_file, Some(&exts)));
    assert!(is_relevant_path(&matching_file, Some(&exts)));
}
