//! The command-line interface end to end, in a fake home: scan / clean / auto / usage /
//! config / schedule / themes. Tests that write configuration or delete files run on Linux
//! only, where `XDG_CONFIG_HOME` keeps them out of the developer's real settings.

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::{Path, PathBuf};

/// A `cleansys` whose home, caches, config and project roots all live in `root`.
fn cli(root: &Path) -> Command {
    let home = root.join("home");
    std::fs::create_dir_all(&home).expect("create the fake home");
    let mut cmd = Command::cargo_bin("cleansys").expect("the cleansys binary is built");
    cmd.env_clear();
    for keep in [
        "PATH",
        "SYSTEMROOT",
        "SystemRoot",
        "WINDIR",
        "TEMP",
        "TMP",
        "TMPDIR",
    ] {
        if let Some(v) = std::env::var_os(keep) {
            cmd.env(keep, v);
        }
    }
    let s = |p: PathBuf| p.to_string_lossy().into_owned();
    cmd.env("CLEANSYS_HOME", s(home.clone()))
        .env("HOME", s(home.clone()))
        .env("USERPROFILE", s(home.clone()))
        .env("XDG_CACHE_HOME", s(home.join(".cache")))
        .env("XDG_DATA_HOME", s(home.join(".local/share")))
        .env("XDG_CONFIG_HOME", s(root.join("config")))
        .env("XDG_STATE_HOME", s(home.join(".local/state")))
        .env("LOCALAPPDATA", s(home.join("AppData/Local")))
        .env("APPDATA", s(home.join("AppData/Roaming")))
        .env("CLEANSYS_SCAN_ROOTS", s(root.join("projects")))
        .env("CLEANSYS_DEEP_ROOTS", s(root.join("deep")));
    cmd
}

fn plant(path: &Path, bytes: usize) {
    std::fs::create_dir_all(path.parent().expect("a file path has a parent"))
        .expect("create the parent folder");
    std::fs::write(path, vec![1u8; bytes]).expect("write the planted file");
}

/// npm's cache: a plain user-land cleaner with the same layout on every OS.
fn npm_cache(root: &Path) -> PathBuf {
    root.join("home/.npm/_cacache/content-v2/blob.bin")
}

fn json(out: &[u8]) -> serde_json::Value {
    serde_json::from_slice(out)
        .unwrap_or_else(|e| panic!("not JSON ({e}): {}", String::from_utf8_lossy(out)))
}

