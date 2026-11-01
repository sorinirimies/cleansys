//! Graphical views built from `tui-chart` and `tui-piechart`:
//!
//! * the **dashboard** — state of the disk and where the reclaimable space is,
//! * the per-cleaner **detail** view — what one cleaner would remove,
//! * the clean **report** — what a run freed, shown when it finishes.
//!
//! Data is derived from [`App`] on the fly (the pure `*_data` functions are unit-tested);
//! the `render_*` functions only draw it.

use cleansys_core::{format_size, Risk, Status};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
    Frame,
};
use tui_chart::Sparkline;
use tui_piechart::{PieChart, PieSlice};

use crate::app::App;
use crate::theme::th;

/// Size unit that keeps chart numbers readable: `(divisor, suffix)`.
pub fn unit_for(max_bytes: u64) -> (f64, &'static str) {
    const KB: f64 = 1024.0;
    match max_bytes as f64 {
        m if m >= KB * KB * KB => (KB * KB * KB, "GB"),
        m if m >= KB * KB => (KB * KB, "MB"),
        m if m >= KB => (KB, "KB"),
        _ => (1.0, "B"),
    }
}

/// Shorten `s` to `max` chars with a trailing `…`.
pub fn clip(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!(
            "{}…",
            s.chars().take(max.saturating_sub(1)).collect::<String>()
        )
    }
}

/// Last path component (or the whole string).
fn base_name(path: &str) -> &str {
    path.trim_end_matches(['/', '\\'])
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(path)
}

fn palette() -> [Color; 6] {
    let t = th();
    [
        t.accent,
        t.success,
        t.warning,
        t.error,
        t.secondary,
        t.muted,
    ]
}

fn frame_block(title: impl Into<String>) -> Block<'static> {
    Block::default()
        .title(title.into())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(th().muted))
}

// ── data ───────────────────────────────────────────────────────────────

/// What the dashboard shows.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct DashboardData {
    /// Reclaimable bytes per category, biggest first (empty ones dropped).
    pub by_category: Vec<(String, u64)>,
    /// Reclaimable bytes per risk level: safe, moderate, caution.
    pub by_risk: [u64; 3],
    /// Biggest cleaners `(name, bytes)`, biggest first.
    pub top: Vec<(String, u64)>,
    pub total: u64,
}

pub fn dashboard_data(app: &App) -> DashboardData {
    let mut d = DashboardData::default();
    let mut all: Vec<(String, u64)> = Vec::new();
    for (ci, cat) in app.categories.iter().enumerate() {
        let mut sum = 0;
        for (ii, item) in cat.items.iter().enumerate() {
            let Some(info) = app.board.get(ci, ii) else {
                continue;
            };
            if info.bytes == 0 {
                continue;
            }
            sum += info.bytes;
            d.by_risk[match item.risk {
                Risk::Safe => 0,
                Risk::Moderate => 1,
                Risk::Caution => 2,
            }] += info.bytes;
            all.push((item.name.clone(), info.bytes));
        }
        if sum > 0 {
            let name = cat
                .name
                .trim_end_matches(cleansys_core::model::ROOT_SUFFIX)
                .to_string();
            d.by_category.push((name, sum));
        }
    }
    d.by_category.sort_by_key(|(_, b)| std::cmp::Reverse(*b));
    all.sort_by_key(|(_, b)| std::cmp::Reverse(*b));
    d.total = all.iter().map(|(_, b)| b).sum();
    all.truncate(12);
    d.top = all;
    d
}

/// What the clean report shows.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct ReportData {
    /// Freed per cleaner `(name, bytes)`, biggest first (only cleaners that freed something).
    pub cleaners: Vec<(String, u64)>,
    /// Freed per category, biggest first.
    pub by_category: Vec<(String, u64)>,
    pub freed: u64,
    pub items: usize,
    pub ok: usize,
    pub nothing: usize,
    /// `(name, message)` of cleaners that failed or were cancelled.
    pub failed: Vec<(String, String)>,
}

