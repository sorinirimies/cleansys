use anyhow::{Result, Context};
use log::{debug, info, warn};
use std::process::Command;
use users::{get_current_uid, get_effective_uid, get_current_username};

/// Check if the program is running with root privileges
pub fn check_root() -> bool {
    get_effective_uid() == 0
}

/// Get the current user's username
pub fn get_username() -> String {
    get_current_username()
        .unwrap_or_else(|| "unknown".into())
        .into_string()
        .unwrap_or_else(|_| "unknown".to_string())
}

/// Get the current user's UID
pub fn get_uid() -> u32 {
    get_current_uid()
}

/// Elevate privileges using sudo
pub fn elevate_privileges(command: &str, args: &[&str]) -> Result<bool> {
    if check_root() {
        // Already root, no need to elevate
        debug!("Already running as root, no need to elevate");
        return Ok(true);
    }

    info!("Attempting to elevate privileges with sudo");
    
    let mut sudo_command = Command::new("sudo");
    sudo_command.arg("-v")
        .arg("-p")
        .arg("[sudo] password for %u: ");
    
    match sudo_command.status() {
        Ok(status) if status.success() => {
            debug!("Successfully authenticated with sudo");
            
            // Now run the actual command with sudo
            let mut cmd = Command::new("sudo");
            cmd.arg(command);
            cmd.args(args);
            
            debug!("Running sudo command: {:?}", cmd);
            
            match cmd.status() {
                Ok(status) if status.success() => {
                    info!("Successfully executed command with sudo");
                    Ok(true)
                },
                Ok(_) => {
                    warn!("Command executed with sudo but returned non-zero status");
                    Ok(false)
                },
                Err(e) => {
                    warn!("Failed to execute command with sudo: {}", e);
                    Err(e).context("Failed to execute command with sudo")
                }
            }
        },
        Ok(_) => {
            warn!("Failed to authenticate with sudo");
            Ok(false)
        },
        Err(e) => {
            warn!("Error during sudo authentication: {}", e);
            Err(e).context("Error during sudo authentication")
        }
    }
}

/// Check if a file or directory is owned by the current user
pub fn is_owned_by_current_user(path: &str) -> Result<bool> {
    let output = Command::new("stat")
        .args(&["-c", "%u", path])
        .output()
        .context("Failed to run stat command")?;
    
    if !output.status.success() {
        return Ok(false);
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let owner_uid = stdout.trim().parse::<u32>().unwrap_or(0);
    
    Ok(owner_uid == get_current_uid())
}

/// Check if the current user has write permission to a file or directory
pub fn has_write_permission(path: &str) -> Result<bool> {
    let output = Command::new("test")
        .args(&["-w", path])
        .status()
        .context("Failed to run test command")?;
    
    Ok(output.success())
}

/// Check if a command needs root privileges to run
pub fn command_needs_root(command: &str) -> bool {
    match command {
        "apt" | "apt-get" | "dpkg" | "pacman" | "dnf" | "yum" | "zypper" | 
        "journalctl" | "systemctl" => true,
        _ => false
    }
}