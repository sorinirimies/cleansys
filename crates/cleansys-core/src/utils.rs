use anyhow::{Context, Result};
use colored::*;
use std::io::{self, Write};
use std::process::Command;
#[cfg(unix)]
use users::get_effective_uid;

/// Check if the program is running with root privileges
#[cfg(unix)]
pub fn check_root() -> bool {
    get_effective_uid() == 0
}

#[cfg(not(unix))]
pub fn check_root() -> bool {
    false
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

#[cfg(not(unix))]
pub fn elevate_if_needed() -> Result<bool> {
    print_warning("System cleaners are only available on Unix-like systems.");
    Ok(false)
}

/// Execute a command with sudo if not already root
/// This function handles terminal raw mode properly for TUI applications
/// It assumes sudo credentials are already cached (via password dialog or sudo -v)
#[cfg(unix)]
pub fn execute_with_sudo(command: &str, args: &[&str]) -> Result<std::process::Output> {
    use std::process::Stdio;

    if check_root() {
        // Already root, execute directly
        Command::new(command)
            .args(args)
            .output()
            .context(format!("Failed to execute command: {}", command))
    } else {
        // Use sudo with non-interactive mode and cached credentials
        // The -n flag prevents sudo from prompting for a password
        let mut sudo_args = vec!["-n", command];
        sudo_args.extend_from_slice(args);

        Command::new("sudo")
            .args(sudo_args)
            .stdin(Stdio::null())
            .output()
            .context(format!("Failed to execute command with sudo: {}", command))
    }
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
    print!("{} {} ", prompt, yes_no);
    io::stdout().flush()?;

    let mut response = String::new();
    io::stdin().read_line(&mut response)?;

    match response.trim().to_lowercase().as_str() {
        "y" | "yes" => Ok(true),
        "n" | "no" => Ok(false),
        "" => Ok(default),
        _ => {
            print_warning("Invalid response. Please enter 'y' or 'n'.");
            confirm(prompt, default)
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

/// Get the size of a directory or file in bytes
pub fn get_size(path: &str) -> Result<u64> {
    let output = std::process::Command::new("du")
        .args(["-sb", path])
        .output()?;

    if !output.status.success() {
        return Ok(0);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parts: Vec<&str> = stdout.split_whitespace().collect();
    if parts.is_empty() {
        return Ok(0);
    }

    match parts[0].parse::<u64>() {
        Ok(size) => Ok(size),
        Err(_) => Ok(0),
    }
}
