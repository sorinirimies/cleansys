//! Paths the user unticked in the details view; removal helpers refuse them.
//!
//! The front-ends set this just before a preview/run and clear it afterwards.
//! (A process-wide set keeps [`crate::RunOptions`] `Copy`, which every cleaner
//! function relies on.) Scans always run with the set empty.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

static SKIPPED: Mutex<Option<HashSet<PathBuf>>> = Mutex::new(None);

fn lock() -> std::sync::MutexGuard<'static, Option<HashSet<PathBuf>>> {
    SKIPPED.lock().unwrap_or_else(|e| e.into_inner())
}

/// Replace the skip set.
pub fn set_skipped(paths: impl IntoIterator<Item = PathBuf>) {
    let set: HashSet<PathBuf> = paths.into_iter().collect();
    *lock() = (!set.is_empty()).then_some(set);
}

/// Forget every skipped path (frees the memory).
pub fn clear_skipped() {
    *lock() = None;
}

/// Is `path`, or any directory above it, in the skip set?
pub fn is_skipped(path: &Path) -> bool {
    let guard = lock();
    let Some(set) = guard.as_ref() else {
        return false;
    };
    path.ancestors().any(|a| set.contains(a))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skips_exact_paths_and_their_children() {
        // Single test: the set is process-wide.
        clear_skipped();
        assert!(!is_skipped(Path::new("/a/b")));
        set_skipped([PathBuf::from("/a/b")]);
        assert!(is_skipped(Path::new("/a/b")));
        assert!(is_skipped(Path::new("/a/b/c/d")));
        assert!(!is_skipped(Path::new("/a/bc")));
        assert!(!is_skipped(Path::new("/a")));
        clear_skipped();
        assert!(!is_skipped(Path::new("/a/b")));
    }
}
