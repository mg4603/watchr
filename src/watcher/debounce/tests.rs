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
#[traced_test]
fn test_send_command() {
    let (tx, rx) = mpsc_channel();
    send_command(
        tx,
        "echo test_ran".to_string(),
        Some("test".to_string()),
    );

    assert!(logs_contain("sending command event"));
    assert!(matches!(rx.try_recv(), Ok(WatchEvent::Command { .. })))
}

#[test]
#[traced_test]
fn test_log_event_covers_debug_fields() {
    let event = create_debounced_event("access", "main.rs");
    log_event(&event);
    assert!(logs_contain("event detail"));
}

#[test]
#[traced_test]
fn test_has_matching_event_no_match() {
    let tmp_dir = TempDir::new().unwrap();
    let file_1 = tmp_dir.path().join("file_access.txt");
    fs::write(&file_1, "test").unwrap();
    let file_2 = tmp_dir.path().join("readme.md");
    fs::write(&file_2, "test").unwrap();

    let events = vec![
        create_debounced_event("access", &file_1.display().to_string()),
        create_debounced_event("modify", &file_2.display().to_string()),
    ];

    let exts = vec!["rs".to_string()];
    assert!(!has_matching_event(&events, Some(&exts)));

    assert!(logs_contain("file_access"));
    assert!(logs_contain("readme.md"));
}

#[test]
#[traced_test]
fn test_has_matching_event_with_short_circuit() {
    let tmp_dir = TempDir::new().unwrap();
    let file_1 = tmp_dir.path().join("file_access.txt");
    fs::write(&file_1, "test").unwrap();
    let file_2 = tmp_dir.path().join("file_a.rs");
    fs::write(&file_2, "test").unwrap();
    let file_3 = tmp_dir.path().join("file_b.rs");
    fs::write(&file_3, "test").unwrap();

    let events = vec![
        create_debounced_event("access", &file_1.display().to_string()),
        create_debounced_event("create", &file_2.display().to_string()),
        create_debounced_event("remove", &file_3.display().to_string()),
    ];
    let exts = vec!["rs".to_string()];
    assert!(has_matching_event(&events, Some(&exts)));

    assert!(logs_contain("file_access"));
    assert!(logs_contain("file_a"));
    assert!(!logs_contain("file_b"));
}

#[traced_test]
#[test]
fn test_handle_events_no_ext() {
    let result = create_debounced_event_result(false, "modify");
    let (tx, rx) = mpsc_channel();
    handle_events(result, None, None, "pwd".to_string(), tx);

    assert!(logs_contain("received debounced events count=1"));
    assert!(matches!(rx.try_recv(), Ok(WatchEvent::Command { .. })));
}

#[test]
fn test_handle_events_name_in_emitted_watch_event() {
    let result = create_debounced_event_result(false, "modify");
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
    let result = create_debounced_event_result(false, "modify");
    let (tx, rx) = mpsc_channel();
    handle_events(
        result,
        None,
        Some(vec!["rs".to_string()]),
        "pwd".to_string(),
        tx,
    );

    assert!(matches!(rx.try_recv(), Ok(WatchEvent::Command { .. })));
}

#[test]
fn test_handle_event_no_matching_ext() {
    let result = create_debounced_event_result(false, "modify");
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
    let result = create_debounced_event_result(true, "modify");
    let (tx, rx) = mpsc_channel();
    handle_events(result, None, None, "pwd".to_string(), tx);

    // mpsc::TryRecvError::Empty
    assert!(matches!(rx.try_recv(), Err(..)))
}

#[test]
fn test_handle_events_non_mutation_event() {
    let result = create_debounced_event_result(false, "access");
    let (tx, rx) = mpsc_channel();
    handle_events(result, None, None, "pwd".to_string(), tx);
    assert!(matches!(rx.try_recv(), Err(..)));
}

#[test]
fn test_handle_events_mutation_event() {
    let result = create_debounced_event_result(false, "modify");
    let (tx, rx) = mpsc_channel();
    handle_events(result, None, None, "pwd".to_string(), tx);
    assert!(matches!(rx.try_recv(), Ok(WatchEvent::Command { .. })));
}

#[test]
fn test_is_mutation() {
    assert!(!is_mutation(&event_kind("access")));
    assert!(is_mutation(&event_kind("create")));
    assert!(is_mutation(&event_kind("remove")));
    assert!(is_mutation(&event_kind("modify")));
}

#[test]
fn test_check_extensions() {
    let exts: Vec<String> = vec!["rs".to_string()];
    assert!(check_extensions(Path::new("src/main.rs"), None));
    assert!(check_extensions(Path::new("src/main.rs"), Some(&exts)));
    assert!(!check_extensions(Path::new("src/main.py"), Some(&exts)));
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

    let result = create_debounced_event_result(false, "modify");
    handler(result);

    assert!(matches!(rx.try_recv(), Ok(WatchEvent::Command { .. })))
}
