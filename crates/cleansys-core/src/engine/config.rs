//! Engine configuration (project scan roots, exclusions, staleness).
//!
//! Stored at `~/.config/cleansys/engine.json`. The environment variable
//! `CLEANSYS_SCAN_ROOTS` (a platform path list: `:` on Unix, `;` on Windows)
//! overrides `scan_roots`.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Directory names under `$HOME` that commonly hold source checkouts.
const DEFAULT_ROOT_NAMES: &[&str] = &[
    "Projects",
    "projects",
    "dev",
    "Dev",
    "src",
    "code",
    "Code",
    "repos",
    "Repos",
    "workspace",
    "Workspace",
    "work",
    "Work",
    "Developer",
    "git",
    "AndroidStudioProjects",
    "IdeaProjects",
    "RustroverProjects",
    "StudioProjects",
    "GolandProjects",
    "PycharmProjects",
    "WebstormProjects",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EngineConfig {
    /// Directories scanned for projects. Empty = auto-detect common ones.
    pub scan_roots: Vec<String>,
    /// Maximum directory depth below a root to search.
    pub max_depth: usize,
    /// Only clean build output whose project was untouched this many days
    /// (0 = clean regardless of age). Default 14: never nukes a project you are
    /// actively building.
    pub min_age_days: u64,
    /// Glob patterns (`~`, `$VAR` allowed) never to delete.
    pub exclude: Vec<String>,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            scan_roots: Vec::new(),
            max_depth: 6,
            min_age_days: 14,
            exclude: Vec::new(),
        }
    }
}

/// Idle-day choices offered by the day selector in every front-end.
pub const MIN_AGE_CHOICES: [u64; 8] = [0, 1, 3, 7, 14, 30, 60, 90];

/// The next (`step > 0`) or previous (`step < 0`) choice after `current`,
/// clamped at both ends. Values not in the list snap to the nearest step.
pub fn step_min_age(current: u64, step: i32) -> u64 {
    let idx = MIN_AGE_CHOICES
        .iter()
        .position(|d| *d >= current)
        .unwrap_or(MIN_AGE_CHOICES.len() - 1);
    let exact = MIN_AGE_CHOICES[idx] == current;
    let next = match step.signum() {
        1 if exact => idx + 1,
        1 => idx,
        -1 => idx.saturating_sub(1),
        _ => idx,
    };
    MIN_AGE_CHOICES[next.min(MIN_AGE_CHOICES.len() - 1)]
}

/// Human label: `0` → "any age", `7` → "7 days".
pub fn min_age_label(days: u64) -> String {
    match days {
        0 => "any age".to_string(),
        1 => "1 day".to_string(),
        d => format!("{d} days"),
    }
}

impl EngineConfig {
    /// Persist a new idle-day threshold (load → set → save).
    pub fn save_min_age_days(days: u64) -> Result<()> {
        let mut cfg = Self::load();
        cfg.min_age_days = days;
        cfg.save()
    }

    pub fn path() -> Result<PathBuf> {
        Ok(crate::settings::settings_dir()?.join("engine.json"))
    }

    /// Load config, falling back to defaults on any problem.
    pub fn load() -> Self {
        let mut cfg = Self::path()
            .ok()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|s| serde_json::from_str::<Self>(&s).ok())
            .unwrap_or_default();
        if let Ok(v) = std::env::var("CLEANSYS_SCAN_ROOTS") {
            cfg.scan_roots = std::env::split_paths(&v)
                .map(|p| p.to_string_lossy().into_owned())
                .filter(|s| !s.is_empty())
                .collect();
        }
        cfg
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::path()?;
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).context("create config dir")?;
        }
        std::fs::write(&path, serde_json::to_string_pretty(self)?).context("write engine.json")
    }

    /// Effective, existing, de-duplicated scan roots.
    pub fn effective_roots(&self) -> Vec<PathBuf> {
        let mut roots: Vec<PathBuf> = if self.scan_roots.is_empty() {
            crate::cleaners::platform::home_dir()
                .map(|h| DEFAULT_ROOT_NAMES.iter().map(|n| h.join(n)).collect())
                .unwrap_or_default()
        } else {
            self.scan_roots
                .iter()
                .map(|r| PathBuf::from(super::paths::expand(r, &super::paths::lookup_env)))
                .collect()
        };
        roots.retain(|r| r.is_absolute() && r.is_dir());
        roots.sort();
        roots.dedup();
        // Case-insensitive filesystems (macOS/Windows) can list the same
        // directory under two spellings ("Projects" vs "projects").
        let mut seen = std::collections::HashSet::new();
        roots.retain(|r| seen.insert(dir_identity(r)));
        // Drop roots nested inside another root (avoid double scanning).
        let all = roots.clone();
        roots.retain(|r| !all.iter().any(|o| o != r && r.starts_with(o)));
        roots
    }
}

#[cfg(unix)]
fn dir_identity(p: &std::path::Path) -> (u64, u64) {
    use std::os::unix::fs::MetadataExt;
    std::fs::metadata(p)
        .map(|m| (m.dev(), m.ino()))
        .unwrap_or((0, 0))
}

#[cfg(not(unix))]
fn dir_identity(p: &std::path::Path) -> String {
    std::fs::canonicalize(p)
        .map(|c| c.to_string_lossy().to_lowercase())
        .unwrap_or_default()
}

#[cfg(test)]
mod min_age_step_tests {
    use super::*;

    #[test]
    fn steps_through_the_choices_and_clamps() {
        assert_eq!(step_min_age(14, 1), 30);
        assert_eq!(step_min_age(14, -1), 7);
        assert_eq!(step_min_age(0, -1), 0);
        assert_eq!(step_min_age(90, 1), 90);
        assert_eq!(step_min_age(10, 1), 14, "off-grid values snap upward");
        assert_eq!(step_min_age(10, -1), 7);
        assert_eq!(min_age_label(0), "any age");
        assert_eq!(min_age_label(7), "7 days");
    }
}
