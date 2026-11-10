use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap},
};
// Using tui-checkbox library for consistent checkbox symbols across the application
use tui_checkbox::symbols as checkbox_symbols;

use crate::app::{App, InfoTab, InputKind, SettingsRow};
use cleansys_core::{Status, format_size};

pub fn ui(f: &mut Frame, app: &mut App) {
    // Start every frame from a clean slate so nothing from the previous view
    // (title text, progress panels) bleeds through when views switch.
    f.render_widget(Clear, f.area());
    crate::theme::set_current(std::sync::Arc::clone(&app.theme));
    // Paint the theme's background and default text colour over the whole screen.
    f.render_widget(
        Block::default().style(
            Style::default()
                .bg(crate::theme::bg(&app.theme))
                .fg(app.theme.foreground),
        ),
        f.area(),
    );

    // Adjust title and footer heights based on terminal size
    // Use the frame's real size, not the cached one (it lags behind until a resize event).
    app.terminal_width = f.area().width;
    app.terminal_height = f.area().height;
    let (title_height, footer_height, min_content_height) = if app.terminal_height < 20 {
        // Very small terminals: minimal UI
        (2, 2, 6)
    } else if app.terminal_height < 30 {
        // Small terminals: compact UI
        (2, 2, 8)
    } else if app.terminal_height < 40 {
        // Medium terminals: standard UI
        (3, 4, 10)
    } else {
        // Large terminals: spacious UI
        (3, 4, 12)
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(title_height),    // Title
            Constraint::Min(min_content_height), // Main content
            Constraint::Length(footer_height),   // Footer
        ])
        .split(f.area());

    render_title(f, app, chunks[0]);

    if app.show_help {
        app.help_scroll = render_help(f, chunks[1], app.help_scroll);
    } else {
        render_main_content(f, app, chunks[1]);
    }

    render_footer(f, app, chunks[2]);

    // Render password prompt as overlay if visible
    if app.password_prompt.is_visible() {
        app.password_prompt.render(f, f.area());
    }

    if app.needs_admin_notice {
        render_admin_notice(f, f.area());
    } else if app.awaiting_run_confirmation {
        render_confirm_run(f, app, f.area());
    } else if app.info_open {
        render_info(f, app, f.area());
    } else if app.schedule_open {
        render_schedule(f, app, f.area());
    } else if app.theme_picker_open {
        render_theme_picker(f, app, f.area());
    } else if app.preview_open {
        render_preview(f, app, f.area());
    } else if app.report_open {
        crate::charts::render_report(f, app, f.area());
    } else if let Some(u) = &app.usage {
        crate::usage_view::render(f, u, f.area());
    } else if app.dashboard_open {
        crate::charts::render_dashboard(f, app, f.area());
    } else if app.detail_open {
        crate::charts::render_detail(f, app, f.area());
    }
}

/// Dark text for badges / buttons drawn on a coloured background.
fn on_accent() -> Color {
    match crate::theme::bg(&crate::theme::th()) {
        Color::Reset => Color::Black,
        c => c,
    }
}

/// A small filled label (` MODERATE `, ` USER `) like the GUI's badges.
fn badge(text: &str, bg: Color) -> Span<'static> {
    Span::styled(
        format!(" {text} "),
        Style::default()
            .fg(on_accent())
            .bg(bg)
            .add_modifier(Modifier::BOLD),
    )
}

/// `key label` pairs as spans: bold coloured key, muted label.
fn key_hints(pairs: &[(&str, &str, Color)]) -> Vec<Span<'static>> {
    let mut v = Vec::new();
    for (i, (k, label, color)) in pairs.iter().enumerate() {
        if i > 0 {
            v.push(Span::styled(
                "  ·  ",
                Style::default().fg(crate::theme::th().muted),
            ));
        }
        v.push(Span::styled(
            (*k).to_string(),
            Style::default().fg(*color).add_modifier(Modifier::BOLD),
        ));
        v.push(Span::styled(
            format!(" {label}"),
            Style::default().fg(crate::theme::th().muted),
        ));
    }
    v
}

/// The header bar, like the GUI's: brand + scope badge on the left, search field in the
/// middle and the global keys on the right.
fn render_title(f: &mut Frame, app: &App, area: Rect) {
    let t = crate::theme::th();
    let boxed = area.height >= 3;
    let block = Block::default()
        .borders(if boxed { Borders::ALL } else { Borders::BOTTOM })
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(t.border));
    let inner = block.inner(area);
    f.render_widget(block, area);
    if inner.height == 0 || inner.width < 10 {
        return;
    }

    let mut left = vec![
        Span::raw(" "),
        Span::styled(
            "CleanSys",
            Style::default().fg(t.accent).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        if app.is_root {
            badge("ROOT", t.error)
        } else {
            badge("USER", t.secondary)
        },
    ];
    if inner.width >= 60 {
        left.push(Span::styled(
            format!(" {}", cleansys_core::appinfo::version_label()),
            Style::default().fg(t.muted),
        ));
    }
    if app.is_running {
        left.push(Span::styled(
            format!("  {} RUNNING", cleansys_core::anim::spinner(app.anim_tick)),
            Style::default().fg(t.warning).add_modifier(Modifier::BOLD),
        ));
    }

    let right = if inner.width >= 100 {
        key_hints(&[
            ("?", "help", t.accent),
            ("o", "settings", t.accent),
            ("i", "about", t.accent),
            ("q", "quit", t.error),
        ])
    } else {
        key_hints(&[("?", "help", t.accent), ("q", "quit", t.error)])
    };
    // Not enough room for both: shrink the key hints, then clip the brand part.
    let left_full = Line::from(left.clone()).width();
    let mut right = right;
    if left_full + Line::from(right.clone()).width() + 2 > inner.width as usize {
        right = key_hints(&[("?", "", t.accent), ("q", "", t.error)]);
    }
    let right_w = Line::from(right.clone()).width() as u16 + 1;
    let left = clip_spans(
        left,
        (inner.width as usize).saturating_sub(right_w as usize + 1),
    );
    let left_w = Line::from(left.clone()).width() as u16;

    // The search field sits between the two when there is room.
    let room = inner.width.saturating_sub(left_w + right_w + 2);
    if room >= 18 {
        let searching = app.filter_active || !app.filter.is_empty();
        let text = if searching {
            format!(
                "/ {}{}",
                app.filter,
                if app.filter_active { "▏" } else { "" }
            )
        } else {
            "/ search cleaners…".to_string()
        };
        let style = if searching {
            Style::default().fg(t.foreground)
        } else {
            Style::default().fg(t.muted)
        };
        let field = Rect {
            x: inner.x + left_w + 2,
            y: inner.y,
            width: room.saturating_sub(1),
            height: 1,
        };
        let mut shown = if searching {
            shorten_left(&text, room as usize - 1)
        } else {
            clip_str(&text, room as usize - 1)
        };
        shown.push(' ');
        f.render_widget(
            Paragraph::new(Span::styled(
                shown,
                style.bg(t.surface.unwrap_or(t.selection_bg)),
            )),
            field,
        );
    }
    f.render_widget(Paragraph::new(Line::from(left)), inner);
    f.render_widget(
        Paragraph::new(Line::from(right)).alignment(ratatui::layout::Alignment::Right),
        inner,
    );
}

/// One `key  label  value` row of the options panel.
fn option_row(key: &str, label: &str, value: Span<'static>, width: usize) -> Line<'static> {
    let t = crate::theme::th();
    let used = text_width(key).max(4) + 1 + text_width(label) + value.width();
    let gap = width.saturating_sub(used).max(1);
    Line::from(vec![
        Span::styled(
            format!("{key:<4}"),
            Style::default().fg(t.accent).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(label.to_string(), Style::default().fg(t.foreground)),
        Span::raw(" ".repeat(gap)),
        value,
    ])
}

/// On/off value in the theme's success / muted colours.
fn onoff(on: bool) -> Span<'static> {
    let t = crate::theme::th();
    if on {
        Span::styled("● on", Style::default().fg(t.success))
    } else {
        Span::styled("○ off", Style::default().fg(t.muted))
    }
}

/// The live options side panel (`O`): grouped cells, every key works as shown.
fn render_options(f: &mut Frame, app: &App, area: Rect) {
    let t = crate::theme::th();
    let outer = panel(" ⚙ Options ").title_bottom(Line::from(" Shift+O close ").right_aligned());
    let inner = outer.inner(area);
    f.render_widget(outer, area);
    let w = inner.width.saturating_sub(2) as usize;
    let val = |s: String| Span::styled(s, Style::default().fg(t.secondary));
    let theme_name = if app.follow_system {
        format!(
            "System · {}",
            crate::theme::picker_name(app.theme_index + 1)
        )
    } else {
        crate::theme::picker_name(app.theme_index + 1).to_string()
    };
    let groups: Vec<(&str, Vec<Line<'static>>)> = vec![
        (
            "View",
            vec![
                option_row("e", "hide empty", onoff(app.hide_empty), w),
                option_row("L", "activity log", onoff(app.show_log), w),
                option_row("D", "storage chart", val("›".into()), w),
            ],
        ),
        (
            "Cleaning",
            vec![
                option_row("y", "confirm run", onoff(app.confirmation_mode), w),
                option_row(
                    "[ ]",
                    "idle days",
                    val(cleansys_core::engine::config::min_age_label(app.min_age_days).to_string()),
                    w,
                ),
                option_row("r", "recommended", val("›".into()), w),
            ],
        ),
        ("Theme", vec![option_row("T", "theme", val(theme_name), w)]),
        (
            "More",
            vec![
                option_row("o", "settings", val("›".into()), w),
                option_row("S", "schedule", val("›".into()), w),
                option_row("i", "about", val("›".into()), w),
            ],
        ),
    ];
    let mut y = inner.y;
    for (name, lines) in groups {
        let h = lines.len() as u16 + 2;
        if y + 1 + h > inner.y + inner.height {
            break;
        }
        f.render_widget(
            Paragraph::new(Span::styled(
                format!(" {name}"),
                Style::default().fg(t.muted).add_modifier(Modifier::BOLD),
            )),
            Rect::new(inner.x, y, inner.width, 1),
        );
        let cell = Rect::new(inner.x, y + 1, inner.width, h);
        f.render_widget(
            Paragraph::new(lines).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(t.border)),
            ),
            cell,
        );
        y += 1 + h;
    }
}

