//! Deleting one user-chosen file or folder (the disk-usage browser): guarded, and to the
//! trash by default so it can be undone.
//!
//! Trash follows the freedesktop.org spec on Linux (`files/` + `info/*.trashinfo` under
//! `$XDG_DATA_HOME/Trash`) and `~/.Trash` on macOS. A move across drives is not possible
//! (the trash lives on the home drive); that is reported, and only a permanent delete
//! works there. Windows has no trash here: permanent only.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::config::EngineConfig;
use super::safety::{is_excluded, is_protected};

/// How to get rid of the entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Move to the trash (undoable).
    Trash,
    /// Remove for good.
    Permanent,
}

/// Whether [`Mode::Trash`] works on this platform.
pub const TRASH_SUPPORTED: bool = cfg!(any(target_os = "linux", target_os = "macos"));

/// A reason to think twice before deleting `path` (shown in the confirmation), if any.
pub fn warning_for(path: &Path) -> Option<&'static str> {
    let s = path.to_string_lossy();
    if s.contains("/steamapps") || s.contains("/Steam/") || s.ends_with("/Steam") {
        return Some(
            "Steam game data: uninstall games from Steam instead, or it will think they are installed.",
        );
    }
    if let Some(home) = crate::cleaners::platform::home_dir()
        && (path.starts_with(home.join(".config")) || path.starts_with(home.join(".local/share")))
    {
        return Some("Application data: the app may lose its settings or saved data.");
    }
    None
}

/// Delete `path`. Refuses protected and excluded paths and anything that is not an existing
/// file or folder. Returns where it went (the trash path) for [`Mode::Trash`].
pub fn delete_entry(
    path: &Path,
    mode: Mode,
    cfg: &EngineConfig,
) -> Result<Option<PathBuf>, String> {
    if is_protected(path) {
        return Err("this location is protected and is never deleted".into());
    }
    if is_excluded(path, &cfg.exclude) {
        return Err("this path is in your \"never delete\" list".into());
    }
    let meta = fs::symlink_metadata(path).map_err(|e| format!("cannot read it: {e}"))?;
    match mode {
        Mode::Permanent => {
            let r = if meta.is_dir() {
                crate::utils::remove_dir_all_parallel(path)
            } else {
                fs::remove_file(path)
            };
            r.map(|()| None).map_err(|e| e.to_string())
        }
        Mode::Trash => move_to_trash(path).map(Some),
    }
}

#[cfg(target_os = "linux")]
fn trash_dir() -> Option<PathBuf> {
    std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| crate::cleaners::platform::home_dir().map(|h| h.join(".local/share")))
        .map(|d| d.join("Trash"))
}

