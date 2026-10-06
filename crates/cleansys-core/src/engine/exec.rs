//! Executes [`CleanerSpec`]s: path deletion and project-artifact scanning.

use anyhow::Result;
use log::{debug, warn};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use super::config::EngineConfig;
use super::paths;
use super::safety::{is_excluded, is_protected};
use super::spec::{Action, CleanerSpec};
use crate::cleaners::cleaned_item::{CleanedItem, CleaningResult, RunOptions};
use crate::utils::{confirm, format_size, get_size, print_success};

/// Hard cap on directories visited per project scan (runaway protection).
const MAX_VISITED_DIRS: usize = 300_000;

/// Run every action of `spec` using the on-disk [`EngineConfig`].
pub fn run_spec(spec: &CleanerSpec, opts: RunOptions) -> Result<CleaningResult> {
    run_spec_with(spec, opts, &EngineConfig::load())
}

/// Like [`run_spec`] with an explicit config (used by tests).
pub fn run_spec_with(
    spec: &CleanerSpec,
    opts: RunOptions,
    cfg: &EngineConfig,
) -> Result<CleaningResult> {
    let mut result = CleaningResult::new();
    for action in &spec.actions {
        match action {
            Action::Delete {
                contents_only,
                recreate,
                label,
                sudo,
                ..
            } => {
                let label = label.as_deref().unwrap_or(&spec.name);
                let elevated = sudo.unwrap_or(spec.requires_root);
                for template in action.delete_templates() {
                    for path in paths::resolve(template, &paths::lookup_env) {
                        if *contents_only {
                            if let Ok(rd) = fs::read_dir(&path) {
                                let mut kids: Vec<PathBuf> =
                                    rd.flatten().map(|e| e.path()).collect();
                                kids.sort();
                                for kid in kids {
                                    remove_path(
                                        &mut result,
                                        &kid,
                                        label,
                                        opts,
                                        false,
                                        elevated,
                                        cfg,
                                    )?;
                                }
                            }
                        } else {
                            remove_path(&mut result, &path, label, opts, *recreate, elevated, cfg)?;
                        }
                    }
                }
            }
            Action::Command {
                program,
                args,
                sudo,
                measure,
                label,
            } => {
                let label = label.as_deref().unwrap_or(&spec.name);
                run_command(&mut result, program, args, *sudo, measure, label, opts)?;
            }
            Action::ProjectArtifacts {
                markers,
                artifacts,
                label,
            } => {
                let label = label.as_deref().unwrap_or(&spec.name);
                for (project, artifact) in find_project_artifacts(markers, artifacts, cfg) {
                    let name = project
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_default();
                    let item_label = format!("{label}: {name}");
                    remove_path(&mut result, &artifact, &item_label, opts, false, false, cfg)?;
                }
            }
        }
    }
    Ok(result)
}

/// Whether anything this spec targets exists on disk right now. Used to hide
/// cleaners for software that isn't installed. Project scanners are always
/// considered present.
pub fn spec_has_targets(spec: &CleanerSpec) -> bool {
    spec.actions.iter().any(|a| match a {
        Action::ProjectArtifacts { .. } => true,
        Action::Command { program, .. } => paths::find_program(program).is_some(),
        Action::Delete { .. } => a
            .delete_templates()
            .iter()
            .any(|t| !paths::resolve(t, &paths::lookup_env).is_empty()),
    })
}

