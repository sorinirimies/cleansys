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
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use tui_chart::{BarChart, BarGroup, BarOrientation, Gauge, Sparkline, Treemap, TreemapItem};
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
        t.text_secondary,
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

/// Horizontal bar chart of `(label, bytes)` rows (at most `max_rows`).
fn bytes_bars(title: &str, rows: &[(String, u64)], max_rows: usize, color: Color) -> BarChart {
    let rows = &rows[..rows.len().min(max_rows)];
    let (div, unit) = unit_for(rows.iter().map(|(_, b)| *b).max().unwrap_or(0));
    let groups = rows
        .iter()
        .map(|(n, b)| BarGroup::new(clip(n, 18), [*b as f64 / div]))
        .collect();
    BarChart::new(groups)
        .orientation(BarOrientation::Horizontal)
        .group_gap(0)
        .colors(vec![color])
        .show_values(true)
        .block(frame_block(format!(" {title} ({unit}) ")))
}

fn share_pie<'a>(title: &str, rows: &'a [(String, u64)], max_slices: usize) -> PieChart<'a> {
    let colors = palette();
    let slices = rows
        .iter()
        .take(max_slices)
        .enumerate()
        .map(|(i, (n, b))| PieSlice::new(n.as_str(), *b as f64, colors[i % colors.len()]))
        .collect();
    PieChart::new(slices)
        .show_legend(true)
        .show_percentages(true)
        .high_resolution(true)
        .block(frame_block(format!(" {title} ")))
}

/// A single-value gauge with a label.
fn percent_gauge(title: &str, pct: f64, label: String, color: Color) -> Gauge<'static> {
    Gauge::new(pct.clamp(0.0, 100.0))
        .color(color)
        .label(label)
        .block(frame_block(format!(" {title} ")))
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
        .style(Style::default().bg(th().bg).fg(th().text));
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
                    "💽 Disk used",
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
                    "♻ Reclaimable (of used)",
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
                    .block(frame_block(" 💽 Disk ")),
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
        " 📊 System dashboard ",
        " Esc/D close · R rescan ",
    );
    let d = dashboard_data(app);
    let roomy = inner.height >= 26;
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(if inner.height >= 12 { 3 } else { 0 }),
            Constraint::Min(6),
            Constraint::Length(if roomy { (inner.height / 3).max(6) } else { 0 }),
        ])
        .split(inner);
    if rows[0].height > 0 {
        disk_gauges(f, app, rows[0], d.total);
    }

    if d.by_category.is_empty() {
        let msg = if app.board.is_scanning() {
            "⠿ Measuring what every cleaner can free…"
        } else {
            "Nothing to clean right now — the system is tidy ✨"
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
    let colors = [th().success, th().warning, th().error];
    let slices: Vec<PieSlice> = [
        ("Safe", d.by_risk[0], colors[0]),
        ("Moderate", d.by_risk[1], colors[1]),
        ("Caution", d.by_risk[2], colors[2]),
    ]
    .into_iter()
    .filter(|(_, b, _)| *b > 0)
    .map(|(n, b, c)| PieSlice::new(n, b as f64, c))
    .collect();
    if let Some(right) = right {
        if risk_rows.is_empty() {
            f.render_widget(frame_block(" By risk "), right);
        } else {
            f.render_widget(
                PieChart::new(slices)
                    .show_legend(true)
                    .show_percentages(true)
                    .high_resolution(true)
                    .block(frame_block(" By risk ")),
                right,
            );
        }
    }
    if rows[2].height == 0 {
        return;
    }

    let items = d
        .top
        .iter()
        .map(|(n, b)| TreemapItem::new(format!("{} {}", clip(n, 18), format_size(*b)), *b as f64));
    f.render_widget(
        Treemap::new(items)
            .show_values(false)
            .gradient(th().border, th().success)
            .block(frame_block(format!(
                " Biggest cleaners · {} reclaimable in total ",
                format_size(d.total)
            ))),
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
        " 🔎 Cleaner details ",
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
            format!("⚠ scan failed: {}", i.error.clone().unwrap_or_default())
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
                        Style::default().fg(th().text).add_modifier(Modifier::BOLD),
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
                    Style::default().fg(th().text_secondary),
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
            Constraint::Length(if right_area.height >= 16 { 5 } else { 0 }),
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
            " 🧹 Cleaning report (live) "
        } else {
            " 🎉 Cleaning report "
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
        Some(s) if s.failed > 0 => ("⚠ Done with problems", th().warning),
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
                    Style::default().fg(th().text_secondary),
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
                Style::default().fg(th().text_secondary),
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
                    "💽 Disk used now",
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
        _ => f.render_widget(frame_block(" 💽 Disk "), gauges[1]),
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
