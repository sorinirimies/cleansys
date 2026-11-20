//! The breadth-first, level-parallel directory walk shared by the project scan and the
//! deep (large file / duplicate) scan.

use log::warn;
use std::path::{Path, PathBuf};

/// Walk `roots` level by level, spreading each level over the rayon pool.
///
/// `visit(dir, depth)` returns what it found in `dir` and the sub-directories to enter next.
/// The walk stops after `max_depth` levels, or (with a warning naming `what`) once more than
/// `max_dirs` directories were queued.
pub(super) fn walk_levels<T, F>(
    roots: Vec<PathBuf>,
    max_depth: usize,
    max_dirs: usize,
    what: &str,
    visit: F,
) -> Vec<T>
where
    T: Send,
    F: Fn(&Path, usize) -> (Option<T>, Vec<PathBuf>) + Sync,
{
    use rayon::prelude::*;
    let mut found = Vec::new();
    let mut frontier = roots;
    let mut depth = 0usize;
    let mut visited = 0usize;
    while !frontier.is_empty() && depth <= max_depth {
        visited += frontier.len();
        if visited > max_dirs {
            warn!("{what} stopped: too many directories");
            break;
        }
        let level: Vec<(Option<T>, Vec<PathBuf>)> =
            frontier.par_iter().map(|d| visit(d, depth)).collect();
        frontier = Vec::new();
        for (hit, children) in level {
            found.extend(hit);
            frontier.extend(children);
        }
        depth += 1;
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree() -> tempfile::TempDir {
        let d = tempfile::tempdir().unwrap();
        for p in ["a/b/c", "a/x", "z"] {
            std::fs::create_dir_all(d.path().join(p)).unwrap();
        }
        d
    }

    fn children(dir: &Path) -> (Option<String>, Vec<PathBuf>) {
        let kids = std::fs::read_dir(dir)
            .unwrap()
            .flatten()
            .map(|e| e.path())
            .collect();
        (
            Some(dir.file_name().unwrap().to_string_lossy().into_owned()),
            kids,
        )
    }

    #[test]
    fn visits_every_level_and_collects_hits() {
        let d = tree();
        let mut names = walk_levels(vec![d.path().to_path_buf()], 10, 100, "t", |p, _| {
            children(p)
        });
        names.sort();
        assert_eq!(names.len(), 6);
        assert!(names.contains(&"c".to_string()));
    }

    #[test]
    fn depth_and_directory_caps_stop_the_walk() {
        let d = tree();
        let shallow = walk_levels(vec![d.path().to_path_buf()], 1, 100, "t", |p, _| {
            children(p)
        });
        assert_eq!(
            shallow.len(),
            3,
            "root plus its two children; a/b is level 2"
        );
        let capped = walk_levels(vec![d.path().to_path_buf()], 10, 2, "t", |p, _| children(p));
        assert_eq!(capped.len(), 1, "only the root fits under the cap");
    }
}
