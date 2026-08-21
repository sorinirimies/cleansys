//! View (rendering) logic for the CleanSys Iced GUI.

use cleansys_core::{format_size, Status};
use iced::widget::{
    button, checkbox, column, container, row, rule, scrollable, text, text_input, Space,
};
use iced::{Alignment, Color, Element, Length};

use crate::icons;
use crate::message::Message;
use crate::state::CleanSysGui;
use crate::theme::ThemeColors;
use crate::theme_selector::theme_selector;

/// Render the full application view.
pub fn view(state: &CleanSysGui) -> Element<'_, Message> {
    let c = state.colors();

    if state.needs_password {
        return password_dialog(state, &c);
    }

    let content = column![
        top_bar(state, &c),
        controls_bar(state, &c),
        tab_bar(state, &c),
        active_category_panel(state, &c),
        log_panel(state, &c),
    ]
    .spacing(14)
    .padding(20)
    .height(Length::Fill);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_theme: &iced::Theme| container::Style {
            background: Some(c.bg.into()),
            text_color: Some(c.text_primary),
            ..Default::default()
        })
        .into()
}

// ── Sections ────────────────────────────────────────────────────────────────

fn top_bar<'a>(state: &'a CleanSysGui, c: &ThemeColors) -> Element<'a, Message> {
    let c = *c;
    let root_badge = if state.is_root {
        badge("ROOT", c.green)
    } else {
        badge("USER", c.accent)
    };

    let title_row = row![
        text("🧹 CleanSys").size(26).color(c.text_primary),
        root_badge,
        Space::new().width(Length::Fill),
        text("Theme").size(13).color(c.text_secondary),
        theme_selector(state.theme_index),
    ]
    .spacing(12)
    .align_y(Alignment::Center);

    container(
        column![
            title_row,
            text(if state.is_root {
                "Running with root privileges — all cleaners can run directly."
            } else {
                "Running as a normal user — system cleaners will prompt for your sudo password."
            })
            .size(13)
            .color(c.text_secondary),
        ]
        .spacing(4),
    )
    .padding([16, 20])
    .width(Length::Fill)
    .style(move |_theme: &iced::Theme| container::Style {
        background: Some(c.header_bg.into()),
        border: iced::Border {
            radius: 10.0.into(),
            ..Default::default()
        },
        ..Default::default()
    })
    .into()
}

fn controls_bar<'a>(state: &'a CleanSysGui, c: &ThemeColors) -> Element<'a, Message> {
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
        .size(14)
        .color(c.text_primary),
        text(format!(
            "{} item(s) selected across all categories",
            selected
        ))
        .size(12)
        .color(c.muted),
    ]
    .spacing(2);

    container(
        row![run_button, summary]
            .spacing(20)
            .align_y(Alignment::Center),
    )
    .padding(16)
    .width(Length::Fill)
    .style(surface_style(*c))
    .into()
}

fn tab_bar<'a>(state: &'a CleanSysGui, _c: &ThemeColors) -> Element<'a, Message> {
    let tabs: Vec<Element<'a, Message>> = state
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

fn active_category_panel<'a>(state: &'a CleanSysGui, c: &ThemeColors) -> Element<'a, Message> {
    let Some(category) = state.categories.get(state.active_tab) else {
        return Space::new().into();
    };
    let cat_idx = state.active_tab;

    let title_row = row![
        text(category.description.clone()).size(13).color(c.muted),
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

    let items: Vec<Element<'a, Message>> = category
        .items
        .iter()
        .enumerate()
        .map(|(item_idx, item)| item_row(cat_idx, item_idx, item, c))
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
    .style(surface_style(*c))
    .into()
}

