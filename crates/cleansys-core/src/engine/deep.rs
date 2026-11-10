//! Deep clean: find big and long-untouched files in the configured deep roots.
//!
//! Strictly bounded: only the given roots are searched, symlinks are never followed, a
//! drive mounted below a root is skipped, hidden folders and build/dependency trees are
//! not entered (other cleaners own those), and the walk stops after [`MAX_VISITED_DIRS`]
//! directories. Only the biggest [`MAX_MATCHES`] hits are kept.

use log::warn;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use super::config::EngineConfig;
use super::exec::{SKIP_DESCENT, device_of_path, same_device};
use super::safety::{is_excluded, is_protected};

/// Hard cap on directories visited per deep search.
const MAX_VISITED_DIRS: usize = 500_000;
/// Most matches kept (the biggest ones).
const MAX_MATCHES: usize = 5_000;
/// Folders deeper than this below a root are not searched.
const MAX_DEPTH: usize = 24;

/// Trees owned by an installed application: a file in there is part of that app, not
/// something forgotten (Wine prefixes, Steam libraries, the Android SDK, ...).
const APP_TREES: &[&str] = &[
    "drive_c",
    "steamapps",
    "Sdk",
    "Program Files",
    "Program Files (x86)",
    "WindowsApps",
    "Windows",
];

type Hits = Vec<(PathBuf, u64)>;

/// What to look for.
#[derive(Debug, Clone)]
pub struct Query {
    pub roots: Vec<PathBuf>,
    pub min_bytes: u64,
    /// Not modified for at least this long (zero = any age).
    pub min_age: Duration,
    /// Lower-case extensions without the dot; empty = any.
    pub extensions: Vec<String>,
}

impl Query {
    fn accepts(&self, name: &str, size: u64, modified: Option<SystemTime>) -> bool {
        if size == 0 || size < self.min_bytes {
            return false;
        }
        if !self.extensions.is_empty() {
            let lower = name.to_ascii_lowercase();
            if !self
                .extensions
                .iter()
                .any(|e| lower.ends_with(&format!(".{e}")))
            {
                return false;
            }
        }
        if !self.min_age.is_zero() {
            let age = modified
                .and_then(|m| SystemTime::now().duration_since(m).ok())
                .unwrap_or_default();
            if age < self.min_age {
                return false;
            }
        }
        true
    }
}

/// `(path, bytes)` of every matching file, biggest first.
pub fn find_files(q: &Query, cfg: &EngineConfig) -> Vec<(PathBuf, u64)> {
    use rayon::prelude::*;
    let mut hits: Vec<(PathBuf, u64)> = Vec::new();
    let mut frontier: Vec<PathBuf> = q.roots.clone();
    let mut depth = 0usize;
    let mut visited = 0usize;
    while !frontier.is_empty() && depth <= MAX_DEPTH {
        visited += frontier.len();
        if visited > MAX_VISITED_DIRS {
            warn!("deep scan stopped: too many directories");
            break;
        }
        let level: Vec<(Hits, Vec<PathBuf>)> =
            frontier.par_iter().map(|d| scan_dir(d, q)).collect();
        frontier = Vec::new();
        for (found, children) in level {
            hits.extend(found);
            frontier.extend(children);
        }
        depth += 1;
    }
    hits.retain(|(p, _)| !is_protected(p) && !is_excluded(p, &cfg.exclude));
    hits.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    hits.truncate(MAX_MATCHES);
    hits
}

fn scan_dir(dir: &Path, q: &Query) -> (Hits, Vec<PathBuf>) {
    let Ok(rd) = fs::read_dir(dir) else {
        return (Vec::new(), Vec::new());
    };
    let here = device_of_path(dir);
    let mut found = Vec::new();
    let mut children = Vec::new();
    for e in rd.flatten() {
        let Ok(ft) = e.file_type() else { continue };
        if ft.is_symlink() {
            continue;
        }
        let name = e.file_name();
        let Some(name) = name.to_str() else { continue };
        if ft.is_file() {
            if let Ok(m) = e.metadata()
                && q.accepts(name, m.len(), m.modified().ok())
            {
                found.push((e.path(), m.len()));
            }
        } else if ft.is_dir()
            && !name.starts_with('.')
            && !SKIP_DESCENT.contains(&name)
            && !APP_TREES.contains(&name)
            && !name.ends_with(".app")
            && !(name == "mod" && dir.file_name().is_some_and(|d| d == "pkg")) // Go module cache
            && same_device(here, &e)
        {
            children.push(e.path());
        }
    }
    (found, children)
}

