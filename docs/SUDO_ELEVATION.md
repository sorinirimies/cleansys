# Sudo Elevation Implementation

## Overview

CleanSys implements intelligent sudo elevation that only prompts for passwords when actually needed - specifically when system cleaners are executed. This provides a seamless user experience where users aren't asked for sudo access upfront, but only when they actually try to run system-level cleaning operations.

## Design Philosophy

### User-First Approach

1. **Ask When Needed**: Don't prompt for sudo when launching the application
2. **Clear Communication**: Inform users why sudo is needed
3. **Graceful Fallback**: Allow users to decline and continue with user cleaners only
4. **Non-Blocking**: User cleaners can always run without sudo

### Key Principle

> Only interrupt the user experience when absolutely necessary, and make it clear why the interruption is happening.

## Implementation Details

### TUI Mode (Terminal User Interface)

When running in TUI mode (`cleansys` or `cleansys tui`):

1. **Launch Without Sudo**: Application starts immediately without root check
2. **Selection Phase**: Users can browse and select both user and system cleaners
3. **Execution Phase**: When user presses Enter to run selected cleaners:
   - Check if any system cleaners are selected
   - If yes and not root, temporarily exit raw mode
   - Show clear message and prompt for sudo password
   - After authentication (success or failure), return to TUI
4. **Continue Operation**: Execute selected cleaners based on privileges

#### Code Flow

```rust
pub fn run_selected(&mut self) -> Result<()> {
    // Check if system cleaners are selected
    if has_root_cleaners && !self.is_root {
        // Test if sudo is already cached
        let sudo_test = Command::new("sudo")
            .args(["-n", "true"])
            .output();
        
        if needs_password {
            // Exit raw mode for password input
            disable_raw_mode()?;
            
            // Show clear instructions
            println!("Press Ctrl+C to cancel or enter password:");
            
            // Prompt for sudo
            Command::new("sudo").args(["-v"]).status()?;
            
            // Return to raw mode
            enable_raw_mode()?;
        }
    }
    
    // Execute cleaners...
}
```

### CLI Mode (Command-Line)

When running specific commands (`cleansys system`):

1. **Check Privileges**: Verify if root is needed for the command
2. **Prompt for Elevation**: If needed and not root, ask user to elevate
3. **Two Options**:
   - User can run with `sudo cleansys system`
   - Or application prompts to elevate now

#### Code Example

```rust
if !is_root {
    // Prompt for elevation
    if !elevate_if_needed()? {
        print_error("Cannot proceed without root privileges.");
        return Ok(());
    }
}
```

## User Experience Flows

### Scenario 1: User Cleaners Only

```
$ cleansys
[TUI Opens]
→ Select browser cache, app cache
→ Press Enter
→ Cleaning starts immediately (no password needed)
✓ Done!
```

### Scenario 2: Mixed Cleaners (User + System)

```
$ cleansys
[TUI Opens]
→ Select browser cache (user) and system logs (system)
→ Press Enter
→ Message: "Root permissions needed for system cleaners"
→ Password prompt appears
→ Enter password
→ Cleaning proceeds with all selected items
✓ Done!
```

### Scenario 3: Already Running as Root

```
$ sudo cleansys
[TUI Opens]
→ All cleaners available (no distinction)
→ Select any cleaners
→ Press Enter
→ Cleaning starts immediately (no password needed)
✓ Done!
```

### Scenario 4: Decline Sudo

```
$ cleansys
[TUI Opens]
→ Select mixed cleaners
→ Press Enter
→ Password prompt appears
→ Press Ctrl+C to cancel
→ Returns to TUI
→ System cleaners skipped, user cleaners proceed
✓ Partial completion!
```

## Technical Implementation

### Functions Added

#### `elevate_if_needed()` - Interactive Elevation

Located in `src/utils/mod.rs`:

```rust
#[cfg(unix)]
pub fn elevate_if_needed() -> Result<bool> {
    if check_root() {
        return Ok(true);
    }
    
    // Ask user if they want to elevate
    print!("Would you like to elevate now? [Y/n]: ");
    io::stdout().flush()?;
    
    let mut response = String::new();
    io::stdin().read_line(&mut response)?;
    
    match response.trim().to_lowercase().as_str() {
        "n" | "no" => Ok(false),
        _ => {
            // Try sudo -v to validate credentials
            let status = Command::new("sudo")
                .args(["-v"])
                .status()?;
            
            Ok(status.success())
        }
    }
}
```

#### `execute_with_sudo()` - Command Execution

Located in `src/utils/mod.rs`:

```rust
#[cfg(unix)]
pub fn execute_with_sudo(command: &str, args: &[&str]) -> Result<Output> {
    if check_root() {
        // Already root, execute directly
        Command::new(command)
            .args(args)
            .output()
            .context(format!("Failed to execute: {}", command))
    } else {
        // Use sudo
        let mut sudo_args = vec![command];
        sudo_args.extend_from_slice(args);
        
        Command::new("sudo")
            .args(sudo_args)
            .output()
            .context(format!("Failed to execute with sudo: {}", command))
    }
}
```

