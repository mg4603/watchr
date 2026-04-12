use std::process;
use std::sync::mpsc;
use std::time::Duration;

use notify_debouncer_full::{DebounceEventResult, new_debouncer, notify, notify::RecursiveMode};
use thiserror::Error;

use crate::config::WatchrConfig;

#[derive(Error, Debug)]
pub enum WatcherError {
    #[error("notify-debouncer error: {0}")]
    Notify(#[from] notify::Error),
}

#[derive(Debug)]
pub enum WatchEvent {
    Command(String),
    Shutdown,
}

pub fn run_watch(config: WatchrConfig) -> Result<(), WatcherError> {
    let (tx, rx) = mpsc::channel();
    let mut debouncers = Vec::new();

    for entry in config.entries {
        let tx = tx.clone();

        let mut debouncer = new_debouncer(
            Duration::from_millis(config.debounce_ms),
            None,
            move |result: DebounceEventResult| match result {
                Ok(events) => {
                    for event in events {
                        for path in event.paths.clone() {
                            if !path.is_file() {
                                continue;
                            }
                            if entry.ext.is_none() {
                                let _ = tx.send(WatchEvent::Command(entry.command.clone()));
                                continue;
                            }

                            let ext = path.extension();
                            for ext_ in entry.ext.clone().unwrap() {
                                if ext_ == ext.unwrap().display().to_string() {
                                    let _ = tx.send(WatchEvent::Command(entry.command.clone()));
                                }
                            }
                        }
                    }
                }
                Err(errors) => {
                    for e in errors {
                        println!("{:?}", e);
                    }
                }
            },
        )?;

        for dir in entry.dirs {
            let dir = dir.clone();
            debouncer.watch(dir, RecursiveMode::Recursive)?;
        }
        debouncers.push(debouncer);
    }

    // drop initial sender after creating clones
    drop(tx);

    loop {
        match rx.recv() {
            Ok(WatchEvent::Command(cmd)) => {
                let output = process::Command::new("sh").arg("-c").arg(&cmd).output();
                match output {
                    Ok(out) => println!("{:?}", out),
                    Err(e) => println!("{:?}", e),
                }
            }
            Ok(WatchEvent::Shutdown) => {}
            Err(_) => break,
        }
    }
    Ok(())
}
