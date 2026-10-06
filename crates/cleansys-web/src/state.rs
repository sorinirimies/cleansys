//! Shared application state: categories, background scan results, selection and
//! the clean run in progress.
//!
//! Everything here is plain Rust (no web framework), so it is unit-tested directly.
//! The web layer only takes cheap [`Snapshot`]s of it while rendering.

use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex, MutexGuard};

use cleansys_core::engine::headless::{self, Job, Outcome, OutcomeStatus};
use cleansys_core::{
    CleanerCategory, Risk, RunOptions, ScanBoard, ScanInfo, check_root, spawn_scan,
};

/// Where a clean run is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Phase {
    #[default]
    Idle,
    Running,
    Done,
}

/// One finished (or skipped/failed) cleaner of a run.
#[derive(Debug, Clone)]
pub struct RunLine {
    pub name: String,
    pub status: LineStatus,
    pub bytes: u64,
    pub items: usize,
    pub detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineStatus {
    Ok,
    Skipped,
    Failed,
}

/// Progress of the current / last clean run.
#[derive(Debug, Clone, Default)]
pub struct RunState {
    pub phase: Phase,
    pub total: usize,
    pub done: usize,
    pub current: Option<String>,
    pub lines: Vec<RunLine>,
    pub freed: u64,
}

/// One cleaner as the page sees it.
#[derive(Debug, Clone)]
pub struct ItemView {
    pub category: usize,
    /// Index within its category.
    pub idx: usize,
    pub id: String,
    pub name: String,
    pub description: String,
    pub risk: Risk,
    pub requires_root: bool,
    pub selected: bool,
    pub scan: Option<ScanInfo>,
    /// Can this cleaner be ticked here (root cleaners need a root server)?
    pub selectable: bool,
}

/// One category as the page sees it.
#[derive(Debug, Clone)]
pub struct CatView {
    pub index: usize,
    pub name: String,
    pub description: String,
    pub root: bool,
    pub bytes: Option<u64>,
    pub ticked: usize,
    pub visible: bool,
}

/// An immutable copy of everything a page render needs.
#[derive(Debug, Clone)]
pub struct Snapshot {
    pub categories: Vec<CatView>,
    pub items: Vec<ItemView>,
    pub scanning: bool,
    pub scan_done: usize,
    pub scan_total: usize,
    pub complete: bool,
    pub total_bytes: u64,
    pub selected_count: usize,
    pub selected_bytes: u64,
    pub run: RunState,
    pub is_root: bool,
}

pub struct Inner {
    categories: Vec<CleanerCategory>,
    board: ScanBoard,
    scan_rx: Option<Receiver<(usize, usize, ScanInfo)>>,
    run: RunState,
    is_root: bool,
}

/// Cheaply clonable handle to the shared state.
#[derive(Clone)]
pub struct Shared(Arc<Mutex<Inner>>);

/// The root side of the UI: the built-in "System Cleaners" and every root engine category.
pub fn is_root_category(c: &CleanerCategory) -> bool {
    c.name == "System Cleaners" || c.name.ends_with(cleansys_core::model::ROOT_SUFFIX)
}

impl Shared {
    /// Build from an explicit category list (tests) — no scan is started.
    pub fn new(categories: Vec<CleanerCategory>) -> Self {
        let board = ScanBoard::new(&categories);
        Shared(Arc::new(Mutex::new(Inner {
            categories,
            board,
            scan_rx: None,
            run: RunState::default(),
            is_root: check_root(),
        })))
    }

    /// Load the real cleaners (restoring nothing: the web UI starts unticked).
    pub fn load() -> Self {
        Self::new(cleansys_core::load_categories())
    }

    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.0.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Force the root flag (tests).
    pub fn set_root(&self, root: bool) {
        self.lock().is_root = root;
    }

    // ── scanning ───────────────────────────────────────────────────

    /// Start measuring every cleaner in the background (no-op while scanning/running).
    pub fn start_scan(&self) {
        let mut g = self.lock();
        start_scan_locked(&mut g);
    }

    /// Drain finished scan results. Call on every request.
    pub fn poll(&self) {
        let mut g = self.lock();
        let Inner { board, scan_rx, .. } = &mut *g;
        let mut finished = false;
        if let Some(rx) = scan_rx.as_ref() {
            while let Ok((c, i, info)) = rx.try_recv() {
                finished |= board.record(c, i, info);
            }
        }
        if finished {
            *scan_rx = None;
        }
    }

    // ── selection ──────────────────────────────────────────────────

    /// Flip one cleaner by id. Root cleaners are ignored unless the server is root.
    pub fn toggle(&self, id: &str) -> bool {
        let mut g = self.lock();
        let is_root = g.is_root;
        for item in g.categories.iter_mut().flat_map(|c| c.items.iter_mut()) {
            if item.id == id {
                if item.requires_root && !is_root {
                    return false;
                }
                item.selected = !item.selected;
                return true;
            }
        }
        false
    }

