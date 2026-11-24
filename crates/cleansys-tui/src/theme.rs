//! Colour themes for the TUI: every core preset (plus "System"), a picker popup helper.

use std::cell::RefCell;
use std::sync::{Arc, OnceLock};
use tui_nuances::Theme;

/// `tui-nuances` is the single source of truth: views read [`Theme`] fields directly via
/// [`th`] instead of hard-coding colours, so every theme restyles the whole screen.
///
/// The TUI offers **every** palette tui-nuances ships (not only the 43 that the GUI and web
/// share through `cleansys-core`). A theme keeps its cleansys name when it has one, so saved
/// settings and the other front-ends keep working; the rest are named from their slug.
#[derive(Debug, Clone)]
pub struct ThemeEntry {
    /// Shown in the picker and written to `settings.json` for themes the core also has.
    pub label: String,
    /// The tui-nuances name that builds the palette.
    pub key: &'static str,
    /// Index of the same-named preset in `cleansys_core::THEME_NAMES`, if the GUI/web have it.
    pub core: Option<usize>,
}

/// Core names that tui-nuances spells differently (they used to fall back to Default).
const ALIASES: &[(&str, &str)] = &[
    ("Gruvbox Dark", "gruvbox"),
    ("Catppuccin Frappé", "catppuccin-frappe"),
    ("Atom One Dark", "one-dark"),
];

