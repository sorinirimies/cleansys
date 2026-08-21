//! Small platform-integration helpers for the GUI: desktop notifications and
//! (Windows-only) relaunching the process elevated.

/// Send a desktop notification announcing that a cleaning run finished.
/// Best-effort: failures are logged but never surfaced to the UI (not every
/// desktop environment/session has a notification daemon available, and
/// that's fine — the in-app activity log always has the same information).
pub fn notify_completion(summary: &str) {
    let result = notify_rust::Notification::new()
        .summary("CleanSys")
        .body(summary)
        .appname("CleanSys")
        .show();

    if let Err(e) = result {
        log::debug!("desktop notification failed (non-fatal): {e}");
    }
}

/// Relaunch the current executable with an elevation request.
///
/// On Windows this uses `ShellExecuteW` with the `"runas"` verb, which
/// triggers the standard UAC consent prompt. On other platforms there is no
/// equivalent single-click relaunch (Unix elevation is handled by the
/// sudo-password dialog instead), so this is a no-op.
pub fn relaunch_as_admin() {
    #[cfg(target_os = "windows")]
    {
        if let Err(e) = windows_impl::relaunch_elevated() {
            log::warn!("Failed to relaunch as Administrator: {e}");
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        log::debug!("relaunch_as_admin() is a no-op on this platform");
    }
}

#[cfg(target_os = "windows")]
mod windows_impl {
    use anyhow::{Context, Result};
    use std::os::windows::ffi::OsStrExt;
    use windows::core::PCWSTR;
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    /// Null-terminated UTF-16 encoding of an OS string, for use with the
    /// `*W` (wide-character) Win32 APIs.
    fn to_wide(s: &std::ffi::OsStr) -> Vec<u16> {
        s.encode_wide().chain(std::iter::once(0)).collect()
    }

    pub fn relaunch_elevated() -> Result<()> {
        let exe = std::env::current_exe().context("failed to determine current executable")?;
        let exe_wide = to_wide(exe.as_os_str());
        let verb_wide = to_wide(std::ffi::OsStr::new("runas"));

        // SAFETY: all string pointers reference `Vec<u16>` buffers that are
        // kept alive for the duration of this call, and are properly
        // null-terminated as `ShellExecuteW` requires.
        let result = unsafe {
            ShellExecuteW(
                None,
                PCWSTR(verb_wide.as_ptr()),
                PCWSTR(exe_wide.as_ptr()),
                PCWSTR::null(),
                PCWSTR::null(),
                SW_SHOWNORMAL,
            )
        };

        // ShellExecuteW returns a value > 32 on success (per Win32 docs);
        // anything else (including the user declining the UAC prompt)
        // indicates failure.
        if (result.0 as isize) <= 32 {
            anyhow::bail!("ShellExecuteW returned error code {}", result.0 as isize);
        }

        // Exit this (unelevated) instance now that the elevated copy is starting.
        std::process::exit(0);
    }
}
