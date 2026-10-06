//! View (rendering) logic for the CleanSys Iced GUI.

use cleansys_core::{format_size, Status};
use iced::widget::{
    button, checkbox, column, container, pick_list, responsive, row, rule, scrollable, text,
    text_input, tooltip, Space,
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
    if state.schedule_open {
        return schedule_dialog(state, &c);
    }
    if state.preview_open {
        return preview_dialog(state, &c);
    }

    let content = main_content(state, &c);

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

/// Width breakpoints (logical pixels) for the responsive layout.
const WIDE: f32 = 980.0; // sidebar + list
const MEDIUM: f32 = 700.0; // narrower sidebar
                           // below MEDIUM: no sidebar — a category drop-down above the list

/// How the main screen is laid out at a given window width.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Layout {
    Wide,
    Medium,
    Narrow,
}

impl Layout {
    fn for_width(w: f32) -> Self {
        if w >= WIDE {
            Layout::Wide
        } else if w >= MEDIUM {
            Layout::Medium
        } else {
            Layout::Narrow
        }
    }

    fn sidebar_width(self) -> Option<f32> {
        match self {
            Layout::Wide => Some(300.0),
            Layout::Medium => Some(230.0),
            Layout::Narrow => None,
        }
    }
}

/// The main (non-overlay) screen. Responsive: it re-lays itself out from the
/// real window width (`iced::widget::responsive`), see [`Layout`].
fn main_content<'a>(state: &'a CleanSysGui, c: &ThemeColors) -> Element<'a, Message> {
    let c = *c;
    responsive(move |size| main_layout(state, &c, size.width)).into()
}

/// The screen for a concrete width. Separate from [`main_content`] so tests
/// can build every layout directly.
fn main_layout<'a>(state: &'a CleanSysGui, c: &ThemeColors, width: f32) -> Element<'a, Message> {
    let layout = Layout::for_width(width);
    let pad = if layout == Layout::Narrow { 10 } else { 16 };

    let list_pane = column![
        category_header(state, c, layout),
        item_list(state, c, layout),
    ]
    .spacing(10)
    .width(Length::Fill)
    .height(Length::Fill);

    let body: Element<'a, Message> = match layout.sidebar_width() {
        Some(w) => row![sidebar(state, c, w), list_pane]
            .spacing(14)
            .height(Length::Fill)
            .into(),
        None => list_pane.into(),
    };

    let mut page = column![header(state, c, layout), body].spacing(12);
    if state.show_log {
        page = page.push(log_panel(state, c));
    }
    page = page.push(action_bar(state, c, layout));

    page.padding(pad)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

// ── Header ───────────────────────────────────────────────────────────────────