fn norm(s: &str) -> String {
    s.chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

fn title_case(slug: &str) -> String {
    slug.split(['-', '_', ' '])
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut c = w.chars();
            c.next().map_or_else(String::new, |f| {
                f.to_uppercase().collect::<String>() + c.as_str()
            })
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Every selectable theme: `Default` first, then A–Z.
pub fn entries() -> &'static [ThemeEntry] {
    static ALL: OnceLock<Vec<ThemeEntry>> = OnceLock::new();
    ALL.get_or_init(|| {
        let core_idx = |name: &str| {
            cleansys_core::THEME_NAMES
                .iter()
                .position(|n| norm(n) == norm(name))
        };
        let mut v: Vec<ThemeEntry> = Theme::names()
            .into_iter()
            .map(|key| {
                let alias = ALIASES.iter().find(|(_, k)| *k == key).map(|(n, _)| *n);
                let core = alias.map_or_else(|| core_idx(key), core_idx);
                let label = core.map_or_else(
                    || title_case(key),
                    |i| cleansys_core::THEME_NAMES[i].to_string(),
                );
                ThemeEntry { label, key, core }
            })
            .collect();
        v.sort_by_key(|e| (norm(&e.label) != "default", e.label.to_lowercase()));
        v
    })
}

/// Number of themes (not counting the picker's leading "System").
pub fn count() -> usize {
    entries().len()
}

/// Theme by index into [`entries`] (out of range → Default).
pub fn by_index(index: usize) -> Arc<Theme> {
    static CACHE: OnceLock<Vec<Arc<Theme>>> = OnceLock::new();
    let all = CACHE.get_or_init(|| {
        entries()
            .iter()
            .map(|e| Arc::new(Theme::by_name(e.key).unwrap_or_default()))
            .collect()
    });
    Arc::clone(all.get(index).unwrap_or(&all[0]))
}

/// Index of the entry that matches a saved or typed name (label, slug or core name,
/// ignoring case, spaces and punctuation).
pub fn index_for_name(name: &str) -> Option<usize> {
    let n = norm(name);
    entries()
        .iter()
        .position(|e| norm(&e.label) == n || norm(e.key) == n)
}

/// Index of the entry for core preset `core_index` (what "follow system" resolves to).
pub fn index_of_core(core_index: usize) -> usize {
    entries()
        .iter()
        .position(|e| e.core == Some(core_index))
        .unwrap_or(0)
}

/// The name to write to `settings.json` for entry `i`: the shared core name when the
/// GUI/web have the preset too (they stay in sync), else the tui-nuances slug.
pub fn name_for_save(i: usize) -> String {
    let e = &entries()[i.min(count() - 1)];
    e.core.map_or_else(
        || e.key.to_string(),
        |c| cleansys_core::THEME_NAMES[c].to_string(),
    )
}

/// Entry selected by saved settings (`System` is handled by the caller).
pub fn index_for_settings(s: &cleansys_core::Settings) -> usize {
    s.theme_name
        .as_deref()
        .and_then(index_for_name)
        .unwrap_or_else(|| index_of_core(s.theme_index()))
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

/// Picker entries: `System` first, then every theme. Entry `0` is "follow system";
/// entry `n + 1` is theme `n`.
pub fn picker_len() -> usize {
    count() + 1
}

/// Display name of picker entry `i`.
pub fn picker_name(i: usize) -> &'static str {
    if i == 0 {
        cleansys_core::SYSTEM_THEME_NAME
    } else {
        entries()[(i - 1).min(count() - 1)].label.as_str()
    }
}

/// Find a theme by name for `--theme` (`System` or any theme, case-insensitive).
/// `Some(None)` = follow system, `Some(Some(i))` = theme `i`, `None` = unknown.
pub fn find(name: &str) -> Option<Option<usize>> {
    if cleansys_core::is_system_name(name) {
        return Some(None);
    }
    index_for_name(name).map(Some)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_theme_resolves_and_contrasts() {
        for i in 0..count() {
            let t = by_index(i);
            assert_ne!(bg(&t), t.foreground, "theme {i} ({})", entries()[i].label);
        }
    }

    #[test]
    fn the_tui_offers_everything_tui_nuances_ships() {
        assert_eq!(count(), Theme::names().len());
        assert!(
            count() > cleansys_core::THEME_COUNT,
            "more than the shared 43"
        );
        for key in Theme::names() {
            assert!(entries().iter().any(|e| e.key == key), "{key} missing");
        }
        // No two entries share a label (the picker would show duplicates).
        let mut labels: Vec<String> = entries().iter().map(|e| e.label.to_lowercase()).collect();
        labels.sort();
        labels.dedup();
        assert_eq!(labels.len(), count());
    }

    #[test]
    fn every_shared_core_theme_resolves_to_a_real_palette_not_default() {
        // Three core names used to fall back to Default silently (tui-nuances spells them
        // gruvbox / catppuccin-frappe / one-dark).
        for (i, name) in cleansys_core::THEME_NAMES.iter().enumerate() {
            let at = index_of_core(i);
            assert_eq!(entries()[at].core, Some(i), "{name} is offered");
            assert_eq!(entries()[at].label, *name, "keeps its cleansys name");
            if *name != "Default" {
                assert_ne!(
                    *by_index(at),
                    *by_index(0),
                    "{name} must not be the Default palette"
                );
            }
        }
    }

    #[test]
    fn default_comes_first_and_the_rest_are_alphabetical() {
        assert_eq!(entries()[0].label, "Default");
        let rest: Vec<String> = entries()[1..]
            .iter()
            .map(|e| e.label.to_lowercase())
            .collect();
        let mut sorted = rest.clone();
        sorted.sort();
        assert_eq!(rest, sorted);
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
    fn picker_leads_with_system_and_ends_with_the_last_theme() {
        assert_eq!(picker_len(), count() + 1);
        assert_eq!(picker_name(0), "System");
        assert_eq!(picker_name(1), "Default");
        assert_eq!(
            picker_name(picker_len() - 1),
            entries().last().unwrap().label
        );
    }

    #[test]
    fn names_resolve_by_label_slug_or_core_name_ignoring_case_and_punctuation() {
        let tokyo = index_for_name("tokyo night").unwrap();
        assert_eq!(index_for_name("Tokyo-Night"), Some(tokyo));
        assert_eq!(entries()[tokyo].label, "Tokyo Night");
        // Themes only tui-nuances has, by slug or by the label we show.
        let pip = index_for_name("pip-boy").expect("pip-boy is offered");
        assert_eq!(index_for_name("Pip Boy"), Some(pip));
        assert!(
            index_for_name("Gruvbox Dark").is_some(),
            "the old core name still works"
        );
        assert_eq!(index_for_name("nope"), None);
    }

    #[test]
    fn find_handles_system_themes_and_unknown() {
        assert_eq!(find("system"), Some(None));
        assert_eq!(
            find("tokyo night"),
            Some(Some(index_for_name("Tokyo Night").unwrap()))
        );
        assert!(matches!(find("pip-boy"), Some(Some(_))));
        assert_eq!(find("nope"), None);
    }

    #[test]
    fn saved_names_keep_the_gui_in_sync_for_shared_themes_only() {
        let tokyo = index_for_name("Tokyo Night").unwrap();
        assert_eq!(
            name_for_save(tokyo),
            "Tokyo Night",
            "core name: the GUI/web can read it"
        );
        let pip = index_for_name("pip-boy").unwrap();
        assert_eq!(
            name_for_save(pip),
            "pip-boy",
            "TUI-only theme saved by slug"
        );
        // Round trip through settings.
        let s = cleansys_core::Settings {
            theme_name: Some(name_for_save(pip)),
            ..Default::default()
        };
        assert_eq!(index_for_settings(&s), pip);
        let s = cleansys_core::Settings {
            theme_name: Some("Tokyo Night".into()),
            ..Default::default()
        };
        assert_eq!(index_for_settings(&s), tokyo);
        // No / unknown name: the core fallback (Default).
        assert_eq!(index_for_settings(&cleansys_core::Settings::default()), 0);
    }
}
