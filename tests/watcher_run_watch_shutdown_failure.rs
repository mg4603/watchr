use watchr::config::WatcherConfig;
use watchr::entry::WatcherEntry;
use watchr::watcher::shutdown::{CtrlCHandler, ShutdownHandler};
use watchr::watcher::{WatcherError, run_watch};

use std::sync::mpsc::channel as mpsc_channel;

use tempfile::TempDir;

#[test]
fn handler_registration_failure() {
    let tmp = TempDir::new().unwrap();

    let (tx, _rx) = mpsc_channel();
    let _ = CtrlCHandler.install(tx);

    let entry = WatcherEntry {
        name: Some("test".to_string()),
        dirs: vec![tmp.path().to_path_buf()],
        ext: None,
        command: "cargo test".to_string(),
    };
    let config = WatcherConfig { debounce_ms: 500, entries: vec![entry] };
    let mut stdout = Vec::new();
    let err = run_watch(config, &mut stdout, CtrlCHandler).unwrap_err();

    assert!(matches!(err, WatcherError::SignalHandler(_)));
}
