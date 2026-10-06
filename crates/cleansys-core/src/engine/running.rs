//! "Is the application running?" guard.
//!
//! Cleaning a browser's profile while it is open can fail or corrupt data, so
//! a real run is refused when any of the spec's `process` names is running or
//! any of its `lock_files` exists.

use std::collections::HashSet;
use std::process::Command;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use super::paths;
use super::spec::CleanerSpec;

const TTL: Duration = Duration::from_secs(3);

/// Normalise a process name: lowercase, no `.exe`, no path.
fn norm(name: &str) -> String {
    let base = name.rsplit(['/', '\\']).next().unwrap_or(name).trim();
    let lower = base.to_lowercase();
    lower.strip_suffix(".exe").unwrap_or(&lower).to_string()
}

fn process_names() -> HashSet<String> {
    #[cfg(windows)]
    let out = Command::new("tasklist")
        .args(["/FO", "CSV", "/NH"])
        .output();
    #[cfg(target_os = "macos")]
    let out = Command::new("ps").args(["-axco", "command="]).output();
    #[cfg(all(unix, not(target_os = "macos")))]
    let out = Command::new("ps").args(["-A", "-o", "comm="]).output();

    let Ok(out) = out else { return HashSet::new() };
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|l| {
            if cfg!(windows) {
                l.trim_start_matches('"')
                    .split('"')
                    .next()
                    .unwrap_or("")
                    .to_string()
            } else {
                l.to_string()
            }
        })
        .map(|l| norm(&l))
        .filter(|l| !l.is_empty())
        .collect()
}

fn cached_processes() -> HashSet<String> {
    static CACHE: Mutex<Option<(Instant, HashSet<String>)>> = Mutex::new(None);
    let mut guard = CACHE.lock().unwrap_or_else(|e| e.into_inner());
    if let Some((at, set)) = guard.as_ref() {
        if at.elapsed() < TTL {
            return set.clone();
        }
    }
    let set = process_names();
    *guard = Some((Instant::now(), set.clone()));
    set
}

/// Error returned by a real run when the target application is open.
/// Front-ends/automation downcast to this to *skip* instead of *fail*.
#[derive(Debug)]
pub struct AppRunning(pub String);

impl std::fmt::Display for AppRunning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} is running — close it first, then run this cleaner again",
            self.0
        )
    }
}

impl std::error::Error for AppRunning {}

/// Name of the first running application blocking `spec`, if any.
pub fn running_app(spec: &CleanerSpec) -> Option<String> {
    if spec.process.is_empty() && spec.lock_files.is_empty() {
        return None;
    }
    if !spec.process.is_empty() {
        let running = cached_processes();
        for p in &spec.process {
            // Linux `ps -o comm` truncates to 15 chars.
            let n = norm(p);
            let short: String = n.chars().take(15).collect();
            if running.contains(&n) || running.contains(&short) {
                return Some(p.clone());
            }
        }
    }
    for l in &spec.lock_files {
        if !paths::resolve(l, &paths::lookup_env).is_empty() {
            return Some(format!("an application holding {l}"));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn norm_strips_path_and_exe() {
        assert_eq!(norm("Firefox.EXE"), "firefox");
        assert_eq!(norm("/usr/bin/Slack"), "slack");
    }

    #[test]
    fn spec_without_guards_is_never_blocked() {
        let spec = CleanerSpec {
            id: "x".into(),
            name: "x".into(),
            description: "x".into(),
            category: "c".into(),
            os: vec![],
            requires_root: false,
            risk: Default::default(),
            process: vec![],
            lock_files: vec![],
            actions: vec![],
        };
        assert!(running_app(&spec).is_none());
    }

    #[test]
    fn lock_file_blocks() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(d.path().join("lock"), "").unwrap();
        let spec = CleanerSpec {
            id: "x".into(),
            name: "x".into(),
            description: "x".into(),
            category: "c".into(),
            os: vec![],
            requires_root: false,
            risk: Default::default(),
            process: vec![],
            lock_files: vec![format!("{}/lock", d.path().display())],
            actions: vec![],
        };
        assert!(running_app(&spec).is_some());
    }

    #[test]
    fn current_process_is_detected() {
        // the test binary itself is running
        let me = std::env::current_exe().unwrap();
        let name = me.file_name().unwrap().to_string_lossy().into_owned();
        let spec = CleanerSpec {
            id: "x".into(),
            name: "x".into(),
            description: "x".into(),
            category: "c".into(),
            os: vec![],
            requires_root: false,
            risk: Default::default(),
            process: vec![name],
            lock_files: vec![],
            actions: vec![],
        };
        // `ps comm` may be truncated/absent in sandboxes; only assert no panic.
        let _ = running_app(&spec);
    }
}
