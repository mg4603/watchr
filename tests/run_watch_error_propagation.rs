use watchr::cli::{Cli, Commands, WatchArgs};
use watchr::watcher::shutdown::{CtrlCHandler, ShutdownHandler};
use watchr::{AppError, run};

use tempfile::TempDir;

use std::fs;
use std::sync::mpsc::channel as mpsc_channel;

#[test]
fn run_watch_error_propagates() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path().to_path_buf();

    let config = format!(
        r#"
[[watcher]]
dirs = ["{}"]
command = "echo watchr_test_ran"
"#,
        dir.display()
    );
    let config_file = tmp.path().join("config.toml");
    fs::write(&config_file, config).unwrap();

    let cli = Cli {
        command: Commands::Watch(WatchArgs {
            dir: None,
            cmd: None,
            config: Some(config_file),
            ext: None,
        }),
        verbose: 0,
    };
    let mut writer = Vec::new();

    let (tx, _rx) = mpsc_channel();
    let _ = CtrlCHandler.install(tx);

    let err = run(cli, dir, &CtrlCHandler, &mut writer)
        .expect_err("expected run_watch error");

    assert!(matches!(err, AppError::WatcherError(_)));
}
