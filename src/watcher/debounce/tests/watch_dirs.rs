use super::*;

struct FakeDirectoryWatcher {
    watched_dirs: Vec<PathBuf>,
    fail_on_call: Option<usize>,
}

impl FakeDirectoryWatcher {
    fn new(fail_on_call: Option<usize>) -> Self {
        Self { watched_dirs: Vec::new(), fail_on_call }
    }
}

impl DirectoryWatcher for FakeDirectoryWatcher {
    fn watch_recursive(
        &mut self,
        path: &Path,
    ) -> Result<(), notify::Error> {
        let call = self.watched_dirs.len();

        if self.fail_on_call == Some(call) {
            return Err(notify::Error::generic("watch failed"));
        }

        self.watched_dirs.push(path.to_path_buf());
        Ok(())
    }
}

fn assert_watch_dirs_succeeds(dirs: &[PathBuf]) {
    let mut watcher = FakeDirectoryWatcher::new(None);
    let result = watch_dirs(&mut watcher, dirs);

    assert!(result.is_ok());
    assert_eq!(watcher.watched_dirs, dirs);
}

fn assert_watch_dirs_fails(
    dirs: &[PathBuf],
    fail_on_call: usize,
    expected_watched_dirs: &[PathBuf],
    expected_error_path: &Path,
) {
    let mut watcher = FakeDirectoryWatcher::new(Some(fail_on_call));
    let result = watch_dirs(&mut watcher, dirs);

    assert!(matches!(
        result,
        Err(
            WatcherError::Watch {path, ..} )if path == expected_error_path,

    ));
    assert_eq!(watcher.watched_dirs, expected_watched_dirs);
}

#[test]
fn watches_single_directory() {
    let dirs = vec![PathBuf::from("/dir1")];
    assert_watch_dirs_succeeds(&dirs);
}

#[test]
fn watches_multiple_directories() {
    let dirs = vec![
        PathBuf::from("/dir1"),
        PathBuf::from("/dir2"),
        PathBuf::from("/dir3"),
    ];
    assert_watch_dirs_succeeds(&dirs);
}

#[test]
fn succeeds_when_no_directories_are_provided() {
    assert_watch_dirs_succeeds(&[]);
}

#[test]
fn returns_error_when_watching_fails() {
    let dirs = vec![PathBuf::from("/dir1")];
    assert_watch_dirs_fails(&dirs, 0, &[], &dirs[0]);
}

#[test]
fn stops_watching_after_first_failure() {
    let dirs = vec![
        PathBuf::from("/dir1"),
        PathBuf::from("/dir2"),
        PathBuf::from("/dir3"),
    ];
    assert_watch_dirs_fails(&dirs, 1, &dirs[..1], &dirs[1]);
}
