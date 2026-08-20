//! Sudo/root authentication helpers shared by the TUI password prompt and the
//! GUI authentication dialog.
//!
//! This module contains no UI framework code — front-ends own their own
//! widgets/state and call into these functions to perform the actual
//! authentication.

use anyhow::Result;
use std::io::Write;
use std::process::{Command, Stdio};

/// Attempt to authenticate as root using `sudo -S -v` with the given password
/// piped over stdin. Returns `Ok(true)` if authentication succeeded, `Ok(false)`
/// if the password was rejected, and `Err` if `sudo` could not be invoked at all.
pub fn authenticate_sudo(password: &str) -> Result<bool> {
    let mut child = Command::new("sudo")
        .arg("-S")
        .arg("-v")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        writeln!(stdin, "{}", password)?;
    }

    let status = child.wait()?;
    Ok(status.success())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(unix)]
    fn authenticate_sudo_with_wrong_password_does_not_panic() {
        // We can't guarantee the outcome (depends on the test machine's sudo
        // config and cached credentials), just that the call completes.
        let _ = authenticate_sudo("definitely-not-the-real-password-12345");
    }
}
