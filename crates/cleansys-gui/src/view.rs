//! View (rendering) logic for the CleanSys Iced GUI.

use cleansys_core::{format_size, EntryState, Status};
use iced::widget::{
    button, canvas, checkbox, column, container, pick_list, responsive, row, rule, scrollable,
    text, text_input, tooltip, Space,
};
use iced::{Alignment, Color, Element, Length};

use crate::icons;
use crate::message::Message;
use crate::state::{CleanSysGui, SettingsTab};
use crate::theme::ThemeColors;
use crate::theme_selector::theme_selector;
use crate::window::WindowPreset;

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
    if state.settings_open {
        return settings_dialog(state, &c);
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
    // The badge shows the scope you are working in: ROOT while a system (root)
    // category is open or when the app itself runs as root, otherwise USER.
    let in_root_section = state.search.trim().is_empty()
        && state
            .categories
            .get(state.active_tab)
            .is_some_and(|cat| is_root_category(&cat.name, &cat.items));
    let root_badge = if state.is_root || in_root_section {
        badge("ROOT", c.red)
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

    let (settings_label, about_label) = if layout == Layout::Narrow {
        ("⚙", "ℹ")
    } else {
        ("⚙ Settings", "ℹ About")
    };
    let settings_btn = button(text(settings_label).size(13))
        .padding([8, 12])
        .style(button::secondary)
        .on_press_maybe(
            (!state.is_running).then_some(Message::OpenSettings(SettingsTab::Settings)),
        );
    let about_btn = button(text(about_label).size(13))
        .padding([8, 12])
        .style(button::secondary)
        .on_press(Message::OpenSettings(SettingsTab::About));

    let title = row![
        text("🧹 CleanSys")
            .size(if layout == Layout::Narrow { 20 } else { 24 })
            .color(c.text_primary),
        // The version is always visible.
        text(cleansys_core::appinfo::version_label())
            .size(12)
            .color(c.muted),
        root_badge
    ]
    .spacing(10)
    .align_y(Alignment::Center);

    let content: Element<'a, Message> = match layout {
        Layout::Narrow => column![
            row![
                title,
                Space::new().width(Length::Fill),
                schedule,
                settings_btn,
                about_btn
            ]
            .spacing(6)
            .align_y(Alignment::Center),
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
            settings_btn,
            about_btn,
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
        None if state.board.is_scanning() => ring(&c, state.anim_tick, 13.0),
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
            ring(&c, state.anim_tick, 16.0),
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
            Some(s) if s.bytes > 0 => {
                let mut col = column![
                    text(format_size(s.bytes))
                        .size(16)
                        .color(size_color(&c, s.bytes)),
                    text(format!("{} item(s)", s.items)).size(11).color(c.muted),
                ]
                .align_x(Alignment::End);
                if state.board.entry_state(cat_idx, item_idx) == EntryState::Partial {
                    col = col.push(
                        text(format!(
                            "{} selected",
                            format_size(state.board.item_selected_bytes(cat_idx, item_idx))
                        ))
                        .size(11)
                        .color(c.accent),
                    );
                }
                col.into()
            }
            Some(_) => text("nothing to clean").size(12).color(c.muted).into(),
            // Still being measured: a round progress ring, like the web UI's.
            None if state.board.is_scanning() => row![
                ring(&c, state.anim_tick, 16.0),
                text("measuring…").size(12).color(c.muted),
            ]
            .spacing(6)
            .align_y(Alignment::Center)
            .into(),
            None => Space::new().into(),
        },
    };

    let has_entries = info.is_some_and(|s| !s.entries.is_empty());
    let open = state.expanded.contains(&(cat_idx, item_idx));
    let mut name_row = row![name_check, badges]
        .spacing(10)
        .align_y(Alignment::Center);
    if has_entries {
        name_row = name_row.push(
            button(text(if open { "▾ Details" } else { "▸ Details" }).size(11))
                .padding([2, 8])
                .style(button::text)
                .on_press(Message::ToggleExpand(cat_idx, item_idx)),
        );
    }
    let mut left = column![name_row].spacing(3);
    left = left.push(text(item.description.clone()).size(12).color(c.muted));
    if let Some(top) = info.and_then(|s| s.top_path.as_ref()) {
        left = left.push(text(format!("📁 {top}")).size(11).color(c.text_secondary));
    }

    // Expanded details: every path this cleaner would remove, each tickable.
    if let (true, Some(scan)) = (open, info) {
        let any_skippable = scan.entries.iter().any(|e| e.skippable);
        let mut list = column![].spacing(2);
        if any_skippable {
            list = list.push(
                row![
                    button(text("Select all").size(11))
                        .padding([2, 8])
                        .style(button::secondary)
                        .on_press(Message::SetEntries(cat_idx, item_idx, true)),
                    button(text("Select none").size(11))
                        .padding([2, 8])
                        .style(button::secondary)
                        .on_press(Message::SetEntries(cat_idx, item_idx, false)),
                    text("ticked paths are cleaned when this cleaner runs")
                        .size(11)
                        .color(c.muted),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
            );
        } else {
            list = list.push(
                text("These are removed together — individual paths can't be unticked.")
                    .size(11)
                    .color(c.muted),
            );
        }
        for e in &scan.entries {
            let on = state.board.entry_selected(&e.path);
            let path = e.path.clone();
            let mut cb = checkbox(on)
                .label(format!("{} — {}", e.label, format_size(e.bytes)))
                .size(14)
                .text_size(12);
            if e.skippable {
                cb = cb.on_toggle(move |_| Message::ToggleEntry(path.clone()));
            }
            list = list.push(column![cb, text(e.path.clone()).size(10).color(c.muted)].spacing(0));
        }
        if scan.items > scan.entries.len() {
            list = list.push(
                text(format!(
                    "… and {} smaller item(s) not listed",
                    scan.items - scan.entries.len()
                ))
                .size(11)
                .color(c.muted),
            );
        }
        left = left.push(
            container(scrollable(list.padding([0, 10])).height(Length::Shrink))
                .max_height(320)
                .padding([4, 0]),
        );
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
            progress_label(
                &c,
                state.anim_tick,
                format!(
                    "{label} {}/{} · {}%",
                    state.operations_completed,
                    state.operations_total,
                    cleansys_core::anim::percent(
                        state.operations_completed,
                        state.operations_total
                    )
                )
            ),
            animated_bar(&c, state.progress_fraction(), state.anim_tick),
        ]
        .spacing(4)
        .width(Length::Fill)
        .into()
    } else if scanning {
        let done = state.board.total - state.board.pending;
        column![
            progress_label(
                &c,
                state.anim_tick,
                format!(
                    "Scanning your system… {done}/{} · {}%",
                    state.board.total,
                    cleansys_core::anim::percent(done, state.board.total)
                )
            ),
            animated_bar(
                &c,
                cleansys_core::anim::fraction(done, state.board.total),
                state.anim_tick
            ),
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
    } else if scanning {
        // A partial sum would mislead: say we are still measuring.
        format!("🧹 Clean {selected} · measuring…")
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
    let age_opts: Vec<Choice<u64>> = {
        let mut v: Vec<u64> = cleansys_core::engine::config::MIN_AGE_CHOICES.to_vec();
        if !v.contains(&state.min_age_days) {
            v.push(state.min_age_days);
            v.sort_unstable();
        }
        v.into_iter()
            .map(|d| choice(d, cleansys_core::engine::config::min_age_label(d)))
            .collect()
    };
    let age_sel = age_opts
        .iter()
        .find(|o| o.value == state.min_age_days)
        .cloned();
    let age: Element<'a, Message> = tooltip(
        row![
            text("Idle ≥").size(12).color(c.muted),
            pick_list(age_opts, age_sel, |o: Choice<u64>| Message::SetMinAge(o.value))
                .text_size(12)
                .padding([4, 8]),
        ]
        .spacing(6)
        .align_y(Alignment::Center),
        container(
            text("Project build output (target/, node_modules, …) is only offered when its project was untouched this long")
                .size(12),
        )
        .padding(6)
        .style(container::rounded_box),
        tooltip::Position::Top,
    )
    .into();
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
            age,
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
                age,
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
            row![hide, age, Space::new().width(Length::Fill)]
                .spacing(10)
                .align_y(Alignment::Center),
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
    .width(Length::Fill);

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
    // The card fills the window up to a comfortable maximum and is centred,
    // so dialogs adapt to narrow windows instead of overflowing.
    container(container(card).width(Length::Fill).max_width(620))
        .padding(16)
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
    .width(Length::Fill);

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
    .width(Length::Fill);

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
    .width(Length::Fill);

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
    .width(Length::Fill);

    let card = container(content).padding(8).style(surface_style(c));
    modal_backdrop(card.into(), c)
}

// ── Animated progress ─────────────────────────────────────────────────────────

/// A round progress ring: a faint track with an accent arc that turns and breathes, drawn on
/// a canvas. The GUI's counterpart of the web UI's `.spin` and the TUI's braille spinner.
struct Ring {
    tick: u32,
    color: Color,
    track: Color,
    stroke: f32,
}

impl canvas::Program<Message> for Ring {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        use std::f32::consts::TAU;
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let center = frame.center();
        let radius = (bounds.width.min(bounds.height) / 2.0 - self.stroke).max(1.0);
        frame.stroke(
            &canvas::Path::circle(center, radius),
            canvas::Stroke::default()
                .with_color(self.track)
                .with_width(self.stroke),
        );
        // One turn per second; the arc grows and shrinks between ~60° and ~240°.
        let (start, sweep) = cleansys_core::anim::ring_arc(self.tick, 20, 40);
        let arc = canvas::Path::new(|b| {
            b.arc(canvas::path::Arc {
                center,
                radius,
                start_angle: iced::Radians(start * TAU),
                end_angle: iced::Radians((start + sweep) * TAU),
            });
        });
        frame.stroke(
            &arc,
            canvas::Stroke::default()
                .with_color(self.color)
                .with_width(self.stroke)
                .with_line_cap(canvas::LineCap::Round),
        );
        vec![frame.into_geometry()]
    }
}

