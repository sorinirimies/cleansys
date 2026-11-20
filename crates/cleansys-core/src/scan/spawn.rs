//! Starting, running and stopping the background scan.

use std::collections::{HashSet, VecDeque};
use std::sync::mpsc::{Receiver, channel};
use std::sync::{Arc, Mutex};

use super::ScanInfo;
use crate::cleaners::cleaned_item::RunOptions;
use crate::model::CleanerCategory;

/// Start a background scan of every cleaner. Results arrive on the returned
/// channel as `(category, item, info)`; the second value is the job count.
pub fn spawn_scan(categories: &[CleanerCategory]) -> (Receiver<(usize, usize, ScanInfo)>, usize) {
    let (tx, rx) = channel();
    let total = spawn_scan_with(categories, move |ci, ii, info| {
        tx.send((ci, ii, info)).is_ok()
    });
    (rx, total)
}

/// Like [`spawn_scan`], but every result is handed to `sink(category, item, info)` the
/// moment its cleaner finishes (on a scan worker thread). Return `false` from the sink to
/// cancel the jobs that have not started yet. Returns the job count. This is the single
/// scan implementation every front-end (TUI, GUI, web) goes through.
pub fn spawn_scan_with<F>(categories: &[CleanerCategory], sink: F) -> usize
where
    F: Fn(usize, usize, ScanInfo) -> bool + Send + Sync + 'static,
{
    spawn_scan_timeout(categories, scan_timeout(), sink)
}

/// A running scan that can be stopped, e.g. when the user starts a clean.
#[derive(Debug, Clone)]
pub struct ScanHandle {
    /// Scan jobs started.
    pub total: usize,
    cancelled: Arc<std::sync::atomic::AtomicBool>,
}

impl ScanHandle {
    /// Jobs that have not started are skipped and results still in flight are dropped.
    pub fn cancel(&self) {
        self.cancelled
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }
}

/// [`spawn_scan_with`] that leaves out cleaners `include(category, item, cleaner)` rejects
/// (the user's opt-outs) and returns a [`ScanHandle`] to stop it.
pub fn spawn_scan_filtered<F, I>(categories: &[CleanerCategory], include: I, sink: F) -> ScanHandle
where
    F: Fn(usize, usize, ScanInfo) -> bool + Send + Sync + 'static,
    I: Fn(usize, usize, &crate::model::CleanerItem) -> bool,
{
    spawn_inner(categories, scan_timeout(), include, sink)
}

/// The `(category, item)` pairs of cleaners that scans leave out, per `disabled` ids.
pub fn disabled_positions(
    categories: &[CleanerCategory],
    disabled: &[String],
) -> HashSet<(usize, usize)> {
    categories
        .iter()
        .enumerate()
        .flat_map(|(ci, c)| {
            c.items
                .iter()
                .enumerate()
                .filter(|(_, i)| disabled.contains(&i.id))
                .map(move |(ii, _)| (ci, ii))
        })
        .collect()
}

pub(super) fn spawn_scan_timeout<F>(
    categories: &[CleanerCategory],
    timeout: std::time::Duration,
    sink: F,
) -> usize
where
    F: Fn(usize, usize, ScanInfo) -> bool + Send + Sync + 'static,
{
    spawn_inner(categories, timeout, |_, _, _| true, sink).total
}

