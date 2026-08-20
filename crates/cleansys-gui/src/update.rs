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
