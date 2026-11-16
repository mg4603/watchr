use super::*;

#[test]
fn handler_emits_command_on_file_modification() {
    let (tx, rx) = mpsc_channel();
    let mut handler = debounced_events_result_handler(
        Some("test".to_string()),
        None,
        "pwd".to_string(),
        tx,
    );

    let (file, _tmp) = create_temp_file("file.txt", "test");
    let result = Ok(vec![create_debounced_event("modify", vec![file])]);

    handler(result);

    match rx.try_recv().unwrap() {
        WatchEvent::Command { cmd, name } => {
            assert_eq!(cmd, "pwd");
            assert_eq!(name, Some("test".to_string()));
        }
        event => panic!("Expected WatchEvent::Command, got {event:?}"),
    }
}

#[test]
fn handler_accepts_missing_optional_configuration() {
    let cmd = "pwd".to_string();
    let (tx, rx) = mpsc_channel();
    let mut handler =
        debounced_events_result_handler(None, None, cmd.clone(), tx);

    let (file, _tmp) = create_temp_file("file.txt", "test");
    let result = Ok(vec![create_debounced_event("modify", vec![file])]);

    handler(result);

    match rx.try_recv().unwrap() {
        WatchEvent::Command { cmd: c, name } => {
            assert_eq!(c, cmd);
            assert_eq!(name, None);
        }
        event => panic!("Expected WatchEvent::Command got {event:?}"),
    }
}

#[test]
fn handler_can_process_multiple_event_results() {
    let (tx, rx) = mpsc_channel();
    let cmd = "pwd".to_string();
    let mut handler =
        debounced_events_result_handler(None, None, cmd.clone(), tx);

    let (file, _tmp) = create_temp_file("file.txt", "test");
    let result1 =
        Ok(vec![create_debounced_event("modify", vec![file.clone()])]);
    let result2 = Ok(vec![create_debounced_event("modify", vec![file])]);
    handler(result1);
    handler(result2);

    assert_eq!(rx.try_iter().count(), 2);
}
