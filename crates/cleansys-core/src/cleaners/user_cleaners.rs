use anyhow::Result;
use log::{debug, warn};
use std::fs::{self, remove_dir_all, remove_file};
#[cfg(target_os = "linux")]
use std::os::unix::fs::MetadataExt;
use std::path::Path;

use crate::cleaners::cleaned_item::{CleanedItem, CleanerFn, CleaningResult};
use crate::cleaners::platform;
use crate::utils::{confirm, format_size, get_size, print_success};

pub struct CleanerInfo {
    pub name: &'static str,
    pub description: &'static str,
    pub function: CleanerFn,
}

pub fn list_cleaners() -> Vec<String> {
    get_cleaners()
        .iter()
        .map(|c| format!("{}: {}", c.name, c.description))
        .collect()
}

/// Every user-level cleaner is listed on every platform, but each one only
/// resolves the paths that actually exist/apply on the current OS (see
/// `cleaners::platform`) — so e.g. "Trash" silently does nothing on Windows
/// rather than failing, and "Browser Caches" checks OS-appropriate paths.
pub fn get_cleaners() -> Vec<CleanerInfo> {
    vec![
        CleanerInfo {
            name: "Browser Caches",
            description: "Clean Firefox, Chrome/Chromium, Edge, and Safari caches",
            function: clean_browser_caches,
        },
        CleanerInfo {
            name: "Application Caches",
            description: "Clean general-purpose application caches",
            function: clean_app_caches,
        },
        CleanerInfo {
            name: "Thumbnail Caches",
            description: "Clean thumbnail/preview image caches",
            function: clean_thumbnail_caches,
        },
        CleanerInfo {
            name: "Temporary Files",
            description: "Clean temporary files owned by the current user",
            function: clean_temp_files,
        },
        CleanerInfo {
            name: "Package Manager Caches",
            description: "Clean user package manager caches (pip, npm, cargo)",
            function: clean_package_caches,
        },
        CleanerInfo {
            name: "Trash",
            description: "Empty the trash / recycle bin",
            function: clean_trash,
        },
    ]
}

pub fn run_all(skip_confirmation: bool) -> Result<()> {
    let cleaners = get_cleaners();
    let mut total_saved: u64 = 0;

    for cleaner in cleaners {
        if skip_confirmation || confirm(&format!("Run '{}'?", cleaner.name), true)? {
            match (cleaner.function)(skip_confirmation) {
                Ok(result) => {
                    total_saved += result.total_bytes;
                    print_success(&format!(
                        "{} completed: freed {} across {} item(s)",
                        cleaner.name,
                        format_size(result.total_bytes),
                        result.item_count()
                    ));
                }
                Err(err) => {
                    crate::utils::print_error(&format!("Error in {}: {}", cleaner.name, err));
                }
            }
        }
    }

    print_success(&format!("Total space freed: {}", format_size(total_saved)));
    Ok(())
}

/// Remove a path (file or directory) and record it as a [`CleanedItem`] in
/// `result` if the size measured beforehand was non-zero. Recreates the
/// directory afterwards when `recreate` is `true` (some caches, like
/// thumbnails, are expected to exist as an empty directory afterwards).
fn clean_path(
    result: &mut CleaningResult,
    path: &Path,
    label: &str,
    skip_confirmation: bool,
    recreate: bool,
) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    let size = get_size(&path.to_string_lossy())?;
    if size == 0 {
        return Ok(());
    }

    if !skip_confirmation
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

    let is_dir = path.is_dir();
    let removal = if is_dir {
        remove_dir_all(path)
    } else {
        remove_file(path)
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

    let item = if is_dir {
        CleanedItem::directory(path.to_path_buf(), size, label)
    } else {
        CleanedItem::file(path.to_path_buf(), size, label)
    };
    result.add_item(item);

    Ok(())
}

fn clean_browser_caches(skip_confirmation: bool) -> Result<CleaningResult> {
    let mut result = CleaningResult::new();
    for (label, path) in platform::browser_cache_paths() {
        clean_path(&mut result, &path, &label, skip_confirmation, false)?;
    }
    Ok(result)
}

