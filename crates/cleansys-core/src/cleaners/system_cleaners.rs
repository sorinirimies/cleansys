use anyhow::Result;
#[cfg(target_os = "linux")]
use log::info;
use log::{debug, warn};
use std::path::Path;
use std::process::Command;

use crate::cleaners::cleaned_item::{CleanedItem, CleanerFn, CleaningResult, RunOptions};
#[cfg(any(target_os = "macos", target_os = "windows"))]
use crate::cleaners::platform;
#[cfg(target_os = "linux")]
use crate::utils::check_root;
#[cfg(target_os = "linux")]
use crate::utils::print_warning;
use crate::utils::{confirm, execute_with_sudo, format_size, get_size, print_error, print_success};

/// Information about a system cleaner.
pub struct CleanerInfo {
    /// The name of the cleaner.
    pub name: &'static str,
    /// A description of what the cleaner does.
    pub description: &'static str,
    /// The function that performs the cleaning operation.
    pub function: CleanerFn,
    /// Whether this specific cleaner needs root/Administrator privileges.
    ///
    /// Unlike Linux (where every system-level path needs root), not every
    /// "system cleaner" needs elevated privileges on every platform —
    /// notably Homebrew on macOS must **not** be run as root. This is
    /// declared per-cleaner rather than assumed true for the whole category.
    pub requires_root: bool,
}

/// Lists all available system cleaners with their descriptions.
pub fn list_cleaners() -> Vec<String> {
    get_cleaners()
        .iter()
        .map(|c| format!("{}: {}", c.name, c.description))
        .collect()
}

/// Returns the system cleaners applicable to the current platform.
///
/// Unlike the user-level cleaners (which check OS-appropriate paths behind a
/// single shared list), system cleaners differ enough in *mechanism*
/// (apt/pacman/dnf on Linux vs. Homebrew on macOS vs. no equivalent concept
/// on Windows) that each platform gets its own cleaner list.
pub fn get_cleaners() -> Vec<CleanerInfo> {
    #[cfg(target_os = "linux")]
    {
        linux_cleaners()
    }
    #[cfg(target_os = "macos")]
    {
        macos_cleaners()
    }
    #[cfg(target_os = "windows")]
    {
        windows_cleaners()
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        Vec::new()
    }
}

/// Runs all system cleaners.
pub fn run_all(skip_confirmation: bool) -> Result<()> {
    let cleaners = get_cleaners();
    let mut total_saved: u64 = 0;
    let opts = if skip_confirmation {
        RunOptions::execute()
    } else {
        RunOptions::execute_with_confirmation()
    };

    for cleaner in cleaners {
        if skip_confirmation || confirm(&format!("Run '{}'?", cleaner.name), true)? {
            match (cleaner.function)(opts) {
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
                    print_error(&format!("Error in {}: {}", cleaner.name, err));
                }
            }
        }
    }

    print_success(&format!("Total space freed: {}", format_size(total_saved)));
    Ok(())
}

/// Measure the real size of `path` before and after calling `action`, and
/// record the actual bytes freed (never an estimate/guess) as a
/// [`CleanedItem`] in `result` if anything was actually freed.
///
/// In [`RunOptions::preview`] mode, `action` (which typically shells out to a
/// mutating command like `apt-get clean`) is never invoked; instead the
/// entire pre-existing size of `path` is reported as the projected amount
/// that a real run would free.
fn measure_around<F>(
    result: &mut CleaningResult,
    path: &Path,
    label: &str,
    opts: RunOptions,
    action: F,
) -> Result<()>
where
    F: FnOnce() -> Result<bool>,
{
    let before = get_size(&path.to_string_lossy()).unwrap_or(0);

    if opts.dry_run {
        if before > 0 {
            result.add_item(CleanedItem::directory(path.to_path_buf(), before, label));
        }
        return Ok(());
    }

    let succeeded = action()?;
    if !succeeded {
        return Ok(());
    }
    let after = get_size(&path.to_string_lossy()).unwrap_or(before);
    let freed = before.saturating_sub(after);

    if freed > 0 {
        print_success(&format!("{label}: freed {}", format_size(freed)));
        result.add_item(CleanedItem::directory(path.to_path_buf(), freed, label));
    }

    Ok(())
}