/// Remove (or, in preview mode, merely record) one path.
pub(crate) fn remove_path(
    result: &mut CleaningResult,
    path: &Path,
    label: &str,
    opts: RunOptions,
    recreate: bool,
    elevated: bool,
    cfg: &EngineConfig,
) -> Result<()> {
    let Ok(meta) = fs::symlink_metadata(path) else {
        return Ok(());
    };
    if is_protected(path) {
        warn!("refusing to touch protected path {path:?}");
        return Ok(());
    }
    if is_excluded(path, &cfg.exclude) {
        debug!("excluded by user pattern: {path:?}");
        return Ok(());
    }

    let is_symlink = meta.file_type().is_symlink();
    let is_dir = meta.is_dir() && !is_symlink;
    let size = get_size(&path.to_string_lossy())?;
    if size == 0 && !is_symlink {
        return Ok(());
    }

    let make_item = || {
        if is_dir {
            CleanedItem::directory(path.to_path_buf(), size, label)
        } else {
            CleanedItem::file(path.to_path_buf(), size, label)
        }
    };

    if opts.dry_run {
        result.add_item(make_item());
        return Ok(());
    }

    if !opts.skip_confirmation
        && !confirm(
            &format!(
                "Clean {label} at {path:?} ({} to be freed)?",
                format_size(size)
            ),
            true,
        )?
    {
        return Ok(());
    }

    let removal = if elevated && cfg!(unix) && !crate::utils::check_root() {
        // Privileged path: delegate to `sudo rm -rf` (the path already passed
        // the protected/exclusion checks above and is absolute).
        let p = path.to_string_lossy();
        match crate::utils::execute_with_sudo("rm", &["-rf", "--", p.as_ref()]) {
            Ok(o) if o.status.success() => Ok(()),
            Ok(o) => Err(std::io::Error::other(
                String::from_utf8_lossy(&o.stderr).trim().to_string(),
            )),
            Err(e) => Err(std::io::Error::other(e.to_string())),
        }
    } else if is_dir {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    };
    if let Err(e) = removal {
        warn!("Failed to remove {label} at {path:?}: {e}");
        return Ok(());
    }
    if recreate && is_dir {
        fs::create_dir_all(path).ok();
    }

    print_success(&format!(
        "Cleaned {label} at {path:?} ({})",
        format_size(size)
    ));
    result.add_item(make_item());
    Ok(())
}

/// Directories whose contents are never searched for nested projects
/// (build output / dependency trees are huge and contain no projects).
const SKIP_DESCENT: &[&str] = &[
    "node_modules",
    "target",
    "build",
    "dist",
    "out",
    "Pods",
    "venv",
    "site-packages",
    "__pycache__",
    "_build",
    "zig-cache",
    "zig-out",
    "DerivedData",
    "Carthage",
];

/// One scanned directory: its path and the names of the files directly in it.
struct DirFiles {
    path: PathBuf,
    files: Vec<String>,
}

type Index = std::sync::Arc<Vec<DirFiles>>;

/// How long a scanned directory index is reused (so a whole category of
/// project cleaners shares a single filesystem walk).
const INDEX_TTL: Duration = Duration::from_secs(30);

fn index_cache() -> &'static std::sync::Mutex<Option<(String, std::time::Instant, Index)>> {
    static CACHE: std::sync::OnceLock<
        std::sync::Mutex<Option<(String, std::time::Instant, Index)>>,
    > = std::sync::OnceLock::new();
    CACHE.get_or_init(|| std::sync::Mutex::new(None))
}

/// Walk the configured roots once (non-hidden, non-symlink dirs, bounded
/// depth) and record the files in every directory.
fn project_index(cfg: &EngineConfig) -> Index {
    let roots = cfg.effective_roots();
    let key = format!("{roots:?}|{}", cfg.max_depth);
    if let Ok(guard) = index_cache().lock() {
        if let Some((k, at, idx)) = guard.as_ref() {
            if *k == key && at.elapsed() < INDEX_TTL {
                return idx.clone();
            }
        }
    }

    let mut out = Vec::new();
    let mut visited = 0usize;
    'roots: for root in roots {
        let mut stack = vec![(root, 0usize)];
        while let Some((dir, depth)) = stack.pop() {
            visited += 1;
            if visited > MAX_VISITED_DIRS {
                warn!("project scan aborted: too many directories");
                break 'roots;
            }
            let Ok(rd) = fs::read_dir(&dir) else { continue };
            let mut files = Vec::new();
            for e in rd.flatten() {
                let Ok(ft) = e.file_type() else { continue };
                let Some(name) = e.file_name().to_str().map(str::to_owned) else {
                    continue;
                };
                if ft.is_file() {
                    files.push(name);
                } else if ft.is_dir()
                    && !ft.is_symlink()
                    && depth < cfg.max_depth
                    && !name.starts_with('.')
                    && !SKIP_DESCENT.contains(&name.as_str())
                {
                    stack.push((e.path(), depth + 1));
                }
            }
            if !files.is_empty() {
                out.push(DirFiles { path: dir, files });
            }
        }
    }
    let idx: Index = std::sync::Arc::new(out);
    if let Ok(mut guard) = index_cache().lock() {
        *guard = Some((key, std::time::Instant::now(), idx.clone()));
    }
    idx
}

