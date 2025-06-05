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
        .args(&["-sb", path])
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

/// Execute a command with sudo if needed
pub fn execute_with_sudo(command: &str, args: &[&str]) -> Result<std::process::Output> {
    if check_root() {
        // Already running as root, execute directly
        std::process::Command::new(command)
            .args(args)
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to execute {}: {}", command, e))
    } else {
        // Need sudo - try with password prompt
        print_warning(&format!(
            "Executing '{}' requires administrator privileges",
            command
        ));
        print_warning("You may be prompted for your password...");

        let mut cmd = std::process::Command::new("sudo");
        cmd.arg("-S"); // Read password from stdin
        cmd.arg(command);
        cmd.args(args);

        let output = cmd
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to execute sudo {}: {}", command, e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("incorrect password") || stderr.contains("authentication failure") {
                return Err(anyhow::anyhow!(
                    "Authentication failed. Please check your password."
                ));
            } else if stderr.contains("not in the sudoers file") {
                return Err(anyhow::anyhow!("User not authorized to run sudo commands."));
            } else {
                return Err(anyhow::anyhow!("Command failed: {}", stderr));
            }
        }

        Ok(output)
    }
}

/// Check if a command exists in PATH
pub fn command_exists(command: &str) -> bool {
    std::process::Command::new("which")
        .arg(command)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Execute a system cleaner function with proper sudo handling
pub fn execute_system_cleaner<F>(name: &str, cleaner_fn: F) -> Result<u64>
where
    F: FnOnce(bool) -> Result<u64>,
{
    if check_root() {
        // Already root, execute directly
        cleaner_fn(true)
    } else {
        // Inform user about sudo requirement
        print_warning(&format!(
            "System cleaner '{}' requires administrator privileges",
            name
        ));

        // Try to execute with the cleaner function
        // The cleaner function itself should handle sudo calls
        match cleaner_fn(true) {
            Ok(bytes) => {
                print_success(&format!("Successfully executed '{}' with sudo", name));
                Ok(bytes)
            }
            Err(e) => {
                let error_msg = e.to_string();
                if error_msg.contains("Authentication failed")
                    || error_msg.contains("incorrect password")
                {
                    print_error("Authentication failed. Please check your password and try again.");
                    Err(anyhow::anyhow!("Authentication failed for {}", name))
                } else if error_msg.contains("not authorized") || error_msg.contains("sudoers") {
                    print_error("User not authorized to run sudo commands.");
                    print_warning(
                        "Please add your user to the sudo group or run cleansys as root.",
                    );
                    Err(anyhow::anyhow!("Authorization failed for {}", name))
                } else {
                    print_error(&format!("System cleaner '{}' failed: {}", name, e));
                    print_warning(
                        "Tip: Try running 'sudo cleansys' for system cleaning operations.",
                    );
                    Err(anyhow::anyhow!("Execution failed for {}: {}", name, e))
                }
            }
        }
    }
}