fn spawn_inner<F, I>(
    categories: &[CleanerCategory],
    timeout: std::time::Duration,
    include: I,
    sink: F,
) -> ScanHandle
where
    F: Fn(usize, usize, ScanInfo) -> bool + Send + Sync + 'static,
    I: Fn(usize, usize, &crate::model::CleanerItem) -> bool,
{
    // Slowest cleaners (as measured by the previous scan) go first: they are the long
    // pole, so starting them early keeps every worker busy until the end.
    let history = load_scan_times();
    let mut list: Vec<_> = categories
        .iter()
        .enumerate()
        .flat_map(|(ci, c)| c.items.iter().enumerate().map(move |(ii, i)| (ci, ii, i)))
        .filter(|(ci, ii, i)| include(*ci, *ii, i))
        .map(|(ci, ii, i)| (ci, ii, i.id.clone(), i.function.clone()))
        .collect();
    list.sort_by_key(|(_, _, id, _)| std::cmp::Reverse(history.get(id).copied().unwrap_or(0)));
    let jobs: VecDeque<_> = list.into_iter().collect();
    let total = jobs.len();
    crate::utils::clear_size_cache();
    crate::engine::deep::clear_index();
    let times = Arc::new(Mutex::new(std::collections::HashMap::<String, u64>::new()));
    let remaining = Arc::new(std::sync::atomic::AtomicUsize::new(total));
    let cancelled = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let handle = ScanHandle {
        total,
        cancelled: Arc::clone(&cancelled),
    };
    let sink = Arc::new(sink);
    let timing = std::env::var_os("CLEANSYS_SCAN_TIMING").is_some();
    // One shared pool: cleaner jobs and the directory walks inside them use the same
    // workers (work-stealing), and each result is sent the moment its cleaner finishes.
    let pool = crate::utils::scan_pool();
    // One slot per job: when it started and whether its result was already delivered. A
    // watchdog reports a job that hangs (a stalled mount, a command waiting on a lock) as
    // failed so the front-end's spinner ends; the late real result is then dropped.
    let slots: Arc<Vec<ScanSlot>> = Arc::new(
        jobs.iter()
            .map(|_| ScanSlot {
                started: Mutex::new(None),
                reported: std::sync::atomic::AtomicBool::new(false),
            })
            .collect(),
    );
    let targets: Arc<Vec<(usize, usize)>> = Arc::new(jobs.iter().map(|j| (j.0, j.1)).collect());
    // Deliver one result exactly once, whichever of worker / watchdog gets there first.
    let report = {
        let slots = Arc::clone(&slots);
        let times = Arc::clone(&times);
        let remaining = Arc::clone(&remaining);
        let cancelled = Arc::clone(&cancelled);
        let sink = Arc::clone(&sink);
        Arc::new(
            move |n: usize, ci: usize, ii: usize, id: String, ms: u64, info: ScanInfo| {
                // Cancelled (a clean started, or the app closed): nobody wants this result.
                if cancelled.load(std::sync::atomic::Ordering::SeqCst) {
                    return;
                }
                if slots[n]
                    .reported
                    .swap(true, std::sync::atomic::Ordering::SeqCst)
                {
                    return;
                }
                times
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .insert(id, ms);
                if remaining.fetch_sub(1, std::sync::atomic::Ordering::SeqCst) == 1 {
                    save_scan_times(&times.lock().unwrap_or_else(|e| e.into_inner()));
                }
                if !sink(ci, ii, info) {
                    // receiver dropped (app closed): skip the rest
                    cancelled.store(true, std::sync::atomic::Ordering::Relaxed);
                }
            },
        )
    };
    let ids: Arc<Vec<String>> = Arc::new(jobs.iter().map(|j| j.2.clone()).collect());
    for (n, (ci, ii, id, f)) in jobs.into_iter().enumerate() {
        let cancelled = Arc::clone(&cancelled);
        let slots = Arc::clone(&slots);
        let report = Arc::clone(&report);
        pool.spawn(move || {
            if cancelled.load(std::sync::atomic::Ordering::Relaxed) {
                return;
            }
            let t0 = std::time::Instant::now();
            *slots[n].started.lock().unwrap_or_else(|e| e.into_inner()) = Some(t0);
            // A panicking cleaner must not take the whole pool (rayon aborts on escaped panics) down.
            let info = ScanInfo::from_result(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(RunOptions::preview())))
                    .unwrap_or_else(|_| Err(anyhow::anyhow!("cleaner panicked")))
                    .map_err(|e| e.to_string()),
            );
            let ms = t0.elapsed().as_millis() as u64;
            if timing {
                eprintln!("scan {ms:>7} ms {id} ({} bytes)", info.bytes);
            }
            report(n, ci, ii, id, ms, info);
        });
    }
    if total > 0 {
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_millis(500));
                if remaining.load(std::sync::atomic::Ordering::SeqCst) == 0
                    || cancelled.load(std::sync::atomic::Ordering::Relaxed)
                {
                    return;
                }
                for (n, slot) in slots.iter().enumerate() {
                    let started = *slot.started.lock().unwrap_or_else(|e| e.into_inner());
                    if let Some(t0) = started
                        && t0.elapsed() >= timeout
                        && !slot.reported.load(std::sync::atomic::Ordering::SeqCst)
                    {
                        let (ci, ii) = targets[n];
                        let info = ScanInfo::from_result(Err(format!(
                            "scan timed out after {}s",
                            timeout.as_secs()
                        )));
                        report(
                            n,
                            ci,
                            ii,
                            ids[n].clone(),
                            t0.elapsed().as_millis() as u64,
                            info,
                        );
                    }
                }
            }
        });
    }
    handle
}

struct ScanSlot {
    started: Mutex<Option<std::time::Instant>>,
    reported: std::sync::atomic::AtomicBool,
}

/// How long one cleaner may scan before it is reported as timed out
/// (`CLEANSYS_SCAN_TIMEOUT_SECS` overrides; default 5 minutes).
fn scan_timeout() -> std::time::Duration {
    let secs = std::env::var("CLEANSYS_SCAN_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .filter(|n| *n > 0)
        .unwrap_or(300);
    std::time::Duration::from_secs(secs)
}

fn scan_times_path() -> Option<std::path::PathBuf> {
    crate::settings::settings_dir()
        .ok()
        .map(|d| d.join("scan_times.json"))
}

pub(crate) fn load_scan_times() -> std::collections::HashMap<String, u64> {
    scan_times_path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub(crate) fn save_scan_times(times: &std::collections::HashMap<String, u64>) {
    if let (Some(p), Ok(json)) = (scan_times_path(), serde_json::to_string(times)) {
        let _ = std::fs::write(p, json);
    }
}