#[cfg(target_os = "macos")]
fn trash_dir() -> Option<PathBuf> {
    crate::cleaners::platform::home_dir().map(|h| h.join(".Trash"))
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn trash_dir() -> Option<PathBuf> {
    None
}

fn move_to_trash(path: &Path) -> Result<PathBuf, String> {
    let base = trash_dir().ok_or("no trash on this system — use permanent delete")?;
    move_into(path, &base, cfg!(target_os = "linux"))
}

/// Move `path` into the trash at `base` (testable: `base` is explicit).
fn move_into(path: &Path, base: &Path, write_info: bool) -> Result<PathBuf, String> {
    let files = base.join("files");
    let info = base.join("info");
    fs::create_dir_all(&files).map_err(|e| e.to_string())?;
    if write_info {
        fs::create_dir_all(&info).map_err(|e| e.to_string())?;
    }
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .ok_or("no file name")?;
    // A name nobody else uses in the trash (the info file is created exclusively).
    let mut n = 0u32;
    let (dest, info_path) = loop {
        let candidate = if n == 0 {
            name.clone()
        } else {
            format!("{name}.{n}")
        };
        let dest = files.join(&candidate);
        let ip = info.join(format!("{candidate}.trashinfo"));
        if dest.symlink_metadata().is_err() && (!write_info || ip.symlink_metadata().is_err()) {
            break (dest, ip);
        }
        n += 1;
        if n > 10_000 {
            return Err("trash is full of files with this name".into());
        }
    };
    if write_info {
        let body = format!(
            "[Trash Info]\nPath={}\nDeletionDate={}\n",
            percent_encode(&path.to_string_lossy()),
            now_stamp()
        );
        fs::write(&info_path, body).map_err(|e| e.to_string())?;
    }
    match fs::rename(path, &dest) {
        Ok(()) => Ok(dest),
        Err(e) => {
            if write_info {
                let _ = fs::remove_file(&info_path);
            }
            Err(if is_cross_device(&e) {
                "it is on another drive than your trash — use permanent delete".to_string()
            } else {
                e.to_string()
            })
        }
    }
}

fn is_cross_device(e: &io::Error) -> bool {
    // EXDEV is 18 on Linux and macOS.
    matches!(e.raw_os_error(), Some(18)) || e.kind() == io::ErrorKind::CrossesDevices
}

/// Percent-encode a path for `.trashinfo` (keeps `/` and unreserved characters).
fn percent_encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'/' | b'-' | b'_' | b'.' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// `YYYY-MM-DDTHH:MM:SS` (UTC) of now.
fn now_stamp() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let (days, rem) = (secs / 86_400, secs % 86_400);
    // Civil-from-days (Howard Hinnant).
    let z = days as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protected_and_excluded_paths_are_refused() {
        let cfg = EngineConfig::default();
        assert!(delete_entry(Path::new("/usr"), Mode::Permanent, &cfg).is_err());
        assert!(delete_entry(Path::new("relative"), Mode::Trash, &cfg).is_err());
        let d = tempfile::tempdir().unwrap();
        let f = d.path().join("keep.txt");
        fs::write(&f, "x").unwrap();
        let cfg = EngineConfig {
            exclude: vec![f.to_string_lossy().into_owned()],
            ..EngineConfig::default()
        };
        assert!(delete_entry(&f, Mode::Permanent, &cfg).is_err());
        assert!(f.exists());
    }

    #[test]
    fn permanent_removes_files_and_folders() {
        let d = tempfile::tempdir().unwrap();
        let sub = d.path().join("dir/inner");
        fs::create_dir_all(&sub).unwrap();
        fs::write(sub.join("f"), "x").unwrap();
        let file = d.path().join("one");
        fs::write(&file, "x").unwrap();
        let cfg = EngineConfig::default();
        delete_entry(&d.path().join("dir"), Mode::Permanent, &cfg).unwrap();
        delete_entry(&file, Mode::Permanent, &cfg).unwrap();
        assert!(!d.path().join("dir").exists() && !file.exists());
    }

    #[test]
    fn trash_moves_keeps_info_and_never_overwrites() {
        let d = tempfile::tempdir().unwrap();
        let trash = d.path().join("Trash");
        for _ in 0..2 {
            let f = d.path().join("my file.txt");
            fs::write(&f, "x").unwrap();
            move_into(&f, &trash, true).unwrap();
            assert!(!f.exists());
        }
        assert!(trash.join("files/my file.txt").exists());
        assert!(trash.join("files/my file.txt.1").exists());
        let info = fs::read_to_string(trash.join("info/my file.txt.trashinfo")).unwrap();
        assert!(info.starts_with("[Trash Info]\nPath="));
        assert!(info.contains("my%20file.txt"));
        assert!(info.contains("DeletionDate=20"));
    }

    #[test]
    fn stamp_is_well_formed() {
        let s = now_stamp();
        assert_eq!(s.len(), 19);
        assert_eq!(&s[10..11], "T");
    }

    #[test]
    fn warns_for_steam_and_app_data() {
        assert!(warning_for(Path::new("/x/steamapps/common/g")).is_some());
        assert!(warning_for(Path::new("/x/plain")).is_none());
    }
}
