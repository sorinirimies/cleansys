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
    /// Index of the same-named theme in `cleansys_core::theme_names()` (GUI/web share it).
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

/// Index into the *curated* core themes that `key` is the same theme as, if any.
fn curated_index(key: &str) -> Option<usize> {
    let want = ALIASES
        .iter()
        .find(|(_, k)| *k == key)
        .map_or(key, |(n, _)| *n);
    cleansys_core::CURATED_THEME_NAMES
        .iter()
        .position(|n| norm(n) == norm(want))
}

/// The name a tui-nuances theme goes by everywhere: the curated cleansys name when it has
/// one, else its slug in Title Case. The generated core table uses the same names.
fn label_for(key: &str) -> String {
    curated_index(key).map_or_else(
        || title_case(key),
        |i| cleansys_core::CURATED_THEME_NAMES[i].to_string(),
    )
}

/// Every selectable theme: `Default` first, then A–Z.
pub fn entries() -> &'static [ThemeEntry] {
    static ALL: OnceLock<Vec<ThemeEntry>> = OnceLock::new();
    ALL.get_or_init(|| {
        let mut v: Vec<ThemeEntry> = Theme::names()
            .into_iter()
            .map(|key| {
                let label = label_for(key);
                let core = cleansys_core::theme_names()
                    .iter()
                    .position(|n| *n == label);
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

/// The name to write to `settings.json` for entry `i`: the shared core name, so the TUI,
/// GUI and web stay in sync (the slug only if the generated core table is stale).
pub fn name_for_save(i: usize) -> String {
    let e = &entries()[i.min(count() - 1)];
    e.core.map_or_else(
        || e.key.to_string(),
        |c| cleansys_core::theme_names()[c].to_string(),
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

/// Colour slots of a tui-nuances theme as core's `AppTheme` wants them.
#[cfg(test)]
fn to_core(t: &Theme) -> (bool, [u32; 15], [u32; 8]) {
    use ratatui::style::Color;
    use tui_nuances::color::{mix, relative_luminance, to_rgb};
    let hex = |c: Color| -> u32 {
        let (r, g, b) = to_rgb(c).unwrap_or((0, 0, 0));
        (u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b)
    };
    // `None` background = the terminal's own; pick black/white from the text colour.
    let bg = t.background.unwrap_or_else(|| {
        if relative_luminance(t.foreground) > 0.5 {
            Color::Rgb(0, 0, 0)
        } else {
            Color::Rgb(255, 255, 255)
        }
    });
    let dark = relative_luminance(bg) < 0.5;
    let surface = t.surface.unwrap_or(bg);
    let colors = [
        hex(bg),
        hex(surface),
        hex(t.border),
        hex(t.selection_bg),
        hex(t.foreground),
        hex(mix(t.foreground, t.muted, 0.5)),
        hex(t.muted),
        hex(t.accent),
        hex(t.success),
        hex(t.warning),
        hex(t.error),
        hex(t.success),
        hex(t.error),
        hex(t.muted),
        hex(t.info),
    ];
    let mut graph = [0u32; 8];
    for (i, g) in graph.iter_mut().enumerate() {
        *g = hex(t.palette[i % t.palette.len().max(1)]);
    }
    (dark, colors, graph)
}

/// The text of `cleansys-core/src/theme/extra.rs`: every tui-nuances theme that is not one
/// of the curated ones, A–Z.
#[cfg(test)]
fn extra_table_source() -> String {
    let mut rows: Vec<(String, Theme)> = Theme::names()
        .into_iter()
        .filter(|k| curated_index(k).is_none())
        .map(|k| (label_for(k), Theme::by_name(k).unwrap_or_default()))
        .collect();
    rows.sort_by_key(|(l, _)| l.to_lowercase());
    let mut out = String::from(
        "//! GENERATED by `UPDATE_THEMES=1 cargo test -p cleansys theme_tables`. Do not edit.\n\n\
         pub(super) struct Extra {\n    pub name: &'static str,\n    pub dark: bool,\n    \
         pub colors: [u32; 15],\n    pub graph: [u32; 8],\n}\n\n\
         pub(super) const EXTRA: &[Extra] = &[\n",
    );
    let arr = |v: &[u32]| {
        v.iter()
            .map(|c| format!("0x{c:06X}"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    for (label, t) in &rows {
        let (dark, colors, graph) = to_core(t);
        out.push_str(&format!(
            "    Extra {{\n        name: {label:?},\n        dark: {dark},\n        colors: [{}],\n        graph: [{}],\n    }},\n",
            arr(&colors),
            arr(&graph)
        ));
    }
    out.push_str("];\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Regenerate with `UPDATE_THEMES=1 cargo test -p cleansys theme_tables` after a
    /// `tui-nuances` upgrade; otherwise this fails when the two have drifted apart.
    #[test]
    fn theme_tables_core_extras_match_tui_nuances() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../cleansys-core/src/theme/extra.rs"
        );
        let want = extra_table_source();
        if std::env::var_os("UPDATE_THEMES").is_some() {
            std::fs::write(path, &want).unwrap();
            return;
        }
        // Line endings are not content: a Windows checkout may have converted them.
        let have = std::fs::read_to_string(path).unwrap().replace("\r\n", "\n");
        assert!(
            have == want,
            "core's generated theme table is stale: run `UPDATE_THEMES=1 cargo test -p cleansys theme_tables`"
        );
    }

    #[test]
    fn every_theme_in_the_tui_is_also_in_core_by_the_same_name() {
        for e in entries() {
            let c = e
                .core
                .unwrap_or_else(|| panic!("{} is missing from core", e.label));
            assert_eq!(cleansys_core::theme_names()[c], e.label);
        }
        assert_eq!(cleansys_core::theme_count(), count());
    }

    #[test]
    fn generated_extras_keep_text_readable_on_their_background() {
        let lum = |c: cleansys_core::Rgb| {
            let f = |v: u8| {
                let v = f64::from(v) / 255.0;
                if v <= 0.03928 {
                    v / 12.92
                } else {
                    ((v + 0.055) / 1.055).powf(2.4)
                }
            };
            0.2126 * f(c.r) + 0.7152 * f(c.g) + 0.0722 * f(c.b)
        };
        let ratio = |a, b| {
            let (x, y) = (lum(a), lum(b));
            (x.max(y) + 0.05) / (x.min(y) + 0.05)
        };
        let mut bad = Vec::new();
        for i in cleansys_core::CURATED_THEME_COUNT..cleansys_core::theme_count() {
            let t = cleansys_core::theme_by_index(i);
            let name = cleansys_core::theme_names()[i];
            if ratio(t.text_primary, t.background) < 4.5 {
                bad.push(format!("{name}: body text"));
            }
            if ratio(t.accent, t.background) < 2.0 {
                bad.push(format!("{name}: accent"));
            }
            if ratio(t.text_muted, t.background) < 1.5 {
                bad.push(format!("{name}: hints"));
            }
            // the dark flag agrees with the actual background
            assert_eq!(t.is_dark, lum(t.background) < 0.5, "{name}");
        }
        assert!(bad.is_empty(), "unreadable generated themes: {bad:?}");
    }

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
            count() > cleansys_core::CURATED_THEME_COUNT,
            "more than the curated 43"
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
        for (i, name) in cleansys_core::CURATED_THEME_NAMES.iter().enumerate() {
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
    fn saved_names_are_the_shared_core_names() {
        let tokyo = index_for_name("Tokyo Night").unwrap();
        assert_eq!(name_for_save(tokyo), "Tokyo Night");
        let pip = index_for_name("pip-boy").unwrap();
        assert_eq!(
            name_for_save(pip),
            "Pip Boy",
            "every theme is saved by its shared name"
        );
        // Round trip through settings, by shared name and by the old slug.
        for saved in [name_for_save(pip), "pip-boy".to_string()] {
            let s = cleansys_core::Settings {
                theme_name: Some(saved),
                ..Default::default()
            };
            assert_eq!(index_for_settings(&s), pip);
        }
        let s = cleansys_core::Settings {
            theme_name: Some("Tokyo Night".into()),
            ..Default::default()
        };
        assert_eq!(index_for_settings(&s), tokyo);
        // No / unknown name: the core fallback (Default).
        assert_eq!(index_for_settings(&cleansys_core::Settings::default()), 0);
    }
}
