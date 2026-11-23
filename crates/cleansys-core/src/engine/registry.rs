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
    ("deep.toml", include_str!("builtin/deep.toml")),
    ("dev_tools.toml", include_str!("builtin/dev_tools.toml")),
    ("ai_llm.toml", include_str!("builtin/ai_llm.toml")),
    (
        "developer_more.toml",
        include_str!("builtin/developer_more.toml"),
    ),
    (
        "developer_extra.toml",
        include_str!("builtin/developer_extra.toml"),
    ),
    ("ecosystems.toml", include_str!("builtin/ecosystems.toml")),
    ("browsers.toml", include_str!("builtin/browsers.toml")),
    (
        "browsers_extra.toml",
        include_str!("builtin/browsers_extra.toml"),
    ),
    ("apps.toml", include_str!("builtin/apps.toml")),
    ("games.toml", include_str!("builtin/games.toml")),
    ("system.toml", include_str!("builtin/system.toml")),
    ("system_more.toml", include_str!("builtin/system_more.toml")),
    ("system_core.toml", include_str!("builtin/system_core.toml")),
    ("containers.toml", include_str!("builtin/containers.toml")),
    ("privacy.toml", include_str!("builtin/privacy.toml")),
    (
        "platform_extra.toml",
        include_str!("builtin/platform_extra.toml"),
    ),
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
    let mut specs = load_declared();
    // Cache folders of installed apps that no cleaner above knows about.
    let found = super::discover::discover(&specs);
    merge(&mut specs, found);
    specs
}

