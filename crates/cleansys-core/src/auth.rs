//! Sudo/root authentication helpers shared by the TUI password prompt and the
//! GUI authentication dialog.
//!
//! This module contains no UI framework code — front-ends own their own
//! widgets/state and call into these functions to perform the actual
//! authentication.

use anyhow::Result;
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::{Mutex, OnceLock};

/// Process-wide cache of the last successfully-validated sudo password.
///
/// GUI front-ends have no controlling TTY, so `sudo`'s own ticket cache
/// (normally keyed per-TTY/session) is not a reliable way to reuse
/// credentials across the separate `sudo` child processes spawned for each
/// cleaner — `sudo -n <cmd>` can fail even immediately after a successful
/// [`authenticate_sudo`] call, depending on the platform's sudoers
/// configuration. Caching the raw password here lets
/// [`crate::utils::execute_with_sudo`] re-authenticate every single
/// privileged command the same reliable way `authenticate_sudo` itself
/// does (piping it to `sudo -S`), instead of depending on ticket reuse.
fn password_cell() -> &'static Mutex<Option<String>> {
    static CELL: OnceLock<Mutex<Option<String>>> = OnceLock::new();
    CELL.get_or_init(|| Mutex::new(None))
}

/// Cache a validated sudo password for later privileged commands to reuse.
pub fn cache_sudo_password(password: String) {
    if let Ok(mut guard) = password_cell().lock() {
        *guard = Some(password);
    }
}

/// Clear any cached sudo password (call once a run finishes, the dialog is
/// cancelled, or the app is closing).
pub fn clear_cached_sudo_password() {
    if let Ok(mut guard) = password_cell().lock() {
        guard.take();
    }
}

/// Return a clone of the cached sudo password, if one is set.
pub fn cached_sudo_password() -> Option<String> {
    password_cell().lock().ok().and_then(|guard| guard.clone())
}

/// Attempt to authenticate as root using `sudo -S -v` with the given password
/// piped over stdin. Returns `Ok(true)` if authentication succeeded, `Ok(false)`
/// if the password was rejected, and `Err` if `sudo` could not be invoked at all.
///
/// On success, the password is also cached (see [`cache_sudo_password`]) so
/// that subsequent privileged commands run via
/// [`crate::utils::execute_with_sudo`] can re-authenticate reliably without
/// depending on `sudo`'s own (TTY/session-keyed) credential cache.
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
    let success = status.success();
    if success {
        cache_sudo_password(password.to_string());
    }
    Ok(success)
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

    #[test]
    fn cached_sudo_password_round_trips_and_clears() {
        // Single test exercising cache/get/clear in sequence against the
        // process-global cell, since running separate tests concurrently
        // against shared global state would race.
        clear_cached_sudo_password();
        assert_eq!(cached_sudo_password(), None);

        cache_sudo_password("hunter2".to_string());
        assert_eq!(cached_sudo_password().as_deref(), Some("hunter2"));

        // Caching again overwrites, it doesn't accumulate.
        cache_sudo_password("new-password".to_string());
        assert_eq!(cached_sudo_password().as_deref(), Some("new-password"));

        clear_cached_sudo_password();
        assert_eq!(cached_sudo_password(), None);

        // Clearing an already-empty cache is a harmless no-op.
        clear_cached_sudo_password();
        assert_eq!(cached_sudo_password(), None);
    }
}
