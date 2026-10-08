//! Window sizing: presets, keyboard shortcuts and the remembered size.
//!
//! macOS window-manager shortcuts don't always resize a winit window, so the app offers
//! its own: presets, grow/shrink, maximise, full screen — as shortcuts and in
//! Settings → Window.

use iced::keyboard::{key::Named, Key, Modifiers};

use crate::message::Message;

/// Smallest window the layouts support.
pub const MIN_SIZE: (f32, f32) = (420.0, 520.0);

/// Largest size the resizer will ask for (keeps a typo'd value from making a giant window).
pub const MAX_SIZE: (f32, f32) = (8000.0, 6000.0);

/// First-run size.
pub const DEFAULT_SIZE: (f32, f32) = (1180.0, 780.0);

/// Step of the grow / shrink shortcuts.
pub const GROW_FACTOR: f32 = 1.12;

/// Named window sizes, one per responsive layout (and a big one).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowPreset {
    /// Narrow layout: category drop-down, icon buttons.
    Compact,
    /// Medium layout: narrower sidebar.
    Medium,
    /// Wide layout: full sidebar.
    Wide,
    /// Roomy desktop window.
    Large,
}

impl WindowPreset {
    pub const ALL: [WindowPreset; 4] = [
        WindowPreset::Compact,
        WindowPreset::Medium,
        WindowPreset::Wide,
        WindowPreset::Large,
    ];

    /// Logical size in pixels.
    pub fn size(self) -> (f32, f32) {
        match self {
            WindowPreset::Compact => (460.0, 820.0),
            WindowPreset::Medium => (800.0, 720.0),
            WindowPreset::Wide => DEFAULT_SIZE,
            WindowPreset::Large => (1440.0, 900.0),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            WindowPreset::Compact => "Compact",
            WindowPreset::Medium => "Medium",
            WindowPreset::Wide => "Wide",
            WindowPreset::Large => "Large",
        }
    }

    /// `1`…`4` → the preset a ⌘/Ctrl+digit shortcut selects.
    pub fn from_digit(c: &str) -> Option<Self> {
        match c {
            "1" => Some(WindowPreset::Compact),
            "2" => Some(WindowPreset::Medium),
            "3" => Some(WindowPreset::Wide),
            "4" => Some(WindowPreset::Large),
            _ => None,
        }
    }

    /// The preset whose size is exactly `size`, if any.
    pub fn matching(size: (f32, f32)) -> Option<Self> {
        Self::ALL.into_iter().find(|p| {
            let (w, h) = p.size();
            (size.0 - w).abs() < 1.5 && (size.1 - h).abs() < 1.5
        })
    }
}

/// Keep a requested size inside what the layouts (and the screen) can use.
pub fn clamp_size((w, h): (f32, f32)) -> (f32, f32) {
    (
        w.clamp(MIN_SIZE.0, MAX_SIZE.0).round(),
        h.clamp(MIN_SIZE.1, MAX_SIZE.1).round(),
    )
}

/// `size` scaled by `factor`, clamped.
pub fn scaled(size: (f32, f32), factor: f32) -> (f32, f32) {
    clamp_size((size.0 * factor, size.1 * factor))
}

/// The window size to open with: `CLEANSYS_GUI_SIZE=WxH` > the saved size > the default.
pub fn initial_size(saved: Option<(u32, u32)>) -> (f32, f32) {
    let env = std::env::var("CLEANSYS_GUI_SIZE").ok().and_then(|v| {
        let (w, h) = v.split_once('x')?;
        Some((w.parse().ok()?, h.parse().ok()?))
    });
    let size = env
        .or_else(|| saved.map(|(w, h)| (w as f32, h as f32)))
        .unwrap_or(DEFAULT_SIZE);
    clamp_size(size)
}

/// Map a key press to a window action. `⌘` on macOS, `Ctrl` elsewhere:
///
/// * `⌘1`–`⌘4` presets · `⌘0` reset · `⌘=` / `⌘+` grow · `⌘-` shrink
/// * `⌃⌘F` (macOS) / `F11` full screen · `⌘⇧M` maximise
pub fn shortcut(key: &Key, mods: Modifiers) -> Option<Message> {
    if let Key::Named(Named::F11) = key {
        return Some(Message::ToggleFullscreen);
    }
    if !mods.command() {
        return None;
    }
    let Key::Character(c) = key else {
        return None;
    };
    let c = c.as_str();
    if let Some(p) = WindowPreset::from_digit(c) {
        return Some(Message::WindowPreset(p));
    }
    match c {
        "0" => Some(Message::WindowReset),
        "=" | "+" => Some(Message::WindowScale(GROW_FACTOR)),
        "-" | "_" => Some(Message::WindowScale(1.0 / GROW_FACTOR)),
        "f" | "F" if mods.control() => Some(Message::ToggleFullscreen),
        "m" | "M" if mods.shift() => Some(Message::ToggleMaximize),
        _ => None,
    }
}

