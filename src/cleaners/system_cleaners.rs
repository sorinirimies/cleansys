use anyhow::Result;
use log::{debug, info, warn};
use std::fs::{self, read_dir, remove_dir_all, remove_file};
use std::path::Path;
use std::process::Command;

use crate::utils::{
    check_root, confirm, format_size, get_size, print_error, print_success, print_warning,
};

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
            name: "Package Manager Caches",
            description: "Clean package manager caches (apt, pacman, dnf, etc.)",
            function: clean_package_caches,
        },
        CleanerInfo {
            name: "System Logs",
            description: "Clean old system logs",
            function: clean_system_logs,
        },
        CleanerInfo {
            name: "System Caches",
            description: "Clean system-wide cache directories",
            function: clean_system_caches,
        },
        CleanerInfo {
            name: "Temporary Files",
            description: "Clean system temporary files",
            function: clean_temp_files,
        },
        CleanerInfo {
            name: "Old Kernels",
            description: "Remove old unused kernels",
            function: clean_old_kernels,
        },
        CleanerInfo {
            name: "Crash Reports",
            description: "Remove system crash reports and core dumps",
            function: clean_crash_reports,
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

fn clean_package_caches(_skip_confirmation: bool) -> Result<u64> {
    let mut bytes_saved = 0;

    info!("Starting package cache cleaning...");

    // Check if we have root privileges
    if !check_root() {
        return Err(anyhow::anyhow!(
            "Root privileges required for package cache cleaning"
        ));
    }

    // Detect package manager and clean caches
    if std::path::Path::new("/usr/bin/apt-get").exists()
        || std::path::Path::new("/usr/bin/apt").exists()
    {
        info!("Found APT package manager, cleaning cache...");
        let cache_size = get_size("/var/cache/apt/archives/").unwrap_or(5 * 1024 * 1024);

        let output = Command::new("apt-get").args(["clean"]).output()?;

        if output.status.success() {
            info!("Successfully cleaned APT cache");
            bytes_saved += cache_size;
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("Failed to clean APT cache: {}", stderr);
        }

        // Also clean autoclean
        let output = Command::new("apt-get").args(["autoclean"]).output()?;

        if output.status.success() {
            info!("Successfully cleaned APT autoclean");
            bytes_saved += cache_size / 2;
        }
    }

    if std::path::Path::new("/usr/bin/pacman").exists() {
        info!("Found Pacman package manager, cleaning cache...");
        let cache_size = get_size("/var/cache/pacman/pkg/").unwrap_or(20 * 1024 * 1024);

        let output = Command::new("pacman")
            .args(["-Sc", "--noconfirm"])
            .output()?;

        if output.status.success() {
            info!("Successfully cleaned Pacman cache");
            bytes_saved += cache_size;
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("Failed to clean Pacman cache: {}", stderr);
        }
    }

    if std::path::Path::new("/usr/bin/dnf").exists() {
        info!("Found DNF package manager, cleaning cache...");
        let cache_size = get_size("/var/cache/dnf/").unwrap_or(10 * 1024 * 1024);

        let output = Command::new("dnf").args(["clean", "all"]).output()?;

        if output.status.success() {
            info!("Successfully cleaned DNF cache");
            bytes_saved += cache_size;
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("Failed to clean DNF cache: {}", stderr);
        }
    }

    info!(
        "Package cache cleaning completed, freed: {}",
        format_size(bytes_saved)
    );
    Ok(bytes_saved)
}

fn clean_system_logs(skip_confirmation: bool) -> Result<u64> {
    let log_paths = vec!["/var/log"];

    let mut bytes_saved = 0;

    for log_path in log_paths {
        let path = Path::new(log_path);
        if path.exists() {
            // Calculate size of files we can safely remove
            let mut size_to_clean = 0;

            if let Ok(entries) = read_dir(path) {
                for entry in entries.flatten() {
                    let file_path = entry.path();
                    let filename = file_path.file_name().unwrap_or_default().to_string_lossy();

                    // Skip current log files and only target rotated logs
                    if file_path.is_file()
                        && (filename.ends_with(".gz")
                            || filename.ends_with(".old")
                            || filename.contains(".1")
                            || filename.contains(".2"))
                    {
                        if let Ok(metadata) = fs::metadata(&file_path) {
                            size_to_clean += metadata.len();
                        }
                    }
                }
            }

            if size_to_clean > 0 {
                if skip_confirmation
                    || confirm(
                        &format!(
                            "Clean old logs in {} ({} to be freed)?",
                            log_path,
                            format_size(size_to_clean)
                        ),
                        true,
                    )?
                {
                    // Use find to delete old log files
                    let output = Command::new("find")
                        .args([
                            log_path, "-type", "f", "-name", "*.gz", "-o", "-name", "*.old", "-o",
                            "-name", "*.1", "-o", "-name", "*.2", "-o", "-name", "*.3", "-o",
                            "-name", "*.4", "-delete",
                        ])
                        .output()?;

                    if output.status.success() {
                        print_success(&format!("Cleaned old logs in {}", log_path));
                        bytes_saved += size_to_clean;
                    } else {
                        print_error(&format!("Failed to clean logs in {}", log_path));
                    }
                }
            } else {
                debug!("No old logs found in {}", log_path);
            }
        }
    }

    // Additionally, use journalctl to vacuum logs if available
    if Command::new("which")
        .arg("journalctl")
        .output()?
        .status
        .success()
    {
        // Get current journal size
        let output = Command::new("journalctl").args(["--disk-usage"]).output()?;

        let disk_usage = String::from_utf8_lossy(&output.stdout);
        debug!("Journal disk usage: {}", disk_usage);

        // Estimate size - this is a rough approximation as we can't easily parse the output
        let journal_size: u64 = 100 * 1024 * 1024; // Default 100MB estimation

        if skip_confirmation || confirm("Vacuum system journal logs?", true)? {
            // Keep only logs from the last week
            let status = Command::new("journalctl")
                .args(["--vacuum-time=7d"])
                .status()?;

            if status.success() {
                print_success("Cleaned system journal logs");
                bytes_saved += journal_size / 2; // Estimate we saved half of the journal size
            } else {
                print_error("Failed to clean system journal logs");
            }
        }
    }

    Ok(bytes_saved)
}

fn clean_system_caches(skip_confirmation: bool) -> Result<u64> {
    let cache_paths = vec![
        "/var/cache/ldconfig",
        "/var/cache/fontconfig",
        "/var/cache/man",
    ];

    let mut bytes_saved = 0;

    for cache_path in cache_paths {
        let path = Path::new(cache_path);
        if path.exists() {
            let size = get_size(cache_path)?;

            if size > 0
                && (skip_confirmation
                    || confirm(
                        &format!(
                            "Clean system cache in {} ({} to be freed)?",
                            cache_path,
                            format_size(size)
                        ),
                        true,
                    )?)
            {
                if path.is_dir() {
                    // Remove content but keep the directory
                    if let Ok(entries) = read_dir(path) {
                        for entry in entries.flatten() {
                            let file_path = entry.path();

                            if file_path.is_file() {
                                if let Err(e) = remove_file(&file_path) {
                                    warn!("Failed to remove file {:?}: {}", file_path, e);
                                }
                            } else if file_path.is_dir() {
                                if let Err(e) = remove_dir_all(&file_path) {
                                    warn!("Failed to remove directory {:?}: {}", file_path, e);
                                }
                            }
                        }
                    }
                } else if path.is_file() {
                    if let Err(e) = remove_file(path) {
                        warn!("Failed to remove file {:?}: {}", path, e);
                        continue;
                    }
                }

                print_success(&format!("Cleaned system cache in {}", cache_path));
                bytes_saved += size;
            }
        }
    }

    // Run updatedb to update locate database if it exists
    if Command::new("which")
        .arg("updatedb")
        .output()?
        .status
        .success()
        && (skip_confirmation || confirm("Update locate database?", true)?)
    {
        let status = Command::new("updatedb").status()?;

        if status.success() {
            print_success("Updated locate database");
        } else {
            print_error("Failed to update locate database");
        }
    }

    Ok(bytes_saved)
}

fn clean_temp_files(skip_confirmation: bool) -> Result<u64> {
    let temp_paths = vec!["/tmp", "/var/tmp"];

    let mut bytes_saved = 0;

    for temp_path in temp_paths {
        let path = Path::new(temp_path);
        if path.exists() {
            // Calculate size of files we can safely remove (not currently in use)
            let output = Command::new("find")
                .args([
                    temp_path, "-type", "f", "-atime",
                    "+1", // Files not accessed in the last day
                    "-exec", "du", "-sc", "{}", ";",
                ])
                .output()?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut size_to_clean: u64 = 0;

            // Try to parse the total size from du output
            if let Some(total_line) = stdout.lines().last() {
                if let Some(size_str) = total_line.split_whitespace().next() {
                    size_to_clean = size_str.parse::<u64>().unwrap_or(0) * 1024;
                    // Convert KB to bytes
                }
            }

            if size_to_clean > 0 {
                if skip_confirmation
                    || confirm(
                        &format!(
                            "Clean old temporary files in {} ({} to be freed)?",
                            temp_path,
                            format_size(size_to_clean)
                        ),
                        true,
                    )?
                {
                    // Use find to delete old temporary files
                    let status = Command::new("find")
                        .args([
                            temp_path, "-type", "f", "-atime",
                            "+1", // Files not accessed in the last day
                            "-delete",
                        ])
                        .status()?;

                    if status.success() {
                        print_success(&format!("Cleaned old temporary files in {}", temp_path));
                        bytes_saved += size_to_clean;
                    } else {
                        print_error(&format!("Failed to clean temporary files in {}", temp_path));
                    }
                }
            } else {
                debug!("No old temporary files found in {}", temp_path);
            }
        }
    }

    Ok(bytes_saved)
}

fn clean_old_kernels(skip_confirmation: bool) -> Result<u64> {
    let mut bytes_saved = 0;

    // Only try to clean kernels on systems with apt (Debian/Ubuntu)
    if Command::new("which").arg("apt").output()?.status.success()
        && Command::new("which").arg("dpkg").output()?.status.success()
    {
        // Get current kernel version
        let output = Command::new("uname").arg("-r").output()?;
        let current_kernel = String::from_utf8_lossy(&output.stdout).trim().to_string();
        debug!("Current kernel: {}", current_kernel);

        // List installed kernels
        let output = Command::new("dpkg")
            .args(["-l", "linux-image-*"])
            .output()?;

        let installed_kernels = String::from_utf8_lossy(&output.stdout);

        // Count how many kernels are installed
        let kernel_count = installed_kernels
            .lines()
            .filter(|l| l.contains("linux-image-") && l.starts_with("ii"))
            .count();

        debug!("Found {} installed kernels", kernel_count);

        // Only clean if we have more than 2 kernels (current + previous)
        if kernel_count > 2 {
            // Estimate size to be cleaned (average kernel size is around 200MB)
            let estimated_size = (kernel_count - 2) as u64 * 200 * 1024 * 1024;

            if skip_confirmation
                || confirm(
                    &format!(
                        "Remove old kernels (approximately {} to be freed)?",
                        format_size(estimated_size)
                    ),
                    true,
                )?
            {
                // Check if we have purge-old-kernels command (from byobu package)
                if Command::new("which")
                    .arg("purge-old-kernels")
                    .output()?
                    .status
                    .success()
                {
                    let status = Command::new("purge-old-kernels")
                        .args(["--keep", "1"])
                        .status()?;

                    if status.success() {
                        print_success("Removed old kernels");
                        bytes_saved += estimated_size;
                    } else {
                        print_error("Failed to remove old kernels");
                    }
                } else {
                    // Use apt to clean old kernels - this is less safe, so we'll skip it
                    print_warning("purge-old-kernels not found. Install byobu package for safer kernel cleanup.");
                }
            }
        } else {
            debug!("Not enough kernels installed to clean");
        }
    }

    Ok(bytes_saved)
}

fn clean_crash_reports(skip_confirmation: bool) -> Result<u64> {
    let crash_paths = vec!["/var/crash", "/var/lib/systemd/coredump"];

    let mut bytes_saved = 0;

    for crash_path in crash_paths {
        let path = Path::new(crash_path);
        if path.exists() {
            let size = get_size(crash_path)?;

            if size > 0
                && (skip_confirmation
                    || confirm(
                        &format!(
                            "Clean crash reports in {} ({} to be freed)?",
                            crash_path,
                            format_size(size)
                        ),
                        true,
                    )?)
            {
                if path.is_dir() {
                    // Remove content but keep the directory
                    if let Ok(entries) = read_dir(path) {
                        for entry in entries.flatten() {
                            let file_path = entry.path();

                            if file_path.is_file() {
                                if let Err(e) = remove_file(&file_path) {
                                    warn!("Failed to remove file {:?}: {}", file_path, e);
                                }
                            } else if file_path.is_dir() {
                                if let Err(e) = remove_dir_all(&file_path) {
                                    warn!("Failed to remove directory {:?}: {}", file_path, e);
                                }
                            }
                        }
                    }
                }

                print_success(&format!("Cleaned crash reports in {}", crash_path));
                bytes_saved += size;
            }
        }
    }

    // Clean core dumps if we can find any
    if Command::new("which").arg("find").output()?.status.success() {
        let output = Command::new("find")
            .args([
                "/", "-name", "core", "-o", "-name", "core.*", "-type", "f", "-size",
                "+10k", // Only files larger than 10KB
                "-exec", "du", "-sc", "{}", ";",
            ])
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut size_to_clean: u64 = 0;

        // Try to parse the total size from du output
        if let Some(total_line) = stdout.lines().last() {
            if let Some(size_str) = total_line.split_whitespace().next() {
                size_to_clean = size_str.parse::<u64>().unwrap_or(0) * 1024; // Convert KB to bytes
            }
        }

        if size_to_clean > 0
            && (skip_confirmation
                || confirm(
                    &format!(
                        "Clean core dumps across system ({} to be freed)?",
                        format_size(size_to_clean)
                    ),
                    true,
                )?)
        {
            let status = Command::new("find")
                .args([
                    "/", "-name", "core", "-o", "-name", "core.*", "-type", "f", "-size",
                    "+10k", // Only files larger than 10KB
                    "-delete",
                ])
                .status()?;

            if status.success() {
                print_success("Cleaned core dumps");
                bytes_saved += size_to_clean;
            } else {
                print_error("Failed to clean core dumps");
            }
        }
    }

    Ok(bytes_saved)
}
