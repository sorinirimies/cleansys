//! Per-platform cache/temp/trash path resolution.
//!
//! Centralises every `#[cfg(target_os = "...")]` branch used by the cleaner
//! implementations so `user_cleaners.rs`/`system_cleaners.rs` stay readable:
//! the *logic* (measure size, confirm, remove, record) is shared, only the
//! *paths* differ per operating system.

use directories::BaseDirs;
use std::path::PathBuf;

/// Resolve the user's home directory, or `None` if it can't be determined.
pub fn home_dir() -> Option<PathBuf> {
    BaseDirs::new().map(|b| b.home_dir().to_path_buf())
}

/// `(label, path)` pairs for every browser cache we know how to clean on the
/// current platform.
pub fn browser_cache_paths() -> Vec<(String, PathBuf)> {
    let Some(home) = home_dir() else {
        return Vec::new();
    };

    #[cfg(target_os = "linux")]
    {
        let mut paths = vec![
            (
                "Chrome cache".to_string(),
                home.join(".config/google-chrome/Default/Cache"),
            ),
            (
                "Chromium cache".to_string(),
                home.join(".config/chromium/Default/Cache"),
            ),
            (
                "Chrome cache".to_string(),
                home.join(".cache/google-chrome"),
            ),
            ("Chromium cache".to_string(), home.join(".cache/chromium")),
        ];
        let firefox_root = home.join(".mozilla/firefox");
        if let Ok(entries) = std::fs::read_dir(&firefox_root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir()
                    && path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .ends_with(".default")
                {
                    paths.push(("Firefox cache".to_string(), path.join("cache2")));
                }
            }
        }
        paths
    }

    #[cfg(target_os = "macos")]
    {
        let mut paths = vec![
            (
                "Chrome cache".to_string(),
                home.join("Library/Caches/Google/Chrome/Default/Cache"),
            ),
            (
                "Safari cache".to_string(),
                home.join("Library/Caches/com.apple.Safari"),
            ),
        ];
        let firefox_root = home.join("Library/Caches/Firefox/Profiles");
        if let Ok(entries) = std::fs::read_dir(&firefox_root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    paths.push(("Firefox cache".to_string(), path.join("cache2")));
                }
            }
        }
        paths
    }

    #[cfg(target_os = "windows")]
    {
        let local_app_data = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
        let mut paths = Vec::new();
        if let Some(lad) = &local_app_data {
            paths.push((
                "Chrome cache".to_string(),
                lad.join("Google/Chrome/User Data/Default/Cache"),
            ));
            paths.push((
                "Edge cache".to_string(),
                lad.join("Microsoft/Edge/User Data/Default/Cache"),
            ));
            let firefox_root = lad.join("Mozilla/Firefox/Profiles");
            if let Ok(entries) = std::fs::read_dir(&firefox_root) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        paths.push(("Firefox cache".to_string(), path.join("cache2")));
                    }
                }
            }
        }
        paths
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        Vec::new()
    }
}

/// The general-purpose application cache directory for the current platform
/// (e.g. `~/.cache` on Linux, `~/Library/Caches` on macOS).
///
/// Windows has no single equivalent convention, so this returns `None` there
/// — see [`windows_cache_paths`] instead.
pub fn app_cache_dir() -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        BaseDirs::new().map(|b| b.cache_dir().to_path_buf())
    }
    #[cfg(target_os = "macos")]
    {
        home_dir().map(|h| h.join("Library/Caches"))
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        None
    }
}

/// Subdirectory names inside [`app_cache_dir`] that must never be deleted
/// (critical to desktop session / rendering functioning correctly).
pub fn protected_cache_names() -> &'static [&'static str] {
    #[cfg(target_os = "linux")]
    {
        &["dconf", "fontconfig", "mesa_shader_cache"]
    }
    #[cfg(target_os = "macos")]
    {
        &["CloudKit", "com.apple.bird"]
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        &[]
    }
}

/// Windows-only cache locations with no cross-platform equivalent
/// (`%LOCALAPPDATA%\Microsoft\Windows\INetCache`, etc.).
pub fn windows_cache_paths() -> Vec<(String, PathBuf)> {
    #[cfg(target_os = "windows")]
    {
        let Some(lad) = std::env::var_os("LOCALAPPDATA").map(PathBuf::from) else {
            return Vec::new();
        };
        vec![
            (
                "Windows Internet cache".to_string(),
                lad.join("Microsoft/Windows/INetCache"),
            ),
            ("Windows crash dumps".to_string(), lad.join("CrashDumps")),
        ]
    }
    #[cfg(not(target_os = "windows"))]
    {
        Vec::new()
    }
}