/// Measure, and if not in preview mode remove, the contents of a single
/// path (file or whole directory tree) — the common pattern shared by most
/// "rm -rf $path/*" style system cleaners.
fn measure_and_remove(
    result: &mut CleaningResult,
    path: &Path,
    label: &str,
    opts: RunOptions,
    remove: impl FnOnce() -> Result<std::process::Output>,
) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    let size = get_size(&path.to_string_lossy())?;
    if size == 0 {
        return Ok(());
    }

    if opts.dry_run {
        result.add_item(CleanedItem::directory(path.to_path_buf(), size, label));
        return Ok(());
    }

    if !opts.skip_confirmation
        && !confirm(
            &format!("Clean {label} ({} to be freed)?", format_size(size)),
            true,
        )?
    {
        return Ok(());
    }

    match remove() {
        Ok(out) if out.status.success() => {
            print_success(&format!("Cleaned {label} ({})", format_size(size)));
            result.add_item(CleanedItem::directory(path.to_path_buf(), size, label));
        }
        Ok(_) => warn!("Failed to clean {label}"),
        Err(e) => warn!("Failed to execute cleanup for {label}: {e}"),
    }

    Ok(())
}

// ── Linux ────────────────────────────────────────────────────────────────────

#[cfg(target_os = "linux")]
fn linux_cleaners() -> Vec<CleanerInfo> {
    vec![
        CleanerInfo {
            name: "Package Manager Caches",
            description: "Clean package manager caches (apt, pacman, dnf)",
            function: clean_package_caches,
            requires_root: true,
        },
        CleanerInfo {
            name: "System Logs",
            description: "Clean rotated system logs and vacuum the systemd journal",
            function: clean_system_logs,
            requires_root: true,
        },
        CleanerInfo {
            name: "System Caches",
            description: "Clean system-wide cache directories (fontconfig, man, ldconfig)",
            function: clean_system_caches,
            requires_root: true,
        },
        CleanerInfo {
            name: "Temporary Files",
            description: "Clean old temporary files in /tmp and /var/tmp",
            function: clean_temp_files,
            requires_root: true,
        },
        CleanerInfo {
            name: "Old Kernels",
            description: "Remove old unused kernels (requires purge-old-kernels)",
            function: clean_old_kernels,
            requires_root: true,
        },
        CleanerInfo {
            name: "Crash Reports",
            description: "Remove system crash reports and core dumps",
            function: clean_crash_reports,
            requires_root: true,
        },
    ]
}

