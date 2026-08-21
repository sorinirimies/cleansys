//! Update logic (Elm-style) for the CleanSys Iced GUI.

use cleansys_core::{format_size, RunOptions, Status};
use iced::Task;

use crate::message::Message;
use crate::state::CleanSysGui;

/// Handle a [`Message`], mutating `state` and returning any follow-up work.
pub fn update(state: &mut CleanSysGui, message: Message) -> Task<Message> {
    match message {
        Message::ToggleItem(cat_idx, item_idx) => {
            if let Some(item) = state
                .categories
                .get_mut(cat_idx)
                .and_then(|c| c.items.get_mut(item_idx))
            {
                item.selected = !item.selected;
            }
            state.save_selections();
            Task::none()
        }

        Message::SelectAllCategory(cat_idx) => {
            if let Some(category) = state.categories.get_mut(cat_idx) {
                for item in &mut category.items {
                    item.selected = true;
                }
            }
            state.save_selections();
            Task::none()
        }

        Message::DeselectAllCategory(cat_idx) => {
            if let Some(category) = state.categories.get_mut(cat_idx) {
                for item in &mut category.items {
                    item.selected = false;
                }
            }
            state.save_selections();
            Task::none()
        }

        Message::SelectAllEverywhere => {
            for category in &mut state.categories {
                for item in &mut category.items {
                    item.selected = true;
                }
            }
            state.save_selections();
            Task::none()
        }

        Message::DeselectAllEverywhere => {
            for category in &mut state.categories {
                for item in &mut category.items {
                    item.selected = false;
                }
            }
            state.save_selections();
            Task::none()
        }

        Message::SwitchCategoryTab(idx) => {
            if idx < state.categories.len() {
                state.active_tab = idx;
            }
            Task::none()
        }

        Message::ThemeChanged(idx) => {
            if idx < cleansys_core::THEME_COUNT {
                state.theme_index = idx;
                state.save_theme();
            }
            Task::none()
        }

        Message::ClearLog => {
            state.logs.clear();
            Task::none()
        }

        Message::RequestRun => {
            if state.is_running || state.selected_indices().is_empty() {
                if state.selected_indices().is_empty() {
                    state.push_log(
                        "No items selected. Please select at least one cleaner.".to_string(),
                    );
                }
                Task::none()
            } else {
                state.confirm_run_pending = true;
                Task::none()
            }
        }

        Message::CancelRunRequest => {
            state.confirm_run_pending = false;
            Task::none()
        }

        Message::ConfirmRun => {
            state.confirm_run_pending = false;
            run_selected(state)
        }

        Message::RequestPreview => request_preview(state),

        Message::ClosePreview => {
            state.preview_open = false;
            state.preview_results.clear();
            Task::none()
        }

        Message::PreviewFinished(cat_idx, item_idx, result) => {
            if let Some(item) = state
                .categories
                .get(cat_idx)
                .and_then(|c| c.items.get(item_idx))
            {
                let name = item.name.clone();
                match result {
                    Ok(cleaning_result) => state.preview_results.push((name, cleaning_result)),
                    Err(err) => state.push_log(format!(
                        "\u{26a0}\u{fe0f}  Preview failed for {name}: {err}"
                    )),
                }
            }

            state.operations_completed += 1;
            if state.operations_completed >= state.operations_total {
                state.previewing = false;
                state.preview_open = true;
            }

            Task::none()
        }

        Message::PasswordChanged(value) => {
            state.password_input = value;
            Task::none()
        }

        Message::PasswordCancel => {
            state.needs_password = false;
            state.password_input.clear();
            state.password_error = None;
            state.pending_root_ops.clear();
            Task::none()
        }

        Message::PasswordSubmit => {
            let password = state.password_input.clone();
            Task::perform(
                async move { cleansys_core::authenticate_sudo(&password).unwrap_or(false) },
                Message::AuthenticationResult,
            )
        }

        Message::AuthenticationResult(success) => {
            if success {
                state.needs_password = false;
                state.password_input.clear();
                state.password_error = None;
                state.is_root = true; // sudo credentials are now cached
                start_pending_operations(state)
            } else {
                state.password_error = Some("Incorrect password. Please try again.".to_string());
                state.password_input.clear();
                Task::none()
            }
        }

        Message::AdminNoticeAcknowledged => {
            state.needs_admin_notice = false;
            state.pending_root_ops.clear();
            Task::none()
        }

        Message::RelaunchAsAdmin => {
            crate::platform::relaunch_as_admin();
            Task::none()
        }

        Message::OperationFinished(cat_idx, item_idx, result) => {
            let mut log_lines: Vec<String> = Vec::new();
            if let Some(item) = state
                .categories
                .get_mut(cat_idx)
                .and_then(|c| c.items.get_mut(item_idx))
            {
                match result {
                    Ok(cleaning_result) => {
                        let bytes = cleaning_result.total_bytes;
                        item.status = Some(Status::Success(format!(
                            "Cleaned {} across {} item(s)",
                            format_size(bytes),
                            cleaning_result.item_count()
                        )));
                        item.bytes_cleaned = bytes;
                        state.total_bytes_cleaned += bytes;

                        if bytes == 0 {
                            log_lines.push(format!(
                                "\u{2139}\u{fe0f}  {}: nothing to clean (already empty on this platform)",
                                item.name
                            ));
                        } else {
                            log_lines.push(format!(
                                "\u{2705} {}: freed {} across {} item(s)",
                                item.name,
                                format_size(bytes),
                                cleaning_result.item_count()
                            ));
                            for cleaned in &cleaning_result.items {
                                log_lines.push(format!(
                                    "   \u{2192} {} ({})",
                                    cleaned.path_str(),
                                    format_size(cleaned.size)
                                ));
                            }
                        }
                        item.last_result = Some(cleaning_result);
                    }
                    Err(err) => {
                        item.status = Some(Status::Error(err.clone()));
                        log_lines.push(format!("\u{274c} {}: {}", item.name, err));
                    }
                }
            }
            for line in log_lines {
                state.push_log(line);
            }

            state.operations_completed += 1;

            let still_running = state
                .categories
                .iter()
                .flat_map(|c| &c.items)
                .any(|i| matches!(i.status, Some(Status::Pending | Status::Running)));

            if !still_running {
                state.is_running = false;
                let summary = format!(
                    "Cleaning complete \u{2014} total freed: {}",
                    format_size(state.total_bytes_cleaned)
                );
                state.push_log(format!("\u{1f389} {summary}"));
                crate::platform::notify_completion(&summary);
            }

            Task::none()
        }
    }
}

