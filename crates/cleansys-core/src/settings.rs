//! Simple JSON-backed settings persistence, shared by the TUI and GUI.
//!
//! Settings are stored at `~/.config/cleansys/settings.json` (GUI) and
//! `~/.config/cleansys/tui-settings.json` (TUI) — or the platform-appropriate
//! config directory. Writes are atomic: content is first written to a
//! `NamedTempFile` in the same directory and then `persist()`-ed into place,
//! so a crash mid-write can never produce a corrupted file.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::io::Write as _;
use std::path::{Path, PathBuf};

/// Persisted user preferences.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Settings {
    /// Name of the selected theme (see [`crate::theme::THEME_NAMES`]).
    /// Stored by name (not index) so preset reordering across releases
    /// doesn't silently change a user's saved theme.
    #[serde(default)]
    pub theme_name: Option<String>,
}

impl Settings {
    /// Resolve the persisted theme name to an index via
    /// [`crate::theme::theme_index_by_name`], defaulting to `0`.
    pub fn theme_index(&self) -> usize {
        self.theme_name
            .as_deref()
            .map(crate::theme::theme_index_by_name)
            .unwrap_or(0)
    }
}

/// Returns the settings directory (`~/.config/cleansys/` or equivalent).
pub fn settings_dir() -> Result<PathBuf> {
    let dirs = directories::BaseDirs::new().context("could not determine config directory")?;
    Ok(dirs.config_dir().join("cleansys"))
}

/// Full path to the GUI JSON settings file.
pub fn settings_json_path() -> Result<PathBuf> {
    Ok(settings_dir()?.join("settings.json"))
}

/// Full path to the TUI-specific JSON settings file.
pub fn tui_settings_json_path() -> Result<PathBuf> {
    Ok(settings_dir()?.join("tui-settings.json"))
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

/// Load GUI application settings (`settings.json`).
///
/// Returns [`Settings::default`] when the file does not exist yet (first
/// run) or is malformed (the file is preserved for manual recovery).
pub fn load_settings() -> Result<Settings> {
    load_from(&settings_json_path()?)
}

/// Persist GUI application settings.
pub fn save_settings(settings: &Settings) -> Result<()> {
    save_to(&settings_json_path()?, settings)
}

/// Load TUI-specific settings (`tui-settings.json`).
pub fn load_tui_settings() -> Result<Settings> {
    load_from(&tui_settings_json_path()?)
}

/// Persist TUI-specific settings.
pub fn save_tui_settings(settings: &Settings) -> Result<()> {
    save_to(&tui_settings_json_path()?, settings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn default_settings_have_no_theme() {
        let s = Settings::default();
        assert!(s.theme_name.is_none());
        assert_eq!(s.theme_index(), 0);
    }

    #[test]
    fn theme_index_resolves_known_name() {
        let s = Settings {
            theme_name: Some("Dracula".to_string()),
        };
        assert_eq!(
            s.theme_index(),
            crate::theme::theme_index_by_name("Dracula")
        );
    }

    #[test]
    fn theme_index_falls_back_for_unknown_name() {
        let s = Settings {
            theme_name: Some("Not A Real Theme".to_string()),
        };
        assert_eq!(s.theme_index(), 0);
    }

    #[test]
    fn round_trip_save_and_load() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("settings.json");

        let settings = Settings {
            theme_name: Some("Nord".to_string()),
        };
        save_to(&path, &settings).unwrap();

        let loaded = load_from(&path).unwrap();
        assert_eq!(loaded.theme_name.as_deref(), Some("Nord"));
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