/// Redundant copies of identical files: `(path, bytes)` of every file that has a
/// byte-for-byte twin elsewhere in the roots, biggest first. One copy per group — the
/// oldest, then the shortest path — is the keeper and is never listed. Hard links to the
/// same data are not duplicates (removing one frees nothing).
pub fn find_duplicates(q: &Query, cfg: &EngineConfig) -> Vec<(PathBuf, u64)> {
    use rayon::prelude::*;
    use std::collections::HashMap;

    // Candidates by size: a file with a unique size cannot have a twin.
    let mut by_size: HashMap<u64, Vec<PathBuf>> = HashMap::new();
    for (p, size) in find_files(q, cfg) {
        by_size.entry(size).or_default().push(p);
    }
    let groups: Vec<(u64, Vec<PathBuf>)> = by_size
        .into_iter()
        .map(|(size, v)| (size, unique_inodes(v)))
        .filter(|(_, v)| v.len() > 1)
        .collect();

    let mut out: Vec<(PathBuf, u64)> = groups
        .par_iter()
        .flat_map_iter(|(size, paths)| {
            // Cheap first: hash the first 64 KiB, then the whole file of the survivors.
            let mut by_head: HashMap<u64, Vec<&PathBuf>> = HashMap::new();
            for p in paths {
                if let Some(h) = hash_file(p, Some(64 * 1024)) {
                    by_head.entry(h).or_default().push(p);
                }
            }
            let mut dups = Vec::new();
            for same_head in by_head.into_values().filter(|v| v.len() > 1) {
                let mut by_full: HashMap<u64, Vec<&PathBuf>> = HashMap::new();
                for p in same_head {
                    if let Some(h) = hash_file(p, None) {
                        by_full.entry(h).or_default().push(p);
                    }
                }
                for mut twins in by_full.into_values().filter(|v| v.len() > 1) {
                    twins.sort_by_key(|p| {
                        (
                            fs::metadata(p).and_then(|m| m.modified()).ok(),
                            p.as_os_str().len(),
                            (*p).clone(),
                        )
                    });
                    let keeper = twins[0];
                    // The hash is not proof: only offer files that really are identical.
                    dups.extend(
                        twins[1..]
                            .iter()
                            .filter(|p| same_bytes(keeper, p))
                            .map(|p| ((*p).clone(), *size)),
                    );
                }
            }
            dups
        })
        .collect();
    out.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    out
}

/// Drop paths that are hard links to a file already in the list.
#[cfg(unix)]
fn unique_inodes(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    use std::os::unix::fs::MetadataExt;
    let mut seen = std::collections::HashSet::new();
    paths
        .into_iter()
        .filter(|p| {
            fs::symlink_metadata(p)
                .map(|m| seen.insert((m.dev(), m.ino())))
                .unwrap_or(false)
        })
        .collect()
}

#[cfg(not(unix))]
fn unique_inodes(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    paths
}

/// Hash of the first `limit` bytes (or the whole file).
fn hash_file(p: &Path, limit: Option<usize>) -> Option<u64> {
    use std::hash::Hasher;
    use std::io::Read;
    let mut f = fs::File::open(p).ok()?;
    let mut h = std::collections::hash_map::DefaultHasher::new();
    let mut buf = vec![0u8; 64 * 1024];
    let mut left = limit.unwrap_or(usize::MAX);
    while left > 0 {
        let want = buf.len().min(left);
        let n = f.read(&mut buf[..want]).ok()?;
        if n == 0 {
            break;
        }
        h.write(&buf[..n]);
        left -= n;
    }
    Some(h.finish())
}

