use super::*;

#[test]
fn test_to_entry_init() {
    let init = Commands::Init;
    assert!(init.to_entry().unwrap().is_none());
}

#[test]
fn test_to_entry_watch_dir_none() {
    let watch = Commands::Watch {
        dir: None,
        ext: None,
        config: None,
        cmd: Some("cargo test".to_string()),
    };
    assert!(matches!(watch.to_entry(), Err(CliError::MalformedEntry)));
}

#[test]
fn test_to_entry_cmd_dir_none() {
    let watch =
        Commands::Watch { dir: None, ext: None, config: None, cmd: None };
    assert!(watch.to_entry().unwrap().is_none())
}

#[test]
fn test_to_entry_cmd_none() {
    let watch = Commands::Watch {
        dir: Some(PathBuf::from("./")),
        ext: None,
        config: None,
        cmd: None,
    };
    assert!(matches!(watch.to_entry(), Err(CliError::MalformedEntry)));
}

#[test]
fn test_to_entry_with_extension() {
    let watch = Commands::Watch {
        dir: Some(PathBuf::from("./")),
        ext: Some(vec!["rs".to_string(), "toml".to_string()]),
        cmd: Some("cargo test".to_string()),
        config: None,
    };

    let entry = watch.to_entry().unwrap().unwrap();
    assert_eq!(entry.command, "cargo test".to_string());
    assert_eq!(
        entry.ext,
        Some(vec!["rs".to_string(), "toml".to_string()])
    );
}

#[test]
fn test_to_entry_happy_path() {
    let watch = Commands::Watch {
        dir: Some(PathBuf::from("./")),
        ext: None,
        config: None,
        cmd: Some("cargo test".to_string()),
    };

    assert!(matches!(
        watch.to_entry().unwrap(),
        Some(WatcherEntry { .. })
    ));
    let entry = watch.to_entry().unwrap().unwrap();
    assert_eq!(entry.dirs, vec![PathBuf::from("./")]);
    assert!(entry.ext.is_none());
    assert_eq!(entry.command, "cargo test".to_string());
}

#[test]
fn test_config_path_init() {
    let init = Commands::Init;
    assert!(init.config_path().is_none());
}

#[test]
fn test_config_path_watch_config_none() {
    let watch =
        Commands::Watch { dir: None, ext: None, cmd: None, config: None };
    assert!(watch.config_path().is_none())
}

#[test]
fn test_config_path_watch_config_is_not_none() {
    let watch = Commands::Watch {
        dir: None,
        ext: None,
        cmd: None,
        config: Some(PathBuf::from("./")),
    };
    assert_eq!(watch.config_path(), Some(PathBuf::from("./").as_path()));
}

#[test]
fn test_is_init_true() {
    let init = Commands::Init;
    assert!(init.is_init());
}

#[test]
fn test_is_init_false() {
    let watch =
        Commands::Watch { dir: None, ext: None, cmd: None, config: None };
    assert!(!watch.is_init());
}
