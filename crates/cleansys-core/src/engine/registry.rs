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
                    if expanded.is_empty() {
                        continue; // an optional env var such as ${CARGO_TARGET_DIR:-} is unset
                    }
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
                    age_by_contents: true,
                    min_age_days,
                    ..
                } = a
                {
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
    fn sqlite_cleaners_name_only_plain_tables_and_refuse_while_the_app_runs() {
        use crate::engine::exec::is_sql_identifier;
        use crate::engine::spec::Action;
        let mut seen = 0;
        for s in all_specs() {
            for a in &s.actions {
                if let Action::SqliteRows { tables, .. } = a {
                    seen += 1;
                    assert!(!tables.is_empty(), "{}: no tables", s.id);
                    for t in tables {
                        assert!(
                            is_sql_identifier(t),
                            "{}: `{t}` is not a plain identifier",
                            s.id
                        );
                    }
                    assert!(
                        !s.process.is_empty(),
                        "{}: must refuse while the app runs",
                        s.id
                    );
                }
            }
        }
        assert!(seen >= 2, "expected the Ladybird sqlite cleaners");
    }

    #[test]
    fn commands_that_remove_only_part_of_their_folder_preview_only_that_part() {
        use crate::engine::spec::Action;
        for id in [
            "sys-lin-pacman-orphans-cache",
            "sys-lin-snap-old-revisions",
            "sys-lin-journal",
            "sys-lin-old-kernels",
        ] {
            let spec = all_specs()
                .into_iter()
                .find(|s| s.id == id)
                .unwrap_or_else(|| panic!("{id} missing"));
            let has = spec.actions.iter().any(|a| {
                matches!(a, Action::Command { measure, preview_args, .. }
                    if !measure.is_empty() && !preview_args.is_empty())
            });
            assert!(
                has,
                "{id} must list its candidates in preview (preview_args)"
            );
        }
    }

    #[test]
    fn shell_scripts_in_command_cleaners_reach_the_shell_intact() {
        use crate::engine::spec::Action;
        let args_of = |id: &str| -> Vec<String> {
            let s = all_specs().into_iter().find(|s| s.id == id).unwrap();
            s.actions
                .iter()
                .flat_map(|a| match a {
                    Action::Command {
                        args, preview_args, ..
                    } => args.iter().chain(preview_args).cloned().collect(),
                    _ => Vec::new(),
                })
                .map(|a| crate::engine::paths::expand(&a, &crate::engine::paths::lookup_env))
                .collect()
        };
        let snap = args_of("sys-lin-snap-old-revisions").join("\n");
        assert!(snap.contains("print $1, $3"), "awk fields survive: {snap}");
        assert!(
            snap.contains(r#"snap remove "$n" --revision="$r""#),
            "{snap}"
        );
        assert!(
            snap.contains(r#"$1 "_" $3"#),
            "preview builds the file name: {snap}"
        );
        let events = args_of("sys-win-event-logs").join("\n");
        assert!(events.contains("wevtutil.exe cl $_"), "{events}");
    }

    /// The dry-run script of a built-in command cleaner, aimed at `dir` instead of the
    /// real system folder, run with `sh`; its output lines.
    #[cfg(target_os = "linux")]
    fn run_preview_script(
        id: &str,
        real_dir: &str,
        dir: &std::path::Path,
        extra: &[(&str, &str)],
    ) -> Vec<String> {
        use crate::engine::spec::Action;
        let spec = all_specs().into_iter().find(|s| s.id == id).unwrap();
        let Some(Action::Command { preview_args, .. }) = spec.actions.first() else {
            panic!("{id} is not a command cleaner");
        };
        let mut script = crate::engine::paths::expand(&preview_args[1], &|_| None);
        script = script.replace(real_dir, &dir.to_string_lossy());
        for (from, to) in extra {
            script = script.replace(from, to);
        }
        let out = std::process::Command::new("sh")
            .arg("-c")
            .arg(&script)
            .output()
            .unwrap();
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .map(str::to_string)
            .collect()
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn journal_preview_lists_the_oldest_archives_until_the_limit_is_met() {
        let d = tempfile::tempdir().unwrap();
        let mid = d.path().join("machine");
        std::fs::create_dir_all(&mid).unwrap();
        // 1 MiB each, real bytes (disk usage counts, not apparent size).
        let file = |name: &str, age_days: u64| {
            let p = mid.join(name);
            std::fs::write(&p, vec![7u8; 1 << 20]).unwrap();
            let t =
                std::time::SystemTime::now() - std::time::Duration::from_secs(age_days * 86_400);
            std::fs::File::options()
                .write(true)
                .open(&p)
                .unwrap()
                .set_times(std::fs::FileTimes::new().set_modified(t))
                .unwrap();
        };
        file("system.journal", 0); // active: never removed
        file("system@a-1.journal", 30);
        file("system@a-2.journal", 20);
        file("user@b-3.journal", 10);
        // keep 2.5 MiB of the 4 MiB: drop the two oldest archives (4 -> 3 -> 2).
        let out = run_preview_script(
            "sys-lin-journal",
            "/var/log/journal",
            d.path(),
            &[("keep=209715200", "keep=2621440")],
        );
        let names: Vec<&str> = out.iter().map(|l| l.rsplit('/').next().unwrap()).collect();
        assert_eq!(
            names,
            ["system@a-1.journal", "system@a-2.journal"],
            "{out:?}"
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn old_kernel_preview_keeps_the_newest_and_the_running_kernel() {
        let d = tempfile::tempdir().unwrap();
        for v in ["6.8.0-9-generic", "6.8.0-40-generic", "6.8.0-45-generic"] {
            for kind in ["vmlinuz", "initrd.img", "System.map"] {
                std::fs::write(d.path().join(format!("{kind}-{v}")), b"k").unwrap();
            }
        }
        // Pretend the oldest kernel is the one running.
        let out = run_preview_script(
            "sys-lin-old-kernels",
            "/boot",
            d.path(),
            &[("$(uname -r)", "6.8.0-9-generic")],
        );
        let mut names: Vec<&str> = out.iter().map(|l| l.rsplit('/').next().unwrap()).collect();
        names.sort_unstable();
        assert_eq!(
            names,
            [
                "System.map-6.8.0-40-generic",
                "initrd.img-6.8.0-40-generic",
                "vmlinuz-6.8.0-40-generic"
            ],
            "only the old kernel that is not running (6.8.0-9 is, 6.8.0-45 is newest)"
        );
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
                        // Two cleaners may share one SQLite file as long as their tables differ.
                        let keys: Vec<String> = match a {
                            crate::engine::spec::Action::SqliteRows { tables, .. } => {
                                tables.iter().map(|tb| format!("{t}#{tb}")).collect()
                            }
                            _ => vec![t.to_string()],
                        };
                        for key in keys {
                            if let Some(prev) = owner.insert(key.clone(), s.id.clone()) {
                                assert!(
                                    prev == s.id,
                                    "{os}: {key} is deleted by both {prev} and {} (double-counted)",
                                    s.id
                                );
                            }
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
