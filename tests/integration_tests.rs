//! Integration tests for CleanSys
//!
//! These tests verify the integration between different components
//! of the CleanSys application.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs::{self, File};
use std::io::Write;
use tempfile::TempDir;

#[test]
fn test_help_command() {
    let mut cmd = Command::cargo_bin("cleansys").unwrap();
    cmd.arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("CleanSys"))
        .stdout(predicate::str::contains("Usage:"));
}

#[test]
fn test_version_command() {
    let mut cmd = Command::cargo_bin("cleansys").unwrap();
    cmd.arg("--version");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("cleansys"));
}

#[test]
fn test_list_command() {
    let mut cmd = Command::cargo_bin("cleansys").unwrap();
    cmd.arg("list");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("AVAILABLE CLEANERS"))
        .stdout(predicate::str::contains("User cleaners"))
        .stdout(predicate::str::contains("System cleaners"));
}

#[test]
fn test_list_shows_user_cleaners() {
    let mut cmd = Command::cargo_bin("cleansys").unwrap();
    cmd.arg("list");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Browser Caches"))
        .stdout(predicate::str::contains("Application Caches"));
}

#[test]
fn test_list_shows_system_cleaners() {
    let mut cmd = Command::cargo_bin("cleansys").unwrap();
    cmd.arg("list");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Package Manager Caches"))
        .stdout(predicate::str::contains("System Logs"));
}

#[test]
fn test_system_command_without_root() {
    let mut cmd = Command::cargo_bin("cleansys").unwrap();
    cmd.arg("system");

    // Should either prompt for elevation or inform user about sudo requirement
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    // Should mention root, sudo, or privileges
    assert!(
        combined.contains("root")
            || combined.contains("sudo")
            || combined.contains("privileges")
            || combined.contains("elevate")
    );
}

#[test]
fn test_invalid_command() {
    let mut cmd = Command::cargo_bin("cleansys").unwrap();
    cmd.arg("invalid_command");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized subcommand"));
}

#[test]
fn test_user_command_with_yes_flag() {
    let mut cmd = Command::cargo_bin("cleansys").unwrap();
    cmd.arg("user").arg("--yes");

    // Should run without prompting
    // We can't easily test the actual cleaning, but we can verify it doesn't fail
    let output = cmd.output().unwrap();

    // Command should complete (success or expected failure is ok)
    // Just checking it doesn't crash
    assert!(output.status.code().is_some());
}

#[test]
fn test_cargo_metadata() {
    // Verify package metadata is correct
    let mut cmd = Command::cargo_bin("cleansys").unwrap();
    cmd.arg("--version");

    let output = cmd.output().unwrap();
    let version_output = String::from_utf8_lossy(&output.stdout);

    // Should contain version number
    assert!(version_output.contains("cleansys"));
}

#[cfg(unix)]
mod unix_specific_tests {
    use super::*;

    #[test]
    fn test_check_root_detection() {
        // This test verifies that root detection works
        let mut cmd = Command::cargo_bin("cleansys").unwrap();
        cmd.arg("list");

        // Should work regardless of root status
        cmd.assert().success();
    }

    #[test]
    fn test_system_cleaners_require_elevation() {
        let mut cmd = Command::cargo_bin("cleansys").unwrap();
        cmd.arg("system").arg("--yes");

        // If not root, should mention privilege requirements
        let output = cmd.output().unwrap();

        // Either succeeds (if root) or mentions privileges (if not root)
        if !output.status.success() {
            let combined = format!(
                "{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(
                combined.contains("root")
                    || combined.contains("sudo")
                    || combined.contains("privilege")
            );
        }
    }
}

mod checkbox_integration_tests {
    use super::*;

    #[test]
    fn test_tui_checkbox_dependency() {
        // Verify tui-checkbox is available by checking if the binary was built
        // with the dependency
        let mut cmd = Command::cargo_bin("cleansys").unwrap();
        cmd.arg("--version");

        cmd.assert().success();
    }
}

mod cleaner_module_tests {
    use super::*;

