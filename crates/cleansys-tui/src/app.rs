use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal;
use ratatui::widgets::ListState;
use std::time::Instant;

use crate::components::password_prompt::PasswordPrompt;
use cleansys_core::{check_root, format_size, CleanerCategory, CleanerFn, Status};

/// One editable row of the schedule overlay.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScheduleField {
    Frequency,
    Hour,
    Minute,
    /// Weekday (weekly) or day of month (monthly).
    Day,
    Scope,
    /// Linux only: systemd timer vs crontab.
    Backend,
}

/// Outcome of the last finished clean, shown under the cleaner list (like the GUI's
/// "✓ Freed X" headline and the web UI's "Done" summary).
#[derive(Debug, Clone, Default)]
pub struct RunSummary {
    /// Bytes freed.
    pub freed: u64,
    /// Cleaners that ran (including ones that found nothing).
    pub ok: usize,
    /// Cleaners that failed or were cancelled.
    pub failed: usize,
    /// Wall-clock time of the run.
    pub elapsed: String,
    /// The run was cancelled before finishing.
    pub cancelled: bool,
}

/// Cap on activity-log lines kept in memory.
const MAX_LOG_LINES: usize = 500;

/// Type alias for pending operations: (category_index, item_index, name, function, requires_root)
pub type PendingOperation = (usize, usize, String, CleanerFn, bool);

// `Status`, `CleanerItem`, and `CleanerCategory` now live in `cleansys-core`
// so the TUI and GUI front-ends share the exact same domain model.

pub struct App {
    pub categories: Vec<CleanerCategory>,
    pub category_index: usize,
    pub item_list_state: ListState,
    /// Render-side list state (rows of an expanded cleaner are interleaved, so its
    /// selection index differs from `item_list_state`); keeps the scroll offset.
    pub list_view_state: ListState,
    /// Cleaner whose per-path details are expanded: `(category, item)`.
    pub expanded: Option<(usize, usize)>,
    /// Highlighted entry inside the expanded details (`Some` = details have focus).
    pub entry_cursor: Option<usize>,
    /// Idle-day threshold for project build output (shown/adjusted with `[` `]`).
    pub min_age_days: u64,
    pub is_root: bool,
    pub is_running: bool,
    pub operation_start_time: Option<Instant>,
    pub operation_end_time: Option<Instant>,
    /// Bytes freed so far by the current / last run.
    pub total_bytes_cleaned: u64,
    pub show_help: bool,
    pub animation_frame: usize,
    pub last_frame_time: Instant,
    pub terminal_width: u16,
    pub terminal_height: u16,
    /// Worker running the current clean (None when idle).
    pub run: Option<crate::run::RunHandle>,
    /// Cleaners in the current / last run, and how many have finished.
    pub run_total: usize,
    pub run_done: usize,
    /// Name of the cleaner being run right now.
    pub run_current: Option<String>,
    /// Summary of the last finished run (shown under the list until dismissed).
    pub run_summary: Option<RunSummary>,
    /// Whether the activity log panel is open (`L`).
    pub show_log: bool,
    /// Activity log lines (most recent last, capped).
    pub operation_logs: Vec<String>,
    /// One-line notice shown under the list (cleared on the next key).
    pub notice: Option<String>,
    /// Ask for confirmation before running (`y` toggles).
    pub confirmation_mode: bool,
    pub password_prompt: PasswordPrompt,
    pub needs_sudo: bool,
    pub pending_operations: Vec<PendingOperation>,
    /// Whether the "confirm this run" overlay is currently shown, awaiting
    /// a yes/no answer before `pending_operations` actually executes.
    pub awaiting_run_confirmation: bool,
    /// Whether the preview (dry-run) results overlay is currently shown.
    pub preview_open: bool,
    /// Results of the most recent preview run: `(cleaner_name, result)`.
    pub preview_results: Vec<(String, cleansys_core::CleaningResult)>,
    /// Whether the "needs Administrator" notice is shown (Windows only;
    /// there is no interactive sudo-password flow there).
    pub needs_admin_notice: bool,
    /// Selected cleaners staged while `awaiting_run_confirmation` is true.
    pub pending_run_selection: Vec<PendingOperation>,
    /// Background scan results (sizes per cleaner/category).
    pub board: cleansys_core::ScanBoard,
    /// Channel delivering background scan results (drained by [`App::poll_scan`]).
    pub scan_rx: Option<std::sync::mpsc::Receiver<(usize, usize, cleansys_core::ScanInfo)>>,
    /// Hide cleaners that were scanned and have nothing to clean.
    pub hide_empty: bool,
    /// Live filter text for the cleaner list ("" = none).
    pub filter: String,
    /// Whether keystrokes currently go to the filter box.
    pub filter_active: bool,
    /// Whether the schedule (automatic cleaning) overlay is open.
    pub schedule_open: bool,
    /// The schedule being edited in the overlay.
    pub schedule_draft: cleansys_core::engine::schedule::Schedule,
    /// Index into [`App::schedule_fields`] of the highlighted row.
    pub schedule_field: usize,
    /// Feedback line shown at the bottom of the overlay.
    pub schedule_message: String,
    /// Backend name when a job is installed ("launchd", "cron", ...).
    pub schedule_installed: Option<String>,
    /// Result of the most recent automatic run, if any.
    pub schedule_last_run: Option<cleansys_core::engine::schedule::LastRun>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        // Get initial terminal size
        let (width, height) = terminal::size().unwrap_or((80, 24));