fn header<'a>(state: &'a CleanSysGui, c: &ThemeColors, layout: Layout) -> Element<'a, Message> {
    let c = *c;
    let root_badge = if state.is_root {
        badge("ROOT", c.green)
    } else {
        badge("USER", c.accent)
    };

    let search = text_input(
        "Search cleaners…  (browsers, gradle, docker, logs)",
        &state.search,
    )
    .on_input(Message::SearchChanged)
    .padding(8)
    .size(14)
    .width(Length::Fill);
    let clear: Element<'a, Message> = if state.search.is_empty() {
        Space::new().width(Length::Fixed(0.0)).into()
    } else {
        button(text("✕").size(13))
            .padding([6, 10])
            .style(button::secondary)
            .on_press(Message::ClearSearch)
            .into()
    };

    let schedule_label = if layout == Layout::Narrow {
        "⏰"
    } else {
        "⏰ Schedule"
    };
    let schedule = button(text(schedule_label).size(13))
        .padding([8, 12])
        .style(button::secondary)
        .on_press_maybe((!(state.is_running)).then_some(Message::OpenSchedule));

    let title = row![
        text("🧹 CleanSys")
            .size(if layout == Layout::Narrow { 20 } else { 24 })
            .color(c.text_primary),
        root_badge
    ]
    .spacing(10)
    .align_y(Alignment::Center);

    let content: Element<'a, Message> = match layout {
        Layout::Narrow => column![
            row![title, Space::new().width(Length::Fill), schedule].align_y(Alignment::Center),
            row![search, clear].spacing(6).align_y(Alignment::Center),
        ]
        .spacing(8)
        .into(),
        _ => row![
            title,
            Space::new().width(Length::Fixed(12.0)),
            search,
            clear,
            schedule,
            theme_selector(state.theme_index),
        ]
        .spacing(10)
        .align_y(Alignment::Center)
        .into(),
    };

    container(content)
        .padding([12, 16])
        .width(Length::Fill)
        .style(move |_t: &iced::Theme| container::Style {
            background: Some(c.header_bg.into()),
            border: iced::Border {
                radius: 10.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

// ── Sidebar / category navigation ────────────────────────────────────────────

/// `true` for categories on the root/system side of the UI.
fn is_root_category(name: &str, items: &[cleansys_core::CleanerItem]) -> bool {
    name == "System Cleaners"
        || name.ends_with(cleansys_core::model::ROOT_SUFFIX)
        || (!items.is_empty() && items.iter().all(|i| i.requires_root))
}

fn sidebar<'a>(state: &'a CleanSysGui, c: &ThemeColors, width: f32) -> Element<'a, Message> {
    let c = *c;
    let mut user_rows: Vec<Element<'a, Message>> = Vec::new();
    let mut root_rows: Vec<Element<'a, Message>> = Vec::new();

    for (idx, cat) in state.categories.iter().enumerate() {
        if !state.category_visible(idx) {
            continue;
        }
        let row_el = category_button(state, &c, idx);
        if is_root_category(&cat.name, &cat.items) {
            root_rows.push(row_el);
        } else {
            user_rows.push(row_el);
        }
    }

    let section = |label: &'static str, hint: &'static str| -> Element<'a, Message> {
        column![
            text(label).size(11).color(c.muted),
            text(hint).size(10).color(c.muted),
        ]
        .spacing(1)
        .padding([8, 6])
        .into()
    };

    let mut col = column![section("USER LAND", "no password needed")].spacing(3);
    col = col.push(column(user_rows).spacing(3));
    if !root_rows.is_empty() {
        col = col.push(section("SYSTEM · ROOT", "asks for your password"));
        col = col.push(column(root_rows).spacing(3));
    }

    container(scrollable(col.padding([4, 6])).height(Length::Fill))
        .width(Length::Fixed(width))
        .height(Length::Fill)
        .style(surface_style(c))
        .into()
}

fn category_button<'a>(
    state: &'a CleanSysGui,
    c: &ThemeColors,
    idx: usize,
) -> Element<'a, Message> {
    let c = *c;
    let cat = &state.categories[idx];
    let active = idx == state.active_tab && state.search.trim().is_empty();
    let ticked = state.selected_count_in(idx);

    let right: Element<'a, Message> = match state.category_bytes(idx) {
        Some(b) if b > 0 => text(format_size(b))
            .size(12)
            .color(size_color(&c, b))
            .into(),
        Some(_) => text("—").size(12).color(c.muted).into(),
        None if state.board.is_scanning() => text("…").size(12).color(c.muted).into(),
        None => Space::new().into(),
    };
    let tick: Element<'a, Message> = if ticked > 0 {
        text(format!("{ticked} ✓")).size(11).color(c.accent).into()
    } else {
        Space::new().into()
    };

    let label = row![
        text(
            cat.name
                .trim_end_matches(cleansys_core::model::ROOT_SUFFIX)
                .to_string()
        )
        .size(13)
        .width(Length::Fill),
        tick,
        right,
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    button(label)
        .padding([8, 10])
        .width(Length::Fill)
        .style(move |theme: &iced::Theme, status: button::Status| {
            let base = button::text(theme, status);
            let hovered = matches!(status, button::Status::Hovered);
            button::Style {
                background: if active {
                    Some(c.selection.into())
                } else if hovered {
                    Some(c.surface_highlight.into())
                } else {
                    None
                },
                text_color: if active {
                    c.text_primary
                } else {
                    c.text_secondary
                },
                border: iced::Border {
                    radius: 6.0.into(),
                    ..Default::default()
                },
                ..base
            }
        })
        .on_press(Message::SwitchCategoryTab(idx))
        .into()
}

