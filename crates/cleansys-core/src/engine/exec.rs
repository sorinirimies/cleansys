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
                let mut filter = filter
                    .older_than_days(*min_age_days)
                    .by_contents(*age_by_contents);
                if *age_by_contents {
                    filter = filter.pinned(pinned_versions(cfg));
                }
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
                preview_args,
                preview_program,
                label,
            } => {
                let label = label.as_deref().unwrap_or(&spec.name);
                let cmd = CommandSpec {
                    program,
                    args,
                    sudo: *sudo,
                    measure,
                    preview_args,
                    preview_program: preview_program.as_deref().unwrap_or(program),
                };
                run_command(&mut result, &cmd, label, opts)?;
            }
            Action::SqliteRows { tables, label, .. } => {
                let label = label.as_deref().unwrap_or(&spec.name);
                for db in action
                    .delete_templates()
                    .iter()
                    .flat_map(|t| paths::resolve(t, &paths::lookup_env))
                {
                    sqlite_rows(&mut result, &db, tables, label, opts, cfg)?;
                }
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
                names,
                label,
            } => {
                let label = label.as_deref().unwrap_or(&spec.name);
                let query = super::deep::Query {
                    roots: deep_roots_for(roots, cfg),
                    min_bytes: min_mb.saturating_mul(1 << 20),
                    min_age: Duration::from_secs(min_age_days.saturating_mul(86_400)),
                    extensions: extensions.iter().map(|e| e.to_ascii_lowercase()).collect(),
                    names: names.iter().map(|n| n.to_ascii_lowercase()).collect(),
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
                    names: Vec::new(),
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
    /// Versions projects pin (`.nvmrc`, `.tool-versions`, …); never removed.
    pinned: std::sync::Arc<std::collections::HashSet<String>>,
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
            pinned: std::sync::Arc::default(),
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

    fn pinned(mut self, pins: std::sync::Arc<std::collections::HashSet<String>>) -> Self {
        self.pinned = pins;
        self
    }

    /// Is `dir` (or its parent, for `dists/gradle-8.5-bin/<hash>`) a version some project pins?
    fn is_pinned(&self, dir: &Path) -> bool {
        if self.pinned.is_empty() {
            return false;
        }
        [dir.file_name(), dir.parent().and_then(Path::file_name)]
            .into_iter()
            .flatten()
            .any(|n| version_matches_pin(&n.to_string_lossy(), &self.pinned))
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
                // A directory's atime moves when this very scan lists it, so only
                // files count by access time.
                let newest = if meta.is_dir() {
                    meta.modified().ok()
                } else {
                    [meta.modified(), meta.accessed()]
                        .into_iter()
                        .flatten()
                        .max()
                };
                if newest.is_none_or(|t| t >= cutoff) {
                    return true;
                }
            }
        }
        false
    }

    /// Is `dir` the version a version manager currently has selected? Either a sibling
    /// (or uncle) symlink such as sdkman's `current` or fvm's `default` points at it, or
    /// a `version` / `alias/default` file (pyenv, rbenv, nodenv, goenv, nvm) names it.
    /// Idle time alone cannot be trusted: a mount without atime never shows use.
    fn is_selected_version(dir: &Path) -> bool {
        let Ok(real) = fs::canonicalize(dir) else {
            return false;
        };
        let name = dir
            .file_name()
            .map(|n| n.to_string_lossy().trim_start_matches('v').to_string())
            .unwrap_or_default();
        let mut anc = dir.parent();
        for _ in 0..3 {
            let Some(a) = anc else { break };
            if let Ok(rd) = fs::read_dir(a) {
                for e in rd.flatten() {
                    let ep = e.path();
                    let Ok(meta) = fs::symlink_metadata(&ep) else {
                        continue;
                    };
                    // `current` -> <version>, or a shim such as `bin/zig` -> <version>/zig
                    let mut links = Vec::new();
                    if meta.file_type().is_symlink() {
                        links.push(ep);
                    } else if meta.is_dir()
                        && e.file_name() == "bin"
                        && let Ok(b) = fs::read_dir(&ep)
                    {
                        links.extend(b.flatten().map(|x| x.path()).take(200));
                    }
                    for l in links {
                        let is_link =
                            fs::symlink_metadata(&l).is_ok_and(|m| m.file_type().is_symlink());
                        if is_link && fs::canonicalize(&l).is_ok_and(|t| t.starts_with(&real)) {
                            return true;
                        }
                    }
                }
            }
            for f in ["version", "alias/default", ".version"] {
                if let Ok(txt) = fs::read_to_string(a.join(f))
                    && txt
                        .lines()
                        .any(|l| l.trim().trim_start_matches('v') == name && !name.is_empty())
                {
                    return true;
                }
            }
            anc = a.parent();
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
        // When judging by contents, a directory's own atime is meaningless: listing
        // it (our own scan, with relatime) bumps it. Files inside decide instead.
        let newest = if self.by_contents && meta.is_dir() {
            meta.modified().ok()
        } else {
            [meta.modified(), meta.accessed()]
                .into_iter()
                .flatten()
                .max()
        };
        newest.is_some_and(|t| t < cutoff)
            && !(self.by_contents
                && meta.is_dir()
                && (Self::is_selected_version(p)
                    || self.is_pinned(p)
                    || Self::touched_below(p, cutoff)))
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
        Action::SqliteRows { .. } => a
            .delete_templates()
            .iter()
            .any(|t| !paths::resolve(t, &paths::lookup_env).is_empty()),
        Action::Delete { .. } => a
            .delete_templates()
            .iter()
            .any(|t| !paths::resolve(t, &paths::lookup_env).is_empty()),
    })
}

