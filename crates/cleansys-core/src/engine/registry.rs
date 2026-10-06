//! Loading of built-in and user-supplied cleaner specs.

use anyhow::{Context, Result};
use log::warn;
use std::path::{Path, PathBuf};

use super::exec::spec_has_targets;
use super::spec::{CleanerSpec, SpecFile};

/// Embedded built-in definitions (file name, TOML source).
const BUILTIN: &[(&str, &str)] = &[
    ("developer.toml", include_str!("builtin/developer.toml")),
    (
        "build_artifacts.toml",
        include_str!("builtin/build_artifacts.toml"),
    ),
    ("ai_llm.toml", include_str!("builtin/ai_llm.toml")),
    (
        "developer_more.toml",
        include_str!("builtin/developer_more.toml"),
    ),
    ("browsers.toml", include_str!("builtin/browsers.toml")),
    ("apps.toml", include_str!("builtin/apps.toml")),
    ("games.toml", include_str!("builtin/games.toml")),
    ("system.toml", include_str!("builtin/system.toml")),
    ("containers.toml", include_str!("builtin/containers.toml")),
    ("privacy.toml", include_str!("builtin/privacy.toml")),
];

/// Parse one TOML document into specs.
pub fn parse_spec_file(src: &str) -> Result<Vec<CleanerSpec>> {
    let file: SpecFile = toml::from_str(src).context("invalid cleaner TOML")?;
    Ok(file.cleaners)
}

/// Directory scanned for user cleaner definitions.
pub fn user_spec_dir() -> Option<PathBuf> {
    crate::settings::settings_dir()
        .ok()
        .map(|d| d.join("cleaners.d"))
}

/// All specs applicable on this OS whose targets exist on this machine:
/// built-ins first, then user files (same `id` replaces the built-in).
pub fn load_specs() -> Vec<CleanerSpec> {
    let mut specs = load_all_unfiltered(user_spec_dir().as_deref());
    specs.retain(|s| s.applies_to_current_os() && spec_has_targets(s));
    specs
}

/// Built-in + user specs, no OS / existence filtering. Order preserved;
/// user specs override built-ins by `id`.
pub fn load_all_unfiltered(user_dir: Option<&Path>) -> Vec<CleanerSpec> {
    let mut specs: Vec<CleanerSpec> = Vec::new();
    for (file, src) in BUILTIN {
        match parse_spec_file(src) {
            Ok(v) => merge(&mut specs, v),
            Err(e) => warn!("built-in cleaner file {file}: {e:#}"),
        }
    }
    if let Some(dir) = user_dir {
        if let Ok(rd) = std::fs::read_dir(dir) {
            let mut files: Vec<PathBuf> = rd
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|x| x == "toml"))
                .collect();
            files.sort();
            for f in files {
                match std::fs::read_to_string(&f)
                    .map_err(anyhow::Error::from)
                    .and_then(|s| parse_spec_file(&s))
                {
                    Ok(v) => merge(&mut specs, v),
                    Err(e) => warn!("user cleaner file {f:?}: {e:#}"),
                }
            }
        }
    }
    specs
}

/// Every marker file name/glob used by any `project_artifacts` action of the built-in
/// and user cleaner definitions (computed once). The project scan only records these.
pub fn known_project_markers() -> Vec<String> {
    static MARKERS: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
    MARKERS
        .get_or_init(|| {
            let mut out: Vec<String> = Vec::new();
            for spec in load_all_unfiltered(user_spec_dir().as_deref()) {
                for action in &spec.actions {
                    if let super::spec::Action::ProjectArtifacts { markers, .. } = action {
                        for m in markers {
                            if !out.contains(m) {
                                out.push(m.clone());
                            }
                        }
                    }
                }
            }
            out
        })
        .clone()
}

fn merge(into: &mut Vec<CleanerSpec>, new: Vec<CleanerSpec>) {
    for s in new {
        match into.iter_mut().find(|e| e.id == s.id) {
            Some(slot) => *slot = s,
            None => into.push(s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn builtin_specs_parse_and_are_well_formed() {
        for (file, src) in BUILTIN {
            let specs = parse_spec_file(src).unwrap_or_else(|e| panic!("{file}: {e:#}"));
            assert!(!specs.is_empty(), "{file} empty");
        }
        let all = load_all_unfiltered(None);
        let mut ids = HashSet::new();
        for s in &all {
            assert!(ids.insert(s.id.clone()), "duplicate id {}", s.id);
            assert!(!s.name.is_empty() && !s.description.is_empty(), "{}", s.id);
            assert!(!s.actions.is_empty(), "{} has no actions", s.id);
            for os in &s.os {
                assert!(
                    ["linux", "macos", "windows"].contains(&os.as_str()),
                    "{}: bad os {os}",
                    s.id
                );
            }
        }
    }

    #[test]
    fn builtin_delete_paths_are_never_protected_or_relative() {
        use crate::engine::{paths, safety};
        for s in load_all_unfiltered(None) {
            if !s.applies_to_current_os() {
                continue; // e.g. `C:\\Windows` is not absolute on Unix
            }
            for a in &s.actions {
                for t in a.delete_templates() {
                    let expanded = paths::expand(t, &paths::lookup_env);
                    let is_glob = expanded.contains(['*', '?', '[']);
                    let p = std::path::PathBuf::from(
                        expanded.split(['*', '?', '[']).next().unwrap_or(""),
                    );
                    assert!(p.is_absolute(), "{}: {t} not absolute", s.id);
                    if is_glob {
                        // A glob's literal prefix is a *parent* directory whose
                        // children get removed one by one (each re-checked at
                        // delete time), so only require it to be a real
                        // directory prefix, not a bare filesystem root.
                        assert!(p.components().count() >= 3, "{}: {t} too broad", s.id);
                    } else {
                        assert!(!safety::is_protected(&p), "{}: {t} protected", s.id);
                    }
                }
            }
        }
    }

    #[test]
    fn user_spec_overrides_builtin_by_id() {
        let dir = tempfile::tempdir().unwrap();
        let first = load_all_unfiltered(None)[0].clone();
        std::fs::write(
            dir.path().join("o.toml"),
            format!(
                "[[cleaner]]\nid=\"{}\"\nname=\"Custom\"\ndescription=\"d\"\n[[cleaner.action]]\ntype=\"delete\"\npaths=[\"/nonexistent/x\"]\n",
                first.id
            ),
        )
        .unwrap();
        let all = load_all_unfiltered(Some(dir.path()));
        assert_eq!(
            all.iter().find(|s| s.id == first.id).unwrap().name,
            "Custom"
        );
    }

    #[test]
    fn broken_user_file_is_ignored() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("bad.toml"), "not [valid").unwrap();
        assert!(!load_all_unfiltered(Some(dir.path())).is_empty());
    }
}
