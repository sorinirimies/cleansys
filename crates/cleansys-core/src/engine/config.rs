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

/// Scan-depth choices offered by the settings screens.
pub const MAX_DEPTH_CHOICES: [usize; 7] = [2, 3, 4, 5, 6, 8, 10];

/// Next (`step > 0`) / previous (`step < 0`) depth choice, clamped; off-grid values snap.
pub fn step_max_depth(current: usize, step: i32) -> usize {
    let idx = MAX_DEPTH_CHOICES
        .iter()
        .position(|d| *d >= current)
        .unwrap_or(MAX_DEPTH_CHOICES.len() - 1);
    let exact = MAX_DEPTH_CHOICES[idx] == current;
    let next = match step.signum() {
        1 if exact => idx + 1,
        1 => idx,
        -1 => idx.saturating_sub(1),
        _ => idx,
    };
    MAX_DEPTH_CHOICES[next.min(MAX_DEPTH_CHOICES.len() - 1)]
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
    /// Load → change → save, so front-ends editing different fields never clobber each other.
    pub fn update<R>(change: impl FnOnce(&mut Self) -> R) -> Result<R> {
        let mut cfg = Self::load();
        let out = change(&mut cfg);
        cfg.save()?;
        Ok(out)
    }

    /// The roots as the settings screens list and edit them: the configured ones, or — when
    /// none are configured — what is auto-detected (so removing one keeps the rest).
    pub fn roots_for_editing(&self) -> Vec<String> {
        if self.scan_roots.is_empty() {
            self.effective_roots()
                .iter()
                .map(|p| p.to_string_lossy().into_owned())
                .collect()
        } else {
            self.scan_roots.clone()
        }
    }

    /// Add a project scan root. Accepts `~`/`$VAR`; the directory must exist.
    pub fn add_scan_root(&mut self, input: &str) -> std::result::Result<(), String> {
        let input = input.trim();
        if input.is_empty() {
            return Err("enter a folder".into());
        }
        let expanded = super::paths::expand(input, &super::paths::lookup_env);
        let path = std::path::Path::new(&expanded);
        if !path.is_absolute() {
            return Err("use an absolute path (or start with ~)".into());
        }
        if !path.is_dir() {
            return Err(format!("{expanded} is not a folder"));
        }
        let mut roots = self.roots_for_editing();
        if roots.iter().any(|r| r == input) {
            return Err("already listed".into());
        }
        roots.push(input.to_string());
        self.scan_roots = roots;
        Ok(())
    }

    /// Remove the root at `index` (as listed by [`Self::roots_for_editing`]).
    pub fn remove_scan_root(&mut self, index: usize) -> std::result::Result<(), String> {
        let mut roots = self.roots_for_editing();
        if index >= roots.len() {
            return Err("no such folder".into());
        }
        roots.remove(index);
        self.scan_roots = roots;
        Ok(())
    }

    /// Add a glob that is never deleted (`~`, `$VAR` allowed).
    pub fn add_exclude(&mut self, input: &str) -> std::result::Result<(), String> {
        let input = input.trim();
        if input.is_empty() {
            return Err("enter a path or pattern".into());
        }
        if glob::Pattern::new(&super::paths::expand(input, &super::paths::lookup_env)).is_err() {
            return Err("not a valid pattern".into());
        }
        if self.exclude.iter().any(|e| e == input) {
            return Err("already listed".into());
        }
        self.exclude.push(input.to_string());
        Ok(())
    }

    /// Remove the exclusion at `index`.
    pub fn remove_exclude(&mut self, index: usize) -> std::result::Result<(), String> {
        if index >= self.exclude.len() {
            return Err("no such exclusion".into());
        }
        self.exclude.remove(index);
        Ok(())
    }

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
        // Write to a temp file and rename it over the target: readers (a scan, the
        // scheduled job, another front-end) never see a truncated, half-written file,
        // which `load` would silently turn into the defaults.
        let tmp = path.with_extension(format!("json.tmp{}", std::process::id()));
        std::fs::write(&tmp, serde_json::to_string_pretty(self)?).context("write engine.json")?;
        std::fs::rename(&tmp, &path)
            .inspect_err(|_| {
                let _ = std::fs::remove_file(&tmp);
            })
            .context("replace engine.json")
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

#[cfg(test)]
mod edit_tests {
    use super::*;

    #[test]
    fn depth_steps_and_clamps() {
        assert_eq!(step_max_depth(6, 1), 8);
        assert_eq!(step_max_depth(6, -1), 5);
        assert_eq!(step_max_depth(2, -1), 2);
        assert_eq!(step_max_depth(10, 1), 10);
        assert_eq!(step_max_depth(7, 1), 8, "off-grid values snap upward");
    }

    #[test]
    fn scan_roots_validate_dedupe_and_remove() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().to_string_lossy().into_owned();
        let mut cfg = EngineConfig {
            scan_roots: vec!["/already/here".into()],
            ..EngineConfig::default()
        };
        assert!(cfg.add_scan_root("").is_err());
        assert!(cfg.add_scan_root("relative/dir").is_err());
        assert!(cfg.add_scan_root("/definitely/not/a/dir/xyz").is_err());
        cfg.add_scan_root(&p).unwrap();
        assert_eq!(cfg.scan_roots, vec!["/already/here".to_string(), p.clone()]);
        assert!(cfg.add_scan_root(&p).is_err(), "duplicates are refused");
        cfg.remove_scan_root(0).unwrap();
        assert_eq!(cfg.scan_roots, vec![p]);
        assert!(cfg.remove_scan_root(5).is_err());
    }

    #[test]
    fn excludes_validate_dedupe_and_remove() {
        let mut cfg = EngineConfig::default();
        assert!(cfg.add_exclude("  ").is_err());
        assert!(cfg.add_exclude("[").is_err(), "bad glob");
        cfg.add_exclude("~/keep/**").unwrap();
        assert!(cfg.add_exclude("~/keep/**").is_err());
        cfg.remove_exclude(0).unwrap();
        assert!(cfg.exclude.is_empty());
        assert!(cfg.remove_exclude(0).is_err());
    }
}