fn render_main_content(f: &mut Frame, app: &mut App, area: Rect) {
    // Responsive: a sidebar of categories on wide terminals; on narrow ones the
    // sidebar is replaced by a one-line category bar (Tab / Shift+Tab to switch).
    // The run panel (progress while cleaning, summary afterwards, activity log) sits
    // under the list, so the main view never changes while a clean runs.
    let panel_h = run_panel_height(app, area.height);
    let (area, panel_area) = if panel_h > 0 {
        let split = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(6), Constraint::Length(panel_h)])
            .split(area);
        (split[0], Some(split[1]))
    } else {
        (area, None)
    };
    let area = if app.options_open && area.width >= 70 {
        let split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(40), Constraint::Length(34)])
            .split(area);
        render_options(f, app, split[1]);
        split[0]
    } else {
        area
    };
    let use_sidebar = area.width >= 90;
    let content = if use_sidebar {
        let sidebar_w = (area.width * 30 / 100).clamp(30, 44).min(area.width / 2);
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(sidebar_w), Constraint::Min(30)])
            .split(area);
        // Tall sidebars also get a treemap of reclaimable space per category.
        let treemap_h = 10;
        if chunks[0].height >= 24 && category_sizes(app).len() > 1 {
            let side = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(8), Constraint::Length(treemap_h)])
                .split(chunks[0]);
            render_categories(f, app, side[0]);
            render_space_treemap(f, app, side[1]);
        } else {
            render_categories(f, app, chunks[0]);
        }
        chunks[1]
    } else {
        area
    };

    render_cleaner_pane(f, app, content, !use_sidebar);
    if let Some(panel) = panel_area {
        render_run_panel(f, app, panel);
    }
}

/// Rows of the status part of the run panel (progress / summary box, or a one-line notice).
fn status_rows(app: &App) -> u16 {
    if app.is_running || app.run_summary.is_some() {
        3
    } else {
        u16::from(app.notice.is_some())
    }
}

/// Rows of the activity log (only on tall enough terminals).
fn log_rows(app: &App, avail: u16) -> u16 {
    if app.show_log && avail >= 22 {
        // Taller while it has the focus, so there is something to scroll through.
        if app.focus == crate::app::Focus::Activity && avail >= 30 {
            14
        } else {
            7
        }
    } else {
        0
    }
}

/// Rows the run panel needs (0 = nothing to show).
fn run_panel_height(app: &App, avail: u16) -> u16 {
    (status_rows(app) + log_rows(app, avail)).min(avail.saturating_sub(8))
}

/// Progress while cleaning, the outcome afterwards, a one-line notice, and the
/// activity log — the TUI counterpart of the GUI action bar / the web progress panel.
fn render_run_panel(f: &mut Frame, app: &App, area: Rect) {
    let top_h = status_rows(app).min(area.height);
    let log_h = area.height - top_h;
    let top = Rect {
        height: top_h,
        ..area
    };
    let log = Rect {
        y: area.y + top_h,
        height: log_h,
        ..area
    };

    if top_h > 0 {
        if app.is_running {
            render_progress_bar(f, app, top);
        } else if let Some(sum) = &app.run_summary {
            render_summary(f, sum, top);
        } else if let Some(msg) = &app.notice {
            f.render_widget(
                Paragraph::new(Span::styled(
                    msg.clone(),
                    Style::default().fg(crate::theme::th().warning),
                )),
                top,
            );
        }
    }
    if log_h >= 3 {
        render_activity_log(f, app, log);
    }
}

/// A text progress bar: filled `█` with a highlight sweeping along it, empty `░`.
/// Same look as the GUI's bar and the web's `.progress` (see `cleansys_core::anim`).
fn animated_bar(width: usize, frac: f32, tick: u32) -> Vec<Span<'static>> {
    let (filled, (hl_start, hl_len)) = cleansys_core::anim::bar_cells(width, frac, tick, 16, 4);
    let base = Style::default().fg(crate::theme::th().success);
    let hot = Style::default()
        .fg(tui_nuances::color::lighten(
            crate::theme::th().success,
            0.45,
        ))
        .add_modifier(Modifier::BOLD);
    let empty = Style::default().fg(crate::theme::th().muted);
    vec![
        Span::styled("█".repeat(hl_start), base),
        Span::styled("█".repeat(hl_len), hot),
        Span::styled("█".repeat(filled - hl_start - hl_len), base),
        Span::styled("░".repeat(width - filled), empty),
    ]
}

fn render_progress_bar(f: &mut Frame, app: &App, area: Rect) {
    let ratio = cleansys_core::anim::fraction(app.run_done, app.run_total);
    let title = format!(
        " Cleaning {}/{} · freed {} · {} ",
        app.run_done,
        app.run_total,
        format_size(app.total_bytes_cleaned),
        app.get_elapsed_time()
    );
    let block = Block::default()
        .title(title)
        .title_bottom(Line::from(" q / Esc: cancel · L: log ").right_aligned())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(crate::theme::th().success));
    let inner = block.inner(area);
    f.render_widget(block, area);
    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let label = match &app.run_current {
        Some(name) => format!("running {name}"),
        None => "starting…".to_string(),
    };
    let pct = cleansys_core::anim::percent(app.run_done, app.run_total);
    let spin = cleansys_core::anim::spinner(app.anim_tick);
    let text = format!(" {pct:>3}% · {label}");
    let text: String = text.chars().take((inner.width as usize) / 2).collect();
    // spinner + space, the bar, then the text
    let bar_w = (inner.width as usize).saturating_sub(2 + text.chars().count());
    let mut spans = vec![Span::styled(
        format!("{spin} "),
        Style::default().fg(crate::theme::th().accent),
    )];
    spans.extend(animated_bar(bar_w, ratio, app.anim_tick));
    spans.push(Span::raw(text));
    f.render_widget(Paragraph::new(Line::from(spans)), inner);
}

fn render_summary(f: &mut Frame, sum: &crate::app::RunSummary, area: Rect) {
    let (icon, color) = if sum.cancelled {
        ("⏹", crate::theme::th().warning)
    } else if sum.failed > 0 {
        ("⚠", crate::theme::th().warning)
    } else {
        ("✓", crate::theme::th().success)
    };
    let mut spans = vec![
        Span::styled(
            format!("{icon} Freed {}", format_size(sum.freed)),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!(" across {} cleaner(s)", sum.ok)),
    ];
    if sum.failed > 0 {
        spans.push(Span::styled(
            format!(" · {} failed/cancelled", sum.failed),
            Style::default().fg(crate::theme::th().error),
        ));
    }
    spans.push(Span::styled(
        format!(" · {}", sum.elapsed),
        Style::default().fg(crate::theme::th().muted),
    ));
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color))
        .title(if sum.cancelled {
            " Cancelled "
        } else {
            " Done "
        })
        .title_bottom(Line::from(" P: report · L: log · Esc: dismiss ").right_aligned());
    f.render_widget(Paragraph::new(Line::from(spans)).block(block), area);
}

fn render_activity_log(f: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == crate::app::Focus::Activity;
    let total = app.operation_logs.len();
    let rows = area.height.saturating_sub(2) as usize;
    let scroll = app.log_scroll.min(total.saturating_sub(rows));
    let end = total - scroll;
    let start = end.saturating_sub(rows);
    let title = if scroll > 0 {
        format!(" Activity · {}-{end}/{total} ↑ ", start + 1)
    } else {
        " Activity ".to_string()
    };
    let hint = if focused {
        " ↑/↓ PgUp/PgDn scroll · End latest · Tab next "
    } else {
        " L: focus "
    };
    let block = focus_panel(title, focused).title_bottom(Line::from(hint).right_aligned());
    let inner = block.inner(area);
    let width = inner.width as usize;
    let lines: Vec<Line> = app.operation_logs[start..end]
        .iter()
        .map(|l| {
            let color = if l.starts_with('✗') {
                crate::theme::th().error
            } else if l.starts_with('✓') {
                crate::theme::th().success
            } else {
                crate::theme::th().secondary
            };
            let text = clip_str(l, width);
            Line::from(Span::styled(text, Style::default().fg(color)))
        })
        .collect();
    let body = if lines.is_empty() {
        vec![Line::from(Span::styled(
            "No activity yet.",
            Style::default().fg(crate::theme::th().muted),
        ))]
    } else {
        lines
    };
    f.render_widget(Paragraph::new(body).block(block), area);
}

/// Colour a size by how much it is: small = normal, big = warm.
fn size_style(bytes: u64) -> Style {
    const GB: u64 = 1 << 30;
    const MB: u64 = 1 << 20;
    if bytes >= 5 * GB {
        Style::default()
            .fg(crate::theme::th().error)
            .add_modifier(Modifier::BOLD)
    } else if bytes >= 500 * MB {
        Style::default().fg(crate::theme::th().warning)
    } else {
        Style::default().fg(crate::theme::th().foreground)
    }
}

