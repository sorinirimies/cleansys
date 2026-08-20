//! Application state for the CleanSys Iced GUI.

use cleansys_core::{CleanerCategory, Status};

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
}

impl Default for CleanSysGui {
    fn default() -> Self {
        Self::new()
    }
}

impl CleanSysGui {
    /// Construct a fresh application state with all known cleaners loaded.
    pub fn new() -> Self {
        Self {
            categories: cleansys_core::load_categories(),
            logs: Vec::new(),
            total_bytes_cleaned: 0,
            is_running: false,
            is_root: cleansys_core::check_root(),
            needs_password: false,
            password_input: String::new(),
            password_error: None,
            pending_root_ops: Vec::new(),
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

    /// True if any selected item requires root and we don't already have it.
    pub fn selection_needs_root(&self) -> bool {
        !self.is_root
            && self
                .categories
                .iter()
                .flat_map(|c| &c.items)
                .any(|i| i.selected && i.requires_root)
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
}
