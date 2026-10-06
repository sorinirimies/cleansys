//! Colour themes: the core's palettes as CSS custom properties.
//!
//! `/style.css` is written against `var(--bg)`, `var(--accent)` … with a dark default;
//! a `<style>` block from [`css`] overrides those variables.

use cleansys_core::{Rgb, THEME_COUNT, THEME_NAMES, theme_by_index, theme_index_by_name};

fn hex(c: Rgb) -> String {
    format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b)
}

/// CSS custom properties for the theme at `index` (clamped), as one `:root{…}` rule.
pub fn css(index: usize) -> String {
    let t = theme_by_index(index.min(THEME_COUNT - 1));
    let vars = [
        ("bg", t.background),
        ("panel", t.surface),
        ("line", t.border),
        ("sel", t.selection),
        ("fg", t.text_primary),
        ("fg2", t.text_secondary),
        ("dim", t.text_muted),
        ("accent", t.accent),
        ("ok", t.success),
        ("warn", t.warning),
        ("err", t.error),
    ];
    let body: String = vars
        .iter()
        .map(|(k, c)| format!("--{k}:{};", hex(*c)))
        .collect();
    format!(
        ":root{{color-scheme:{};{body}}}",
        if t.is_dark { "dark" } else { "light" }
    )
}

/// Pick the theme: an explicit, valid `requested` name wins, otherwise `saved`.
pub fn resolve(requested: Option<&str>, saved: usize) -> usize {
    match requested {
        Some(n) if THEME_NAMES.contains(&n) => theme_index_by_name(n),
        _ => saved,
    }
}

pub fn names() -> &'static [&'static str] {
    THEME_NAMES
}

/// Every theme with its colours, for `/api/themes`.
pub fn catalogue() -> serde_json::Value {
    (0..THEME_COUNT)
        .map(|i| {
            let t = theme_by_index(i);
            serde_json::json!({
                "name": THEME_NAMES[i],
                "dark": t.is_dark,
                "background": hex(t.background),
                "surface": hex(t.surface),
                "border": hex(t.border),
                "selection": hex(t.selection),
                "text": hex(t.text_primary),
                "text_secondary": hex(t.text_secondary),
                "text_muted": hex(t.text_muted),
                "accent": hex(t.accent),
                "success": hex(t.success),
                "warning": hex(t.warning),
                "error": hex(t.error),
            })
        })
        .collect::<Vec<_>>()
        .into()
}

/// The saved theme (shared with the GUI via `settings.json`).
pub fn saved_index() -> usize {
    cleansys_core::load_settings()
        .map(|s| s.theme_index())
        .unwrap_or(0)
}

/// Remember `index` for the GUI and web (best effort).
pub fn save_index(index: usize) {
    if let Ok(mut s) = cleansys_core::load_settings() {
        s.theme_name = Some(THEME_NAMES[index.min(THEME_COUNT - 1)].to_string());
        let _ = cleansys_core::save_settings(&s);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_theme_yields_all_variables() {
        for i in 0..THEME_COUNT {
            let c = css(i);
            for v in ["--bg:", "--panel:", "--accent:", "--ok:", "--err:"] {
                assert!(c.contains(v), "{i}: {c}");
            }
        }
    }

    #[test]
    fn explicit_valid_theme_wins_unknown_is_ignored() {
        assert_eq!(resolve(Some(THEME_NAMES[3]), 7), 3);
        assert_eq!(resolve(Some("nope"), 7), 7);
        assert_eq!(resolve(None, 7), 7);
    }

    #[test]
    fn catalogue_lists_every_theme() {
        assert_eq!(catalogue().as_array().unwrap().len(), THEME_COUNT);
    }
}
