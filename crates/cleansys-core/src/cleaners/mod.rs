//! Cleaner modules for system and user-level cleanup operations.

/// Individual cleaned item / cleaning result data structures.
pub mod cleaned_item;

/// Per-platform (Linux/macOS/Windows) path resolution shared by all cleaners.
pub mod platform;

/// System-level cleaners that require root privileges.
pub mod system_cleaners;

/// User-level cleaners that work without elevated permissions.
pub mod user_cleaners;
