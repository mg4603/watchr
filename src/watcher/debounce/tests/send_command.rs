use super::*;

#[test]
fn logs_at_debug_level() {
    let recorder = LevelRecorder(Arc::new(Mutex::new(Vec::new())));
    let _guard = tracing::subscriber::set_default(
        tracing_subscriber::registry()
            .with(tracing_subscriber::filter::LevelFilter::INFO)
            .with(recorder.clone()),
    );

    let (tx, _rx) = mpsc_channel();
    send_command(tx, "level_check".to_string(), None);

    // At INFO, a debug-level event must not be recorded
    assert!(recorder.0.lock().unwrap().is_empty());
}

#[test]
#[traced_test]
fn with_name() {
    let (tx, rx) = mpsc_channel();
    send_command(tx, "echo test".to_string(), Some("test".to_string()));

    assert!(logs_contain("sending Command event"));
    assert!(logs_contain("echo test"));
    match rx.try_recv() {
        Ok(WatchEvent::Command { cmd, name }) => {
            assert_eq!(cmd, "echo test");
            assert_eq!(name, Some("test".to_string()));
        }
        other => panic!("expected Command event, got {other:?}"),
    }
}

#[test]
#[traced_test]
fn without_name() {
    let (tx, rx) = mpsc_channel();
    send_command(tx, "ls -la".to_string(), None);

    assert!(logs_contain("sending Command event"));
    assert!(logs_contain("ls -la"));
    match rx.try_recv() {
        Ok(WatchEvent::Command { cmd, name }) => {
            assert_eq!(cmd, "ls -la");
            assert_eq!(name, None);
        }
        other => panic!("expected Command event, got {other:?}"),
    }
}

#[test]
#[traced_test]
fn empty_command() {
    let (tx, rx) = mpsc_channel();
    send_command(tx, String::new(), None);

    assert!(logs_contain("sending Command event"));
    match rx.try_recv() {
        Ok(WatchEvent::Command { cmd, name }) => {
            assert_eq!(cmd, "");
            assert_eq!(name, None);
        }
        other => panic!("expected Command event, got {other:?}"),
    }
}

#[test]
#[traced_test]
fn unicode_and_special_chars() {
    let (tx, rx) = mpsc_channel();
    let cmd = "echo \"héllo $HOME | grep 'x'\"".to_string();
    send_command(tx, cmd.clone(), Some("ünïcode".to_string()));

    assert!(logs_contain("sending Command event"));
    assert!(logs_contain("echo \"héllo $HOME | grep 'x'\""));
    match rx.recv() {
        Ok(WatchEvent::Command { cmd: c, name }) => {
            assert_eq!(c, cmd);
            assert_eq!(name, Some("ünïcode".to_string()));
        }
        other => panic!("expected Command event, got {other:?}"),
    }
}

#[test]
#[traced_test]
fn multiple_events_in_order() {
    let (tx, rx) = mpsc_channel();
    send_command(tx.clone(), "first".to_string(), Some("a".to_string()));
    send_command(tx, "second".to_string(), None);

    assert!(
        matches!(rx.try_recv(), Ok(WatchEvent::Command{cmd, ..}) if cmd == "first")
    );
    assert!(matches!(
        rx.try_recv(),
        Ok(WatchEvent::Command { cmd, name: None }) if cmd == "second"
    ));
    assert!(rx.try_recv().is_err()); // nothing extra was sent
}

#[test]
#[traced_test]
fn dropped_receiver_logs_warning() {
    let (tx, rx) = mpsc_channel();
    drop(rx);

    send_command(tx, "ghost".to_string(), None);
    assert!(logs_contain("sending Command event"));
    assert!(logs_contain("ghost"));
    assert!(logs_contain(
        "receiver dropped, failed to send Command event"
    ));
}

#[test]
fn logs_at_warn_level_when_receiver_dropped() {
    let recorder = LevelRecorder(Arc::new(Mutex::new(Vec::new())));
    let _guard = tracing::subscriber::set_default(
        tracing_subscriber::registry()
            .with(tracing_subscriber::filter::LevelFilter::WARN)
            .with(recorder.clone()),
    );

    let (tx, rx) = mpsc_channel();
    drop(rx);

    send_command(tx, "gone".to_string(), None);

    let events = recorder.0.lock().unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0], tracing::Level::WARN);
}

#[test]
fn no_warn_when_send_succeeds() {
    let recorder = LevelRecorder(Arc::new(Mutex::new(Vec::new())));
    let _guard = tracing::subscriber::set_default(
        tracing_subscriber::registry()
            .with(tracing_subscriber::filter::LevelFilter::WARN)
            .with(recorder.clone()),
    );

    let (tx, _rx) = mpsc_channel();
    send_command(tx, "ok_check".to_string(), None);

    assert!(recorder.0.lock().unwrap().is_empty());
}