fn item_row<'a>(
    cat_idx: usize,
    item_idx: usize,
    item: &'a cleansys_core::CleanerItem,
    c: &ThemeColors,
) -> Element<'a, Message> {
    let box_ = checkbox(item.selected)
        .label(item.name.clone())
        .size(16)
        .on_toggle(move |_| Message::ToggleItem(cat_idx, item_idx));

    let root_tag: Element<'_, Message> = if item.requires_root {
        badge("ROOT", c.accent)
    } else {
        Space::new().into()
    };

    let status_line: Element<'_, Message> = match &item.status {
        Some(Status::Success(msg)) => row![
            icon(icons::CHECK_CIRCLE_FILL, c.green),
            text(msg.clone()).size(12).color(c.green),
        ]
        .spacing(6)
        .align_y(Alignment::Center)
        .into(),
        Some(Status::Error(msg)) => row![
            icon(icons::X_CIRCLE, c.red),
            text(msg.clone()).size(12).color(c.red),
        ]
        .spacing(6)
        .align_y(Alignment::Center)
        .into(),
        Some(Status::Running) => row![
            icon(icons::ARROW_REPEAT, c.accent),
            text("running…").size(12).color(c.accent),
        ]
        .spacing(6)
        .align_y(Alignment::Center)
        .into(),
        Some(Status::Pending) => row![
            icon(icons::CLOCK, c.muted),
            text("pending").size(12).color(c.muted),
        ]
        .spacing(6)
        .align_y(Alignment::Center)
        .into(),
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

    let mut body = column![
        header_row,
        text(item.description.clone()).size(12).color(c.muted),
    ]
    .spacing(2);

    // Show a real per-file/per-directory breakdown of what was actually
    // cleaned (path + size), not just the aggregate total.
    if let Some(result) = &item.last_result {
        if !result.items.is_empty() {
            let mut items_sorted: Vec<_> = result.items.iter().collect();
            items_sorted.sort_by_key(|i| std::cmp::Reverse(i.size));

            let mut detail_lines: Vec<Element<'a, Message>> = items_sorted
                .iter()
                .take(5)
                .map(|cleaned| {
                    text(format!(
                        "    • {} — {}",
                        cleaned.path_str(),
                        format_size(cleaned.size)
                    ))
                    .size(11)
                    .color(c.text_secondary)
                    .into()
                })
                .collect();

            if items_sorted.len() > 5 {
                detail_lines.push(
                    text(format!("    … and {} more", items_sorted.len() - 5))
                        .size(11)
                        .color(c.muted)
                        .into(),
                );
            }

            body = body.push(column(detail_lines).spacing(1));
        }
    }

    container(body)
        .padding([8, 10])
        .width(Length::Fill)
        .style(item_row_style(*c))
        .into()
}

fn log_panel<'a>(state: &'a CleanSysGui, c: &ThemeColors) -> Element<'a, Message> {
    let log_lines: Vec<Element<'a, Message>> = if state.logs.is_empty() {
        vec![text("No activity yet.").size(12).color(c.muted).into()]
    } else {
        state
            .logs
            .iter()
            .rev()
            .take(200)
            .map(|l| text(l.clone()).size(12).color(c.text_secondary).into())
            .collect()
    };

    container(
        column![
            row![
                text("Activity log").size(14).color(c.text_primary),
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
    .style(surface_style(*c))
    .into()
}

// ── Small shared widgets ─────────────────────────────────────────────────────

/// A reusable "elevated surface" container style (cards, panels) tinted by
/// the active theme.
fn surface_style(c: ThemeColors) -> impl Fn(&iced::Theme) -> container::Style {
    move |_theme: &iced::Theme| container::Style {
        background: Some(c.surface.into()),
        border: iced::Border {
            color: c.border,
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    }
}

/// A slightly-highlighted row background, used for individual cleaner items.
fn item_row_style(c: ThemeColors) -> impl Fn(&iced::Theme) -> container::Style {
    move |_theme: &iced::Theme| container::Style {
        background: Some(c.surface_highlight.into()),
        border: iced::Border {
            radius: 6.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

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

fn password_dialog<'a>(state: &'a CleanSysGui, c: &ThemeColors) -> Element<'a, Message> {
    let c = *c;
    let mut content = column![
        row![
            icon(icons::EXCLAMATION_TRIANGLE, c.accent),
            text("Authentication required")
                .size(22)
                .color(c.text_primary),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
        text("System cleaners require root privileges. Enter your password to continue:")
            .size(14)
            .color(c.text_secondary),
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
        content = content.push(text(format!("❌ {}", err)).size(13).color(c.red));
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

    let card = container(content).padding(8).style(surface_style(c));

    container(card)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(move |_theme: &iced::Theme| container::Style {
            background: Some(c.bg.into()),
            text_color: Some(c.text_primary),
            ..Default::default()
        })
        .into()
}
