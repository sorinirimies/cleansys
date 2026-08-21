//! Application state for the CleanSys Iced GUI.

use cleansys_core::{CleanerCategory, Status};

use crate::theme::ThemeColors;

/// Top-level state for the CleanSys GUI application.
pub struct CleanSysGui {
    /// Cleaner categories and items (shared domain model from `cleansys-core`).
    pub categories: Vec<CleanerCategory>,
    /// Rolling log of operation messages shown in the activity panel.
    pub logs: Vec<String>,
    /// Total bytes freed across all completed operations in the current run.
    pub total_bytes_cleaned: u64,
    /// Whether cleaners are currently running.
    pub is_running: bool,
    /// Whether the process is running with root/administrator privileges.
    pub is_root: bool,
    /// Whether the sudo authentication dialog is visible.
    pub needs_password: bool,
    /// The password currently typed into the authentication dialog.
    pub password_input: String,
    /// Error message shown in the authentication dialog, if any.
    pub password_error: Option<String>,
    /// Operations queued to run once sudo authentication succeeds.
    pub pending_root_ops: Vec<(usize, usize)>,
    /// Index of the currently active category tab.
    pub active_tab: usize,
    /// Index of the currently selected UI theme (see `cleansys_core::THEME_NAMES`).
    pub theme_index: usize,
    /// Whether the "confirm this run" dialog is visible.
    pub confirm_run_pending: bool,
    /// Whether a preview (dry-run) is currently being computed.
    pub previewing: bool,
    /// Whether the preview results dialog is visible.
    pub preview_open: bool,
    /// Results of the most recent preview run: `(cleaner_name, result)`.
    pub preview_results: Vec<(String, cleansys_core::CleaningResult)>,
    /// Total number of operations in the current run (for the progress bar).
    pub operations_total: usize,
    /// Number of operations completed so far in the current run.
    pub operations_completed: usize,
    /// Whether the "needs Administrator" notice is visible (Windows only;
    /// Windows has no interactive sudo-password flow, so this replaces the
    /// password dialog when elevation is required there).
    pub needs_admin_notice: bool,
}

impl Default for CleanSysGui {
    fn default() -> Self {
        Self::new()
    }
}

impl CleanSysGui {
    /// Construct a fresh application state with all known cleaners loaded.
    pub fn new() -> Self {
        let settings = cleansys_core::load_settings().unwrap_or_default();
        let mut categories = cleansys_core::load_categories();
        for category in &mut categories {
            for item in &mut category.items {
                item.selected = settings.is_selected(&category.name, &item.name);
            }
        }
        Self {
            categories,
            logs: Vec::new(),
            total_bytes_cleaned: 0,
            is_running: false,
            is_root: cleansys_core::check_root(),
            needs_password: false,
            password_input: String::new(),
            password_error: None,
            pending_root_ops: Vec::new(),
            active_tab: 0,
            theme_index: settings.theme_index(),
            confirm_run_pending: false,
            previewing: false,
            preview_open: false,
            preview_results: Vec::new(),
            operations_total: 0,
            operations_completed: 0,
            needs_admin_notice: false,
        }
    }

    /// Number of currently selected items across all categories.
    pub fn selected_count(&self) -> usize {
        self.categories
            .iter()
            .flat_map(|c| &c.items)
            .filter(|i| i.selected)
            .count()
    }

    /// Number of currently selected items within a specific category.
    pub fn selected_count_in(&self, cat_idx: usize) -> usize {
        self.categories
            .get(cat_idx)
            .map(|c| c.items.iter().filter(|i| i.selected).count())
            .unwrap_or(0)
    }

    /// True if any selected item requires root and we don't already have it.
    pub fn selection_needs_root(&self) -> bool {
        !self.is_root
            && self
                .categories
                .iter()
                .flat_map(|c| &c.items)
                .any(|i| i.selected && i.requires_root)
    }

    /// `(category_index, item_index)` pairs of every currently-selected item.
    pub fn selected_indices(&self) -> Vec<(usize, usize)> {
        self.categories
            .iter()
            .enumerate()
            .flat_map(|(ci, c)| {
                c.items
                    .iter()
                    .enumerate()
                    .filter(|(_, i)| i.selected)
                    .map(move |(ii, _)| (ci, ii))
            })
            .collect()
    }