#[cfg(target_os = "linux")]
fn clean_package_caches(opts: RunOptions) -> Result<CleaningResult> {
    let mut result = CleaningResult::new();
    info!("Starting package cache cleaning...");

    if !opts.dry_run && !check_root() {
        return Err(anyhow::anyhow!(
            "Root privileges required for package cache cleaning"
        ));
    }

    if Path::new("/usr/bin/apt-get").exists() || Path::new("/usr/bin/apt").exists() {
        info!("Found APT package manager, cleaning cache...");
        measure_around(
            &mut result,
            Path::new("/var/cache/apt/archives"),
            "APT cache",
            opts,
            || {
                let output = execute_with_sudo("apt-get", &["clean"])?;
                if !output.status.success() {
                    warn!(
                        "Failed to clean APT cache: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
                Ok(output.status.success())
            },
        )?;
    }

    if Path::new("/usr/bin/pacman").exists() {
        info!("Found Pacman package manager, cleaning cache...");
        measure_around(
            &mut result,
            Path::new("/var/cache/pacman/pkg"),
            "Pacman cache",
            opts,
            || {
                let output = execute_with_sudo("pacman", &["-Sc", "--noconfirm"])?;
                if !output.status.success() {
                    warn!(
                        "Failed to clean Pacman cache: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
                Ok(output.status.success())
            },
        )?;
    }

    if Path::new("/usr/bin/dnf").exists() {
        info!("Found DNF package manager, cleaning cache...");
        measure_around(
            &mut result,
            Path::new("/var/cache/dnf"),
            "DNF cache",
            opts,
            || {
                let output = execute_with_sudo("dnf", &["clean", "all"])?;
                if !output.status.success() {
                    warn!(
                        "Failed to clean DNF cache: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
                Ok(output.status.success())
            },
        )?;
    }

    info!(
        "Package cache cleaning completed, freed: {}",
        format_size(result.total_bytes)
    );
    Ok(result)
}

#[cfg(target_os = "linux")]
fn clean_system_logs(opts: RunOptions) -> Result<CleaningResult> {
    let mut result = CleaningResult::new();
    let log_path = Path::new("/var/log");

    if log_path.exists() {
        // Measure just the rotated/compressed logs we're actually targeting,
        // not the whole of /var/log (which includes live logs we never touch).
        let mut size_to_clean = 0u64;
        if let Ok(entries) = std::fs::read_dir(log_path) {
            for entry in entries.flatten() {
                let file_path = entry.path();
                let filename = file_path.file_name().unwrap_or_default().to_string_lossy();
                if file_path.is_file()
                    && (filename.ends_with(".gz")
                        || filename.ends_with(".old")
                        || filename.contains(".1")
                        || filename.contains(".2"))
                {
                    size_to_clean += std::fs::metadata(&file_path).map(|m| m.len()).unwrap_or(0);
                }
            }
        }

        if size_to_clean > 0 {
            if opts.dry_run {
                result.add_item(CleanedItem::directory(
                    log_path.to_path_buf(),
                    size_to_clean,
                    "rotated system logs",
                ));
            } else if opts.skip_confirmation
                || confirm(
                    &format!(
                        "Clean old logs in /var/log ({} to be freed)?",
                        format_size(size_to_clean)
                    ),
                    true,
                )?
            {
                let output = execute_with_sudo(
                    "find",
                    &[
                        "/var/log", "-type", "f", "-name", "*.gz", "-o", "-name", "*.old", "-o",
                        "-name", "*.1", "-o", "-name", "*.2", "-o", "-name", "*.3", "-o", "-name",
                        "*.4", "-delete",
                    ],
                )?;

                if output.status.success() {
                    print_success(&format!(
                        "Cleaned old logs in /var/log ({})",
                        format_size(size_to_clean)
                    ));
                    result.add_item(CleanedItem::directory(
                        log_path.to_path_buf(),
                        size_to_clean,
                        "rotated system logs",
                    ));
                } else {
                    print_error("Failed to clean logs in /var/log");
                }
            } else {
                debug!("No old logs found in /var/log");
            }
        }
    }

    // Vacuum the systemd journal, measuring the real size before/after.
    let has_journalctl = Command::new("which")
        .arg("journalctl")
        .output()?
        .status
        .success();

    if has_journalctl {
        if opts.dry_run {
            let before = get_size("/var/log/journal").unwrap_or(0);
            if before > 0 {
                result.add_item(CleanedItem::directory(
                    Path::new("/var/log/journal").to_path_buf(),
                    before,
                    "systemd journal",
                ));
            }
        } else if opts.skip_confirmation || confirm("Vacuum system journal logs?", true)? {
            measure_around(
                &mut result,
                Path::new("/var/log/journal"),
                "systemd journal",
                opts,
                || {
                    let output = execute_with_sudo("journalctl", &["--vacuum-time=7d"])?;
                    if !output.status.success() {
                        print_error("Failed to clean system journal logs");
                    }
                    Ok(output.status.success())
                },
            )?;
        }
    }

    Ok(result)
}

#[cfg(target_os = "linux")]
fn clean_system_caches(opts: RunOptions) -> Result<CleaningResult> {
    let mut result = CleaningResult::new();
    let cache_paths = ["/var/cache/fontconfig", "/var/cache/man"];

    for cache_path in cache_paths {
        measure_and_remove(&mut result, Path::new(cache_path), cache_path, opts, || {
            execute_with_sudo("sh", &["-c", &format!("rm -rf {cache_path}/*")])
        })?;
    }

    if !opts.dry_run {
        let has_updatedb = Command::new("which")
            .arg("updatedb")
            .output()?
            .status
            .success();
        if has_updatedb && (opts.skip_confirmation || confirm("Update locate database?", true)?) {
            let output = execute_with_sudo("updatedb", &[])?;
            if output.status.success() {
                print_success("Updated locate database");
            } else {
                print_error("Failed to update locate database");
            }
        }
    }

    Ok(result)
}

#[cfg(target_os = "linux")]
fn clean_temp_files(opts: RunOptions) -> Result<CleaningResult> {
    let mut result = CleaningResult::new();

    for temp_path in ["/tmp", "/var/tmp"] {
        let path = Path::new(temp_path);
        if !path.exists() {
            continue;
        }

        // Only target files not accessed in the last day, matching the
        // previous behaviour, but measure real freed bytes via before/after
        // rather than parsing `du` output from a slow per-file `find -exec`.
        let before = get_size(temp_path).unwrap_or(0);
        let has_old_files = Command::new("find")
            .args([temp_path, "-type", "f", "-atime", "+1", "-print", "-quit"])
            .output()
            .map(|o| !o.stdout.is_empty())
            .unwrap_or(false);

        if !has_old_files {
            debug!("No old temporary files found in {temp_path}");
            continue;
        }

        if opts.dry_run {
            if before > 0 {
                result.add_item(CleanedItem::directory(
                    path.to_path_buf(),
                    before,
                    temp_path,
                ));
            }
            continue;
        }

        if opts.skip_confirmation
            || confirm(&format!("Clean old temporary files in {temp_path}?"), true)?
        {
            let output = execute_with_sudo(
                "find",
                &[temp_path, "-type", "f", "-atime", "+1", "-delete"],
            )?;

            if output.status.success() {
                let after = get_size(temp_path).unwrap_or(before);
                let freed = before.saturating_sub(after);
                if freed > 0 {
                    print_success(&format!(
                        "Cleaned old temporary files in {temp_path} ({})",
                        format_size(freed)
                    ));
                    result.add_item(CleanedItem::directory(path.to_path_buf(), freed, temp_path));
                }
            } else {
                print_error(&format!("Failed to clean temporary files in {temp_path}"));
            }
        }
    }

    Ok(result)
}

#[cfg(target_os = "linux")]
fn clean_old_kernels(opts: RunOptions) -> Result<CleaningResult> {
    let mut result = CleaningResult::new();

    if !(Command::new("which").arg("apt").output()?.status.success()
        && Command::new("which").arg("dpkg").output()?.status.success())
    {
        return Ok(result);
    }

    let output = Command::new("dpkg")
        .args(["-l", "linux-image-*"])
        .output()?;
    let installed_kernels = String::from_utf8_lossy(&output.stdout);
    let kernel_count = installed_kernels
        .lines()
        .filter(|l| l.contains("linux-image-") && l.starts_with("ii"))
        .count();
    debug!("Found {kernel_count} installed kernels");

    if kernel_count <= 2 {
        debug!("Not enough kernels installed to clean");
        return Ok(result);
    }

    if !Command::new("which")
        .arg("purge-old-kernels")
        .output()?
        .status
        .success()
    {
        print_warning(
            "purge-old-kernels not found. Install the byobu package for safer kernel cleanup.",
        );
        return Ok(result);
    }

    if opts.dry_run {
        let before = get_size("/boot").unwrap_or(0);
        if before > 0 {
            result.add_item(CleanedItem::directory(
                Path::new("/boot").to_path_buf(),
                before,
                "old kernels",
            ));
        }
        return Ok(result);
    }

    if opts.skip_confirmation
        || confirm(
            &format!(
                "Remove old kernels ({} installed, keeping 1)?",
                kernel_count
            ),
            true,
        )?
    {
        measure_around(&mut result, Path::new("/boot"), "old kernels", opts, || {
            let output = execute_with_sudo("purge-old-kernels", &["--keep", "1"])?;
            if !output.status.success() {
                print_error("Failed to remove old kernels");
            }
            Ok(output.status.success())
        })?;
    }

    Ok(result)
}

#[cfg(target_os = "linux")]
fn clean_crash_reports(opts: RunOptions) -> Result<CleaningResult> {
    let mut result = CleaningResult::new();

    for crash_path in ["/var/crash", "/var/lib/systemd/coredump"] {
        measure_and_remove(&mut result, Path::new(crash_path), crash_path, opts, || {
            execute_with_sudo("sh", &["-c", &format!("rm -rf {crash_path}/*")])
        })?;
    }

    Ok(result)
}

// ── macOS ────────────────────────────────────────────────────────────────────

#[cfg(target_os = "macos")]
fn macos_cleaners() -> Vec<CleanerInfo> {
    vec![
        CleanerInfo {
            name: "Homebrew Cache",
            description: "Clean the Homebrew download cache (brew cleanup)",
            function: clean_homebrew_cache,
            // Homebrew must NOT be run as root/sudo.
            requires_root: false,
        },
        CleanerInfo {
            name: "Xcode Derived Data",
            description:
                "Clean Xcode DerivedData build caches (~/Library/Developer/Xcode/DerivedData)",
            function: clean_xcode_derived_data,
            requires_root: false,
        },
        CleanerInfo {
            name: "iOS Simulator Caches",
            description: "Clean unavailable iOS/watchOS/tvOS Simulator devices and their caches",
            function: clean_ios_simulator_caches,
            requires_root: false,
        },
        CleanerInfo {
            name: "System Logs",
            description: "Remove old rotated system logs",
            function: clean_system_logs,
            requires_root: true,
        },
        CleanerInfo {
            name: "Crash Reports",
            description: "Remove system diagnostic/crash reports",
            function: clean_crash_reports,
            requires_root: true,
        },
    ]
}

#[cfg(target_os = "macos")]
fn clean_homebrew_cache(opts: RunOptions) -> Result<CleaningResult> {
    let mut result = CleaningResult::new();

    if Command::new("which")
        .arg("brew")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        let cache_dir_output = Command::new("brew").arg("--cache").output()?;
        let cache_dir = String::from_utf8_lossy(&cache_dir_output.stdout)
            .trim()
            .to_string();
        let cache_path = Path::new(&cache_dir);

        if cache_path.exists() {
            if opts.dry_run {
                let before = get_size(&cache_dir).unwrap_or(0);
                if before > 0 {
                    result.add_item(CleanedItem::directory(
                        cache_path.to_path_buf(),
                        before,
                        "Homebrew cache",
                    ));
                }
            } else if opts.skip_confirmation
                || confirm("Clean Homebrew cache (brew cleanup)?", true)?
            {
                measure_around(&mut result, cache_path, "Homebrew cache", opts, || {
                    let output = Command::new("brew").args(["cleanup", "-s"]).output()?;
                    if !output.status.success() {
                        warn!(
                            "brew cleanup failed: {}",
                            String::from_utf8_lossy(&output.stderr)
                        );
                    }
                    Ok(output.status.success())
                })?;
            }
        }
    } else {
        debug!("Homebrew not installed — skipping Homebrew cache cleaner");
    }

    Ok(result)
}

/// Xcode's build system cache — regenerated automatically on next build, and
/// commonly grows into tens of GB over time. Safe to delete wholesale.
#[cfg(target_os = "macos")]
fn clean_xcode_derived_data(opts: RunOptions) -> Result<CleaningResult> {
    let mut result = CleaningResult::new();
    let Some(home) = platform::home_dir() else {
        return Ok(result);
    };
    let path = home.join("Library/Developer/Xcode/DerivedData");

    if path.exists() {
        let size = get_size(&path.to_string_lossy())?;
        if size == 0 {
            return Ok(result);
        }

        if opts.dry_run {
            result.add_item(CleanedItem::directory(path, size, "Xcode DerivedData"));
            return Ok(result);
        }

        if opts.skip_confirmation
            || confirm(
                &format!(
                    "Clean Xcode DerivedData ({} to be freed)?",
                    format_size(size)
                ),
                true,
            )?
        {
            match std::fs::remove_dir_all(&path) {
                Ok(()) => {
                    std::fs::create_dir_all(&path).ok();
                    print_success(&format!(
                        "Cleaned Xcode DerivedData ({})",
                        format_size(size)
                    ));
                    result.add_item(CleanedItem::directory(path, size, "Xcode DerivedData"));
                }
                Err(e) => warn!("Failed to clean Xcode DerivedData: {e}"),
            }
        }
    }

    Ok(result)
}

/// Removes unavailable (deleted-device-family) iOS/watchOS/tvOS Simulator
/// runtimes' caches via `xcrun simctl delete unavailable`, then measures the
/// Simulator caches directory before/after.
#[cfg(target_os = "macos")]
fn clean_ios_simulator_caches(opts: RunOptions) -> Result<CleaningResult> {
    let mut result = CleaningResult::new();
    let Some(home) = platform::home_dir() else {
        return Ok(result);
    };
    let caches_path = home.join("Library/Developer/CoreSimulator/Caches");

    if !Command::new("which")
        .arg("xcrun")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        debug!("xcrun not found — skipping iOS Simulator cache cleaner");
        return Ok(result);
    }

    if caches_path.exists() {
        if opts.dry_run {
            let before = get_size(&caches_path.to_string_lossy()).unwrap_or(0);
            if before > 0 {
                result.add_item(CleanedItem::directory(
                    caches_path,
                    before,
                    "iOS Simulator caches",
                ));
            }
            return Ok(result);
        }

        if opts.skip_confirmation
            || confirm(
                "Delete unavailable Simulator devices and clean their caches?",
                true,
            )?
        {
            measure_around(
                &mut result,
                &caches_path,
                "iOS Simulator caches",
                opts,
                || {
                    let output = Command::new("xcrun")
                        .args(["simctl", "delete", "unavailable"])
                        .output()?;
                    if !output.status.success() {
                        warn!(
                            "xcrun simctl delete unavailable failed: {}",
                            String::from_utf8_lossy(&output.stderr)
                        );
                    }
                    Ok(output.status.success())
                },
            )?;
        }
    }

    Ok(result)
}

#[cfg(target_os = "macos")]
fn clean_system_logs(opts: RunOptions) -> Result<CleaningResult> {
    let mut result = CleaningResult::new();
    let log_path = Path::new("/private/var/log");

    if log_path.exists() {
        let mut size_to_clean = 0u64;
        if let Ok(entries) = std::fs::read_dir(log_path) {
            for entry in entries.flatten() {
                let file_path = entry.path();
                let filename = file_path.file_name().unwrap_or_default().to_string_lossy();
                if file_path.is_file() && (filename.ends_with(".gz") || filename.contains(".0")) {
                    size_to_clean += std::fs::metadata(&file_path).map(|m| m.len()).unwrap_or(0);
                }
            }
        }

        if size_to_clean > 0 {
            if opts.dry_run {
                result.add_item(CleanedItem::directory(
                    log_path.to_path_buf(),
                    size_to_clean,
                    "rotated system logs",
                ));
            } else if opts.skip_confirmation
                || confirm(
                    &format!(
                        "Clean old rotated logs in /private/var/log ({} to be freed)?",
                        format_size(size_to_clean)
                    ),
                    true,
                )?
            {
                let output = execute_with_sudo(
                    "find",
                    &["/private/var/log", "-type", "f", "-name", "*.gz", "-delete"],
                )?;
                if output.status.success() {
                    print_success(&format!(
                        "Cleaned old rotated logs in /private/var/log ({})",
                        format_size(size_to_clean)
                    ));
                    result.add_item(CleanedItem::directory(
                        log_path.to_path_buf(),
                        size_to_clean,
                        "rotated system logs",
                    ));
                }
            }
        }
    }

    Ok(result)
}

#[cfg(target_os = "macos")]
fn clean_crash_reports(opts: RunOptions) -> Result<CleaningResult> {
    let mut result = CleaningResult::new();
    let paths = [
        "/Library/Logs/DiagnosticReports".to_string(),
        platform::home_dir()
            .map(|h| {
                h.join("Library/Logs/DiagnosticReports")
                    .to_string_lossy()
                    .to_string()
            })
            .unwrap_or_default(),
    ];

    for crash_path in paths {
        if crash_path.is_empty() {
            continue;
        }
        measure_and_remove(
            &mut result,
            Path::new(&crash_path),
            &crash_path,
            opts,
            || execute_with_sudo("sh", &["-c", &format!("rm -rf '{crash_path}'/*")]),
        )?;
    }

    Ok(result)
}

// ── Windows ──────────────────────────────────────────────────────────────────

#[cfg(target_os = "windows")]
fn windows_cleaners() -> Vec<CleanerInfo> {
    vec![
        CleanerInfo {
            name: "Windows Update Cache",
            description: "Clean the Windows Update download cache (requires Administrator)",
            function: clean_windows_update_cache,
            requires_root: true,
        },
        CleanerInfo {
            name: "System Temp Files",
            description: "Clean C:\\Windows\\Temp (requires Administrator)",
            function: clean_windows_system_temp,
            requires_root: true,
        },
        CleanerInfo {
            name: "Recycle Bin",
            description: "Empty the Recycle Bin for all drives (requires Administrator)",
            function: clean_recycle_bin,
            requires_root: false,
        },
    ]
}

#[cfg(target_os = "windows")]
fn windows_dir() -> Option<std::path::PathBuf> {
    std::env::var_os("SystemRoot").map(std::path::PathBuf::from)
}

#[cfg(target_os = "windows")]
fn clean_windows_update_cache(opts: RunOptions) -> Result<CleaningResult> {
    let mut result = CleaningResult::new();
    let Some(win_dir) = windows_dir() else {
        return Ok(result);
    };
    let path = win_dir.join("SoftwareDistribution").join("Download");

    if path.exists() {
        let size = get_size(&path.to_string_lossy())?;
        if size == 0 {
            return Ok(result);
        }

        if opts.dry_run {
            result.add_item(CleanedItem::directory(path, size, "Windows Update cache"));
            return Ok(result);
        }

        if !opts.skip_confirmation
            && !confirm(
                &format!(
                    "Clean Windows Update cache ({} to be freed)?",
                    format_size(size)
                ),
                true,
            )?
        {
            return Ok(result);
        }

        match std::fs::remove_dir_all(&path) {
            Ok(()) => {
                std::fs::create_dir_all(&path).ok();
                print_success(&format!(
                    "Cleaned Windows Update cache ({})",
                    format_size(size)
                ));
                result.add_item(CleanedItem::directory(path, size, "Windows Update cache"));
            }
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "Failed to clean Windows Update cache: {e} (try running CleanSys as Administrator)"
                ));
            }
        }
    }

    Ok(result)
}

#[cfg(target_os = "windows")]
fn clean_windows_system_temp(opts: RunOptions) -> Result<CleaningResult> {
    let mut result = CleaningResult::new();
    let Some(win_dir) = windows_dir() else {
        return Ok(result);
    };
    let path = win_dir.join("Temp");

    if !path.exists() {
        return Ok(result);
    }

    let total_size = get_size(&path.to_string_lossy())?;
    if total_size == 0 {
        return Ok(result);
    }

    if opts.dry_run {
        result.add_item(CleanedItem::directory(
            path,
            total_size,
            "Windows system temp",
        ));
        return Ok(result);
    }

    if !opts.skip_confirmation
        && !confirm(
            &format!(
                "Clean C:\\Windows\\Temp ({} to be freed)?",
                format_size(total_size)
            ),
            true,
        )?
    {
        return Ok(result);
    }

    let mut freed = 0u64;
    let mut any_permission_denied = false;
    if let Ok(entries) = std::fs::read_dir(&path) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            let entry_size = get_size(&entry_path.to_string_lossy()).unwrap_or(0);
            let removal = if entry_path.is_dir() {
                std::fs::remove_dir_all(&entry_path)
            } else {
                std::fs::remove_file(&entry_path)
            };
            match removal {
                Ok(()) => freed += entry_size,
                Err(_) => any_permission_denied = true,
            }
        }
    }

    if freed > 0 {
        print_success(&format!(
            "Cleaned C:\\Windows\\Temp ({})",
            format_size(freed)
        ));
        result.add_item(CleanedItem::directory(path, freed, "Windows system temp"));
    } else if any_permission_denied {
        return Err(anyhow::anyhow!(
            "Could not clean C:\\Windows\\Temp — try running CleanSys as Administrator"
        ));
    }

    Ok(result)
}

