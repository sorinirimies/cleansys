use anyhow::{Context, Result};
use colored::*;
use std::io::{self, Write};
use std::process::Command;
#[cfg(unix)]
use users::get_effective_uid;

/// Check if the program is running with root privileges (Unix) or an
/// elevated/Administrator token (Windows).
#[cfg(unix)]
pub fn check_root() -> bool {
    get_effective_uid() == 0
}

/// Check if the current process token is elevated (running "as Administrator").
#[cfg(windows)]
pub fn check_root() -> bool {
    is_elevated::is_elevated()
}

#[cfg(not(any(unix, windows)))]
pub fn check_root() -> bool {
    false
}

/// Whether this platform's elevation model matches the interactive
/// sudo-password-prompt flow (Unix: `sudo -S`). Windows uses UAC/Administrator
/// tokens instead, which front-ends should present very differently (there is
/// no password to type — the user must relaunch the process elevated).
pub const fn supports_sudo_prompt() -> bool {
    cfg!(unix)
}

/// Prompt for sudo elevation if not already root
/// Returns true if elevation succeeded or already root, false otherwise
#[cfg(unix)]
pub fn elevate_if_needed() -> Result<bool> {
    if check_root() {
        return Ok(true);
    }

    print_warning("System cleaners require root privileges.");
    println!("You can either:");
    println!("  1. Run this command again with sudo");
    println!("  2. Enter your password to elevate now");
    print!("\nWould you like to elevate now? [Y/n]: ");
    io::stdout().flush()?;

    let mut response = String::new();
    io::stdin().read_line(&mut response)?;

    match response.trim().to_lowercase().as_str() {
        "n" | "no" => {
            print_warning("Skipping system cleaners. Only user cleaners will run.");
            Ok(false)
        }
        _ => {
            // Try to validate sudo access by running a simple command
            print!("Authenticating... ");
            io::stdout().flush()?;

            let status = Command::new("sudo")
                .args(["-v"])
                .status()
                .context("Failed to execute sudo")?;

            if status.success() {
                println!("{}", "✓ Authentication successful".green());
                Ok(true)
            } else {
                print_error("Authentication failed. Skipping system cleaners.");
                Ok(false)
            }
        }
    }
}

#[cfg(windows)]
pub fn elevate_if_needed() -> Result<bool> {
    if check_root() {
        return Ok(true);
    }
    print_warning(
        "Some system cleaners require Administrator privileges. Restart CleanSys as Administrator (right-click → 'Run as administrator') to use them.",
    );
    Ok(false)
}

#[cfg(not(any(unix, windows)))]
pub fn elevate_if_needed() -> Result<bool> {
    print_warning("System cleaners are only available on Unix-like systems and Windows.");
    Ok(false)
}