/// Run an external program, reporting bytes freed from `measure` paths.
fn run_command(
    result: &mut CleaningResult,
    program: &str,
    args: &[String],
    sudo: bool,
    measure: &[String],
    label: &str,
    opts: RunOptions,
) -> Result<()> {
    if paths::find_program(program).is_none() {
        debug!("{program} not found; skipping {label}");
        return Ok(());
    }
    let measured: Vec<PathBuf> = measure
        .iter()
        .flat_map(|t| paths::resolve(t, &paths::lookup_env))
        .filter(|p| !is_protected(p))
        .collect();
    let before: Vec<u64> = measured
        .iter()
        .map(|p| get_size(&p.to_string_lossy()).unwrap_or(0))
        .collect();

    if opts.dry_run {
        for (p, b) in measured.iter().zip(&before) {
            if *b > 0 {
                result.add_item(CleanedItem::directory(p.clone(), *b, label));
            }
        }
        return Ok(());
    }

    let args: Vec<String> = args
        .iter()
        .map(|a| paths::expand(a, &paths::lookup_env))
        .collect();
    let shown = format!("{program} {}", args.join(" "));
    if !opts.skip_confirmation && !confirm(&format!("{label}: run `{}`?", shown.trim()), true)? {
        return Ok(());
    }

    let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let output = if sudo {
        crate::utils::execute_with_sudo(program, &arg_refs)?
    } else {
        std::process::Command::new(program)
            .args(&arg_refs)
            .output()?
    };
    if !output.status.success() {
        anyhow::bail!(
            "{label}: `{}` failed: {}",
            shown.trim(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    let mut freed_any = false;
    for (p, b) in measured.iter().zip(&before) {
        let after = get_size(&p.to_string_lossy()).unwrap_or(*b);
        let freed = b.saturating_sub(after);
        if freed > 0 {
            freed_any = true;
            result.add_item(CleanedItem::directory(p.clone(), freed, label));
        }
    }
    if freed_any || measured.is_empty() {
        print_success(&format!("{label}: ran `{}`", shown.trim()));
    }
    Ok(())
}

/// Scan project roots and return `(project_dir, artifact_dir)` pairs.
pub fn find_project_artifacts(
    markers: &[String],
    artifacts: &[String],
    cfg: &EngineConfig,
) -> Vec<(PathBuf, PathBuf)> {
    let patterns: Vec<glob::Pattern> = markers
        .iter()
        .filter_map(|m| glob::Pattern::new(m).ok())
        .collect();
    let min_age = Duration::from_secs(cfg.min_age_days.saturating_mul(86_400));
    let mut found = Vec::new();

    for d in project_index(cfg).iter() {
        if !d
            .files
            .iter()
            .any(|f| patterns.iter().any(|p| p.matches(f)))
        {
            continue;
        }
        for art in artifacts {
            let candidate = d.path.join(art);
            let is_real_dir = fs::symlink_metadata(&candidate)
                .map(|m| m.is_dir())
                .unwrap_or(false);
            if is_real_dir && is_stale(d, &candidate, min_age) {
                found.push((d.path.clone(), candidate));
            }
        }
    }
    found.sort();
    found.dedup();
    found
}

/// A project is "stale" when nothing in it (its files, the artifact dir,
/// the artifact's top-level entries) changed within `min_age`.
fn is_stale(project: &DirFiles, artifact: &Path, min_age: Duration) -> bool {
    if min_age.is_zero() {
        return true;
    }
    let mut newest = SystemTime::UNIX_EPOCH;
    let mut bump = |t: std::io::Result<SystemTime>| {
        if let Ok(t) = t {
            if t > newest {
                newest = t;
            }
        }
    };
    for f in &project.files {
        bump(fs::metadata(project.path.join(f)).and_then(|m| m.modified()));
    }
    bump(fs::metadata(artifact).and_then(|m| m.modified()));
    if let Ok(rd) = fs::read_dir(artifact) {
        for e in rd.flatten() {
            bump(e.metadata().and_then(|m| m.modified()));
        }
    }
    SystemTime::now()
        .duration_since(newest)
        .map(|age| age >= min_age)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::spec::Risk;

    fn cfg_for(root: &Path) -> EngineConfig {
        EngineConfig {
            scan_roots: vec![root.to_string_lossy().into_owned()],
            ..EngineConfig::default()
        }
    }

    fn write(p: &Path, bytes: usize) {
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, vec![b'x'; bytes]).unwrap();
    }

    #[test]
    fn finds_rust_and_gradle_artifacts_but_not_unmarked_dirs() {
        let d = tempfile::tempdir().unwrap();
        let r = d.path();
        write(&r.join("rustproj/Cargo.toml"), 1);
        write(&r.join("rustproj/target/debug/x"), 100);
        write(&r.join("android/settings.gradle.kts"), 1);
        write(&r.join("android/app/build.gradle.kts"), 1);
        write(&r.join("android/app/build/out.apk"), 100);
        write(&r.join("android/.gradle/cache"), 100);
        write(&r.join("android/build/x"), 100);
        write(&r.join("stray/target/x"), 100); // no Cargo.toml
        let cfg = cfg_for(r);

        let rust = find_project_artifacts(&["Cargo.toml".into()], &["target".into()], &cfg);
        assert_eq!(rust.len(), 1);
        assert!(rust[0].1.ends_with("rustproj/target"));

        let gradle = find_project_artifacts(
            &[
                "build.gradle".into(),
                "build.gradle.kts".into(),
                "settings.gradle.kts".into(),
            ],
            &["build".into(), ".gradle".into()],
            &cfg,
        );
        let names: Vec<String> = gradle
            .iter()
            .map(|(_, a)| a.strip_prefix(r).unwrap().to_string_lossy().into_owned())
            .collect();
        assert!(names.contains(&"android/.gradle".to_string()), "{names:?}");
        assert!(names.contains(&"android/build".to_string()), "{names:?}");
        assert!(
            names.contains(&"android/app/build".to_string()),
            "{names:?}"
        );
        assert_eq!(names.len(), 3);
    }

    #[test]
    fn dry_run_keeps_files_real_run_deletes() {
        let d = tempfile::tempdir().unwrap();
        let r = d.path();
        write(&r.join("p/Cargo.toml"), 1);
        write(&r.join("p/target/a.bin"), 1000);
        let spec = CleanerSpec {
            id: "t".into(),
            name: "Rust".into(),
            description: "d".into(),
            category: "c".into(),
            os: vec![],
            requires_root: false,
            risk: Risk::Safe,
            actions: vec![Action::ProjectArtifacts {
                markers: vec!["Cargo.toml".into()],
                artifacts: vec!["target".into()],
                label: None,
            }],
        };
        let cfg = cfg_for(r);

        let preview = run_spec_with(&spec, RunOptions::preview(), &cfg).unwrap();
        assert_eq!(preview.total_bytes, 1000);
        assert!(r.join("p/target/a.bin").exists());

        let real = run_spec_with(&spec, RunOptions::execute(), &cfg).unwrap();
        assert_eq!(real.total_bytes, 1000);
        assert!(!r.join("p/target").exists());
        assert!(r.join("p/Cargo.toml").exists());
    }

    #[test]
    fn exclusions_are_respected() {
        let d = tempfile::tempdir().unwrap();
        let r = d.path();
        write(&r.join("keep/Cargo.toml"), 1);
        write(&r.join("keep/target/a"), 10);
        let mut cfg = cfg_for(r);
        cfg.exclude = vec![format!("{}/keep", r.display())];
        let mut res = CleaningResult::new();
        remove_path(
            &mut res,
            &r.join("keep/target"),
            "x",
            RunOptions::execute(),
            false,
            false,
            &cfg,
        )
        .unwrap();
        assert_eq!(res.item_count(), 0);
        assert!(r.join("keep/target/a").exists());
    }

    #[test]
    fn min_age_skips_fresh_projects() {
        let d = tempfile::tempdir().unwrap();
        let r = d.path();
        write(&r.join("p/Cargo.toml"), 1);
        write(&r.join("p/target/a"), 10);
        let mut cfg = cfg_for(r);
        cfg.min_age_days = 30;
        let found = find_project_artifacts(&["Cargo.toml".into()], &["target".into()], &cfg);
        assert!(found.is_empty());
    }

    #[test]
    fn refuses_protected_paths() {
        let mut res = CleaningResult::new();
        remove_path(
            &mut res,
            Path::new("/usr"),
            "x",
            RunOptions::preview(),
            false,
            false,
            &EngineConfig::default(),
        )
        .unwrap();
        assert_eq!(res.item_count(), 0);
    }
}
