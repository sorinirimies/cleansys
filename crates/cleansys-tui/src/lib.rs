//! # cleansys
//!
//! Ratatui-based terminal user interface and CLI for CleanSys. Published to
//! crates.io as `cleansys` (crate directory: `cleansys-tui`).
//!
//! Ratatui-based terminal user interface and CLI for CleanSys.
//!
//! Shared domain logic (cleaners, permission checks, formatting) lives in
//! [`cleansys_core`]; this crate is only responsible for rendering and
//! terminal input handling.

#![allow(missing_docs)]

/// Application state and key-handling logic for the TUI.
pub mod app;

/// Graphical dashboard, per-cleaner detail and clean report (tui-chart / tui-piechart).
pub mod charts;

/// Reusable UI components (e.g. the sudo password prompt).
pub mod components;

/// Event handling for terminal input and resize events.
pub mod events;

/// Text-based interactive menu (non-TUI fallback interface).
pub mod menu;

pub mod run;

/// Colour themes (every core preset plus "follow system").
pub mod theme;

/// Rendering logic for the terminal UI.
pub mod render;

/// Desktop notification helper (best-effort; failures never surface to the UI).
pub mod notifications;

/// Re-export commonly used types for convenience.
pub use app::App;
pub use components::password_prompt::PasswordPrompt;
pub use menu::Menu;
