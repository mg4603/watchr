use super::*;

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
