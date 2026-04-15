# watchr

Watch directories and run commands automatically when files 
change.

## Features
- Watch multiple directories simultaneously
- Filter by file extension
- Debounced change detection (configurable)
- TOML configurable file or pure CLI mode
- Graceful shutdown on `Ctrl+C`
- Built-in rust for performance and reliability

## Installation

### From source
```bash
git clone https://github.com/mg4603/watchr.git
cd watchr
cargo build --release
# Binary will be at target/release/watchr
```

### Using Cargo
```bash
cargo install --path .
```

## Quick Start

### Using configuration file

Create a `.watchr.toml` in your project:

```bash
watchr init
```

Edit `.watchr.toml`:
```toml
debounce_ms = 500

[[watcher]]
name = "tests"
dirs = ["src/", "tests/"]
ext = ["rs"]
command = "cargo test"

[[watcher]]
name = "lint"
dirs = ["src/"]
command = "cargo clippy"
```

Start watching:
```bash
watchr watch
```

### Pure CLI Mode

Watch without a config file:

```bash
watchr watch src/ --cmd "cargo test"
```

Watch with extension filter:

```bash
watchr watch src/ --ext rs,toml --cmd "cargo test"
```

## Configuration

## Config File Format

`.watchr.toml` supports the following structure:

```toml
# Global debounce time in milliseconds (default: 500)
debounce_ms = 500

# Watcher entries (can define multiple)
[[watcher]]
name = "entry-name"           # Optional: descriptive name
dirs = ["src/", "tests/"]     # Required: directories to watch
ext = ["rs", "toml"]          # Optional: file extensions (omit to watch all)
command = "cargo test"        # Required: command to run on changes
```

### Config Resolution

`watchr` seraches for `.watchr.toml` by walking up the directory 
tree from the current working directory.

Override with explicit path:

```bash
watchr watch --config /path/to/.watch.toml
```

## CLI reference

### Commands

#### `watchr init`

Generate a `.watchr.toml` template in the current directory.

```bash
watchr init
```

Errors if `.watchr.toml` already exists.

### `watchr watch`

Starts watching for file chagnes.

**Flags:**  
- `--config <PATH>`    - Explicit path to config file

- `--dir <DIR>`        - Directory to watch (CLI mode, 
                         requires --cmd)

- `--cmd <COMMAND>`    - Command to run on changes (CLI mode, 
                         requires --dir)

- `--ext <EXTENSIONS>` - Comma-separated list of extensions to
                         filter (CLI mode)

**Examples:**  

```bash
# Use .watchr.toml from current directory or parent
watchr watch

# Use specific config file
watchr watch --config ~/my-project/.watchr.toml

# CLI mode: watch src/, run tests
watchr watch --dir src/ --cmd "cargo test"

# CLI mode: watch src/, filter .rs and .toml files
watchr watch --dir src/ --cmd "cargo test" --ext rs,toml
```

## Examples

### Run tests on Rust file changes

```toml
[[watcher]]
name = "tests"
dirs = ["src/", "tests/"]
ext = ["rs"]
command = "cargo test"
```

### Build on source changes

```toml
[[watcher]]
name = "build"
dirs = ["src/"]
ext = ["rs"]
command = "cargo build"
```

### Run linter and formatter

```toml
[[watcher]]
name = "lint"
dirs = ["src/"]
ext = ["rs"]
command = "cargo clippy && cargo fmt"
```

### Watch multiple directories with different commands

```toml
debounce_ms = 300

[[watcher]]
name = "backend"
dirs = ["backend/src/"]
ext = ["rs"]
command = "cargo test --manifest-path backend/Cargo.toml"

[[watcher]]
name = "frontend"
dirs = ["frontend/src/"]
ext = ["js", "jsx"]
command = "npm test --prefix frontend"
```

## How It Works

1. **File Watching**: Uses `notify-debouncer-full` for efficient
                      filesystem monitoring

2. **Deboucing**    : Groups rapid file changes within the 
                      debounce window

3. **Filtering**    : Check file extensions (if specified) before
                      running commands

4. **Execution**    : Runs commands via `sh -c "command"` to 
                      support shell syntax

5. **Shutdown**     : Catches `Ctrl+C` for graceful cleanup


## Error Handling

- **Directory not found** : Validates all directories exist before
                            starting

- **Config errors**       : Clear messages from TOML parsing 
                            failures

- **Command failures**    : Logs errors but continues watching

- **Signal handling**     : Graceful shutdown on `Ctrl+C`


## Contributing
See [CONTRIBUTING](CONTRIBUTING.md) for setup and contribution guidelines.

## License
MIT © Michael George

## Architecture Decisions

See [ADRs](docs/adr) for detailed architecture decision records: 
- [ADR-0001: File watching debounce strategy](docs/adr/0001-file-watching-debounce.md)
- [ADR-0002: TOML config format](docs/adr/0002-config-toml.md)
