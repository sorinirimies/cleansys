//! Every cleaner that works on the user's home must find what it is meant to clean on the
//! platform the tests run on (Linux, macOS and Windows in CI).
//!
//! For each such cleaner a file is planted at each of its path templates inside a fake
//! home, then `cleansys scan --json` runs with that home and must report the cleaner as
//! having something to free. This catches wrong per-OS paths, broken templating (`~`,
//! `$LOCALAPPDATA`, `%APPDATA%`, `${VAR:-default}`) and glob mistakes before a release.

use cleansys_core::engine::spec::Action;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn os() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        "linux"
    }
}

/// The environment the child scan sees: everything points into `home`.
fn fake_env(home: &Path, root: &Path) -> BTreeMap<String, String> {
    let s = |p: PathBuf| p.to_string_lossy().into_owned();
    let mut env = BTreeMap::new();
    for (k, v) in [
        ("CLEANSYS_HOME", s(home.to_path_buf())),
        ("HOME", s(home.to_path_buf())),
        ("USERPROFILE", s(home.to_path_buf())),
        ("XDG_CACHE_HOME", s(home.join(".cache"))),
        ("XDG_DATA_HOME", s(home.join(".local/share"))),
        ("XDG_CONFIG_HOME", s(home.join(".config"))),
        ("XDG_STATE_HOME", s(home.join(".local/state"))),
        ("LOCALAPPDATA", s(home.join("AppData/Local"))),
        ("APPDATA", s(home.join("AppData/Roaming"))),
        ("CLEANSYS_SCAN_ROOTS", s(root.join("projects"))),
        ("CLEANSYS_DEEP_ROOTS", s(root.join("deep"))),
    ] {
        env.insert(k.to_string(), v);
    }
    env
}

/// Plain delete cleaners for this OS that need no filters, age or root.
fn plain_home_cleaners() -> Vec<cleansys_core::engine::spec::CleanerSpec> {
    cleansys_core::engine::registry::load_all_unfiltered(None)
        .into_iter()
        .filter(|s| !s.requires_root && (s.os.is_empty() || s.os.iter().any(|o| o == os())))
        .filter(|s| {
            !s.actions.is_empty()
                && s.actions.iter().all(|a| {
                    matches!(
                        a,
                        Action::Delete {
                            name_regex: None,
                            not_name_regex: None,
                            path_regex: None,
                            not_path_regex: None,
                            min_age_days: 0,
                            age_by_contents: false,
                            ..
                        }
                    )
                })
        })
        .collect()
}

#[test]
fn every_home_cleaner_finds_its_files_on_this_os() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path().join("home");
    let env = fake_env(&home, tmp.path());
    let lookup = |name: &str| env.get(&name.to_ascii_uppercase()).cloned();
    let home_s = home.to_string_lossy().into_owned();

    let mut planted: Vec<String> = Vec::new();
    for spec in plain_home_cleaners() {
        let mut any = false;
        for action in &spec.actions {
            for t in action.delete_templates_for(os()) {
                if t.contains(['[', '{', '?']) {
                    continue; // glob classes: no single file name to plant
                }
                let expanded = cleansys_core::engine::paths::expand(t, &lookup);
                if !expanded.starts_with(&home_s) {
                    continue; // system-wide or temp paths are out of scope here
                }
                let dir = PathBuf::from(expanded.replace('*', "x"));
                if std::fs::create_dir_all(&dir).is_ok()
                    && std::fs::write(dir.join("planted.bin"), vec![7u8; 4096]).is_ok()
                {
                    any = true;
                }
            }
        }
        if any {
            planted.push(spec.id.clone());
        }
    }
    assert!(
        planted.len() > 50,
        "only {} cleaners could be planted",
        planted.len()
    );
    eprintln!("checking {} home cleaners on {}", planted.len(), os());

    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_cleansys"));
    cmd.env_clear();
    for keep in [
        "PATH",
        "SYSTEMROOT",
        "SystemRoot",
        "WINDIR",
        "TMPDIR",
        "TEMP",
        "TMP",
    ] {
        if let Some(v) = std::env::var_os(keep) {
            cmd.env(keep, v);
        }
    }
    cmd.envs(&env).arg("scan").arg("--json");
    for id in &planted {
        cmd.arg("--id").arg(id);
    }
    let out = cmd.output().unwrap();
    assert!(
        out.status.success(),
        "scan failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&out.stdout)
        .unwrap_or_else(|e| panic!("not JSON ({e}): {}", String::from_utf8_lossy(&out.stdout)));
    let found: BTreeMap<String, u64> = report
        .as_array()
        .unwrap()
        .iter()
        .map(|c| {
            (
                c["id"].as_str().unwrap().to_string(),
                c["bytes"].as_u64().unwrap_or(0),
            )
        })
        .collect();
    let missed: Vec<&String> = planted
        .iter()
        .filter(|id| found.get(*id).copied().unwrap_or(0) == 0)
        .collect();
    assert!(
        missed.is_empty(),
        "{} of {} cleaners found nothing at their own {} paths: {missed:?}",
        missed.len(),
        planted.len(),
        os()
    );
}