fn clean_app_caches(skip_confirmation: bool) -> Result<CleaningResult> {
    let mut result = CleaningResult::new();

    let Some(cache_dir) = platform::app_cache_dir() else {
        debug!(
            "No general-purpose application cache directory on {}",
            platform::platform_name()
        );
        return clean_windows_only_caches(skip_confirmation);
    };

    debug!("Cache directory: {:?}", cache_dir);

    if cache_dir.exists() {
        if let Ok(entries) = fs::read_dir(&cache_dir) {
            let protected = platform::protected_cache_names();
            for entry in entries.flatten() {
                let path = entry.path();
                let name = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                if protected.contains(&name.as_str()) {
                    debug!("Skipping critical cache directory: {:?}", path);
                    continue;
                }
                if path.is_dir() {
                    clean_path(
                        &mut result,
                        &path,
                        &format!("'{name}' cache"),
                        skip_confirmation,
                        false,
                    )?;
                }
            }
        }
    }

    // On Windows there's no single "~/.cache" convention, so we additionally
    // sweep the handful of well-known Windows-specific cache locations.
    let windows_extra = clean_windows_only_caches(skip_confirmation)?;
    result.merge(windows_extra);

    Ok(result)
}

fn clean_windows_only_caches(skip_confirmation: bool) -> Result<CleaningResult> {
    let mut result = CleaningResult::new();
    for (label, path) in platform::windows_cache_paths() {
        clean_path(&mut result, &path, &label, skip_confirmation, false)?;
    }
    Ok(result)
}

fn clean_thumbnail_caches(skip_confirmation: bool) -> Result<CleaningResult> {
    let mut result = CleaningResult::new();
    for dir in platform::thumbnail_cache_paths() {
        clean_path(
            &mut result,
            &dir,
            "thumbnail cache",
            skip_confirmation,
            true,
        )?;
    }
    Ok(result)
}

#[cfg(target_os = "linux")]
fn clean_temp_files(skip_confirmation: bool) -> Result<CleaningResult> {
    let mut result = CleaningResult::new();
    let tmp_dir = platform::temp_dir();

    if tmp_dir.exists() {
        if let Ok(entries) = fs::read_dir(&tmp_dir) {
            for entry in entries.flatten() {
                let path = entry.path();

                // On shared Linux /tmp, only touch files owned by the current user.
                if let Ok(metadata) = fs::metadata(&path) {
                    if metadata.uid() == users::get_current_uid() {
                        clean_path(
                            &mut result,
                            &path,
                            "temporary file",
                            skip_confirmation,
                            false,
                        )?;
                    }
                }
            }
        }
    }

    Ok(result)
}

#[cfg(not(target_os = "linux"))]
fn clean_temp_files(skip_confirmation: bool) -> Result<CleaningResult> {
    let mut result = CleaningResult::new();
    let tmp_dir = platform::temp_dir();

    // macOS/Windows temp directories are already per-user, so every entry
    // can be considered for cleanup without an extra ownership check.
    if tmp_dir.exists() {
        if let Ok(entries) = fs::read_dir(&tmp_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                clean_path(
                    &mut result,
                    &path,
                    "temporary file",
                    skip_confirmation,
                    false,
                )?;
            }
        }
    }

    Ok(result)
}

fn clean_package_caches(skip_confirmation: bool) -> Result<CleaningResult> {
    let mut result = CleaningResult::new();

    for (label, path) in platform::package_cache_paths() {
        clean_path(&mut result, &path, &label, skip_confirmation, true)?;
    }

    // Clean the yarn cache via the yarn CLI, if available — measure its real
    // cache directory size before/after rather than guessing.
    if let Ok(dir_output) = std::process::Command::new("yarn")
        .args(["cache", "dir"])
        .output()
    {
        if dir_output.status.success() {
            let yarn_dir = String::from_utf8_lossy(&dir_output.stdout)
                .trim()
                .to_string();
            let yarn_path = Path::new(&yarn_dir);
            if yarn_path.exists() {
                let size = get_size(&yarn_dir).unwrap_or(0);
                if size > 0
                    && (skip_confirmation
                        || confirm(
                            &format!("Clean yarn cache ({} to be freed)?", format_size(size)),
                            true,
                        )?)
                {
                    if let Ok(output) = std::process::Command::new("yarn")
                        .args(["cache", "clean"])
                        .output()
                    {
                        if output.status.success() {
                            print_success(&format!("Cleaned yarn cache ({})", format_size(size)));
                            result.add_item(CleanedItem::directory(
                                yarn_path.to_path_buf(),
                                size,
                                "yarn cache",
                            ));
                        }
                    }
                }
            }
        }
    }

    Ok(result)
}