    /// Fraction of the current run's operations completed so far, in
    /// `0.0..=1.0`. `0.0` when no run is in progress.
    pub fn progress_fraction(&self) -> f32 {
        if self.operations_total == 0 {
            0.0
        } else {
            (self.operations_completed as f32 / self.operations_total as f32).clamp(0.0, 1.0)
        }
    }

    /// Whether this platform supports the interactive sudo-password
    /// elevation flow (Unix). Windows uses UAC/Administrator tokens instead.
    pub fn supports_sudo_prompt(&self) -> bool {
        cleansys_core::utils::supports_sudo_prompt()
    }

    /// Push a line to the activity log, keeping only the most recent entries.
    pub fn push_log(&mut self, line: impl Into<String>) {
        self.logs.push(line.into());
        if self.logs.len() > 500 {
            self.logs.remove(0);
        }
    }

    /// Mark every selected item as `Status::Pending` in preparation for a run.
    pub fn mark_selected_pending(&mut self) {
        for category in &mut self.categories {
            for item in &mut category.items {
                if item.selected {
                    item.status = Some(Status::Pending);
                }
            }
        }
    }

    /// Derive the full [`ThemeColors`] from the currently active core theme.
    ///
    /// Call this at the top of view functions: `let c = state.colors();`
    pub fn colors(&self) -> ThemeColors {
        ThemeColors::from_core(&cleansys_core::theme_by_index(self.theme_index))
    }

    /// Return a custom `iced::Theme` derived from the active core theme, for
    /// the top-level `iced::application(...).theme(...)` callback.
    pub fn iced_theme(&self) -> iced::Theme {
        crate::theme::iced_theme_for(self.theme_index)
    }

