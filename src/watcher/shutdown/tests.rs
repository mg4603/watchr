use super::*;
use std::sync::mpsc::channel as mpsc_channel;

#[test]
fn send_shutdown_emits_shutdown_event() {
    let (tx, rx) = mpsc_channel();
    send_shutdown(&tx);
    assert!(matches!(rx.recv(), Ok(WatchEvent::Shutdown)));
}

#[test]
fn send_shutdown_ignores_closed_channel() {
    let (tx, rx) = mpsc_channel();
    drop(rx);
    send_shutdown(&tx); // must not panic
}

#[test]
fn ctrc_c_handler_registration() {
    let (tx, _rx) = mpsc_channel();
    assert!(CtrlCHandler.install(tx.clone()).is_ok());
    assert!(CtrlCHandler.install(tx).is_err());
}
