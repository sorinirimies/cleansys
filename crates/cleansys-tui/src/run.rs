//! Background clean worker.
//!
//! Like the GUI and the web UI, a clean runs on a worker thread that reports progress
//! over a channel, so a long delete never freezes the interface: the UI thread only
//! drains messages ([`crate::app::App::poll_run`]) and keeps drawing.

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

#[cfg(test)]
mod tests {
    use super::*;
    use cleansys_core::cleaner_fn;
    use std::time::Duration;

    /// The skip set is process-wide, so tests that spawn workers must not overlap.
    static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());
    fn serial() -> std::sync::MutexGuard<'static, ()> {
        SERIAL.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn drain(h: &RunHandle) -> Vec<RunMsg> {
        let mut out = Vec::new();
        loop {
            match h.rx.recv_timeout(Duration::from_secs(5)) {
                Ok(RunMsg::Done) => {
                    out.push(RunMsg::Done);
                    return out;
                }
                Ok(m) => out.push(m),
                Err(_) => panic!("worker did not finish"),
            }
        }
    }

    fn ok_job(cat: usize, item: usize, root: bool) -> Job {
        (
            cat,
            item,
            format!("job{item}"),
            cleaner_fn(|_| Ok(CleaningResult::new())),
            root,
        )
    }

    #[test]
    fn reports_started_finished_and_done_in_order() {
        let _guard = serial();
        let h = spawn(
            vec![ok_job(0, 0, false), ok_job(0, 1, false)],
            vec![],
            false,
        );
        let msgs = drain(&h);
        let kinds: Vec<&str> = msgs
            .iter()
            .map(|m| match m {
                RunMsg::Started(..) => "S",
                RunMsg::Finished(_, _, Ok(_)) => "F",
                RunMsg::Finished(_, _, Err(_)) => "E",
                RunMsg::Cancelled(..) => "C",
                RunMsg::Done => "D",
            })
            .collect();
        assert_eq!(kinds, ["S", "F", "S", "F", "D"]);
    }

    #[test]
    fn root_cleaner_without_sudo_fails_with_a_hint() {
        let _guard = serial();
        let h = spawn(vec![ok_job(0, 0, true)], vec![], false);
        let msgs = drain(&h);
        match &msgs[1] {
            RunMsg::Finished(0, 0, Err(e)) => assert!(e.contains("sudo"), "{e}"),
            _ => panic!("expected a sudo failure"),
        }
    }

    #[test]
    fn cleaner_errors_are_trimmed_to_the_last_segment() {
        let _guard = serial();
        let job: Job = (
            0,
            0,
            "bad".into(),
            cleaner_fn(|_| Err(anyhow::anyhow!("outer: inner: disk on fire"))),
            false,
        );
        let h = spawn(vec![job], vec![], false);
        match &drain(&h)[1] {
            RunMsg::Finished(_, _, Err(e)) => assert_eq!(e, "disk on fire"),
            _ => panic!("expected an error"),
        }
    }

    #[test]
    fn cancel_during_a_job_cancels_the_ones_after_it() {
        let _guard = serial();
        // Job 0 blocks until the test has requested cancellation, so job 1 must be skipped.
        let (release_tx, release_rx) = std::sync::mpsc::channel::<()>();
        let release_rx = std::sync::Mutex::new(release_rx);
        let (running_tx, running_rx) = std::sync::mpsc::channel::<()>();
        let running_tx = std::sync::Mutex::new(running_tx);
        let first: Job = (
            0,
            0,
            "slow".into(),
            cleaner_fn(move |_| {
                let _ = running_tx.lock().unwrap().send(());
                let _ = release_rx.lock().unwrap().recv();
                Ok(CleaningResult::new())
            }),
            false,
        );
        let h = spawn(vec![first, ok_job(0, 1, false)], vec![], false);
        running_rx.recv_timeout(Duration::from_secs(5)).unwrap(); // job 0 is executing
        h.cancel();
        release_tx.send(()).unwrap();
        let msgs = drain(&h);
        assert!(msgs
            .iter()
            .any(|m| matches!(m, RunMsg::Finished(0, 0, Ok(_)))));
        assert!(msgs.iter().any(|m| matches!(m, RunMsg::Cancelled(0, 1))));
        assert!(!msgs.iter().any(|m| matches!(m, RunMsg::Started(0, 1))));
    }

    #[test]
    fn unticked_paths_are_skipped_while_the_job_runs_and_cleared_after() {
        let _guard = serial();
        use std::sync::atomic::AtomicBool;
        let seen = Arc::new(AtomicBool::new(false));
        let probe = Arc::clone(&seen);
        let job: Job = (
            0,
            0,
            "probe".into(),
            cleaner_fn(move |opts| {
                assert!(opts.honor_skips, "runs must honour the skip set");
                probe.store(
                    cleansys_core::engine::skip::is_skipped(std::path::Path::new(
                        "/tui-run-test/keep/me",
                    )),
                    Ordering::SeqCst,
                );
                Ok(CleaningResult::new())
            }),
            false,
        );
        let h = spawn(vec![job], vec![PathBuf::from("/tui-run-test/keep")], false);
        drain(&h);
        assert!(
            seen.load(Ordering::SeqCst),
            "child of a skipped path is skipped"
        );
        assert!(!cleansys_core::engine::skip::is_skipped(
            std::path::Path::new("/tui-run-test/keep/me")
        ));
    }
}
