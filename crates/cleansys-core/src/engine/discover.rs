//! Detect per-app cache folders on this machine that no built-in cleaner knows about and
//! offer one cleaner for each ("Detected App Caches").
//!
//! Only well-known cache roots are looked at (`~/.cache`, `~/Library/Caches`, and the
//! `Cache`-style folders directly under `%LOCALAPPDATA%` / `%APPDATA%` app folders), and
//! only their direct children. Folders already covered by a built-in or user cleaner, symlinks,
//! empty folders, protected paths and OS-managed folders are left out.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use super::spec::{Action, CleanerSpec, Risk, current_os};
use super::{paths, safety};

/// Category the discovered cleaners are filed under.
pub const CATEGORY: &str = "Detected App Caches";

/// Upper bound on discovered cleaners, so the list stays usable (most recently used first).
const MAX_DISCOVERED: usize = 60;

/// Cache sub-folder names looked for inside Windows app folders.
const WIN_CACHE_NAMES: &[&str] = &["Cache", "Code Cache", "GPUCache", "cache", "Caches"];

/// Folders directly under a cache root that are managed by the OS or hold state, not cache.
fn is_managed(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    n.starts_with("com.apple.")
        || n.starts_with('.')
        || matches!(
            n.as_str(),
            "tmp"
                | "temp"
                | "temporaryitems"
                | "fontconfig"
                | "thumbnails"
                | "cloudkit"
                | "metadata"
                | "geoservices"
        )
}

fn slug(name: &str) -> String {
    let s: String = name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    s.split('-')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Candidate cache folders of the current OS (before filtering).
fn candidates() -> Vec<PathBuf> {
    let t = |tpl: &str| paths::resolve(tpl, &paths::lookup_env);
    match current_os() {
        "linux" => t("$XDG_CACHE_HOME/*"),
        "macos" => t("~/Library/Caches/*"),
        "windows" => {
            let mut out = Vec::new();
            for base in ["$LOCALAPPDATA", "$APPDATA"] {
                for name in WIN_CACHE_NAMES {
                    out.extend(t(&format!("{base}/*/{name}")));
                }
            }
            out
        }
        _ => Vec::new(),
    }
}

/// Every path an existing cleaner already deletes (globs resolved).
fn covered_paths(known: &[CleanerSpec]) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for spec in known {
        for action in &spec.actions {
            let measured: &[String] = match action {
                Action::Command { measure, .. } => measure,
                _ => &[],
            };
            for tpl in action
                .delete_templates()
                .into_iter()
                .chain(measured.iter().map(String::as_str))
            {
                out.extend(paths::resolve(tpl, &paths::lookup_env));
            }
        }
    }
    out
}

fn has_entries(dir: &Path) -> bool {
    std::fs::read_dir(dir).is_ok_and(|mut rd| rd.next().is_some())
}

fn mtime(p: &Path) -> std::time::SystemTime {
    std::fs::metadata(p)
        .and_then(|m| m.modified())
        .unwrap_or(std::time::UNIX_EPOCH)
}

/// Pure filter + builder, separated for testing.
pub fn build_specs(mut found: Vec<PathBuf>, covered: &[PathBuf]) -> Vec<CleanerSpec> {
    found.sort();
    found.dedup();
    found.retain(|p| {
        let Some(name) = p.file_name().and_then(|n| n.to_str()) else {
            return false;
        };
        let meta = std::fs::symlink_metadata(p);
        meta.is_ok_and(|m| m.is_dir() && !m.file_type().is_symlink())
            && !is_managed(name)
            && !safety::is_protected(p)
            && has_entries(p)
            && !covered.iter().any(|c| c.starts_with(p) || p.starts_with(c))
    });
    found.sort_by_key(|p| std::cmp::Reverse(mtime(p)));
    found.truncate(MAX_DISCOVERED);

    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for p in found {
        // Windows apps sit one level deeper (`<App>/Cache`): name the app, not the cache dir.
        let app = if WIN_CACHE_NAMES.iter().any(|n| p.ends_with(n)) {
            p.parent().and_then(|x| x.file_name())
        } else {
            p.file_name()
        };
        let app = app
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let mut id = format!("detected-{}", slug(&app));
        while !seen.insert(id.clone()) {
            id.push_str("-2");
        }
        let dir = p.to_string_lossy().into_owned();
        out.push(CleanerSpec {
            id,
            name: format!("{app} Cache"),
            description: format!("Detected cache folder of {app} ({dir}); rebuilt by the app"),
            category: CATEGORY.to_string(),
            os: Vec::new(),
            requires_root: false,
            risk: Risk::Moderate,
            process: Vec::new(),
            lock_files: Vec::new(),
            actions: vec![Action::Delete {
                paths: vec![dir],
                linux: Vec::new(),
                macos: Vec::new(),
                windows: Vec::new(),
                contents_only: true,
                recreate: false,
                files_only: false,
                name_regex: None,
                not_name_regex: None,
                path_regex: None,
                not_path_regex: None,
                sudo: None,
                label: Some(format!("{app} cache")),
            }],
        });
    }
    out
}