/// Is `name` a plain SQL identifier (so it can be spliced into a statement)?
pub(crate) fn is_sql_identifier(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        && !name.starts_with(|c: char| c.is_ascii_digit())
}

/// Preview or delete the rows of `tables` in the SQLite file `db` (the file stays).
/// SQLite is compiled in, so this works on every platform without a `sqlite3` program.
fn sqlite_rows(
    result: &mut CleaningResult,
    db: &Path,
    tables: &[String],
    label: &str,
    opts: RunOptions,
    cfg: &EngineConfig,
) -> Result<()> {
    use rusqlite::{Connection, OpenFlags};
    if is_protected(db) || is_excluded(db, &cfg.exclude) || !db.is_file() {
        return Ok(());
    }
    let tables: Vec<&String> = tables.iter().filter(|t| is_sql_identifier(t)).collect();
    if tables.is_empty() {
        return Ok(());
    }
    let flags = if opts.dry_run {
        OpenFlags::SQLITE_OPEN_READ_ONLY
    } else {
        OpenFlags::SQLITE_OPEN_READ_WRITE
    } | OpenFlags::SQLITE_OPEN_NO_MUTEX;
    let Ok(conn) = Connection::open_with_flags(db, flags) else {
        return Ok(());
    };
    let _ = conn.busy_timeout(Duration::from_secs(2));
    // Measure the row payload; a table this schema version lacks is skipped, and a file
    // that is not a database at all fails here and is left alone.
    let mut rows = 0u64;
    let mut bytes = 0u64;
    let mut present = Vec::new();
    for t in &tables {
        let exists: Result<i64, _> = conn.query_row(
            "SELECT count(*) FROM sqlite_master WHERE type='table' AND name=?1",
            [t.as_str()],
            |r| r.get(0),
        );
        match exists {
            Ok(1) => {}
            Ok(_) => continue,
            Err(e) => {
                debug!("{label}: {db:?} is not a readable database: {e}");
                return Ok(());
            }
        }
        let cols: Vec<String> = conn
            .prepare(&format!("SELECT name FROM pragma_table_info('{t}')"))
            .and_then(|mut st| {
                st.query_map([], |r| r.get::<_, String>(0))?
                    .collect::<Result<Vec<_>, _>>()
            })
            .unwrap_or_default();
        let n: i64 = conn
            .query_row(&format!("SELECT count(*) FROM \"{t}\""), [], |r| r.get(0))
            .unwrap_or(0);
        if n > 0 && !cols.is_empty() {
            let sum = cols
                .iter()
                .map(|c| {
                    format!(
                        "coalesce(length(CAST(\"{}\" AS BLOB)),0)",
                        c.replace('"', "")
                    )
                })
                .collect::<Vec<_>>()
                .join("+");
            let b: i64 = conn
                .query_row(
                    &format!("SELECT coalesce(sum({sum}),0) FROM \"{t}\""),
                    [],
                    |r| r.get(0),
                )
                .unwrap_or(0);
            bytes += u64::try_from(b).unwrap_or(0);
        }
        rows += u64::try_from(n).unwrap_or(0);
        present.push(t.as_str());
    }
    if rows == 0 {
        return Ok(());
    }
    let item_label = format!("{label}: {rows} row(s)");
    if !opts.dry_run {
        let sql = present
            .iter()
            .map(|t| format!("DELETE FROM \"{t}\";"))
            .collect::<Vec<_>>()
            .join(" ");
        if let Err(e) = conn.execute_batch(&format!("BEGIN; {sql} COMMIT;")) {
            warn!("could not clear rows in {db:?} (database busy or locked?): {e}");
            return Ok(());
        }
        let _ = conn.execute_batch("VACUUM;");
    }
    result.add_item(CleanedItem::file(
        db.to_path_buf(),
        bytes.max(rows),
        &item_label,
    ));
    Ok(())
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

    // The pin files go into every walk too: otherwise the runtimes cleaner's first
    // request misses the shared index and forces a second full walk of the projects.
    let mut patterns: Vec<String> = super::registry::known_project_markers();
    for n in PIN_FILES
        .iter()
        .map(|s| (*s).to_string())
        .chain(needed.iter().cloned())
    {
        if !patterns.contains(&n) {
            patterns.push(n);
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
/// The parts of an [`Action::Command`] that `run_command` needs.
struct CommandSpec<'a> {
    program: &'a str,
    args: &'a [String],
    sudo: bool,
    measure: &'a [String],
    preview_args: &'a [String],
    preview_program: &'a str,
}

/// What a dry run of a command would remove: the absolute paths in its output.
/// `None` when the dry run could not be run or failed.
fn dry_run_candidates(program: &str, preview_args: &[String]) -> Option<Vec<PathBuf>> {
    let args: Vec<String> = preview_args
        .iter()
        .map(|a| paths::expand(a, &paths::lookup_env))
        .collect();
    let out = std::process::Command::new(program)
        .args(&args)
        .stdin(std::process::Stdio::null())
        .output()
        .ok()?;
    if !out.status.success() {
        debug!("dry run of {program} failed");
        return None;
    }
    let mut seen = std::collections::HashSet::new();
    Some(
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .map(str::trim)
            .map(PathBuf::from)
            .filter(|p| p.is_absolute() && p.exists() && seen.insert(p.clone()))
            .collect(),
    )
}

fn run_command(
    result: &mut CleaningResult,
    cmd: &CommandSpec<'_>,
    label: &str,
    opts: RunOptions,
) -> Result<()> {
    let CommandSpec {
        program,
        args,
        sudo,
        measure,
        preview_args,
        preview_program,
    } = *cmd;
    if paths::find_program(program).is_none() {
        debug!("{program} not found; skipping {label}");
        return Ok(());
    }
    let measured: Vec<PathBuf> = measure
        .iter()
        .flat_map(|t| paths::resolve(t, &paths::lookup_env))
        .filter(|p| !is_protected(p))
        .collect();
    if opts.dry_run && !preview_args.is_empty() {
        // Report exactly what the command would remove, not the whole folder around it.
        for p in dry_run_candidates(preview_program, preview_args).unwrap_or_default() {
            if is_protected(&p) {
                continue;
            }
            let size = get_size(&p.to_string_lossy()).unwrap_or(0);
            if size > 0 {
                result.add_item(if p.is_dir() {
                    CleanedItem::directory(p, size, label)
                } else {
                    CleanedItem::file(p, size, label)
                });
            }
        }
        return Ok(());
    }
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
                (is_real_dir
                    && is_stale(d, &candidate, min_age)
                    && !holds_tracked_files(&d.path, art))
                .then(|| (d.path.clone(), candidate))
            })
        })
        .collect();
    found.sort();
    found.dedup();
    found
}