    /// The "recommended" preset (skips cleaners found empty).
    pub fn select_recommended(&self) -> usize {
        let mut g = self.lock();
        let Inner {
            categories,
            board,
            is_root,
            ..
        } = &mut *g;
        board.select_recommended(categories, *is_root)
    }

    /// Untick everything.
    pub fn select_none(&self) {
        let mut g = self.lock();
        for item in g.categories.iter_mut().flat_map(|c| c.items.iter_mut()) {
            item.selected = false;
        }
    }

    /// Tick (or untick) every selectable cleaner in one category.
    pub fn set_category(&self, category: usize, on: bool) {
        let mut g = self.lock();
        let is_root = g.is_root;
        if let Some(c) = g.categories.get_mut(category) {
            for item in &mut c.items {
                if !item.requires_root || is_root {
                    item.selected = on;
                }
            }
        }
    }

    // ── views ──────────────────────────────────────────────────────

    pub fn snapshot(&self) -> Snapshot {
        let g = self.lock();
        let mut categories = Vec::new();
        let mut items = Vec::new();
        for (ci, c) in g.categories.iter().enumerate() {
            categories.push(CatView {
                index: ci,
                name: c.name.clone(),
                description: c.description.clone(),
                root: is_root_category(c),
                bytes: g.board.category_bytes(ci),
                ticked: c.items.iter().filter(|i| i.selected).count(),
                visible: true,
            });
            for (ii, it) in c.items.iter().enumerate() {
                items.push(ItemView {
                    category: ci,
                    idx: ii,
                    id: it.id.clone(),
                    name: it.name.clone(),
                    description: it.description.clone(),
                    risk: it.risk,
                    requires_root: it.requires_root,
                    selected: it.selected,
                    scan: g.board.get(ci, ii).cloned(),
                    selectable: !it.requires_root || g.is_root,
                });
            }
        }
        Snapshot {
            categories,
            items,
            scanning: g.board.is_scanning(),
            scan_done: g.board.total.saturating_sub(g.board.pending),
            scan_total: g.board.total,
            complete: g.board.complete(),
            total_bytes: g.board.total_bytes(),
            selected_count: g
                .categories
                .iter()
                .flat_map(|c| &c.items)
                .filter(|i| i.selected)
                .count(),
            selected_bytes: g.board.selected_bytes(&g.categories),
            run: g.run.clone(),
            is_root: g.is_root,
        }
    }

    /// `(category, item)` pairs to list for a category / filter / hide-empty setting.
    pub fn visible(&self, active: usize, filter: &str, hide_empty: bool) -> Vec<(usize, usize)> {
        let g = self.lock();
        g.board
            .visible_items(&g.categories, active, filter, hide_empty)
    }

    /// Is a category worth showing in the navigation?
    pub fn category_visible(&self, cat: usize, hide_empty: bool) -> bool {
        let g = self.lock();
        g.board.category_visible(&g.categories, cat, hide_empty)
    }

    // ── preview / run ──────────────────────────────────────────────

    /// The ticked cleaners as jobs (root ones only when the server is root).
    pub fn selected_jobs(&self) -> Vec<Job> {
        jobs_locked(&self.lock())
    }

    /// Dry-run the ticked cleaners (blocking — call from a blocking thread).
    pub fn preview(&self) -> Vec<Outcome> {
        let jobs = self.selected_jobs();
        headless::execute(&jobs, RunOptions::preview())
    }

    /// Start cleaning the ticked cleaners on a background thread.
    /// Err = nothing to do / already running.
    pub fn start_run(&self) -> Result<usize, &'static str> {
        let jobs = {
            let mut g = self.lock();
            if g.run.phase == Phase::Running {
                return Err("a clean is already running");
            }
            let jobs = jobs_locked(&g);
            if jobs.is_empty() {
                return Err("nothing selected");
            }
            g.run = RunState {
                phase: Phase::Running,
                total: jobs.len(),
                ..RunState::default()
            };
            jobs
        };
        let n = jobs.len();
        let this = self.clone();
        std::thread::spawn(move || {
            for job in jobs {
                this.lock().run.current = Some(job.name.clone());
                let outcome =
                    headless::execute(std::slice::from_ref(&job), RunOptions::execute()).pop();
                let mut g = this.lock();
                if let Some(o) = outcome {
                    let (status, detail) = match &o.status {
                        OutcomeStatus::Ok => (LineStatus::Ok, String::new()),
                        OutcomeStatus::Skipped(w) => (LineStatus::Skipped, w.clone()),
                        OutcomeStatus::Failed(w) => (LineStatus::Failed, w.clone()),
                    };
                    g.run.freed += o.bytes;
                    g.run.lines.push(RunLine {
                        name: o.name,
                        status,
                        bytes: o.bytes,
                        items: o.items.len(),
                        detail,
                    });
                }
                g.run.done += 1;
            }
            let mut g = this.lock();
            g.run.current = None;
            g.run.phase = Phase::Done;
            // Untick what was cleaned and re-measure so the lists are current.
            for item in g.categories.iter_mut().flat_map(|c| c.items.iter_mut()) {
                item.selected = false;
            }
            start_scan_locked(&mut g);
        });
        Ok(n)
    }