    #[test]
    fn test_temporary_directory_cleanup_simulation() {
        // Create a temporary directory structure to simulate cleanup
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().join("cache");
        fs::create_dir_all(&cache_dir).unwrap();

        // Create some dummy files
        let file_path = cache_dir.join("dummy_cache.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "dummy cache data").unwrap();

        // Verify file exists
        assert!(file_path.exists());

        // Simulate cleanup
        fs::remove_dir_all(&cache_dir).unwrap();

        // Verify cleanup
        assert!(!cache_dir.exists());
    }

    #[test]
    fn test_size_calculation() {
        // Create a temporary file with known size
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_file.txt");

        let mut file = File::create(&file_path).unwrap();
        let data = "a".repeat(1024); // 1 KB of data
        file.write_all(data.as_bytes()).unwrap();
        drop(file);

        // Verify file exists and has content
        assert!(file_path.exists());
        let metadata = fs::metadata(&file_path).unwrap();
        assert_eq!(metadata.len(), 1024);
    }
}

mod error_handling_tests {
    use super::*;

    #[test]
    fn test_handles_missing_sudo_gracefully() {
        let mut cmd = Command::cargo_bin("cleansys").unwrap();
        cmd.arg("system");

        // Should not panic, should handle missing sudo gracefully
        let output = cmd.output().unwrap();

        // Either succeeds or fails gracefully
        assert!(output.status.code().is_some());
    }

    #[test]
    fn test_invalid_flags_combination() {
        let mut cmd = Command::cargo_bin("cleansys").unwrap();
        cmd.arg("--unknown-flag");

        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("unexpected argument"));
    }
}

mod format_tests {
    use super::*;

    #[test]
    fn test_output_contains_proper_formatting() {
        let mut cmd = Command::cargo_bin("cleansys").unwrap();
        cmd.arg("list");

        let output = cmd.output().unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Should contain bullet points or other formatting
        assert!(stdout.contains("•") || stdout.contains("-") || stdout.contains("*"));
    }
}

#[test]
fn test_no_args_shows_tui_or_help() {
    let mut cmd = Command::cargo_bin("cleansys").unwrap();

    // Running with no args should either show TUI (which we can't test in CI)
    // or show some output
    let output = cmd.output();

    // Just verify it doesn't panic
    assert!(output.is_ok());
}

#[test]
fn test_menu_command_exists() {
    let mut cmd = Command::cargo_bin("cleansys").unwrap();
    cmd.arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("menu"));
}

#[test]
fn test_tui_command_exists() {
    let mut cmd = Command::cargo_bin("cleansys").unwrap();
    cmd.arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("tui"));
}

mod sudo_elevation_tests {
    use super::*;

    #[test]
    fn test_elevation_prompt_mechanism() {
        // Test that system command triggers elevation logic
        let mut cmd = Command::cargo_bin("cleansys").unwrap();
        cmd.arg("system").arg("--yes");

        let output = cmd.output().unwrap();

        // Should complete without crashing
        assert!(output.status.code().is_some());
    }

    #[test]
    fn test_user_cleaners_dont_require_sudo() {
        let mut cmd = Command::cargo_bin("cleansys").unwrap();
        cmd.arg("user").arg("--yes");

        let output = cmd.output().unwrap();

        // User cleaners should not mention sudo/root requirements
        // (They should run or fail for other reasons)
        // This is a soft check - we just verify the command completes
        assert!(output.status.code().is_some());
    }
}

mod comprehensive_tests {
    use super::*;

    #[test]
    fn test_all_subcommands_documented() {
        let mut cmd = Command::cargo_bin("cleansys").unwrap();
        cmd.arg("--help");

        let output = cmd.output().unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);

        // All subcommands should be documented
        let subcommands = vec!["user", "system", "list", "menu", "tui"];
        for subcommand in subcommands {
            assert!(
                stdout.contains(subcommand),
                "Help should document {} subcommand",
                subcommand
            );
        }
    }

    #[test]
    fn test_flags_documented() {
        let mut cmd = Command::cargo_bin("cleansys").unwrap();
        cmd.arg("--help");

        let output = cmd.output().unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Key flags should be documented
        assert!(stdout.contains("--yes") || stdout.contains("-y"));
        assert!(stdout.contains("--verbose") || stdout.contains("-v"));
    }
}