/// A round progress ring of `size` pixels, animated by `tick`.
fn ring<'a>(c: &ThemeColors, tick: u32, size: f32) -> Element<'a, Message> {
    canvas(Ring {
        tick,
        color: c.accent,
        track: c.surface_highlight,
        stroke: (size / 7.0).max(1.5),
    })
    .width(Length::Fixed(size))
    .height(Length::Fixed(size))
    .into()
}

/// Ring + label: the text line above a progress bar.
fn progress_label<'a>(c: &ThemeColors, tick: u32, label: String) -> Element<'a, Message> {
    row![
        ring(c, tick, 16.0),
        text(label).size(13).color(c.text_primary),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}

/// A progress bar whose filled part carries a highlight that sweeps along it — the same
/// look as the TUI's bar and the web's `.progress` (see `cleansys_core::anim`).
///
/// Built from fill-portions only (no canvas, no pixel maths): the filled part and the rest
/// share 1000 portions, and inside the filled part the highlight's offset changes with `tick`.
fn animated_bar<'a>(c: &ThemeColors, frac: f32, tick: u32) -> Element<'a, Message> {
    const TOTAL: u16 = 1000;
    const HIGHLIGHT: u16 = 260;
    let c = *c;
    let filled = ((frac.clamp(0.0, 1.0) * f32::from(TOTAL)).round() as u16).min(TOTAL);
    let seg = |portion: u16| Length::FillPortion(portion.max(1));
    let radius = 4.0;

    let filled_part: Element<'a, Message> = if filled == 0 {
        Space::new().into()
    } else {
        let lead = (cleansys_core::anim::sweep(tick, 20) * f32::from(TOTAL - HIGHLIGHT)) as u16;
        let sheen = container(Space::new())
            .width(seg(HIGHLIGHT))
            .height(Length::Fill)
            .style(move |_t: &iced::Theme| container::Style {
                background: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.38).into()),
                border: iced::Border {
                    radius: radius.into(),
                    ..Default::default()
                },
                ..Default::default()
            });
        container(row![
            Space::new().width(seg(lead)),
            sheen,
            Space::new().width(seg(TOTAL - HIGHLIGHT - lead)),
        ])
        .width(seg(filled))
        .height(Length::Fill)
        .style(move |_t: &iced::Theme| container::Style {
            background: Some(c.accent.into()),
            border: iced::Border {
                radius: radius.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
    };
    let rest: Element<'a, Message> = if filled >= TOTAL {
        Space::new().into()
    } else {
        Space::new().width(seg(TOTAL - filled)).into()
    };
    // A fully empty / full bar has one zero-width side: give it no portion at all.
    let bar = match filled {
        0 => row![Space::new().width(Length::Fill)],
        f if f >= TOTAL => row![filled_part],
        _ => row![filled_part, rest],
    };
    container(bar)
        .width(Length::Fill)
        .height(Length::Fixed(8.0))
        .clip(true)
        .style(move |_t: &iced::Theme| container::Style {
            background: Some(c.surface_highlight.into()),
            border: iced::Border {
                radius: radius.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

// ── Settings / About dialog ───────────────────────────────────────────────────

/// One labelled row of the settings form.
fn setting_row<'a>(
    c: &ThemeColors,
    label: &'a str,
    control: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    row![
        text(label)
            .size(13)
            .color(c.text_secondary)
            .width(Length::Fixed(230.0)),
        control.into(),
    ]
    .spacing(10)
    .align_y(Alignment::Center)
    .into()
}

fn settings_dialog<'a>(state: &'a CleanSysGui, c: &ThemeColors) -> Element<'a, Message> {
    let c = *c;
    let tab_btn = |label: &'static str, tab: SettingsTab| {
        button(text(label).size(13))
            .padding([6, 14])
            .style(if state.settings_tab == tab {
                button::primary
            } else {
                button::secondary
            })
            .on_press(Message::SettingsTabSelected(tab))
    };
    let tabs = row![
        tab_btn("⚙ Settings", SettingsTab::Settings),
        tab_btn("ℹ About", SettingsTab::About),
        Space::new().width(Length::Fill),
        text(cleansys_core::appinfo::title())
            .size(13)
            .color(c.muted),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let body: Element<'a, Message> = match state.settings_tab {
        SettingsTab::Settings => settings_tab_body(state, &c),
        SettingsTab::About => about_tab_body(&c),
    };

    let footer = row![
        text(state.settings_message.clone())
            .size(12)
            .color(c.accent)
            .width(Length::Fill),
        button(text("Close"))
            .padding([8, 16])
            .style(button::secondary)
            .on_press(Message::CloseSettings),
    ]
    .spacing(10)
    .align_y(Alignment::Center);

    let content = column![
        tabs,
        rule::horizontal(1),
        scrollable(body).height(Length::Fill),
        rule::horizontal(1),
        footer,
    ]
    .spacing(12)
    .padding(24)
    .width(Length::Fill)
    .height(Length::Fill);

    let card = container(content)
        .padding(8)
        .height(Length::Fill)
        .style(surface_style(c));
    modal_backdrop(card.into(), c)
}