/// Colour a size by how much it is: small = normal, big = warm.
fn size_color(c: &ThemeColors, bytes: u64) -> Color {
    const GB: u64 = 1 << 30;
    const MB: u64 = 1 << 20;
    if bytes >= 5 * GB {
        c.red
    } else if bytes >= 500 * MB {
        c.yellow
    } else {
        c.text_primary
    }
}

/// Category title + description + bulk buttons (or the search summary), and
/// — on narrow windows — the category drop-down that replaces the sidebar.
fn category_header<'a>(
    state: &'a CleanSysGui,
    c: &ThemeColors,
    layout: Layout,
) -> Element<'a, Message> {
    let c = *c;
    let searching = !state.search.trim().is_empty();

    let mut col = column![].spacing(8);

    if layout == Layout::Narrow && !searching {
        #[derive(Clone, PartialEq, Eq)]
        struct Opt(usize, String);
        impl std::fmt::Display for Opt {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.1)
            }
        }
        let opts: Vec<Opt> = state
            .categories
            .iter()
            .enumerate()
            .filter(|(i, _)| state.category_visible(*i))
            .map(|(i, cat)| {
                let size = state
                    .category_bytes(i)
                    .filter(|b| *b > 0)
                    .map(|b| format!("  ·  {}", format_size(b)))
                    .unwrap_or_default();
                Opt(i, format!("{}{size}", cat.name))
            })
            .collect();
        let selected = opts.iter().find(|o| o.0 == state.active_tab).cloned();
        col = col.push(
            pick_list(opts, selected, |o: Opt| Message::SwitchCategoryTab(o.0))
                .text_size(14.0)
                .width(Length::Fill),
        );
    }

    let (title, subtitle) = if searching {
        let n = state.visible_items().len();
        (
            format!("Search results ({n})"),
            format!("Matching “{}” in every category", state.search.trim()),
        )
    } else if let Some(cat) = state.categories.get(state.active_tab) {
        (
            cat.name
                .trim_end_matches(cleansys_core::model::ROOT_SUFFIX)
                .to_string(),
            cat.description.clone(),
        )
    } else {
        (String::new(), String::new())
    };

    let cat_idx = state.active_tab;
    let bulk: Element<'a, Message> = if searching {
        Space::new().into()
    } else {
        row![
            button(text("All").size(12))
                .padding([4, 10])
                .style(button::secondary)
                .on_press(Message::SelectAllCategory(cat_idx)),
            button(text("None").size(12))
                .padding([4, 10])
                .style(button::secondary)
                .on_press(Message::DeselectAllCategory(cat_idx)),
        ]
        .spacing(6)
        .into()
    };

    col = col.push(
        row![
            column![
                text(title).size(20).color(c.text_primary),
                text(subtitle).size(12).color(c.muted),
            ]
            .spacing(2)
            .width(Length::Fill),
            bulk,
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    );
    col.into()
}

// ── Cleaner list ─────────────────────────────────────────────────────────────

fn item_list<'a>(state: &'a CleanSysGui, c: &ThemeColors, layout: Layout) -> Element<'a, Message> {
    let c = *c;
    let visible = state.visible_items();

    let body: Element<'a, Message> = if visible.is_empty() {
        let msg = if !state.search.trim().is_empty() {
            "No cleaner matches your search."
        } else if state.board.is_scanning() {
            "Scanning your system…"
        } else {
            "Nothing to clean here — this category is already tidy ✨"
        };
        container(text(msg).size(14).color(c.muted))
            .padding(30)
            .center_x(Length::Fill)
            .into()
    } else {
        let rows: Vec<Element<'a, Message>> = visible
            .into_iter()
            .map(|(ci, ii)| item_row(state, ci, ii, &c, layout))
            .collect();
        scrollable(column(rows).spacing(6).padding(iced::Padding {
            right: 12.0,
            ..Default::default()
        }))
        .height(Length::Fill)
        .into()
    };

    container(body)
        .padding(10)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(surface_style(c))
        .into()
}