fn clean_trash(skip_confirmation: bool) -> Result<CleaningResult> {
    let mut result = CleaningResult::new();

    for dir in platform::trash_paths() {
        if !dir.exists() {
            continue;
        }
        let size = get_size(&dir.to_string_lossy())?;
        if size == 0 {
            continue;
        }
        debug!("Trash found at {:?}, size: {}", dir, format_size(size));

        if skip_confirmation
            || confirm(
                &format!(
                    "Empty trash at {:?} ({} to be freed)?",
                    dir,
                    format_size(size)
                ),
                true,
            )?
        {
            // Linux XDG trash: files/ + info/ subdirectories.
            let files_dir = dir.join("files");
            let info_dir = dir.join("info");

            if files_dir.exists() || info_dir.exists() {
                if files_dir.exists() {
                    remove_dir_all(&files_dir).ok();
                    fs::create_dir_all(&files_dir).ok();
                }
                if info_dir.exists() {
                    remove_dir_all(&info_dir).ok();
                    fs::create_dir_all(&info_dir).ok();
                }
            } else {
                // macOS ~/.Trash: flat directory of trashed files.
                if let Ok(entries) = fs::read_dir(&dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            remove_dir_all(&path).ok();
                        } else {
                            remove_file(&path).ok();
                        }
                    }
                }
            }

            print_success(&format!(
                "Emptied trash at {:?} ({})",
                dir,
                format_size(size)
            ));
            result.add_item(CleanedItem::directory(dir, size, "trash"));
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_cleaners_returns_all_six() {
        let cleaners = get_cleaners();
        assert_eq!(cleaners.len(), 6);
        assert!(cleaners.iter().any(|c| c.name == "Browser Caches"));
        assert!(cleaners.iter().any(|c| c.name == "Trash"));
    }

    #[test]
    fn list_cleaners_includes_name_and_description() {
        let list = list_cleaners();
        assert_eq!(list.len(), 6);
        assert!(list[0].contains(':'));
    }

    #[test]
    fn clean_path_skips_nonexistent_path() {
        let mut result = CleaningResult::new();
        clean_path(
            &mut result,
            Path::new("/definitely/does/not/exist/cleansys-test"),
            "test",
            true,
            false,
        )
        .unwrap();
        assert_eq!(result.item_count(), 0);
    }

    #[test]
    fn clean_path_removes_file_and_records_item() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("victim.txt");
        std::fs::write(&file_path, "hello world").unwrap();

        let mut result = CleaningResult::new();
        clean_path(&mut result, &file_path, "test file", true, false).unwrap();

        assert!(!file_path.exists());
        assert_eq!(result.item_count(), 1);
        assert_eq!(result.items[0].label, "test file");
        assert!(result.total_bytes > 0);
    }

    #[test]
    fn clean_path_skips_empty_file() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("empty.txt");
        std::fs::write(&file_path, "").unwrap();

        let mut result = CleaningResult::new();
        clean_path(&mut result, &file_path, "test file", true, false).unwrap();

        // Zero-byte files are skipped entirely (nothing to report/free).
        assert!(file_path.exists());
        assert_eq!(result.item_count(), 0);
    }

    #[test]
    fn clean_path_recreates_directory_when_requested() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("cache_dir");
        std::fs::create_dir_all(&target).unwrap();
        std::fs::write(target.join("data.bin"), vec![0u8; 128]).unwrap();

        let mut result = CleaningResult::new();
        clean_path(&mut result, &target, "recreated cache", true, true).unwrap();

        assert!(target.exists());
        assert!(target.is_dir());
        assert_eq!(std::fs::read_dir(&target).unwrap().count(), 0);
        assert_eq!(result.item_count(), 1);
    }
}
