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

    let (file, _tmp) = create_temp_file("file.txt", "test");
    let result = Ok(vec![create_debounced_event("modify", vec![file])]);
    handler(result);

    assert!(matches!(rx.try_recv(), Ok(WatchEvent::Command { .. })))
}