/// Empty the Recycle Bin for all fixed drives via the Shell32
/// `SHEmptyRecycleBinW` API — the same call Explorer's own "Empty Recycle
/// Bin" menu item uses, so no elevated privileges are actually required for
/// the current user's own Recycle Bin (multi-user machines may still
/// restrict other users' bins, in which case the call simply reports 0
/// items freed for those).
#[cfg(target_os = "windows")]
fn clean_recycle_bin(opts: RunOptions) -> Result<CleaningResult> {
    let mut result = CleaningResult::new();

    // There is no cheap, official way to query the Recycle Bin's *current*
    // size before emptying it without walking every drive's hidden
    // `$Recycle.Bin` folder (which requires elevated access to enumerate
    // correctly per-SID); rather than mis-report a size, we surface the
    // fact that this happened via the item's label instead.
    if opts.dry_run {
        result.add_item(CleanedItem::directory(
            std::path::PathBuf::from("Recycle Bin"),
            0,
            "Recycle Bin (size unknown until emptied)",
        ));
        return Ok(result);
    }

    if !opts.skip_confirmation && !confirm("Empty the Recycle Bin?", true)? {
        return Ok(result);
    }

    match windows_shell::empty_recycle_bin() {
        Ok(()) => {
            print_success("Emptied the Recycle Bin");
            result.add_item(CleanedItem::directory(
                std::path::PathBuf::from("Recycle Bin"),
                0,
                "Recycle Bin",
            ));
        }
        Err(e) => warn!("Failed to empty Recycle Bin: {e}"),
    }

    Ok(result)
}

