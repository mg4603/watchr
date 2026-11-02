use watchr::config::WatcherConfig;
use watchr::entry::WatcherEntry;
use watchr::watcher::WatcherError;
use watchr::watcher::run_watch;
use watchr::watcher::shutdown::CtrlCHandler;

use tempfile::TempDir;

use std::fs;
use std::io::{Error, Result, Write};

pub(crate) struct FailingWriter;
impl Write for FailingWriter {
    fn write(&mut self, _buf: &[u8]) -> Result<usize> {
        Err(Error::other("write failed"))
    }
    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

#[test]
fn run_event_loop_failure() {
    let tmp = TempDir::new().unwrap();
    let entry = WatcherEntry {
        name: Some("test".to_string()),
        dirs: vec![tmp.path().to_path_buf()],
        ext: None,
        command: "echo test".to_string(),
    };
    let config = WatcherConfig { debounce_ms: 500, entries: vec![entry] };

    let handle = std::thread::spawn(move || {
        let mut writer = FailingWriter;
        run_watch(config, &mut writer, CtrlCHandler)
    });

    std::thread::sleep(std::time::Duration::from_millis(200));
    let file = tmp.path().join("test");
    fs::write(&file, "test").unwrap();

    let result = handle.join().unwrap();
    assert!(matches!(result, Err(WatcherError::WriteError(_))));
}
