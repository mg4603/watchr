use super::*;

mod run_tests {
    use super::*;

    use std::io;
    use std::sync::mpsc::Sender;

    use tempfile::TempDir;

    use cli::CliError::MalformedEntry;
    use cli::WatchArgs;
    use init::InitError;
    use watcher::WatchEvent;

    struct NoOpHandler;

    impl ShutdownHandler for NoOpHandler {
        fn install(
            &self,
            _tx: Sender<WatchEvent>,
        ) -> Result<(), WatcherError> {
            Ok(())
        }
    }

    #[test]
    fn run_init_error_propagates() {
        let dir = PathBuf::from("/nonexistent/path");

        let cli = Cli { command: Commands::Init, verbose: 0 };
        let mut writer = Vec::new();

        let err = run(cli, dir, &NoOpHandler, &mut writer)
            .expect_err("expected run_init error");
        assert!(matches!(
            err,
            AppError::InitError(InitError::Io(e)) if e.kind() == io::ErrorKind::NotFound
        ));
    }

    #[test]
    fn to_entry_error_propagates() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().to_path_buf();
        let cli = Cli {
            command: Commands::Watch(WatchArgs {
                dir: Some(dir.clone()),
                cmd: None,
                config: None,
                ext: None,
            }),
            verbose: 0,
        };
        let mut writer = Vec::new();

        let err = run(cli, dir, &NoOpHandler, &mut writer)
            .expect_err("expected to_entry error");
        assert!(matches!(err, AppError::CliError(MalformedEntry)));
    }

    #[test]
    fn reads_config_path_from_args_and_prepare_config_error_propagates() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().to_path_buf();
        let config_file = dir.join("nonexistent.toml");

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

        let err = run(cli, dir, &NoOpHandler, &mut writer)
            .expect_err("expected parse_config error");
        assert!(
            matches!(err, AppError::ConfigError(ConfigError::Io(e)) if e.kind() == io::ErrorKind::NotFound)
        );
    }
}
