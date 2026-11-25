//! Simple JSON-backed settings persistence, shared by every front-end.
//!
//! Settings are stored at `~/.config/cleansys/settings.json` — or the platform-appropriate
//! config directory. Writes are atomic: content is first written to a `NamedTempFile` in the same directory and then `persist()`-ed into place,
//! so a crash mid-write can never produce a corrupted file.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::io::Write as _;
use std::path::{Path, PathBuf};

/// Persisted user preferences.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Settings {
    /// Name of the selected theme (see [`crate::theme::theme_names`]).
    /// Stored by name (not index) so preset reordering across releases
    /// doesn't silently change a user's saved theme.
    #[serde(default)]
    pub theme_name: Option<String>,
    /// Names of cleaners that were selected (checked) the last time the
    /// app was closed, restored on next launch. Stored as `"Category: Item"`
    /// pairs so identically-named cleaners in different categories don't
    /// collide.
    #[serde(default)]
    pub selected_cleaners: Vec<String>,
    /// Hide cleaners that have nothing to clean (`None` = default: on).
    #[serde(default)]
    pub hide_empty: Option<bool>,
    /// Ask before running a clean (`None` = default: on).
    #[serde(default)]
    pub confirm_before_run: Option<bool>,
    /// Last GUI window size in logical pixels (`width`, `height`).
    #[serde(default)]
    pub window_size: Option<(u32, u32)>,
    /// Height of the GUI activity panel in pixels (dragged by the user).
    #[serde(default)]
    pub log_height: Option<u32>,
    /// Worker threads for scans (`None`/`0` = automatic: 2× the CPU cores, 4–32).
    /// Lower it on spinning disks or network mounts. `CLEANSYS_SCAN_THREADS` overrides.
    #[serde(default)]
    pub scan_threads: Option<usize>,
    /// Ids of cleaners the user opted out of scanning (and so out of showing a size).
    /// A cleaner's own choice beats its category's, which beats the built-in default.
    #[serde(default)]
    pub disabled_cleaners: Vec<String>,
    /// Ids of cleaners the user opted into scanning although their category is off.
    #[serde(default)]
    pub enabled_cleaners: Vec<String>,
    /// Categories the user switched scanning off for.
    #[serde(default)]
    pub disabled_categories: Vec<String>,
    /// Categories the user switched scanning on for although they start off
    /// (see `scan_defaults.toml`).
    #[serde(default)]
    pub enabled_categories: Vec<String>,
    /// Offer a cleaner for every installed app's cache folder (`None` = default: on).
    /// Applies from the next start.
    #[serde(default)]
    pub detect_app_caches: Option<bool>,
}

impl Settings {
    /// Resolve the persisted theme name to an index via
    /// [`crate::theme::theme_index_by_name`], defaulting to `0`.
    pub fn theme_index(&self) -> usize {
        match self.theme_name.as_deref() {
            Some(n) if crate::theme::system::is_system_name(n) => {
                crate::theme::system::current_system_theme_index()
            }
            Some(n) => crate::theme::theme_index_by_name(n),
            None => 0,
        }
    }

    /// Whether the theme follows the OS light/dark setting (`theme_name == "System"`).
    pub fn follow_system_theme(&self) -> bool {
        self.theme_name
            .as_deref()
            .is_some_and(crate::theme::system::is_system_name)
    }

    /// Whether empty cleaners are hidden after a scan (default: yes).
    pub fn hide_empty(&self) -> bool {
        self.hide_empty.unwrap_or(true)
    }

    /// Whether a clean asks for confirmation first (default: yes).
    pub fn confirm_before_run(&self) -> bool {
        self.confirm_before_run.unwrap_or(true)
    }

    /// Whether installed apps' caches are detected and offered (default: yes).
    pub fn detect_app_caches(&self) -> bool {
        self.detect_app_caches.unwrap_or(true)
    }

    /// Is this cleaner scanned? Its own choice wins, then its category's, then the
    /// built-in default (`scan_defaults.toml`).
    pub fn scan_enabled(&self, id: &str, category: &str) -> bool {
        if contains(&self.disabled_cleaners, id) {
            false
        } else if contains(&self.enabled_cleaners, id) {
            true
        } else {
            self.category_scan_enabled(category)
        }
    }

