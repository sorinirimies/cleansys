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
    if let Ok(mut guard) = password_cell().lock()
        && let Some(old) = guard.replace(password)
    {
        wipe(old);
    }
}

/// Best-effort overwrite of a password buffer before it is freed, so it
/// doesn't linger in freed heap memory. Volatile writes stop the compiler
/// from eliding the (otherwise dead) stores.
#[allow(
    unsafe_code,
    reason = "zeroes the password bytes in place before the String is dropped"
)]
fn wipe(mut s: String) {
    // SAFETY: all-zero bytes are valid UTF-8, so the String stays valid.
    unsafe {
        for b in s.as_bytes_mut() {
            std::ptr::write_volatile(b, 0);
        }
    }
}

/// Clear any cached sudo password (call once a run finishes, the dialog is
/// cancelled, or the app is closing).
pub fn clear_cached_sudo_password() {
    if let Ok(mut guard) = password_cell().lock()
        && let Some(old) = guard.take()
    {
        wipe(old);
    }
}

/// Return a clone of the cached sudo password, if one is set.
pub fn cached_sudo_password() -> Option<String> {
    password_cell().lock().ok().and_then(|guard| guard.clone())
}

/// Why a sudo password check did not succeed — so the UI can say what really happened
/// instead of "incorrect password" for everything.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthFailure {
    /// sudo rejected the password.
    WrongPassword,
    /// The account is locked after too many failed attempts (PAM `pam_faillock` /
    /// `pam_tally2`): even the right password fails until the lock expires.
    LockedOut,
    /// This user may not use sudo at all.
    NotAllowed,
    /// sudo refused for another reason; its own message is kept.
    Other(String),
}

impl AuthFailure {
    /// A message for the password dialog.
    pub fn message(&self) -> String {
        match self {
            Self::WrongPassword => "Incorrect password. Please try again.".to_string(),
            Self::LockedOut => "Your account is temporarily locked after too many failed sudo \
                attempts — even the right password is refused until the lock expires (usually \
                10 minutes). `faillock --user $USER --reset` clears it."
                .to_string(),
            Self::NotAllowed => "This user is not allowed to use sudo.".to_string(),
            Self::Other(m) => format!("sudo refused: {m}"),
        }
    }
}

/// Classify sudo's error output after a failed `sudo -S -v`.
pub fn classify_sudo_failure(stderr: &str) -> AuthFailure {
    let s = stderr.to_ascii_lowercase();
    if s.contains("account locked")
        || s.contains("account is locked")
        || s.contains("the account is locked")
        || s.contains("maximum number of tries exceeded")
        || s.contains("faillock")
    {
        AuthFailure::LockedOut
    } else if s.contains("not in the sudoers")
        || s.contains("is not allowed to run sudo")
        || s.contains("may not run sudo")
    {
        AuthFailure::NotAllowed
    } else if s.is_empty()
        || s.contains("incorrect password")
        || s.contains("sorry, try again")
        || s.contains("authentication failure")
    {
        AuthFailure::WrongPassword
    } else {
        AuthFailure::Other(stderr.trim().lines().last().unwrap_or("").to_string())
    }
}

/// Check `password` with sudo **once** and say why it failed.
///
/// `sudo -S -v -p ''` reads exactly one line: no prompt text is mixed into the output,
/// and stdin is closed after the password, so sudo does not retry with empty input —
/// every retry would count as another failure towards a `pam_faillock` lockout. A
/// trailing newline or carriage return the user's input picked up is not part of the
/// password. On success the password is cached for later privileged commands (see
/// [`cache_sudo_password`]).
pub fn check_sudo_password(password: &str) -> Result<std::result::Result<(), AuthFailure>> {
    let password = password.trim_end_matches(['\r', '\n']);
    let mut child = Command::new("sudo")
        .args(["-S", "-v", "-p", ""])
        // sudo's messages must be parseable whatever the user's language.
        .env("LC_ALL", "C")
        .env("LANG", "C")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        writeln!(stdin, "{password}")?;
        // dropping stdin here closes it: sudo cannot read a second (empty) attempt
    }
    let out = child.wait_with_output()?;
    if out.status.success() {
        cache_sudo_password(password.to_string());
        return Ok(Ok(()));
    }
    Ok(Err(classify_sudo_failure(&String::from_utf8_lossy(
        &out.stderr,
    ))))
}

/// [`check_sudo_password`] as a yes/no: `Ok(true)` accepted, `Ok(false)` refused for any
/// reason, `Err` if `sudo` could not be run at all.
pub fn authenticate_sudo(password: &str) -> Result<bool> {
    Ok(check_sudo_password(password)?.is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Runs the real `sudo` with a wrong password: every run counts as a failed login
    /// towards a `pam_faillock` lockout, so it only runs when asked for
    /// (`cargo test -- --ignored`).
    #[test]
    #[cfg(unix)]
    #[ignore = "counts as a failed sudo login (pam_faillock); run with --ignored"]
    fn authenticate_sudo_with_wrong_password_does_not_panic() {
        // We can't guarantee the outcome (depends on the test machine's sudo
        // config and cached credentials), just that the call completes.
        let _ = authenticate_sudo("definitely-not-the-real-password-12345");
    }

    #[test]
    fn sudo_failures_are_told_apart() {
        use AuthFailure::*;
        assert_eq!(
            classify_sudo_failure("Sorry, try again.\nsudo: 1 incorrect password attempt\n"),
            WrongPassword
        );
        assert_eq!(classify_sudo_failure(""), WrongPassword);
        assert_eq!(
            classify_sudo_failure(
                "sudo: pam_faillock(sudo:auth): Consecutive login failures for user sorin account temporarily locked\nsudo: account locked\n"
            ),
            LockedOut
        );
        assert_eq!(
            classify_sudo_failure("The account is locked due to 3 failed logins.\n"),
            LockedOut
        );
        assert_eq!(
            classify_sudo_failure(
                "sorin is not in the sudoers file.  This incident will be reported.\n"
            ),
            NotAllowed
        );
        assert_eq!(
            classify_sudo_failure("sudo: a terminal is required to read the password\n"),
            Other("sudo: a terminal is required to read the password".into())
        );
        assert!(LockedOut.message().contains("faillock"));
        assert_eq!(
            WrongPassword.message(),
            "Incorrect password. Please try again."
        );
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