/// `⌘` or `Ctrl+`, for hints.
pub fn modifier_label() -> &'static str {
    if cfg!(target_os = "macos") {
        "⌘"
    } else {
        "Ctrl+"
    }
}

/// One-line shortcut cheat sheet for the settings dialog.
pub fn shortcut_hint() -> String {
    let m = modifier_label();
    let fs = if cfg!(target_os = "macos") {
        "⌃⌘F".to_string()
    } else {
        "F11".to_string()
    };
    format!(
        "{m}1–4 presets · {m}0 reset · {m}+ / {m}− grow / shrink · {m}⇧M maximise · {fs} full screen"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ch(s: &str) -> Key {
        Key::Character(s.into())
    }

    fn cmd() -> Modifiers {
        if cfg!(target_os = "macos") {
            Modifiers::LOGO
        } else {
            Modifiers::CTRL
        }
    }

    #[test]
    fn presets_land_in_their_responsive_layouts() {
        // Breakpoints live in view.rs (WIDE 980, MEDIUM 700): each preset must hit its own.
        assert!(WindowPreset::Compact.size().0 < 700.0);
        assert!((700.0..980.0).contains(&WindowPreset::Medium.size().0));
        assert!(WindowPreset::Wide.size().0 >= 980.0);
        assert!(WindowPreset::Large.size().0 > WindowPreset::Wide.size().0);
        for p in WindowPreset::ALL {
            let (w, h) = p.size();
            assert!(
                w >= MIN_SIZE.0 && h >= MIN_SIZE.1,
                "{p:?} below the minimum"
            );
            assert_eq!(WindowPreset::matching((w, h)), Some(p));
        }
        assert_eq!(WindowPreset::matching((1000.0, 1000.0)), None);
    }

    #[test]
    fn sizes_are_clamped_and_scaled() {
        assert_eq!(clamp_size((10.0, 10.0)), MIN_SIZE);
        assert_eq!(clamp_size((99_999.0, 99_999.0)), MAX_SIZE);
        assert_eq!(clamp_size((800.4, 600.6)), (800.0, 601.0));
        let (w, h) = scaled((1000.0, 800.0), GROW_FACTOR);
        assert!(w > 1000.0 && h > 800.0);
        assert_eq!(
            scaled((430.0, 530.0), 0.5),
            MIN_SIZE,
            "shrinking stops at the minimum"
        );
    }

    #[test]
    fn initial_size_prefers_the_saved_size_and_sanitises_it() {
        // (CLEANSYS_GUI_SIZE is not set in unit tests.)
        assert_eq!(initial_size(None), DEFAULT_SIZE);
        assert_eq!(initial_size(Some((900, 700))), (900.0, 700.0));
        assert_eq!(initial_size(Some((10, 10))), MIN_SIZE);
    }

    #[test]
    fn shortcuts_map_to_window_actions() {
        assert!(matches!(
            shortcut(&ch("1"), cmd()),
            Some(Message::WindowPreset(WindowPreset::Compact))
        ));
        assert!(matches!(
            shortcut(&ch("3"), cmd()),
            Some(Message::WindowPreset(WindowPreset::Wide))
        ));
        assert!(matches!(
            shortcut(&ch("0"), cmd()),
            Some(Message::WindowReset)
        ));
        assert!(matches!(
            shortcut(&ch("="), cmd()),
            Some(Message::WindowScale(f)) if f > 1.0
        ));
        assert!(matches!(
            shortcut(&ch("-"), cmd()),
            Some(Message::WindowScale(f)) if f < 1.0
        ));
        assert!(matches!(
            shortcut(&ch("m"), cmd() | Modifiers::SHIFT),
            Some(Message::ToggleMaximize)
        ));
        assert!(matches!(
            shortcut(&Key::Named(Named::F11), Modifiers::empty()),
            Some(Message::ToggleFullscreen)
        ));
        assert!(matches!(
            shortcut(&ch("f"), cmd() | Modifiers::CTRL),
            Some(Message::ToggleFullscreen)
        ));
    }

    #[test]
    fn plain_keys_and_unknown_shortcuts_are_ignored() {
        // Typing digits into the search box must never resize the window.
        assert!(shortcut(&ch("1"), Modifiers::empty()).is_none());
        assert!(shortcut(&ch("="), Modifiers::SHIFT).is_none());
        assert!(shortcut(&ch("x"), cmd()).is_none());
        assert!(shortcut(&ch("m"), cmd()).is_none(), "maximise needs Shift");
        assert!(shortcut(&Key::Named(Named::Enter), cmd()).is_none());
    }

    #[test]
    fn hint_mentions_every_shortcut() {
        let h = shortcut_hint();
        for needle in ["1–4", "reset", "grow", "maximise", "full screen"] {
            assert!(h.contains(needle), "{h}");
        }
    }
}