fn item_row<'a>(
    state: &'a CleanSysGui,
    cat_idx: usize,
    item_idx: usize,
    c: &ThemeColors,
    layout: Layout,
) -> Element<'a, Message> {
    let c = *c;
    let item = &state.categories[cat_idx].items[item_idx];
    let info = state.scan_info(cat_idx, item_idx);

    let name_check = checkbox(item.selected)
        .label(item.name.clone())
        .size(17)
        .text_size(14)
        .on_toggle(move |_| Message::ToggleItem(cat_idx, item_idx));

    let mut badges = row![].spacing(6).align_y(Alignment::Center);
    match item.risk {
        cleansys_core::Risk::Safe => {}
        cleansys_core::Risk::Moderate => badges = badges.push(badge("MODERATE", c.yellow)),
        cleansys_core::Risk::Caution => badges = badges.push(badge("CAUTION", c.red)),
    }
    if item.requires_root {
        badges = badges.push(badge("ROOT", c.accent));
    }

    // Right-hand column: run status while/after running, else the scanned size.
    let right: Element<'a, Message> = match &item.status {
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
            text("queued").size(12).color(c.muted),
        ]
        .spacing(6)
        .align_y(Alignment::Center)
        .into(),
        None => match info {
            Some(s) if s.error.is_some() => text("scan failed").size(12).color(c.red).into(),
            Some(s) if s.bytes > 0 => column![
                text(format_size(s.bytes))
                    .size(16)
                    .color(size_color(&c, s.bytes)),
                text(format!("{} item(s)", s.items)).size(11).color(c.muted),
            ]
            .align_x(Alignment::End)
            .into(),
            Some(_) => text("nothing to clean").size(12).color(c.muted).into(),
            None => text(if state.board.is_scanning() {
                "scanning…"
            } else {
                ""
            })
            .size(12)
            .color(c.muted)
            .into(),
        },
    };

    let mut left = column![row![name_check, badges]
        .spacing(10)
        .align_y(Alignment::Center)]
    .spacing(3);
    left = left.push(text(item.description.clone()).size(12).color(c.muted));
    if let Some(top) = info.and_then(|s| s.top_path.as_ref()) {
        left = left.push(text(format!("📁 {top}")).size(11).color(c.text_secondary));
    }

    // Per-path breakdown of the last *real* run.
    if let Some(result) = &item.last_result {
        let mut sorted: Vec<_> = result.items.iter().collect();
        sorted.sort_by_key(|i| std::cmp::Reverse(i.size));
        for cleaned in sorted.iter().take(4) {
            left = left.push(
                text(format!(
                    "   • {} — {}",
                    cleaned.path_str(),
                    format_size(cleaned.size)
                ))
                .size(11)
                .color(c.text_secondary),
            );
        }
        if sorted.len() > 4 {
            left = left.push(
                text(format!("   … and {} more", sorted.len() - 4))
                    .size(11)
                    .color(c.muted),
            );
        }
    }

    let content: Element<'a, Message> = if layout == Layout::Narrow {
        column![left, right].spacing(6).into()
    } else {
        row![left.width(Length::Fill), right]
            .spacing(12)
            .align_y(Alignment::Center)
            .into()
    };

    let selected = item.selected;
    container(content)
        .padding([10, 12])
        .width(Length::Fill)
        .style(move |_t: &iced::Theme| container::Style {
            background: Some(
                if selected {
                    c.selection
                } else {
                    c.surface_highlight
                }
                .into(),
            ),
            border: iced::Border {
                radius: 8.0.into(),
                color: if selected {
                    c.accent
                } else {
                    Color::TRANSPARENT
                },
                width: if selected { 1.0 } else { 0.0 },
            },
            ..Default::default()
        })
        .into()
}

// ── Action bar (always visible) ──────────────────────────────────────────────

