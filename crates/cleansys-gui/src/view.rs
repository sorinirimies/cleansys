//! View (rendering) logic for the CleanSys Iced GUI.

use cleansys_core::{format_size, Status};
use iced::widget::{button, checkbox, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Element, Length};

use crate::message::Message;
use crate::state::CleanSysGui;

/// Render the full application view.
pub fn view(state: &CleanSysGui) -> Element<'_, Message> {
    if state.needs_password {
        return password_dialog(state);
    }

    let header = column![
        text("CleanSys").size(28),
        text(if state.is_root {
            "Running with root privileges".to_string()
        } else {
            "Running as a normal user — system cleaners will prompt for sudo".to_string()
        })
        .size(14),
    ]
    .spacing(4);

    let categories = column(
        state
            .categories
            .iter()
            .enumerate()
            .map(|(idx, category)| category_view(idx, category))
            .collect::<Vec<_>>(),
    )
    .spacing(16);

    let selected = state.selected_count();
    let run_label = if state.is_running {
        "Cleaning…".to_string()
    } else if selected == 0 {
        "Run selected".to_string()
    } else {
        format!("Run {} selected", selected)
    };

    let run_button = button(text(run_label)).on_press_maybe(if state.is_running || selected == 0 {
        None
    } else {
        Some(Message::RunSelected)
    });

    let summary = text(format!(
        "Total freed this run: {}",
        format_size(state.total_bytes_cleaned)
    ));

    let controls = row![run_button, summary]
        .spacing(16)
        .align_y(Alignment::Center);

    let log_lines: Vec<Element<'_, Message>> = state
        .logs
        .iter()
        .rev()
        .take(200)
        .map(|l| text(l.clone()).size(13).into())
        .collect();

    let log_panel = column![
        row![
            text("Activity log").size(16),
            button(text("Clear")).on_press(Message::ClearLog),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
        scrollable(column(log_lines).spacing(2)).height(Length::Fixed(180.0)),
    ]
    .spacing(6);

    let content = column![
        header,
        controls,
        scrollable(categories).height(Length::Fill),
        log_panel,
    ]
    .spacing(16)
    .padding(20);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn category_view(
    cat_idx: usize,
    category: &cleansys_core::CleanerCategory,
) -> Element<'_, Message> {
    let title_row = row![
        text(category.name.clone()).size(20),
        button(text("All")).on_press(Message::SelectAllCategory(cat_idx)),
        button(text("None")).on_press(Message::DeselectAllCategory(cat_idx)),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let items = column(
        category
            .items
            .iter()
            .enumerate()
            .map(|(item_idx, item)| item_view(cat_idx, item_idx, item))
            .collect::<Vec<_>>(),
    )
    .spacing(6);

    column![
        title_row,
        text(category.description.clone()).size(13),
        items
    ]
    .spacing(6)
    .into()
}

fn item_view<'a>(
    cat_idx: usize,
    item_idx: usize,
    item: &'a cleansys_core::CleanerItem,
) -> Element<'a, Message> {
    let status_glyph = match &item.status {
        Some(status) => status.get_animation_frame(0),
        None => "",
    };

    let label = format!(
        "{}{} — {}",
        item.name,
        if item.requires_root { " (root)" } else { "" },
        item.description
    );

    let box_ = checkbox(item.selected)
        .label(label)
        .on_toggle(move |_| Message::ToggleItem(cat_idx, item_idx));

    let status_text: Element<'_, Message> = match &item.status {
        Some(Status::Success(msg)) => text(format!("{} {}", status_glyph, msg)).size(12).into(),
        Some(Status::Error(msg)) => text(format!("{} {}", status_glyph, msg)).size(12).into(),
        Some(Status::Running) => text(format!("{} running…", status_glyph)).size(12).into(),
        Some(Status::Pending) => text(format!("{} pending", status_glyph)).size(12).into(),
        None => text("").size(12).into(),
    };

    row![box_, status_text]
        .spacing(10)
        .align_y(Alignment::Center)
        .into()
}

fn password_dialog(state: &CleanSysGui) -> Element<'_, Message> {
    let mut content = column![
        text("🔒 Authentication required").size(22),
        text("System cleaners require root privileges. Enter your password to continue:").size(14),
        text_input("Password", &state.password_input)
            .secure(true)
            .on_input(Message::PasswordChanged)
            .on_submit(Message::PasswordSubmit)
            .padding(8),
    ]
    .spacing(12)
    .padding(24)
    .max_width(420);

    if let Some(err) = &state.password_error {
        content = content.push(text(format!("❌ {}", err)).size(13));
    }

    content = content.push(
        row![
            button(text("Authenticate")).on_press(Message::PasswordSubmit),
            button(text("Cancel")).on_press(Message::PasswordCancel),
        ]
        .spacing(12),
    );

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
