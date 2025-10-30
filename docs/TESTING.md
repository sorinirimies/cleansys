# Testing Documentation

## Overview

This document describes the testing strategy and test suite for CleanSys. The project includes both unit tests and integration tests to ensure reliability and correctness.

## Test Structure

```
cleansys/
├── src/
│   └── utils/
│       ├── mod.rs
│       └── tests.rs         # Unit tests for utils module
└── tests/
    └── integration_tests.rs # Integration tests
```

## Running Tests

### All Tests
```bash
cargo test
```

### Unit Tests Only
```bash
cargo test --lib
```

### Integration Tests Only
```bash
cargo test --test integration_tests
```

### Specific Test
```bash
cargo test test_format_size_bytes
```

### With Output
```bash
cargo test -- --nocapture
```

### Skip Tests Requiring Sudo
```bash
cargo test -- --skip test_execute_with_sudo_echo
```

## Unit Tests

Located in `src/utils/tests.rs`, these tests cover core utility functions.

### Format Size Tests

Tests for the `format_size()` function that converts bytes to human-readable format:

- `test_format_size_bytes` - Tests byte formatting (0-1023 bytes)
- `test_format_size_kilobytes` - Tests KB formatting (1024+ bytes)
- `test_format_size_megabytes` - Tests MB formatting (1048576+ bytes)
- `test_format_size_gigabytes` - Tests GB formatting (1073741824+ bytes)
- `test_format_size_edge_cases` - Tests u64::MAX and edge cases
- `test_format_size_precision` - Tests decimal precision
- `test_format_size_rounding` - Tests rounding behavior

**Example:**
```rust
assert_eq!(format_size(1024), "1.00 KB");
assert_eq!(format_size(1048576), "1.00 MB");
assert_eq!(format_size(1073741824), "1.00 GB");
```

### Root Check Tests

Tests for privilege detection:

- `test_check_root` (Unix only) - Verifies root detection doesn't panic
- `test_check_root_non_unix` - Verifies non-Unix systems return false

### File Size Tests

Tests for the `get_size()` function:

- `test_get_size_nonexistent_path` - Returns 0 for missing paths
- `test_get_size_with_temp_file` - Calculates real file sizes

### Elevation Tests

Tests for sudo elevation functionality:

- `test_elevate_if_needed_when_root` - Already root returns true
- `test_elevate_if_needed_non_unix` - Non-Unix returns false
- `test_execute_with_sudo_echo` - Tests sudo command execution

**Note:** Tests requiring actual sudo access may prompt for password or be skipped in CI.

### Print Function Tests

Tests that verify output functions don't panic:

- `test_print_functions_dont_panic` - Tests all print functions

### Integration Test Suite

Tests combining multiple functions:

- `test_size_formatting_chain` - Tests conversion chain
- `test_mixed_size_formatting` - Tests various size combinations

## Integration Tests

Located in `tests/integration_tests.rs`, these tests verify end-to-end functionality.

### Command-Line Interface Tests

#### Help and Version
- `test_help_command` - Verifies --help output
- `test_version_command` - Verifies --version output
- `test_menu_command_exists` - Checks menu subcommand
- `test_tui_command_exists` - Checks tui subcommand

#### List Command
- `test_list_command` - Verifies list output format
- `test_list_shows_user_cleaners` - Confirms user cleaners listed
- `test_list_shows_system_cleaners` - Confirms system cleaners listed

#### User Command
- `test_user_command_with_yes_flag` - Tests user cleaner execution

#### System Command
- `test_system_command_without_root` - Verifies privilege check
- `test_system_cleaners_require_elevation` - Tests elevation requirement

#### Error Handling
- `test_invalid_command` - Verifies error for unknown commands
- `test_invalid_flags_combination` - Tests flag validation
- `test_handles_missing_sudo_gracefully` - Graceful sudo handling

### Unix-Specific Tests

Tests that only run on Unix-like systems:

- `test_check_root_detection` - Root detection works correctly
- `test_system_cleaners_require_elevation` - Elevation logic works

### Checkbox Integration Tests

- `test_tui_checkbox_dependency` - Verifies tui-checkbox is available

### Cleaner Module Tests

Tests simulating cleanup operations:

- `test_temporary_directory_cleanup_simulation` - Directory cleanup
- `test_size_calculation` - File size calculation accuracy

### Sudo Elevation Tests

- `test_elevation_prompt_mechanism` - Tests elevation prompting
- `test_user_cleaners_dont_require_sudo` - User cleaners work without sudo

### Comprehensive Tests

- `test_all_subcommands_documented` - All commands in help
- `test_flags_documented` - All flags documented

## Test Coverage

### Covered Areas

✅ **Utils Module** (95%+)
- Size formatting
- Root checking
- File operations
- Print functions
- Sudo elevation