/// Gather selected items, either running them immediately or — if any need
/// root and we don't have it yet — showing the appropriate elevation UI
/// first (sudo password dialog on Unix, an "needs Administrator" notice on
/// Windows, since there is no password to type there).
fn run_selected(state: &mut CleanSysGui) -> Task<Message> {
    if state.is_running {
        return Task::none();
    }

    let selected = state.selected_indices();

    if selected.is_empty() {
        state.push_log("No items selected. Please select at least one cleaner.".to_string());
        return Task::none();
    }

    if state.selection_needs_root() {
        state.pending_root_ops = selected;
        if state.supports_sudo_prompt() {
            state.needs_password = true;
        } else {
            state.needs_admin_notice = true;
        }
        return Task::none();
    }

    state.pending_root_ops = selected;
    start_pending_operations(state)
}

/// Spawn background tasks for every operation in `pending_root_ops`.
fn start_pending_operations(state: &mut CleanSysGui) -> Task<Message> {
    let ops = std::mem::take(&mut state.pending_root_ops);
    if ops.is_empty() {
        return Task::none();
    }

    state.is_running = true;
    state.total_bytes_cleaned = 0;
    state.operations_total = ops.len();
    state.operations_completed = 0;
    state.mark_selected_pending();

    let mut tasks = Vec::with_capacity(ops.len());
    for (cat_idx, item_idx) in ops {
        let Some(item) = state
            .categories
            .get_mut(cat_idx)
            .and_then(|c| c.items.get_mut(item_idx))
        else {
            continue;
        };
        item.status = Some(Status::Running);
        let function = item.function;
        let name = item.name.clone();
        state.push_log(format!("\u{1f504} Running: {}", name));

        tasks.push(Task::perform(
            async move { function(RunOptions::execute()) },
            move |result| {
                Message::OperationFinished(cat_idx, item_idx, result.map_err(|e| e.to_string()))
            },
        ));
    }

    Task::batch(tasks)
}