/// One animated spinner glyph, as spans that can be dropped
/// into any `Line` (the widget implements `Into<Text>`).
fn spinner_spans(app: &App) -> Vec<Span<'static>> {
    // The same braille spinner the GUI shows (cleansys_core::anim).
    vec![Span::styled(
        cleansys_core::anim::spinner(app.anim_tick),
        Style::default().fg(crate::theme::th().accent),
    )]
}

/// ` (X to free)` for the footer — a spinner instead of numbers while the scan is still
/// running (the partial sum would be misleading).
fn to_free_spans(app: &App) -> Vec<Span<'static>> {
    if app.board.is_scanning() {
        let mut v = vec![Span::styled(
            " (",
            Style::default().fg(crate::theme::th().muted),
        )];
        v.extend(spinner_spans(app));
        v.push(Span::styled(
            " measuring…)",
            Style::default().fg(crate::theme::th().muted),
        ));
        return v;
    }
    let b = app.board.selected_bytes(&app.categories);
    if b > 0 {
        vec![Span::styled(
            format!(" ({} to free)", format_size(b)),
            Style::default().fg(crate::theme::th().success),
        )]
    } else {
        Vec::new()
    }
}

/// `left` spans, then `right` pushed against the right edge of `width` columns.
fn row_line<'a>(left: Vec<Span<'a>>, right: Span<'a>, width: usize) -> Line<'a> {
    row_line_spans(left, vec![right], width)
}

/// Like [`row_line`] with several spans on the right. The right part always stays whole;
/// the left part is clipped (with `…`) to what is left, so a row never overflows `width`.
fn row_line_spans<'a>(left: Vec<Span<'a>>, right: Vec<Span<'a>>, width: usize) -> Line<'a> {
    let right_w: usize = right.iter().map(Span::width).sum();
    let room = width.saturating_sub(right_w + 1);
    let mut left = clip_spans(left, room);
    let used = left.iter().map(Span::width).sum::<usize>() + right_w;
    left.push(Span::raw(" ".repeat(width.saturating_sub(used).max(1))));
    left.extend(right);
    Line::from(left)
}

/// Display width of `s` in terminal columns.
fn text_width(s: &str) -> usize {
    unicode_width::UnicodeWidthStr::width(s)
}

/// `s` cut to at most `max` columns, ending in `…` when something was dropped.
fn clip_str(s: &str, max: usize) -> String {
    if text_width(s) <= max {
        return s.to_string();
    }
    if max == 0 {
        return String::new();
    }
    let mut out = String::new();
    let mut w = 0;
    for c in s.chars() {
        let cw = unicode_width::UnicodeWidthChar::width(c).unwrap_or(0);
        if w + cw > max - 1 {
            break;
        }
        out.push(c);
        w += cw;
    }
    out.push('…');
    out
}

/// Spans cut to at most `max` columns in total; the span that does not fit ends in `…`.
fn clip_spans<'a>(spans: Vec<Span<'a>>, max: usize) -> Vec<Span<'a>> {
    let total: usize = spans.iter().map(Span::width).sum();
    if total <= max {
        return spans;
    }
    let mut out = Vec::new();
    let mut left = max;
    for sp in spans {
        let w = sp.width();
        if w <= left {
            left -= w;
            out.push(sp);
            continue;
        }
        if left > 0 {
            out.push(Span::styled(clip_str(&sp.content, left), sp.style));
        }
        break;
    }
    out
}

/// Key hints packed into at most `rows` lines of `width` columns. Hints are in priority
/// order: whatever does not fit is dropped (from the end), never cut in half.
fn fit_hints(pairs: &[(&str, &str, Color)], width: usize, rows: usize) -> Vec<Line<'static>> {
    const SEP: usize = 5; // "  ·  "
    let mut lines: Vec<Vec<(&str, &str, Color)>> = vec![Vec::new()];
    let mut used = 0usize;
    for &(k, l, c) in pairs {
        let w = text_width(k) + if l.is_empty() { 0 } else { 1 + text_width(l) };
        let need = if used == 0 { w } else { used + SEP + w };
        if need <= width {
            used = need;
        } else if lines.len() < rows && w <= width {
            lines.push(Vec::new());
            used = w;
        } else {
            continue;
        }
        if let Some(cur) = lines.last_mut() {
            cur.push((k, l, c));
        }
    }
    lines
        .into_iter()
        .filter(|l| !l.is_empty())
        .map(|l| Line::from(key_hints(&l)))
        .collect()
}

/// A [`panel`] whose border switches to the theme's focus colour when `focused`.
fn focus_panel<'a>(title: impl Into<Line<'a>>, focused: bool) -> Block<'a> {
    let t = crate::theme::th();
    panel(title).border_style(Style::default().fg(if focused {
        t.border_focused
    } else {
        t.border
    }))
}

/// A rounded panel in the theme's border colour (the GUI's cards).
fn panel<'a>(title: impl Into<Line<'a>>) -> Block<'a> {
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(crate::theme::th().border))
}

/// `true` for categories on the root/system side (kept visibly separate).
fn is_root_category(c: &cleansys_core::CleanerCategory) -> bool {
    c.name == "System Cleaners" || c.name.ends_with(cleansys_core::model::ROOT_SUFFIX)
}

/// Scan progress / total, for block titles: a spinner while measuring, the total after.
fn scan_line(app: &App) -> Line<'static> {
    if app.board.is_scanning() {
        let done = app.board.total - app.board.pending;
        let mut spans = spinner_spans(app);
        spans.push(Span::styled(
            format!(" scanning {done}/{} ", app.board.total),
            Style::default().fg(crate::theme::th().accent),
        ));
        // A small bar with the same sweeping highlight as the clean progress bar.
        spans.extend(animated_bar(
            8,
            cleansys_core::anim::fraction(done, app.board.total),
            app.anim_tick,
        ));
        Line::from(spans)
    } else if app.board.complete() {
        Line::from(Span::styled(
            format!("{} can be freed", format_size(app.board.total_bytes())),
            Style::default().fg(crate::theme::th().success),
        ))
    } else {
        Line::default()
    }
}

/// `(name, bytes)` of every category with something to reclaim.
fn category_sizes(app: &App) -> Vec<(String, u64)> {
    app.categories
        .iter()
        .enumerate()
        .filter_map(|(i, c)| match app.board.category_bytes_when_done(i) {
            Some(b) if b > 0 => Some((
                c.name
                    .trim_end_matches(cleansys_core::model::ROOT_SUFFIX)
                    .to_string(),
                b,
            )),
            _ => None,
        })
        .collect()
}

/// Treemap (tui-chart) of reclaimable space per category.
fn render_space_treemap(f: &mut Frame, app: &App, area: Rect) {
    let items = category_sizes(app)
        .into_iter()
        .map(|(n, b)| tui_chart::TreemapItem::new(n, b as f64));
    let chart = tui_chart::Treemap::new(items)
        .gradient(crate::theme::th().border, crate::theme::th().success)
        .show_values(false)
        .block(panel("Space"));
    f.render_widget(chart, area);
}

fn render_categories(f: &mut Frame, app: &App, area: Rect) {
    let inner_w = area.width.saturating_sub(2) as usize;
    let mut items: Vec<ListItem> = Vec::new();
    let mut shown_user_header = false;
    let mut shown_root_header = false;

    for (i, category) in app.categories.iter().enumerate() {
        if !app
            .board
            .category_visible(&app.categories, i, app.hide_empty)
        {
            continue;
        }
        let root = is_root_category(category);
        if !root && !shown_user_header {
            shown_user_header = true;
            items.push(ListItem::new(Line::from(vec![
                Span::styled(
                    "USER LAND",
                    Style::default()
                        .fg(crate::theme::th().muted)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    if inner_w >= 34 {
                        "  no password needed"
                    } else {
                        ""
                    },
                    Style::default()
                        .fg(crate::theme::th().muted)
                        .add_modifier(Modifier::DIM),
                ),
            ])));
        }
        if root && !shown_root_header {
            shown_root_header = true;
            items.push(ListItem::new(Line::from(vec![
                Span::styled(
                    "SYSTEM · ROOT",
                    Style::default()
                        .fg(crate::theme::th().muted)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    match (inner_w >= 34, app.is_root) {
                        (false, _) => "",
                        (true, true) => "  running as root",
                        (true, false) => "  asks for your password",
                    },
                    Style::default()
                        .fg(crate::theme::th().muted)
                        .add_modifier(Modifier::DIM),
                ),
            ])));
        }

        let active = i == app.category_index && app.filter.is_empty();
        let ticked = category.items.iter().filter(|it| it.selected).count();
        let name = category
            .name
            .trim_end_matches(cleansys_core::model::ROOT_SUFFIX)
            .to_string();
        let mut right: Vec<Span> = Vec::new();
        if ticked > 0 {
            right.push(Span::styled(
                format!("{ticked}✓  "),
                Style::default().fg(crate::theme::th().success),
            ));
        }
        right.push(match app.board.category_bytes_when_done(i) {
            Some(b) if b > 0 => Span::styled(format_size(b), size_style(b)),
            Some(_) => Span::styled("—", Style::default().fg(crate::theme::th().muted)),
            None if app.board.is_scanning() => spinner_spans(app)
                .into_iter()
                .next()
                .unwrap_or_else(|| Span::raw("…")),
            None => Span::raw(""),
        });
        // Truncate the name so the size column never gets pushed out.
        let left = vec![Span::raw(if active { "▎ " } else { "  " }), Span::raw(name)];
        let style = if active {
            Style::default()
                .fg(crate::theme::th().selection_fg)
                .bg(crate::theme::th().selection_bg)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        items.push(ListItem::new(row_line_spans(left, right, inner_w)).style(style));
    }

    let list = List::new(items).block(
        focus_panel("Categories", app.focus == crate::app::Focus::Categories)
            .title_bottom(scan_line(app).right_aligned()),
    );
    f.render_widget(list, area);
}

/// Per-path rows shown under an expanded cleaner (`☑ label  path  size`).
fn entry_rows(app: &App, ci: usize, ii: usize, width: usize) -> Vec<ListItem<'static>> {
    let Some(info) = app.board.get(ci, ii) else {
        return Vec::new();
    };
    let any_skippable = info.entries.iter().any(|e| e.skippable);
    let mut rows = Vec::with_capacity(info.entries.len() + 2);
    for e in &info.entries {
        let on = app.board.entry_selected(&e.path);
        let (mark, style) = match (e.skippable, on) {
            (true, true) => ("[x]", Style::default().fg(crate::theme::th().success)),
            (true, false) => ("[ ]", Style::default().fg(crate::theme::th().foreground)),
            (false, _) => (" · ", Style::default().fg(crate::theme::th().muted)),
        };
        let size = format_size(e.bytes);
        let head = format!("    {mark} {}", e.label);
        let used = text_width(&head) + text_width(&size) + 4;
        let room = width.saturating_sub(used);
        let path = shorten_left(&e.path, room);
        let left = vec![
            Span::styled(head, style),
            Span::raw("  "),
            Span::styled(path, Style::default().fg(crate::theme::th().muted)),
        ];
        rows.push(ListItem::new(row_line(
            left,
            Span::styled(size, size_style(e.bytes)),
            width,
        )));
    }
    let hidden = info.items.saturating_sub(info.entries.len());
    let hint = if any_skippable {
        "    ↑/↓ move · Space tick · a all · n none · ←/Esc back"
    } else {
        "    removed together — paths can't be unticked · ←/Esc back"
    };
    rows.push(ListItem::new(Line::from(Span::styled(
        clip_str(
            &if hidden > 0 {
                format!("{hint}  (+{hidden} smaller not listed)")
            } else {
                hint.to_string()
            },
            width,
        ),
        Style::default().fg(crate::theme::th().muted),
    ))));
    rows
}

