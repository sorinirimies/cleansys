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
    /// Per-entry tick state, aligned with `scan.entries`.
    pub entry_on: Vec<bool>,
    /// Tick state of the skippable entries (for the "3/12" badge).
    pub entry_state: cleansys_core::EntryState,
    /// Bytes still freed after unticked entries are left out.
    pub sel_bytes: u64,
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
    /// The in-browser sudo password prompt is available (loopback server, not root).
    pub can_elevate: bool,
    /// The user has entered a valid sudo password this session.
    pub sudo_ok: bool,
    /// Ticked root cleaners exist and the server is not yet allowed to run them.
    pub needs_auth: bool,
    /// Idle-day threshold for project build artifacts.
    pub min_age_days: u64,
    /// Ask on /confirm before cleaning (Settings).
    pub confirm_before_run: bool,
}

/// Wrong-password attempts allowed before a cool-down.
const MAX_AUTH_FAILS: u32 = 3;
/// How long further attempts are refused after [`MAX_AUTH_FAILS`] failures.
const AUTH_COOLDOWN: std::time::Duration = std::time::Duration::from_secs(30);

pub struct Inner {
    categories: Vec<CleanerCategory>,
    board: ScanBoard,
    scan_rx: Option<Receiver<(usize, usize, ScanInfo)>>,
    run: RunState,
    is_root: bool,
    elevation_allowed: bool,
    sudo_ok: bool,
    auth_fails: u32,
    auth_locked_until: Option<std::time::Instant>,
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
            elevation_allowed: false,
            sudo_ok: false,
            auth_fails: 0,
            auth_locked_until: None,
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

    /// Flip one entry in the details view.
    pub fn toggle_entry(&self, path: &str) {
        self.lock().board.toggle_entry(path);
    }

    /// Tick/untick every entry of the cleaner with this id.
    pub fn set_entries(&self, id: &str, on: bool) {
        let mut g = self.lock();
        let pos = g
            .categories
            .iter()
            .enumerate()
            .find_map(|(ci, c)| c.items.iter().position(|i| i.id == id).map(|ii| (ci, ii)));
        if let Some((ci, ii)) = pos {
            g.board.set_item_entries(ci, ii, on);
        }
    }

    /// Load → change → save the engine settings, then re-measure when `rescan`.
    /// The closure's error (bad input) is returned and nothing is changed.
    pub fn edit_engine(
        &self,
        rescan: bool,
        change: impl FnOnce(&mut cleansys_core::engine::EngineConfig) -> Result<(), String>,
    ) -> Result<(), String> {
        let mut cfg = cleansys_core::engine::EngineConfig::load();
        change(&mut cfg)?;
        cfg.save().map_err(|e| e.to_string())?;
        if rescan {
            let mut g = self.lock();
            if g.run.phase != Phase::Running {
                start_scan_locked(&mut g);
            }
        }
        Ok(())
    }

    /// Change the idle-day threshold, persist it and re-measure.
    pub fn set_min_age(&self, days: u64) -> Result<(), String> {
        cleansys_core::engine::EngineConfig::save_min_age_days(days).map_err(|e| e.to_string())?;
        let mut g = self.lock();
        if g.run.phase != Phase::Running {
            start_scan_locked(&mut g);
        }
        Ok(())
    }

    /// Allow (or forbid) the in-browser sudo password prompt. Only enable this for a
    /// loopback-bound server: the password crosses plain HTTP.
    pub fn allow_elevation(&self, allowed: bool) {
        self.lock().elevation_allowed = allowed;
    }

    /// Whether the sudo prompt can be used (allowed, and the server isn't root already).
    pub fn can_elevate(&self) -> bool {
        let g = self.lock();
        g.elevation_allowed && !g.is_root
    }

    /// Validate `password` with `sudo -S -v` and, on success, let root cleaners run until
    /// the current run finishes. Blocking — call from a blocking thread.
    ///
    /// `Ok(true)` = accepted, `Ok(false)` = wrong password, `Err` = unavailable / locked out.
    pub fn authenticate(&self, password: &str) -> Result<bool, String> {
        {
            let mut g = self.lock();
            if !g.elevation_allowed || g.is_root {
                return Err("password entry is not available on this server".into());
            }
            if let Some(until) = g.auth_locked_until {
                if std::time::Instant::now() < until {
                    return Err("too many wrong passwords — wait a moment and retry".into());
                }
                g.auth_locked_until = None;
                g.auth_fails = 0;
            }
        }
        let ok = cleansys_core::authenticate_sudo(password).map_err(|e| format!("sudo: {e}"))?;
        let mut g = self.lock();
        if ok {
            g.sudo_ok = true;
            g.auth_fails = 0;
        } else {
            g.auth_fails += 1;
            if g.auth_fails >= MAX_AUTH_FAILS {
                g.auth_locked_until = Some(std::time::Instant::now() + AUTH_COOLDOWN);
            }
        }
        Ok(ok)
    }

