use anyhow::{Context, Result};
use directories::BaseDirs;
use log::{debug, warn};
use std::fs::{self, read_dir, remove_dir_all, remove_file};
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
use std::path::Path;

use crate::utils::{confirm, format_size, get_size, print_error, print_success};

pub struct CleanerInfo {
    pub name: &'static str,
    pub description: &'static str,
    pub function: fn(bool) -> Result<u64>,
}

pub fn list_cleaners() -> Vec<String> {
    get_cleaners()
        .iter()
        .map(|c| format!("{}: {}", c.name, c.description))
        .collect()
}

pub fn get_cleaners() -> Vec<CleanerInfo> {
    vec![
        CleanerInfo {
            name: "Browser Caches",
            description: "Clean Firefox and Chrome/Chromium caches",
            function: clean_browser_caches,
        },
        CleanerInfo {
            name: "Application Caches",
            description: "Clean application caches in ~/.cache",
            function: clean_app_caches,
        },
        CleanerInfo {
            name: "Thumbnail Caches",
            description: "Clean thumbnail caches",
            function: clean_thumbnail_caches,
        },
        CleanerInfo {
            name: "Temporary Files",
            description: "Clean temporary files in /tmp owned by the user",
            function: clean_temp_files,
        },
        CleanerInfo {
            name: "Package Manager Caches",
            description: "Clean user package manager caches like pip, npm, cargo",
            function: clean_package_caches,
        },
        CleanerInfo {
            name: "Trash",
            description: "Empty trash folder",
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
                Ok(bytes) => {
                    total_saved += bytes;
                    print_success(&format!(
                        "{} completed: freed {}",
                        cleaner.name,
                        format_size(bytes)
                    ));
                }
                Err(err) => {
                    print_error(&format!("Error in {}: {}", cleaner.name, err));
                }
            }
        }
    }

    print_success(&format!("Total space freed: {}", format_size(total_saved)));
    Ok(())
}

fn clean_browser_caches(skip_confirmation: bool) -> Result<u64> {
    let mut bytes_saved = 0;
    let base_dirs = BaseDirs::new().context("Failed to get base directories")?;
    let home_dir = base_dirs.home_dir();

    // Firefox cache
    let firefox_path = home_dir.join(".mozilla/firefox");
    if firefox_path.exists() {
        debug!("Firefox directory found at {:?}", firefox_path);

        if let Ok(entries) = read_dir(&firefox_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir()
                    && path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .ends_with(".default")
                {
                    let cache_path = path.join("cache2");

                    if cache_path.exists() {
                        let size = get_size(cache_path.to_str().unwrap_or(""))?;

                        if skip_confirmation
                            || confirm(
                                &format!(
                                    "Clean Firefox cache ({} to be freed)?",
                                    format_size(size)
                                ),
                                true,
                            )?
                        {
                            remove_dir_all(&cache_path)
                                .context("Failed to remove Firefox cache")?;
                            print_success("Firefox cache cleaned");
                            bytes_saved += size;
                        }
                    }
                }
            }
        }
    }

    // Chrome/Chromium cache
    let chrome_paths = vec![
        home_dir.join(".config/google-chrome/Default/Cache"),
        home_dir.join(".config/chromium/Default/Cache"),
        home_dir.join(".cache/google-chrome"),
        home_dir.join(".cache/chromium"),
    ];

    for path in chrome_paths {
        if path.exists() {
            debug!("Chrome/Chromium cache found at {:?}", path);
            let size = get_size(path.to_str().unwrap_or(""))?;

            if skip_confirmation
                || confirm(
                    &format!(
                        "Clean Chrome/Chromium cache at {:?} ({} to be freed)?",
                        path,
                        format_size(size)
                    ),
                    true,
                )?
            {
                remove_dir_all(&path).context("Failed to remove Chrome/Chromium cache")?;
                print_success(&format!("Chrome/Chromium cache at {:?} cleaned", path));
                bytes_saved += size;
            }
        }
    }

    Ok(bytes_saved)
}

fn clean_app_caches(skip_confirmation: bool) -> Result<u64> {
    let base_dirs = BaseDirs::new().context("Failed to get base directories")?;
    let cache_dir = base_dirs.cache_dir();
    let mut bytes_saved = 0;

    debug!("Cache directory: {:?}", cache_dir);

    if cache_dir.exists() {
        // Get list of directories in cache_dir
        if let Ok(entries) = read_dir(cache_dir) {
            for entry in entries.flatten() {
                let path = entry.path();

                // Skip certain critical directories
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                if ["dconf", "fontconfig", "mesa_shader_cache"].contains(&name.as_ref()) {
                    debug!("Skipping critical cache directory: {:?}", path);
                    continue;
                }

                if path.is_dir() {
                    let size = get_size(path.to_str().unwrap_or(""))?;

                    if skip_confirmation
                        || confirm(
                            &format!(
                                "Clean cache for '{}' ({} to be freed)?",
                                name,
                                format_size(size)
                            ),
                            true,
                        )?
                    {
                        if let Err(e) = remove_dir_all(&path) {
                            warn!("Failed to remove cache directory {:?}: {}", path, e);
                            continue;
                        }

                        print_success(&format!("Cleaned '{}' cache", name));
                        bytes_saved += size;
                    }
                }
            }
        }
    }

    Ok(bytes_saved)
}

