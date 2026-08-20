//! View (rendering) logic for the CleanSys Iced GUI.

use cleansys_core::{format_size, Status};
use iced::widget::{
    button, checkbox, column, container, row, rule, scrollable, text, text_input, Space,
};
use iced::{Alignment, Color, Element, Length};

use crate::icons;
use crate::message::Message;
use crate::state::CleanSysGui;

const MUTED: Color = Color::from_rgba(0.72, 0.74, 0.78, 1.0);
const DIM: Color = Color::from_rgba(0.55, 0.57, 0.62, 1.0);
const SUCCESS: Color = Color::from_rgb(0.36, 0.78, 0.45);
const DANGER: Color = Color::from_rgb(0.92, 0.42, 0.42);
const ACCENT: Color = Color::from_rgb(0.51, 0.53, 0.94);

/// Render the full application view.
pub fn view(state: &CleanSysGui) -> Element<'_, Message> {
    if state.needs_password {
        return password_dialog(state);
    }

    let content = column![
        top_bar(state),
        controls_bar(state),
        tab_bar(state),
        active_category_panel(state),
        log_panel(state),
    ]
    .spacing(14)
    .padding(20)
    .height(Length::Fill);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

// ── Sections ────────────────────────────────────────────────────────────────

fn top_bar(state: &CleanSysGui) -> Element<'_, Message> {
    let root_badge = if state.is_root {
        badge("ROOT", SUCCESS)
    } else {
        badge("USER", ACCENT)
    };

    container(
        column![
            row![text("🧹 CleanSys").size(26), root_badge]
                .spacing(12)
                .align_y(Alignment::Center),
            text(if state.is_root {
                "Running with root privileges — all cleaners can run directly."
            } else {
                "Running as a normal user — system cleaners will prompt for your sudo password."
            })
            .size(13)
            .color(MUTED),
        ]
        .spacing(4),
    )
    .padding([16, 20])
    .width(Length::Fill)
    .style(top_bar_style)
    .into()
}

fn top_bar_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Color::from_rgb(0.145, 0.153, 0.19).into()),
        border: iced::Border {
            radius: 10.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

fn controls_bar(state: &CleanSysGui) -> Element<'_, Message> {
    let selected = state.selected_count();
    let run_label = if state.is_running {
        "⏳ Cleaning…".to_string()
    } else if selected == 0 {
        "Select cleaners to run".to_string()
    } else {
        format!("▶ Run {} selected", selected)
    };

    let run_button = button(text(run_label).size(14))
        .padding([10, 20])
        .style(button::primary)
        .on_press_maybe(if state.is_running || selected == 0 {
            None
        } else {
            Some(Message::RunSelected)
        });

    let summary = column![
        text(format!(
            "Total freed this run: {}",
            format_size(state.total_bytes_cleaned)
        ))
        .size(14),
        text(format!(
            "{} item(s) selected across all categories",
            selected
        ))
        .size(12)
        .color(DIM),
    ]
    .spacing(2);

    container(
        row![run_button, summary]
            .spacing(20)
            .align_y(Alignment::Center),
    )
    .padding(16)
    .width(Length::Fill)
    .style(container::rounded_box)
    .into()
}

fn tab_bar(state: &CleanSysGui) -> Element<'_, Message> {
    let tabs: Vec<Element<'_, Message>> = state
        .categories
        .iter()
        .enumerate()
        .map(|(idx, category)| {
            let is_active = idx == state.active_tab;
            let icon_glyph = if category.items.iter().any(|i| i.requires_root) {
                "🛡️"
            } else {
                "👤"
            };
            let label = text(format!(
                "{icon_glyph} {} ({}/{})",
                category.name,
                state.selected_count_in(idx),
                category.items.len()
            ))
            .size(13);

            button(label)
                .padding([8, 16])
                .style(if is_active {
                    button::primary
                } else {
                    button::secondary
                })
                .on_press(Message::SwitchCategoryTab(idx))
                .into()
        })
        .collect();

    row(tabs).spacing(8).into()
}

fn active_category_panel(state: &CleanSysGui) -> Element<'_, Message> {
    let Some(category) = state.categories.get(state.active_tab) else {
        return Space::new().into();
    };
    let cat_idx = state.active_tab;

    let title_row = row![
        text(category.description.clone()).size(13).color(MUTED),
        Space::new().width(Length::Fill),
        button(text("Select all").size(12))
            .padding([5, 12])
            .style(button::secondary)
            .on_press(Message::SelectAllCategory(cat_idx)),
        button(text("Select none").size(12))
            .padding([5, 12])
            .style(button::secondary)
            .on_press(Message::DeselectAllCategory(cat_idx)),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let items: Vec<Element<'_, Message>> = category
        .items
        .iter()
        .enumerate()
        .map(|(item_idx, item)| item_row(cat_idx, item_idx, item))
        .collect();

    let list = scrollable(column(items).spacing(4)).height(Length::Fill);

    container(
        column![title_row, rule::horizontal(1), list]
            .spacing(12)
            .height(Length::Fill),
    )
    .padding(18)
    .width(Length::Fill)
    .height(Length::Fill)
    .style(container::rounded_box)
    .into()
}

