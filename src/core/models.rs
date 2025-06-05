use anyhow::Result;

/// Represents the status of a cleaner operation
#[derive(Debug, Clone, PartialEq)]
pub enum CleanerStatus {
    /// The cleaner is currently running
    Running,
    /// The cleaner completed successfully
    Success,
    /// The cleaner encountered an error
    Error,
    /// The cleaner is waiting to be run
    Pending,
}

/// Represents a single cleaner operation
#[derive(Debug, Clone)]
pub struct CleanerItem {
    /// Name of the cleaner
    pub name: String,
    /// Description of what the cleaner does
    pub description: String,
    /// Whether the cleaner requires root privileges
    pub requires_root: bool,
    /// Whether the cleaner is selected for execution
    pub selected: bool,
    /// Function to execute the cleaning operation
    pub function: fn(bool) -> Result<u64>,
    /// Number of bytes cleaned by this operation
    pub bytes_cleaned: u64,
    /// Current status of the cleaner
    pub status: CleanerStatus,
}

/// Represents a category of cleaners
#[derive(Debug, Clone)]
pub struct CleanerCategory {
    /// Name of the category
    pub name: String,
    /// Description of the category
    pub description: String,
    /// List of cleaner items in this category
    pub items: Vec<CleanerItem>,
}

/// Represents the view mode of the application
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewMode {
    /// View showing user land cleaners
    UserLand,
    /// View showing root cleaners
    Root,
}

/// Represents a cleaning result message
#[derive(Debug, Clone)]
pub struct CleanerMessage {
    /// Name of the cleaner that generated the message
    pub cleaner_name: String,
    /// Content of the message
    pub message: String,
    /// Whether the message represents an error
    pub is_error: bool,
    /// Number of bytes cleaned (if successful)
    pub bytes_cleaned: Option<u64>,
}