//! Framework-agnostic domain model shared by the TUI and GUI front-ends.
//!
//! Nothing in this module depends on `ratatui`, `crossterm`, or `iced` — it is
//! pure application state that both front-ends render in their own way.

use anyhow::Result;

use crate::cleaners::{system_cleaners, user_cleaners};

/// The outcome of running (or attempting to run) a single cleaner.
#[derive(Debug, Clone)]
pub enum Status {
    /// The cleaner is queued but has not started yet.
    Pending,
    /// The cleaner is currently executing.
    Running,
    /// The cleaner finished successfully; the string is a human-readable summary.
    Success(String),
    /// The cleaner failed; the string is a human-readable error message.
    Error(String),
}

impl Status {
    /// Return a single-glyph representation of this status, using `frame` to
    /// select an animation frame for the `Running` state (spinner).
    pub fn get_animation_frame(&self, frame: usize) -> &'static str {
        match self {
            Status::Running => {
                const SPINNER: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
                SPINNER[frame % SPINNER.len()]
            }
            Status::Success(_) => "✓",
            Status::Error(_) => "✗",
            Status::Pending => "•",
        }
    }
}

/// A single selectable cleaning operation (e.g. "Browser Caches").
pub struct CleanerItem {
    /// Human-readable name of the cleaner.
    pub name: String,
    /// Short description of what the cleaner removes.
    pub description: String,
    /// Whether this cleaner needs root/administrator privileges to run.
    pub requires_root: bool,
    /// Whether the user has selected this cleaner to run.
    pub selected: bool,
    /// The function that performs the actual cleaning.
    /// Takes `dry_run: bool` (when `false`, files are actually removed) and
    /// returns the number of bytes freed.
    pub function: fn(bool) -> Result<u64>,
    /// Bytes freed by the most recent run of this cleaner.
    pub bytes_cleaned: u64,
    /// Current run status, if the cleaner has been queued/run at least once.
    pub status: Option<Status>,
}

/// A named group of related [`CleanerItem`]s (e.g. "User Land Cleaners").
pub struct CleanerCategory {
    /// Category display name.
    pub name: String,
    /// Category description.
    pub description: String,
    /// The cleaners that belong to this category.
    pub items: Vec<CleanerItem>,
}

/// Build the default set of categories (User + System) with all known
/// cleaners loaded from [`user_cleaners`] and [`system_cleaners`].
///
/// This is shared between the TUI and GUI front-ends so both present the
/// exact same list of cleaners.
pub fn load_categories() -> Vec<CleanerCategory> {
    let mut user_items = Vec::new();
    for cleaner in user_cleaners::get_cleaners() {
        user_items.push(CleanerItem {
            name: cleaner.name.to_string(),
            description: cleaner.description.to_string(),
            requires_root: false,
            selected: false,
            function: cleaner.function,
            bytes_cleaned: 0,
            status: None,
        });
    }

    let mut system_items = Vec::new();
    for cleaner in system_cleaners::get_cleaners() {
        system_items.push(CleanerItem {
            name: cleaner.name.to_string(),
            description: cleaner.description.to_string(),
            requires_root: true,
            selected: false,
            function: cleaner.function,
            bytes_cleaned: 0,
            status: None,
        });
    }

    vec![
        CleanerCategory {
            name: "User Land Cleaners".to_string(),
            description: "Clean user-specific files and caches".to_string(),
            items: user_items,
        },
        CleanerCategory {
            name: "System Cleaners".to_string(),
            description: "Clean system files and caches (requires root)".to_string(),
            items: system_items,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_categories_has_user_and_system() {
        let categories = load_categories();
        assert_eq!(categories.len(), 2);
        assert_eq!(categories[0].name, "User Land Cleaners");
        assert_eq!(categories[1].name, "System Cleaners");
        assert!(!categories[0].items.is_empty());
        assert!(!categories[1].items.is_empty());
        assert!(categories[1].items.iter().all(|i| i.requires_root));
        assert!(categories[0].items.iter().all(|i| !i.requires_root));
    }

    #[test]
    fn status_animation_frames() {
        assert_eq!(Status::Pending.get_animation_frame(0), "•");
        assert_eq!(Status::Success("ok".into()).get_animation_frame(0), "✓");
        assert_eq!(Status::Error("bad".into()).get_animation_frame(0), "✗");
        let running = Status::Running;
        assert_eq!(running.get_animation_frame(0), "⠋");
        assert_eq!(running.get_animation_frame(1), "⠙");
    }
}