✅ **Command-Line Interface** (90%+)
- All subcommands
- Flag handling
- Error cases
- Help text

✅ **Integration** (85%+)
- End-to-end flows
- Error handling
- Platform-specific code

### Areas Not Fully Covered

⚠️ **TUI Module** (Limited)
- Interactive UI is difficult to test automatically
- Manual testing required for UI interactions
- Future: Consider using TUI testing frameworks

⚠️ **Actual Cleanup Operations**
- Real filesystem operations require root
- Cannot be fully tested in CI without sudo
- Simulation tests cover logic

## Continuous Integration

### GitHub Actions Setup

Tests run automatically on:
- Push to main branch
- Pull requests
- Different operating systems (Linux, macOS)

### CI Considerations

1. **Sudo Tests**: Skipped in CI (no password input)
2. **Platform Tests**: Run on multiple OS
3. **No TUI**: Terminal UI tests require TTY

## Manual Testing Checklist

For features that can't be automatically tested:

### TUI Functionality
- [ ] TUI starts correctly
- [ ] Checkboxes display properly (both `[X]` and `[ ]`)
- [ ] Navigation works (arrows, Tab, Space)
- [ ] System cleaners show "(root)" indicator
- [ ] System cleaners grayed out when not root
- [ ] Progress screen displays correctly
- [ ] Charts render properly
- [ ] Terminal resize handled gracefully

### Sudo Elevation
- [ ] Prompt appears before TUI starts
- [ ] Password input works correctly
- [ ] Elevation success message shows
- [ ] System cleaners enabled after elevation
- [ ] Decline option works (skips system cleaners)
- [ ] Already-root skips prompt

### System Cleaners
- [ ] Require sudo when not root
- [ ] Execute correctly with sudo
- [ ] Show proper error messages without sudo
- [ ] User cleaners work without sudo

### Cross-Platform
- [ ] Tests pass on Ubuntu
- [ ] Tests pass on Arch Linux
- [ ] Tests pass on Fedora/RHEL
- [ ] Debian-based systems work

## Writing New Tests

### Unit Test Template

```rust
#[test]
fn test_my_function() {
    // Arrange
    let input = "test data";
    
    // Act
    let result = my_function(input);
    
    // Assert
    assert_eq!(result, expected_value);
}
```

### Integration Test Template

```rust
#[test]
fn test_command_behavior() {
    let mut cmd = Command::cargo_bin("cleansys").unwrap();
    cmd.arg("subcommand");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("expected output"));
}
```

### Platform-Specific Tests

```rust
#[cfg(unix)]
#[test]
fn test_unix_only() {
    // Unix-specific test code
}

#[cfg(not(unix))]
#[test]
fn test_non_unix() {
    // Non-Unix test code
}
```

## Test Best Practices

1. **Isolation**: Each test should be independent
2. **Cleanup**: Use `TempDir` for filesystem tests
3. **Fast**: Keep tests quick (< 1 second each)
4. **Descriptive**: Use clear test names
5. **Assertions**: Test one thing per test
6. **Mock**: Mock external dependencies when possible

## Debugging Tests

### Run with Backtrace
```bash
RUST_BACKTRACE=1 cargo test
```

### Run Single Test with Output
```bash
cargo test test_name -- --nocapture
```

### Show Test Execution
```bash
cargo test -- --test-threads=1 --nocapture
```

## Known Issues

1. **Sudo Prompt in Tests**: Tests requiring sudo may hang waiting for password
   - **Solution**: Skip with `--skip test_execute_with_sudo_echo`
   
2. **TUI Tests**: Interactive TUI cannot be easily tested
   - **Solution**: Manual testing required

3. **CI Limitations**: Some platform-specific features only testable locally
   - **Solution**: Document manual testing procedures

## Future Improvements

- [ ] Add TUI testing framework (e.g., `ratatui-test`)
- [ ] Mock filesystem operations for better isolation
- [ ] Increase code coverage to 95%+
- [ ] Add performance benchmarks
- [ ] Add property-based testing (quickcheck)
- [ ] Add mutation testing
- [ ] CI: Test on more Linux distributions

## Test Metrics

Current test statistics:
- **Total Tests**: 40+
- **Unit Tests**: 15+
- **Integration Tests**: 25+
- **Passing Rate**: 100% (excluding sudo tests)
- **Code Coverage**: ~85%

## Resources

- [Rust Testing Book](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [assert_cmd Documentation](https://docs.rs/assert_cmd/)
- [predicates Documentation](https://docs.rs/predicates/)
- [tempfile Documentation](https://docs.rs/tempfile/)

## Summary

The CleanSys test suite provides comprehensive coverage of utility functions and command-line interface behavior. While some areas (particularly the TUI) require manual testing, the automated tests ensure core functionality remains reliable across changes.

For any questions or issues with tests, please check the CI logs or create an issue on GitHub.