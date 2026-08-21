//! # cleansys-core
//!
//! Shared, framework-free logic reused by both the `cleansys-tui` (Ratatui)
//! and `cleansys-gui` (Iced) front-ends of CleanSys.
//!
//! | Module | What lives here |
//! |--------|-----------------|
//! | [`cleaners`] | System- and user-level cleaner implementations |
//! | [`model`] | UI-agnostic domain model (`CleanerItem`, `CleanerCategory`, `Status`) |
//! | [`auth`] | Sudo/root authentication helpers |
//! | [`utils`] | Permission checks, formatting, confirmation prompts |
//!
//! This crate has NO TUI or GUI dependencies.

#![allow(missing_docs)]

/// Authentication helpers (sudo elevation) shared by all front-ends.
pub mod auth;

/// Cleaner implementations for system and user-level cleanup operations.
pub mod cleaners;

/// Framework-agnostic domain model (cleaner items, categories, run status).
pub mod model;

/// Persisted user preferences (theme, etc.), shared by the TUI and GUI.
pub mod settings;

/// Platform-agnostic UI theme presets, shared by the TUI and GUI.
pub mod theme;

/// Utility functions for permissions, formatting, and error handling.
pub mod utils;

// Convenience re-exports
pub use auth::authenticate_sudo;
pub use cleaners::cleaned_item::{CleanedItem, CleanedItemType, CleanerFn, CleaningResult};
pub use cleaners::{system_cleaners, user_cleaners};
pub use model::{load_categories, CleanerCategory, CleanerItem, Status};
pub use settings::{load_settings, save_settings, Settings};
pub use theme::{theme_by_index, theme_index_by_name, AppTheme, Rgb, THEME_COUNT, THEME_NAMES};
pub use utils::{check_root, confirm, format_size, get_size, print_error, print_header};
