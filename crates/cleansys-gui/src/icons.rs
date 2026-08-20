//! Named char constants for the Bootstrap Icons font glyphs used in the GUI.
//!
//! Using named constants instead of raw unicode literals documents intent and
//! avoids silent codepoint typos. Render with `.font(icons::FONT)`.

use iced::Font;

/// The Bootstrap Icons font, loaded via `iced_fonts::BOOTSTRAP_FONT_BYTES` in `main.rs`.
pub const FONT: Font = Font::with_name("bootstrap-icons");

/// `check-circle-fill` — successful operation.
pub const CHECK_CIRCLE_FILL: char = '\u{f26a}';
/// `x-circle` — failed operation.
pub const X_CIRCLE: char = '\u{f623}';
/// `arrow-repeat` — in-progress / running spinner glyph.
pub const ARROW_REPEAT: char = '\u{f130}';
/// `clock` — queued / pending.
pub const CLOCK: char = '\u{f293}';
/// `shield-lock` (approximated with `exclamation-triangle`) — requires root.
pub const EXCLAMATION_TRIANGLE: char = '\u{f33b}';
