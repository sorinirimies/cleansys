//! Colour themes for the TUI: every core preset (plus "System"), a picker popup helper.

use std::cell::RefCell;
use std::sync::{Arc, OnceLock};
use tui_nuances::Theme;

/// `tui-nuances` is the single source of truth: views read [`Theme`] fields directly via
/// [`th`] instead of hard-coding colours, so every theme restyles the whole screen.
///
/// Theme by index into [`cleansys_core::THEME_NAMES`] (out of range → Default).
/// Colours come from the `tui-nuances` theme of the same name.
pub fn by_index(index: usize) -> Arc<Theme> {
    static CACHE: OnceLock<Vec<Arc<Theme>>> = OnceLock::new();
    let all = CACHE.get_or_init(|| {
        cleansys_core::THEME_NAMES
            .iter()
            .map(|n| Arc::new(Theme::by_name(n).unwrap_or_default()))
            .collect()
    });
    Arc::clone(all.get(index).unwrap_or(&all[0]))
}

thread_local! {
    static CURRENT: RefCell<Arc<Theme>> = RefCell::new(by_index(0));
}

/// Make `t` the theme [`th`] returns (called once per frame by `render::ui`).
pub fn set_current(t: Arc<Theme>) {
    CURRENT.with(|c| *c.borrow_mut() = t);
}

/// The theme of the frame being drawn.
pub fn th() -> Arc<Theme> {
    CURRENT.with(|c| Arc::clone(&c.borrow()))
}

/// The terminal background colour of `t` (`Reset` for terminal-native themes).
pub fn bg(t: &Theme) -> ratatui::style::Color {
    t.background.unwrap_or_default()
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
    fn every_theme_resolves_and_contrasts() {
        for i in 0..cleansys_core::THEME_COUNT {
            let t = by_index(i);
            assert_ne!(bg(&t), t.foreground, "theme {i}");
        }
    }

    #[test]
    fn out_of_range_falls_back_to_default() {
        assert_eq!(*by_index(9999), *by_index(0));
    }

    #[test]
    fn current_theme_is_settable() {
        let t = by_index(9);
        set_current(Arc::clone(&t));
        assert_eq!(*th(), *t);
        set_current(by_index(0));
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