fn action_bar<'a>(state: &'a CleanSysGui, c: &ThemeColors, layout: Layout) -> Element<'a, Message> {
    let c = *c;
    let selected = state.selected_count();
    let busy = state.is_running || state.previewing;
    let scanning = state.board.is_scanning();
    let reclaim = state.selected_reclaimable();
    let compact = layout != Layout::Wide;

    // Summary / progress
    let summary: Element<'a, Message> = if state.is_running || state.previewing {
        let label = if state.previewing {
            "Previewing"
        } else {
            "Cleaning"
        };
        column![
            text(format!(
                "{label} {}/{}…",
                state.operations_completed, state.operations_total
            ))
            .size(13)
            .color(c.text_primary),
            iced::widget::progress_bar(0.0..=1.0, state.progress_fraction())
                .girth(Length::Fixed(6.0)),
        ]
        .spacing(4)
        .width(Length::Fill)
        .into()
    } else if scanning {
        let done = state.board.total - state.board.pending;
        column![
            text(format!(
                "Scanning your system… {done}/{}",
                state.board.total
            ))
            .size(13)
            .color(c.text_primary),
            iced::widget::progress_bar(
                0.0..=1.0,
                if state.board.total == 0 {
                    0.0
                } else {
                    done as f32 / state.board.total as f32
                }
            )
            .girth(Length::Fixed(6.0)),
        ]
        .spacing(4)
        .width(Length::Fill)
        .into()
    } else {
        let headline = if state.total_bytes_cleaned > 0 {
            format!("✓ Freed {}", format_size(state.total_bytes_cleaned))
        } else if selected == 0 {
            format!(
                "{} can be freed — tick cleaners or press Recommended",
                format_size(state.total_reclaimable())
            )
        } else {
            format!("{selected} selected · {} to free", format_size(reclaim))
        };
        column![
            text(headline)
                .size(14)
                .color(if state.total_bytes_cleaned > 0 {
                    c.green
                } else {
                    c.text_primary
                }),
            text(format!(
                "{} reclaimable in total",
                format_size(state.total_reclaimable())
            ))
            .size(11)
            .color(c.muted),
        ]
        .spacing(2)
        .width(Length::Fill)
        .into()
    };

    let clean_label = if state.is_running {
        "⏳ Cleaning…".to_string()
    } else if selected == 0 {
        "Select cleaners".to_string()
    } else if reclaim > 0 {
        format!("🧹 Clean {selected} · {}", format_size(reclaim))
    } else {
        format!("🧹 Clean {selected}")
    };
    let clean = button(text(clean_label).size(14))
        .padding([10, 18])
        .style(button::primary)
        .on_press_maybe((!busy && selected > 0).then_some(Message::RequestRun));

    // Small secondary button with a hover tooltip (essential when compact
    // layouts reduce the label to an icon).
    let small =
        |label: &str, tip: &'static str, msg: Message, enabled: bool| -> Element<'a, Message> {
            tooltip(
                button(text(label.to_string()).size(12))
                    .padding([6, 10])
                    .style(button::secondary)
                    .on_press_maybe(enabled.then_some(msg)),
                container(text(tip).size(12))
                    .padding(6)
                    .style(container::rounded_box),
                tooltip::Position::Top,
            )
            .into()
        };

    let recommended = small(
        if compact { "✨" } else { "✨ Recommended" },
        "Tick only the safe, user-land cleaners",
        Message::SelectRecommended,
        !busy,
    );
    let none = small(
        if compact { "☐" } else { "Select none" },
        "Untick everything",
        Message::DeselectAllEverywhere,
        !busy,
    );
    let rescan = small(
        if compact { "⟳" } else { "⟳ Rescan" },
        "Measure what every cleaner can free again",
        Message::ScanAll,
        !busy && !scanning,
    );
    let preview = small(
        if compact { "🔍" } else { "🔍 Preview" },
        "Show exactly what would be removed (deletes nothing)",
        Message::RequestPreview,
        !busy && selected > 0,
    );
    let hide = checkbox(state.hide_empty)
        .label(if compact {
            "Hide empty"
        } else {
            "Hide empty cleaners"
        })
        .size(15)
        .text_size(12)
        .on_toggle(|_| Message::ToggleHideEmpty);
    let log_btn = small(
        if state.show_log {
            "Activity ▾"
        } else {
            "Activity ▴"
        },
        "Show or hide the activity log",
        Message::ToggleLog,
        true,
    );

    let content: Element<'a, Message> = match layout {
        Layout::Wide => row![
            summary,
            hide,
            log_btn,
            rescan,
            recommended,
            none,
            preview,
            clean,
        ]
        .spacing(10)
        .align_y(Alignment::Center)
        .into(),
        Layout::Medium => column![
            row![summary, clean].spacing(12).align_y(Alignment::Center),
            row![
                hide,
                Space::new().width(Length::Fill),
                log_btn,
                rescan,
                recommended,
                none,
                preview
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ]
        .spacing(8)
        .into(),
        Layout::Narrow => column![
            summary,
            row![recommended, none, preview, rescan, log_btn].spacing(6),
            row![hide, Space::new().width(Length::Fill)].align_y(Alignment::Center),
            clean.width(Length::Fill),
        ]
        .spacing(8)
        .into(),
    };

    container(content)
        .padding([10, 14])
        .width(Length::Fill)
        .style(surface_style(c))
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
                text("Activity").size(13).color(c.text_primary),
                Space::new().width(Length::Fill),
                button(text("Clear").size(11))
                    .padding([3, 8])
                    .style(button::secondary)
                    .on_press(Message::ClearLog),
            ]
            .align_y(Alignment::Center),
            scrollable(column(log_lines).spacing(2)).height(Length::Fixed(120.0)),
        ]
        .spacing(6),
    )
    .padding(10)
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