    /// The display name of the currently active theme.
    pub fn current_theme_name(&self) -> &'static str {
        cleansys_core::THEME_NAMES
            .get(self.theme_index)
            .copied()
            .unwrap_or("Default")
    }

    /// Build the full [`cleansys_core::Settings`] snapshot for the current
    /// state (theme + selected cleaners), for persistence.
    pub fn current_settings(&self) -> cleansys_core::Settings {
        let selected_cleaners = self
            .categories
            .iter()
            .flat_map(|c| {
                let cat_name = c.name.clone();
                c.items
                    .iter()
                    .filter(|i| i.selected)
                    .map(move |i| cleansys_core::Settings::selection_key(&cat_name, &i.name))
            })
            .collect();

        cleansys_core::Settings {
            theme_name: Some(self.current_theme_name().to_string()),
            selected_cleaners,
        }
    }

    /// Persist the current theme and cleaner selections to `settings.json`
    /// (best-effort; failures are logged but never surfaced to the UI).
    pub fn save_theme(&self) {
        if let Err(e) = cleansys_core::save_settings(&self.current_settings()) {
            log::warn!("failed to save theme preference: {e}");
        }
    }

    /// Persist the current cleaner selections (and theme) to `settings.json`.
    pub fn save_selections(&self) {
        if let Err(e) = cleansys_core::save_settings(&self.current_settings()) {
            log::warn!("failed to save cleaner selections: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_loads_categories_and_defaults() {
        let state = CleanSysGui::new();
        assert_eq!(state.categories.len(), 2);
        assert_eq!(state.active_tab, 0);
        assert_eq!(state.selected_count(), 0);
        assert!(state.logs.is_empty());
        assert_eq!(state.total_bytes_cleaned, 0);
        assert!(!state.is_running);
        assert!(!state.needs_password);
    }

    #[test]
    fn selected_count_tracks_selections() {
        let mut state = CleanSysGui::new();
        assert_eq!(state.selected_count(), 0);
        state.categories[0].items[0].selected = true;
        assert_eq!(state.selected_count(), 1);
        assert_eq!(state.selected_count_in(0), 1);
        assert_eq!(state.selected_count_in(1), 0);
        state.categories[1].items[0].selected = true;
        assert_eq!(state.selected_count(), 2);
    }

    #[test]
    fn selected_count_in_out_of_range_is_zero() {
        let state = CleanSysGui::new();
        assert_eq!(state.selected_count_in(99), 0);
    }

    /// Find `(cat_idx, item_idx)` of a system item that actually requires
    /// root on this platform (not every "System Cleaners" entry does —
    /// e.g. Homebrew on macOS must not run as root).
    fn first_root_required_item(state: &CleanSysGui) -> (usize, usize) {
        for (ci, category) in state.categories.iter().enumerate() {
            for (ii, item) in category.items.iter().enumerate() {
                if item.requires_root {
                    return (ci, ii);
                }
            }
        }
        panic!("expected at least one root-requiring cleaner on this platform");
    }

    #[test]
    fn selection_needs_root_when_not_root_and_system_item_selected() {
        let mut state = CleanSysGui::new();
        state.is_root = false;
        assert!(!state.selection_needs_root());

        let (ci, ii) = first_root_required_item(&state);
        state.categories[ci].items[ii].selected = true;
        assert!(state.selection_needs_root());
    }

    #[test]
    fn selection_needs_root_false_when_already_root() {
        let mut state = CleanSysGui::new();
        state.is_root = true;
        let (ci, ii) = first_root_required_item(&state);
        state.categories[ci].items[ii].selected = true;
        assert!(!state.selection_needs_root());
    }

    #[test]
    fn selection_needs_root_false_for_user_only_selection() {
        let mut state = CleanSysGui::new();
        state.is_root = false;
        state.categories[0].items[0].selected = true;
        assert!(!state.selection_needs_root());
    }

    #[test]
    fn push_log_caps_at_500_entries() {
        let mut state = CleanSysGui::new();
        for i in 0..600 {
            state.push_log(format!("line {i}"));
        }
        assert_eq!(state.logs.len(), 500);
        // Oldest entries should have been evicted; the log should end with the
        // most recent line.
        assert_eq!(state.logs.last().unwrap(), "line 599");
    }

    #[test]
    fn mark_selected_pending_only_affects_selected_items() {
        let mut state = CleanSysGui::new();
        state.categories[0].items[0].selected = true;
        state.mark_selected_pending();

        assert!(matches!(
            state.categories[0].items[0].status,
            Some(Status::Pending)
        ));
        assert!(state.categories[0].items[1].status.is_none());
    }

    #[test]
    fn theme_index_defaults_in_range() {
        let state = CleanSysGui::new();
        assert!(state.theme_index < cleansys_core::THEME_COUNT);
    }

    #[test]
    fn current_theme_name_matches_index() {
        let mut state = CleanSysGui::new();
        state.theme_index = cleansys_core::theme_index_by_name("Dracula");
        assert_eq!(state.current_theme_name(), "Dracula");
    }

    #[test]
    fn current_theme_name_falls_back_for_out_of_range_index() {
        let mut state = CleanSysGui::new();
        state.theme_index = 9999;
        assert_eq!(state.current_theme_name(), "Default");
    }

    #[test]
    fn colors_does_not_panic_for_any_theme() {
        let mut state = CleanSysGui::new();
        for i in 0..cleansys_core::THEME_COUNT {
            state.theme_index = i;
            let _ = state.colors();
            let _ = state.iced_theme();
        }
    }

    #[test]
    fn selected_indices_returns_selected_pairs() {
        let mut state = CleanSysGui::new();
        state.categories[0].items[0].selected = true;
        state.categories[0].items[2].selected = true;
        let indices = state.selected_indices();
        assert_eq!(indices, vec![(0, 0), (0, 2)]);
    }

    #[test]
    fn progress_fraction_is_zero_with_no_operations() {
        let state = CleanSysGui::new();
        assert_eq!(state.progress_fraction(), 0.0);
    }

    #[test]
    fn progress_fraction_computes_ratio() {
        let mut state = CleanSysGui::new();
        state.operations_total = 4;
        state.operations_completed = 1;
        assert_eq!(state.progress_fraction(), 0.25);
        state.operations_completed = 4;
        assert_eq!(state.progress_fraction(), 1.0);
    }

    #[test]
    fn current_settings_includes_selected_cleaners() {
        let mut state = CleanSysGui::new();
        state.categories[0].items[0].selected = true;
        let name = state.categories[0].items[0].name.clone();
        let cat_name = state.categories[0].name.clone();
        let settings = state.current_settings();
        assert!(settings.is_selected(&cat_name, &name));
    }

    #[test]
    fn new_restores_selection_from_settings() {
        // Can't easily isolate the real config dir in a unit test, but we can
        // at least confirm current_settings() round-trips through is_selected
        // the same way new() consults it.
        let mut state = CleanSysGui::new();
        state.categories[0].items[1].selected = true;
        let settings = state.current_settings();
        let cat_name = state.categories[0].name.clone();
        let item_name = state.categories[0].items[1].name.clone();
        assert!(settings.is_selected(&cat_name, &item_name));
        assert!(!settings.is_selected(&cat_name, "Definitely Not A Real Cleaner"));
    }
}
