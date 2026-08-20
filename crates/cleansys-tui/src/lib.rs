//! # cleansys-tui
//!
//! Ratatui-based terminal user interface and CLI for CleanSys.
//!
//! Shared domain logic (cleaners, permission checks, formatting) lives in
//! [`cleansys_core`]; this crate is only responsible for rendering and
//! terminal input handling.

#![allow(missing_docs)]

/// Application state and key-handling logic for the TUI.
pub mod app;

/// Reusable UI components (e.g. the sudo password prompt).
pub mod components;

/// Event handling for terminal input and resize events.
pub mod events;

/// Text-based interactive menu (non-TUI fallback interface).
pub mod menu;

/// Pie chart widget for data visualization.
pub mod pie_chart;

/// Rendering logic for the terminal UI.
pub mod render;

/// Re-export commonly used types for convenience.
pub use app::App;
pub use components::password_prompt::PasswordPrompt;
pub use menu::Menu;
