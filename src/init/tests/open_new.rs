use super::*;

#[test]
fn creates_new_file_when_missing() {
    let dir = TempDir::new().unwrap();
    let f = open_new(&config_path(&dir)).expect("should create file");
    assert!(config_path(&dir).exists());
    drop(f);
}

#[test]
fn second_open_new_call_fails_with_file_already_exists() {
    let dir = TempDir::new().unwrap();
    let path = config_path(&dir);
    let f = open_new(&path).expect("first calls should create the file");
    drop(f);

    let err = open_new(&path).expect_err("second call must fail");
    assert!(matches!(err, InitError::FileAlreadyExists(p) if p == path));
}

#[test]
fn returns_file_already_exists_when_present() {
    let dir = TempDir::new().unwrap();
    let path = config_path(&dir);
    std::fs::write(&path, b"existing").unwrap();

    let err = open_new(&path).expect_err("should fail when file exists");
    assert!(matches!(err, InitError::FileAlreadyExists(p) if p == path));
    assert_eq!(std::fs::read(path).unwrap(), b"existing");
}

#[test]
fn created_file_accepts_writes() {
    let dir = TempDir::new().unwrap();
    let mut f = open_new(&config_path(&dir)).unwrap();
    f.write_all(b"[watcher]\n").expect("writable");
    f.sync_all().expect("sync works");
    drop(f);

    assert_eq!(
        std::fs::read_to_string(config_path(&dir)).unwrap(),
        "[watcher]\n"
    );
}

#[test]
fn io_error_for_missing_parent_dir() {
    let path = TempDir::new().unwrap().path().join("na").join("test");
    let err = open_new(&path).expect_err("should fail");
    assert!(matches!(err, InitError::Io(_)));
    assert!(err.to_string().starts_with("failed to write config file"));
}

#[test]
fn io_error_when_path_is_a_directory() {
    let dir = TempDir::new().unwrap();
    let sub = dir.path().join("subdir");
    std::fs::create_dir(&sub).unwrap();

    let err =
        open_new(&sub).expect_err("opening a directory should fail");
    assert!(
        matches!(err, InitError::Io(e) if e.kind() == io::ErrorKind::InvalidInput)
    );
}
