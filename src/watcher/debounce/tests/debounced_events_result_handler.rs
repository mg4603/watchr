use super::*;

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
