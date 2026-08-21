//! Desktop notification helper for the CleanSys TUI.

/// Send a desktop notification announcing that a cleaning run finished.
/// Best-effort: failures are logged (debug level) but never surfaced to the
/// UI or block the terminal \u2014 not every environment has a notification
/// daemon (e.g. a bare SSH session), and that's fine since the in-app
/// operation log always has the same information.
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
