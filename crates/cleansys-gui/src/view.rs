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
    if state.needs_admin_notice {
        return admin_notice_dialog(state, &c);
    }
    if state.confirm_run_pending {
        return confirm_run_dialog(state, &c);
    }
    if state.preview_open {
        return preview_dialog(state, &c);
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

    let busy = state.is_running || state.previewing;

    let run_button = button(text(run_label).size(14))
        .padding([10, 20])
        .style(button::primary)
        .on_press_maybe(if busy || selected == 0 {
            None
        } else {
            Some(Message::RequestRun)
        });

    let preview_label = if state.previewing {
        "⏳ Previewing…".to_string()
    } else {
        "🔍 Preview".to_string()
    };
    let preview_button = button(text(preview_label).size(14))
        .padding([10, 16])
        .style(button::secondary)
        .on_press_maybe(if busy || selected == 0 {
            None
        } else {
            Some(Message::RequestPreview)
        });

    let select_all_button = button(text("Select all").size(12))
        .padding([6, 12])
        .style(button::secondary)
        .on_press_maybe((!busy).then_some(Message::SelectAllEverywhere));
    let select_none_button = button(text("Select none").size(12))
        .padding([6, 12])
        .style(button::secondary)
        .on_press_maybe((!busy).then_some(Message::DeselectAllEverywhere));

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

    let mut rows = column![row![
        run_button,
        preview_button,
        Space::new().width(Length::Fixed(12.0)),
        select_all_button,
        select_none_button,
        Space::new().width(Length::Fill),
        summary,
    ]
    .spacing(10)
    .align_y(Alignment::Center)]
    .spacing(10);

    if busy && state.operations_total > 0 {
        let label = if state.previewing {
            format!(
                "Previewing {}/{}…",
                state.operations_completed, state.operations_total
            )
        } else {
            format!(
                "Cleaning {}/{}…",
                state.operations_completed, state.operations_total
            )
        };
        rows = rows.push(
            column![
                iced::widget::progress_bar(0.0..=1.0, state.progress_fraction())
                    .girth(Length::Fixed(8.0)),
                text(label).size(11).color(c.muted),
            ]
            .spacing(4),
        );
    }

    container(rows)
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

/// A generic centered modal wrapper: card content on top of a full-window
/// backdrop tinted with the theme's background colour.
fn modal_backdrop<'a>(card: Element<'a, Message>, c: ThemeColors) -> Element<'a, Message> {
    container(card)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(move |_theme: &iced::Theme| container::Style {
            background: Some(Color::from_rgba(c.bg.r, c.bg.g, c.bg.b, 0.97).into()),
            text_color: Some(c.text_primary),
            ..Default::default()
        })
        .into()
}

fn confirm_run_dialog<'a>(state: &'a CleanSysGui, c: &ThemeColors) -> Element<'a, Message> {
    let c = *c;
    let selected = state.selected_indices();
    let total_selected = selected.len();
    let needs_root = state.selection_needs_root();

    let mut names: Vec<Element<'a, Message>> = selected
        .iter()
        .filter_map(|(ci, ii)| {
            state
                .categories
                .get(*ci)
                .and_then(|c| c.items.get(*ii))
                .map(|item| {
                    let suffix = if item.requires_root { " (root)" } else { "" };
                    text(format!("  \u{2022} {}{}", item.name, suffix))
                        .size(13)
                        .color(c_text(&c, item.requires_root))
                        .into()
                })
        })
        .collect();
    if names.len() > 10 {
        let remaining = names.len() - 9;
        names.truncate(9);
        names.push(
            text(format!("  \u{2026} and {remaining} more"))
                .size(13)
                .color(c.muted)
                .into(),
        );
    }

    let mut content = column![
        row![
            icon(icons::EXCLAMATION_TRIANGLE, c.accent),
            text("Confirm cleaning").size(22).color(c.text_primary),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
        text(format!(
            "This will permanently delete files for {total_selected} selected cleaner(s):"
        ))
        .size(14)
        .color(c.text_secondary),
        column(names).spacing(2),
    ]
    .spacing(12)
    .padding(28)
    .max_width(480);

    if needs_root {
        content = content.push(
            text("Some of these require root/Administrator privileges.")
                .size(12)
                .color(c.accent),
        );
    }

    content = content.push(
        row![
            button(text("Run now"))
                .padding([8, 16])
                .style(button::primary)
                .on_press(Message::ConfirmRun),
            button(text("Cancel"))
                .padding([8, 16])
                .style(button::secondary)
                .on_press(Message::CancelRunRequest),
        ]
        .spacing(12),
    );

    let card = container(content).padding(8).style(surface_style(c));
    modal_backdrop(card.into(), c)
}

/// Pick a text colour: accent for root-requiring items, primary otherwise.
fn c_text(c: &ThemeColors, requires_root: bool) -> Color {
    if requires_root {
        c.accent
    } else {
        c.text_primary
    }
}