    /// Is this category scanned unless a cleaner in it says otherwise?
    pub fn category_scan_enabled(&self, category: &str) -> bool {
        if contains(&self.disabled_categories, category) {
            false
        } else {
            contains(&self.enabled_categories, category) || scan_on_by_default(category)
        }
    }

    /// Ids of the loaded cleaners that scans leave out.
    pub fn scan_off_ids(&self, categories: &[crate::CleanerCategory]) -> Vec<String> {
        categories
            .iter()
            .flat_map(|c| c.items.iter().map(move |i| (c, i)))
            .filter(|(c, i)| !self.scan_enabled(&i.id, &c.name))
            .map(|(_, i)| i.id.clone())
            .collect()
    }

    /// Opt one cleaner in or out of scanning, whatever its category does.
    pub fn set_scan_enabled(&mut self, id: &str, enabled: bool) {
        self.disabled_cleaners.retain(|d| d != id);
        self.enabled_cleaners.retain(|d| d != id);
        let list = if enabled {
            &mut self.enabled_cleaners
        } else {
            &mut self.disabled_cleaners
        };
        list.push(id.to_string());
        list.sort();
    }

    /// Switch a whole category on or off for scanning. The choices made for single
    /// cleaners in it (`ids`) are dropped, so the category decides for all of them.
    pub fn set_category_scan(&mut self, category: &str, enabled: bool, ids: &[String]) {
        self.disabled_categories.retain(|c| c != category);
        self.enabled_categories.retain(|c| c != category);
        let list = if enabled {
            &mut self.enabled_categories
        } else {
            &mut self.disabled_categories
        };
        list.push(category.to_string());
        list.sort();
        self.disabled_cleaners.retain(|d| !ids.contains(d));
        self.enabled_cleaners.retain(|d| !ids.contains(d));
    }

    /// The saved GUI window size, if it looks sane (a corrupt or absurd value is ignored).
    pub fn saved_window_size(&self) -> Option<(u32, u32)> {
        self.window_size
            .filter(|(w, h)| (200..=16_000).contains(w) && (200..=16_000).contains(h))
    }

    /// Build the `"Category: Item"` key used to identify a selected cleaner.
    pub fn selection_key(category_name: &str, item_name: &str) -> String {
        format!("{category_name}: {item_name}")
    }

    /// Whether the given category/item pair was selected last time.
    pub fn is_selected(&self, category_name: &str, item_name: &str) -> bool {
        self.selected_cleaners
            .iter()
            .any(|k| k == &Self::selection_key(category_name, item_name))
    }
}

fn contains(list: &[String], item: &str) -> bool {
    list.iter().any(|x| x == item)
}

/// Does a category scan out of the box? (`scan_defaults.toml`; unlisted = yes.)
pub fn scan_on_by_default(category: &str) -> bool {
    !off_by_default().iter().any(|c| c == category)
}

/// The categories that start with scanning off.
pub fn off_by_default() -> &'static [String] {
    #[derive(Deserialize)]
    struct Defaults {
        off_by_default: Vec<String>,
    }
    static OFF: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
    OFF.get_or_init(|| {
        toml::from_str::<Defaults>(include_str!("scan_defaults.toml"))
            .map(|d| d.off_by_default)
            .unwrap_or_default()
    })
}

/// Returns the settings directory (`~/.config/cleansys/` or equivalent).
pub fn settings_dir() -> Result<PathBuf> {
    let dirs = directories::BaseDirs::new().context("could not determine config directory")?;
    Ok(dirs.config_dir().join("cleansys"))
}

/// Full path to the JSON settings file.
pub fn settings_json_path() -> Result<PathBuf> {
    Ok(settings_dir()?.join("settings.json"))
}

/// Load settings from any JSON path.
fn load_from(path: &Path) -> Result<Settings> {
    if path.exists() {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        return match serde_json::from_str::<Settings>(&content) {
            Ok(s) => Ok(s),
            Err(e) => {
                log::warn!("settings file {path:?} is malformed ({e}); using defaults");
                Ok(Settings::default())
            }
        };
    }
    Ok(Settings::default())
}