    /// Dismiss a finished run's summary.
    pub fn clear_run(&self) {
        let mut g = self.lock();
        if g.run.phase == Phase::Done {
            g.run = RunState::default();
        }
    }
}

fn jobs_locked(g: &Inner) -> Vec<Job> {
    let mut jobs = Vec::new();
    for c in &g.categories {
        for it in &c.items {
            if it.selected && (!it.requires_root || g.is_root) {
                jobs.push(Job {
                    id: it.id.clone(),
                    name: it.name.clone(),
                    category: c.name.clone(),
                    risk: it.risk,
                    requires_root: it.requires_root,
                    function: it.function.clone(),
                });
            }
        }
    }
    jobs
}

fn start_scan_locked(g: &mut Inner) {
    if g.board.is_scanning() {
        return;
    }
    g.board = ScanBoard::new(&g.categories);
    let (rx, total) = spawn_scan(&g.categories);
    g.board.start(total);
    g.scan_rx = Some(rx);
}

#[cfg(test)]
mod tests {
    use super::*;
    use cleansys_core::{CleanedItem, CleanerItem, CleaningResult, cleaner_fn};
    use std::path::PathBuf;

    fn item(id: &str, bytes: u64, root: bool, risk: Risk) -> CleanerItem {
        CleanerItem {
            id: id.into(),
            risk,
            name: format!("Name {id}"),
            description: "desc".into(),
            requires_root: root,
            selected: false,
            function: cleaner_fn(move |opts| {
                let mut r = CleaningResult::new();
                if bytes > 0 {
                    r.add_item(CleanedItem::file(
                        PathBuf::from(format!("/x/{id_len}", id_len = bytes)),
                        bytes,
                        "t",
                    ));
                }
                let _ = opts;
                Ok(r)
            }),
            bytes_cleaned: 0,
            last_result: None,
            status: None,
        }
    }

    fn shared() -> Shared {
        let s = Shared::new(vec![
            CleanerCategory {
                name: "User Land Cleaners".into(),
                description: "u".into(),
                items: vec![
                    item("u1", 100, false, Risk::Safe),
                    item("u2", 0, false, Risk::Safe),
                    item("u3", 50, false, Risk::Caution),
                ],
            },
            CleanerCategory {
                name: "System Cleaners".into(),
                description: "s".into(),
                items: vec![item("s1", 10, true, Risk::Safe)],
            },
        ]);
        s.set_root(false);
        s
    }

    fn wait_scan(s: &Shared) {
        for _ in 0..200 {
            s.poll();
            if s.snapshot().complete {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        panic!("scan did not finish");
    }

    #[test]
    fn scan_fills_sizes_and_recommended_skips_empty_and_root() {
        let s = shared();
        s.start_scan();
        wait_scan(&s);
        let snap = s.snapshot();
        assert_eq!(snap.total_bytes, 160);
        assert!(snap.categories[1].root && !snap.categories[0].root);
        assert_eq!(s.select_recommended(), 1); // u1 only: u2 empty, u3 caution, s1 root
        assert_eq!(s.snapshot().selected_count, 1);
    }

    #[test]
    fn root_cleaners_cannot_be_ticked_without_root() {
        let s = shared();
        assert!(!s.toggle("s1"));
        s.set_root(true);
        assert!(s.toggle("s1"));
        assert_eq!(s.snapshot().selected_count, 1);
        s.select_none();
        assert_eq!(s.snapshot().selected_count, 0);
    }

    #[test]
    fn category_selection_skips_root_items() {
        let s = shared();
        s.set_category(0, true);
        assert_eq!(s.snapshot().selected_count, 3);
        s.set_category(1, true);
        assert_eq!(s.snapshot().selected_count, 3, "root item stays unticked");
    }

    #[test]
    fn preview_reports_without_deleting_and_run_completes() {
        let s = shared();
        assert!(s.toggle("u1"));
        let out = s.preview();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].bytes, 100);

        assert_eq!(s.start_run(), Ok(1));
        for _ in 0..300 {
            if s.snapshot().run.phase == Phase::Done {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let snap = s.snapshot();
        assert_eq!(snap.run.phase, Phase::Done);
        assert_eq!(snap.run.lines.len(), 1);
        assert_eq!(snap.selected_count, 0, "ticks are cleared after a run");
        s.clear_run();
        assert_eq!(s.snapshot().run.phase, Phase::Idle);
    }

    #[test]
    fn nothing_selected_and_double_start_are_rejected() {
        let s = shared();
        assert_eq!(s.start_run(), Err("nothing selected"));
    }

    #[test]
    fn filter_searches_across_categories() {
        let s = shared();
        assert_eq!(s.visible(0, "s1", true), vec![(1, 0)]);
        assert_eq!(s.visible(0, "", true).len(), 3);
    }
}