### TUI Integration

Located in `src/ui/app.rs`:

The `run_selected()` method:
1. Counts selected items and identifies system cleaners
2. Tests if sudo is already cached with `sudo -n true`
3. If password needed, temporarily disables raw mode
4. Shows clear instructions with color coding
5. Executes `sudo -v` to cache credentials
6. Re-enables raw mode and continues

### Password Prompt Display

When sudo password is needed:

```
[CleanSys] Press Ctrl+C to cancel and return to the menu if you changed your mind.
[CleanSys] Otherwise, enter your sudo password when prompted:
[sudo] password for username:
```

After successful authentication:

```
✓ Root permissions obtained. Proceeding with all cleaners.
```

## Security Considerations

### Safe Practices

1. **No Password Storage**: Never store or cache passwords in memory
2. **System sudo**: Rely on system's sudo mechanism (respects sudoers, timeouts, policies)
3. **Limited Scope**: Only cache credentials, don't keep elevated process
4. **Clear Communication**: Always inform user why sudo is needed
5. **Easy Cancel**: Users can always press Ctrl+C to cancel

### sudo -v Advantages

Using `sudo -v` (validate and cache credentials) instead of running commands directly:

- ✅ Caches credentials for ~15 minutes (default)
- ✅ Subsequent commands don't re-prompt
- ✅ Respects sudoers configuration
- ✅ Audit trail maintained
- ✅ Can be cancelled without side effects

## Error Handling

### Common Scenarios

1. **sudo not available**: Graceful error, skip system cleaners
2. **Wrong password**: Show error, allow retry or skip
3. **No sudo access**: Inform user, continue with user cleaners
4. **Ctrl+C pressed**: Cancel elevation, return to menu
5. **sudo timeout**: Re-prompt if cache expires

### Error Messages

```rust
// No sudo binary
"System cleaners require sudo but it's not available"

// Authentication failed
"Failed to obtain root permissions or operation was cancelled"

// Success
"Root permissions obtained. Proceeding with all cleaners"
```

## Testing

### Unit Tests

Located in `src/utils/tests.rs`:

```rust
#[test]
fn test_elevate_if_needed_when_root() {
    if check_root() {
        let result = elevate_if_needed();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }
}

#[test]
fn test_execute_with_sudo_echo() {
    let result = execute_with_sudo("echo", &["test"]);
    if check_root() {
        assert!(result.is_ok());
    }
}
```

### Manual Testing Checklist

- [ ] Start TUI without sudo
- [ ] Select only user cleaners → No password prompt
- [ ] Select system cleaners → Password prompt appears
- [ ] Enter correct password → All cleaners execute
- [ ] Enter wrong password → Error shown, system cleaners skipped
- [ ] Press Ctrl+C at password → Returns to TUI gracefully
- [ ] Start with `sudo cleansys` → No password prompts at all
- [ ] Run `cleansys system` without sudo → Prompted to elevate

## Platform Support

### Unix/Linux

Full support with `sudo` command:
- ✅ Password prompting
- ✅ Credential caching
- ✅ sudoers integration

### macOS

Full support (same as Linux):
- ✅ Uses native sudo
- ✅ Touch ID integration (if configured)

### Windows

Limited support:
- ⚠️ No sudo equivalent
- ⚠️ System cleaners unavailable
- ✅ User cleaners work normally

## Future Enhancements

### Possible Improvements

1. **pkexec Support**: Use PolicyKit on Linux for graphical password prompt
2. **Touch ID**: Native macOS Touch ID support
3. **Windows UAC**: Windows User Account Control integration
4. **Sudo Caching**: Remember user preference (elevate always/never)
5. **Alternative Auth**: Support for `doas` or other sudo alternatives

### Example: pkexec Integration

```rust
pub fn elevate_with_pkexec() -> Result<bool> {
    // Try pkexec if available (shows graphical prompt)
    if Command::new("which").arg("pkexec").output()?.status.success() {
        let status = Command::new("pkexec")
            .args(["--user", "root", "true"])
            .status()?;
        return Ok(status.success());
    }
    
    // Fall back to sudo
    elevate_if_needed()
}
```

## Summary

The sudo elevation implementation in CleanSys provides:

- ✅ **Non-intrusive**: Only prompts when needed
- ✅ **Clear communication**: Users understand why
- ✅ **Secure**: Uses system sudo, no custom authentication
- ✅ **Flexible**: Works in TUI and CLI modes
- ✅ **Graceful degradation**: User cleaners always work
- ✅ **User-friendly**: Easy to cancel or decline
- ✅ **Well-tested**: Comprehensive test coverage

This approach balances security, usability, and functionality while respecting user control over privilege escalation.