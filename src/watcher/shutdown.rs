use super::{WatchEvent, WatcherError};
use std::sync::mpsc::Sender;
/// Installs a Ctrl+C handler that triggers a graceful shutdown.
///
/// When SIGINT or SIGTERM is received, [`WatchEvent::Shutdown`]
/// message is sent through the provided channel.
///
/// # Arguments
///
/// * `tx` - Channel sender used to propagate shutdown events
///
/// # Errors
/// Returns [`WatcherError::SignalHandler`] if the handler cannot
/// be registered.
///
/// # Examples
///
/// For internal reference only - `create_shutdown_handler` is not
/// part of the public API and this example cannot be compiled or
/// run externally
///
/// ```ignore
/// let (tx, _) = std::sync::mpsc::channel();
/// let ctrlc_handler = create_shutdown_handler(tx)?;
/// ```
pub(super) fn create_shutdown_handler(
    tx: Sender<WatchEvent>,
) -> Result<(), WatcherError> {
    ctrlc::try_set_handler(move || {
        let _ = tx.send(WatchEvent::Shutdown);
    })?;
    Ok(())
}
