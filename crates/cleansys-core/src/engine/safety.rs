//! Guard rails: paths that must never be deleted, plus user exclusions.

use std::path::{Path, PathBuf};

/// Absolute system directories that are never removable (the directory
/// itself; cache *subdirectories* below them can still be targeted).
const PROTECTED: &[&str] = &[
    "/",
    "/bin",
    "/boot",
    "/dev",
    "/etc",
    "/home",
    "/lib",
    "/lib64",
    "/opt",
    "/proc",
    "/root",
    "/run",
    "/sbin",
    "/srv",
    "/sys",
    "/tmp",
    "/usr",
    "/var",
    "/Users",
    "/System",
    "/Library",
    "/Applications",
    "/private",
    "/Volumes",
    "/usr/local",
    "/var/lib",
    "/var/log",
    "/var/cache",
];

/// `true` when removing `path` could damage the system or the user's home
/// layout: filesystem roots, system dirs, the home dir or any ancestor of it,
/// relative paths, and paths with `..` components.
pub fn is_protected(path: &Path) -> bool {
    if !path.is_absolute() {
        return true;
    }
    if path
        .components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return true;
    }
    // Filesystem / drive roots ("C:\" has one normal component fewer).
    if path.parent().is_none() {
        return true;
    }
    let s = path.to_string_lossy();
    let trimmed = s.trim_end_matches('/');
    if PROTECTED
        .iter()
        .any(|p| *p == trimmed || (trimmed.is_empty() && *p == "/"))
    {
        return true;
    }
    if let Some(home) = crate::cleaners::platform::home_dir() {
        // home itself or any ancestor of home
        if home == path || home.starts_with(path) {
            return true;
        }
        // Top-level user dirs that hold personal data.
        for d in [
            "Documents",
            "Desktop",
            "Pictures",
            "Music",
            "Videos",
            "Movies",
            "Downloads",
            "Library",
            "Applications",
            ".ssh",
            ".gnupg",
            ".config",
            ".local",
            ".cache",
        ] {
            if path == home.join(d) {
                return true;
            }
        }
        if path == home.join(".local/share") {
            return true;
        }
    }
    false
}

/// `true` when `path` matches one of the user's exclusion glob patterns
/// (matched against the full path and every ancestor, so excluding a
/// directory also protects everything inside it).
pub fn is_excluded(path: &Path, patterns: &[String]) -> bool {
    if patterns.is_empty() {
        return false;
    }
    let opts = glob::MatchOptions {
        case_sensitive: !cfg!(any(target_os = "windows", target_os = "macos")),
        require_literal_separator: false,
        require_literal_leading_dot: false,
    };
    let compiled: Vec<glob::Pattern> = patterns
        .iter()
        .filter_map(|p| {
            glob::Pattern::new(&super::paths::expand(p, &super::paths::lookup_env)).ok()
        })
        .collect();
    let mut cur: Option<PathBuf> = Some(path.to_path_buf());
    while let Some(p) = cur {
        if compiled.iter().any(|pat| pat.matches_path_with(&p, opts)) {
            return true;
        }
        cur = p.parent().map(Path::to_path_buf);
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roots_and_relative_protected() {
        assert!(is_protected(Path::new("/")));
        assert!(is_protected(Path::new("/usr")));
        assert!(is_protected(Path::new("relative/dir")));
        assert!(is_protected(Path::new("/tmp/../etc")));
    }

    #[test]
    fn home_protected_but_cache_subdir_is_not() {
        if let Some(home) = crate::cleaners::platform::home_dir() {
            assert!(is_protected(&home));
            assert!(is_protected(home.parent().unwrap()));
            assert!(is_protected(&home.join("Documents")));
            assert!(!is_protected(&home.join(".cache/huggingface")));
            assert!(!is_protected(&home.join("proj/target")));
        }
    }

    #[test]
    fn exclusions_cover_descendants() {
        let pats = vec!["/work/keep".to_string(), "**/important/**".to_string()];
        assert!(is_excluded(Path::new("/work/keep/target"), &pats));
        assert!(is_excluded(Path::new("/a/important/b/target"), &pats));
        assert!(!is_excluded(Path::new("/work/other/target"), &pats));
    }
}
