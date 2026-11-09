use super::*;

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
