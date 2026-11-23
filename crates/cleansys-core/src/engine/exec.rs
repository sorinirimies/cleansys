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
const MAX_VISITED_DIRS: usize = 200_000;

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
    if !opts.dry_run
        && let Some(app) = super::running::running_app(spec)
    {
        return Err(super::running::AppRunning(app).into());
    }
    let mut result = CleaningResult::new();
    if !opts.dry_run {
        crate::utils::clear_size_cache();
    }
    for action in &spec.actions {
        match action {
            Action::Delete {
                contents_only,
                recreate,
                label,
                sudo,
                files_only,
                name_regex,
                not_name_regex,
                path_regex,
                not_path_regex,
                min_age_days,
                age_by_contents,
                ..
            } => {
                let label = label.as_deref().unwrap_or(&spec.name);
                let elevated = sudo.unwrap_or(spec.requires_root);
                let Some(filter) = PathFilter::new(
                    name_regex.as_deref(),
                    not_name_regex.as_deref(),
                    path_regex.as_deref(),
                    not_path_regex.as_deref(),
                ) else {
                    warn!("{}: invalid regex in cleaner; skipping action", spec.id);
                    continue;
                };
                let filter = filter
                    .older_than_days(*min_age_days)
                    .by_contents(*age_by_contents);
                for template in action.delete_templates() {
                    for path in paths::resolve(template, &paths::lookup_env) {
                        let targets: Vec<PathBuf> = if *files_only {
                            files_below(&path, &filter)
                        } else if *contents_only {
                            let rd = fs::read_dir(&path).into_iter().flatten().flatten();
                            let mut kids: Vec<PathBuf> = if opts.dry_run {
                                // Start measuring each entry while the listing is still running.
                                crate::utils::size_while_discovering(
                                    rd.map(|e| e.path()).filter(|t| filter.accepts(t)),
                                )
                            } else {
                                rd.map(|e| e.path()).collect()
                            };
                            kids.sort();
                            kids
                        } else {
                            vec![path.clone()]
                        };
                        let recreate_here = *recreate && !*contents_only && !*files_only;
                        let targets: Vec<PathBuf> =
                            targets.into_iter().filter(|t| filter.accepts(t)).collect();
                        if opts.dry_run {
                            crate::utils::prewarm_sizes(&targets);
                        }
                        for t in targets {
                            remove_path(
                                &mut result,
                                &t,
                                label,
                                opts,
                                recreate_here,
                                elevated,
                                cfg,
                            )?;
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
                let found = find_project_artifacts(markers, artifacts, cfg);
                if opts.dry_run {
                    let arts: Vec<PathBuf> = found.iter().map(|(_, a)| a.clone()).collect();
                    crate::utils::prewarm_sizes(&arts);
                }
                for (project, artifact) in found {
                    let name = project
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_default();
                    let item_label = format!("{label}: {name}");
                    remove_path(&mut result, &artifact, &item_label, opts, false, false, cfg)?;
                }
            }
            Action::LargeFiles {
                roots,
                min_mb,
                min_age_days,
                extensions,
                label,
            } => {
                let label = label.as_deref().unwrap_or(&spec.name);
                let query = super::deep::Query {
                    roots: deep_roots_for(roots, cfg),
                    min_bytes: min_mb.saturating_mul(1 << 20),
                    min_age: Duration::from_secs(min_age_days.saturating_mul(86_400)),
                    extensions: extensions.iter().map(|e| e.to_ascii_lowercase()).collect(),
                };
                for (path, _) in super::deep::find_files(&query, cfg) {
                    remove_path(&mut result, &path, label, opts, false, false, cfg)?;
                }
            }
            Action::Duplicates {
                roots,
                min_mb,
                label,
            } => {
                let label = label.as_deref().unwrap_or(&spec.name);
                let query = super::deep::Query {
                    roots: deep_roots_for(roots, cfg),
                    min_bytes: min_mb.saturating_mul(1 << 20),
                    min_age: Duration::ZERO,
                    extensions: Vec::new(),
                };
                for (path, _) in super::deep::find_duplicates(&query, cfg) {
                    remove_path(&mut result, &path, label, opts, false, false, cfg)?;
                }
            }
        }
    }
    Ok(result)
}

/// The folders a deep action searches: its own `roots`, else the configured deep roots.
fn deep_roots_for(roots: &[String], cfg: &EngineConfig) -> Vec<PathBuf> {
    if roots.is_empty() {
        cfg.effective_deep_roots()
    } else {
        super::config::normalize_roots(
            roots
                .iter()
                .flat_map(|r| paths::resolve(r, &paths::lookup_env))
                .collect(),
        )
    }
}

/// Optional regex filters for `delete` actions.
struct PathFilter {
    name: Option<regex::Regex>,
    not_name: Option<regex::Regex>,
    path: Option<regex::Regex>,
    not_path: Option<regex::Regex>,
    /// Newest of modified/accessed must be before this moment.
    cutoff: Option<std::time::SystemTime>,
    /// Age a directory by the newest file below it (see `Action::Delete::age_by_contents`).
    by_contents: bool,
}

impl PathFilter {
    fn new(
        name: Option<&str>,
        not_name: Option<&str>,
        path: Option<&str>,
        not_path: Option<&str>,
    ) -> Option<Self> {
        let c = |s: Option<&str>| -> Result<Option<regex::Regex>, ()> {
            s.map(|r| regex::Regex::new(r).map_err(|_| ())).transpose()
        };
        Some(Self {
            name: c(name).ok()?,
            not_name: c(not_name).ok()?,
            path: c(path).ok()?,
            not_path: c(not_path).ok()?,
            cutoff: None,
            by_contents: false,
        })
    }

    /// Only accept items untouched (neither written nor read) for `days` (0 = no limit).
    fn older_than_days(mut self, days: u64) -> Self {
        self.cutoff =
            (days > 0).then(|| std::time::SystemTime::now() - Duration::from_secs(days * 86_400));
        self
    }

    fn by_contents(mut self, on: bool) -> Self {
        self.by_contents = on;
        self
    }

    /// Any file below `dir` touched at or after `cutoff`? Stops at the first one;
    /// bounded so a huge tree cannot stall the scan (too big to judge = in use).
    fn touched_below(dir: &Path, cutoff: std::time::SystemTime) -> bool {
        let mut stack = vec![dir.to_path_buf()];
        let mut seen = 0usize;
        while let Some(d) = stack.pop() {
            let Ok(rd) = fs::read_dir(&d) else { continue };
            for e in rd.flatten() {
                seen += 1;
                if seen > 200_000 {
                    return true;
                }
                let Ok(meta) = fs::symlink_metadata(e.path()) else {
                    continue;
                };
                if meta.is_dir() {
                    stack.push(e.path());
                }
                let newest = [meta.modified(), meta.accessed()]
                    .into_iter()
                    .flatten()
                    .max();
                if newest.is_none_or(|t| t >= cutoff) {
                    return true;
                }
            }
        }
        false
    }

    fn is_old_enough(&self, p: &Path) -> bool {
        let Some(cutoff) = self.cutoff else {
            return true;
        };
        let Ok(meta) = fs::symlink_metadata(p) else {
            return false;
        };
        let newest = [meta.modified(), meta.accessed()]
            .into_iter()
            .flatten()
            .max();
        newest.is_some_and(|t| t < cutoff)
            && !(self.by_contents && meta.is_dir() && Self::touched_below(p, cutoff))
    }

    fn accepts(&self, p: &Path) -> bool {
        let full = p.to_string_lossy();
        let name = p
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        self.name.as_ref().is_none_or(|r| r.is_match(&name))
            && self.not_name.as_ref().is_none_or(|r| !r.is_match(&name))
            && self.path.as_ref().is_none_or(|r| r.is_match(&full))
            && self.not_path.as_ref().is_none_or(|r| !r.is_match(&full))
            && self.is_old_enough(p)
    }
}

/// Regular files below `root` that pass `filter` (symlinks are never followed; a plain
/// file yields itself). Filtering while walking keeps memory proportional to the
/// *matches*, not to every file in the tree. Deterministic order.
fn files_below(root: &Path, filter: &PathFilter) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(p) = stack.pop() {
        let Ok(meta) = fs::symlink_metadata(&p) else {
            continue;
        };
        if meta.file_type().is_symlink() {
            continue;
        }
        if meta.is_file() {
            if filter.accepts(&p) {
                out.push(p);
            }
        } else if meta.is_dir()
            && let Ok(rd) = fs::read_dir(&p)
        {
            stack.extend(rd.flatten().map(|e| e.path()));
        }
    }
    out.sort();
    out
}

/// Whether anything this spec targets exists on disk right now. Used to hide
/// cleaners for software that isn't installed. Project scanners are always
/// considered present.
pub fn spec_has_targets(spec: &CleanerSpec) -> bool {
    spec.actions.iter().any(|a| match a {
        Action::ProjectArtifacts { .. } | Action::LargeFiles { .. } | Action::Duplicates { .. } => {
            true
        }
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
    if opts.honor_skips && super::skip::is_skipped(path) {
        debug!("unticked in the details view: {path:?}");
        return Ok(());
    }

    let is_symlink = meta.file_type().is_symlink();
    let is_dir = meta.is_dir() && !is_symlink;
    let size = if opts.dry_run {
        crate::utils::get_size_cached(path)
    } else {
        get_size(&path.to_string_lossy())?
    };
    if size == 0 && !is_symlink {
        return Ok(());
    }

    let make_item = || {
        if is_dir {
            CleanedItem::directory(path.to_path_buf(), size, label).skippable()
        } else {
            CleanedItem::file(path.to_path_buf(), size, label).skippable()
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
        crate::utils::remove_dir_all_parallel(path)
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

/// Device id of a directory (`None` where unsupported or unreadable).
#[cfg(unix)]
pub(super) fn device_of_path(p: &Path) -> Option<u64> {
    use std::os::unix::fs::MetadataExt;
    fs::metadata(p).ok().map(|m| m.dev())
}

#[cfg(not(unix))]
pub(super) fn device_of_path(_: &Path) -> Option<u64> {
    None
}

/// The entry lives on the same filesystem as its parent. Another drive, a network
/// share or a pseudo filesystem mounted below a scan root is never walked: the
/// project scan stays on the drive the root itself is on.
#[cfg(unix)]
pub(super) fn same_device(parent: Option<u64>, e: &fs::DirEntry) -> bool {
    use std::os::unix::fs::MetadataExt;
    match (parent, e.metadata()) {
        (Some(p), Ok(m)) => m.dev() == p,
        _ => true,
    }
}

#[cfg(not(unix))]
pub(super) fn same_device(_: Option<u64>, _: &fs::DirEntry) -> bool {
    true
}

/// Directories whose contents are never searched for nested projects
/// (build output / dependency trees are huge and contain no projects).
pub(super) const SKIP_DESCENT: &[&str] = &[
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

/// One scanned directory that contains at least one *marker* file, plus the names of
/// those marker files. (Only directories with a marker are kept, and only the marker
/// names — an index of every file under ~/Projects would cost hundreds of MB.)
struct DirFiles {
    path: PathBuf,
    files: Vec<String>,
}

/// The finished walk and the marker patterns it was built for.
struct IndexData {
    patterns: Vec<String>,
    dirs: Vec<DirFiles>,
}

type Index = std::sync::Arc<IndexData>;

/// How long a scanned directory index is reused (so a whole category of
/// project cleaners shares a single filesystem walk).
const INDEX_TTL: Duration = Duration::from_secs(30);

fn index_cache() -> &'static std::sync::Mutex<Option<(String, std::time::Instant, Index)>> {
    static CACHE: std::sync::OnceLock<
        std::sync::Mutex<Option<(String, std::time::Instant, Index)>>,
    > = std::sync::OnceLock::new();
    CACHE.get_or_init(|| std::sync::Mutex::new(None))
}

/// One level of the walk: marker files found in `dir`, and its sub-directories to visit.
fn scan_one_dir(
    dir: &Path,
    depth: usize,
    cfg: &EngineConfig,
    patterns: &[glob::Pattern],
) -> (Option<DirFiles>, Vec<PathBuf>) {
    let Ok(rd) = fs::read_dir(dir) else {
        return (None, Vec::new());
    };
    let mut files = Vec::new();
    let mut children = Vec::new();
    let here = device_of_path(dir);
    for e in rd.flatten() {
        let Ok(ft) = e.file_type() else { continue };
        let name = e.file_name();
        let Some(name) = name.to_str() else { continue };
        if ft.is_file() {
            if patterns.iter().any(|p| p.matches(name)) {
                files.push(name.to_owned());
            }
        } else if ft.is_dir()
            && !ft.is_symlink()
            && depth < cfg.max_depth
            && !name.starts_with('.')
            && !SKIP_DESCENT.contains(&name)
            && same_device(here, &e)
        {
            children.push(e.path());
        }
    }
    let found = (!files.is_empty()).then(|| DirFiles {
        path: dir.to_path_buf(),
        files,
    });
    (found, children)
}

/// Walk the configured roots once (non-hidden, non-symlink dirs, bounded depth),
/// level by level with each level spread over the rayon pool, recording the marker
/// files (any of `needed` plus every marker the cleaner definitions use).
fn project_index(cfg: &EngineConfig, needed: &[String]) -> Index {
    // Serialise builds so parallel scans share one walk instead of each starting their
    // own. The walk itself runs on a helper thread (and so on rayon's *global* pool):
    // a scan worker that held this lock while running a nested parallel walk would
    // steal other scan jobs, which would re-walk everything or queue behind the lock,
    // making the scan crawl or appear stuck. Here the waiting worker just blocks.
    static BUILD: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _build = BUILD
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let roots = cfg.effective_roots();
    let key = format!("{roots:?}|{}", cfg.max_depth);

    if let Ok(mut guard) = index_cache().lock() {
        if let Some((k, at, idx)) = guard.as_ref() {
            let covers = needed.iter().all(|n| idx.patterns.contains(n));
            if *k == key && at.elapsed() < INDEX_TTL && covers {
                return idx.clone();
            }
        }
        // Miss/stale: drop old index now so it can't linger while rebuilding.
        guard.take();
    }

    let mut patterns: Vec<String> = super::registry::known_project_markers();
    for n in needed {
        if !patterns.contains(n) {
            patterns.push(n.clone());
        }
    }
    let compiled: Vec<glob::Pattern> = patterns
        .iter()
        .filter_map(|m| glob::Pattern::new(m).ok())
        .collect();

    let dirs = std::thread::scope(|scope| {
        scope
            .spawn(|| {
                super::walk::walk_levels(
                    roots,
                    usize::MAX,
                    MAX_VISITED_DIRS,
                    "project scan",
                    |d, depth| scan_one_dir(d, depth, cfg, &compiled),
                )
            })
            .join()
            .unwrap_or_default()
    });
    let mut dirs = dirs;
    dirs.sort_by(|a, b| a.path.cmp(&b.path));

    let idx: Index = std::sync::Arc::new(IndexData { patterns, dirs });
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
    use rayon::prelude::*;

    // The per-project stat calls (artifact exists? stale?) are independent: run them in parallel.
    let index = project_index(cfg, markers);
    let mut found: Vec<(PathBuf, PathBuf)> = index
        .dirs
        .par_iter()
        .filter(|d| {
            d.files
                .iter()
                .any(|f| patterns.iter().any(|p| p.matches(f)))
        })
        .flat_map_iter(|d| {
            artifacts.iter().filter_map(move |art| {
                let candidate = d.path.join(art);
                let is_real_dir = fs::symlink_metadata(&candidate).is_ok_and(|m| m.is_dir());
                (is_real_dir && is_stale(d, &candidate, min_age))
                    .then(|| (d.path.clone(), candidate))
            })
        })
        .collect();
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
        if let Ok(t) = t
            && t > newest
        {
            newest = t;
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
        .is_ok_and(|age| age >= min_age)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::spec::Risk;

    fn cfg_for(root: &Path) -> EngineConfig {
        EngineConfig {
            scan_roots: vec![root.to_string_lossy().into_owned()],
            min_age_days: 0,
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
        // Paths, not strings: Windows joins components with `\`.
        let names: Vec<&std::path::Path> = gradle
            .iter()
            .map(|(_, a)| a.strip_prefix(r).unwrap())
            .collect();
        for want in ["android/.gradle", "android/build", "android/app/build"] {
            assert!(
                names.contains(&std::path::Path::new(want)),
                "{want}: {names:?}"
            );
        }
        assert_eq!(names.len(), 3);
    }

    #[test]
    fn min_age_keeps_recently_touched_files() {
        let d = tempfile::tempdir().unwrap();
        let old = d.path().join("old.tmp");
        let fresh = d.path().join("fresh.tmp");
        write(&old, 10);
        write(&fresh, 10);
        let three_days_ago = SystemTime::now() - Duration::from_secs(3 * 86_400);
        fs::File::options()
            .write(true)
            .open(&old)
            .unwrap()
            .set_times(
                fs::FileTimes::new()
                    .set_modified(three_days_ago)
                    .set_accessed(three_days_ago),
            )
            .unwrap();

        let any = PathFilter::new(None, None, None, None).unwrap();
        assert!(any.accepts(&fresh), "no age limit accepts everything");

        let aged = any.older_than_days(1);
        assert!(aged.accepts(&old));
        assert!(!aged.accepts(&fresh));
        assert_eq!(files_below(d.path(), &aged), vec![old]);
    }

    #[test]
    fn age_by_contents_keeps_a_directory_whose_files_are_in_use() {
        let d = tempfile::tempdir().unwrap();
        let tool = d.path().join("tool-1.0");
        fs::create_dir_all(tool.join("bin")).unwrap();
        write(&tool.join("bin/run"), 10);
        let old = SystemTime::now() - Duration::from_secs(200 * 86_400);
        let times = fs::FileTimes::new().set_modified(old).set_accessed(old);
        // The directories look idle, the binary inside was just used.
        for dir in [tool.join("bin"), tool.clone()] {
            fs::File::open(&dir).unwrap().set_times(times).unwrap();
        }
        let by_dir = PathFilter::new(None, None, None, None)
            .unwrap()
            .older_than_days(30);
        assert!(
            by_dir.accepts(&tool),
            "judged by the directory alone it looks idle"
        );
        let by_contents = PathFilter::new(None, None, None, None)
            .unwrap()
            .older_than_days(30)
            .by_contents(true);
        assert!(
            !by_contents.accepts(&tool),
            "a freshly used file inside keeps it"
        );
        fs::File::options()
            .write(true)
            .open(tool.join("bin/run"))
            .unwrap()
            .set_times(times)
            .unwrap();
        assert!(by_contents.accepts(&tool), "everything inside is idle too");
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
            process: vec![],
            lock_files: vec![],
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

    #[cfg(unix)]
    #[test]
    fn same_device_accepts_children_on_one_filesystem() {
        let d = tempfile::tempdir().unwrap();
        fs::create_dir_all(d.path().join("sub")).unwrap();
        let here = device_of_path(d.path());
        assert!(here.is_some());
        let e = fs::read_dir(d.path()).unwrap().flatten().next().unwrap();
        assert!(same_device(here, &e));
        // a different device id (a mount point below the root) is refused
        assert!(!same_device(here.map(|x| x.wrapping_add(1)), &e));
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
