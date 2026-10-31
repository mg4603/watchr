//! Provides Ctrl+C/SIGINT and SIGTERM handlers that initiate
//! a graceful shutdown via the watcher's event channel.
use super::{WatchEvent, WatcherError};
use std::sync::mpsc::Sender;
/// Installs a handler that sends [`WatchEvent::Shutdown`] on
/// Ctrl+C.
///
/// # Errors
/// Returns [`WatcherError::SignalHandler`] if registration fails.
pub(super) fn create_shutdown_handler(
    tx: Sender<WatchEvent>,
) -> Result<(), WatcherError> {
    ctrlc::try_set_handler(move || {
        let _ = tx.send(WatchEvent::Shutdown);
    })?;
    Ok(())
}