/// Thin wrapper around the Win32 Shell API, isolated so the rest of this
/// module never has to deal with raw FFI directly.
#[cfg(target_os = "windows")]
mod windows_shell {
    use anyhow::Result;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::Shell::{
        SHEmptyRecycleBinW, SHERB_NOCONFIRMATION, SHERB_NOPROGRESSUI, SHERB_NOSOUND,
    };

    /// Empty the Recycle Bin for every drive, silently (no confirmation
    /// dialog, no progress UI, no sound — CleanSys already asked the user).
    pub fn empty_recycle_bin() -> Result<()> {
        // SAFETY: `SHEmptyRecycleBinW(None, None, flags)` empties the
        // Recycle Bin for *all* drives when both the window handle and root
        // path are null, which is exactly the documented, supported usage.
        let result = unsafe {
            SHEmptyRecycleBinW(
                Some(HWND(std::ptr::null_mut())),
                None,
                SHERB_NOCONFIRMATION | SHERB_NOPROGRESSUI | SHERB_NOSOUND,
            )
        };
        result.map_err(|e| anyhow::anyhow!("SHEmptyRecycleBinW failed: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_cleaners_returns_platform_appropriate_list() {
        let cleaners = get_cleaners();
        // Every currently-supported platform (linux/macos/windows) should
        // return a non-empty list; unknown platforms fall back to empty.
        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        assert!(!cleaners.is_empty());

        for cleaner in &cleaners {
            assert!(!cleaner.name.is_empty());
            assert!(!cleaner.description.is_empty());
        }
    }

    #[test]
    fn list_cleaners_formats_name_and_description() {
        for entry in list_cleaners() {
            assert!(entry.contains(':'));
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_cleaners_includes_expected_names() {
        let names: Vec<&str> = linux_cleaners().iter().map(|c| c.name).collect();
        assert!(names.contains(&"Package Manager Caches"));
        assert!(names.contains(&"System Logs"));
        assert!(names.contains(&"Old Kernels"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_cleaners_includes_expected_names() {
        let names: Vec<&str> = macos_cleaners().iter().map(|c| c.name).collect();
        assert!(names.contains(&"Homebrew Cache"));
        assert!(names.contains(&"Crash Reports"));
        assert!(names.contains(&"Xcode Derived Data"));
        assert!(names.contains(&"iOS Simulator Caches"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn homebrew_cache_does_not_require_root() {
        // Regression guard: Homebrew explicitly refuses to run as root, so
        // this cleaner must never be lumped into the "needs sudo" bucket
        // the way Linux system cleaners are.
        let homebrew = macos_cleaners()
            .into_iter()
            .find(|c| c.name == "Homebrew Cache")
            .expect("Homebrew Cache cleaner should exist on macOS");
        assert!(!homebrew.requires_root);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn xcode_cleaners_do_not_require_root() {
        let cleaners = macos_cleaners();
        for name in ["Xcode Derived Data", "iOS Simulator Caches"] {
            let c = cleaners.iter().find(|c| c.name == name).unwrap();
            assert!(!c.requires_root, "{name} should not require root");
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_system_cleaners_all_require_root() {
        assert!(linux_cleaners().iter().all(|c| c.requires_root));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn windows_cleaners_includes_expected_names() {
        let names: Vec<&str> = windows_cleaners().iter().map(|c| c.name).collect();
        assert!(names.contains(&"Windows Update Cache"));
        assert!(names.contains(&"Recycle Bin"));
    }
}