        let mut app = App {
            categories: Vec::new(),
            category_index: 0,
            item_list_state: ListState::default(),
            list_view_state: ListState::default(),
            expanded: None,
            entry_cursor: None,
            min_age_days: cleansys_core::engine::EngineConfig::load().min_age_days,
            is_root: check_root(),
            is_running: false,
            operation_start_time: None,
            operation_end_time: None,
            total_bytes_cleaned: 0,
            show_help: false,
            animation_frame: 0,
            last_frame_time: Instant::now(),
            terminal_width: width,
            terminal_height: height,
            run: None,
            run_total: 0,
            run_done: 0,
            run_current: None,
            run_summary: None,
            show_log: false,
            operation_logs: Vec::new(),
            notice: None,
            confirmation_mode: true,
            password_prompt: PasswordPrompt::new(),
            needs_sudo: false,
            pending_operations: Vec::new(),
            awaiting_run_confirmation: false,
            preview_open: false,
            preview_results: Vec::new(),
            needs_admin_notice: false,
            pending_run_selection: Vec::new(),
            board: cleansys_core::ScanBoard::default(),
            scan_rx: None,
            hide_empty: true,
            filter: String::new(),
            filter_active: false,
            schedule_open: false,
            schedule_draft: cleansys_core::engine::schedule::Schedule::default(),
            schedule_field: 0,
            schedule_message: String::new(),
            schedule_installed: None,
            schedule_last_run: None,
        };
        app.item_list_state.select(Some(0));

