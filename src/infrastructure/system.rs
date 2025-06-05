use std::fs::{self, read_dir, remove_dir_all, remove_file};
use std::path::Path;
use std::os::unix::fs::MetadataExt;
use std::process::Command;
use anyhow::{Result, Context};
use log::{debug, warn};

/// Get the size of a directory or file in bytes
pub fn get_size(path: &str) -> Result<u64> {
    let output = Command::new("du")
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

/// Safely remove a directory and all its contents
pub fn remove_directory(path: &Path) -> Result<()> {
    if path.exists() && path.is_dir() {
        debug!("Removing directory: {:?}", path);
        remove_dir_all(path).context(format!("Failed to remove directory: {:?}", path))?;
        Ok(())
    } else {
        warn!("Directory does not exist: {:?}", path);
        Ok(())
    }
}

/// Safely remove a file
pub fn remove_file_path(path: &Path) -> Result<()> {
    if path.exists() && path.is_file() {
        debug!("Removing file: {:?}", path);
        remove_file(path).context(format!("Failed to remove file: {:?}", path))?;
        Ok(())
    } else {
        warn!("File does not exist: {:?}", path);
        Ok(())
    }
}

/// Create a directory and all parent directories
pub fn create_directory(path: &Path) -> Result<()> {
    debug!("Creating directory: {:?}", path);
    fs::create_dir_all(path).context(format!("Failed to create directory: {:?}", path))?;
    Ok(())
}

/// Execute a system command and return its output
pub fn execute_command(command: &str, args: &[&str]) -> Result<String> {
    debug!("Executing command: {} {}", command, args.join(" "));
    let output = Command::new(command)
        .args(args)
        .output()?;
    
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    
    if output.status.success() {
        Ok(stdout)
    } else {
        Err(anyhow::anyhow!("Command failed: {}", stderr))
    }
}

/// Check if a command is available on the system
pub fn command_exists(command: &str) -> bool {
    Command::new("which")
        .arg(command)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Get information about available disk space
pub fn get_disk_space(path: &str) -> Result<(u64, u64)> {
    let output = Command::new("df")
        .args(&["-B1", path])  // -B1 for bytes
        .output()?;
    
    if !output.status.success() {
        return Err(anyhow::anyhow!("Failed to get disk space information"));
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    
    if lines.len() < 2 {
        return Err(anyhow::anyhow!("Unexpected df output format"));
    }
    
    let parts: Vec<&str> = lines[1].split_whitespace().collect();
    if parts.len() < 4 {
        return Err(anyhow::anyhow!("Unexpected df output format"));
    }
    
    let total = parts[1].parse::<u64>().context("Failed to parse total disk space")?;
    let used = parts[2].parse::<u64>().context("Failed to parse used disk space")?;
    
    Ok((total, used))
}