fn clean_thumbnail_caches(skip_confirmation: bool) -> Result<u64> {
    let base_dirs = BaseDirs::new().context("Failed to get base directories")?;
    let home_dir = base_dirs.home_dir();
    let thumbnail_dirs = vec![
        home_dir.join(".thumbnails"),
        home_dir.join(".cache/thumbnails"),
    ];

    let mut bytes_saved = 0;

    for dir in thumbnail_dirs {
        if dir.exists() {
            let size = get_size(dir.to_str().unwrap_or(""))?;
            debug!(
                "Thumbnail cache found at {:?}, size: {}",
                dir,
                format_size(size)
            );

            if skip_confirmation
                || confirm(
                    &format!(
                        "Clean thumbnail cache at {:?} ({} to be freed)?",
                        dir,
                        format_size(size)
                    ),
                    true,
                )?
            {
                remove_dir_all(&dir).context("Failed to remove thumbnail cache")?;
                fs::create_dir_all(&dir).context("Failed to recreate thumbnail directory")?;
                print_success(&format!("Cleaned thumbnail cache at {:?}", dir));
                bytes_saved += size;
            }
        }
    }

    Ok(bytes_saved)
}

#[cfg(unix)]
fn clean_temp_files(skip_confirmation: bool) -> Result<u64> {
    let tmp_dir = Path::new("/tmp");
    let mut bytes_saved = 0;

    if tmp_dir.exists() {
        if let Ok(entries) = read_dir(tmp_dir) {
            for entry in entries.flatten() {
                let path = entry.path();

                // Check if the file or directory is owned by the current user
                if let Ok(metadata) = fs::metadata(&path) {
                    let uid = metadata.uid();

                    if uid == users::get_current_uid() {
                        let size = get_size(path.to_str().unwrap_or(""))?;

                        if skip_confirmation
                            || confirm(
                                &format!(
                                    "Remove temporary file/directory {:?} ({} to be freed)?",
                                    path,
                                    format_size(size)
                                ),
                                true,
                            )?
                        {
                            if path.is_dir() {
                                if let Err(e) = remove_dir_all(&path) {
                                    warn!("Failed to remove directory {:?}: {}", path, e);
                                    continue;
                                }
                            } else if let Err(e) = remove_file(&path) {
                                warn!("Failed to remove file {:?}: {}", path, e);
                                continue;
                            }

                            print_success(&format!("Removed temporary {:?}", path));
                            bytes_saved += size;
                        }
                    }
                }
            }
        }
    }

    Ok(bytes_saved)
}

#[cfg(not(unix))]
fn clean_temp_files(_skip_confirmation: bool) -> Result<u64> {
    Ok(0)
}

fn clean_package_caches(skip_confirmation: bool) -> Result<u64> {
    let base_dirs = BaseDirs::new().context("Failed to get base directories")?;
    let home_dir = base_dirs.home_dir();

    let cache_locations = vec![
        (home_dir.join(".cache/pip"), "pip"),
        (home_dir.join(".npm/_cacache"), "npm"),
        (home_dir.join(".cargo/.crates.toml.lock"), "cargo lock file"),
        (
            home_dir.join(".cargo/.package-cache"),
            "cargo package cache",
        ),
    ];

    let mut bytes_saved = 0;

    for (path, name) in cache_locations {
        if path.exists() {
            let size = get_size(path.to_str().unwrap_or(""))?;
            debug!(
                "{} cache found: {:?}, size: {}",
                name,
                path,
                format_size(size)
            );

            if skip_confirmation
                || confirm(
                    &format!("Clean {} cache ({} to be freed)?", name, format_size(size)),
                    true,
                )?
            {
                if path.is_dir() {
                    if let Err(e) = remove_dir_all(&path) {
                        warn!("Failed to remove {} cache: {}", name, e);
                        continue;
                    }
                    fs::create_dir_all(&path).ok(); // Recreate empty directory
                } else if let Err(e) = remove_file(&path) {
                    warn!("Failed to remove {} cache: {}", name, e);
                    continue;
                }

                print_success(&format!("Cleaned {} cache", name));
                bytes_saved += size;
            }
        }
    }

    // Clean yarn cache with the yarn command if available
    if skip_confirmation || confirm("Clean yarn cache?", true)? {
        if let Ok(output) = std::process::Command::new("yarn")
            .arg("cache")
            .arg("clean")
            .output()
        {
            if output.status.success() {
                print_success("Cleaned yarn cache");
                // Since we can't easily determine the size, estimate 10MB
                bytes_saved += 10 * 1024 * 1024;
            }
        }
    }

    Ok(bytes_saved)
}

fn clean_trash(skip_confirmation: bool) -> Result<u64> {
    let base_dirs = BaseDirs::new().context("Failed to get base directories")?;
    let home_dir = base_dirs.home_dir();
    let trash_dirs = vec![
        home_dir.join(".local/share/Trash"),
        Path::new("~/.Trash").to_path_buf(),
    ];

    let mut bytes_saved = 0;

    for dir in trash_dirs {
        if dir.exists() {
            let size = get_size(dir.to_str().unwrap_or(""))?;
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
                // Remove files and info subdirectories in trash
                let files_dir = dir.join("files");
                let info_dir = dir.join("info");

                if files_dir.exists() {
                    remove_dir_all(&files_dir).context("Failed to empty trash files")?;
                    fs::create_dir_all(&files_dir).ok();
                }

                if info_dir.exists() {
                    remove_dir_all(&info_dir).context("Failed to empty trash info")?;
                    fs::create_dir_all(&info_dir).ok();
                }

                print_success(&format!("Emptied trash at {:?}", dir));
                bytes_saved += size;
            }
        }
    }

    Ok(bytes_saved)
}
