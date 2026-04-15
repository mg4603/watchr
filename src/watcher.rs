use std::path::PathBuf;
use std::process;
use std::sync::mpsc::{
    Receiver, Sender, channel as mpsc_channel,
};
use std::time::Duration;

use notify_debouncer_full::notify::{
    RecommendedWatcher, RecursiveMode,
};
use notify_debouncer_full::{
    DebounceEventResult, Debouncer, NoCache, new_debouncer,
    notify,
};
use thiserror::Error;

use crate::config::WatchrConfig;
use crate::entry::WatchrEntry;

#[derive(Error, Debug)]
pub enum WatcherError {
    #[error("notify-debouncer error: {0}")]
    Notify(#[from] notify::Error),

    #[error("failed to create shutdown handler: {0}")]
    SignalHandler(#[from] ctrlc::Error),
}

#[derive(Debug)]
pub enum WatchEvent {
    Command(String),
    Shutdown,
}

pub fn run_watch(
    config: WatchrConfig,
) -> Result<(), WatcherError> {
    let (tx, rx) = mpsc_channel();

    create_shutdown_handler(tx.clone())?;

    let _debouncers = create_debouncers(
        config.debounce_ms,
        config.entries,
        tx.clone(),
    )?;

    // drop initial sender after creating clones
    drop(tx);

    run_event_loop(rx);
    Ok(())
}

fn create_shutdown_handler(
    tx: Sender<WatchEvent>,
) -> Result<(), WatcherError> {
    ctrlc::try_set_handler(move || {
        let _ = tx.send(WatchEvent::Shutdown);
    })?;
    Ok(())
}

fn handle_events(
    result: DebounceEventResult,
    exts: Option<Vec<String>>,
    command: String,
    tx: Sender<WatchEvent>,
) {
    match result {
        Ok(events) => {
            if exts.as_ref().is_none() {
                let _ = tx
                    .send(WatchEvent::Command(command.clone()));
                return;
            }

            let paths: Vec<PathBuf> = events
                .into_iter()
                .flat_map(|event| event.paths.clone())
                .collect();

            for path in paths {
                if !path.is_file() {
                    continue;
                }

                if let (Some(ext), Some(exts)) = (
                    path.extension().and_then(|e| e.to_str()),
                    exts.as_deref(),
                ) && exts.iter().any(|e| e == ext)
                {
                    let _ = tx.send(WatchEvent::Command(
                        command.clone(),
                    ));
                    return;
                }
            }
        }
        Err(errors) => {
            for e in errors {
                println!("{:?}", e);
            }
        }
    }
}

fn create_debouncers(
    debounce_ms: u64,
    entries: Vec<WatchrEntry>,
    tx: Sender<WatchEvent>,
) -> Result<
    Vec<Debouncer<RecommendedWatcher, NoCache>>,
    WatcherError,
> {
    let mut debouncers = Vec::new();
    for entry in entries {
        let tx = tx.clone();

        let mut debouncer = new_debouncer(
            Duration::from_millis(debounce_ms),
            None,
            move |result: DebounceEventResult| {
                handle_events(
                    result,
                    entry.ext.clone(),
                    entry.command.clone(),
                    tx.clone(),
                );
            },
        )?;

        for dir in &entry.dirs {
            debouncer.watch(dir, RecursiveMode::Recursive)?;
        }
        debouncers.push(debouncer);
    }
    Ok(debouncers)
}

fn run_event_loop(rx: Receiver<WatchEvent>) {
    loop {
        match rx.recv() {
            Ok(WatchEvent::Command(cmd)) => {
                let output = process::Command::new("sh")
                    .arg("-c")
                    .arg(&cmd)
                    .output();

                match output {
                    Ok(out) => println!("{:?}", out),
                    Err(e) => println!("{:?}", e),
                }
            }
            Ok(WatchEvent::Shutdown) => {
                println!("Shutting down gracefully...");
                break;
            }
            Err(_) => break,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::time::Instant;

    use notify_debouncer_full::DebouncedEvent;
    use notify_debouncer_full::notify::ErrorKind;
    use notify_debouncer_full::notify::event::{
        Event, EventKind, ModifyKind,
    };

    fn create_debounced_event_result(
        error: bool,
    ) -> DebounceEventResult {
        if error {
            return Err(vec![notify::Error {
                kind: ErrorKind::Generic("custom".to_string()),
                paths: vec![PathBuf::from("./")],
            }]);
        }

        Ok(vec![DebouncedEvent {
            event: Event {
                kind: EventKind::Modify(ModifyKind::Any),
                paths: vec![PathBuf::from("src/main.rs")],
                attrs: Default::default(),
            },
            time: Instant::now(),
        }])
    }

    #[test]
    fn test_handle_events_no_ext() {
        let result = create_debounced_event_result(false);
        let (tx, rx) = mpsc_channel();
        handle_events(result, None, "pwd".to_string(), tx);

        assert!(matches!(
            rx.try_recv(),
            Ok(WatchEvent::Command(_))
        ));
    }

    #[test]
    fn test_handle_event_matching_ext() {
        let result = create_debounced_event_result(false);
        let (tx, rx) = mpsc_channel();
        handle_events(
            result,
            Some(vec!["rs".to_string()]),
            "pwd".to_string(),
            tx,
        );

        assert!(matches!(
            rx.try_recv(),
            Ok(WatchEvent::Command(_))
        ));
    }

    #[test]
    fn test_handle_event_no_matching_ext() {
        let result = create_debounced_event_result(false);
        let (tx, rx) = mpsc_channel();
        handle_events(
            result,
            Some(vec!["txt".to_string()]),
            "pwd".to_string(),
            tx,
        );

        // mpsc::TryRecvError::Empty
        assert!(matches!(rx.try_recv(), Err(..)));
    }

    #[test]
    fn test_hand_event_error_result() {
        let result = create_debounced_event_result(true);
        let (tx, rx) = mpsc_channel();
        handle_events(result, None, "pwd".to_string(), tx);

        // mpsc::TryRecvError::Empty
        assert!(matches!(rx.try_recv(), Err(..)))
    }
}
