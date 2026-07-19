# Changelog

All noteable changes to this project will be documented in this 
file.

This format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [0.1.0] - 2026-07-19

### Added
- Initial project setup with CI workflow (#1)
- Core depdencencies: `notify-debouncer-full`, `clap`, `serde`, 
  `toml` (#3)
- Config file parsing via `.watchr.toml` with `WatchrEntry` and
  `WatchrConfig` structs (#4)
- CLI interface with `--dir`, `--ext`, `--cmd`, and `--config` 
  flags (#4)
- File watcher core: watch mutliple paths, filter by extension,
  debounce rapid changes, run command on change (#6)
- Graceful shutdown on Ctrl+C (#7)
- `watchr init` command to generate a `.watchr.toml` template (#9)
- Config file resolve that walks up the directory tree from a 
  starting point (#11)
- Full orchestration in `main`: CLI parsing, config resolution,
  directory validation, and watcher startup (#15)
- README with installation, configuration and usage documentation
  (#19)
- Inline documentation for all modules, structs, enums, and 
  functions (#20)

### Changed
- Command output formatting: replaced raw debug output with 
  readable success/failure status, exit codes, and formatted
  stdout/stderr (#23)

[0.1.0]: https://github.com/mg4603/watchr/releases/tag/v0.1.0
