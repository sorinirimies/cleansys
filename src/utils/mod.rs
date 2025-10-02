use anyhow::Result;
use colored::*;
use std::io::{self, Write};
use users::get_effective_uid;

/// Check if the program is running with root privileges
pub fn check_root() -> bool {
    get_effective_uid() == 0
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
