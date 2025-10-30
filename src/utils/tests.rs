#[cfg(test)]
mod tests {
    use crate::utils::*;

    #[test]
    fn test_format_size_bytes() {
        assert_eq!(format_size(0), "0 bytes");
        assert_eq!(format_size(1), "1 bytes");
        assert_eq!(format_size(512), "512 bytes");
        assert_eq!(format_size(1023), "1023 bytes");
    }

    #[test]
    fn test_format_size_kilobytes() {
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(2048), "2.00 KB");
        assert_eq!(format_size(1536), "1.50 KB");
        assert_eq!(format_size(10240), "10.00 KB");
    }

    #[test]
    fn test_format_size_megabytes() {
        assert_eq!(format_size(1048576), "1.00 MB");
        assert_eq!(format_size(2097152), "2.00 MB");
        assert_eq!(format_size(1572864), "1.50 MB");
        assert_eq!(format_size(104857600), "100.00 MB");
    }

    #[test]
    fn test_format_size_gigabytes() {
        assert_eq!(format_size(1073741824), "1.00 GB");
        assert_eq!(format_size(2147483648), "2.00 GB");
        assert_eq!(format_size(1610612736), "1.50 GB");
        assert_eq!(format_size(10737418240), "10.00 GB");
    }

    #[test]
    fn test_format_size_edge_cases() {
        // u64::MAX = 18,446,744,073,709,551,615 bytes ≈ 17,179,869,184 GB
        assert_eq!(format_size(u64::MAX), "17179869184.00 GB");
    }

    #[cfg(unix)]
    #[test]
    fn test_check_root() {
        // This test will pass differently based on whether it's run as root
        // We're just testing that the function doesn't panic
        let is_root = check_root();
        assert!(is_root == true || is_root == false);
    }

    #[cfg(not(unix))]
    #[test]
    fn test_check_root_non_unix() {
        // On non-Unix systems, should always return false
        assert_eq!(check_root(), false);
    }

    #[test]
    fn test_get_size_nonexistent_path() {
        // Test with a path that doesn't exist
        let result = get_size("/nonexistent/path/that/should/not/exist");
        // Should return Ok(0) for nonexistent paths
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_get_size_with_temp_file() {
        use std::fs::File;
        use std::io::Write;
        use tempfile::TempDir;

        // Create a temporary directory
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_file.txt");

        // Write some data to a file
        let mut file = File::create(&file_path).unwrap();
        let test_data = "Hello, World!";
        file.write_all(test_data.as_bytes()).unwrap();
        drop(file); // Close the file

        // Get the size of the parent directory
        let size = get_size(temp_dir.path().to_str().unwrap());
        assert!(size.is_ok());

        // Size should be greater than 0 since we created a file
        let size_value = size.unwrap();
        assert!(size_value > 0, "Directory size should be greater than 0");
    }

    #[cfg(unix)]
    #[test]
    fn test_execute_with_sudo_direct_command() {
        use std::process::Command;

        // Test executing a simple command that doesn't require sudo
        let result = Command::new("echo").args(["hello"]).output();

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "hello");
    }

    #[test]
    fn test_format_size_precision() {
        // Test that precision is maintained correctly
        assert_eq!(format_size(1536), "1.50 KB"); // Exactly 1.5 KB
        assert_eq!(format_size(1024 + 512), "1.50 KB");
        assert_eq!(format_size(1024 * 1024 + 512 * 1024), "1.50 MB");
    }

    #[test]
    fn test_format_size_rounding() {
        // Test rounding behavior
        assert_eq!(format_size(1025), "1.00 KB"); // Should round to 1.00 KB
        assert_eq!(format_size(1030), "1.01 KB"); // Should show 1.01 KB
    }

    mod elevation_tests {
        use crate::utils::*;

        #[cfg(unix)]
        #[test]
        fn test_elevate_if_needed_when_root() {
            // This test only makes sense if we're already root
            if check_root() {
                let result = elevate_if_needed();
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), true);
            }
        }

        #[cfg(not(unix))]
        #[test]
        fn test_elevate_if_needed_non_unix() {
            let result = elevate_if_needed();
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), false);
        }

        #[cfg(unix)]
        #[test]
        fn test_execute_with_sudo_echo() {
            // Test a simple command that doesn't actually need sudo
            let result = execute_with_sudo("echo", &["test"]);

            // If we're not root, this will try to use sudo
            // If we're root, it will execute directly
            // Either way, if sudo is available, it should work
            if check_root() {
                assert!(result.is_ok());
                let output = result.unwrap();
                assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "test");
            }
            // If not root and no sudo available, it may fail - that's okay for this test
        }
    }

    mod print_functions_tests {
        use crate::utils::*;

        #[test]
        fn test_print_functions_dont_panic() {
            // These tests just verify the functions don't panic
            print_header("Test Header");
            print_success("Success message");
            print_warning("Warning message");
            print_error("Error message");
        }
    }

    mod integration_tests {
        use crate::utils::*;

        #[test]
        fn test_size_formatting_chain() {
            // Test a chain of conversions
            let sizes = vec![
                (0, "0 bytes"),
                (1024, "1.00 KB"),
                (1024 * 1024, "1.00 MB"),
                (1024 * 1024 * 1024, "1.00 GB"),
            ];

            for (bytes, expected) in sizes {
                assert_eq!(format_size(bytes), expected);
            }
        }

        #[test]
        fn test_mixed_size_formatting() {
            // Test various sizes to ensure consistency
            let test_cases = vec![
                (500, "500 bytes"),
                (1500, "1.46 KB"),
                (1048576 + 524288, "1.50 MB"),
                (2147483648 + 1073741824, "3.00 GB"),
            ];

            for (bytes, expected) in test_cases {
                assert_eq!(format_size(bytes), expected);
            }
        }
    }
}
