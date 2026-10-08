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

/// macOS: the standard Window menu (Zoom, Move & Resize / tiling).
#[cfg(target_os = "macos")]
pub mod macos;

/// Window presets, resize shortcuts and the remembered size.
pub mod window;

/// Desktop notifications and (Windows-only) elevated relaunch helper.
pub mod platform;

pub use message::Message;
pub use state::CleanSysGui;
pub use update::update;
pub use view::view;

/// Keyboard shortcuts, window resize events and the progress animation tick.
pub fn subscription(state: &CleanSysGui) -> iced::Subscription<Message> {
    use iced::{event, keyboard, window as win, Event};
    let events = event::listen_with(|event, _status, _id| match event {
        Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => {
            window::shortcut(&key, modifiers)
        }
        Event::Window(win::Event::Resized(size)) => {
            Some(Message::WindowResized(size.width, size.height))
        }
        _ => None,
    });
    // The spinner and the bar highlight only tick while something is in progress.
    if state.is_animating() {
        iced::Subscription::batch([events, iced::Subscription::run(animation_stream)])
    } else {
        events
    }
}

/// Interval of the animation tick.
const ANIMATION_STEP: std::time::Duration = std::time::Duration::from_millis(50);

/// Emits [`Message::AnimationTick`] every [`ANIMATION_STEP`]. The blocking sleep is fine:
/// each subscription runs on its own worker.
fn animation_stream() -> impl iced::futures::Stream<Item = Message> {
    iced::futures::stream::unfold((), |()| async {
        std::thread::sleep(ANIMATION_STEP);
        Some((Message::AnimationTick, ()))
    })
}
