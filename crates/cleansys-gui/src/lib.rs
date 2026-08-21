//! # cleansys-gui
//!
//! Iced-based desktop GUI for CleanSys.
//!
//! Domain logic (cleaners, permission checks, formatting) lives in
//! [`cleansys_core`]; this crate only owns presentation state and rendering.

#![allow(missing_docs)]

/// Message type describing every user interaction and async result.
pub mod message;

/// Application state.
pub mod state;

/// Update (Elm-style) logic.
pub mod update;

/// View (rendering) logic.
pub mod view;

/// Bootstrap icon font glyph constants.
pub mod icons;

/// Theme colour derivation (`ThemeColors::from_core`, custom `iced::Theme`).
pub mod theme;

/// Theme picker widget (`pick_list` of all `cleansys-core` themes).
pub mod theme_selector;

pub use message::Message;
pub use state::CleanSysGui;
pub use update::update;
pub use view::view;