pub fn report_data(app: &App) -> ReportData {
    let mut r = ReportData::default();
    for cat in &app.categories {
        let mut cat_sum = 0;
        for item in &cat.items {
            match &item.status {
                Some(Status::Success(_)) => {
                    r.ok += 1;
                    if item.bytes_cleaned == 0 {
                        r.nothing += 1;
                    } else {
                        r.cleaners.push((item.name.clone(), item.bytes_cleaned));
                        cat_sum += item.bytes_cleaned;
                    }
                    r.items += item.last_result.as_ref().map_or(0, |x| x.item_count());
                }
                Some(Status::Error(m)) => r.failed.push((item.name.clone(), m.clone())),
                _ => {}
            }
        }
        if cat_sum > 0 {
            let name = cat
                .name
                .trim_end_matches(cleansys_core::model::ROOT_SUFFIX)
                .to_string();
            r.by_category.push((name, cat_sum));
        }
    }
    r.cleaners.sort_by_key(|(_, b)| std::cmp::Reverse(*b));
    r.by_category.sort_by_key(|(_, b)| std::cmp::Reverse(*b));
    r.freed = r.cleaners.iter().map(|(_, b)| b).sum();
    r
}

// ── widgets ────────────────────────────────────────────────────────────

/// Fixed-width text bar: `frac` of `width` cells filled with eighth-block precision.
fn bar_text(frac: f64, width: usize) -> (String, String) {
    const PART: [char; 8] = [' ', '▏', '▎', '▍', '▌', '▋', '▊', '▉'];
    let eighths = (frac.clamp(0.0, 1.0) * width as f64 * 8.0).round() as usize;
    let full = (eighths / 8).min(width);
    let rem = if full < width { eighths % 8 } else { 0 };
    let mut filled = "█".repeat(full);
    if rem > 0 {
        filled.push(PART[rem]);
    }
    let used = full + usize::from(rem > 0);
    (filled, "░".repeat(width.saturating_sub(used)))
}

/// Pad / clip `s` to exactly `w` display cells (chars).
fn fit(s: &str, w: usize) -> String {
    let c = clip(s, w);
    format!("{c:<w$}")
}

/// Ranked horizontal bars: `label  ████▍░░░  size  pct`, one coloured row per entry.
struct RankedBars<'a> {
    title: String,
    rows: &'a [(String, u64)],
    max_rows: usize,
}

impl Widget for RankedBars<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = frame_block(format!(" {} ", self.title));
        let inner = block.inner(area);
        block.render(area, buf);
        if inner.width < 24 || inner.height == 0 {
            return;
        }
        let rows = &self.rows[..self.rows.len().min(self.max_rows)];
        let total: u64 = self.rows.iter().map(|(_, b)| *b).sum();
        let max = rows.iter().map(|(_, b)| *b).max().unwrap_or(0).max(1);
        let w = inner.width as usize;
        let label_w = (w / 3).clamp(10, 26);
        let tail_w = 17; // " 123.45 GB  99%"
        let bar_w = w.saturating_sub(label_w + tail_w + 2).max(4);
        let colors = palette();
        let lines: Vec<Line> = rows
            .iter()
            .enumerate()
            .map(|(i, (name, bytes))| {
                let color = colors[i % colors.len()];
                let (filled, empty) = bar_text(*bytes as f64 / max as f64, bar_w);
                let pct = if total == 0 {
                    0
                } else {
                    (*bytes as f64 * 100.0 / total as f64).round() as u64
                };
                Line::from(vec![
                    Span::styled(fit(name, label_w), Style::default().fg(th().foreground)),
                    Span::raw(" "),
                    Span::styled(filled, Style::default().fg(color)),
                    Span::styled(empty, Style::default().fg(th().border)),
                    Span::styled(
                        format!(" {:>9}", format_size(*bytes)),
                        Style::default()
                            .fg(th().foreground)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(format!(" {pct:>3}%"), Style::default().fg(th().muted)),
                ])
            })
            .collect();
        Paragraph::new(lines).render(inner, buf);
    }
}