#[test]
fn scan_json_reports_the_planted_cache_and_deletes_nothing() {
    let t = tempfile::tempdir().unwrap();
    plant(&npm_cache(t.path()), 50_000);
    let out = cli(t.path())
        .args(["scan", "--json", "--id", "dev-npm-yarn-pnpm-bun"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v = json(&out.stdout);
    let c = &v.as_array().unwrap()[0];
    assert_eq!(c["id"], "dev-npm-yarn-pnpm-bun");
    assert_eq!(c["category"], "Developer Caches");
    assert!(c["bytes"].as_u64().unwrap() >= 50_000);
    assert!(c["items"][0]["path"].as_str().unwrap().contains("_cacache"));
    assert!(npm_cache(t.path()).exists(), "a scan never deletes");
}

#[test]
fn an_unknown_cleaner_id_is_an_error_that_names_it() {
    let t = tempfile::tempdir().unwrap();
    cli(t.path())
        .args(["scan", "--id", "no-such-cleaner"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("no-such-cleaner"));
}

#[test]
fn scan_by_category_returns_only_that_category() {
    let t = tempfile::tempdir().unwrap();
    plant(&npm_cache(t.path()), 10_000);
    let out = cli(t.path())
        .args(["scan", "--json", "--category", "Developer Caches"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v = json(&out.stdout);
    let all = v.as_array().unwrap();
    assert!(!all.is_empty());
    assert!(all.iter().all(|c| c["category"] == "Developer Caches"));
    assert!(
        all.iter()
            .any(|c| c["id"] == "dev-npm-yarn-pnpm-bun" && c["bytes"].as_u64() > Some(0))
    );
}

#[test]
fn clean_dry_run_reports_but_keeps_the_files() {
    let t = tempfile::tempdir().unwrap();
    plant(&npm_cache(t.path()), 20_000);
    let out = cli(t.path())
        .args([
            "clean",
            "--dry-run",
            "--yes",
            "--json",
            "--id",
            "dev-npm-yarn-pnpm-bun",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(String::from_utf8_lossy(&out.stdout).contains("dev-npm-yarn-pnpm-bun"));
    assert!(npm_cache(t.path()).exists(), "--dry-run must not delete");
}

#[cfg(target_os = "linux")]
#[test]
fn clean_for_real_removes_only_what_the_cleaner_owns() {
    let t = tempfile::tempdir().unwrap();
    let cache = npm_cache(t.path());
    let neighbour = t.path().join("home/.npm/keep-me.txt");
    plant(&cache, 20_000);
    plant(&neighbour, 10);
    cli(t.path())
        .args(["clean", "--yes", "--id", "dev-npm-yarn-pnpm-bun"])
        .assert()
        .success();
    assert!(!cache.exists(), "the cache is gone");
    assert!(neighbour.exists(), "files beside it stay");
}

#[test]
fn auto_dry_run_lists_recommended_cleaners_without_deleting() {
    let t = tempfile::tempdir().unwrap();
    plant(&npm_cache(t.path()), 20_000);
    let out = cli(t.path())
        .args(["auto", "--dry-run", "--json"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(json(&out.stdout).is_array());
    assert!(npm_cache(t.path()).exists());
}

#[test]
fn usage_shows_the_biggest_folder_first() {
    let t = tempfile::tempdir().unwrap();
    let dir = t.path().join("tree");
    plant(&dir.join("big/a.bin"), 300_000);
    plant(&dir.join("small/b.bin"), 1_000);
    let out = cli(t.path())
        .args([
            "usage",
            &dir.to_string_lossy(),
            "--depth",
            "1",
            "--top",
            "5",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text = String::from_utf8_lossy(&out.stdout);
    let (big, small) = (text.find("big"), text.find("small"));
    assert!(big.is_some() && small.is_some(), "{text}");
    assert!(big < small, "biggest first:\n{text}");
}

#[test]
fn list_themes_offers_every_theme_and_an_unknown_theme_is_refused() {
    let t = tempfile::tempdir().unwrap();
    let out = cli(t.path()).arg("--list-themes").output().unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        text.lines().count() > 100,
        "{} themes",
        text.lines().count()
    );
    assert!(text.lines().any(|l| l == "System") && text.lines().any(|l| l == "Pip Boy"));
    cli(t.path())
        .args(["--theme", "definitely-not-a-theme", "list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown theme"));
}

#[cfg(target_os = "linux")]
#[test]
fn config_changes_are_saved_and_shown() {
    let t = tempfile::tempdir().unwrap();
    let projects = t.path().join("code");
    std::fs::create_dir_all(&projects).unwrap();
    let p = projects.to_string_lossy().into_owned();
    for args in [
        vec!["config", "add-root", p.as_str()],
        vec!["config", "exclude", "**/keep-this/**"],
        vec!["config", "min-age", "30"],
    ] {
        cli(t.path()).args(&args).assert().success();
    }
    let shown = cli(t.path()).args(["config", "show"]).output().unwrap();
    let text = String::from_utf8_lossy(&shown.stdout);
    assert!(text.contains("keep-this") && text.contains("30"), "{text}");
    assert!(
        text.contains("from CLEANSYS_SCAN_ROOTS"),
        "says the env overrides it: {text}"
    );
    assert!(
        text.contains(&format!("saved in engine.json: {p}")),
        "{text}"
    );
    // The session's CLEANSYS_SCAN_ROOTS (set by `cli`) must never be written to disk.
    let engine = t.path().join("config/cleansys/engine.json");
    let saved = std::fs::read_to_string(&engine).unwrap();
    assert!(saved.contains(&p), "{saved}");
    assert!(
        !saved.contains("projects"),
        "env override leaked into engine.json: {saved}"
    );
    let no_env = cli(t.path())
        .env_remove("CLEANSYS_SCAN_ROOTS")
        .args(["config", "show"])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&no_env.stdout);
    assert!(text.contains("(configured)") && text.contains(&p), "{text}");
    cli(t.path())
        .args(["config", "remove-root", p.as_str()])
        .assert()
        .success();
    let saved = std::fs::read_to_string(&engine).unwrap();
    assert!(!saved.contains(&p), "root removed: {saved}");
}

#[cfg(target_os = "linux")]
#[test]
fn schedule_show_works_without_a_schedule() {
    let t = tempfile::tempdir().unwrap();
    cli(t.path()).args(["schedule", "show"]).assert().success();
}

#[cfg(target_os = "linux")]
#[test]
fn scanning_lists_switches_a_category_off_and_refuses_unknown_names() {
    let t = tempfile::tempdir().unwrap();
    let list = |t: &std::path::Path| {
        String::from_utf8_lossy(&cli(t).arg("scanning").output().unwrap().stdout).to_string()
    };
    let before = list(t.path());
    assert!(before.contains("Project Build Artifacts") && before.contains("off"));
    cli(t.path())
        .args(["scanning", "on", "project build artifacts"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Scanning on for category"));
    let line = |s: &str| {
        s.lines()
            .find(|l| l.contains("Project Build Artifacts"))
            .unwrap()
            .to_string()
    };
    assert!(line(&list(t.path())).trim_end().ends_with("on"));
    cli(t.path())
        .args(["scanning", "off", "no-such-thing"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown category or cleaner id"));
}
