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

/// Window presets, resize shortcuts and the remembered size.
pub mod window;

/// Desktop notifications and (Windows-only) elevated relaunch helper.
pub mod platform;

pub use message::Message;
pub use state::CleanSysGui;
pub use update::update;
pub use view::view;

/// Keyboard shortcuts and window resize events.
pub fn subscription(_state: &CleanSysGui) -> iced::Subscription<Message> {
    use iced::{event, keyboard, window as win, Event};
    event::listen_with(|event, _status, _id| match event {
        Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => {
            window::shortcut(&key, modifiers)
        }
        Event::Window(win::Event::Resized(size)) => {
            Some(Message::WindowResized(size.width, size.height))
        }
        _ => None,
    })
}
