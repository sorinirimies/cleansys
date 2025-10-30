//! Cleaner modules for system and user-level cleanup operations.

/// System-level cleaners that require root privileges.
pub mod system_cleaners;

/// User-level cleaners that work without elevated permissions.
pub mod user_cleaners;