/// Files in which projects pin a tool version.
const PIN_FILES: &[&str] = &[
    ".nvmrc",
    ".node-version",
    ".python-version",
    ".ruby-version",
    ".go-version",
    ".java-version",
    ".terraform-version",
    ".tool-versions",
    ".sdkmanrc",
    "rust-toolchain",
    "rust-toolchain.toml",
    "gradle-wrapper.properties",
    "go.mod",
];

/// Channel names that select "whatever is newest", not one installed version.
const FLOATING: &[&str] = &[
    "stable", "beta", "nightly", "latest", "system", "node", "lts", "lts/*", "current",
];

/// The version strings a pin file names (normalised: no leading `v`).
fn pins_in(file_name: &str, text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in text.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let values: Vec<String> = match file_name {
            ".tool-versions" => line
                .split_whitespace()
                .skip(1)
                .map(str::to_string)
                .collect(),
            ".sdkmanrc" => line
                .split_once('=')
                .map(|(_, v)| vec![v.trim().to_string()])
                .unwrap_or_default(),
            "rust-toolchain.toml" => line
                .strip_prefix("channel")
                .and_then(|r| r.split_once('='))
                .map(|(_, v)| vec![v.trim().trim_matches('"').to_string()])
                .unwrap_or_default(),
            "gradle-wrapper.properties" => line
                .strip_prefix("distributionUrl")
                .and_then(|r| r.rsplit('/').next())
                .and_then(|zip| zip.strip_suffix(".zip"))
                .map(|dist| {
                    // gradle-8.5-bin -> both the dist folder name and the bare version
                    let bare = dist
                        .trim_start_matches("gradle-")
                        .trim_end_matches("-bin")
                        .trim_end_matches("-all");
                    vec![dist.to_string(), bare.to_string()]
                })
                .unwrap_or_default(),
            "go.mod" => line
                .strip_prefix("toolchain ")
                .map(|v| vec![v.trim().to_string()])
                .unwrap_or_default(),
            _ => vec![line.to_string()],
        };
        out.extend(
            values
                .into_iter()
                .map(|v| v.trim_start_matches('v').to_string())
                .filter(|v| !v.is_empty() && !FLOATING.contains(&v.as_str())),
        );
    }
    out
}

