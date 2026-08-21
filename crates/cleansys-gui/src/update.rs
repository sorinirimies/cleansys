//! Update logic (Elm-style) for the CleanSys Iced GUI.

use cleansys_core::{format_size, Status};
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
            Task::none()
        }

        Message::SelectAllCategory(cat_idx) => {
            if let Some(category) = state.categories.get_mut(cat_idx) {
                for item in &mut category.items {
                    item.selected = true;
                }
            }
            Task::none()
        }

        Message::DeselectAllCategory(cat_idx) => {
            if let Some(category) = state.categories.get_mut(cat_idx) {
                for item in &mut category.items {
                    item.selected = false;
                }
            }
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

        Message::RunSelected => run_selected(state),

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

        Message::OperationFinished(cat_idx, item_idx, result) => {
            let mut log_line = None;
            if let Some(item) = state
                .categories
                .get_mut(cat_idx)
                .and_then(|c| c.items.get_mut(item_idx))
            {
                match result {
                    Ok(bytes) => {
                        item.status =
                            Some(Status::Success(format!("Cleaned {}", format_size(bytes))));
                        item.bytes_cleaned = bytes;
                        state.total_bytes_cleaned += bytes;
                        log_line = Some(format!("✅ {}: freed {}", item.name, format_size(bytes)));
                    }
                    Err(err) => {
                        item.status = Some(Status::Error(err.clone()));
                        log_line = Some(format!("❌ {}: {}", item.name, err));
                    }
                }
            }
            if let Some(line) = log_line {
                state.push_log(line);
            }

            let still_running = state
                .categories
                .iter()
                .flat_map(|c| &c.items)
                .any(|i| matches!(i.status, Some(Status::Pending | Status::Running)));

            if !still_running {
                state.is_running = false;
                state.push_log(format!(
                    "🎉 Done — total freed: {}",
                    format_size(state.total_bytes_cleaned)
                ));
            }

            Task::none()
        }
    }
}

/// Gather selected items, either running them immediately or — if any need
/// root and we don't have it yet — showing the sudo password dialog first.
fn run_selected(state: &mut CleanSysGui) -> Task<Message> {
    if state.is_running {
        return Task::none();
    }

    let selected: Vec<(usize, usize)> = state
        .categories
        .iter()
        .enumerate()
        .flat_map(|(ci, c)| {
            c.items
                .iter()
                .enumerate()
                .filter(|(_, i)| i.selected)
                .map(move |(ii, _)| (ci, ii))
        })
        .collect();

    if selected.is_empty() {
        state.push_log("No items selected. Please select at least one cleaner.".to_string());
        return Task::none();
    }

    if state.selection_needs_root() {
        state.pending_root_ops = selected;
        state.needs_password = true;
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
        state.push_log(format!("🔄 Running: {}", name));

        tasks.push(Task::perform(
            async move { function(true) },
            move |result| {
                Message::OperationFinished(cat_idx, item_idx, result.map_err(|e| e.to_string()))
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
    fn run_selected_with_nothing_selected_logs_message_and_does_not_run() {
        let mut state = CleanSysGui::new();
        let _ = update(&mut state, Message::RunSelected);
        assert!(!state.is_running);
        assert!(state.logs.iter().any(|l| l.contains("No items selected")));
    }

    #[test]
    fn run_selected_with_user_only_selection_starts_immediately() {
        let mut state = CleanSysGui::new();
        state.categories[0].items[0].selected = true;
        let _ = update(&mut state, Message::RunSelected);

        assert!(state.is_running);
        assert!(!state.needs_password);
        assert!(matches!(
            state.categories[0].items[0].status,
            Some(Status::Running)
        ));
    }

    #[test]
    fn run_selected_with_root_item_shows_password_dialog_when_not_root() {
        let mut state = CleanSysGui::new();
        state.is_root = false;
        state.categories[1].items[0].selected = true;

        let _ = update(&mut state, Message::RunSelected);

        assert!(state.needs_password);
        assert!(!state.is_running);
        assert_eq!(state.pending_root_ops, vec![(1, 0)]);
    }

    #[test]
    fn run_selected_with_root_item_runs_immediately_when_already_root() {
        let mut state = CleanSysGui::new();
        state.is_root = true;
        state.categories[1].items[0].selected = true;

        let _ = update(&mut state, Message::RunSelected);

        assert!(!state.needs_password);
        assert!(state.is_running);
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
        state.pending_root_ops = vec![(1, 0)];
        state.categories[1].items[0].selected = true;

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

        let _ = update(&mut state, Message::OperationFinished(0, 0, Ok(2048)));

        assert_eq!(state.total_bytes_cleaned, 2048);
        assert!(matches!(
            state.categories[0].items[0].status,
            Some(Status::Success(_))
        ));
        // No other items are pending/running, so the run should be marked done.
        assert!(!state.is_running);
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

        let _ = update(&mut state, Message::OperationFinished(0, 0, Ok(10)));

        // Another item is still pending, so the overall run isn't finished yet.
        assert!(state.is_running);
    }
}