fn bytes_bars<'a>(
    title: &str,
    rows: &'a [(String, u64)],
    max_rows: usize,
    _color: Color,
) -> RankedBars<'a> {
    RankedBars {
        title: title.to_string(),
        rows,
        max_rows,
    }
}

/// `■ name  size  pct` legend lines for `parts`, in as many columns as fit.
fn legend_lines(
    parts: &[(String, u64, Color)],
    total: u64,
    width: u16,
    rows: u16,
) -> Vec<Line<'static>> {
    let col_w = (width as usize).clamp(24, 30);
    let cols = ((width as usize) / col_w).max(1);
    let rows = rows as usize;
    let mut lines: Vec<Vec<Span<'static>>> = vec![Vec::new(); rows.max(1)];
    for (i, (name, bytes, color)) in parts.iter().enumerate() {
        let (col, row) = (i / rows.max(1), i % rows.max(1));
        if col >= cols {
            break;
        }
        let pct = if total == 0 {
            0.0
        } else {
            *bytes as f64 * 100.0 / total as f64
        };
        let name_w = col_w - 15;
        lines[row].push(Span::styled("■ ", Style::default().fg(*color)));
        lines[row].push(Span::styled(
            fit(name, name_w),
            Style::default().fg(th().foreground),
        ));
        lines[row].push(Span::styled(
            format!("{:>9}", format_size(*bytes)),
            Style::default().fg(th().muted),
        ));
        lines[row].push(Span::styled(
            format!("{pct:>4.0}%  "),
            Style::default().fg(th().muted),
        ));
    }
    lines.into_iter().map(Line::from).collect()
}

/// A pie (not too big, not braille-noisy) with a coloured legend next to it.
struct ShareChart<'a> {
    title: String,
    rows: &'a [(String, u64)],
    max_slices: usize,
    /// Slice colours; the shared palette when `None`.
    colors: Option<Vec<Color>>,
}

impl Widget for ShareChart<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = frame_block(format!(" {} ", self.title));
        let inner = block.inner(area);
        block.render(area, buf);
        if inner.width < 12 || inner.height < 3 {
            return;
        }
        let colors = self.colors.clone().unwrap_or_else(|| palette().to_vec());
        let rows = &self.rows[..self.rows.len().min(self.max_slices)];
        let total: u64 = rows.iter().map(|(_, b)| *b).sum();
        let parts: Vec<(String, u64, Color)> = rows
            .iter()
            .enumerate()
            .map(|(i, (n, b))| (n.clone(), *b, colors[i % colors.len()]))
            .collect();
        // Cells are ~twice as tall as wide: a circle needs 2*height columns.
        let pie_h = inner.height.min(14);
        let pie_w = (pie_h * 2).min(inner.width.saturating_sub(26)).max(6);
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(pie_w + 1), Constraint::Min(10)])
            .split(inner);
        let slices: Vec<PieSlice> = parts
            .iter()
            .map(|(n, b, c)| PieSlice::new(n.as_str(), *b as f64, *c))
            .collect();
        PieChart::new(slices)
            .show_legend(false)
            .show_percentages(false)
            .high_resolution(true)
            .render(
                Rect {
                    y: cols[0].y + (cols[0].height.saturating_sub(pie_h)) / 2,
                    height: pie_h.min(cols[0].height),
                    ..cols[0]
                },
                buf,
            );
        let legend = legend_lines(&parts, total, 30, parts.len() as u16);
        let top = cols[1].y + cols[1].height.saturating_sub(legend.len() as u16) / 2;
        let rect = Rect {
            y: top,
            height: (legend.len() as u16).min(cols[1].height),
            ..cols[1]
        };
        Paragraph::new(legend).render(rect, buf);
    }
}

