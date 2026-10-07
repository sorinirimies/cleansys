//! Background clean worker.
//!
//! The TUI used to delete things on the UI thread (paced by a timer), which froze
//! the interface during a long delete. Like the GUI and the web UI, runs now happen
//! on a worker thread that reports progress over a channel; the UI thread only drains
//! messages ([`crate::app::App::poll_run`]) and keeps drawing.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;

use cleansys_core::{CleanerFn, CleaningResult, RunOptions};

/// One cleaner to run: `(category, item, name, function, requires_root)`.
pub type Job = (usize, usize, String, CleanerFn, bool);

/// Progress reported by the worker.
pub enum RunMsg {
    /// A cleaner started.
    Started(usize, usize),
    /// A cleaner finished (`Err` = message to show).
    Finished(usize, usize, Result<CleaningResult, String>),
    /// A cleaner was skipped because the run was cancelled.
    Cancelled(usize, usize),
    /// Every job was handled; the worker is exiting.
    Done,
}

/// Handle to a running clean.
pub struct RunHandle {
    pub rx: Receiver<RunMsg>,
    cancel: Arc<AtomicBool>,
}

impl RunHandle {
    /// Ask the worker to stop after the cleaner it is currently running.
    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::SeqCst);
    }
}

/// Run `jobs` one after another on a new thread. `skipped` are the paths the user
/// unticked in the details view; they are left alone.
pub fn spawn(jobs: Vec<Job>, skipped: Vec<PathBuf>, is_root: bool) -> RunHandle {
    let (tx, rx) = channel();
    let cancel = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&cancel);
    std::thread::spawn(move || {
        cleansys_core::engine::skip::set_skipped(skipped);
        for (cat, item, _name, function, requires_root) in jobs {
            if flag.load(Ordering::SeqCst) {
                let _ = tx.send(RunMsg::Cancelled(cat, item));
                continue;
            }
            let _ = tx.send(RunMsg::Started(cat, item));
            let result =
                if requires_root && !is_root && cleansys_core::cached_sudo_password().is_none() {
                    Err("Requires sudo - restart with 'sudo cleansys'".to_string())
                } else {
                    function(RunOptions::execute().with_skips()).map_err(|e| {
                        e.to_string()
                            .split(':')
                            .next_back()
                            .unwrap_or("Unknown error")
                            .trim()
                            .to_string()
                    })
                };
            if tx.send(RunMsg::Finished(cat, item, result)).is_err() {
                break; // UI gone
            }
        }
        cleansys_core::engine::skip::clear_skipped();
        let _ = tx.send(RunMsg::Done);
    });
    RunHandle { rx, cancel }
}
