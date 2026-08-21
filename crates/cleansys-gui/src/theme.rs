//! Theme helpers for CleanSys's GUI.
//!
//! Colours are derived from the unified `cleansys_core::AppTheme` definitions
//! so the GUI renders exactly the same 43-theme catalogue as the rest of the
//! CleanSys ecosystem (and, in spirit, the same catalogue as GitKraft).

use cleansys_core::AppTheme;
use iced::{Color, Theme};

/// A resolved set of colours derived from the active theme, for the small
/// number of custom-styled elements in the GUI (badges, cards, status text).
///
/// Everything else (buttons, checkboxes, text inputs, pick-lists, scrollbars)
/// is themed automatically via [`iced_theme`], which builds a custom
/// `iced::Theme::Palette` from the same core theme.
#[derive(Debug, Clone, Copy)]
pub struct ThemeColors {
    pub accent: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub muted: Color,
    pub bg: Color,
    pub surface: Color,
    pub surface_highlight: Color,
    pub header_bg: Color,
    pub border: Color,
    pub selection: Color,
    pub green: Color,
    pub red: Color,
    pub yellow: Color,
}

fn clamp(v: f32) -> f32 {
    v.clamp(0.0, 1.0)
}

/// Shift every RGB channel of `base` by `delta` (positive = lighter, negative = darker).
fn shift(base: Color, delta: f32) -> Color {
    Color {
        r: clamp(base.r + delta),
        g: clamp(base.g + delta),
        b: clamp(base.b + delta),
        a: base.a,
    }
}

/// Convert a core [`cleansys_core::Rgb`] to an [`iced::Color`].
fn rgb_to_iced(rgb: cleansys_core::Rgb) -> Color {
    Color::from_rgb8(rgb.r, rgb.g, rgb.b)
}

impl ThemeColors {
    /// Build a complete GUI colour set from the core's platform-agnostic theme.
    pub fn from_core(t: &AppTheme) -> Self {
        let bg = rgb_to_iced(t.background);
        let surface = rgb_to_iced(t.surface);

        let sign: f32 = if t.is_dark { 1.0 } else { -1.0 };
        let surface_highlight = shift(surface, sign * 0.04);
        let header_bg = shift(bg, sign * 0.02);

        Self {
            accent: rgb_to_iced(t.accent),
            text_primary: rgb_to_iced(t.text_primary),
            text_secondary: rgb_to_iced(t.text_secondary),
            muted: rgb_to_iced(t.text_muted),
            bg,
            surface,
            surface_highlight,
            header_bg,
            border: rgb_to_iced(t.border),
            selection: rgb_to_iced(t.selection),
            green: rgb_to_iced(t.success),
            red: rgb_to_iced(t.error),
            yellow: rgb_to_iced(t.warning),
        }
    }
}

/// Build a custom `iced::Theme` whose palette is derived from the active core
/// theme, so every built-in Iced widget (checkboxes, buttons, text inputs,
/// pick-lists, scrollbars) automatically inherits the right colours without
/// each one needing an explicit `.style()` override.
pub fn iced_theme_for(index: usize) -> Theme {
    let core = cleansys_core::theme_by_index(index);
    let name = cleansys_core::THEME_NAMES
        .get(index)
        .copied()
        .unwrap_or("Default")
        .to_string();

    let palette = iced::theme::Palette {
        background: rgb_to_iced(core.background),
        text: rgb_to_iced(core.text_primary),
        primary: rgb_to_iced(core.accent),
        success: rgb_to_iced(core.success),
        warning: rgb_to_iced(core.warning),
        danger: rgb_to_iced(core.error),
    };

    Theme::custom(name, palette)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_core_produces_valid_colors() {
        for i in 0..cleansys_core::THEME_COUNT {
            let core = cleansys_core::theme_by_index(i);
            let colors = ThemeColors::from_core(&core);
            // Just make sure nothing panics and alpha stays sane.
            assert!(colors.bg.a > 0.0);
            assert!(colors.text_primary.a > 0.0);
        }
    }

    #[test]
    fn iced_theme_for_all_indices_does_not_panic() {
        for i in 0..cleansys_core::THEME_COUNT {
            let _ = iced_theme_for(i);
        }
    }

    #[test]
    fn iced_theme_for_out_of_range_falls_back_to_default_name() {
        // theme_by_index falls back to default() for out-of-range, and the
        // name lookup falls back to "Default" too.
        let theme = iced_theme_for(9999);
        assert_eq!(format!("{theme}"), "Default");
    }
}