fn share_pie<'a>(title: &str, rows: &'a [(String, u64)], max_slices: usize) -> ShareChart<'a> {
    ShareChart {
        title: title.to_string(),
        rows,
        max_slices,
        colors: None,
    }
}

/// One stacked proportional bar (two rows tall) with a legend underneath: the
/// readable replacement for a treemap when only a handful of big items matter.
struct StackedShare<'a> {
    title: String,
    rows: &'a [(String, u64)],
}

impl Widget for StackedShare<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = frame_block(format!(" {} ", self.title));
        let inner = block.inner(area);
        block.render(area, buf);
        if inner.width < 20 || inner.height < 3 {
            return;
        }
        let colors = palette();
        let shown = self.rows.len().min(8);
        let total: u64 = self.rows.iter().map(|(_, b)| *b).sum();
        let parts: Vec<(String, u64, Color)> = self.rows[..shown]
            .iter()
            .enumerate()
            .map(|(i, (n, b))| (n.clone(), *b, colors[i % colors.len()]))
            .collect();
        let shown_total: u64 = parts.iter().map(|(_, b, _)| *b).sum();
        let mut parts = parts;
        if total > shown_total {
            parts.push(("others".into(), total - shown_total, th().border));
        }
        // Largest-remainder widths so the segments always add up to the full width.
        let w = inner.width as usize;
        let mut widths: Vec<usize> = parts
            .iter()
            .map(|(_, b, _)| (*b as f64 / total.max(1) as f64 * w as f64) as usize)
            .collect();
        let mut left = w.saturating_sub(widths.iter().sum());
        let mut order: Vec<usize> = (0..parts.len()).collect();
        order.sort_by_key(|&i| std::cmp::Reverse(parts[i].1));
        for i in order.into_iter().cycle().take(left) {
            widths[i] += 1;
            left -= 1;
            if left == 0 {
                break;
            }
        }
        let bar: Vec<Span> = parts
            .iter()
            .zip(&widths)
            .map(|((_, _, c), n)| Span::styled("█".repeat(*n), Style::default().fg(*c)))
            .collect();
        Paragraph::new(vec![Line::from(bar.clone()), Line::from(bar)])
            .render(Rect { height: 2, ..inner }, buf);
        let legend_h = inner.height - 3;
        if legend_h > 0 {
            let rows = (parts.len() as u16)
                .div_ceil(((inner.width as usize / 30).max(1)) as u16)
                .min(legend_h)
                .max(1);
            Paragraph::new(legend_lines(&parts, total, inner.width, rows)).render(
                Rect {
                    y: inner.y + 3,
                    height: legend_h,
                    ..inner
                },
                buf,
            );
        }
    }
}

/// A single-value gauge: label on the left, filled bar with the percentage on the right.
struct PercentBar {
    title: String,
    pct: f64,
    label: String,
    color: Color,
}

impl Widget for PercentBar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = frame_block(format!(" {} ", self.title));
        let inner = block.inner(area);
        block.render(area, buf);
        if inner.width < 16 || inner.height == 0 {
            return;
        }
        let w = inner.width as usize;
        let pct_txt = format!(" {:>5.1}%", self.pct);
        let label_w = self
            .label
            .chars()
            .count()
            .min(w.saturating_sub(pct_txt.len() + 14));
        let bar_w = w.saturating_sub(label_w + pct_txt.len() + 2).max(4);
        let (filled, empty) = bar_text(self.pct / 100.0, bar_w);
        Paragraph::new(Line::from(vec![
            Span::styled(
                clip(&self.label, label_w),
                Style::default().fg(th().foreground),
            ),
            Span::raw("  "),
            Span::styled(filled, Style::default().fg(self.color)),
            Span::styled(empty, Style::default().fg(th().border)),
            Span::styled(
                pct_txt,
                Style::default().fg(self.color).add_modifier(Modifier::BOLD),
            ),
        ]))
        .render(inner, buf);
    }
}

