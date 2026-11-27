use super::*;

use std::sync::mpsc::{Receiver, TryRecvError};

fn handle_test_events(
    kind: &str,
    paths: Vec<PathBuf>,
    extensions: Option<Vec<String>>,
) -> (Sender<WatchEvent>, Receiver<WatchEvent>) {
    let (tx, rx) = mpsc_channel();
    let result = Ok(vec![create_debounced_event(kind, paths)]);
    handle_events(
        result,
        None,
        extensions,
        "pwd".to_string(),
        tx.clone(),
    );

    (tx, rx)
}

// -------------------- successful event handling   -----------------
#[traced_test]
#[test]
fn no_extension_filter() {
    let (file, _tmp) = create_temp_file("file.txt", "test");
    let (_tx, rx) = handle_test_events("modify", vec![file], None);

    assert!(logs_contain("received debounced events count=1"));
    assert!(matches!(rx.try_recv(), Ok(WatchEvent::Command { .. })));
}

#[test]
fn mutation_event() {
    let (file, _tmp) = create_temp_file("file.txt", "test");
    let (_tx, rx) = handle_test_events("modify", vec![file], None);

    assert!(matches!(rx.try_recv(), Ok(WatchEvent::Command { .. })));
}

#[test]
fn non_mutation_event() {
    let (file, _tmp) = create_temp_file("file.txt", "test");
    let (_tx, rx) = handle_test_events("access", vec![file], None);

    assert!(matches!(rx.try_recv(), Err(TryRecvError::Empty)))
}

#[test]
fn empty_events() {
    let (tx, rx) = mpsc_channel();
    let result = Ok(vec![]);
    handle_events(result, None, None, "pwd".to_string(), tx.clone());

    assert!(matches!(rx.try_recv(), Err(TryRecvError::Empty)))
}

// --------------- Command emission
#[test]
fn name_in_emitted_watch_event() {
    let (file, _tmp) = create_temp_file("file.txt", "test");
    let result = Ok(vec![create_debounced_event("modify", vec![file])]);
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
fn multiple_events_emit_single_command() {
    let (file_1, _tmp) = create_temp_file("file1.txt", "test");
    let (file_2, _tmp) = create_temp_file("file2.txt", "test");
    let (file_3, _tmp) = create_temp_file("file3.txt", "test");

    let result = Ok(vec![
        create_debounced_event("modify", vec![file_1]),
        create_debounced_event("create", vec![file_2]),
        create_debounced_event("remove", vec![file_3]),
    ]);

    let (tx, rx) = mpsc_channel();
    handle_events(result, None, None, "pwd".to_string(), tx);

    assert!(matches!(rx.try_recv(), Ok(WatchEvent::Command { .. })));
    assert!(matches!(rx.try_recv(), Err(..)));
}

// --------------------- Extension filtering integration --------------
#[test]
fn matching_ext() {
    let (file, _tmp) = create_temp_file("main.rs", "fn main() {}");
    let exts = vec!["rs".to_string()];

    let (_tx, rx) = handle_test_events("modify", vec![file], Some(exts));

    assert!(matches!(rx.try_recv(), Ok(WatchEvent::Command { .. })));
}

#[test]
fn no_matching_ext() {
    let (file, _tmp) = create_temp_file("main.rs", "fn main() {}");
    let exts = vec!["txt".to_string()];
    let (_tx, rx) = handle_test_events("modify", vec![file], Some(exts));

    assert!(matches!(rx.try_recv(), Err(TryRecvError::Empty)))
}

// ------------------- Error handling ------------------
#[test]
#[traced_test]
fn error_result() {
    let result = create_error_debounce_event_result();

    let (tx, rx) = mpsc_channel();
    handle_events(result, None, None, "pwd".to_string(), tx.clone());

    assert!(logs_contain("failed to process file watch event"));
    assert!(matches!(rx.try_recv(), Err(TryRecvError::Empty)))
}