        app
    }

    // ── scanning, filtering, list navigation ──────────────────────────

    /// Start measuring every cleaner in the background (read-only).
    pub fn start_scan(&mut self) {
        if self.board.is_scanning() {
            return;
        }
        self.collapse_details();
        self.board = cleansys_core::ScanBoard::new(&self.categories);
        let (rx, total) = cleansys_core::spawn_scan(&self.categories);
        self.board.start(total);
        self.scan_rx = Some(rx);
    }

    /// Drain finished scan results (call every tick). Returns `true` if
    /// anything changed.
    pub fn poll_scan(&mut self) -> bool {
        let mut changed = false;
        let mut finished = false;
        if let Some(rx) = &self.scan_rx {
            while let Ok((ci, ii, info)) = rx.try_recv() {
                finished |= self.board.record(ci, ii, info);
                changed = true;
            }
        }
        if finished {
            self.scan_rx = None;
            self.ensure_category_visible();
        }
        changed
    }

    // ── details (per-path) view ────────────────────────────────────────

    /// Per-path entries of the expanded cleaner.
    pub fn expanded_entries(&self) -> &[cleansys_core::ScanEntry] {
        self.expanded
            .and_then(|(c, i)| self.board.get(c, i))
            .map_or(&[], |s| s.entries.as_slice())
    }

    /// Expand the highlighted cleaner's paths and move focus into them.
    pub fn expand_current(&mut self) {
        let Some((c, i)) = self.current_item() else {
            return;
        };
        if self.board.get(c, i).is_some_and(|s| !s.entries.is_empty()) {
            self.expanded = Some((c, i));
            self.entry_cursor = Some(0);
        }
    }

    pub fn collapse_details(&mut self) {
        self.expanded = None;
        self.entry_cursor = None;
    }

    pub fn entry_next(&mut self) {
        let n = self.expanded_entries().len();
        if let (Some(cur), true) = (self.entry_cursor, n > 0) {
            self.entry_cursor = Some((cur + 1).min(n - 1));
        }
    }

    pub fn entry_previous(&mut self) {
        if let Some(cur) = self.entry_cursor {
            self.entry_cursor = Some(cur.saturating_sub(1));
        }
    }

    /// Tick/untick the highlighted path (no-op for paths that can't be skipped).
    pub fn entry_toggle(&mut self) {
        let Some(cur) = self.entry_cursor else { return };
        let target = self
            .expanded_entries()
            .get(cur)
            .filter(|e| e.skippable)
            .map(|e| e.path.clone());
        if let Some(path) = target {
            self.board.toggle_entry(&path);
        }
    }

    /// Tick/untick every path of the expanded cleaner.
    pub fn entry_set_all(&mut self, on: bool) {
        if let Some((c, i)) = self.expanded {
            self.board.set_item_entries(c, i, on);
        }
    }

    /// Step the idle-day threshold (`[` = fewer days, `]` = more), save it and re-measure.
    pub fn step_min_age(&mut self, step: i32) {
        if self.is_running || self.board.is_scanning() {
            return;
        }
        let next = cleansys_core::engine::config::step_min_age(self.min_age_days, step);
        if next == self.min_age_days {
            return;
        }
        match cleansys_core::engine::EngineConfig::save_min_age_days(next) {
            Ok(()) => {
                self.min_age_days = next;
                self.start_scan();
            }
            Err(e) => self.log(format!("❌ Could not save the idle-days setting: {e}")),
        }
    }

    /// `(category, item)` pairs shown in the list right now.
    pub fn view_items(&self) -> Vec<(usize, usize)> {
        self.board.visible_items(
            &self.categories,
            self.category_index,
            &self.filter,
            self.hide_empty,
        )
    }

    /// The highlighted cleaner, if any.
    pub fn current_item(&self) -> Option<(usize, usize)> {
        self.view_items()
            .get(self.item_list_state.selected()?)
            .copied()
    }

    fn category_shown(&self, idx: usize) -> bool {
        self.board
            .category_visible(&self.categories, idx, self.hide_empty)
    }

    /// If the active category got hidden (empty after a scan), move on.
    pub fn ensure_category_visible(&mut self) {
        if self.categories.is_empty() || self.category_shown(self.category_index) {
            return;
        }
        if let Some(first) = self
            .board
            .first_visible_category(&self.categories, self.hide_empty)
        {
            self.category_index = first;
            self.item_list_state.select(Some(0));
        }
    }

    pub fn toggle_hide_empty(&mut self) {
        self.hide_empty = !self.hide_empty;
        self.ensure_category_visible();
        self.item_list_state.select(Some(0));
    }

    pub fn start_filter(&mut self) {
        self.filter_active = true;
    }

    pub fn clear_filter(&mut self) {
        self.filter.clear();
        self.filter_active = false;
        self.item_list_state.select(Some(0));
    }

    /// Key handling while the filter box has focus. Returns `true` if the key
    /// was consumed.
    pub fn handle_filter_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc => self.clear_filter(),
            KeyCode::Enter => self.filter_active = false,
            KeyCode::Backspace => {
                self.filter.pop();
                self.item_list_state.select(Some(0));
            }
            KeyCode::Down => self.next_item(),
            KeyCode::Up => self.previous_item(),
            KeyCode::Char(c) => {
                self.filter.push(c);
                self.item_list_state.select(Some(0));
            }
            _ => {}
        }
    }

    pub fn next_item(&mut self) {
        let len = self.view_items().len();
        if len == 0 {
            return;
        }
        let i = match self.item_list_state.selected() {
            Some(i) if i + 1 < len => i + 1,
            _ => 0,
        };
        self.item_list_state.select(Some(i));
    }

    pub fn previous_item(&mut self) {
        let len = self.view_items().len();
        if len == 0 {
            return;
        }
        let i = match self.item_list_state.selected() {
            Some(0) | None => len - 1,
            Some(i) => i - 1,
        };
        self.item_list_state.select(Some(i));
    }

    pub fn toggle_selected(&mut self) {
        if let Some((ci, ii)) = self.current_item() {
            // Allow selection even for root items, will prompt for password later
            let item = &mut self.categories[ci].items[ii];
            item.selected = !item.selected;
        }
    }

    pub fn next_category(&mut self) {
        let n = self.categories.len();
        for step in 1..=n {
            let cand = (self.category_index + step) % n;
            if self.category_shown(cand) {
                self.category_index = cand;
                break;
            }
        }
        self.item_list_state.select(Some(0));
    }

    pub fn previous_category(&mut self) {
        let n = self.categories.len();
        for step in 1..=n {
            let cand = (self.category_index + n - step) % n;
            if self.category_shown(cand) {
                self.category_index = cand;
                break;
            }
        }
        self.item_list_state.select(Some(0));
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    /// Tick every cleaner currently listed (the category, or the filter hits).
    pub fn select_all(&mut self) {
        for (ci, ii) in self.view_items() {
            // Allow selection of all items, will handle root permissions later
            self.categories[ci].items[ii].selected = true;
        }
    }

    /// Untick every cleaner currently listed.
    pub fn deselect_all(&mut self) {
        for (ci, ii) in self.view_items() {
            self.categories[ci].items[ii].selected = false;
        }
    }

    /// Select every item across every category (not just the active tab).
    pub fn select_all_everywhere(&mut self) {
        for category in &mut self.categories {
            for item in &mut category.items {
                item.selected = true;
            }
        }
    }

    /// Deselect every item across every category (not just the active tab).
    pub fn deselect_all_everywhere(&mut self) {
        for category in &mut self.categories {
            for item in &mut category.items {
                item.selected = false;
            }
        }
    }

    /// Gather selected cleaners and either show the confirmation overlay
    /// (when `confirmation_mode` is on) or start execution immediately.
    pub fn request_run(&mut self) -> Result<()> {
        if self.is_running {
            return Ok(());
        }

        let has_selected = self
            .categories
            .iter()
            .any(|c| c.items.iter().any(|i| i.selected));

        if !has_selected {
            self.set_notice(
                "Nothing selected — tick cleaners first (Space, or r for recommended).",
            );
            return Ok(());
        }

        let mut selected_cleaners = Vec::new();
        for (cat_idx, category) in self.categories.iter().enumerate() {
            for (item_idx, item) in category.items.iter().enumerate() {
                if item.selected {
                    let name = item.name.clone();
                    let function = item.function.clone();
                    selected_cleaners.push((cat_idx, item_idx, name, function, item.requires_root));
                }
            }
        }

        if selected_cleaners.is_empty() {
            self.set_notice(
                "Nothing selected — tick cleaners first (Space, or r for recommended).",
            );
            return Ok(());
        }

        if self.confirmation_mode {
            self.pending_run_selection = selected_cleaners;
            self.awaiting_run_confirmation = true;
            Ok(())
        } else {
            self.begin_execution(selected_cleaners)
        }
    }

    /// User confirmed the run in the confirmation overlay.
    pub fn confirm_pending_run(&mut self) -> Result<()> {
        self.awaiting_run_confirmation = false;
        let selected_cleaners = std::mem::take(&mut self.pending_run_selection);
        self.begin_execution(selected_cleaners)
    }

    /// User cancelled the confirmation overlay.
    pub fn cancel_run_confirmation(&mut self) {
        self.awaiting_run_confirmation = false;
        self.pending_run_selection.clear();
    }

    /// Preview (dry-run) every selected cleaner synchronously: measures real
    /// sizes/paths without deleting anything or invoking any mutating
    /// external command, and shows the results in an overlay.
    pub fn run_preview(&mut self) {
        if self.is_running {
            return;
        }

        let selected: Vec<(String, cleansys_core::CleanerFn)> = self
            .categories
            .iter()
            .flat_map(|c| c.items.iter())
            .filter(|i| i.selected)
            .map(|i| (i.name.clone(), i.function.clone()))
            .collect();

        if selected.is_empty() {
            self.set_notice(
                "Nothing selected — tick cleaners first (Space, or r for recommended).",
            );
            return;
        }

        self.preview_results.clear();
        // Paths unticked in the details view are left out of the preview too.
        cleansys_core::engine::skip::set_skipped(self.board.skipped_paths());
        for (name, function) in selected {
            match function(cleansys_core::RunOptions::preview().with_skips()) {
                Ok(result) => self.preview_results.push((name, result)),
                Err(e) => self.log(format!("⚠️  Preview failed for {name}: {e}")),
            }
        }
        cleansys_core::engine::skip::clear_skipped();
        self.preview_open = true;
    }

    /// Tick exactly the recommended cleaners (safe, user-land unless root).
    /// Returns how many are now selected.
    pub fn select_recommended(&mut self) -> usize {
        self.board
            .select_recommended(&mut self.categories, self.is_root)
    }

    // ── schedule overlay ───────────────────────────────────────────────

    /// Open the schedule overlay, loading any saved schedule.
    pub fn open_schedule(&mut self) {
        use cleansys_core::engine::schedule as sch;
        self.schedule_draft = sch::Schedule::load().unwrap_or_default();
        self.schedule_installed = sch::installed().map(|j| j.backend.to_string());
        self.schedule_last_run = sch::LastRun::load();
        self.schedule_field = 0;
        self.schedule_message = if self.schedule_installed.is_some() {
            "Active. Change values and press Enter to update, d to remove.".into()
        } else {
            "Not scheduled yet. Pick values and press Enter to enable.".into()
        };
        self.schedule_open = true;
    }

    pub fn close_schedule(&mut self) {
        self.schedule_open = false;
    }

    /// Rows shown for the current draft (day/backend rows are conditional).
    pub fn schedule_fields(&self) -> Vec<ScheduleField> {
        use cleansys_core::engine::schedule::Frequency;
        let mut f = vec![
            ScheduleField::Frequency,
            ScheduleField::Hour,
            ScheduleField::Minute,
        ];
        if self.schedule_draft.frequency != Frequency::Daily {
            f.push(ScheduleField::Day);
        }
        f.push(ScheduleField::Scope);
        if cfg!(all(unix, not(target_os = "macos"))) {
            f.push(ScheduleField::Backend);
        }
        f
    }

    pub fn schedule_next_field(&mut self) {
        let n = self.schedule_fields().len();
        self.schedule_field = (self.schedule_field + 1) % n;
    }

    pub fn schedule_prev_field(&mut self) {
        let n = self.schedule_fields().len();
        self.schedule_field = (self.schedule_field + n - 1) % n;
    }

    /// Change the highlighted field by `delta` (−1 / +1; wraps).
    pub fn schedule_adjust(&mut self, delta: i32) {
        use cleansys_core::engine::schedule::{Backend, Frequency, Scope};
        let fields = self.schedule_fields();
        let Some(field) = fields.get(self.schedule_field).copied() else {
            return;
        };
        let d = &mut self.schedule_draft;
        let wrap = |v: i32, lo: i32, hi: i32| -> i32 {
            let span = hi - lo + 1;
            lo + (v - lo).rem_euclid(span)
        };
        match field {
            ScheduleField::Frequency => {
                let order = [Frequency::Daily, Frequency::Weekly, Frequency::Monthly];
                let i = order.iter().position(|f| *f == d.frequency).unwrap_or(1) as i32;
                d.frequency = order[wrap(i + delta, 0, 2) as usize];
            }
            ScheduleField::Hour => d.hour = wrap(i32::from(d.hour) + delta, 0, 23) as u8,
            ScheduleField::Minute => {
                // 5-minute steps keep the UI quick; CLI allows any minute.
                d.minute = wrap(i32::from(d.minute) + delta * 5, 0, 59) as u8;
                if !d.minute.is_multiple_of(5) && delta != 0 {
                    d.minute -= d.minute % 5;
                }
            }
            ScheduleField::Day => match d.frequency {
                Frequency::Weekly => d.weekday = wrap(i32::from(d.weekday) + delta, 0, 6) as u8,
                Frequency::Monthly => {
                    d.day_of_month = wrap(i32::from(d.day_of_month) + delta, 1, 28) as u8
                }
                Frequency::Daily => {}
            },
            ScheduleField::Scope => {
                let order = [Scope::Recommended, Scope::Extended, Scope::Selected];
                let i = order.iter().position(|s| *s == d.scope).unwrap_or(0) as i32;
                d.scope = order[wrap(i + delta, 0, 2) as usize];
            }
            ScheduleField::Backend => {
                let order = [Backend::Auto, Backend::Systemd, Backend::Cron];
                let i = order.iter().position(|b| *b == d.backend).unwrap_or(0) as i32;
                d.backend = order[wrap(i + delta, 0, 2) as usize];
            }
        }
        // Keep the highlighted row valid if rows appeared/disappeared.
        let n = self.schedule_fields().len();
        if self.schedule_field >= n {
            self.schedule_field = n - 1;
        }
    }

    /// Ids of the currently ticked user-land cleaners (for `Scope::Selected`).
    fn selected_user_ids(&self) -> Vec<String> {
        self.categories
            .iter()
            .flat_map(|c| &c.items)
            .filter(|i| i.selected && !i.requires_root)
            .map(|i| i.id.clone())
            .collect()
    }

    /// Install/update the OS job from the draft.
    pub fn schedule_apply(&mut self) {
        use cleansys_core::engine::schedule as sch;
        if self.schedule_draft.scope == sch::Scope::Selected {
            self.schedule_draft.ids = self.selected_user_ids();
            if self.schedule_draft.ids.is_empty() {
                self.schedule_message =
                    "Scope 'selected' needs ticked cleaners — close this, tick some, reopen (or press r for the recommended set).".into();
                return;
            }
        }
        match sch::install(&self.schedule_draft) {
            Ok(job) => {
                self.schedule_installed = Some(job.backend.to_string());
                self.schedule_message = format!(
                    "✓ Scheduled: {} via {}",
                    self.schedule_draft.describe(),
                    job.backend
                );
            }
            Err(e) => self.schedule_message = format!("✗ {e:#}"),
        }
    }

    /// Remove the OS job and saved schedule.
    pub fn schedule_remove(&mut self) {
        match cleansys_core::engine::schedule::remove() {
            Ok(()) => {
                self.schedule_installed = None;
                self.schedule_message = "✓ Schedule removed.".into();
            }
            Err(e) => self.schedule_message = format!("✗ {e:#}"),
        }
    }

    /// Close the preview results overlay.
    pub fn close_preview(&mut self) {
        self.preview_open = false;
        self.preview_results.clear();
    }

    /// Test-only public wrapper around the private `Self::start_operations`,
    /// so integration tests in `tests/` (a separate crate, which can only
    /// see `pub` items) can drive the exact same post-authentication code
    /// path `handle_key` uses. Only compiled into debug builds.
    #[cfg(debug_assertions)]
    pub fn start_operations_for_tests(&mut self, selected_cleaners: &[PendingOperation]) {
        self.start_operations(selected_cleaners);
    }

    /// Append a line to the activity log (bounded).
    pub fn log(&mut self, line: impl Into<String>) {
        self.operation_logs.push(line.into());
        if self.operation_logs.len() > MAX_LOG_LINES {
            let excess = self.operation_logs.len() - MAX_LOG_LINES;
            self.operation_logs.drain(..excess);
        }
    }

    /// Show a one-line notice under the list (cleared on the next key press).
    pub fn set_notice(&mut self, msg: impl Into<String>) {
        self.notice = Some(msg.into());
    }

    /// Start the given cleaners on a background worker: mark them queued, open the
    /// activity log and return immediately. Progress arrives through [`App::poll_run`].
    /// Shared by the direct (already-elevated) path in [`Self::begin_execution`] and
    /// the post-password-authentication path in [`Self::handle_key`].
    fn start_operations(&mut self, selected_cleaners: &[PendingOperation]) {
        self.is_running = true;
        self.operation_start_time = Some(Instant::now());
        self.operation_end_time = None;
        self.total_bytes_cleaned = 0;
        self.run_summary = None;
        self.run_total = selected_cleaners.len();
        self.run_done = 0;
        self.run_current = None;
        self.show_log = true;
        self.operation_logs.clear();
        self.collapse_details();

        // Reset status for all items, then queue the selection.
        for category in &mut self.categories {
            for item in &mut category.items {
                item.bytes_cleaned = 0;
                item.status = None;
            }
        }
        for (cat_idx, item_idx, _, _, _) in selected_cleaners {
            self.categories[*cat_idx].items[*item_idx].status = Some(Status::Pending);
        }
        self.log(format!("Cleaning {} cleaner(s)…", selected_cleaners.len()));

        self.run = Some(crate::run::spawn(
            selected_cleaners.to_vec(),
            self.board.skipped_paths(),
            self.is_root,
        ));
    }

    /// Drain worker messages (call every tick). Returns `true` if anything changed.
    pub fn poll_run(&mut self) -> bool {
        use crate::run::RunMsg;
        let mut msgs = Vec::new();
        if let Some(run) = &self.run {
            while let Ok(m) = run.rx.try_recv() {
                msgs.push(m);
            }
        }
        let changed = !msgs.is_empty();
        let mut finished = false;
        for m in msgs {
            match m {
                RunMsg::Started(c, i) => {
                    if let Some(item) = self.categories.get_mut(c).and_then(|c| c.items.get_mut(i))
                    {
                        item.status = Some(Status::Running);
                        let name = item.name.clone();
                        self.log(format!("🔄 Running: {name}"));
                        self.run_current = Some(name);
                    }
                }
                RunMsg::Finished(c, i, result) => self.finish_cleaner(c, i, result),
                RunMsg::Cancelled(c, i) => {
                    if let Some(item) = self.categories.get_mut(c).and_then(|c| c.items.get_mut(i))
                    {
                        item.status = Some(Status::Error("Cancelled".to_string()));
                    }
                    self.run_done += 1;
                }
                RunMsg::Done => finished = true,
            }
        }
        if finished {
            self.finish_run();
        }
        changed
    }

    fn finish_cleaner(
        &mut self,
        c: usize,
        i: usize,
        result: Result<cleansys_core::CleaningResult, String>,
    ) {
        self.run_done += 1;
        let Some(item) = self.categories.get_mut(c).and_then(|c| c.items.get_mut(i)) else {
            return;
        };
        let name = item.name.clone();
        let root = item.requires_root;
        match result {
            Ok(res) => {
                let bytes = res.total_bytes;
                item.status = Some(Status::Success(format!(
                    "Cleaned {name}{} ({}, {} item(s))",
                    if root { " (root)" } else { "" },
                    format_size(bytes),
                    res.item_count()
                )));
                item.bytes_cleaned = bytes;
                item.last_result = Some(res.clone());
                self.total_bytes_cleaned += bytes;
                if bytes == 0 {
                    self.log(format!("ℹ️  {name}: nothing to clean"));
                } else {
                    self.log(format!(
                        "✅ {name}: freed {} across {} item(s)",
                        format_size(bytes),
                        res.item_count()
                    ));
                    for it in res.items.iter().take(20) {
                        self.log(format!("   → {} ({})", it.path_str(), format_size(it.size)));
                    }
                    if res.items.len() > 20 {
                        self.log(format!("   … and {} more", res.items.len() - 20));
                    }
                }
            }
            Err(msg) => {
                item.status = Some(Status::Error(msg.clone()));
                self.log(format!("❌ {name}: {msg}"));
            }
        }
    }

    fn finish_run(&mut self) {
        self.run = None;
        self.is_running = false;
        self.run_current = None;
        self.operation_end_time = Some(Instant::now());
        let (mut ok, mut failed) = (0, 0);
        let mut cancelled = false;
        for item in self.categories.iter().flat_map(|c| &c.items) {
            match &item.status {
                Some(Status::Success(_)) => ok += 1,
                Some(Status::Error(m)) => {
                    failed += 1;
                    cancelled |= m == "Cancelled";
                }
                _ => {}
            }
        }
        let summary = RunSummary {
            freed: self.total_bytes_cleaned,
            ok,
            failed,
            elapsed: self.get_elapsed_time(),
            cancelled,
        };
        let line = format!(
            "🎉 Cleaning {} — freed {}",
            if cancelled { "cancelled" } else { "complete" },
            format_size(summary.freed)
        );
        self.log(line.clone());
        crate::notifications::notify_completion(&line);
        self.run_summary = Some(summary);
        // Sizes changed — measure again so the list shows what is left.
        self.start_scan();
    }

    /// Stop the run after the cleaner currently executing (remaining ones are
    /// marked cancelled by the worker).
    pub fn cancel_run(&mut self) {
        if let Some(run) = &self.run {
            run.cancel();
            self.log("⏹ Cancelling after the current cleaner…");
        }
    }

    /// Dismiss the finished-run summary.
    pub fn dismiss_summary(&mut self) {
        self.run_summary = None;
    }

    pub fn get_elapsed_time(&self) -> String {
        match self.operation_start_time {
            Some(start) => {
                let elapsed = self
                    .operation_end_time
                    .map_or_else(|| start.elapsed(), |end| end.duration_since(start));
                if elapsed.as_secs() < 60 {
                    format!("{}s", elapsed.as_secs())
                } else {
                    format!("{}m {}s", elapsed.as_secs() / 60, elapsed.as_secs() % 60)
                }
            }
            None => "0s".to_string(),
        }
    }

    /// Advance the spinner animation (called every frame).
    pub fn update_animation(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_frame_time).as_millis() > 100 {
            self.animation_frame = (self.animation_frame + 1) % 10;
            self.last_frame_time = now;
        }
    }

    /// Actually start execution of the given selected cleaners: prompts for
    /// elevation if needed (sudo password on Unix, an "Administrator
    /// required" notice on Windows), or starts the run directly.
    fn begin_execution(&mut self, selected_cleaners: Vec<PendingOperation>) -> Result<()> {
        let has_root_operations = selected_cleaners.iter().any(|(_, _, _, _, root)| *root);

        // Check if we need elevation. `is_root` reflects whether the process
        // itself was launched with actual root privileges (e.g. `sudo
        // cleansys`) and never changes at runtime; `password_prompt` tracks
        // whether the user has already authenticated via the in-app sudo
        // dialog this session. Both must be considered here, or a user who
        // already authenticated once would be re-prompted for their
        // password on every subsequent run.
        if has_root_operations && !self.is_root && !self.password_prompt.is_authenticated() {
            self.pending_operations.clone_from(&selected_cleaners);
            if cleansys_core::utils::supports_sudo_prompt() {
                self.needs_sudo = true;
                self.password_prompt.show();
            } else {
                self.needs_admin_notice = true;
            }
            return Ok(());
        }

        self.start_operations(&selected_cleaners);

        Ok(())
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        // If password prompt is visible, handle password input first
        if self.password_prompt.is_visible() {
            match key.code {
                KeyCode::Enter => {
                    // Submit password and authenticate
                    match self.password_prompt.submit() {
                        Ok(true) => {
                            // Authentication successful, proceed with operations
                            self.needs_sudo = false;
                            self.password_prompt.hide();

                            // Now start the actual cleaning operations
                            let selected_cleaners = self.pending_operations.clone();
                            self.pending_operations.clear();

                            if !selected_cleaners.is_empty() {
                                self.start_operations(&selected_cleaners);
                            }
                        }
                        Ok(false) => {
                            // Authentication failed, stay on prompt
                        }
                        Err(e) => {
                            self.log(format!("❌ Authentication error: {e}"));
                            self.password_prompt.hide();
                            self.needs_sudo = false;
                            self.pending_operations.clear();
                        }
                    }
                }
                KeyCode::Esc => {
                    // Cancel password prompt
                    self.password_prompt.cancel();
                    self.needs_sudo = false;
                    self.pending_operations.clear();
                }
                KeyCode::Char(c) => {
                    self.password_prompt.add_char(c);
                }
                KeyCode::Backspace => {
                    self.password_prompt.remove_char();
                }
                _ => {}
            }
            return Ok(false);
        }

        // "Needs Administrator" notice (Windows only — no interactive sudo flow there)
        if self.needs_admin_notice {
            match key.code {
                KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => {
                    self.needs_admin_notice = false;
                    self.pending_operations.clear();
                }
                _ => {}
            }
            return Ok(false);
        }

        // Run confirmation overlay
        if self.awaiting_run_confirmation {
            match key.code {
                KeyCode::Enter | KeyCode::Char('y') => {
                    self.confirm_pending_run()?;
                }
                KeyCode::Esc | KeyCode::Char('n') => {
                    self.cancel_run_confirmation();
                }
                _ => {}
            }
            return Ok(false);
        }

        // Filter box has focus: every key is text (or navigation)
        if self.filter_active && !self.show_help {
            self.handle_filter_key(key.code);
            return Ok(false);
        }

        // Schedule overlay
        if self.schedule_open {
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('S') => self.close_schedule(),
                KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => self.schedule_next_field(),
                KeyCode::Up | KeyCode::Char('k') | KeyCode::BackTab => self.schedule_prev_field(),
                KeyCode::Left | KeyCode::Char('h') => self.schedule_adjust(-1),
                KeyCode::Right | KeyCode::Char('l') | KeyCode::Char(' ') => self.schedule_adjust(1),
                KeyCode::Enter => self.schedule_apply(),
                KeyCode::Char('d') | KeyCode::Delete => self.schedule_remove(),
                _ => {}
            }
            return Ok(false);
        }

        // Preview results overlay
        if self.preview_open {
            match key.code {
                KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => {
                    self.close_preview();
                }
                _ => {}
            }
            return Ok(false);
        }

        // Details focus: ↑/↓ move between paths, Space ticks one, a/n all/none,
        // ←/Esc/q leave. Everything else is ignored while inside the details.
        if self.entry_cursor.is_some() && !self.show_help && !self.is_running {
            match key.code {
                KeyCode::Down | KeyCode::Char('j') => self.entry_next(),
                KeyCode::Up | KeyCode::Char('k') => self.entry_previous(),
                KeyCode::Char(' ') => self.entry_toggle(),
                KeyCode::Char('a') => self.entry_set_all(true),
                KeyCode::Char('n') => self.entry_set_all(false),
                KeyCode::Left | KeyCode::Right | KeyCode::Esc | KeyCode::Char('q') => {
                    self.collapse_details();
                }
                _ => {}
            }
            return Ok(false);
        }

        // Any key clears the transient notice.
        self.notice = None;

        match (key.code, key.modifiers) {
            // Quit (or leave help / cancel a running clean)
            (KeyCode::Char('q'), _) => {
                if self.show_help {
                    self.show_help = false;
                } else if self.is_running {
                    self.cancel_run();
                } else {
                    return Ok(true);
                }
            }

            // Navigation (stays usable while a clean is running)
            (KeyCode::Down, _) => {
                if !self.show_help {
                    self.next_item();
                }
            }
            (KeyCode::Up, _) => {
                if !self.show_help {
                    self.previous_item();
                }
            }
            (KeyCode::Char('j'), _) => {
                if !self.show_help {
                    self.next_item();
                }
            }
            (KeyCode::Char('k'), _) => {
                if !self.show_help {
                    self.previous_item();
                }
            }
            (KeyCode::Tab, _) => {
                if !self.show_help {
                    self.next_category();
                }
            }
            (KeyCode::BackTab, _) => {
                if !self.show_help {
                    self.previous_category();
                }
            }
            (KeyCode::Home, _) if !self.show_help => {
                self.item_list_state.select(Some(0));
            }
            (KeyCode::End, _) if !self.show_help => {
                let len = self.view_items().len();
                if len > 0 {
                    self.item_list_state.select(Some(len - 1));
                }
            }
            // Expand the highlighted cleaner's per-path details
            (KeyCode::Right, _) => {
                if !self.show_help && !self.is_running {
                    self.expand_current();
                }
            }
            // Idle-day selector for project build output
            (KeyCode::Char('['), _) => {
                if !self.show_help {
                    self.step_min_age(-1);
                }
            }
            (KeyCode::Char(']'), _) => {
                if !self.show_help {
                    self.step_min_age(1);
                }
            }
            // Selection
            (KeyCode::Char(' '), KeyModifiers::NONE) => {
                if !self.show_help && !self.is_running {
                    self.toggle_selected();
                }
            }
            // Run cleaners (shows a confirmation overlay first, unless
            // confirmation prompts are disabled via 'y').
            (KeyCode::Enter, _) => {
                if !self.show_help {
                    self.request_run()?;
                }
            }
            // Preview (dry-run) selected cleaners — measures real sizes/paths
            // without deleting anything.
            (KeyCode::Char('d'), _) => {
                if !self.show_help && !self.is_running {
                    self.run_preview();
                }
            }
            // Recommended preset: tick only the safe, user-land cleaners
            (KeyCode::Char('r'), _) => {
                if !self.show_help && !self.is_running {
                    self.select_recommended();
                }
            }
            // Schedule automatic cleaning
            (KeyCode::Char('S'), _) => {
                if !self.show_help && !self.is_running {
                    self.open_schedule();
                }
            }
            // Help dialog
            (KeyCode::Char('?' | 'h'), _) => {
                self.toggle_help();
            }
            // Activity log panel
            (KeyCode::Char('L'), _) => {
                if !self.show_help {
                    self.show_log = !self.show_log;
                }
            }
            // Filter across every category
            (KeyCode::Char('/'), _) => {
                if !self.show_help {
                    self.start_filter();
                }
            }
            // Hide / show cleaners that have nothing to clean
            (KeyCode::Char('e'), _) => {
                if !self.show_help && !self.is_running {
                    self.toggle_hide_empty();
                }
            }
            // Re-measure everything
            (KeyCode::Char('R'), _) => {
                if !self.show_help && !self.is_running {
                    self.start_scan();
                }
            }
            // Toggle the "ask before running" confirmation
            (KeyCode::Char('y'), _) => {
                if !self.show_help {
                    self.toggle_confirmation_mode();
                }
            }
            // Dismiss the run summary / cancel a clean / clear the filter
            (KeyCode::Esc, _) => {
                if self.show_help {
                    self.show_help = false;
                } else if self.is_running {
                    self.cancel_run();
                } else if self.run_summary.is_some() {
                    self.dismiss_summary();
                } else if !self.filter.is_empty() {
                    self.clear_filter();
                }
            }
            // Select / deselect in this category or everywhere
            (KeyCode::Char('a'), _) => {
                if !self.show_help && !self.is_running {
                    self.select_all();
                }
            }
            (KeyCode::Char('n'), _) => {
                if !self.show_help && !self.is_running {
                    self.deselect_all();
                }
            }
            (KeyCode::Char('A'), _) => {
                if !self.show_help && !self.is_running {
                    self.select_all_everywhere();
                }
            }
            (KeyCode::Char('N'), _) if !self.show_help && !self.is_running => {
                self.deselect_all_everywhere();
            }
            _ => {}
        }

        Ok(false)
    }

    pub fn handle_resize(&mut self, width: u16, height: u16) {
        self.terminal_width = width;
        self.terminal_height = height;
    }

    pub fn toggle_confirmation_mode(&mut self) {
        self.confirmation_mode = !self.confirmation_mode;
    }
}