fn percent_gauge(title: &str, pct: f64, label: String, color: Color) -> PercentBar {
    PercentBar {
        title: title.to_string(),
        pct: pct.clamp(0.0, 100.0),
        label,
        color,
    }
}

fn heading(text: String) -> Line<'static> {
    Line::from(Span::styled(
        text,
        Style::default()
            .fg(th().accent)
            .add_modifier(Modifier::BOLD),
    ))
}

/// Split `area` into a main pane and (when there is room) a side pane:
/// side by side when wide, stacked when narrow but tall, otherwise only the main pane.
fn two_panes(area: Rect) -> (Rect, Option<Rect>) {
    if area.width >= 90 {
        let c = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
            .split(area);
        (c[0], Some(c[1]))
    } else if area.height >= 20 {
        let c = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);
        (c[0], Some(c[1]))
    } else {
        (area, None)
    }
}

fn popup(f: &mut Frame, area: Rect, w_pct: u16, h_pct: u16, title: &str, hint: &str) -> Rect {
    // Small terminals: use the whole screen instead of a cramped popup.
    let (w_pct, h_pct) = if area.width < 100 || area.height < 32 {
        (100, 100)
    } else {
        (w_pct, h_pct)
    };
    let w = (area.width * w_pct / 100).min(area.width);
    let h = (area.height * h_pct / 100).min(area.height);
    let rect = Rect::new(
        area.x + (area.width - w) / 2,
        area.y + (area.height - h) / 2,
        w,
        h,
    );
    f.render_widget(Clear, rect);
    let block = Block::default()
        .title(title.to_string())
        .title_bottom(Line::from(hint.to_string()).right_aligned())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(th().accent))
        .style(
            Style::default()
                .bg(crate::theme::bg(&th()))
                .fg(th().foreground),
        );
    let inner = block.inner(rect);
    f.render_widget(block, rect);
    inner
}

fn disk_gauges(f: &mut Frame, app: &App, area: Rect, reclaimable: u64) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);
    match cleansys_core::utils::disk_usage() {
        Some(d) => {
            let pct = d.used_percent();
            let color = if pct >= 90.0 {
                th().error
            } else if pct >= 75.0 {
                th().warning
            } else {
                th().success
            };
            f.render_widget(
                percent_gauge(
                    "Disk used",
                    pct,
                    format!(
                        "{} of {} used · {} free",
                        format_size(d.used()),
                        format_size(d.total),
                        format_size(d.free)
                    ),
                    color,
                ),
                cols[0],
            );
            let share = if d.used() == 0 {
                0.0
            } else {
                reclaimable as f64 * 100.0 / d.used() as f64
            };
            f.render_widget(
                percent_gauge(
                    "Reclaimable (of used)",
                    share,
                    format!(
                        "{} · {:.1}% of used space{}",
                        format_size(reclaimable),
                        share,
                        if app.board.is_scanning() {
                            " · measuring…"
                        } else {
                            ""
                        }
                    ),
                    th().accent,
                ),
                cols[1],
            );
        }
        None => {
            f.render_widget(
                Paragraph::new("Disk usage is not available on this platform")
                    .block(frame_block(" Disk ")),
                area,
            );
        }
    }
}

// ── dashboard ──────────────────────────────────────────────────────────