/// Save settings to any JSON path (atomic write via `NamedTempFile`).
fn save_to(path: &Path, settings: &Settings) -> Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(parent)
        .with_context(|| format!("failed to create directory {}", parent.display()))?;

    let content = serde_json::to_string_pretty(settings).context("failed to serialise settings")?;

    let mut tmp = tempfile::NamedTempFile::new_in(parent)
        .context("failed to create temporary settings file")?;
    tmp.write_all(content.as_bytes())
        .context("failed to write settings to temporary file")?;
    tmp.persist(path)
        .map_err(|e| anyhow::anyhow!("failed to persist settings file: {e}"))?;

    Ok(())
}

/// Load application settings (`settings.json`).
///
/// Returns [`Settings::default`] when the file does not exist yet (first
/// run) or is malformed (the file is preserved for manual recovery).
pub fn load_settings() -> Result<Settings> {
    load_from(&settings_json_path()?)
}

/// Persist application settings.
pub fn save_settings(settings: &Settings) -> Result<()> {
    save_to(&settings_json_path()?, settings)
}

/// Read-modify-write `settings.json`, so one front-end changing a preference never
/// clobbers fields it doesn't know about.
pub fn update_settings(change: impl FnOnce(&mut Settings)) -> Result<Settings> {
    let mut s = load_settings()?;
    change(&mut s);
    save_settings(&s)?;
    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn preferences_default_to_on_and_old_files_still_load() {
        let s = Settings::default();
        assert!(s.hide_empty() && s.confirm_before_run());
        // A settings.json written before these fields existed.
        let old: Settings =
            serde_json::from_str(r#"{"theme_name":"Nord","selected_cleaners":[]}"#).unwrap();
        assert!(old.hide_empty() && old.confirm_before_run());
        assert_eq!(old.saved_window_size(), None);
    }

    #[test]
    fn scan_opt_out_round_trips_and_defaults_to_on() {
        let mut s = Settings::default();
        assert!(s.scan_enabled("a", "Everyday Junk"));
        s.set_scan_enabled("a", false);
        s.set_scan_enabled("a", false);
        assert!(!s.scan_enabled("a", "Everyday Junk") && s.disabled_cleaners.len() == 1);
        let back: Settings = serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
        assert!(
            !back.scan_enabled("a", "Everyday Junk") && back.scan_enabled("b", "Everyday Junk")
        );
        s.set_scan_enabled("a", true);
        assert!(s.scan_enabled("a", "Everyday Junk") && s.disabled_cleaners.is_empty());
    }

    #[test]
    fn slow_categories_start_off_and_unknown_ones_on() {
        let s = Settings::default();
        assert!(!s.category_scan_enabled("Project Build Artifacts"));
        assert!(s.category_scan_enabled("Web Browsers"));
        assert!(s.category_scan_enabled("My Own Category"));
    }

    #[test]
    fn every_default_off_category_exists_in_the_catalog() {
        let names: Vec<String> = crate::engine::registry::load_all_unfiltered(None)
            .into_iter()
            .map(|c| c.category)
            .collect();
        for c in off_by_default() {
            assert!(
                names.contains(c),
                "unknown category in scan_defaults.toml: {c}"
            );
        }
    }

    #[test]
    fn precedence_is_cleaner_then_category_then_default() {
        let cat = "Project Build Artifacts";
        let mut s = Settings::default();
        s.set_category_scan(cat, true, &[]);
        assert!(s.scan_enabled("x", cat));
        s.set_scan_enabled("x", false);
        assert!(!s.scan_enabled("x", cat) && s.scan_enabled("y", cat));
        // switching the category again drops the per-cleaner choice
        s.set_category_scan(cat, false, &["x".to_string()]);
        assert!(!s.scan_enabled("x", cat) && s.disabled_cleaners.is_empty());
        s.set_scan_enabled("y", true);
        assert!(s.scan_enabled("y", cat) && !s.scan_enabled("z", cat));
        // and a category switched on again re-enables a default-on one after off
        s.set_category_scan("Web Browsers", false, &[]);
        assert!(!s.scan_enabled("w", "Web Browsers"));
        s.set_category_scan("Web Browsers", true, &[]);
        assert!(s.scan_enabled("w", "Web Browsers"));
    }

    #[test]
    fn scan_off_ids_lists_loaded_cleaners_left_out() {
        let cats = crate::load_categories();
        let mut s = Settings::default();
        let off = s.scan_off_ids(&cats);
        for c in &cats {
            for i in &c.items {
                assert_eq!(off.contains(&i.id), !scan_on_by_default(&c.name));
            }
        }
        for c in &cats {
            s.set_category_scan(&c.name, true, &[]);
        }
        assert!(s.scan_off_ids(&cats).is_empty());
    }

    #[test]
    fn absurd_window_sizes_are_ignored() {
        for bad in [(0, 0), (50, 700), (900, 99_999), (u32::MAX, 700)] {
            let s = Settings {
                window_size: Some(bad),
                ..Default::default()
            };
            assert_eq!(s.saved_window_size(), None, "{bad:?}");
        }
    }

    #[test]
    fn system_theme_name_follows_system() {
        let s = Settings {
            theme_name: Some("System".to_string()),
            ..Default::default()
        };
        assert!(s.follow_system_theme());
        assert!(s.theme_index() < crate::theme::theme_count());
        assert!(!Settings::default().follow_system_theme());
    }

    #[test]
    fn default_settings_have_no_theme() {
        let s = Settings::default();
        assert!(s.theme_name.is_none());
        assert_eq!(s.theme_index(), 0);
        assert!(s.selected_cleaners.is_empty());
    }

    #[test]
    fn theme_index_resolves_known_name() {
        let s = Settings {
            theme_name: Some("Dracula".to_string()),
            ..Default::default()
        };
        assert_eq!(
            s.theme_index(),
            crate::theme::theme_index_by_name("Dracula")
        );
    }

    #[test]
    fn themes_beyond_the_curated_set_resolve_by_name_too() {
        // "Pip Boy" exists only in the generated (tui-nuances) part of the catalogue.
        let s = Settings {
            theme_name: Some("Pip Boy".to_string()),
            ..Default::default()
        };
        assert!(s.theme_index() >= crate::theme::CURATED_THEME_COUNT);
        assert_eq!(crate::theme::theme_names()[s.theme_index()], "Pip Boy");
    }

    #[test]
    fn theme_index_falls_back_for_unknown_name() {
        let s = Settings {
            theme_name: Some("Not A Real Theme".to_string()),
            ..Default::default()
        };
        assert_eq!(s.theme_index(), 0);
    }

    #[test]
    fn selection_key_combines_category_and_item() {
        assert_eq!(
            Settings::selection_key("User Land Cleaners", "Browser Caches"),
            "User Land Cleaners: Browser Caches"
        );
    }

    #[test]
    fn is_selected_checks_selected_cleaners() {
        let s = Settings {
            selected_cleaners: vec!["User Land Cleaners: Browser Caches".to_string()],
            ..Default::default()
        };
        assert!(s.is_selected("User Land Cleaners", "Browser Caches"));
        assert!(!s.is_selected("User Land Cleaners", "Trash"));
    }

    #[test]
    fn round_trip_save_and_load() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("settings.json");

        let settings = Settings {
            theme_name: Some("Nord".to_string()),
            selected_cleaners: vec!["User Land Cleaners: Trash".to_string()],
            hide_empty: Some(false),
            confirm_before_run: Some(false),
            window_size: Some((900, 700)),
            log_height: Some(240),
            scan_threads: None,
            detect_app_caches: Some(false),
            ..Default::default()
        };
        save_to(&path, &settings).unwrap();

        let loaded = load_from(&path).unwrap();
        assert_eq!(loaded.theme_name.as_deref(), Some("Nord"));
        assert!(!loaded.hide_empty());
        assert!(!loaded.confirm_before_run());
        assert_eq!(loaded.saved_window_size(), Some((900, 700)));
        assert_eq!(
            loaded.selected_cleaners,
            vec!["User Land Cleaners: Trash".to_string()]
        );
    }

    #[test]
    fn load_from_missing_file_returns_default() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("does-not-exist.json");
        let loaded = load_from(&path).unwrap();
        assert!(loaded.theme_name.is_none());
    }

    #[test]
    fn load_from_malformed_file_returns_default() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("settings.json");
        std::fs::write(&path, "not valid json").unwrap();
        let loaded = load_from(&path).unwrap();
        assert!(loaded.theme_name.is_none());
    }

    #[test]
    fn save_creates_parent_directories() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nested").join("dir").join("settings.json");
        save_to(&path, &Settings::default()).unwrap();
        assert!(path.exists());
    }
}