/// Thumbnail cache location(s) for the current platform.
pub fn thumbnail_cache_paths() -> Vec<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        let Some(home) = home_dir() else {
            return Vec::new();
        };
        vec![home.join(".thumbnails"), home.join(".cache/thumbnails")]
    }
    #[cfg(target_os = "macos")]
    {
        let Some(home) = home_dir() else {
            return Vec::new();
        };
        vec![home.join("Library/Caches/com.apple.quicklook.thumbnailcache")]
    }
    #[cfg(target_os = "windows")]
    {
        let Some(lad) = std::env::var_os("LOCALAPPDATA").map(PathBuf::from) else {
            return Vec::new();
        };
        vec![lad.join("Microsoft/Windows/Explorer")]
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        Vec::new()
    }
}

/// The current user's temp directory (contents owned by the user, safe to
/// clean without extra ownership checks on macOS/Windows since the whole
/// directory is per-user; Linux's shared `/tmp` still filters by UID).
pub fn temp_dir() -> PathBuf {
    #[cfg(target_os = "linux")]
    {
        PathBuf::from("/tmp")
    }
    #[cfg(not(target_os = "linux"))]
    {
        std::env::temp_dir()
    }
}

/// `(label, path)` pairs for user-level package-manager caches.
/// Cargo's registry cache is identical across platforms (keyed off `$CARGO_HOME`/home).
pub fn package_cache_paths() -> Vec<(String, PathBuf)> {
    let Some(home) = home_dir() else {
        return Vec::new();
    };

    let mut paths = vec![(
        "cargo registry cache".to_string(),
        home.join(".cargo/registry/cache"),
    )];

    #[cfg(target_os = "linux")]
    {
        paths.push(("pip cache".to_string(), home.join(".cache/pip")));
        paths.push(("npm cache".to_string(), home.join(".npm/_cacache")));
    }
    #[cfg(target_os = "macos")]
    {
        paths.push(("pip cache".to_string(), home.join("Library/Caches/pip")));
        paths.push(("npm cache".to_string(), home.join(".npm/_cacache")));
    }
    #[cfg(target_os = "windows")]
    {
        if let Some(lad) = std::env::var_os("LOCALAPPDATA").map(PathBuf::from) {
            paths.push(("pip cache".to_string(), lad.join("pip/Cache")));
        }
        if let Some(appdata) = std::env::var_os("APPDATA").map(PathBuf::from) {
            paths.push(("npm cache".to_string(), appdata.join("npm-cache")));
        }
    }

    paths
}

/// Trash/Recycle-Bin directories for the current platform.
///
/// Returns an empty list on Windows: the Recycle Bin is not a plain,
/// safely-walkable folder (it lives under a per-drive, per-SID hidden
/// system directory that requires the shell APIs to enumerate/empty
/// correctly), so we don't pretend to support it there yet.
pub fn trash_paths() -> Vec<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        home_dir()
            .map(|h| vec![h.join(".local/share/Trash")])
            .unwrap_or_default()
    }
    #[cfg(target_os = "macos")]
    {
        home_dir()
            .map(|h| vec![h.join(".Trash")])
            .unwrap_or_default()
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        Vec::new()
    }
}

/// Human-readable name of the current platform, for UI display.
pub fn platform_name() -> &'static str {
    #[cfg(target_os = "linux")]
    {
        "Linux"
    }
    #[cfg(target_os = "macos")]
    {
        "macOS"
    }
    #[cfg(target_os = "windows")]
    {
        "Windows"
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        "this platform"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn home_dir_resolves_on_supported_platforms() {
        assert!(home_dir().is_some());
    }

    #[test]
    fn temp_dir_is_non_empty() {
        assert!(!temp_dir().as_os_str().is_empty());
    }

    #[test]
    fn package_cache_paths_always_includes_cargo() {
        let paths = package_cache_paths();
        assert!(paths.iter().any(|(label, _)| label.contains("cargo")));
    }

    #[test]
    fn platform_name_is_non_empty() {
        assert!(!platform_name().is_empty());
    }

    #[test]
    fn browser_cache_paths_does_not_panic() {
        let _ = browser_cache_paths();
    }

    #[test]
    fn thumbnail_cache_paths_does_not_panic() {
        let _ = thumbnail_cache_paths();
    }

    #[test]
    fn trash_paths_does_not_panic() {
        let _ = trash_paths();
    }
}