/// A pick-list entry pairing a value with its label.
#[derive(Clone, PartialEq, Eq)]
struct Choice<T: Clone + PartialEq + Eq> {
    value: T,
    label: String,
}

impl<T: Clone + PartialEq + Eq> std::fmt::Display for Choice<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label)
    }
}

fn choice<T: Clone + PartialEq + Eq>(value: T, label: impl Into<String>) -> Choice<T> {
    Choice {
        value,
        label: label.into(),
    }
}

/// Dialog for configuring automatic (scheduled) cleaning.
fn schedule_dialog<'a>(state: &'a CleanSysGui, c: &ThemeColors) -> Element<'a, Message> {
    use cleansys_core::engine::schedule::{Backend, Frequency, Scope, WEEKDAYS};

    let c = *c;
    let d = &state.schedule_draft;

    let freq_opts = vec![
        choice(Frequency::Daily, "Daily"),
        choice(Frequency::Weekly, "Weekly"),
        choice(Frequency::Monthly, "Monthly"),
    ];
    let freq_sel = freq_opts.iter().find(|o| o.value == d.frequency).cloned();

    let hour_opts: Vec<_> = (0u8..24).map(|h| choice(h, format!("{h:02}"))).collect();
    let hour_sel = Some(choice(d.hour, format!("{:02}", d.hour)));

    let mut minute_opts: Vec<_> = (0u8..60)
        .step_by(5)
        .map(|m| choice(m, format!("{m:02}")))
        .collect();
    if !minute_opts.iter().any(|o| o.value == d.minute) {
        minute_opts.push(choice(d.minute, format!("{:02}", d.minute)));
        minute_opts.sort_by_key(|o| o.value);
    }
    let minute_sel = Some(choice(d.minute, format!("{:02}", d.minute)));

    let scope_opts = vec![
        choice(Scope::Recommended, "Recommended — safe caches only"),
        choice(Scope::Extended, "Extended — safe + moderate"),
        choice(
            Scope::Selected,
            "Selected — the cleaners ticked in the list",
        ),
    ];
    let scope_sel = scope_opts.iter().find(|o| o.value == d.scope).cloned();

    let mut form = column![row![
        text("Repeat")
            .size(13)
            .color(c.text_secondary)
            .width(Length::Fixed(110.0)),
        pick_list(freq_opts, freq_sel, |o: Choice<Frequency>| {
            Message::ScheduleFrequency(o.value)
        })
        .text_size(13.0)
        .width(Length::Fixed(160.0)),
    ]
    .spacing(10)
    .align_y(Alignment::Center),]
    .spacing(10);

    match d.frequency {
        Frequency::Weekly => {
            let opts: Vec<_> = WEEKDAYS
                .iter()
                .enumerate()
                .map(|(i, n)| choice(i as u8, *n))
                .collect();
            let sel = opts.iter().find(|o| o.value == d.weekday).cloned();
            form = form.push(
                row![
                    text("Weekday")
                        .size(13)
                        .color(c.text_secondary)
                        .width(Length::Fixed(110.0)),
                    pick_list(opts, sel, |o: Choice<u8>| Message::ScheduleWeekday(o.value))
                        .text_size(13.0)
                        .width(Length::Fixed(160.0)),
                ]
                .spacing(10)
                .align_y(Alignment::Center),
            );
        }
        Frequency::Monthly => {
            let opts: Vec<_> = (1u8..=28).map(|n| choice(n, n.to_string())).collect();
            let sel = Some(choice(d.day_of_month, d.day_of_month.to_string()));
            form = form.push(
                row![
                    text("Day of month")
                        .size(13)
                        .color(c.text_secondary)
                        .width(Length::Fixed(110.0)),
                    pick_list(opts, sel, |o: Choice<u8>| Message::ScheduleDayOfMonth(
                        o.value
                    ))
                    .text_size(13.0)
                    .width(Length::Fixed(160.0)),
                ]
                .spacing(10)
                .align_y(Alignment::Center),
            );
        }
        Frequency::Daily => {}
    }

    form = form
        .push(
            row![
                text("Time")
                    .size(13)
                    .color(c.text_secondary)
                    .width(Length::Fixed(110.0)),
                pick_list(hour_opts, hour_sel, |o: Choice<u8>| Message::ScheduleHour(
                    o.value
                ))
                .text_size(13.0)
                .width(Length::Fixed(80.0)),
                text(":").size(14).color(c.text_primary),
                pick_list(minute_opts, minute_sel, |o: Choice<u8>| {
                    Message::ScheduleMinute(o.value)
                })
                .text_size(13.0)
                .width(Length::Fixed(80.0)),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        )
        .push(
            row![
                text("Cleans")
                    .size(13)
                    .color(c.text_secondary)
                    .width(Length::Fixed(110.0)),
                pick_list(scope_opts, scope_sel, |o: Choice<Scope>| {
                    Message::ScheduleScope(o.value)
                })
                .text_size(13.0)
                .width(Length::Fixed(320.0)),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        );

    if cfg!(all(unix, not(target_os = "macos"))) {
        let opts = vec![
            choice(Backend::Auto, "Auto (systemd timer, else cron)"),
            choice(Backend::Systemd, "systemd user timer"),
            choice(Backend::Cron, "cron (crontab)"),
        ];
        let sel = opts.iter().find(|o| o.value == d.backend).cloned();
        form = form.push(
            row![
                text("Backend")
                    .size(13)
                    .color(c.text_secondary)
                    .width(Length::Fixed(110.0)),
                pick_list(opts, sel, |o: Choice<Backend>| Message::ScheduleBackend(
                    o.value
                ))
                .text_size(13.0)
                .width(Length::Fixed(260.0)),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        );
    }

    let status_line = match &state.schedule_installed {
        Some(b) => text(format!("Status: ACTIVE via {b} \u{2014} {}", d.describe()))
            .size(13)
            .color(c.green),
        None => text("Status: not scheduled").size(13).color(c.muted),
    };
    let last_run: Element<'a, Message> = match &state.schedule_last_run {
        Some(l) => text(format!(
            "Last run: {} \u{2014} freed {} ({} cleaners{})",
            l.ago(),
            format_size(l.bytes_freed),
            l.cleaners_run,
            if l.scheduled { ", scheduled" } else { "" }
        ))
        .size(12)
        .color(c.text_secondary)
        .into(),
        None => Space::new().height(Length::Fixed(0.0)).into(),
    };

    let enable_label = if state.schedule_installed.is_some() {
        "Update"
    } else {
        "Enable"
    };
    let buttons = row![
        button(text(enable_label))
            .padding([8, 16])
            .style(button::primary)
            .on_press(Message::ScheduleApply),
        button(text("Remove"))
            .padding([8, 16])
            .style(button::secondary)
            .on_press_maybe(
                state
                    .schedule_installed
                    .is_some()
                    .then_some(Message::ScheduleRemove)
            ),
        Space::new().width(Length::Fill),
        button(text("Close"))
            .padding([8, 16])
            .style(button::secondary)
            .on_press(Message::CloseSchedule),
    ]
    .spacing(10);

    let content = column![
        text("\u{23F0} Automatic cleaning").size(22).color(c.text_primary),
        status_line,
        last_run,
        rule::horizontal(1),
        form,
        rule::horizontal(1),
        text(format!("Will run: {}", d.describe())).size(13).color(c.text_primary),
        text("Unattended runs only touch user-land cleaners, skip apps that are open, and never include caution-risk cleaners unless you picked them.")
            .size(11)
            .color(c.muted),
        text(state.schedule_message.clone()).size(12).color(c.accent),
        buttons,
    ]
    .spacing(12)
    .padding(28)
    .max_width(560);

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
    fn view_root_fills_window_width() {
        // Regression test: main_content()'s column previously only set
        // `.height(Length::Fill)`, never `.width(...)`. iced's `Column`
        // defaults width to `Length::Shrink`, so the whole layout (activity
        // log included) only ever shrank to its natural content width
        // instead of stretching to fill the window on resize.
        let state = CleanSysGui::new();
        let c = state.colors();
        for w in [380.0, 600.0, 800.0, 1000.0, 1600.0] {
            let element = main_layout(&state, &c, w);
            let size = element.as_widget().size();
            assert_eq!(size.width, Length::Fill, "width {w}");
            assert_eq!(size.height, Length::Fill, "width {w}");
        }
    }

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

    #[test]
    fn view_does_not_panic_for_schedule_dialog_in_every_frequency() {
        use cleansys_core::engine::schedule::Frequency;
        for f in [Frequency::Daily, Frequency::Weekly, Frequency::Monthly] {
            let mut state = CleanSysGui::new();
            state.schedule_open = true;
            state.schedule_draft.frequency = f;
            state.schedule_draft.minute = 7; // not a 5-minute step
            let _ = view(&state);
        }
    }

    #[test]
    fn layout_breakpoints() {
        assert_eq!(Layout::for_width(1400.0), Layout::Wide);
        assert_eq!(Layout::for_width(980.0), Layout::Wide);
        assert_eq!(Layout::for_width(800.0), Layout::Medium);
        assert_eq!(Layout::for_width(699.0), Layout::Narrow);
        assert!(Layout::Narrow.sidebar_width().is_none());
        assert!(Layout::Wide.sidebar_width() > Layout::Medium.sidebar_width());
    }

    #[test]
    fn every_layout_renders_with_scan_results_search_and_log() {
        use cleansys_core::ScanInfo;
        let mut state = CleanSysGui::new();
        state.show_log = true;
        state.push_log("hello");
        state.board.start(3);
        state.board.record(
            0,
            0,
            ScanInfo {
                bytes: 6 << 30,
                items: 3,
                top_path: Some("/tmp/x".into()),
                error: None,
            },
        );
        state.board.record(
            0,
            1,
            ScanInfo {
                error: Some("boom".into()),
                ..Default::default()
            },
        );
        state.categories[0].items[0].selected = true;
        let c = state.colors();
        for w in [360.0, 640.0, 760.0, 1100.0] {
            let _ = main_layout(&state, &c, w);
        }
        state.search = "cache".into();
        for w in [360.0, 1100.0] {
            let _ = main_layout(&state, &c, w);
        }
    }
}
