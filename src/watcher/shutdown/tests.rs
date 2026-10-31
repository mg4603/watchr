use super::*;
use std::sync::mpsc::channel as mpsc_channel;

#[test]
fn shutdown_event_is_sent() {
    let (tx, rx) = mpsc_channel();
    send_shutdown(&tx);
    assert!(matches!(rx.recv(), Ok(WatchEvent::Shutdown)));
}