pub fn render_dashboard(f: &mut Frame, app: &App, area: Rect) {
    let inner = popup(
        f,
        area,
        94,
        92,
        " System dashboard ",
        " Esc/D close · R rescan ",
    );
    let d = dashboard_data(app);
    let roomy = inner.height >= 26;
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(if inner.height >= 12 { 3 } else { 0 }),
            Constraint::Min(6),
            Constraint::Length(if roomy {
                (inner.height / 3).clamp(6, 9)
            } else {
                0
            }),
        ])
        .split(inner);
    if rows[0].height > 0 {
        disk_gauges(f, app, rows[0], d.total);
    }

    if d.by_category.is_empty() {
        let msg = if app.board.is_scanning() {
            "⠿ Measuring what every cleaner can free…"
        } else {
            "Nothing to clean right now — the system is tidy"
        };
        f.render_widget(
            Paragraph::new(heading(msg.to_string())).block(frame_block(" Reclaimable ")),
            rows[1],
        );
        return;
    }

    let (left, right) = two_panes(rows[1]);
    f.render_widget(
        bytes_bars(
            "Reclaimable by category",
            &d.by_category,
            (left.height.saturating_sub(2) as usize).clamp(1, 8),
            th().accent,
        ),
        left,
    );
    let risk_rows: Vec<(String, u64)> = [
        ("Safe", d.by_risk[0]),
        ("Moderate", d.by_risk[1]),
        ("Caution", d.by_risk[2]),
    ]
    .into_iter()
    .filter(|(_, b)| *b > 0)
    .map(|(n, b)| (n.to_string(), b))
    .collect();
    let risk_colors: Vec<Color> = [
        (d.by_risk[0], th().success),
        (d.by_risk[1], th().warning),
        (d.by_risk[2], th().error),
    ]
    .into_iter()
    .filter(|(b, _)| *b > 0)
    .map(|(_, c)| c)
    .collect();
    if let Some(right) = right {
        f.render_widget(
            ShareChart {
                title: "By risk".into(),
                rows: &risk_rows,
                max_slices: 3,
                colors: Some(risk_colors),
            },
            right,
        );
    }
    if rows[2].height == 0 {
        return;
    }

    f.render_widget(
        StackedShare {
            title: format!(
                "Biggest cleaners · {} reclaimable in total",
                format_size(d.total)
            ),
            rows: &d.top,
        },
        rows[2],
    );
}

// ── per-cleaner detail ─────────────────────────────────────────────────

