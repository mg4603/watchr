use super::*;

mod resolve_config_path {

    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn explicit_file_is_returned_as_is() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");
        fs::write(&path, "test").unwrap();

        let result = resolve_config_path(&path).unwrap();
        assert_eq!(result, path);
    }

    #[test]
    fn nonexistent_path_is_invalid() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join(".watchr.toml");

        let result = resolve_config_path(&path);
        assert!(matches!(result, Err(ResolverError::InvalidPath)));
    }

    #[test]
    fn directory_delegates_to_find_in_tree() {
        let tmp = TempDir::new().unwrap();
        let conf = tmp.path().join(".watchr.toml");
        fs::write(&conf, "").unwrap();

        let result = resolve_config_path(tmp.path()).unwrap();
        assert_eq!(result, conf);
    }

    #[test]
    fn resolves_relative_path_to_config_file() {
        let tmp = TempDir::new().unwrap();
        let conf = tmp.path().join(".watchr.toml");
        fs::write(&conf, "test").unwrap();

        let sub_dir = tmp.path().join("a");
        fs::create_dir(&sub_dir).unwrap();

        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(tmp.path()).unwrap();
        let result = resolve_config_path(Path::new("a"));
        std::env::set_current_dir(original_cwd).unwrap();

        let resolved = result.expect("relative path should resolve");
        assert!(resolved.is_absolute());
        assert_eq!(resolved, conf);
    }
}

mod find_in_tree {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn found_at_start() {
        let tmp = TempDir::new().unwrap();
        let conf = tmp.path().join(".watchr.toml");
        fs::write(&conf, "test").unwrap();

        let result = find_in_tree(tmp.path(), ".watchr.toml").unwrap();
        assert_eq!(result, conf);
    }

    #[test]
    fn not_found() {
        let tmp = TempDir::new().unwrap();
        let result = find_in_tree(tmp.path(), ".watchr.toml");
        assert!(matches!(result, Err(ResolverError::NotFound)));
    }

    #[test]
    fn found_in_ancestor() {
        let tmp = TempDir::new().unwrap();
        let sub_dir = tmp.path().join("a");
        fs::create_dir(&sub_dir).unwrap();

        let conf = tmp.path().join(".watchr.toml");
        fs::write(&conf, "test").unwrap();

        let result = find_in_tree(&sub_dir, ".watchr.toml").unwrap();
        assert_eq!(result, conf);
    }

    #[test]
    fn skips_directory_named_like_config() {
        let tmp = TempDir::new().unwrap();
        let dir_named_like_config = tmp.path().join(".watchr.toml");
        fs::create_dir(&dir_named_like_config).unwrap();

        let result = find_in_tree(&dir_named_like_config, ".watchr.toml");
        assert!(matches!(result, Err(ResolverError::NotFound)));
    }

    #[test]
    fn prefers_nearest_match() {
        let tmp = TempDir::new().unwrap();
        let sub_dir = tmp.path().join("a");
        fs::create_dir(&sub_dir).unwrap();

        let conf = tmp.path().join(".watchr.toml");
        let sub_conf = sub_dir.join(".watchr.toml");

        fs::write(&conf, "test").unwrap();
        fs::write(&sub_conf, "test").unwrap();

        let result = find_in_tree(&sub_dir, ".watchr.toml").unwrap();
        assert_eq!(result, sub_conf);
    }
}
