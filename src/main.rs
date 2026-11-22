//! Entry point and orchestration layer for `watchr`
//!
//! # Execution Flow
//! 1. Parse CLI arguments using CLI module
//! 2. Determine command (`is_init` function on cli::Commands)
//!    - `init` -> [`run_init`]
//!    - `watch` -> [`prepare_config`] -> [`run_watch`]
//! 3. Execute the selected command via `run()`
//! 4. Returns `Result<(), MainError>` to the caller
//!
//! All functionality is delegated to command-specific modules.
use clap::Parser;

use watchr::cli::Cli;
use watchr::run;
use watchr::tracing::init_tracing;

fn main() {
    let cli = Cli::parse();
    init_tracing(cli.verbose);
    let mut stdout = std::io::stdout();
    if let Err(e) = run(cli, &mut stdout) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