/// Declared (built-in + user) specs that apply here, without the auto-detected ones.
pub fn load_declared() -> Vec<CleanerSpec> {
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
    if let Some(dir) = user_dir
        && let Ok(rd) = std::fs::read_dir(dir)
    {
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
                        // Judge breadth by the whole pattern (e.g. `/mnt/*/.Trash-*/files/*`),
                        // not just the literal prefix (`/mnt/`).
                        let full = std::path::Path::new(&expanded);
                        assert!(full.components().count() >= 3, "{}: {t} too broad", s.id);
                    } else {
                        assert!(!safety::is_protected(&p), "{}: {t} protected", s.id);
                    }
                }
            }
        }
    }

    // ---- catalog guards: rules every built-in cleaner must keep -------------------------

    fn all_specs() -> Vec<CleanerSpec> {
        load_all_unfiltered(None)
    }

    #[test]
    fn credential_cleaners_are_caution_and_refuse_while_the_app_runs() {
        use crate::engine::spec::Risk;
        let creds: Vec<_> = all_specs()
            .into_iter()
            .filter(|s| s.id.ends_with("-passwords"))
            .collect();
        assert!(
            creds.len() >= 9,
            "expected one per browser, got {}",
            creds.len()
        );
        for s in creds {
            assert_eq!(s.risk, Risk::Caution, "{} deletes passwords", s.id);
            assert!(
                !s.process.is_empty(),
                "{}: must refuse while the browser runs",
                s.id
            );
        }
    }

    #[test]
    fn age_gated_cleaners_have_an_age_and_are_never_safe() {
        use crate::engine::spec::{Action, Risk};
        for s in all_specs() {
            for a in &s.actions {
                if let Action::Delete {
                    age_by_contents,
                    min_age_days,
                    ..
                } = a
                {
                    if *age_by_contents {
                        assert!(
                            *min_age_days > 0,
                            "{}: age_by_contents without min_age_days",
                            s.id
                        );
                        assert_ne!(
                            s.risk,
                            Risk::Safe,
                            "{}: removing installed tools is not safe",
                            s.id
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn no_delete_template_can_match_a_whole_home_or_root() {
        use crate::engine::spec::Action;
        let bad = [
            "/",
            "~",
            "~/",
            "~/*",
            "*",
            "/*",
            "$HOME",
            "${HOME}",
            "$HOME/*",
            "$XDG_CACHE_HOME",
            "$XDG_DATA_HOME",
            "$XDG_CONFIG_HOME",
        ];
        for s in all_specs() {
            for a in &s.actions {
                if !matches!(a, Action::Delete { .. }) {
                    continue;
                }
                for os in ["linux", "macos", "windows"] {
                    for t in a.delete_templates_for(os) {
                        assert!(
                            !bad.contains(&t),
                            "{}: `{t}` would delete far too much",
                            s.id
                        );
                        assert!(
                            t.starts_with(['~', '$', '/', '%']) || t.chars().nth(1) == Some(':'),
                            "{}: `{t}` is relative to the current directory",
                            s.id
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn delete_cleaners_have_a_path_on_every_os_they_claim() {
        use crate::engine::spec::Action;
        for s in all_specs() {
            for os in ["linux", "macos", "windows"] {
                if !(s.os.is_empty() || s.os.iter().any(|o| o == os)) {
                    continue;
                }
                // A cleaner that deletes must point somewhere on each OS it claims, or it
                // is dead weight there (os = [...] narrows it).
                let deletes = s.actions.iter().all(|a| matches!(a, Action::Delete { .. }));
                if deletes && !s.actions.is_empty() {
                    let any = s
                        .actions
                        .iter()
                        .any(|a| !a.delete_templates_for(os).is_empty());
                    // Cross-platform cleaners may legitimately be Linux/macOS-only data; require
                    // at least one OS to have paths, and warn-free parity for explicit os lists.
                    if !s.os.is_empty() {
                        assert!(any, "{}: claims {os} but has no {os} paths", s.id);
                    }
                }
            }
            let some = ["linux", "macos", "windows"].iter().any(|os| {
                s.actions
                    .iter()
                    .any(|a| !a.delete_templates_for(os).is_empty())
            });
            let only_deletes = s.actions.iter().all(|a| matches!(a, Action::Delete { .. }));
            if only_deletes && !s.actions.is_empty() {
                assert!(some, "{}: deletes nothing on any OS", s.id);
            }
        }
    }

    #[test]
    fn ladybird_is_covered_on_every_platform_with_real_profile_layout() {
        use crate::engine::spec::Action;
        let specs = all_specs();
        for id in [
            "br-ladybird-cache",
            "br-ladybird-history",
            "br-ladybird-crash-reports",
        ] {
            let s = specs
                .iter()
                .find(|s| s.id == id)
                .unwrap_or_else(|| panic!("{id} missing"));
            assert_eq!(s.category, "Web Browsers");
            assert!(
                !s.process.is_empty(),
                "{id} must refuse while Ladybird runs"
            );
            for os in ["linux", "macos", "windows"] {
                let t: Vec<&str> = s
                    .actions
                    .iter()
                    .flat_map(|a| a.delete_templates_for(os))
                    .collect();
                assert!(!t.is_empty(), "{id}: no {os} path");
                assert!(
                    t.iter().all(|p| p.contains("Ladybird/Profiles/*/")),
                    "{id}: {t:?}"
                );
            }
            assert!(matches!(s.actions[0], Action::Delete { .. }));
        }
        let cache = specs.iter().find(|s| s.id == "br-ladybird-cache").unwrap();
        let linux: Vec<&str> = cache.actions[0].delete_templates_for("linux");
        assert!(
            linux.iter().any(|p| p.ends_with("/RequestServer")),
            "{linux:?}"
        );
    }

    #[test]
    fn project_cleaners_cover_the_major_ecosystems() {
        let ids: std::collections::HashSet<String> =
            all_specs().into_iter().map(|s| s.id).collect();
        for want in [
            "proj-rust",
            "proj-gradle",
            "proj-node-modules",
            "proj-python-venv",
            "proj-haskell",
            "proj-sbt",
            "proj-mill",
            "proj-kotlin-native-js",
            "proj-ruby-bundle",
            "proj-php-vendor",
            "proj-elixir-deps",
            "proj-xcode-local",
            "proj-react-native",
            "proj-meson-ninja",
            "dev-cargo-target-dir",
            "dev-rustup-old-toolchains",
            "dev-unused-toolchains",
            "deep-os-junk-files",
            "deep-vim-swap",
            "deep-backup-files",
        ] {
            assert!(ids.contains(want), "built-in cleaner `{want}` is missing");
        }
        assert!(ids.len() >= 330, "catalog shrank to {}", ids.len());
    }

    #[test]
    fn project_artifacts_stay_inside_their_project() {
        use crate::engine::spec::Action;
        for spec in load_all_unfiltered(None) {
            for a in &spec.actions {
                if let Action::ProjectArtifacts {
                    markers, artifacts, ..
                } = a
                {
                    assert!(!markers.is_empty(), "{}: no markers", spec.id);
                    for art in artifacts {
                        let p = std::path::Path::new(art);
                        assert!(
                            p.is_relative()
                                && !art.contains("..")
                                && !art.contains('*')
                                && !art.is_empty(),
                            "{}: artifact `{art}` must be a plain relative path",
                            spec.id
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn no_two_builtin_cleaners_delete_the_same_path() {
        use std::collections::HashMap;
        for os in ["linux", "macos", "windows"] {
            let mut owner: HashMap<String, String> = HashMap::new();
            for s in load_all_unfiltered(None) {
                if !(s.os.is_empty() || s.os.iter().any(|o| o == os)) {
                    continue;
                }
                for a in &s.actions {
                    for t in a.delete_templates_for(os) {
                        if let Some(prev) = owner.insert(t.to_string(), s.id.clone()) {
                            assert!(
                                prev == s.id,
                                "{os}: {t} is deleted by both {prev} and {} (double-counted)",
                                s.id
                            );
                        }
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
