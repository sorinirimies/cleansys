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
    /// The search box changed (filters cleaners across every category).
    SearchChanged(String),
    /// Clear the search box.
    ClearSearch,
    /// Measure how much every cleaner could free (read-only), in the background.
    ScanAll,
    /// One cleaner's background scan finished.
    ScanItemFinished(usize, usize, Result<cleansys_core::CleaningResult, String>),
    /// Toggle hiding cleaners that have nothing to clean.
    ToggleHideEmpty,
    /// The user grabbed the activity panel's top edge.
    LogDragStart,
    /// The cursor moved (window y) during a panel drag.
    LogDragMoved(f32),
    /// The mouse button was released.
    LogDragEnd,
    /// One animation step (spinner / progress-bar highlight).
    AnimationTick,
    /// Resize the window to a preset (⌘1–4).
    WindowPreset(crate::window::WindowPreset),
    /// Grow / shrink the window by a factor (⌘+ / ⌘−).
    WindowScale(f32),
    /// Back to the default size (⌘0).
    WindowReset,
    /// Toggle full screen (F11 / ⌃⌘F).
    ToggleFullscreen,
    /// Toggle maximised (⌘⇧M).
    ToggleMaximize,
    /// The window was resized (by the user, a shortcut or the window manager).
    WindowResized(f32, f32),
    /// Persist the window size (debounced).
    SaveWindowSize,
    /// Open the Settings / About dialog on a tab.
    OpenSettings(crate::state::SettingsTab),
    /// Close the Settings / About dialog.
    CloseSettings,
    /// Switch tab inside the dialog.
    SettingsTabSelected(crate::state::SettingsTab),
    /// Scan depth chosen in the settings.
    SetMaxDepth(usize),
    /// Toggle "ask before cleaning".
    ToggleConfirm,
    NewRootChanged(String),
    AddRoot,
    RemoveRoot(usize),
    NewExcludeChanged(String),
    AddExclude,
    RemoveExclude(usize),
    /// Open a link from the About tab in the browser.
    OpenUrl(String),
    /// Expand/collapse one cleaner's per-path details.
    ToggleExpand(usize, usize),
    /// Tick/untick one path in a cleaner's details (by absolute path).
    ToggleEntry(String),
    /// Tick/untick every path of one cleaner.
    SetEntries(usize, usize, bool),
    /// Day selector: only offer project build output idle at least this many days.
    SetMinAge(u64),
    /// Show/hide the activity log drawer.
    ToggleLog,
    /// Tick exactly the recommended cleaners (safe, user-land unless root).
    SelectRecommended,
    /// Open the automatic-cleaning (schedule) dialog.
    OpenSchedule,
    /// Close the schedule dialog.
    CloseSchedule,
    ScheduleFrequency(cleansys_core::engine::schedule::Frequency),
    ScheduleHour(u8),
    ScheduleMinute(u8),
    /// Weekday (0 = Sunday) for weekly schedules.
    ScheduleWeekday(u8),
    /// Day of month (1–28) for monthly schedules.
    ScheduleDayOfMonth(u8),
    ScheduleScope(cleansys_core::engine::schedule::Scope),
    ScheduleBackend(cleansys_core::engine::schedule::Backend),
    /// Install / update the OS job from the dialog's values.
    ScheduleApply,
    /// Remove the OS job and saved schedule.
    ScheduleRemove,
}