pub fn render_detail(f: &mut Frame, app: &App, area: Rect) {
    let inner = popup(
        f,
        area,
        92,
        88,
        " Cleaner details ",
        " ↑/↓ other cleaner · ←/Esc close ",
    );
    let Some((ci, ii)) = app.current_item() else {
        f.render_widget(Paragraph::new("No cleaner selected"), inner);
        return;
    };
    let item = &app.categories[ci].items[ii];
    // Compact on short terminals: no description / gauge rows.
    let tall = inner.height >= 22;
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(if tall { 4 } else { 2 }),
            Constraint::Length(if tall { 3 } else { 0 }),
            Constraint::Min(3),
        ])
        .split(inner);

    let risk = match item.risk {
        Risk::Safe => ("safe", th().success),
        Risk::Moderate => ("moderate", th().warning),
        Risk::Caution => ("caution", th().error),
    };
    let info = app.board.get(ci, ii);
    let size_line = match info {
        Some(i) if i.error.is_some() => {
            format!("! scan failed: {}", i.error.clone().unwrap_or_default())
        }
        Some(i) => format!("Would free {} in {} item(s)", format_size(i.bytes), i.items),
        None => "Not measured yet…".to_string(),
    };
    f.render_widget(
        Paragraph::new(
            vec![
                Line::from(vec![
                    Span::styled(
                        item.name.clone(),
                        Style::default()
                            .fg(th().foreground)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("  "),
                    Span::styled(format!("[{}]", risk.0), Style::default().fg(risk.1)),
                    Span::styled(
                        if item.requires_root {
                            "  needs root"
                        } else {
                            ""
                        },
                        Style::default().fg(th().warning),
                    ),
                ]),
                Line::from(Span::styled(size_line, Style::default().fg(th().accent))),
            ]
            .into_iter()
            .chain(tall.then(|| {
                Line::from(Span::styled(
                    item.description.clone(),
                    Style::default().fg(th().secondary),
                ))
            }))
            .collect::<Vec<_>>(),
        ),
        rows[0],
    );

    let Some(info) = info.filter(|i| !i.entries.is_empty()) else {
        f.render_widget(
            Paragraph::new("Nothing to chart — no paths found for this cleaner.")
                .block(frame_block(" Paths ")),
            rows[2],
        );
        return;
    };

    // How much of this cleaner is ticked.
    let picked = app.board.item_selected_bytes(ci, ii);
    let pct = if info.bytes == 0 {
        0.0
    } else {
        picked as f64 * 100.0 / info.bytes as f64
    };
    let listed: u64 = info.entries.iter().map(|e| e.bytes).sum();
    if tall {
        f.render_widget(
            percent_gauge(
                "Selected for cleaning",
                if item.selected { pct } else { 0.0 },
                format!(
                    "{} of {}{}",
                    format_size(if item.selected { picked } else { 0 }),
                    format_size(info.bytes),
                    if info.entries.len() >= cleansys_core::scan::MAX_ENTRIES {
                        format!(" · top {} paths listed", info.entries.len())
                    } else {
                        String::new()
                    }
                ),
                th().success,
            ),
            rows[1],
        );
    }

    let (left, right_area) = two_panes(rows[2]);
    let top: Vec<(String, u64)> = info
        .entries
        .iter()
        .map(|e| {
            let name = if e.label.is_empty() {
                base_name(&e.path)
            } else {
                &e.label
            };
            (name.replace('\'', ""), e.bytes)
        })
        .collect();
    f.render_widget(
        bytes_bars(
            "Biggest paths",
            &top,
            (left.height.saturating_sub(2) as usize).clamp(1, 10),
            th().accent,
        ),
        left,
    );
    let Some(right_area) = right_area else {
        return;
    };

    // Pie: biggest five paths vs everything else.
    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(6),
            Constraint::Length(if right_area.height >= 16 && info.entries.len() >= 8 {
                5
            } else {
                0
            }),
        ])
        .split(right_area);
    let mut shares: Vec<(String, u64)> =
        top.iter().take(5).map(|(n, b)| (clip(n, 14), *b)).collect();
    let rest = info
        .bytes
        .saturating_sub(shares.iter().map(|(_, b)| b).sum::<u64>());
    if rest > 0 {
        shares.push(("others".to_string(), rest));
    }
    f.render_widget(share_pie("Share of the total", &shares, 6), right[0]);
    if right[1].height == 0 {
        return;
    }
    let curve: Vec<f64> = info.entries.iter().map(|e| e.bytes as f64).collect();
    f.render_widget(
        Sparkline::new(curve)
            .color(th().accent)
            .block(frame_block(format!(
                " Size curve · {} path(s), {} listed ",
                info.items,
                format_size(listed)
            ))),
        right[1],
    );
}

// ── clean report ───────────────────────────────────────────────────────