/// Cleaners for cache folders found on this machine that `known` does not cover.
pub fn discover(known: &[CleanerSpec]) -> Vec<CleanerSpec> {
    if std::env::var_os("CLEANSYS_NO_DISCOVER").is_some()
        || !crate::settings::load_settings().map_or(true, |s| s.detect_app_caches())
    {
        return Vec::new();
    }
    build_specs(candidates(), &covered_paths(known))
}

/// Folders that have their own detected cleaner. The broad "Application Caches" sweeper
/// leaves these alone so the same space is never offered twice. Computed once per process.
pub fn is_discovered(dir: &Path) -> bool {
    static SET: std::sync::OnceLock<HashSet<PathBuf>> = std::sync::OnceLock::new();
    SET.get_or_init(|| dirs_of(&discover(&super::registry::load_declared())))
        .contains(dir)
}

/// Folders the "Application Caches" sweeper must leave alone: those with a detected
/// cleaner, and those that hold a path a declared cleaner (e.g. a browser) already
/// deletes. Computed once per process.
pub fn is_claimed(dir: &Path) -> bool {
    static COVERED: std::sync::OnceLock<Vec<PathBuf>> = std::sync::OnceLock::new();
    is_discovered(dir)
        || COVERED
            .get_or_init(|| covered_paths(&super::registry::load_declared()))
            .iter()
            .any(|p| p.starts_with(dir))
}

/// The folders the given (detected) cleaners delete.
fn dirs_of(specs: &[CleanerSpec]) -> HashSet<PathBuf> {
    specs
        .iter()
        .flat_map(|s| &s.actions)
        .flat_map(Action::delete_templates)
        .map(PathBuf::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dir_with_file(root: &Path, name: &str) -> PathBuf {
        let d = root.join(name);
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join("blob"), b"x").unwrap();
        d
    }

    #[test]
    fn builds_one_cleaner_per_app_and_skips_covered_empty_and_managed() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let app = dir_with_file(root, "SomeApp");
        let covered = dir_with_file(root, "Known");
        let apple = dir_with_file(root, "com.apple.thing");
        let dotted = dir_with_file(root, ".hidden");
        let empty = root.join("Empty");
        std::fs::create_dir_all(&empty).unwrap();
        let file = root.join("plain.txt");
        std::fs::write(&file, b"x").unwrap();

        let specs = build_specs(
            vec![app.clone(), covered.clone(), apple, dotted, empty, file],
            &[covered],
        );
        assert_eq!(specs.len(), 1);
        let s = &specs[0];
        assert_eq!(s.id, "detected-someapp");
        assert_eq!(s.name, "SomeApp Cache");
        assert_eq!(s.category, CATEGORY);
        assert!(!s.requires_root);
        assert_eq!(s.risk, Risk::Moderate);
        assert_eq!(
            s.actions[0].delete_templates(),
            vec![app.to_string_lossy().as_ref()]
        );
    }

    #[test]
    fn a_covered_child_hides_the_parent_and_ids_stay_unique() {
        let tmp = tempfile::tempdir().unwrap();
        let a = dir_with_file(tmp.path(), "A b");
        let b = dir_with_file(tmp.path(), "A-b");
        let child = a.join("blob");
        let specs = build_specs(vec![a.clone(), b], std::slice::from_ref(&child));
        assert_eq!(specs.len(), 1, "A b is partly covered");
        let specs = build_specs(vec![a, dir_with_file(tmp.path(), "A-b")], &[]);
        let ids: HashSet<_> = specs.iter().map(|s| s.id.clone()).collect();
        assert_eq!(ids.len(), 2);
    }

    #[test]
    fn detected_folders_are_the_ones_the_sweeper_must_skip() {
        let tmp = tempfile::tempdir().unwrap();
        let app = dir_with_file(tmp.path(), "SomeApp");
        let specs = build_specs(vec![app.clone()], &[]);
        let dirs = dirs_of(&specs);
        assert!(dirs.contains(&app));
        assert!(!dirs.contains(&tmp.path().join("Other")));
    }

    #[test]
    fn names_the_app_not_the_cache_folder_on_windows_layouts() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = dir_with_file(&tmp.path().join("Vendor"), "Cache");
        let specs = build_specs(vec![cache], &[]);
        assert_eq!(specs[0].name, "Vendor Cache");
        assert_eq!(specs[0].id, "detected-vendor");
    }
}
