//! Provides Ctrl+C/SIGINT and SIGTERM handlers that initiate
//! a graceful shutdown via the watcher's event channel.
use super::{WatchEvent, WatcherError};
use std::sync::mpsc::Sender;

pub trait ShutdownHandler {
    fn install(&self, tx: Sender<WatchEvent>)
    -> Result<(), WatcherError>;
}

pub struct CtrlCHandler;

impl ShutdownHandler for CtrlCHandler {
    /// Installs a handler that sends [`WatchEvent::Shutdown`] on
    /// SIGINT(Ctrl+C) / SIGTERM.
    ///
    /// # Errors
    /// Returns [`WatcherError::SignalHandler`] if registration fails.
    fn install(
        &self,
        tx: Sender<WatchEvent>,
    ) -> Result<(), WatcherError> {
        // Note: `try_set_handler` fails if a handler is already installed.
        ctrlc::try_set_handler(move || send_shutdown(&tx))?;
        Ok(())
    }
}

// Standalone so unit tests can trigger shutdown without real signals.
fn send_shutdown(tx: &Sender<WatchEvent>) {
    // Receiver may already be gone; nothing to notify, so ignore.
    let _ = tx.send(WatchEvent::Shutdown);
}

#[cfg(test)]
mod tests;