fn settings_tab_body<'a>(state: &'a CleanSysGui, c: &ThemeColors) -> Element<'a, Message> {
    let c = *c;
    let cfg = &state.engine_cfg;

    let age_opts: Vec<Choice<u64>> = {
        let mut v: Vec<u64> = cleansys_core::engine::config::MIN_AGE_CHOICES.to_vec();
        if !v.contains(&cfg.min_age_days) {
            v.push(cfg.min_age_days);
            v.sort_unstable();
        }
        v.into_iter()
            .map(|d| choice(d, cleansys_core::engine::config::min_age_label(d)))
            .collect()
    };
    let age_sel = age_opts
        .iter()
        .find(|o| o.value == cfg.min_age_days)
        .cloned();
    let depth_opts: Vec<Choice<usize>> = {
        let mut v: Vec<usize> = cleansys_core::engine::config::MAX_DEPTH_CHOICES.to_vec();
        if !v.contains(&cfg.max_depth) {
            v.push(cfg.max_depth);
            v.sort_unstable();
        }
        v.into_iter().map(|d| choice(d, d.to_string())).collect()
    };
    let depth_sel = depth_opts
        .iter()
        .find(|o| o.value == cfg.max_depth)
        .cloned();

    let heading = |t: &'static str| text(t).size(15).color(c.text_primary);

    // Window: presets, maximise, full screen, and the shortcut cheat sheet.
    let preset_opts: Vec<Choice<WindowPreset>> = WindowPreset::ALL
        .into_iter()
        .map(|p| {
            let (w, h) = p.size();
            choice(p, format!("{} — {w:.0}×{h:.0}", p.label()))
        })
        .collect();
    let preset_sel = WindowPreset::matching(state.window_size)
        .and_then(|p| preset_opts.iter().find(|o| o.value == p).cloned());
    let small = |label: &'static str, msg: Message| {
        button(text(label).size(12))
            .padding([6, 12])
            .style(button::secondary)
            .on_press(msg)
    };
    let window_section = column![
        heading("Window"),
        setting_row(
            &c,
            "Size",
            row![
                pick_list(preset_opts, preset_sel, |o: Choice<WindowPreset>| {
                    Message::WindowPreset(o.value)
                })
                .placeholder(format!(
                    "{:.0} × {:.0}",
                    state.window_size.0, state.window_size.1
                ))
                .text_size(13.0)
                .width(Length::Fixed(230.0)),
                small(
                    "Smaller",
                    Message::WindowScale(1.0 / crate::window::GROW_FACTOR)
                ),
                small("Larger", Message::WindowScale(crate::window::GROW_FACTOR)),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ),
        setting_row(
            &c,
            "",
            row![
                small("Maximise", Message::ToggleMaximize),
                small(
                    if state.fullscreen {
                        "Leave full screen"
                    } else {
                        "Full screen"
                    },
                    Message::ToggleFullscreen,
                ),
                small("Reset", Message::WindowReset),
            ]
            .spacing(8),
        ),
        text(format!(
            "Shortcuts: {}. The size is remembered for next time.",
            crate::window::shortcut_hint()
        ))
        .size(11)
        .color(c.muted),
    ]
    .spacing(10);

    let mut col = column![
        window_section,
        heading("Scanning"),
        setting_row(
            &c,
            "Project idle for at least",
            pick_list(age_opts, age_sel, |o: Choice<u64>| Message::SetMinAge(o.value))
                .text_size(13.0)
                .width(Length::Fixed(160.0)),
        ),
        setting_row(
            &c,
            "Scan depth (folders below a root)",
            pick_list(depth_opts, depth_sel, |o: Choice<usize>| Message::SetMaxDepth(
                o.value
            ))
            .text_size(13.0)
            .width(Length::Fixed(100.0)),
        ),
        text("Build output (target/, node_modules, …) is only offered once its project has been untouched this long.")
            .size(11)
            .color(c.muted),
        heading("Behaviour"),
        checkbox(state.hide_empty)
            .label("Hide cleaners with nothing to clean")
            .size(15)
            .text_size(13)
            .on_toggle(|_| Message::ToggleHideEmpty),
        checkbox(state.confirm_before_run)
            .label("Ask before cleaning")
            .size(15)
            .text_size(13)
            .on_toggle(|_| Message::ToggleConfirm),
        heading("Project folders scanned for build output"),
    ]
    .spacing(10);

    for (i, root) in cfg.roots_for_editing().into_iter().enumerate() {
        col = col.push(
            row![
                text(root)
                    .size(13)
                    .color(c.text_primary)
                    .width(Length::Fill),
                button(text("Remove").size(12))
                    .padding([4, 10])
                    .style(button::secondary)
                    .on_press(Message::RemoveRoot(i)),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        );
    }
    col = col.push(
        row![
            text_input("Add a folder, e.g. ~/code", &state.new_root)
                .on_input(Message::NewRootChanged)
                .on_submit(Message::AddRoot)
                .padding(6)
                .size(13)
                .width(Length::Fill),
            button(text("Add").size(12))
                .padding([6, 14])
                .style(button::primary)
                .on_press(Message::AddRoot),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
    );

    col = col.push(heading("Never delete (glob patterns)"));
    for (i, pat) in cfg.exclude.iter().enumerate() {
        col = col.push(
            row![
                text(pat.clone())
                    .size(13)
                    .color(c.text_primary)
                    .width(Length::Fill),
                button(text("Remove").size(12))
                    .padding([4, 10])
                    .style(button::secondary)
                    .on_press(Message::RemoveExclude(i)),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        );
    }
    col = col.push(
        row![
            text_input(
                "Add a path or pattern, e.g. ~/work/keep/**",
                &state.new_exclude
            )
            .on_input(Message::NewExcludeChanged)
            .on_submit(Message::AddExclude)
            .padding(6)
            .size(13)
            .width(Length::Fill),
            button(text("Add").size(12))
                .padding([6, 14])
                .style(button::primary)
                .on_press(Message::AddExclude),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
    );
    col.padding(iced::Padding {
        right: 12.0,
        ..Default::default()
    })
    .into()
}

fn about_tab_body<'a>(c: &ThemeColors) -> Element<'a, Message> {
    let c = *c;
    let mut col = column![
        text(format!("🧹 {}", cleansys_core::appinfo::title()))
            .size(26)
            .color(c.text_primary),
        text(cleansys_core::appinfo::TAGLINE)
            .size(13)
            .color(c.muted),
    ]
    .spacing(6);
    col = col.push(Space::new().height(Length::Fixed(8.0)));
    for (label, value) in cleansys_core::appinfo::about_rows() {
        let control: Element<'a, Message> = if cleansys_core::appinfo::is_link(&value) {
            button(text(value.clone()).size(13))
                .padding([2, 8])
                .style(button::text)
                .on_press(Message::OpenUrl(value))
                .into()
        } else {
            text(value).size(13).color(c.text_primary).into()
        };
        col = col.push(
            row![
                text(label)
                    .size(13)
                    .color(c.text_secondary)
                    .width(Length::Fixed(120.0)),
                control,
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        );
    }
    col.into()
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
    fn view_does_not_panic_for_settings_and_about_dialogs() {
        use crate::state::SettingsTab;
        for tab in [SettingsTab::Settings, SettingsTab::About] {
            let mut state = CleanSysGui::new();
            state.engine_cfg.scan_roots = vec!["/work/a".into()];
            state.engine_cfg.exclude = vec!["~/keep/**".into()];
            state.engine_cfg.max_depth = 7; // not one of the fixed steps
            state.engine_cfg.min_age_days = 10; // likewise
            state.settings_message = "hello".into();
            state.open_settings(tab);
            let _ = view(&state);
        }
    }

    #[test]
    fn view_does_not_panic_while_scanning_cleaning_or_previewing_at_every_progress() {
        for (done, total) in [(0usize, 0usize), (0, 4), (1, 4), (3, 4), (4, 4)] {
            for tick in [0u32, 7, 19, u32::MAX] {
                // cleaning
                let mut state = CleanSysGui::new();
                state.anim_tick = tick;
                state.is_running = true;
                state.operations_total = total;
                state.operations_completed = done;
                let _ = view(&state);
                // previewing
                state.is_running = false;
                state.previewing = true;
                let _ = view(&state);
                // scanning
                state.previewing = false;
                state.board.start(total);
                for _ in 0..done {
                    state.board.record(0, 0, cleansys_core::ScanInfo::default());
                }
                let _ = view(&state);
            }
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
                entries: Vec::new(),
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
