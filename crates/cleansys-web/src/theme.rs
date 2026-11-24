//! Colour themes: the core's palettes as CSS custom properties.
//!
//! `/style.css` is written against `var(--bg)`, `var(--accent)` … with a dark default;
//! a `<style>` block from [`css`] overrides those variables.

use cleansys_core::{Rgb, theme_by_index, theme_count, theme_index_by_name, theme_names};

fn hex(c: Rgb) -> String {
    format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b)
}

/// Pseudo-index meaning "follow the system" (one past the last preset).
pub fn system_index() -> usize {
    theme_count()
}

/// CSS custom properties for the theme at `index` (clamped), as one `:root{…}` rule.
///
/// [`system_index`] yields the dark preset plus a `prefers-color-scheme: light` override,
/// so the browser follows the OS live.
pub fn css(index: usize) -> String {
    if index == system_index() {
        let dark = css_for(cleansys_core::system_theme_index(true));
        let light = css_for(cleansys_core::system_theme_index(false));
        return format!("{dark}@media (prefers-color-scheme:light){{{light}}}");
    }
    css_for(index.min(theme_count() - 1))
}

fn css_for(index: usize) -> String {
    let t = theme_by_index(index);
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
    let body: String = vars.iter().fold(String::new(), |mut acc, (k, c)| {
        use std::fmt::Write;
        let _ = write!(acc, "--{k}:{};", hex(*c));
        acc
    });
    format!(
        ":root{{color-scheme:{};{body}}}",
        if t.is_dark { "dark" } else { "light" }
    )
}

/// Pick the theme: an explicit, valid `requested` name wins, otherwise `saved`.
pub fn resolve(requested: Option<&str>, saved: usize) -> usize {
    match requested {
        Some(n) if cleansys_core::is_system_name(n) => system_index(),
        Some(n) if theme_names().contains(&n) => theme_index_by_name(n),
        _ => saved,
    }
}

pub fn names() -> &'static [&'static str] {
    theme_names()
}

/// Every theme with its colours, for `/api/themes`.
pub fn catalogue() -> serde_json::Value {
    (0..theme_count())
        .map(|i| {
            let t = theme_by_index(i);
            serde_json::json!({
                "name": theme_names()[i],
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
    match cleansys_core::load_settings() {
        Ok(s) if s.follow_system_theme() => system_index(),
        Ok(s) => s.theme_index(),
        Err(_) => 0,
    }
}

/// Remember `index` for the GUI and web (best effort).
pub fn save_index(index: usize) {
    if let Ok(mut s) = cleansys_core::load_settings() {
        s.theme_name = Some(if index == system_index() {
            cleansys_core::SYSTEM_THEME_NAME.to_string()
        } else {
            theme_names()[index.min(theme_count() - 1)].to_string()
        });
        let _ = cleansys_core::save_settings(&s);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_theme_yields_all_variables() {
        for i in 0..theme_count() {
            let c = css(i);
            for v in ["--bg:", "--panel:", "--accent:", "--ok:", "--err:"] {
                assert!(c.contains(v), "{i}: {c}");
            }
        }
    }

    #[test]
    fn explicit_valid_theme_wins_unknown_is_ignored() {
        assert_eq!(resolve(Some(theme_names()[3]), 7), 3);
        assert_eq!(resolve(Some("nope"), 7), 7);
        assert_eq!(resolve(None, 7), 7);
    }

    #[test]
    fn every_theme_of_the_catalogue_is_selectable_by_name_and_saved_by_it() {
        let extra = theme_index_by_name("Pip Boy");
        assert!(
            extra >= cleansys_core::CURATED_THEME_COUNT,
            "an extra theme"
        );
        assert_eq!(resolve(Some("Pip Boy"), 0), extra);
        let cat = catalogue();
        let names: Vec<&str> = cat
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["name"].as_str().unwrap())
            .collect();
        assert!(names.contains(&"Pip Boy") && names.contains(&"Dracula"));
        assert_eq!(names.len(), theme_count());
        assert!(css(extra).contains("--bg:"));
    }

    #[test]
    fn system_resolves_to_system_index() {
        assert_eq!(resolve(Some("System"), 7), system_index());
        assert_eq!(resolve(Some("system"), 7), system_index());
    }

    #[test]
    fn system_css_has_dark_base_and_light_media_override() {
        let c = css(system_index());
        assert!(c.starts_with(":root{color-scheme:dark;"), "{c}");
        assert!(
            c.contains("@media (prefers-color-scheme:light){:root{color-scheme:light;"),
            "{c}"
        );
    }

    #[test]
    fn catalogue_lists_every_theme() {
        assert_eq!(catalogue().as_array().unwrap().len(), theme_count());
    }
}