/// Keep the tail of `s` (the interesting part of a path) within `max` chars.
fn shorten_left(s: &str, max: usize) -> String {
    if text_width(s) <= max {
        return s.to_string();
    }
    if max <= 1 {
        return String::new();
    }
    let mut tail: Vec<char> = Vec::new();
    let mut w = 0;
    for c in s.chars().rev() {
        let cw = unicode_width::UnicodeWidthChar::width(c).unwrap_or(0);
        if w + cw > max - 1 {
            break;
        }
        tail.push(c);
        w += cw;
    }
    tail.reverse();
    format!("…{}", tail.into_iter().collect::<String>())
}

/// The list of cleaners (+ optional category bar and detail box).
fn render_cleaner_pane(f: &mut Frame, app: &mut App, area: Rect, show_category_bar: bool) {
    let show_detail = area.height >= 18;
    let mut constraints = Vec::new();
    if show_category_bar {
        constraints.push(Constraint::Length(1));
    }
    constraints.push(Constraint::Min(5));
    if show_detail {
        constraints.push(Constraint::Length(6));
    }
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);
    let mut idx = 0;

    if show_category_bar {
        let cat = &app.categories[app.category_index];
        let visible: Vec<usize> = (0..app.categories.len())
            .filter(|i| {
                app.board
                    .category_visible(&app.categories, *i, app.hide_empty)
            })
            .collect();
        let pos = visible
            .iter()
            .position(|i| *i == app.category_index)
            .map(|p| p + 1)
            .unwrap_or(1);
        let size = app
            .board
            .category_bytes_when_done(app.category_index)
            .filter(|b| *b > 0)
            .map(|b| format!("  {}", format_size(b)))
            .unwrap_or_default();
        let bar_w = chunks[idx].width as usize;
        let bar = Paragraph::new(Line::from(clip_spans(
            vec![
                Span::styled("‹ ", Style::default().fg(crate::theme::th().muted)),
                Span::styled(
                    cat.name.clone(),
                    Style::default()
                        .fg(crate::theme::th().warning)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("  {pos}/{}{size}", visible.len()),
                    Style::default().fg(crate::theme::th().muted),
                ),
                Span::styled(" › (Tab)", Style::default().fg(crate::theme::th().muted)),
            ]
            .into_iter()
            .chain(if app.board.is_scanning() {
                let mut v = vec![Span::raw("  ")];
                v.extend(scan_line(app).spans);
                v
            } else {
                Vec::new()
            })
            .collect::<Vec<_>>(),
            bar_w,
        )));
        f.render_widget(bar, chunks[idx]);
        idx += 1;
    }

    render_cleaners(f, app, chunks[idx]);
    idx += 1;
    if show_detail {
        render_item_detail(f, app, chunks[idx]);
    }
}

fn render_cleaners(f: &mut Frame, app: &mut App, area: Rect) {
    let inner_w = area.width.saturating_sub(3) as usize; // borders + highlight symbol
    let visible = app.view_items();
    let filtering = !app.filter.trim().is_empty();
    // Rows per cleaner: a 3-line card (name, description, path) when there is room for a
    // handful of them, 2 lines on medium panes, one dense line on small ones.
    let card_lines = if area.height >= 26 {
        3
    } else if area.height >= 14 {
        2
    } else {
        1
    };

    let mut items: Vec<ListItem> = visible
        .iter()
        .map(|&(ci, ii)| {
            let item = &app.categories[ci].items[ii];
            let info = app.board.get(ci, ii);

            let checkbox_symbol = if item.selected {
                checkbox_symbols::CHECKED_X
            } else {
                checkbox_symbols::UNCHECKED_SPACE
            };
            let checkbox_style = if item.selected {
                Style::default()
                    .fg(crate::theme::th().success)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(crate::theme::th().foreground)
            };

            let dimmed = (item.requires_root && !app.is_root)
                || info.is_some_and(|s| s.bytes == 0 && s.error.is_none());
            // DIM (not a dark colour): stays readable on the highlighted row.
            let name_style = if dimmed {
                Style::default()
                    .fg(crate::theme::th().secondary)
                    .add_modifier(Modifier::DIM)
            } else {
                Style::default().fg(crate::theme::th().foreground)
            };

            let mut left = vec![
                Span::styled(checkbox_symbol, checkbox_style),
                Span::raw(" "),
                Span::styled(item.name.clone(), name_style),
            ];
            match item.risk {
                cleansys_core::Risk::Safe => {}
                cleansys_core::Risk::Moderate => {
                    left.push(Span::raw(" "));
                    left.push(badge("MODERATE", crate::theme::th().warning));
                }
                cleansys_core::Risk::Caution => {
                    left.push(Span::raw(" "));
                    left.push(badge("CAUTION", crate::theme::th().error));
                }
            }
            if item.requires_root {
                left.push(Span::styled(
                    " (root)",
                    Style::default().fg(crate::theme::th().error),
                ));
            }
            if filtering {
                left.push(Span::styled(
                    format!("  · {}", app.categories[ci].name),
                    Style::default().fg(crate::theme::th().muted),
                ));
            }

            // Right column: run status, else scanned size.
            let right = match &item.status {
                Some(Status::Running) => Span::styled(
                    format!(
                        "{} running",
                        Status::Running.get_animation_frame(app.animation_frame)
                    ),
                    Style::default().fg(crate::theme::th().warning),
                ),
                Some(Status::Success(_)) if item.bytes_cleaned > 0 => Span::styled(
                    format!("✓ freed {}", format_size(item.bytes_cleaned)),
                    Style::default().fg(crate::theme::th().success),
                ),
                Some(Status::Success(_)) => {
                    Span::styled("✓ done", Style::default().fg(crate::theme::th().success))
                }
                Some(Status::Error(_)) => {
                    Span::styled("✗ error", Style::default().fg(crate::theme::th().error))
                }
                Some(Status::Pending) => {
                    Span::styled("queued", Style::default().fg(crate::theme::th().muted))
                }
                None => match info {
                    Some(s) if s.error.is_some() => {
                        Span::styled("scan failed", Style::default().fg(crate::theme::th().error))
                    }
                    Some(s) if s.bytes > 0 => {
                        Span::styled(format_size(s.bytes), size_style(s.bytes))
                    }
                    Some(_) => Span::styled("—", Style::default().fg(crate::theme::th().muted)),
                    None if app.board.is_scanning() => spinner_spans(app)
                        .into_iter()
                        .next()
                        .unwrap_or_else(|| Span::raw("…")),
                    None => Span::raw(""),
                },
            };
            let first = row_line(left, right, inner_w);
            if card_lines == 1 {
                return ListItem::new(first);
            }
            // Card: description (+ item count) and, when tall enough, the biggest path.
            let count = match info {
                Some(s) if s.bytes > 0 => format!("{} item(s)", s.items),
                _ => String::new(),
            };
            let desc_room = inner_w.saturating_sub(text_width(&count) + 6);
            let desc = clip_str(&item.description, desc_room);
            let second = row_line(
                vec![
                    Span::raw("    "),
                    Span::styled(desc, Style::default().fg(crate::theme::th().muted)),
                ],
                Span::styled(count, Style::default().fg(crate::theme::th().muted)),
                inner_w,
            );
            let mut lines = vec![first, second];
            if card_lines == 3 {
                let path = info
                    .and_then(|s| s.top_path.as_deref())
                    .map(|p| format!("▸ {}", shorten_left(p, inner_w.saturating_sub(8))))
                    .unwrap_or_default();
                lines.push(Line::from(vec![
                    Span::raw("    "),
                    Span::styled(
                        clip_str(&path, inner_w.saturating_sub(4)),
                        Style::default().fg(crate::theme::th().info),
                    ),
                ]));
            }
            ListItem::new(lines)
        })
        .collect();

    let title = if filtering || app.filter_active {
        format!(
            "Cleaners — search: {}{}",
            app.filter,
            if app.filter_active { "▏" } else { "" }
        )
    } else {
        let c = &app.categories[app.category_index];
        let hidden = if app.hide_empty && app.board.complete() {
            "  (empty hidden, e: show)"
        } else {
            ""
        };
        format!(
            "{}{hidden}",
            c.name.trim_end_matches(cleansys_core::model::ROOT_SUFFIX)
        )
    };

    let block = focus_panel(
        title,
        app.filter_active || app.focus == crate::app::Focus::Items,
    );
    let block = block.title_bottom({
        let mut spans = scan_line(app).spans;
        spans.push(Span::styled(
            format!(
                "   idle ≥ {} [ ]",
                cleansys_core::engine::config::min_age_label(app.min_age_days)
            ),
            Style::default().fg(crate::theme::th().accent),
        ));
        Line::from(spans).right_aligned()
    });

    if visible.is_empty() {
        let msg = if filtering {
            "No cleaner matches your search."
        } else if app.board.is_scanning() {
            "Scanning your system…"
        } else {
            "Nothing to clean here — already tidy (e: show empty)"
        };
        f.render_widget(
            Paragraph::new(Span::styled(
                msg,
                Style::default().fg(crate::theme::th().muted),
            ))
            .block(block),
            area,
        );
        return;
    }

    // Interleave the expanded cleaner's per-path rows right under it.
    let mut display_sel = app.item_list_state.selected();
    if let Some((ec, ei)) = app.expanded
        && let Some(p) = visible.iter().position(|&v| v == (ec, ei))
    {
        let rows = entry_rows(app, ec, ei, inner_w);
        let n = rows.len();
        items.splice(p + 1..p + 1, rows);
        display_sel = match (app.entry_cursor, display_sel) {
            (Some(k), _) => Some(p + 1 + k.min(n.saturating_sub(1))),
            (None, Some(s)) if s > p => Some(s + n),
            (None, s) => s,
        };
    }
    app.list_view_state.select(display_sel);

    let items_list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(crate::theme::th().selection_bg),
        )
        .highlight_symbol("▎");

    f.render_stateful_widget(items_list, area, &mut app.list_view_state);
}

