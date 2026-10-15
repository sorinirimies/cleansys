//! Colour themes for the TUI: every core preset (plus "System"), a picker popup helper.

use ratatui::style::Color;
use std::cell::Cell;

use cleansys_core::{AppTheme, Rgb};

/// Every colour slot the UI needs, as ratatui colours. Views read these via [`th`] instead
/// of hard-coding `Color::*`, so every theme restyles the whole screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiTheme {
    pub bg: Color,
    pub surface: Color,
    pub border: Color,
    pub selection: Color,
    pub text: Color,
    pub text_secondary: Color,
    pub muted: Color,
    pub accent: Color,
    pub success: Color,
    /// Brighter success colour (the sweeping highlight on progress bars).
    pub success_hi: Color,
    pub warning: Color,
    pub error: Color,
}

/// Blend 45% towards white.
fn lighten(c: Rgb) -> Rgb {
    let f = |v: u8| (v as u16 + (255 - v as u16) * 45 / 100) as u8;
    Rgb::new(f(c.r), f(c.g), f(c.b))
}

fn rgb(c: Rgb) -> Color {
    Color::Rgb(c.r, c.g, c.b)
}

impl UiTheme {
    /// Convert from the core's platform-agnostic theme.
    pub fn from_core(t: &AppTheme) -> Self {
        Self {
            bg: rgb(t.background),
            surface: rgb(t.surface),
            border: rgb(t.border),
            selection: rgb(t.selection),
            text: rgb(t.text_primary),
            text_secondary: rgb(t.text_secondary),
            muted: rgb(t.text_muted),
            accent: rgb(t.accent),
            success: rgb(t.success),
            success_hi: rgb(lighten(t.success)),
            warning: rgb(t.warning),
            error: rgb(t.error),
        }
    }

    /// Theme by index into [`cleansys_core::THEME_NAMES`] (out of range → Default).
    pub fn by_index(index: usize) -> Self {
        Self::from_core(&cleansys_core::theme_by_index(index))
    }
}

impl Default for UiTheme {
    fn default() -> Self {
        Self::by_index(0)
    }
}

thread_local! {
    static CURRENT: Cell<UiTheme> = Cell::new(UiTheme::by_index(0));
}

/// Make `t` the theme [`th`] returns (called once per frame by `render::ui`).
pub fn set_current(t: UiTheme) {
    CURRENT.with(|c| c.set(t));
}

/// The theme of the frame being drawn.
pub fn th() -> UiTheme {
    CURRENT.with(Cell::get)
}

/// Picker entries: `System` first, then every preset. Entry `0` is "follow system";
/// entry `n + 1` is preset `n`.
pub const PICKER_LEN: usize = cleansys_core::THEME_COUNT + 1;

/// Display name of picker entry `i`.
pub fn picker_name(i: usize) -> &'static str {
    if i == 0 {
        cleansys_core::SYSTEM_THEME_NAME
    } else {
        cleansys_core::THEME_NAMES[i - 1]
    }
}

/// Find a theme by name for `--theme` (`System` or a preset, case-insensitive).
/// `Some(None)` = follow system, `Some(Some(i))` = preset `i`, `None` = unknown.
pub fn find(name: &str) -> Option<Option<usize>> {
    if cleansys_core::is_system_name(name) {
        return Some(None);
    }
    cleansys_core::THEME_NAMES
        .iter()
        .position(|n| n.eq_ignore_ascii_case(name))
        .map(Some)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_theme_resolves_to_truecolor_and_contrasts() {
        for i in 0..cleansys_core::THEME_COUNT {
            let t = UiTheme::by_index(i);
            assert!(matches!(t.bg, Color::Rgb(..)) && matches!(t.accent, Color::Rgb(..)));
            assert_ne!(t.bg, t.text, "theme {i}");
        }
    }

    #[test]
    fn out_of_range_falls_back_to_default() {
        assert_eq!(UiTheme::by_index(9999), UiTheme::by_index(0));
    }

    #[test]
    fn current_theme_is_settable() {
        let t = UiTheme::by_index(9);
        set_current(t);
        assert_eq!(th(), t);
        set_current(UiTheme::default());
    }

    #[test]
    fn picker_leads_with_system() {
        assert_eq!(PICKER_LEN, cleansys_core::THEME_COUNT + 1);
        assert_eq!(picker_name(0), "System");
        assert_eq!(picker_name(1), cleansys_core::THEME_NAMES[0]);
        assert_eq!(
            picker_name(PICKER_LEN - 1),
            *cleansys_core::THEME_NAMES.last().unwrap()
        );
    }

    #[test]
    fn find_handles_system_presets_and_unknown() {
        assert_eq!(find("system"), Some(None));
        assert_eq!(
            find("tokyo night"),
            Some(Some(cleansys_core::theme_index_by_name("Tokyo Night")))
        );
        assert_eq!(find("nope"), None);
    }
}
