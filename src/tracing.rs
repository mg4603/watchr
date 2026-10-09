//! Initializes the tracing subscriber with a configurable
//! verbosity level.
use tracing_subscriber::EnvFilter;

/// Initializes the global tracing subscriber.
///
/// Uses `RUST_LOG` if set; otherwise derives the log level from
/// `verbosity`.
///
/// # Panics
///
/// Panics if the global tracing subscriber has already been
/// installed.
pub fn init_tracing(verbosity: u8) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            EnvFilter::new(verbosity_to_level(verbosity))
        });
    tracing_subscriber::fmt().with_env_filter(filter).init();
}

// Maps a verbosity count to a tracing level string.
fn verbosity_to_level(verbosity: u8) -> &'static str {
    match verbosity {
        0 => "off",
        1 => "info",
        2 => "debug",
        _ => "trace",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verbosity_to_level() {
        assert_eq!(verbosity_to_level(0), "off");
        assert_eq!(verbosity_to_level(1), "info");
        assert_eq!(verbosity_to_level(2), "debug");
        assert_eq!(verbosity_to_level(3), "trace");
    }
}