/// Execute a command with sudo if not already root.
///
/// If a password was previously cached via [`crate::auth::cache_sudo_password`]
/// (set by [`crate::auth::authenticate_sudo`] on success), it is piped
/// directly to `sudo -S` for this specific command — the same reliable
/// mechanism used to validate it in the first place. This avoids depending on
/// `sudo`'s own credential cache, which is normally keyed per-TTY/session and
/// is not guaranteed to be reusable across separate child processes spawned
/// by a GUI front-end that has no controlling TTY at all.
///
/// Falls back to non-interactive `sudo -n` (relying on `sudo`'s own ticket
/// cache) when no password has been cached — e.g. the TUI/CLI, which
/// pre-authenticates via a real interactive `sudo -v` prompt on an actual
/// terminal and never captures the raw password.
#[cfg(unix)]
pub fn execute_with_sudo(command: &str, args: &[&str]) -> Result<std::process::Output> {
    use std::io::Write;
    use std::process::Stdio;

    if check_root() {
        // Already root, execute directly
        return Command::new(command)
            .args(args)
            .output()
            .context(format!("Failed to execute command: {}", command));
    }

    if let Some(password) = crate::auth::cached_sudo_password() {
        let mut child = Command::new("sudo")
            .arg("-S")
            .arg(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context(format!("Failed to execute command with sudo: {}", command))?;

        if let Some(mut stdin) = child.stdin.take() {
            let _ = writeln!(stdin, "{}", password);
        }

        return child
            .wait_with_output()
            .context(format!("Failed to execute command with sudo: {}", command));
    }

    // No cached password (TUI/CLI path) — rely on sudo's own ticket cache.
    // The -n flag prevents sudo from prompting for a password.
    let mut sudo_args = vec!["-n", command];
    sudo_args.extend_from_slice(args);

    Command::new("sudo")
        .args(sudo_args)
        .stdin(Stdio::null())
        .output()
        .context(format!("Failed to execute command with sudo: {}", command))
}

#[cfg(not(unix))]
pub fn execute_with_sudo(command: &str, args: &[&str]) -> Result<std::process::Output> {
    Command::new(command)
        .args(args)
        .output()
        .context(format!("Failed to execute command: {}", command))
}

/// Print a header with a colorful banner
pub fn print_header(text: &str) {
    let width = 60;
    let padding = (width - text.len()) / 2;
    let line = "=".repeat(width);

    println!("\n{}", line.bright_blue());
    println!(
        "{}{}{}",
        " ".repeat(padding),
        text.bright_white().bold(),
        " ".repeat(padding)
    );
    println!("{}\n", line.bright_blue());
}

/// Print a success message
pub fn print_success(message: &str) {
    println!("{} {}", "✓".green().bold(), message);
}

/// Print a warning message
pub fn print_warning(message: &str) {
    println!("{} {}", "!".yellow().bold(), message);
}

/// Print an error message
pub fn print_error(message: &str) {
    eprintln!("{} {}", "✗".red().bold(), message);
}

/// Ask for user confirmation
pub fn confirm(prompt: &str, default: bool) -> Result<bool> {
    let yes_no = if default { "[Y/n]" } else { "[y/N]" };
    loop {
        print!("{} {} ", prompt, yes_no);
        io::stdout().flush()?;

        let mut response = String::new();
        io::stdin().read_line(&mut response)?;

        match response.trim().to_lowercase().as_str() {
            "y" | "yes" => return Ok(true),
            "n" | "no" => return Ok(false),
            "" => return Ok(default),
            _ => {
                print_warning("Invalid response. Please enter 'y' or 'n'.");
                // Loop and re-prompt instead of recursing — an automated
                // or malicious pipe feeding endless invalid lines must not
                // be able to grow the call stack unboundedly.
            }
        }
    }
}

/// Format bytes into human-readable sizes
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

/// Get the size of a directory or file in bytes.
///
/// Implemented as a pure-Rust recursive walk (no shell-out to `du`), so it
/// works identically on Linux, macOS, and Windows — the previous `du -sb`
/// implementation relied on a GNU-only flag and silently reported `0` on
/// BSD/macOS `du`. Symlinks are never followed and are always counted as
/// `0` bytes (matching `du`'s default, non-`-L` behaviour), avoiding both
/// infinite loops on cyclic symlinks and double-counting a target that a
/// regular file/directory entry elsewhere in the same tree may already
/// account for.
pub fn get_size(path: &str) -> Result<u64> {
    const MAX_DEPTH: u32 = 512;
    Ok(dir_size(std::path::Path::new(path), 0, MAX_DEPTH))
}

/// Directory levels (from the root) that fan out across threads. Deeper levels are
/// walked sequentially: by then there is plenty of parallelism and the overhead of
/// splitting tiny directories would only cost time.
const PARALLEL_DEPTH: u32 = 4;

/// Recursively delete a directory tree, spreading the first [`PARALLEL_DEPTH`] levels
/// over the rayon pool. Same semantics as [`std::fs::remove_dir_all`] (symlinks are
/// removed, never followed; the first error aborts), but several times faster on
/// big trees such as `target/`, `node_modules/` or Gradle caches.
pub fn remove_dir_all_parallel(path: &std::path::Path) -> std::io::Result<()> {
    // A symlink (even to a directory) is just a file as far as deletion goes.
    if std::fs::symlink_metadata(path)?.file_type().is_symlink() {
        return std::fs::remove_file(path);
    }
    remove_tree(path, 0)
}

fn remove_tree(path: &std::path::Path, depth: u32) -> std::io::Result<()> {
    let entries: Vec<std::fs::DirEntry> = std::fs::read_dir(path)?.flatten().collect();
    let one = |entry: &std::fs::DirEntry| -> std::io::Result<()> {
        if entry.file_type()?.is_dir() {
            remove_tree(&entry.path(), depth + 1)
        } else {
            std::fs::remove_file(entry.path())
        }
    };
    if depth < PARALLEL_DEPTH && entries.len() > 1 {
        use rayon::prelude::*;
        entries
            .par_iter()
            .map(one)
            .collect::<std::io::Result<()>>()?;
    } else {
        for e in &entries {
            one(e)?;
        }
    }
    std::fs::remove_dir(path)
}

fn dir_size(path: &std::path::Path, depth: u32, max_depth: u32) -> u64 {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(_) => return 0,
    };

    if metadata.file_type().is_symlink() {
        return 0;
    }
    if metadata.is_file() {
        return metadata.len();
    }
    if metadata.is_dir() {
        return dir_total(path, depth, max_depth);
    }
    0
}

/// Sum of everything below the directory `path`.
///
/// Uses `DirEntry::file_type` (no extra syscall) to skip symlinks and recurse into
/// directories, and `DirEntry::metadata` only for regular files. The first
/// [`PARALLEL_DEPTH`] levels are spread over the rayon pool, which is what makes
/// multi-gigabyte trees (Gradle/Cargo caches, `target/`, `node_modules/`) fast.
fn dir_total(path: &std::path::Path, depth: u32, max_depth: u32) -> u64 {
    // Guard against pathological/cyclic directory structures: give up on
    // descending further rather than risk a stack overflow. In practice no real
    // cache/temp/trash directory this tool targets comes close to this depth.
    if depth >= max_depth {
        log::warn!(
            "get_size: max recursion depth ({max_depth}) reached at {path:?}; size may be underestimated"
        );
        return 0;
    }
    let Ok(read) = std::fs::read_dir(path) else {
        return 0;
    };
    let entries: Vec<std::fs::DirEntry> = read.flatten().collect();

    let one = |entry: &std::fs::DirEntry| -> u64 {
        let Ok(ft) = entry.file_type() else { return 0 };
        if ft.is_symlink() {
            0
        } else if ft.is_file() {
            entry.metadata().map(|m| m.len()).unwrap_or(0)
        } else if ft.is_dir() {
            dir_total(&entry.path(), depth + 1, max_depth)
        } else {
            0
        }
    };

    if depth < PARALLEL_DEPTH && entries.len() > 1 {
        use rayon::prelude::*;
        entries
            .par_iter()
            .map(one)
            .reduce(|| 0, u64::saturating_add)
    } else {
        entries.iter().map(one).fold(0, u64::saturating_add)
    }
}
