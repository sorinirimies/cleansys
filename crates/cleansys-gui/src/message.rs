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
    /// Switch the active category tab.
    SwitchCategoryTab(usize),
    /// User picked a different theme from the theme selector.
    ThemeChanged(usize),
    /// Kick off cleaning of all currently-selected items.
    RunSelected,
    /// The password field in the sudo authentication dialog changed.
    PasswordChanged(String),
    /// User pressed Enter / clicked "Authenticate" on the password dialog.
    PasswordSubmit,
    /// User cancelled the sudo authentication dialog.
    PasswordCancel,
    /// A background sudo authentication attempt finished.
    AuthenticationResult(bool),
    /// A single cleaner (category_index, item_index) finished running.
    /// `Ok(result)` with structured per-item detail on success, `Err(message)` on failure.
    OperationFinished(usize, usize, Result<cleansys_core::CleaningResult, String>),
    /// Clear the operation log and reset counters for a fresh run.
    ClearLog,
}
