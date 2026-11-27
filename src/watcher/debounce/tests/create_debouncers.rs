use super::*;

use std::sync::mpsc::channel as mpsc_channel;

#[test]
fn empty_entries() {
    let (tx, _rx) = mpsc_channel();

    let result = create_debouncers(100, vec![], tx);

    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn multiple_entries() {
    let (tx, _rx) = mpsc_channel();

    let dir = TempDir::new().unwrap();
    let dir1 = dir.path().join("dir1");
    let dir2 = dir.path().join("dir2");
    fs::create_dir_all(&dir1).unwrap();
    fs::create_dir_all(&dir2).unwrap();

    let entries = vec![
        WatcherEntry {
            name: Some("test1".to_string()),
            dirs: vec![dir1],
            ext: None,
            command: "pwd".to_string(),
        },
        WatcherEntry {
            name: Some("test2".to_string()),
            dirs: vec![dir2],
            ext: None,
            command: "pwd".to_string(),
        },
    ];

    let result = create_debouncers(100, entries, tx);

    assert!(result.is_ok());

    let debouncers = result.unwrap();
    assert_eq!(debouncers.len(), 2);
}

#[test]
fn propagates_entry_creation_error() {
    let (tx, _rx) = mpsc_channel();

    let dir = PathBuf::from("/nonexistent");
    let entries = vec![WatcherEntry {
        name: Some("test".to_string()),
        dirs: vec![dir.clone()],
        ext: None,
        command: "pwd".to_string(),
    }];

    let err = create_debouncers(100, entries, tx).unwrap_err();
    assert!(matches!(err, WatcherError::Watch{path,..} if path == dir));
}
