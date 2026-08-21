//! Message type for the CleanSys Iced GUI.

/// All state-transition triggers for the CleanSys GUI.
#[derive(Debug, Clone)]
pub enum Message {
    /// Toggle whether a cleaner item (category_index, item_index) is selected.
    ToggleItem(usize, usize),
    /// Select every item in a category.
    SelectAllCategory(usize),
    /// Deselect every item in a category.
    DeselectAllCategory(usize),
    /// Select every item across every category.
    SelectAllEverywhere,
    /// Deselect every item across every category.
    DeselectAllEverywhere,
    /// Switch the active category tab.
    SwitchCategoryTab(usize),
    /// User picked a different theme from the theme selector.
    ThemeChanged(usize),
    /// User clicked "Run selected" — shows the confirmation dialog rather
    /// than cleaning immediately.
    RequestRun,
    /// User confirmed the run in the confirmation dialog.
    ConfirmRun,
    /// User cancelled the run confirmation dialog.
    CancelRunRequest,
    /// User clicked "Preview" — measure what would be cleaned without
    /// deleting anything.
    RequestPreview,
    /// User dismissed the preview results dialog.
    ClosePreview,
    /// A single cleaner's preview measurement finished.
    PreviewFinished(usize, usize, Result<cleansys_core::CleaningResult, String>),
    /// The password field in the sudo authentication dialog changed.
    PasswordChanged(String),
    /// User pressed Enter / clicked "Authenticate" on the password dialog.
    PasswordSubmit,
    /// User cancelled the sudo authentication dialog.
    PasswordCancel,
    /// A background sudo authentication attempt finished.
    AuthenticationResult(bool),
    /// User acknowledged the "needs Administrator" notice (Windows).
    AdminNoticeAcknowledged,
    /// User clicked "Relaunch as Administrator" (Windows only).
    RelaunchAsAdmin,
    /// A single cleaner (category_index, item_index) finished running.
    /// `Ok(result)` with structured per-item detail on success, `Err(message)` on failure.
    OperationFinished(usize, usize, Result<cleansys_core::CleaningResult, String>),
    /// Clear the operation log and reset counters for a fresh run.
    ClearLog,
}
