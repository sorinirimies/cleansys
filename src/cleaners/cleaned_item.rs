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
    pub fn new(path: PathBuf, size: u64, item_type: CleanedItemType) -> Self {
        Self {
            path,
            size,
            item_type,
        }
    }

    /// Create a file item
    pub fn file(path: PathBuf, size: u64) -> Self {
        Self::new(path, size, CleanedItemType::File)
    }

    /// Create a directory item
    pub fn directory(path: PathBuf, size: u64) -> Self {
        Self::new(path, size, CleanedItemType::Directory)
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