fn same_bytes(a: &Path, b: &Path) -> bool {
    use std::io::Read;
    let (Ok(mut fa), Ok(mut fb)) = (fs::File::open(a), fs::File::open(b)) else {
        return false;
    };
    let (mut ba, mut bb) = (vec![0u8; 64 * 1024], vec![0u8; 64 * 1024]);
    loop {
        let Ok(na) = fa.read(&mut ba) else {
            return false;
        };
        let mut nb = 0;
        while nb < na {
            match fb.read(&mut bb[nb..na]) {
                Ok(0) | Err(_) => return false,
                Ok(n) => nb += n,
            }
        }
        if na == 0 {
            return matches!(fb.read(&mut bb), Ok(0));
        }
        if ba[..na] != bb[..na] {
            return false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(p: &Path, bytes: usize) {
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, vec![b'x'; bytes]).unwrap();
    }

    fn query(root: &Path) -> Query {
        Query {
            roots: vec![root.to_path_buf()],
            min_bytes: 1000,
            min_age: Duration::ZERO,
            extensions: Vec::new(),
        }
    }

    #[test]
    fn finds_big_files_biggest_first_and_skips_small_hidden_and_build_dirs() {
        let d = tempfile::tempdir().unwrap();
        let r = d.path();
        write(&r.join("a/big.bin"), 5000);
        write(&r.join("b/bigger.bin"), 9000);
        write(&r.join("small.bin"), 10);
        write(&r.join(".hidden/huge.bin"), 90_000);
        write(&r.join("proj/node_modules/huge.bin"), 90_000);
        write(&r.join("pfx/drive_c/huge.bin"), 90_000);
        write(&r.join("Android/Sdk/huge.img"), 90_000);
        let hits = find_files(&query(r), &EngineConfig::default());
        let names: Vec<_> = hits
            .iter()
            .map(|(p, _)| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, ["bigger.bin", "big.bin"]);
    }

    #[test]
    fn extension_filter_is_case_insensitive() {
        let d = tempfile::tempdir().unwrap();
        let r = d.path();
        write(&r.join("a.ISO"), 2000);
        write(&r.join("b.txt"), 2000);
        let mut q = query(r);
        q.extensions = vec!["iso".into()];
        let hits = find_files(&q, &EngineConfig::default());
        assert_eq!(hits.len(), 1);
        assert!(hits[0].0.ends_with("a.ISO"));
    }

    #[test]
    fn age_filter_skips_fresh_files() {
        let d = tempfile::tempdir().unwrap();
        write(&d.path().join("new.bin"), 2000);
        let mut q = query(d.path());
        q.min_age = Duration::from_secs(30 * 86_400);
        assert!(find_files(&q, &EngineConfig::default()).is_empty());
    }

    #[test]
    fn user_exclusions_are_honoured() {
        let d = tempfile::tempdir().unwrap();
        write(&d.path().join("keep/x.bin"), 2000);
        write(&d.path().join("drop/y.bin"), 2000);
        let cfg = EngineConfig {
            exclude: vec![format!("{}/keep", d.path().display())],
            ..EngineConfig::default()
        };
        let hits = find_files(&query(d.path()), &cfg);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].0.ends_with("drop/y.bin"));
    }

    #[test]
    fn duplicates_keep_the_oldest_and_ignore_near_twins_and_hard_links() {
        let d = tempfile::tempdir().unwrap();
        let r = d.path();
        write(&r.join("a/orig.bin"), 3000);
        std::thread::sleep(Duration::from_millis(20));
        write(&r.join("b/copy.bin"), 3000);
        write(&r.join("c/copy2.bin"), 3000);
        // same size, different content
        fs::create_dir_all(r.join("d")).unwrap();
        let mut other = vec![b'x'; 3000];
        other[2999] = b'y';
        fs::write(r.join("d/other.bin"), other).unwrap();
        #[cfg(unix)]
        fs::hard_link(r.join("a/orig.bin"), r.join("a/link.bin")).unwrap();
        let hits = find_duplicates(&query(r), &EngineConfig::default());
        let mut names: Vec<_> = hits
            .iter()
            .map(|(p, _)| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        names.sort();
        assert_eq!(names, ["copy.bin", "copy2.bin"]);
    }
}