pub fn render_report(f: &mut Frame, app: &App, area: Rect) {
    let live = app.is_running;
    let inner = popup(
        f,
        area,
        94,
        92,
        if live {
            " Cleaning report (live) "
        } else {
            " Cleaning report "
        },
        " Esc/Enter close · P reopen ",
    );
    let r = report_data(app);
    let summary = app.run_summary.as_ref();
    let tall = inner.height >= 20;
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(if tall { 4 } else { 2 }),
            Constraint::Length(if tall { 3 } else { 0 }),
            Constraint::Min(3),
        ])
        .split(inner);

    let (icon, color) = match summary {
        Some(s) if s.cancelled => ("⏹ Cancelled", th().warning),
        Some(s) if s.failed > 0 => ("! Done with problems", th().warning),
        Some(_) => ("✓ Done", th().success),
        None => ("⠿ Running", th().accent),
    };
    let elapsed = summary.map_or_else(|| app.get_elapsed_time(), |s| s.elapsed.clone());
    f.render_widget(
        Paragraph::new(vec![
            Line::from(vec![
                Span::styled(
                    format!("{icon} — freed {}", format_size(r.freed)),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("  in {elapsed}"),
                    Style::default().fg(th().secondary),
                ),
            ]),
            Line::from(Span::styled(
                format!(
                    "{} cleaner(s) ran · {} freed something · {} nothing to clean · {} failed · {} item(s) removed",
                    r.ok + r.failed.len(),
                    r.cleaners.len(),
                    r.nothing,
                    r.failed.len(),
                    r.items
                ),
                Style::default().fg(th().secondary),
            )),
        ]),
        rows[0],
    );

    // Freed out of the planned run, and the disk before/after as free space.
    let gauges = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[1]);
    let planned = app.run_planned.max(r.freed);
    let pct = if planned == 0 {
        0.0
    } else {
        r.freed as f64 * 100.0 / planned as f64
    };
    f.render_widget(
        percent_gauge(
            "Freed of planned",
            pct,
            format!(
                "{} of {} ({pct:.0}%)",
                format_size(r.freed),
                format_size(planned)
            ),
            th().success,
        ),
        gauges[0],
    );
    match (app.disk_before, cleansys_core::utils::disk_usage()) {
        (Some(before), Some(now)) => {
            let gain = now.free as i64 - before.free as i64;
            f.render_widget(
                percent_gauge(
                    "Disk used now",
                    now.used_percent(),
                    format!(
                        "{:.1}% → was {:.1}% · free {} ({}{})",
                        now.used_percent(),
                        before.used_percent(),
                        format_size(now.free),
                        if gain >= 0 { "+" } else { "-" },
                        format_size(gain.unsigned_abs())
                    ),
                    th().accent,
                ),
                gauges[1],
            );
        }
        _ => f.render_widget(frame_block(" Disk "), gauges[1]),
    }

    if r.cleaners.is_empty() {
        let mut lines = vec![heading("Nothing was freed.".to_string())];
        for (n, m) in &r.failed {
            lines.push(Line::from(Span::styled(
                format!("✗ {n}: {m}"),
                Style::default().fg(th().error),
            )));
        }
        f.render_widget(
            Paragraph::new(lines).block(frame_block(" Result ")),
            rows[2],
        );
        return;
    }
    let (left, right_area) = two_panes(rows[2]);
    f.render_widget(
        bytes_bars(
            "Freed per cleaner",
            &r.cleaners,
            (left.height.saturating_sub(2) as usize).clamp(1, 10),
            th().success,
        ),
        left,
    );
    let Some(right_area) = right_area else {
        return;
    };
    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints(if r.failed.is_empty() || right_area.height < 14 {
            vec![Constraint::Min(6)]
        } else {
            vec![
                Constraint::Min(6),
                Constraint::Length(2 + r.failed.len().min(4) as u16),
            ]
        })
        .split(right_area);
    f.render_widget(share_pie("Freed by category", &r.by_category, 6), right[0]);
    if right.len() > 1 {
        let lines: Vec<Line> = r
            .failed
            .iter()
            .take(4)
            .map(|(n, m)| {
                Line::from(Span::styled(
                    format!("✗ {}: {}", clip(n, 22), clip(m, 40)),
                    Style::default().fg(th().error),
                ))
            })
            .collect();
        f.render_widget(
            Paragraph::new(lines).block(frame_block(" Problems ")),
            right[1],
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn units_scale() {
        assert_eq!(unit_for(10).1, "B");
        assert_eq!(unit_for(5 * 1024).1, "KB");
        assert_eq!(unit_for(5 * 1024 * 1024).1, "MB");
        assert_eq!(unit_for(5 * 1024 * 1024 * 1024).1, "GB");
    }

    #[test]
    fn clip_shortens() {
        assert_eq!(clip("abcdef", 4), "abc…");
        assert_eq!(clip("abc", 4), "abc");
    }

    #[test]
    fn base_name_of_paths() {
        assert_eq!(base_name("/a/b/c/"), "c");
        assert_eq!(base_name("plain"), "plain");
    }
}