/// Kick off a preview (dry-run) of every selected cleaner: measures real
/// sizes/paths without deleting anything or invoking any mutating command.
fn request_preview(state: &mut CleanSysGui) -> Task<Message> {
    if state.previewing || state.is_running {
        return Task::none();
    }

    let selected = state.selected_indices();
    if selected.is_empty() {
        state.push_log("No items selected. Please select at least one cleaner.".to_string());
        return Task::none();
    }

    state.previewing = true;
    state.preview_results.clear();
    state.operations_total = selected.len();
    state.operations_completed = 0;
    state.push_log(format!(
        "\u{1f50d} Previewing {} cleaner(s)\u{2026}",
        selected.len()
    ));

    let mut tasks = Vec::with_capacity(selected.len());
    for (cat_idx, item_idx) in selected {
        let Some(item) = state
            .categories
            .get(cat_idx)
            .and_then(|c| c.items.get(item_idx))
        else {
            continue;
        };
        let function = item.function;

        tasks.push(Task::perform(
            async move { function(RunOptions::preview()) },
            move |result| {
                Message::PreviewFinished(cat_idx, item_idx, result.map_err(|e| e.to_string()))
            },
        ));
    }

    Task::batch(tasks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::CleanSysGui;

    #[test]
    fn toggle_item_flips_selection() {
        let mut state = CleanSysGui::new();
        assert!(!state.categories[0].items[0].selected);

        let _ = update(&mut state, Message::ToggleItem(0, 0));
        assert!(state.categories[0].items[0].selected);

        let _ = update(&mut state, Message::ToggleItem(0, 0));
        assert!(!state.categories[0].items[0].selected);
    }

    #[test]
    fn toggle_item_out_of_range_is_a_no_op() {
        let mut state = CleanSysGui::new();
        // Should not panic.
        let _ = update(&mut state, Message::ToggleItem(99, 99));
    }

    #[test]
    fn select_all_and_deselect_all_category() {
        let mut state = CleanSysGui::new();
        let _ = update(&mut state, Message::SelectAllCategory(0));
        assert!(state.categories[0].items.iter().all(|i| i.selected));

        let _ = update(&mut state, Message::DeselectAllCategory(0));
        assert!(state.categories[0].items.iter().all(|i| !i.selected));
    }

    #[test]
    fn select_all_everywhere_selects_every_category() {
        let mut state = CleanSysGui::new();
        let _ = update(&mut state, Message::SelectAllEverywhere);
        assert!(state
            .categories
            .iter()
            .all(|c| c.items.iter().all(|i| i.selected)));

        let _ = update(&mut state, Message::DeselectAllEverywhere);
        assert!(state
            .categories
            .iter()
            .all(|c| c.items.iter().all(|i| !i.selected)));
    }

    #[test]
    fn switch_category_tab_updates_active_tab() {
        let mut state = CleanSysGui::new();
        assert_eq!(state.active_tab, 0);
        let _ = update(&mut state, Message::SwitchCategoryTab(1));
        assert_eq!(state.active_tab, 1);
    }

    #[test]
    fn switch_category_tab_ignores_out_of_range_index() {
        let mut state = CleanSysGui::new();
        let _ = update(&mut state, Message::SwitchCategoryTab(99));
        assert_eq!(state.active_tab, 0);
    }

    #[test]
    fn theme_changed_updates_theme_index() {
        let mut state = CleanSysGui::new();
        let target = cleansys_core::theme_index_by_name("Nord");
        let _ = update(&mut state, Message::ThemeChanged(target));
        assert_eq!(state.theme_index, target);
        assert_eq!(state.current_theme_name(), "Nord");
    }

    #[test]
    fn theme_changed_ignores_out_of_range_index() {
        let mut state = CleanSysGui::new();
        let before = state.theme_index;
        let _ = update(&mut state, Message::ThemeChanged(999_999));
        assert_eq!(state.theme_index, before);
    }

    #[test]
    fn clear_log_empties_logs() {
        let mut state = CleanSysGui::new();
        state.push_log("hello");
        let _ = update(&mut state, Message::ClearLog);
        assert!(state.logs.is_empty());
    }

    #[test]
    fn request_run_with_nothing_selected_logs_message_and_does_not_confirm() {
        let mut state = CleanSysGui::new();
        let _ = update(&mut state, Message::RequestRun);
        assert!(!state.confirm_run_pending);
        assert!(!state.is_running);
        assert!(state.logs.iter().any(|l| l.contains("No items selected")));
    }

    #[test]
    fn request_run_with_selection_shows_confirmation_dialog() {
        let mut state = CleanSysGui::new();
        state.categories[0].items[0].selected = true;
        let _ = update(&mut state, Message::RequestRun);
        assert!(state.confirm_run_pending);
        assert!(!state.is_running);
    }

    #[test]
    fn cancel_run_request_hides_confirmation_dialog() {
        let mut state = CleanSysGui::new();
        state.categories[0].items[0].selected = true;
        let _ = update(&mut state, Message::RequestRun);
        let _ = update(&mut state, Message::CancelRunRequest);
        assert!(!state.confirm_run_pending);
        assert!(!state.is_running);
    }

    #[test]
    fn confirm_run_with_user_only_selection_starts_immediately() {
        let mut state = CleanSysGui::new();
        state.categories[0].items[0].selected = true;
        let _ = update(&mut state, Message::RequestRun);
        let _ = update(&mut state, Message::ConfirmRun);

        assert!(!state.confirm_run_pending);
        assert!(state.is_running);
        assert!(!state.needs_password);
        assert!(matches!(
            state.categories[0].items[0].status,
            Some(Status::Running)
        ));
    }

    fn first_root_required_item(state: &CleanSysGui) -> (usize, usize) {
        for (ci, category) in state.categories.iter().enumerate() {
            for (ii, item) in category.items.iter().enumerate() {
                if item.requires_root {
                    return (ci, ii);
                }
            }
        }
        panic!("expected at least one root-requiring cleaner on this platform");
    }

    #[test]
    fn confirm_run_with_root_item_shows_elevation_ui_when_not_root() {
        let mut state = CleanSysGui::new();
        state.is_root = false;
        let (ci, ii) = first_root_required_item(&state);
        state.categories[ci].items[ii].selected = true;

        let _ = update(&mut state, Message::ConfirmRun);

        assert!(!state.is_running);
        if state.supports_sudo_prompt() {
            assert!(state.needs_password);
        } else {
            assert!(state.needs_admin_notice);
        }
        assert_eq!(state.pending_root_ops, vec![(ci, ii)]);
    }

    #[test]
    fn confirm_run_with_root_item_runs_immediately_when_already_root() {
        let mut state = CleanSysGui::new();
        state.is_root = true;
        let (ci, ii) = first_root_required_item(&state);
        state.categories[ci].items[ii].selected = true;

        let _ = update(&mut state, Message::ConfirmRun);

        assert!(!state.needs_password);
        assert!(state.is_running);
    }

    #[test]
    fn admin_notice_acknowledged_clears_notice_and_pending_ops() {
        let mut state = CleanSysGui::new();
        state.needs_admin_notice = true;
        state.pending_root_ops = vec![(1, 0)];

        let _ = update(&mut state, Message::AdminNoticeAcknowledged);

        assert!(!state.needs_admin_notice);
        assert!(state.pending_root_ops.is_empty());
    }

    #[test]
    fn password_cancel_clears_dialog_state() {
        let mut state = CleanSysGui::new();
        state.needs_password = true;
        state.password_input = "secret".to_string();
        state.password_error = Some("nope".to_string());
        state.pending_root_ops = vec![(1, 0)];

        let _ = update(&mut state, Message::PasswordCancel);

        assert!(!state.needs_password);
        assert!(state.password_input.is_empty());
        assert!(state.password_error.is_none());
        assert!(state.pending_root_ops.is_empty());
    }

    #[test]
    fn password_changed_updates_input() {
        let mut state = CleanSysGui::new();
        let _ = update(&mut state, Message::PasswordChanged("hunter2".to_string()));
        assert_eq!(state.password_input, "hunter2");
    }

    #[test]
    fn authentication_failure_sets_error_and_clears_input() {
        let mut state = CleanSysGui::new();
        state.needs_password = true;
        state.password_input = "wrong".to_string();
        let _ = update(&mut state, Message::AuthenticationResult(false));

        assert!(state.password_error.is_some());
        assert!(state.password_input.is_empty());
        assert!(state.needs_password);
    }

    #[test]
    fn authentication_success_hides_dialog_and_marks_root() {
        let mut state = CleanSysGui::new();
        state.needs_password = true;
        state.is_root = false;
        let (ci, ii) = first_root_required_item(&state);
        state.pending_root_ops = vec![(ci, ii)];
        state.categories[ci].items[ii].selected = true;

        let _ = update(&mut state, Message::AuthenticationResult(true));

        assert!(!state.needs_password);
        assert!(state.is_root);
        assert!(state.is_running);
    }

    #[test]
    fn operation_finished_success_updates_item_and_totals() {
        let mut state = CleanSysGui::new();
        state.categories[0].items[0].status = Some(Status::Running);
        state.is_running = true;
        state.operations_total = 1;

        let mut result = cleansys_core::CleaningResult::new();
        result.add_item(cleansys_core::CleanedItem::file(
            std::path::PathBuf::from("/tmp/foo"),
            2048,
            "test",
        ));
        let _ = update(&mut state, Message::OperationFinished(0, 0, Ok(result)));

        assert_eq!(state.total_bytes_cleaned, 2048);
        assert!(matches!(
            state.categories[0].items[0].status,
            Some(Status::Success(_))
        ));
        assert!(state.categories[0].items[0].last_result.is_some());
        // No other items are pending/running, so the run should be marked done.
        assert!(!state.is_running);
        assert_eq!(state.operations_completed, 1);
    }

    #[test]
    fn operation_finished_error_updates_item_status() {
        let mut state = CleanSysGui::new();
        state.categories[0].items[0].status = Some(Status::Running);
        state.is_running = true;

        let _ = update(
            &mut state,
            Message::OperationFinished(0, 0, Err("boom".to_string())),
        );

        assert!(matches!(
            state.categories[0].items[0].status,
            Some(Status::Error(ref msg)) if msg == "boom"
        ));
        assert!(!state.is_running);
    }

    #[test]
    fn operation_finished_keeps_running_while_others_pending() {
        let mut state = CleanSysGui::new();
        state.categories[0].items[0].status = Some(Status::Running);
        state.categories[0].items[1].status = Some(Status::Pending);
        state.is_running = true;

        let mut result = cleansys_core::CleaningResult::new();
        result.add_item(cleansys_core::CleanedItem::file(
            std::path::PathBuf::from("/tmp/foo"),
            10,
            "test",
        ));
        let _ = update(&mut state, Message::OperationFinished(0, 0, Ok(result)));

        // Another item is still pending, so the overall run isn't finished yet.
        assert!(state.is_running);
    }

    #[test]
    fn request_preview_with_nothing_selected_logs_message() {
        let mut state = CleanSysGui::new();
        let _ = update(&mut state, Message::RequestPreview);
        assert!(!state.previewing);
        assert!(state.logs.iter().any(|l| l.contains("No items selected")));
    }

    #[test]
    fn request_preview_with_selection_starts_previewing() {
        let mut state = CleanSysGui::new();
        state.categories[0].items[0].selected = true;
        let _ = update(&mut state, Message::RequestPreview);
        assert!(state.previewing);
        assert_eq!(state.operations_total, 1);
        // Preview never touches item status/is_running.
        assert!(!state.is_running);
    }

    #[test]
    fn preview_finished_records_result_and_closes_when_done() {
        let mut state = CleanSysGui::new();
        state.previewing = true;
        state.operations_total = 1;
        state.operations_completed = 0;

        let mut result = cleansys_core::CleaningResult::new();
        result.add_item(cleansys_core::CleanedItem::file(
            std::path::PathBuf::from("/tmp/preview"),
            100,
            "test",
        ));
        let _ = update(&mut state, Message::PreviewFinished(0, 0, Ok(result)));

        assert!(!state.previewing);
        assert!(state.preview_open);
        assert_eq!(state.preview_results.len(), 1);
    }

    #[test]
    fn close_preview_clears_results() {
        let mut state = CleanSysGui::new();
        state.preview_open = true;
        state
            .preview_results
            .push(("test".to_string(), cleansys_core::CleaningResult::new()));
        let _ = update(&mut state, Message::ClosePreview);
        assert!(!state.preview_open);
        assert!(state.preview_results.is_empty());
    }
}
