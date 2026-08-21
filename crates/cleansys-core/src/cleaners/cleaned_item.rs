use std::path::PathBuf;

/// Represents a single item that was cleaned
#[derive(Debug, Clone)]
pub struct CleanedItem {
    /// The path of the cleaned file or directory
    pub path: PathBuf,
    /// Size in bytes
    pub size: u64,
    /// Type of item (file, directory, etc.)
    pub item_type: CleanedItemType,
    /// Short human-readable label (e.g. "Firefox cache", "npm cache") shown
    /// alongside the path in the TUI/GUI detailed views.
    pub label: String,
}

/// Type of cleaned item
#[derive(Debug, Clone, PartialEq)]
pub enum CleanedItemType {
    File,
    Directory,
    SymLink,
}

impl CleanedItem {
    /// Create a new cleaned item
    pub fn new(
        path: PathBuf,
        size: u64,
        item_type: CleanedItemType,
        label: impl Into<String>,
    ) -> Self {
        Self {
            path,
            size,
            item_type,
            label: label.into(),
        }
    }

    /// Create a file item
    pub fn file(path: PathBuf, size: u64, label: impl Into<String>) -> Self {
        Self::new(path, size, CleanedItemType::File, label)
    }

    /// Create a directory item
    pub fn directory(path: PathBuf, size: u64, label: impl Into<String>) -> Self {
        Self::new(path, size, CleanedItemType::Directory, label)
    }

    /// Get the path as a string
    pub fn path_str(&self) -> String {
        self.path.to_string_lossy().to_string()
    }

    /// Get the filename
    pub fn filename(&self) -> String {
        self.path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| self.path_str())
    }
}

/// Result of a cleaning operation
#[derive(Debug, Clone)]
pub struct CleaningResult {
    /// Total bytes cleaned
    pub total_bytes: u64,
    /// List of cleaned items
    pub items: Vec<CleanedItem>,
}

impl CleaningResult {
    /// Create a new empty result
    pub fn new() -> Self {
        Self {
            total_bytes: 0,
            items: Vec::new(),
        }
    }

    /// Add a cleaned item
    pub fn add_item(&mut self, item: CleanedItem) {
        self.total_bytes += item.size;
        self.items.push(item);
    }

    /// Add multiple items
    pub fn add_items(&mut self, items: Vec<CleanedItem>) {
        for item in items {
            self.add_item(item);
        }
    }

    /// Merge another result into this one
    pub fn merge(&mut self, other: CleaningResult) {
        self.total_bytes += other.total_bytes;
        self.items.extend(other.items);
    }

    /// Get the number of items cleaned
    pub fn item_count(&self) -> usize {
        self.items.len()
    }
}

impl Default for CleaningResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Options controlling how a cleaner function runs.
///
/// Replaces the old bare `skip_confirmation: bool` parameter so cleaners can
/// also support a real dry-run/preview mode: [`RunOptions::preview`] measures
/// exactly what *would* be removed (real sizes, real paths) without deleting
/// anything or invoking any external command that has side effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunOptions {
    /// When `true`, skip interactive y/n prompts (always `true` from the
    /// TUI/GUI, which have no stdin prompt loop; only meaningful for the
    /// CLI `menu`/`user`/`system` subcommands).
    pub skip_confirmation: bool,
    /// When `true`, measure and report what would be cleaned without
    /// actually deleting anything or running any mutating external command.
    pub dry_run: bool,
}

impl RunOptions {
    /// Actually perform the cleaning (skips interactive prompts).
    pub const fn execute() -> Self {
        Self {
            skip_confirmation: true,
            dry_run: false,
        }
    }

    /// Actually perform the cleaning, honouring interactive confirmation
    /// prompts (used by the CLI `menu`/`user`/`system` subcommands).
    pub const fn execute_with_confirmation() -> Self {
        Self {
            skip_confirmation: false,
            dry_run: false,
        }
    }

    /// Preview mode: measure real sizes/paths, delete nothing.
    pub const fn preview() -> Self {
        Self {
            skip_confirmation: true,
            dry_run: true,
        }
    }
}

/// Signature shared by every cleaner function: takes [`RunOptions`] and
/// returns the structured set of items actually removed (or, in preview
/// mode, that *would be* removed), with real per-item sizes.
pub type CleanerFn = fn(RunOptions) -> anyhow::Result<CleaningResult>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleaned_item_file_has_label() {
        let item = CleanedItem::file(PathBuf::from("/tmp/foo"), 42, "Temp file");
        assert_eq!(item.label, "Temp file");
        assert_eq!(item.item_type, CleanedItemType::File);
        assert_eq!(item.size, 42);
    }

    #[test]
    fn cleaned_item_directory_has_label() {
        let item = CleanedItem::directory(PathBuf::from("/tmp/dir"), 100, "Cache dir");
        assert_eq!(item.label, "Cache dir");
        assert_eq!(item.item_type, CleanedItemType::Directory);
    }

    #[test]
    fn cleaning_result_add_item_updates_total() {
        let mut result = CleaningResult::new();
        result.add_item(CleanedItem::file(PathBuf::from("/a"), 10, "a"));
        result.add_item(CleanedItem::file(PathBuf::from("/b"), 20, "b"));
        assert_eq!(result.total_bytes, 30);
        assert_eq!(result.item_count(), 2);
    }

    #[test]
    fn cleaning_result_merge_combines_totals_and_items() {
        let mut a = CleaningResult::new();
        a.add_item(CleanedItem::file(PathBuf::from("/a"), 10, "a"));
        let mut b = CleaningResult::new();
        b.add_item(CleanedItem::file(PathBuf::from("/b"), 5, "b"));

        a.merge(b);
        assert_eq!(a.total_bytes, 15);
        assert_eq!(a.item_count(), 2);
    }

    #[test]
    fn cleaning_result_default_is_empty() {
        let result = CleaningResult::default();
        assert_eq!(result.total_bytes, 0);
        assert_eq!(result.item_count(), 0);
    }

    #[test]
    fn filename_falls_back_to_path_str_without_file_name() {
        let item = CleanedItem::file(PathBuf::from("/"), 0, "root");
        // "/" has no file_name(), so filename() falls back to path_str().
        assert_eq!(item.filename(), item.path_str());
    }
}

#[cfg(test)]
mod run_options_tests {
    use super::RunOptions;

    #[test]
    fn execute_skips_confirmation_and_is_not_dry_run() {
        let opts = RunOptions::execute();
        assert!(opts.skip_confirmation);
        assert!(!opts.dry_run);
    }

    #[test]
    fn execute_with_confirmation_prompts_and_is_not_dry_run() {
        let opts = RunOptions::execute_with_confirmation();
        assert!(!opts.skip_confirmation);
        assert!(!opts.dry_run);
    }

    #[test]
    fn preview_skips_confirmation_and_is_dry_run() {
        let opts = RunOptions::preview();
        assert!(opts.skip_confirmation);
        assert!(opts.dry_run);
    }
}