/// Every version pinned by a project under the scan roots (cached with the project index).
fn pinned_versions(cfg: &EngineConfig) -> std::sync::Arc<std::collections::HashSet<String>> {
    let needed: Vec<String> = PIN_FILES.iter().map(|s| (*s).to_string()).collect();
    let index = project_index(cfg, &needed);
    let mut pins = std::collections::HashSet::new();
    for d in &index.dirs {
        for f in d.files.iter().filter(|f| PIN_FILES.contains(&f.as_str())) {
            if let Ok(text) = fs::read_to_string(d.path.join(f)) {
                pins.extend(pins_in(f, &text));
            }
        }
    }
    std::sync::Arc::new(pins)
}

/// Does installed version folder `name` (`v20.11.0`, `3.12.1`, `21.0.11-zulu`,
/// `nightly-2026-09-15-x86_64-…`, `toolchain@v0.0.1-go1.22.1.linux-amd64`) satisfy a pin?
/// A pin may be partial (`20`, `3.12`): it matches at a `.`/`-`/`_` boundary.
fn version_matches_pin(name: &str, pins: &std::collections::HashSet<String>) -> bool {
    let n = name.trim_start_matches('v');
    pins.iter().any(|p| {
        n == p
            || [".", "-", "_"]
                .iter()
                .any(|sep| n.starts_with(&format!("{p}{sep}")))
            || n.contains(&format!("-{p}."))
    })
}