fn admin_notice_dialog<'a>(state: &'a CleanSysGui, c: &ThemeColors) -> Element<'a, Message> {
    let c = *c;
    let _ = state;

    let mut content = column![
        row![
            icon(icons::EXCLAMATION_TRIANGLE, c.accent),
            text("Administrator privileges required")
                .size(20)
                .color(c.text_primary),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
        text(
            "One or more selected cleaners need Administrator privileges. \
             Windows doesn't support entering a password inline the way \
             sudo does \u{2014} relaunch CleanSys as Administrator to continue."
        )
        .size(14)
        .color(c.text_secondary),
    ]
    .spacing(14)
    .padding(28)
    .max_width(440);

    content = content.push(
        row![
            button(text("Relaunch as Administrator"))
                .padding([8, 16])
                .style(button::primary)
                .on_press(Message::RelaunchAsAdmin),
            button(text("Cancel"))
                .padding([8, 16])
                .style(button::secondary)
                .on_press(Message::AdminNoticeAcknowledged),
        ]
        .spacing(12),
    );

    let card = container(content).padding(8).style(surface_style(c));
    modal_backdrop(card.into(), c)
}

fn preview_dialog<'a>(state: &'a CleanSysGui, c: &ThemeColors) -> Element<'a, Message> {
    let c = *c;

    let total: u64 = state
        .preview_results
        .iter()
        .map(|(_, r)| r.total_bytes)
        .sum();
    let total_items: usize = state
        .preview_results
        .iter()
        .map(|(_, r)| r.item_count())
        .sum();

    let mut rows: Vec<Element<'a, Message>> = Vec::new();
    for (name, result) in &state.preview_results {
        rows.push(
            text(format!(
                "{name} \u{2014} {} across {} item(s)",
                format_size(result.total_bytes),
                result.item_count()
            ))
            .size(13)
            .color(c.text_primary)
            .into(),
        );
        for item in result.items.iter().take(3) {
            rows.push(
                text(format!(
                    "    \u{2022} {} ({})",
                    item.path_str(),
                    format_size(item.size)
                ))
                .size(11)
                .color(c.text_secondary)
                .into(),
            );
        }
        if result.items.len() > 3 {
            rows.push(
                text(format!("    \u{2026} and {} more", result.items.len() - 3))
                    .size(11)
                    .color(c.muted)
                    .into(),
            );
        }
    }

    if rows.is_empty() {
        rows.push(
            text("Nothing to clean \u{2014} all selected cleaners are already empty.")
                .size(13)
                .color(c.muted)
                .into(),
        );
    }

    let content = column![
        row![
            icon(icons::CHECK_CIRCLE_FILL, c.green),
            text("Preview").size(22).color(c.text_primary),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
        text(format!(
            "Would free {} across {total_items} item(s). Nothing has been deleted.",
            format_size(total)
        ))
        .size(14)
        .color(c.text_secondary),
        rule::horizontal(1),
        scrollable(column(rows).spacing(4)).height(Length::Fixed(280.0)),
        row![button(text("Close"))
            .padding([8, 16])
            .style(button::secondary)
            .on_press(Message::ClosePreview),],
    ]
    .spacing(12)
    .padding(28)
    .max_width(520);

    let card = container(content).padding(8).style(surface_style(c));
    modal_backdrop(card.into(), c)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::CleanSysGui;

    #[test]
    fn view_does_not_panic_for_default_state() {
        let state = CleanSysGui::new();
        let _ = view(&state);
    }

    #[test]
    fn view_does_not_panic_with_selection_and_logs() {
        let mut state = CleanSysGui::new();
        state.categories[0].items[0].selected = true;
        state.push_log("some activity");
        state.push_log("more activity");
        let _ = view(&state);
    }

    #[test]
    fn view_does_not_panic_for_password_dialog() {
        let mut state = CleanSysGui::new();
        state.needs_password = true;
        state.password_error = Some("nope".to_string());
        let _ = view(&state);
    }

    #[test]
    fn view_does_not_panic_for_admin_notice_dialog() {
        let mut state = CleanSysGui::new();
        state.needs_admin_notice = true;
        let _ = view(&state);
    }

    #[test]
    fn view_does_not_panic_for_confirm_run_dialog() {
        let mut state = CleanSysGui::new();
        state.categories[0].items[0].selected = true;
        state.confirm_run_pending = true;
        let _ = view(&state);
    }

    #[test]
    fn view_does_not_panic_for_preview_dialog_empty() {
        let mut state = CleanSysGui::new();
        state.preview_open = true;
        let _ = view(&state);
    }

    #[test]
    fn view_does_not_panic_for_preview_dialog_with_results() {
        let mut state = CleanSysGui::new();
        state.preview_open = true;
        let mut result = cleansys_core::CleaningResult::new();
        result.add_item(cleansys_core::CleanedItem::file(
            std::path::PathBuf::from("/tmp/preview-item"),
            2048,
            "test",
        ));
        state
            .preview_results
            .push(("Test Cleaner".to_string(), result));
        let _ = view(&state);
    }

    #[test]
    fn view_does_not_panic_while_running_with_progress() {
        let mut state = CleanSysGui::new();
        state.is_running = true;
        state.operations_total = 4;
        state.operations_completed = 2;
        let _ = view(&state);
    }

    #[test]
    fn view_does_not_panic_for_every_theme() {
        let mut state = CleanSysGui::new();
        for i in 0..cleansys_core::THEME_COUNT {
            state.theme_index = i;
            let _ = view(&state);
        }
    }

    #[test]
    fn view_does_not_panic_with_item_last_result_detail() {
        let mut state = CleanSysGui::new();
        let mut result = cleansys_core::CleaningResult::new();
        for i in 0..8 {
            result.add_item(cleansys_core::CleanedItem::file(
                std::path::PathBuf::from(format!("/tmp/item-{i}")),
                (i as u64 + 1) * 100,
                "test",
            ));
        }
        state.categories[0].items[0].last_result = Some(result);
        let _ = view(&state);
    }
}
