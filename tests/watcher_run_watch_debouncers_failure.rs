use watchr::config::WatcherConfig;
use watchr::entry::WatcherEntry;
use watchr::watcher::WatcherError;
use watchr::watcher::run_watch;
use watchr::watcher::shutdown::CtrlCHandler;

use tempfile::TempDir;

#[test]
fn create_debouncers_failure() {
    let dir = TempDir::new().unwrap().path().join("non/existent/dir");
    let entry = WatcherEntry {
        name: Some("test".to_string()),
        dirs: vec![dir],
        ext: None,
        command: "cargo test".to_string(),
    };
    let config = WatcherConfig { debounce_ms: 500, entries: vec![entry] };
    let mut writer = Vec::new();
    let err = run_watch(config, &mut writer, CtrlCHandler).unwrap_err();

    assert!(matches!(err, WatcherError::Notify(_)));
}