/// Description + scan details of the highlighted cleaner.
fn render_item_detail(f: &mut Frame, app: &App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();
    let w = area.width.saturating_sub(2) as usize;
    if let Some((ci, ii)) = app.current_item() {
        let item = &app.categories[ci].items[ii];
        let info = app.board.get(ci, ii);
        let risk = match item.risk {
            cleansys_core::Risk::Safe => ("safe", crate::theme::th().success),
            cleansys_core::Risk::Moderate => ("moderate", crate::theme::th().warning),
            cleansys_core::Risk::Caution => ("CAUTION", crate::theme::th().error),
        };
        let mut head = vec![
            Span::styled(
                item.name.clone(),
                Style::default()
                    .fg(crate::theme::th().accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(format!("[{}]", risk.0), Style::default().fg(risk.1)),
        ];
        if item.requires_root {
            head.push(Span::styled(
                " [root]",
                Style::default().fg(crate::theme::th().error),
            ));
        }
        lines.push(Line::from(clip_spans(head, w)));
        lines.push(Line::from(clip_str(&item.description, w)));
        lines.push(Line::from(match info {
            Some(s) if s.error.is_some() => Span::styled(
                clip_str(
                    &format!("Scan failed: {}", s.error.clone().unwrap_or_default()),
                    w,
                ),
                Style::default().fg(crate::theme::th().error),
            ),
            Some(s) if s.bytes > 0 => Span::styled(
                format!("Would free {} in {} item(s)", format_size(s.bytes), s.items),
                size_style(s.bytes),
            ),
            Some(_) => Span::styled(
                "Nothing to clean right now",
                Style::default().fg(crate::theme::th().muted),
            ),
            None => Span::styled(
                "Not measured yet…",
                Style::default().fg(crate::theme::th().muted),
            ),
        }));
        if let Some(top) = info.and_then(|s| s.top_path.as_ref()) {
            lines.push(Line::from(Span::styled(
                format!("Biggest: {}", shorten_left(top, w.saturating_sub(9))),
                Style::default().fg(crate::theme::th().muted),
            )));
        }
    }
    let widget = Paragraph::new(lines).block(panel("Details"));
    f.render_widget(widget, area);
}

/// The GUI's bottom action bar: selection summary on the left, key hints in the middle and
/// the big "Clean" button on the right. Needs 4 rows (box + two lines).
fn render_action_bar(f: &mut Frame, app: &App, area: Rect) {
    let t = crate::theme::th();
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(t.border));
    let inner = block.inner(area);
    f.render_widget(block, area);
    if inner.height == 0 || inner.width < 20 {
        return;
    }

    let n = app
        .categories
        .iter()
        .flat_map(|c| &c.items)
        .filter(|i| i.selected)
        .count();
    let bytes = app.board.selected_bytes(&app.categories);

    // Right: the Clean button (two rows tall, filled).
    let label = if app.is_running {
        format!("{} Cleaning…", cleansys_core::anim::spinner(app.anim_tick))
    } else if n == 0 {
        "Clean".to_string()
    } else if app.board.is_scanning() {
        format!("Clean {n}")
    } else {
        format!("Clean {n} · {}", format_size(bytes))
    };
    let btn_w = (label.chars().count() as u16 + 6).min(inner.width / 2);
    let parts = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(38.min(inner.width / 3)),
            Constraint::Min(0),
            Constraint::Length(btn_w),
        ])
        .split(inner);

    let active = n > 0 && !app.is_running;
    let btn_style = if active {
        Style::default()
            .fg(on_accent())
            .bg(t.primary)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(t.muted)
            .bg(t.surface.unwrap_or(t.selection_bg))
    };
    let btn_rows: Vec<Line> = (0..inner.height)
        .map(|r| {
            if r == (inner.height - 1) / 2 {
                Line::from(label.clone())
            } else {
                Line::default()
            }
        })
        .collect();
    f.render_widget(
        Paragraph::new(btn_rows)
            .style(btn_style)
            .alignment(ratatui::layout::Alignment::Center),
        parts[2],
    );

    // Left: "N selected · X to free" / "Y reclaimable in total".
    let mut top = vec![
        Span::styled(
            format!("{n} selected"),
            Style::default()
                .fg(t.foreground)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" · ", Style::default().fg(t.muted)),
    ];
    if app.board.is_scanning() {
        top.extend(spinner_spans(app));
        top.push(Span::styled(" measuring…", Style::default().fg(t.muted)));
    } else if n == 0 {
        top.push(Span::styled(
            if parts[0].width >= 36 {
                "tick cleaners to free space"
            } else {
                "tick cleaners"
            },
            Style::default().fg(t.muted),
        ));
    } else {
        top.push(Span::styled(
            format!("{} to free", format_size(bytes)),
            Style::default().fg(t.success).add_modifier(Modifier::BOLD),
        ));
    }
    let second = if app.board.complete() {
        Span::styled(
            format!(
                "{} reclaimable in total",
                format_size(app.board.total_bytes())
            ),
            Style::default().fg(t.muted),
        )
    } else {
        Span::styled(
            format!(
                "idle ≥ {}  ([ ])",
                cleansys_core::engine::config::min_age_label(app.min_age_days)
            ),
            Style::default().fg(t.muted),
        )
    };
    let left_w = parts[0].width.saturating_sub(1) as usize;
    f.render_widget(
        Paragraph::new(vec![
            Line::from(clip_spans(top, left_w)),
            Line::from(clip_spans(vec![second], left_w)),
        ]),
        Rect {
            x: parts[0].x + 1,
            width: parts[0].width.saturating_sub(1),
            ..parts[0]
        },
    );

    // Middle: the keys, up to two lines, right-aligned; low-priority ones drop on narrow widths.
    let mid = Rect {
        width: parts[1].width.saturating_sub(1),
        ..parts[1]
    };
    let long = mid.width >= 72;
    let pairs = [
        ("Space", if long { "select" } else { "sel" }, t.warning),
        ("Enter", "run", t.success),
        ("Tab", if long { "category" } else { "cat" }, t.accent),
        ("r", if long { "recommended" } else { "rec" }, t.accent),
        ("/", "search", t.accent),
        ("S", if long { "schedule" } else { "" }, t.accent),
        ("D", if long { "storage chart" } else { "" }, t.accent),
        ("U", if long { "disk usage" } else { "" }, t.accent),
        ("v", if long { "details" } else { "" }, t.accent),
        ("L", "log", t.accent),
        ("?", "help", t.accent),
        ("q", "quit", t.error),
    ];
    f.render_widget(
        Paragraph::new(fit_hints(
            &pairs,
            mid.width as usize,
            inner.height.min(2) as usize,
        ))
        .alignment(ratatui::layout::Alignment::Right),
        mid,
    );
}

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    if area.height >= 4 && area.width >= 70 {
        render_action_bar(f, app, area);
        return;
    }
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(crate::theme::th().muted));

    let inner_area = block.inner(area);

    if inner_area.width < 150 {
        // Narrow terminals: selection summary on one line, compact key hints below.
        let n = app
            .categories
            .iter()
            .flat_map(|c| &c.items)
            .filter(|i| i.selected)
            .count();
        let th = crate::theme::th();
        let hints = fit_hints(
            &[
                ("Space", "Sel", th.warning),
                ("Enter", "Run", th.success),
                ("Tab", "Cat", th.accent),
                ("q", "Quit", th.error),
                ("?", "Help", th.accent),
                ("r", "Rec", th.accent),
                ("D", "Dash", th.accent),
                ("U", "Usage", th.accent),
                ("v", "Detail", th.accent),
                ("S", "Sched", th.accent),
                ("L", "Log", th.accent),
            ],
            inner_area.width as usize,
            1,
        );
        let mut status_spans = vec![
            Span::styled(
                "Selected: ",
                Style::default().fg(crate::theme::th().foreground),
            ),
            Span::styled(
                n.to_string(),
                Style::default()
                    .fg(crate::theme::th().accent)
                    .add_modifier(Modifier::BOLD),
            ),
        ];
        status_spans.push(Span::raw("  •  "));
        status_spans.push(Span::styled(
            format!(
                "Idle ≥ {} ([ ])",
                cleansys_core::engine::config::min_age_label(app.min_age_days)
            ),
            Style::default().fg(crate::theme::th().accent),
        ));
        status_spans.extend(to_free_spans(app));
        let status = Line::from(clip_spans(status_spans, inner_area.width as usize));
        let lines = if inner_area.height >= 2 {
            std::iter::once(status).chain(hints).collect()
        } else {
            hints
        };
        f.render_widget(Paragraph::new(lines), inner_area);
    } else {
        // Main menu footer - organized and clean
        let footer_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(50), // Status info
                Constraint::Min(0),     // Controls (right-aligned, take the rest)
            ])
            .split(inner_area);

        // Status information
        let mut status_spans = vec![
            Span::styled(
                "User: ",
                Style::default()
                    .fg(crate::theme::th().foreground)
                    .add_modifier(Modifier::BOLD),
            ),
            if app.is_root {
                Span::styled(
                    "root",
                    Style::default()
                        .fg(crate::theme::th().error)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(
                    "standard",
                    Style::default()
                        .fg(crate::theme::th().success)
                        .add_modifier(Modifier::BOLD),
                )
            },
            Span::raw("  •  "),
            Span::styled(
                "Selected: ",
                Style::default().fg(crate::theme::th().foreground),
            ),
            Span::styled(
                // Live count of ticked cleaners.
                format!(
                    "{}",
                    app.categories
                        .iter()
                        .flat_map(|c| &c.items)
                        .filter(|i| i.selected)
                        .count()
                ),
                Style::default()
                    .fg(crate::theme::th().accent)
                    .add_modifier(Modifier::BOLD),
            ),
        ];
        status_spans.extend(to_free_spans(app));
        let status_text = vec![Line::from(status_spans)];

        // Controls - organized by function
        let controls_text = vec![Line::from(vec![
            Span::styled(
                "Space",
                Style::default()
                    .fg(crate::theme::th().warning)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(": Select  "),
            Span::styled(
                "Enter",
                Style::default()
                    .fg(crate::theme::th().success)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(": Run  "),
            Span::styled(
                "Tab",
                Style::default()
                    .fg(crate::theme::th().accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(": Category  "),
            Span::styled(
                "r",
                Style::default()
                    .fg(crate::theme::th().accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(": Rec  "),
            Span::styled(
                "S",
                Style::default()
                    .fg(crate::theme::th().accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(": Sched  "),
            Span::styled(
                "D",
                Style::default()
                    .fg(crate::theme::th().accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(": Dash  "),
            Span::styled(
                "v",
                Style::default()
                    .fg(crate::theme::th().accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(": Detail/→  "),
            Span::styled(
                "L",
                Style::default()
                    .fg(crate::theme::th().accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(": Log  "),
            Span::styled(
                "?",
                Style::default()
                    .fg(crate::theme::th().accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(": Help  "),
            Span::styled(
                "q",
                Style::default()
                    .fg(crate::theme::th().error)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(": Quit"),
        ])];

        let status_para = Paragraph::new(status_text);
        let controls_para =
            Paragraph::new(controls_text).alignment(ratatui::layout::Alignment::Right);

        f.render_widget(status_para, footer_chunks[0]);
        f.render_widget(controls_para, footer_chunks[1]);
    }

    f.render_widget(block, area);
}

/// Draws the help; returns the scroll offset actually used (clamped to the content).
fn render_help(f: &mut Frame, area: Rect, scroll: u16) -> u16 {
    let h = |t: &'static str| {
        Line::from(vec![Span::styled(
            t,
            Style::default().add_modifier(Modifier::BOLD),
        )])
    };
    let l = |t: &'static str| Line::from(vec![Span::raw(t)]);
    let help_text = vec![
        Line::from(vec![Span::styled(
            "Cleansys Help",
            Style::default()
                .fg(crate::theme::th().accent)
                .add_modifier(Modifier::BOLD),
        )]),
        l(""),
        h("Navigation:"),
        l("  ↑/↓ (j/k): Navigate cleaners      Home/End: first / last"),
        l("  Tab/Shift+Tab: Switch pane (categories · cleaners · activity)"),
        l("  ←/→: Back out / go in (categories → cleaners → details)"),
        l("  O: Options panel     L: Activity log (↑/↓ PgUp/PgDn scroll)"),
        l("  /: Filter cleaners across all categories (Esc clears)"),
        l("  e: Hide/show cleaners with nothing to clean"),
        l(""),
        h("Settings & info:"),
        l("  o: Settings (idle days, scan depth, folders, exclusions, confirm, hide empty)"),
        l("  i: About — version, developer, GitHub (Enter opens a link)    Tab: switch tab"),
        l(""),
        h("Selecting:"),
        l("  Space: Toggle the highlighted cleaner"),
        l("  →: Expand a cleaner's paths (↑/↓ move, Space tick, a/n all/none, ← back)"),
        l("  a / n: Select / deselect all in this category    A / N: everywhere"),
        l("  r: Select the recommended set (safe, user-land cleaners)"),
        l("  [ / ]: Idle days for project build output (fewer / more), then re-scans"),
        l(""),
        h("Cleaning:"),
        l("  d: Preview (dry-run) — exact paths and sizes, deletes nothing"),
        l("  Enter: Run the selected cleaners (asks for confirmation; y toggles that)"),
        l("  Progress, the outcome and the activity log appear under the list;"),
        l("  q / Esc cancels a running clean, Esc dismisses the summary, L toggles the log"),
        l("  R: Re-scan sizes      S: Schedule automatic cleaning"),
        l("  D: Dashboard (disk, space by category/risk)   v: Charts for the highlighted cleaner"),
        l("  U: Disk usage — where did my disk go? (↑/↓, Enter opens a folder, ← goes up)"),
        l("  P: Report of the last clean (opens automatically when a clean finishes)"),
        l("  T: Theme picker (System follows your OS light/dark; ↑/↓ previews, Esc closes)"),
        l(""),
        h("System Operations:"),
        l("  System cleaners need sudo/root: run 'sudo cleansys' or enter your"),
        l("  password when prompted. Items marked (root) ask for elevation."),
        l(""),
        h("Other:"),
        l("  ?: Show/hide help"),
        l("  q: Quit"),
    ];

    // Rows after wrapping (a line wraps at word boundaries; this is a close estimate).
    let inner_w = area.width.saturating_sub(2).max(1) as usize;
    let rows: usize = help_text
        .iter()
        .map(|l| l.width().div_ceil(inner_w).max(1))
        .sum();
    let max_scroll = (rows as u16).saturating_sub(area.height.saturating_sub(2));
    let scroll = scroll.min(max_scroll);
    let mut block = Block::default().title("Help").borders(Borders::ALL);
    if max_scroll > 0 {
        block = block.title_bottom(
            Line::from(format!(
                " ↑/↓ PgUp/PgDn scroll · {}/{} ",
                scroll, max_scroll
            ))
            .right_aligned(),
        );
    }
    let help = Paragraph::new(help_text)
        .block(block)
        .scroll((scroll, 0))
        .wrap(Wrap { trim: true });

    f.render_widget(help, area);
    scroll
}

/// Compute a centered popup `Rect` covering roughly `width_pct`/`height_pct`
/// of `area`, clamped to a sensible minimum/maximum size.
fn centered_popup(area: Rect, width_pct: u16, height_pct: u16) -> Rect {
    // Never larger than the area itself, even on a tiny terminal.
    let (min_w, min_h) = (30.min(area.width), 10.min(area.height));
    let width = (area.width * width_pct / 100)
        .clamp(min_w, area.width.saturating_sub(2).max(min_w))
        .min(area.width);
    let height = (area.height * height_pct / 100)
        .clamp(min_h, area.height.saturating_sub(2).max(min_h))
        .min(area.height);
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect {
        x,
        y,
        width,
        height,
    }
}

/// Overlay shown before actually cleaning: lists exactly what's selected and
/// requires an explicit Enter/y (confirm) or Esc/n (cancel).
fn render_confirm_run(f: &mut Frame, app: &App, area: Rect) {
    let popup = centered_popup(area, 70, 60);

    let selected_count = app.pending_run_selection.len();
    let mut lines = vec![
        Line::from(vec![Span::styled(
            "! Confirm Cleaning",
            Style::default()
                .fg(crate::theme::th().warning)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::raw(format!(
            "This will permanently delete files for {selected_count} selected cleaner(s):"
        ))]),
        Line::from(vec![Span::raw("")]),
    ];

    for (_, _, name, _, requires_root) in app.pending_run_selection.iter().take(15) {
        let suffix = if *requires_root { " (root)" } else { "" };
        lines.push(Line::from(vec![Span::raw(format!("  • {name}{suffix}"))]));
    }
    if app.pending_run_selection.len() > 15 {
        lines.push(Line::from(vec![Span::styled(
            format!("  … and {} more", app.pending_run_selection.len() - 15),
            Style::default().fg(crate::theme::th().muted),
        )]));
    }

    lines.push(Line::from(vec![Span::raw("")]));
    lines.push(Line::from(vec![Span::styled(
        "Enter/y: Run now    Esc/n: Cancel",
        Style::default()
            .fg(crate::theme::th().success)
            .add_modifier(Modifier::BOLD),
    )]));

    let popup_widget = Paragraph::new(lines)
        .block(
            Block::default()
                .title("Confirm")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(crate::theme::th().warning)),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(Clear, popup);
    f.render_widget(popup_widget, popup);
}

/// Overlay shown for a preview (dry-run): lists what *would* be cleaned and
/// its real measured size, without anything having been deleted.
/// Overlay for configuring automatic (scheduled) cleaning.
fn render_schedule(f: &mut Frame, app: &App, area: Rect) {
    use crate::app::ScheduleField;
    use cleansys_core::engine::schedule::{Backend, Frequency, Scope, WEEKDAYS};

    let popup = centered_popup(area, 70, 80);
    let d = &app.schedule_draft;

    let mut lines = vec![
        Line::from(vec![Span::styled(
            "⏰ Automatic cleaning",
            Style::default()
                .fg(crate::theme::th().accent)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(match &app.schedule_installed {
            Some(b) => Span::styled(
                format!("Status: ACTIVE via {b} — {}", d.describe()),
                Style::default().fg(crate::theme::th().success),
            ),
            None => Span::styled(
                "Status: not scheduled",
                Style::default().fg(crate::theme::th().muted),
            ),
        }),
    ];
    if let Some(l) = &app.schedule_last_run {
        lines.push(Line::from(Span::styled(
            format!(
                "Last run: {} — freed {} ({} cleaners{})",
                l.ago(),
                format_size(l.bytes_freed),
                l.cleaners_run,
                if l.scheduled { ", scheduled" } else { "" }
            ),
            Style::default().fg(crate::theme::th().muted),
        )));
    }
    lines.push(Line::from(""));

    for (i, field) in app.schedule_fields().iter().enumerate() {
        let (label, value) = match field {
            ScheduleField::Frequency => (
                "Repeat",
                match d.frequency {
                    Frequency::Daily => "Daily",
                    Frequency::Weekly => "Weekly",
                    Frequency::Monthly => "Monthly",
                }
                .to_string(),
            ),
            ScheduleField::Hour => ("Hour", format!("{:02}", d.hour)),
            ScheduleField::Minute => ("Minute", format!("{:02}", d.minute)),
            ScheduleField::Day => match d.frequency {
                Frequency::Monthly => ("Day of month", d.day_of_month.to_string()),
                _ => (
                    "Weekday",
                    WEEKDAYS[usize::from(d.weekday.min(6))].to_string(),
                ),
            },
            ScheduleField::Scope => (
                "Cleans",
                match d.scope {
                    Scope::Recommended => "Recommended — safe caches only".to_string(),
                    Scope::Extended => "Extended — safe + moderate".to_string(),
                    Scope::Selected => {
                        "Selected — the cleaners ticked in the main list".to_string()
                    }
                },
            ),
            ScheduleField::Backend => (
                "Backend",
                match d.backend {
                    Backend::Auto => "Auto (systemd timer, else cron)",
                    Backend::Systemd => "systemd user timer",
                    Backend::Cron => "cron (crontab)",
                }
                .to_string(),
            ),
        };
        let selected = i == app.schedule_field;
        let style = if selected {
            Style::default()
                .fg(crate::theme::bg(&crate::theme::th()))
                .bg(crate::theme::th().accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        lines.push(Line::from(vec![
            Span::raw(if selected { "▶ " } else { "  " }),
            Span::styled(
                format!("{label:<14}"),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!(" ◀ {value} ▶ "), style),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(format!("Will run: {}", d.describe())));
    lines.push(Line::from(Span::styled(
        "Unattended runs only touch user-land cleaners, skip apps that are open,",
        Style::default().fg(crate::theme::th().muted),
    )));
    lines.push(Line::from(Span::styled(
        "and never include caution-risk cleaners unless you ticked them (Selected).",
        Style::default().fg(crate::theme::th().muted),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        app.schedule_message.clone(),
        Style::default().fg(crate::theme::th().warning),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "↑/↓ field   ←/→ change   Enter enable/update   d remove   Esc close",
        Style::default()
            .fg(crate::theme::th().success)
            .add_modifier(Modifier::BOLD),
    )));

    let widget = Paragraph::new(lines)
        .block(
            Block::default()
                .title("Schedule")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(crate::theme::th().accent)),
        )
        .wrap(Wrap { trim: false });
    f.render_widget(Clear, popup);
    f.render_widget(widget, popup);
}

fn render_preview(f: &mut Frame, app: &App, area: Rect) {
    let popup = centered_popup(area, 80, 75);

    let total_bytes: u64 = app.preview_results.iter().map(|(_, r)| r.total_bytes).sum();
    let total_items: usize = app
        .preview_results
        .iter()
        .map(|(_, r)| r.item_count())
        .sum();

    let mut lines = vec![
        Line::from(vec![Span::styled(
            "Preview (dry-run)",
            Style::default()
                .fg(crate::theme::th().accent)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::raw(format!(
            "Would free {} across {total_items} item(s). Nothing has been deleted.",
            format_size(total_bytes)
        ))]),
        Line::from(vec![Span::raw("")]),
    ];

    if app.preview_results.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "Nothing to clean — all selected cleaners are already empty.",
            Style::default().fg(crate::theme::th().muted),
        )]));
    }

    for (name, result) in app
        .preview_results
        .iter()
        .filter(|(_, r)| r.total_bytes > 0)
    {
        lines.push(Line::from(vec![Span::styled(
            format!(
                "{name} — {} across {} item(s)",
                format_size(result.total_bytes),
                result.item_count()
            ),
            Style::default().add_modifier(Modifier::BOLD),
        )]));
        for item in result.items.iter().take(3) {
            lines.push(Line::from(vec![Span::raw(format!(
                "    • {} ({})",
                item.path_str(),
                format_size(item.size)
            ))]));
        }
        if result.items.len() > 3 {
            lines.push(Line::from(vec![Span::styled(
                format!("    … and {} more", result.items.len() - 3),
                Style::default().fg(crate::theme::th().muted),
            )]));
        }
        lines.push(Line::from(vec![Span::raw("")]));
    }

    lines.push(Line::from(vec![Span::styled(
        "Enter/Esc/q: Close",
        Style::default()
            .fg(crate::theme::th().success)
            .add_modifier(Modifier::BOLD),
    )]));

    let popup_widget = Paragraph::new(lines)
        .block(
            Block::default()
                .title("Preview")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(crate::theme::th().accent)),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(Clear, popup);
    f.render_widget(popup_widget, popup);
}

/// Overlay shown when a selected cleaner needs Administrator privileges on
/// Windows, where there is no interactive sudo-password prompt to fall back
/// to — the user must restart the process elevated themselves.
fn render_admin_notice(f: &mut Frame, area: Rect) {
    let popup = centered_popup(area, 60, 30);

    let lines = vec![
        Line::from(vec![Span::styled(
            "! Administrator privileges required",
            Style::default()
                .fg(crate::theme::th().warning)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::raw(
            "One or more selected cleaners need Administrator privileges.",
        )]),
        Line::from(vec![Span::raw(
            "Restart CleanSys as Administrator to use them.",
        )]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::styled(
            "Enter/Esc/q: Close",
            Style::default()
                .fg(crate::theme::th().success)
                .add_modifier(Modifier::BOLD),
        )]),
    ];

    let popup_widget = Paragraph::new(lines)
        .block(
            Block::default()
                .title("Administrator required")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(crate::theme::th().warning)),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(Clear, popup);
    f.render_widget(popup_widget, popup);
}

// ── Settings / About overlay ──────────────────────────────────────────────

fn tab_line(app: &App, width: usize) -> Line<'static> {
    let tab = |label: &'static str, active: bool| {
        if active {
            Span::styled(
                format!(" {label} "),
                Style::default()
                    .fg(crate::theme::bg(&crate::theme::th()))
                    .bg(crate::theme::th().accent)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled(
                format!(" {label} "),
                Style::default().fg(crate::theme::th().secondary),
            )
        }
    };
    Line::from(clip_spans(
        vec![
            tab("Settings (o)", app.info_tab == InfoTab::Settings),
            Span::raw(" "),
            tab("About (i)", app.info_tab == InfoTab::About),
            Span::styled(
                "   Tab: switch",
                Style::default().fg(crate::theme::th().muted),
            ),
        ],
        width,
    ))
}

fn render_info(f: &mut Frame, app: &mut App, area: Rect) {
    let popup = centered_popup(area, 82, 88);
    f.render_widget(Clear, popup);
    let block = Block::default()
        .title(format!(" {} ", cleansys_core::appinfo::title()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(crate::theme::th().accent));
    let inner = block.inner(popup);
    f.render_widget(block, popup);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // tabs
            Constraint::Min(3),    // body
            Constraint::Length(2), // message + hint
        ])
        .split(inner);
    f.render_widget(
        Paragraph::new(vec![tab_line(app, chunks[0].width as usize), Line::raw("")]),
        chunks[0],
    );

    match app.info_tab {
        InfoTab::Settings => render_settings_tab(f, app, chunks[1]),
        InfoTab::About => render_about_tab(f, app, chunks[1]),
    }

    let wide = chunks[2].width >= 74;
    let hint = match (app.info_tab, app.settings_input.is_some(), wide) {
        (_, true, _) => "Enter: add · Esc: cancel",
        (InfoTab::Settings, false, true) => {
            "↑/↓ move · ←/→ change · Space toggle/add · d remove · Tab About · Esc close"
        }
        (InfoTab::Settings, false, false) => "↑/↓ ←/→ Space d · Tab · Esc",
        (InfoTab::About, false, true) => {
            "↑/↓ move · Enter: open link in your browser · Tab Settings · Esc close"
        }
        (InfoTab::About, false, false) => "↑/↓ · Enter open · Tab · Esc",
    };
    let hint = clip_str(hint, chunks[2].width as usize);
    let message = clip_str(&app.settings_message, chunks[2].width as usize);
    let msg_style = Style::default().fg(crate::theme::th().warning);
    let lines = vec![
        Line::from(Span::styled(message, msg_style)),
        Line::from(Span::styled(
            hint,
            Style::default().fg(crate::theme::th().muted),
        )),
    ];
    f.render_widget(Paragraph::new(lines), chunks[2]);
}

fn value_span(text: String, focused: bool) -> Span<'static> {
    if focused {
        Span::styled(
            format!("◂ {text} ▸"),
            Style::default()
                .fg(crate::theme::th().accent)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(text, Style::default().fg(crate::theme::th().accent))
    }
}

fn switch_span(on: bool) -> Span<'static> {
    if on {
        Span::styled("[x] on ", Style::default().fg(crate::theme::th().success))
    } else {
        Span::styled("[ ] off", Style::default().fg(crate::theme::th().muted))
    }
}

fn render_settings_tab(f: &mut Frame, app: &mut App, area: Rect) {
    // minus the two columns of the "> " highlight symbol
    let width = (area.width as usize).saturating_sub(2);
    let rows = app.settings_rows();
    let mut items: Vec<ListItem> = Vec::new();
    let mut display_of: Vec<usize> = Vec::new(); // selectable index -> list index
    let header = |t: &'static str| {
        ListItem::new(Line::from(Span::styled(
            t,
            Style::default()
                .fg(crate::theme::th().warning)
                .add_modifier(Modifier::BOLD),
        )))
    };
    let roots = app.engine_cfg.roots_for_editing();
    let deep_roots = app.engine_cfg.deep_roots_for_editing();
    let mut last_group = "";
    for (idx, row) in rows.iter().enumerate() {
        let group = match row {
            SettingsRow::IdleDays | SettingsRow::ScanDepth => "Scanning",
            SettingsRow::HideEmpty | SettingsRow::Confirm => "Behaviour",
            SettingsRow::Root(_) | SettingsRow::AddRoot => {
                "Project folders scanned for build output"
            }
            SettingsRow::DeepRoot(_) | SettingsRow::AddDeepRoot => {
                "Deep clean searches (large / old files)"
            }
            SettingsRow::Exclude(_) | SettingsRow::AddExclude => "Never delete (glob patterns)",
        };
        if group != last_group {
            if !items.is_empty() {
                items.push(ListItem::new(Line::raw("")));
            }
            items.push(header(group));
            last_group = group;
        }
        let focused = idx == app.settings_cursor;
        let label = |t: &str| format!("  {t}");
        let line = match row {
            SettingsRow::IdleDays => row_line(
                vec![Span::raw(label("Project idle for at least"))],
                value_span(
                    cleansys_core::engine::config::min_age_label(app.engine_cfg.min_age_days),
                    focused,
                ),
                width,
            ),
            SettingsRow::ScanDepth => row_line(
                vec![Span::raw(label("Scan depth (folders below each root)"))],
                value_span(app.engine_cfg.max_depth.to_string(), focused),
                width,
            ),
            SettingsRow::HideEmpty => row_line(
                vec![Span::raw(label("Hide cleaners with nothing to clean"))],
                switch_span(app.hide_empty),
                width,
            ),
            SettingsRow::Confirm => row_line(
                vec![Span::raw(label("Ask before cleaning"))],
                switch_span(app.confirmation_mode),
                width,
            ),
            SettingsRow::Root(i) => Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    shorten_left(
                        &roots.get(*i).cloned().unwrap_or_default(),
                        width.saturating_sub(2),
                    ),
                    Style::default().fg(crate::theme::th().foreground),
                ),
            ]),
            SettingsRow::DeepRoot(i) => Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    shorten_left(
                        &deep_roots.get(*i).cloned().unwrap_or_default(),
                        width.saturating_sub(2),
                    ),
                    Style::default().fg(crate::theme::th().foreground),
                ),
            ]),
            SettingsRow::Exclude(i) => Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    shorten_left(
                        &app.engine_cfg.exclude.get(*i).cloned().unwrap_or_default(),
                        width.saturating_sub(2),
                    ),
                    Style::default().fg(crate::theme::th().foreground),
                ),
            ]),
            SettingsRow::AddRoot | SettingsRow::AddDeepRoot | SettingsRow::AddExclude => {
                let kind = if *row == SettingsRow::AddRoot {
                    InputKind::Root
                } else if *row == SettingsRow::AddDeepRoot {
                    InputKind::DeepRoot
                } else {
                    InputKind::Exclude
                };
                match &app.settings_input {
                    Some(inp) if inp.kind == kind => Line::from(vec![
                        Span::styled("  ＋ ", Style::default().fg(crate::theme::th().success)),
                        Span::styled(
                            format!("{}▏", shorten_left(&inp.text, width.saturating_sub(6))),
                            Style::default().fg(crate::theme::th().foreground),
                        ),
                    ]),
                    _ => Line::from(Span::styled(
                        if kind != InputKind::Exclude {
                            "  ＋ Add a folder…"
                        } else {
                            "  ＋ Add a pattern…"
                        },
                        Style::default().fg(crate::theme::th().success),
                    )),
                }
            }
        };
        display_of.push(items.len());
        items.push(ListItem::new(line));
    }
    app.list_info_state
        .select(display_of.get(app.settings_cursor).copied());
    let list = List::new(items)
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(crate::theme::th().muted),
        )
        .highlight_symbol("> ");
    f.render_stateful_widget(list, area, &mut app.list_info_state);
}

fn render_about_tab(f: &mut Frame, app: &mut App, area: Rect) {
    let width = (area.width as usize).saturating_sub(2);
    let rows = cleansys_core::appinfo::about_rows();
    let mut items: Vec<ListItem> = Vec::new();
    items.push(ListItem::new(Line::from(vec![Span::styled(
        format!("  {}", cleansys_core::appinfo::title()),
        Style::default()
            .fg(crate::theme::th().accent)
            .add_modifier(Modifier::BOLD),
    )])));
    items.push(ListItem::new(Line::from(Span::styled(
        format!("  {}", cleansys_core::appinfo::TAGLINE),
        Style::default().fg(crate::theme::th().secondary),
    ))));
    items.push(ListItem::new(Line::raw("")));
    let first_row = items.len();
    for (label, value) in &rows {
        let is_link = cleansys_core::appinfo::is_link(value);
        let style = if is_link {
            Style::default()
                .fg(crate::theme::th().accent)
                .add_modifier(Modifier::UNDERLINED)
        } else {
            Style::default().fg(crate::theme::th().foreground)
        };
        let room = width.saturating_sub(18);
        let shown = shorten_left(value, room);
        items.push(ListItem::new(Line::from(vec![
            Span::styled(
                format!("  {label:<14}"),
                Style::default().fg(crate::theme::th().muted),
            ),
            Span::styled(shown, style),
        ])));
    }
    app.list_info_state.select(Some(
        first_row + app.about_cursor.min(rows.len().saturating_sub(1)),
    ));
    let list = List::new(items)
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(crate::theme::th().muted),
        )
        .highlight_symbol("> ");
    f.render_stateful_widget(list, area, &mut app.list_info_state);
}

/// The theme picker popup: "System" plus every preset, with a colour swatch each.
fn render_theme_picker(f: &mut Frame, app: &App, area: Rect) {
    use ratatui::widgets::ListState;
    let t = std::sync::Arc::clone(&app.theme);
    let h = area.height.saturating_sub(4).clamp(8, 24);
    let w = 44.min(area.width);
    let popup = Rect::new(
        area.x + area.width.saturating_sub(w) / 2,
        area.y + area.height.saturating_sub(h) / 2,
        w,
        h.min(area.height),
    );
    f.render_widget(Clear, popup);

    let items: Vec<ListItem> = (0..crate::theme::PICKER_LEN)
        .map(|i| {
            let swatch = |c: cleansys_core::Rgb| {
                Span::styled("■", Style::default().fg(Color::Rgb(c.r, c.g, c.b)))
            };
            // "System" previews the preset it currently resolves to.
            let preview = if i == 0 {
                cleansys_core::theme_by_index(cleansys_core::current_system_theme_index())
            } else {
                cleansys_core::theme_by_index(i - 1)
            };
            let active = i == app.picker_index();
            ListItem::new(Line::from(vec![
                swatch(preview.accent),
                swatch(preview.success),
                swatch(preview.warning),
                Span::raw(" "),
                Span::styled(
                    crate::theme::picker_name(i),
                    if active {
                        Style::default().fg(t.accent).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(t.foreground)
                    },
                ),
            ]))
        })
        .collect();

    let block = Block::default()
        .title(Span::styled(
            format!(
                " Theme · {}/{} ",
                app.picker_index() + 1,
                crate::theme::PICKER_LEN
            ),
            Style::default().fg(t.accent).add_modifier(Modifier::BOLD),
        ))
        .title_bottom(Span::styled(
            " ↑/↓ preview · Esc/Enter close ",
            Style::default().fg(t.muted),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(t.accent))
        .style(
            Style::default()
                .bg(t.surface.unwrap_or_else(|| crate::theme::bg(&t)))
                .fg(t.foreground),
        );
    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(t.selection_bg)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");
    let mut state = ListState::default().with_selected(Some(app.picker_index()));
    f.render_stateful_widget(list, popup, &mut state);
}
