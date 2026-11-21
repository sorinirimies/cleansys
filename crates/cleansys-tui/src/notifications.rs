//! Desktop notification helper for the CleanSys TUI.

/// Send a desktop notification announcing that a cleaning run finished.
/// Best-effort: failures are logged (debug level) but never surfaced to the
/// UI or block the terminal \u2014 not every environment has a notification
/// daemon (e.g. a bare SSH session), and that's fine since the in-app
/// operation log always has the same information.
pub fn notify_completion(summary: &str) {
    #[cfg(target_os = "macos")]
    ensure_macos_notification_app();

    let result = notify_rust::Notification::new()
        .summary("CleanSys")
        .body(summary)
        .appname("CleanSys")
        .show();

    if let Err(e) = result {
        log::debug!("desktop notification failed (non-fatal): {e}");
    }
}

/// macOS: name the sending application once. Without it `mac-notification-sys` runs an
/// `AppleScript` lookup (`get id of application "use_default"`) that crashes the whole process
/// (SIGSEGV in `NSAppleScript`), leaving the terminal in raw mode with mouse reporting on.
#[cfg(target_os = "macos")]
fn ensure_macos_notification_app() {
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        if let Err(e) = notify_rust::set_application("com.apple.Terminal") {
            log::debug!("could not set notification application (non-fatal): {e}");
        }
    });
}