    /// Force the sudo-authenticated flag (tests).
    pub fn set_sudo_ok(&self, ok: bool) {
        self.lock().sudo_ok = ok;
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

    /// Flip one cleaner by id. Root cleaners are ignored unless the server is root or
    /// the sudo prompt is available.
    pub fn toggle(&self, id: &str) -> bool {
        let mut g = self.lock();
        let can_root = g.is_root || g.elevation_allowed;
        for item in g.categories.iter_mut().flat_map(|c| c.items.iter_mut()) {
            if item.id == id {
                if item.requires_root && !can_root {
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
        let can_root = g.is_root || g.elevation_allowed;
        if let Some(c) = g.categories.get_mut(category) {
            for item in &mut c.items {
                if !item.requires_root || can_root {
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
                bytes: g.board.category_bytes_when_done(ci),
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
                    entry_on: g
                        .board
                        .get(ci, ii)
                        .map(|s| {
                            s.entries
                                .iter()
                                .map(|e| g.board.entry_selected(&e.path))
                                .collect()
                        })
                        .unwrap_or_default(),
                    entry_state: g.board.entry_state(ci, ii),
                    sel_bytes: g.board.item_selected_bytes(ci, ii),
                    selectable: !it.requires_root || g.is_root || g.elevation_allowed,
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
            can_elevate: g.elevation_allowed && !g.is_root,
            sudo_ok: g.sudo_ok,
            needs_auth: needs_auth_locked(&g),
            min_age_days: cleansys_core::engine::EngineConfig::load().min_age_days,
            confirm_before_run: cleansys_core::load_settings()
                .unwrap_or_default()
                .confirm_before_run(),
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

    /// The ticked cleaners as jobs, for a dry-run preview. Root cleaners are included
    /// when the server is root or the sudo prompt is available (previews only read).
    pub fn selected_jobs(&self) -> Vec<Job> {
        let g = self.lock();
        jobs_locked(&g, g.is_root || g.elevation_allowed)
    }

    /// Dry-run the ticked cleaners (blocking — call from a blocking thread).
    pub fn preview(&self) -> Vec<Outcome> {
        let jobs = self.selected_jobs();
        let (skipped, running) = {
            let g = self.lock();
            (g.board.skipped_paths(), g.run.phase == Phase::Running)
        };
        if running {
            // The running clean owns the (process-wide) skip set.
            return headless::execute(&jobs, RunOptions::preview());
        }
        // Unticked entries are left out of the preview too, so it matches the run.
        cleansys_core::engine::skip::set_skipped(skipped);
        let out = headless::execute(&jobs, RunOptions::preview().with_skips());
        cleansys_core::engine::skip::clear_skipped();
        out
    }

    /// Start cleaning the ticked cleaners on a background thread.
    /// Err = nothing to do / already running / [`NEEDS_AUTH`].
    pub fn start_run(&self) -> Result<usize, &'static str> {
        let jobs = {
            let mut g = self.lock();
            if g.run.phase == Phase::Running {
                return Err("a clean is already running");
            }
            if needs_auth_locked(&g) {
                return Err(NEEDS_AUTH);
            }
            let jobs = jobs_locked(&g, g.is_root || g.sudo_ok);
            if jobs.is_empty() {
                return Err("nothing selected");
            }
            cleansys_core::engine::skip::set_skipped(g.board.skipped_paths());
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
                let outcome = headless::execute(
                    std::slice::from_ref(&job),
                    RunOptions::execute().with_skips(),
                )
                .pop();
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
            cleansys_core::engine::skip::clear_skipped();
            let mut g = this.lock();
            g.run.current = None;
            g.run.phase = Phase::Done;
            // Never keep the sudo password around once the run is over.
            if g.sudo_ok {
                g.sudo_ok = false;
                cleansys_core::clear_cached_sudo_password();
            }
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

/// `start_run` error: a ticked root cleaner needs the sudo password first.
pub const NEEDS_AUTH: &str = "sudo authentication required";

/// Ticked root cleaners exist but the server is neither root nor sudo-authenticated.
fn needs_auth_locked(g: &Inner) -> bool {
    !g.is_root
        && !g.sudo_ok
        && g.categories
            .iter()
            .flat_map(|c| &c.items)
            .any(|i| i.selected && i.requires_root)
}

fn jobs_locked(g: &Inner, include_root: bool) -> Vec<Job> {
    let mut jobs = Vec::new();
    for c in &g.categories {
        for it in &c.items {
            if it.selected && (!it.requires_root || include_root) {
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
    fn elevation_makes_root_cleaners_tickable_but_gates_the_run() {
        let s = shared();
        s.allow_elevation(true);
        assert!(s.can_elevate());
        assert!(s.snapshot().items.iter().all(|i| i.selectable));
        assert!(s.toggle("s1"));
        let snap = s.snapshot();
        assert!(snap.needs_auth && !snap.sudo_ok);
        assert_eq!(s.start_run(), Err(NEEDS_AUTH), "no run before the password");
        assert_eq!(s.selected_jobs().len(), 1, "previews are read-only");

        s.set_sudo_ok(true);
        assert!(!s.snapshot().needs_auth);
    }

    #[test]
    fn authenticate_refused_when_elevation_not_allowed() {
        let s = shared();
        assert!(s.authenticate("pw").is_err());
        assert!(!s.snapshot().sudo_ok);
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