/// Does git track anything inside `project/artifact`? Some projects commit what is
/// usually build output (`vendor/`, `build/`, Unreal's `Binaries/`); those must never be
/// cleaned. Without git, or outside a repository, nothing counts as tracked.
fn holds_tracked_files(project: &Path, artifact: &str) -> bool {
    std::process::Command::new("git")
        .arg("-C")
        .arg(project)
        .args(["ls-files", "-z", "--", artifact])
        .stdin(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .output()
        .is_ok_and(|o| o.status.success() && !o.stdout.is_empty())
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

    /// Set both times on a file or directory. Windows only opens a directory with
    /// backup semantics, and the handle needs write access to change times.
    fn set_times_of(p: &Path, times: fs::FileTimes) {
        let mut o = fs::OpenOptions::new();
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            o.write(true).custom_flags(0x0200_0000); // FILE_FLAG_BACKUP_SEMANTICS
        }
        #[cfg(not(windows))]
        o.read(true);
        o.open(p).unwrap().set_times(times).unwrap();
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
    fn age_by_contents_never_removes_the_selected_version() {
        let d = tempfile::tempdir().unwrap();
        let root = d.path().join("candidates/java");
        let (used, spare) = (root.join("21.0.11"), root.join("17.0.1"));
        fs::create_dir_all(&used).unwrap();
        fs::create_dir_all(&spare).unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(&used, root.join("current")).unwrap();
        write(&used.join("release"), 1);
        write(&spare.join("release"), 1);
        let old = SystemTime::now() - Duration::from_secs(400 * 86_400);
        let times = fs::FileTimes::new().set_modified(old).set_accessed(old);
        for p in [&used, &spare, &used.join("release"), &spare.join("release")] {
            set_times_of(p, times);
        }
        let f = PathFilter::new(None, None, None, None)
            .unwrap()
            .older_than_days(120)
            .by_contents(true);
        assert!(
            f.accepts(&spare),
            "an idle, unselected version is removable"
        );
        #[cfg(unix)]
        assert!(
            !f.accepts(&used),
            "the version `current` points to must stay"
        );
        // zvm-style: `bin/zig` -> <version>/zig selects the version.
        let zig = d.path().join("zvm");
        fs::create_dir_all(zig.join("0.13.0")).unwrap();
        fs::create_dir_all(zig.join("0.12.0")).unwrap();
        fs::create_dir_all(zig.join("bin")).unwrap();
        write(&zig.join("0.13.0/zig"), 1);
        write(&zig.join("0.12.0/zig"), 1);
        #[cfg(unix)]
        std::os::unix::fs::symlink(zig.join("0.13.0/zig"), zig.join("bin/zig")).unwrap();
        for p in [
            zig.join("0.13.0"),
            zig.join("0.12.0"),
            zig.join("0.13.0/zig"),
            zig.join("0.12.0/zig"),
        ] {
            set_times_of(&p, times);
        }
        assert!(f.accepts(&zig.join("0.12.0")));
        #[cfg(unix)]
        assert!(!f.accepts(&zig.join("0.13.0")), "bin/zig points into it");
        // pyenv-style: a `version` file names the selected one.
        fs::write(d.path().join("candidates/version"), "17.0.1\n").unwrap();
        assert!(
            !f.accepts(&spare),
            "the version named in `version` must stay"
        );
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
            set_times_of(&dir, times);
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

    /// End to end: a TOML-defined "unused runtimes" cleaner removes only the idle,
    /// unselected versions, in preview and for real.
    #[test]
    fn unused_runtime_cleaner_keeps_selected_and_recent_versions_end_to_end() {
        let d = tempfile::tempdir().unwrap();
        let root = d.path().join("sdk");
        let old = SystemTime::now() - Duration::from_secs(300 * 86_400);
        let times = fs::FileTimes::new().set_modified(old).set_accessed(old);
        // nvm layout: versions/node/<v>, alias/default names the selected one.
        for v in ["v18.0.0", "v20.0.0", "v22.0.0"] {
            write(&root.join(format!("versions/node/{v}/bin/node")), 100);
        }
        fs::create_dir_all(root.join("alias")).unwrap();
        fs::write(root.join("alias/default"), "v20.0.0\n").unwrap();
        for v in ["v18.0.0", "v20.0.0"] {
            for p in [
                root.join(format!("versions/node/{v}/bin/node")),
                root.join(format!("versions/node/{v}/bin")),
                root.join(format!("versions/node/{v}")),
            ] {
                set_times_of(&p, times);
            }
        }
        // v22 stays untouched-now (recent), v20 is selected, v18 is the only removable one.
        let src = format!(
            r#"
[[cleaner]]
id = "t-rt"
name = "Runtimes"
description = "d"
[[cleaner.action]]
type = "delete"
paths = ["{}/versions/node/*"]
min_age_days = 120
age_by_contents = true
"#,
            root.display().to_string().replace('\\', "/")
        );
        let spec = crate::engine::registry::parse_spec_file(&src)
            .unwrap()
            .remove(0);
        let cfg = EngineConfig::default();
        let preview = run_spec_with(&spec, RunOptions::preview(), &cfg).unwrap();
        assert_eq!(preview.items.len(), 1, "{:?}", preview.items);
        assert!(
            root.join("versions/node/v18.0.0").exists(),
            "preview deletes nothing"
        );
        run_spec_with(&spec, RunOptions::execute(), &cfg).unwrap();
        assert!(!root.join("versions/node/v18.0.0").exists());
        assert!(
            root.join("versions/node/v20.0.0").exists(),
            "selected via alias/default"
        );
        assert!(root.join("versions/node/v22.0.0").exists(), "recently used");
    }

    /// A Ladybird-shaped database: cookies + storage + a table that must survive.
    fn ladybird_db(path: &Path) {
        let c = rusqlite::Connection::open(path).unwrap();
        c.execute_batch(
            "CREATE TABLE Cookies (name TEXT, value TEXT, domain TEXT);
             CREATE TABLE WebStorage (k TEXT, v TEXT);
             CREATE TABLE SchemaVersions (store TEXT PRIMARY KEY, version INTEGER NOT NULL);
             INSERT INTO Cookies VALUES ('sid','abcdef','example.org'),('t','xyz','example.com');
             INSERT INTO WebStorage VALUES ('theme','dark');
             INSERT INTO SchemaVersions VALUES ('cookies', 4);",
        )
        .unwrap();
    }

    fn count(db: &Path, table: &str) -> String {
        let c = rusqlite::Connection::open(db).unwrap();
        c.query_row(&format!("SELECT count(*) FROM {table}"), [], |r| {
            r.get::<_, i64>(0)
        })
        .unwrap()
        .to_string()
    }

    fn sqlite_spec(db: &Path, tables: &str) -> CleanerSpec {
        let src = format!(
            r#"
[[cleaner]]
id = "t-sql"
name = "Cookies"
description = "d"
[[cleaner.action]]
type = "sqlite_rows"
paths = ["{}"]
tables = {tables}
"#,
            db.display().to_string().replace('\\', "/")
        );
        crate::engine::registry::parse_spec_file(&src)
            .unwrap()
            .remove(0)
    }

    #[test]
    fn sqlite_rows_previews_then_deletes_only_the_named_tables() {
        let d = tempfile::tempdir().unwrap();
        let db = d.path().join("Ladybird.db");
        ladybird_db(&db);
        let spec = sqlite_spec(&db, r#"["Cookies"]"#);
        let cfg = EngineConfig::default();
        // Windows paths in TOML: forward slashes (backslashes would be escapes).

        let preview = run_spec_with(&spec, RunOptions::preview(), &cfg).unwrap();
        assert_eq!(preview.items.len(), 1, "one database, one item");
        assert!(preview.total_bytes > 0, "the row payload is measured");
        assert_eq!(count(&db, "Cookies"), "2", "preview deletes nothing");

        run_spec_with(&spec, RunOptions::execute(), &cfg).unwrap();
        assert_eq!(count(&db, "Cookies"), "0", "cookies are gone");
        assert_eq!(count(&db, "WebStorage"), "1", "other tables are untouched");
        assert_eq!(
            count(&db, "SchemaVersions"),
            "1",
            "the schema table survives"
        );
        assert!(db.exists(), "the database file itself stays");

        // Nothing left to clear: no item, no error.
        let again = run_spec_with(&spec, RunOptions::preview(), &cfg).unwrap();
        assert!(again.items.is_empty());
    }

    #[test]
    fn sqlite_rows_skips_missing_tables_and_missing_files() {
        let d = tempfile::tempdir().unwrap();
        let db = d.path().join("Ladybird.db");
        ladybird_db(&db);
        let cfg = EngineConfig::default();
        let spec = sqlite_spec(&db, r#"["NoSuchTable"]"#);
        assert!(
            run_spec_with(&spec, RunOptions::preview(), &cfg)
                .unwrap()
                .items
                .is_empty()
        );
        let gone = sqlite_spec(&d.path().join("absent.db"), r#"["Cookies"]"#);
        assert!(
            run_spec_with(&gone, RunOptions::execute(), &cfg)
                .unwrap()
                .items
                .is_empty()
        );
        // A non-database file is left alone and does not abort the run.
        let junk = d.path().join("junk.db");
        fs::write(&junk, b"not sqlite").unwrap();
        let spec = sqlite_spec(&junk, r#"["Cookies"]"#);
        run_spec_with(&spec, RunOptions::execute(), &cfg).unwrap();
        assert_eq!(fs::read(&junk).unwrap(), b"not sqlite");
    }

    #[test]
    fn sql_identifiers_reject_anything_that_could_inject() {
        for ok in ["Cookies", "web_storage", "T1"] {
            assert!(is_sql_identifier(ok), "{ok}");
        }
        for bad in [
            "",
            "1abc",
            "a b",
            "a;b",
            "a\"b",
            "x'--",
            "a-b",
            &"x".repeat(65),
        ] {
            assert!(!is_sql_identifier(bad), "{bad}");
        }
    }

    /// A command that removes only part of a folder: preview must report just that part.
    #[cfg(unix)]
    fn partial_command_spec(dir: &Path, preview: &str, run: &str) -> CleanerSpec {
        let src = format!(
            r#"
[[cleaner]]
id = "t-cmd"
name = "Partial"
description = "d"
[[cleaner.action]]
type = "command"
program = "sh"
args = ["-c", "{run}"]
measure = ["{dir}"]
preview_args = ["-c", "{preview}"]
"#,
            dir = dir.display(),
        );
        crate::engine::registry::parse_spec_file(&src)
            .unwrap()
            .remove(0)
    }

    #[cfg(unix)]
    #[test]
    fn command_preview_reports_only_what_the_dry_run_lists_and_a_real_run_what_it_freed() {
        let d = tempfile::tempdir().unwrap();
        let dir = d.path();
        write(&dir.join("gone.pkg"), 1000);
        write(&dir.join("kept.pkg"), 9000);
        let spec = partial_command_spec(
            dir,
            &format!("printf '%s\\n' '{}/gone.pkg'", dir.display()),
            &format!("rm {}/gone.pkg", dir.display()),
        );
        let cfg = EngineConfig::default();

        let preview = run_spec_with(&spec, RunOptions::preview(), &cfg).unwrap();
        assert_eq!(
            preview.total_bytes, 1000,
            "only the listed file, not the 10000 bytes of the folder: {:?}",
            preview.items
        );
        assert_eq!(preview.items.len(), 1);
        assert!(dir.join("gone.pkg").exists(), "preview removes nothing");

        let real = run_spec_with(&spec, RunOptions::execute(), &cfg).unwrap();
        assert_eq!(real.total_bytes, 1000, "freed = what really disappeared");
        assert!(!dir.join("gone.pkg").exists() && dir.join("kept.pkg").exists());

        // Nothing left to remove: the same dry run now lists nothing, and previews 0.
        let again = run_spec_with(&spec, RunOptions::preview(), &cfg).unwrap();
        assert_eq!(again.total_bytes, 0);
    }

    #[cfg(unix)]
    #[test]
    fn a_failing_or_silent_dry_run_reports_nothing_instead_of_the_whole_folder() {
        let d = tempfile::tempdir().unwrap();
        write(&d.path().join("a.pkg"), 5000);
        let cfg = EngineConfig::default();
        let failing = partial_command_spec(d.path(), "exit 3", "true");
        assert_eq!(
            run_spec_with(&failing, RunOptions::preview(), &cfg)
                .unwrap()
                .total_bytes,
            0
        );
        let silent = partial_command_spec(d.path(), "true", "true");
        assert_eq!(
            run_spec_with(&silent, RunOptions::preview(), &cfg)
                .unwrap()
                .total_bytes,
            0
        );
        // Relative lines, missing files and duplicates are ignored; real files count once.
        let noisy = partial_command_spec(
            d.path(),
            &format!(
                "printf '%s\\n' 'relative.pkg' '/no/such/file' '{p}/a.pkg' '{p}/a.pkg' '==> header'",
                p = d.path().display()
            ),
            "true",
        );
        assert_eq!(
            run_spec_with(&noisy, RunOptions::preview(), &cfg)
                .unwrap()
                .total_bytes,
            5000
        );
    }

    /// `git init` + `git add` a file (the index is what "tracked" means).
    fn git(dir: &Path, args: &[&str]) -> bool {
        std::process::Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .output()
            .is_ok_and(|o| o.status.success())
    }

    #[test]
    fn pin_files_are_read_the_way_their_tools_write_them() {
        let p = |f, t| pins_in(f, t);
        assert_eq!(p(".nvmrc", "v20.11.0\n"), ["20.11.0"]);
        assert_eq!(p(".nvmrc", "lts/*\n"), Vec::<String>::new(), "floating");
        assert_eq!(p(".python-version", "3.12\n3.11.4\n"), ["3.12", "3.11.4"]);
        assert_eq!(
            p(
                ".tool-versions",
                "nodejs 20.1.0\n# c\npython 3.12.1 3.11.0\n"
            ),
            ["20.1.0", "3.12.1", "3.11.0"]
        );
        assert_eq!(p(".sdkmanrc", "java=21.0.11-zulu\n"), ["21.0.11-zulu"]);
        assert_eq!(
            p(
                "rust-toolchain.toml",
                "[toolchain]\nchannel = \"nightly-2026-09-15\"\n"
            ),
            ["nightly-2026-09-15"]
        );
        assert_eq!(
            p("rust-toolchain.toml", "[toolchain]\nchannel = \"stable\"\n"),
            Vec::<String>::new()
        );
        assert_eq!(
            p(
                "gradle-wrapper.properties",
                "distributionUrl=https\\://services.gradle.org/distributions/gradle-8.5-bin.zip\n"
            ),
            ["gradle-8.5-bin", "8.5"]
        );
        assert_eq!(
            p("go.mod", "module x\ngo 1.22\ntoolchain go1.22.1\n"),
            ["go1.22.1"]
        );
    }

    #[test]
    fn installed_folders_match_partial_and_exact_pins_at_boundaries() {
        let pins: std::collections::HashSet<String> = [
            "20",
            "3.12",
            "21.0.11-zulu",
            "nightly-2026-09-15",
            "go1.22.1",
            "gradle-8.5-bin",
        ]
        .into_iter()
        .map(String::from)
        .collect();
        for yes in [
            "v20.11.0",
            "3.12.1",
            "21.0.11-zulu",
            "nightly-2026-09-15-x86_64-unknown-linux-gnu",
            "toolchain@v0.0.1-go1.22.1.linux-amd64",
            "gradle-8.5-bin",
        ] {
            assert!(version_matches_pin(yes, &pins), "{yes}");
        }
        for no in [
            "v200.1.0",
            "3.120.0",
            "21.0.10-zulu",
            "nightly-2026-10-15-x86_64",
            "v18.0.0",
        ] {
            assert!(!version_matches_pin(no, &pins), "{no}");
        }
    }

    #[test]
    fn a_runtime_a_project_pins_is_kept_even_when_idle() {
        let d = tempfile::tempdir().unwrap();
        let home = d.path().join("home");
        let proj = d.path().join("projects/app");
        write(&proj.join("package.json"), 1);
        fs::write(proj.join(".nvmrc"), "18\n").unwrap();
        let old = SystemTime::now() - Duration::from_secs(400 * 86_400);
        let times = fs::FileTimes::new().set_modified(old).set_accessed(old);
        for v in ["v16.0.0", "v18.20.0"] {
            let dir = home.join(format!("node/{v}"));
            write(&dir.join("bin/node"), 10);
            // Deepest first: touching a file would otherwise refresh its folder again.
            for p in [dir.join("bin/node"), dir.join("bin"), dir] {
                set_times_of(&p, times);
            }
        }
        let src = format!(
            r#"
[[cleaner]]
id = "t-pin"
name = "Runtimes"
description = "d"
[[cleaner.action]]
type = "delete"
paths = ["{}/node/*"]
min_age_days = 120
age_by_contents = true
"#,
            home.display().to_string().replace('\\', "/")
        );
        let spec = crate::engine::registry::parse_spec_file(&src)
            .unwrap()
            .remove(0);
        let cfg = cfg_for(&d.path().join("projects"));
        let preview = run_spec_with(&spec, RunOptions::preview(), &cfg).unwrap();
        let hit: Vec<_> = preview.items.iter().map(|i| i.path.clone()).collect();
        assert!(hit.iter().any(|p| p.ends_with("v16.0.0")), "{hit:?}");
        assert!(
            !hit.iter().any(|p| p.ends_with("v18.20.0")),
            "pinned by .nvmrc: {hit:?}"
        );
    }

    #[test]
    fn committed_build_folders_are_never_offered() {
        let d = tempfile::tempdir().unwrap();
        let r = d.path();
        if !git(r, &["init", "-q"]) {
            return; // no git on this machine
        }
        // Two projects in one repo: one commits its vendor/, the other ignores it.
        write(&r.join("kept/composer.json"), 1);
        write(&r.join("kept/vendor/lib.php"), 100);
        write(&r.join("free/composer.json"), 1);
        write(&r.join("free/vendor/lib.php"), 100);
        assert!(git(r, &["add", "-f", "kept/vendor/lib.php"]));
        let found =
            find_project_artifacts(&["composer.json".into()], &["vendor".into()], &cfg_for(r));
        let arts: Vec<&Path> = found.iter().map(|(_, a)| a.as_path()).collect();
        assert!(arts.iter().any(|a| a.ends_with("free/vendor")), "{arts:?}");
        assert!(
            !arts.iter().any(|a| a.ends_with("kept/vendor")),
            "a vendor/ with tracked files must stay: {arts:?}"
        );
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
