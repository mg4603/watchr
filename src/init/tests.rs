use super::*;

use crate::test_helpers::{FailingWriter, FlushFailingWriter};

use std::fs;
use tempfile::TempDir;

mod run_init_tests {
    use super::*;

    #[test]
    fn happy_path() {
        let tmp_dir = TempDir::new().unwrap();
        let path = tmp_dir.path();

        let _ = run_init(path);

        let config_path = path.join(".watchr.toml");
        assert!(config_path.exists());
    }

    #[test]
    #[cfg(unix)]
    fn permission_denied() {
        use std::os::unix::fs::PermissionsExt;

        let tmp_dir = TempDir::new().unwrap();
        let path = tmp_dir.path();

        let perms = fs::Permissions::from_mode(0o555);
        fs::set_permissions(path, perms).unwrap();

        let res = run_init(path);

        // Restore permissions for cleanup
        let perms = fs::Permissions::from_mode(0o755);
        fs::set_permissions(path, perms).unwrap();

        assert!(matches!(res, Err(InitError::Io(_))));
    }
}

mod open_new_tests {
    use super::*;

    #[test]
    fn already_exists() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("test");

        open_new(&path).unwrap();
        let err = open_new(&path).unwrap_err();

        assert!(matches!(err, InitError::FileAlreadyExists));
    }

    #[test]
    fn creates_new_file() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("test");

        open_new(&path).unwrap();

        assert!(path.exists());
    }

    #[test]
    fn existing_content_is_untouched() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("test");
        fs::write(&path, "existing").unwrap();

        open_new(&path).unwrap_err();

        assert_eq!(fs::read_to_string(&path).unwrap(), "existing");
    }

    #[test]
    fn missing_parent_dir_is_io_error() {
        let path = TempDir::new().unwrap().path().join("na").join("test");
        let err = open_new(&path).unwrap_err();
        assert!(matches!(err, InitError::Io(_)));
    }
}

mod write_template_tests {
    use super::*;
    #[test]
    fn happy_path() {
        let mut buf: Vec<u8> = Vec::new();
        write_template(&mut buf, "hello").unwrap();
        assert_eq!(buf, "hello".as_bytes());
    }

    #[test]
    fn write_errors_propagate() {
        let mut writer = FailingWriter;
        let err = write_template(&mut writer, "hello").unwrap_err();
        assert!(matches!(
                err, InitError::Io(e) if e.kind() == std::io::ErrorKind::Other));
    }

    #[test]
    fn flush_errors_propagate() {
        let mut writer = FlushFailingWriter;
        let err = write_template(&mut writer, "hello").unwrap_err();
        assert!(
            matches!(err, InitError::Io(e) if e.kind() == std::io::ErrorKind::Other)
        );
    }
}