fn item_row<'a>(
    cat_idx: usize,
    item_idx: usize,
    item: &'a cleansys_core::CleanerItem,
) -> Element<'a, Message> {
    let box_ = checkbox(item.selected)
        .label(item.name.clone())
        .size(16)
        .on_toggle(move |_| Message::ToggleItem(cat_idx, item_idx));

    let root_tag: Element<'_, Message> = if item.requires_root {
        badge("ROOT", ACCENT)
    } else {
        Space::new().into()
    };

    let status_line: Element<'_, Message> = match &item.status {
        Some(Status::Success(msg)) => row![
            icon(icons::CHECK_CIRCLE_FILL, SUCCESS),
            text(msg.clone()).size(12).color(SUCCESS),
        ]
        .spacing(6)
        .align_y(Alignment::Center)
        .into(),
        Some(Status::Error(msg)) => row![
            icon(icons::X_CIRCLE, DANGER),
            text(msg.clone()).size(12).color(DANGER),
        ]
        .spacing(6)
        .align_y(Alignment::Center)
        .into(),
        Some(Status::Running) => row![
            icon(icons::ARROW_REPEAT, ACCENT),
            text("running…").size(12).color(ACCENT),
        ]
        .spacing(6)
        .align_y(Alignment::Center)
        .into(),
        Some(Status::Pending) => {
            row![icon(icons::CLOCK, DIM), text("pending").size(12).color(DIM),]
                .spacing(6)
                .align_y(Alignment::Center)
                .into()
        }
        None => Space::new().into(),
    };

    let header_row = row![
        box_,
        root_tag,
        Space::new().width(Length::Fill),
        status_line
    ]
    .spacing(10)
    .align_y(Alignment::Center);

    container(
        column![
            header_row,
            text(item.description.clone()).size(12).color(MUTED),
        ]
        .spacing(2),
    )
    .padding([8, 10])
    .width(Length::Fill)
    .style(item_row_style)
    .into()
}

fn item_row_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.03).into()),
        border: iced::Border {
            radius: 6.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

fn log_panel(state: &CleanSysGui) -> Element<'_, Message> {
    let log_lines: Vec<Element<'_, Message>> = if state.logs.is_empty() {
        vec![text("No activity yet.").size(12).color(DIM).into()]
    } else {
        state
            .logs
            .iter()
            .rev()
            .take(200)
            .map(|l| text(l.clone()).size(12).into())
            .collect()
    };

    container(
        column![
            row![
                text("Activity log").size(14),
                Space::new().width(Length::Fill),
                button(text("Clear").size(12))
                    .padding([4, 10])
                    .style(button::secondary)
                    .on_press(Message::ClearLog),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            rule::horizontal(1),
            scrollable(column(log_lines).spacing(3)).height(Length::Fixed(130.0)),
        ]
        .spacing(8),
    )
    .padding(14)
    .width(Length::Fill)
    .style(container::rounded_box)
    .into()
}

// ── Small shared widgets ─────────────────────────────────────────────────────

fn icon(glyph: char, color: Color) -> Element<'static, Message> {
    text(glyph.to_string())
        .font(icons::FONT)
        .size(13)
        .color(color)
        .into()
}

fn badge(label: &'static str, color: Color) -> Element<'static, Message> {
    container(text(label).size(11).color(Color::WHITE))
        .padding([3, 8])
        .style(move |_theme: &iced::Theme| container::Style {
            background: Some(color.into()),
            text_color: Some(Color::WHITE),
            border: iced::Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

fn password_dialog(state: &CleanSysGui) -> Element<'_, Message> {
    let mut content = column![
        row![
            icon(icons::EXCLAMATION_TRIANGLE, ACCENT),
            text("Authentication required").size(22),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
        text("System cleaners require root privileges. Enter your password to continue:")
            .size(14)
            .color(MUTED),
        text_input("Password", &state.password_input)
            .secure(true)
            .on_input(Message::PasswordChanged)
            .on_submit(Message::PasswordSubmit)
            .padding(10),
    ]
    .spacing(14)
    .padding(28)
    .max_width(420);

    if let Some(err) = &state.password_error {
        content = content.push(text(format!("❌ {}", err)).size(13).color(DANGER));
    }

    content = content.push(
        row![
            button(text("Authenticate"))
                .padding([8, 16])
                .style(button::primary)
                .on_press(Message::PasswordSubmit),
            button(text("Cancel"))
                .padding([8, 16])
                .style(button::secondary)
                .on_press(Message::PasswordCancel),
        ]
        .spacing(12),
    );

    let card = container(content).padding(8).style(container::bordered_box);

    container(card)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
