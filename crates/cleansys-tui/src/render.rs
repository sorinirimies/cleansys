use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Chart, Clear, Dataset, List, ListItem, Paragraph, Wrap},
    Frame,
};
// Using tui-checkbox library for consistent checkbox symbols across the application
use tui_checkbox::symbols as checkbox_symbols;
use tui_spinner::{FluxFrames, FluxSpinner};

use crate::app::{App, ChartType, CleanedItemType};
use crate::pie_chart::create_pie_chart_from_distribution;
use cleansys_core::{format_size, Status};

pub fn ui(f: &mut Frame, app: &mut App) {
    // Start every frame from a clean slate so nothing from the previous view
    // (title text, progress panels) bleeds through when views switch.
    f.render_widget(Clear, f.area());

    // Update animation frame if needed
    app.update_animation();

    // Adjust title and footer heights based on terminal size
    let (title_height, footer_height, min_content_height) = if app.terminal_height < 20 {
        // Very small terminals: minimal UI
        (2, 2, 6)
    } else if app.terminal_height < 30 {
        // Small terminals: compact UI
        (2, 2, 8)
    } else if app.terminal_height < 40 {
        // Medium terminals: standard UI
        (3, 3, 10)
    } else {
        // Large terminals: spacious UI
        (3, 3, 12)
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
        render_help(f, chunks[1]);
    } else if app.is_running || app.show_progress_screen {
        render_progress_screen(f, app, chunks[1]);
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
    } else if app.schedule_open {
        render_schedule(f, app, f.area());
    } else if app.preview_open {
        render_preview(f, app, f.area());
    }
}

fn render_title(f: &mut Frame, app: &App, area: Rect) {
    // Adjust title content based on terminal width
    let title_lines = if app.terminal_width < 80 {
        // Narrow terminals: shortened version with dimensions indicator
        let mut lines = vec![Line::from(vec![
            Span::styled(
                "Cleansys",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" - System Cleaner"),
            if app.terminal_width < 60 || app.terminal_height < 20 {
                Span::styled(
                    format!(" [{}x{}]", app.terminal_width, app.terminal_height),
                    Style::default().fg(Color::DarkGray),
                )
            } else {
                Span::raw("")
            },
        ])];

        // Add help line
        lines.push(Line::from(vec![
            Span::styled("?", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" help | "),
            Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" quit"),
        ]));

        lines
    } else {
        // Wide terminals: full version
        vec![
            Line::from(vec![
                Span::styled(
                    "Cleansys",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" - Modern System Cleaner for Linux, macOS & Windows"),
            ]),
            Line::from(vec![
                Span::raw("Press "),
                Span::styled("?", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" for help, "),
                Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to quit"),
            ]),
        ]
    };

    let title = Paragraph::new(title_lines).block(Block::default().borders(Borders::BOTTOM));

    f.render_widget(title, area);

    // Animated "loading" spinner (via the tui-spinner crate) in the
    // top-right corner of the title bar while a cleaning run is active.
    if app.is_running && area.width > 14 {
        let spinner_width = 12u16;
        let spinner_area = Rect {
            x: area.x + area.width.saturating_sub(spinner_width + 1),
            y: area.y,
            width: spinner_width,
            height: 1,
        };

        let label = Line::from(vec![
            Span::raw(" "),
            Span::styled(
                "RUNNING",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
        ]);
        let label_width = label.width() as u16;

        let (label_area, glyph_area) = (
            Rect {
                width: label_width.min(spinner_width),
                ..spinner_area
            },
            Rect {
                x: spinner_area.x + label_width.min(spinner_width),
                width: spinner_area.width.saturating_sub(label_width),
                ..spinner_area
            },
        );

        f.render_widget(Paragraph::new(label), label_area);
        f.render_widget(
            FluxSpinner::new(app.animation_frame as u64)
                .frames(FluxFrames::CLASSIC)
                .color(Color::Cyan),
            glyph_area,
        );
    }
}

fn render_main_content(f: &mut Frame, app: &mut App, area: Rect) {
    // Responsive: a sidebar of categories on wide terminals; on narrow ones the
    // sidebar is replaced by a one-line category bar (Tab / Shift+Tab to switch).
    let use_sidebar = area.width >= 90;
    let content = if use_sidebar {
        let sidebar_w = (area.width * 28 / 100).clamp(26, 40);
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(sidebar_w), Constraint::Min(30)])
            .split(area);
        render_categories(f, app, chunks[0]);
        chunks[1]
    } else {
        area
    };

    if app.detailed_view {
        render_details(f, app, content);
    } else {
        render_cleaner_pane(f, app, content, !use_sidebar);
    }
}

fn render_progress_screen(f: &mut Frame, app: &mut App, area: Rect) {
    // Render both progress and details in a unified view
    render_unified_progress_view(f, app, area);
}

fn render_unified_progress_view(f: &mut Frame, app: &mut App, area: Rect) {
    // Update app counters first
    app.update_counters();

    // Ultra-compact layout for extremely small terminals
    if area.width < 50 || area.height < 15 {
        render_ultra_compact_view(f, app, area);
        return;
    }

    // Show 2-section layout: Combined Progress Overview + Removed Items
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(if app.terminal_height >= 35 {
                55
            } else if app.terminal_height >= 25 {
                50
            } else {
                45
            }), // Combined progress overview - responsive percentage
            Constraint::Percentage(if app.terminal_height >= 35 {
                45
            } else if app.terminal_height >= 25 {
                50
            } else {
                55
            }), // Removed items window - responsive percentage
        ])
        .margin(1)
        .split(area);

    // ===== TOP SECTION: Combined Progress Overview =====
    render_combined_progress_overview(f, app, main_chunks[0]);

    // ===== BOTTOM SECTION: Removed Items Window =====
    render_removed_items_window(f, app, main_chunks[1]);
}

fn render_combined_progress_overview(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title("📊 Progress Overview & Operations")
        .title_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner_area = block.inner(area);

    // Responsive height allocation based on terminal size - make chart area bigger
    let stats_height = if area.height < 15 {
        5 // Minimal height for very short terminals
    } else if area.height < 20 {
        7 // Compact layout for short terminals
    } else if area.height < 25 {
        9 // Medium layout
    } else {
        12 // Standard height for normal terminals - much bigger for better chart
    };

    // Split into top (stats + chart) and bottom (operations)
    let main_sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(stats_height), // Stats and chart section
            Constraint::Min(6),               // Operations section
        ])
        .split(inner_area);

    // Top section: Progress stats and chart
    render_progress_stats_and_chart(f, app, main_sections[0]);

    // Bottom section: Operations summary
    render_operations_summary(f, app, main_sections[1]);

    f.render_widget(block, area);
}

fn render_progress_stats_and_chart(f: &mut Frame, app: &App, area: Rect) {
    let elapsed_time = app.get_elapsed_time();
    let total_ops = app.operation_count;
    let completed_ops = total_ops.saturating_sub(app.errors_count);
    let progress_percent = completed_ops
        .checked_mul(100)
        .and_then(|v| v.checked_div(total_ops))
        .unwrap_or(0);

    // Responsive layout based on terminal width - give chart much more space
    let show_chart = area.width >= 80; // Hide chart on narrow terminals

    let horizontal_chunks = if show_chart {
        let stats_percent = if area.width < 100 {
            45 // Much more space for chart on narrow terminals
        } else if area.width < 130 {
            40 // Balanced layout for medium terminals - chart gets 60%
        } else {
            35 // Even more space for chart on wide terminals - chart gets 65%
        };

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(stats_percent),
                Constraint::Percentage(100 - stats_percent),
            ])
            .split(area)
    } else {
        // Use full width for stats when chart is hidden
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(100)])
            .split(area)
    };

    // Left side: Progress stats
    let stats_lines = vec![
        Line::from(vec![
            Span::styled(
                "Progress: ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{}%", progress_percent),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!(" ({}/{})", completed_ops, total_ops)),
            Span::raw("  ⏱️ "),
            Span::styled(
                elapsed_time,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::raw("█".repeat((progress_percent * 35) / 100)),
            Span::styled(
                "░".repeat(35 - (progress_percent * 35) / 100),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        Line::from(vec![
            Span::styled("✅ ", Style::default().fg(Color::Green)),
            Span::styled(
                format!("{} OK", completed_ops),
                Style::default().fg(Color::Green),
            ),
            Span::raw("  "),
            Span::styled("⚡ ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!(
                    "{} Active",
                    if app.is_running {
                        total_ops.saturating_sub(completed_ops)
                    } else {
                        0
                    }
                ),
                Style::default().fg(Color::Yellow),
            ),
            Span::raw("  "),
            Span::styled("❌ ", Style::default().fg(Color::Red)),
            Span::styled(
                format!("{} Errors", app.errors_count),
                Style::default().fg(Color::Red),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "💾 Total freed: ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format_size(app.total_bytes_cleaned),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ];

    let stats_para = Paragraph::new(stats_lines);
    f.render_widget(stats_para, horizontal_chunks[0]);

    // Right side: Chart (only if terminal is wide enough)
    if show_chart && horizontal_chunks.len() > 1 {
        match app.chart_type {
            ChartType::Bar => {
                render_vertical_bar_chart(f, app, horizontal_chunks[1]);
            }
            ChartType::PieCount => {
                render_pie_chart_distribution(f, app, horizontal_chunks[1]);
            }
            ChartType::PieSize => {
                render_pie_chart_size_distribution(f, app, horizontal_chunks[1]);
            }
        }
    }
}

fn render_ultra_compact_view(f: &mut Frame, app: &App, area: Rect) {
    let elapsed_time = app.get_elapsed_time();
    let total_ops = app.operation_count;
    let completed_ops = total_ops.saturating_sub(app.errors_count);
    let progress_percent = completed_ops
        .checked_mul(100)
        .and_then(|v| v.checked_div(total_ops))
        .unwrap_or(0);

    // Ultra-compact single block with essential info only
    let compact_lines = vec![
        Line::from(vec![Span::styled(
            format!("Cleansys [{}x{}]", area.width, area.height),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled(
                format!("{}% ", progress_percent),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("█".repeat(
                ((progress_percent * (area.width.saturating_sub(10) as usize)) / 100).min(30),
            )),
        ]),
        Line::from(vec![
            Span::styled(
                format!("✅{} ❌{} ", completed_ops, app.errors_count),
                Style::default().fg(Color::White),
            ),
            Span::styled(
                format_size(app.total_bytes_cleaned),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                format!("⏱️{} ", elapsed_time),
                Style::default().fg(Color::Cyan),
            ),
            Span::styled(
                if app.is_running { "RUNNING" } else { "DONE" },
                Style::default().fg(if app.is_running {
                    Color::Yellow
                } else {
                    Color::Green
                }),
            ),
        ]),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let para = Paragraph::new(compact_lines)
        .block(block)
        .wrap(Wrap { trim: true });

    f.render_widget(para, area);
}

fn render_vertical_bar_chart(f: &mut Frame, app: &App, area: Rect) {
    // Get real data from cleaned items
    let category_distribution = app.get_category_distribution();

    // Only show chart if we have real data
    if !category_distribution.is_empty() {
        // Use real data, limit to top 6 categories to fit in chart
        let limited_data: Vec<_> = category_distribution.iter().take(6).collect();
        let max_count = limited_data
            .iter()
            .map(|(_, count, _)| *count)
            .max()
            .unwrap_or(1) as f64;

        let chart_data: Vec<(f64, f64)> = limited_data
            .iter()
            .enumerate()
            .map(|(i, (_, count, _))| (i as f64, *count as f64))
            .collect();

        let category_names: Vec<&str> = limited_data
            .iter()
            .map(|(name, _, _)| {
                // Truncate label for narrow terminals
                if area.width < 80 {
                    if name.len() > 6 {
                        &name[..6]
                    } else {
                        name
                    }
                } else if area.width < 100 {
                    if name.len() > 8 {
                        &name[..8]
                    } else {
                        name
                    }
                } else if name.len() > 12 {
                    &name[..12]
                } else {
                    name
                }
            })
            .collect();

        // Create dataset for the chart
        let dataset = Dataset::default()
            .name("Cleaned Items")
            .marker(symbols::Marker::Block)
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .data(&chart_data);

        // Create x-axis labels
        let x_labels = if category_names.len() <= 3 {
            vec![
                Span::raw(category_names.first().unwrap_or(&"").to_string()),
                Span::raw(category_names.get(1).unwrap_or(&"").to_string()),
                Span::raw(category_names.get(2).unwrap_or(&"").to_string()),
            ]
        } else {
            vec![
                Span::raw(category_names.first().unwrap_or(&"").to_string()),
                Span::raw(
                    category_names
                        .get(category_names.len() / 2)
                        .unwrap_or(&"")
                        .to_string(),
                ),
                Span::raw(category_names.last().unwrap_or(&"").to_string()),
            ]
        };

        // Create y-axis labels
        let y_max = (max_count * 1.1).max(1.0); // Add 10% padding, minimum 1
        let y_labels = vec![
            Span::raw("0"),
            Span::raw(format!("{}", (y_max / 2.0) as u64)),
            Span::raw(format!("{}", y_max as u64)),
        ];

        let chart = Chart::new(vec![dataset])
            .block(
                Block::default()
                    .title(if area.width < 50 {
                        "Items (Bar)"
                    } else {
                        "Items Distribution (Bar Chart)"
                    })
                    .title_style(
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    )
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .x_axis(
                Axis::default()
                    .title(if area.width >= 80 { "Categories" } else { "" })
                    .style(Style::default().fg(Color::White))
                    .bounds([0.0, (category_names.len().max(3) - 1) as f64])
                    .labels(x_labels),
            )
            .y_axis(
                Axis::default()
                    .title(if area.width >= 80 { "Count" } else { "" })
                    .style(Style::default().fg(Color::White))
                    .bounds([0.0, y_max])
                    .labels(y_labels),
            );

        f.render_widget(chart, area);
    }
}

fn render_operations_summary(f: &mut Frame, app: &App, area: Rect) {
    // Two columns — the real queue of this run, split by user land / root.
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(48), // User operations
            Constraint::Percentage(4),  // Spacing
            Constraint::Percentage(48), // System operations
        ])
        .split(area);

    let column = |root: bool, title: &'static str, color: Color| -> Vec<ListItem<'static>> {
        let mut rows = vec![
            ListItem::new(Line::from(Span::styled(
                title,
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ))),
            ListItem::new(Line::from("")),
        ];
        let capacity = (area.height as usize).saturating_sub(2);
        let queue: Vec<&cleansys_core::CleanerItem> = app
            .categories
            .iter()
            .flat_map(|c| &c.items)
            .filter(|i| i.requires_root == root && (i.selected || i.status.is_some()))
            .collect();
        for item in queue.iter().take(capacity.saturating_sub(1).max(1)) {
            let (glyph, style) = match &item.status {
                Some(Status::Running) => (
                    Status::Running.get_animation_frame(app.animation_frame),
                    Style::default().fg(Color::Yellow),
                ),
                Some(Status::Success(_)) => ("✓", Style::default().fg(Color::Green)),
                Some(Status::Error(_)) => ("✗", Style::default().fg(Color::Red)),
                Some(Status::Pending) | None => ("•", Style::default().fg(Color::DarkGray)),
            };
            let mut spans = vec![
                Span::styled(format!("{glyph} "), style),
                Span::styled(item.name.clone(), Style::default().fg(Color::White)),
            ];
            if item.bytes_cleaned > 0 {
                spans.push(Span::styled(
                    format!("  {}", format_size(item.bytes_cleaned)),
                    Style::default().fg(Color::Green),
                ));
            }
            if root && !app.is_root {
                spans.push(Span::styled(" (sudo)", Style::default().fg(Color::Yellow)));
            }
            rows.push(ListItem::new(Line::from(spans)));
        }
        let hidden = queue
            .len()
            .saturating_sub(capacity.saturating_sub(1).max(1));
        if hidden > 0 {
            rows.push(ListItem::new(Line::from(Span::styled(
                format!("… and {hidden} more"),
                Style::default().fg(Color::DarkGray),
            ))));
        }
        if queue.is_empty() {
            rows.push(ListItem::new(Line::from(Span::styled(
                "(none)",
                Style::default().fg(Color::DarkGray),
            ))));
        }
        rows
    };

    f.render_widget(
        List::new(column(false, "👤 USER OPERATIONS", Color::Green)),
        columns[0],
    );
    f.render_widget(
        List::new(column(true, "🔒 SYSTEM OPERATIONS", Color::Yellow)),
        columns[2],
    );
}

fn render_pie_chart_distribution(f: &mut Frame, app: &App, area: Rect) {
    let category_distribution = app.get_category_distribution();

    // Only show real data from actual cleaning operations, and only when
    // there's enough room for tui-piechart to draw something meaningful.
    if !category_distribution.is_empty() && area.width >= 20 && area.height >= 8 {
        let chart = create_pie_chart_from_distribution(
            &category_distribution,
            "Items Distribution (Count)",
            false, // Use count-based distribution
        )
        .show_percentages(area.width >= 40)
        .show_legend(area.width >= 50 || area.height >= 16);

        f.render_widget(chart, area);
    }
}

fn render_pie_chart_size_distribution(f: &mut Frame, app: &App, area: Rect) {
    let category_distribution = app.get_category_distribution();

    // Only show real data from actual cleaning operations, and only when
    // there's enough room for tui-piechart to draw something meaningful.
    if !category_distribution.is_empty() && area.width >= 20 && area.height >= 8 {
        let chart = create_pie_chart_from_distribution(
            &category_distribution,
            "Items Distribution (Size)",
            true, // Use size-based distribution
        )
        .show_percentages(area.width >= 40)
        .show_legend(area.width >= 50 || area.height >= 16);

        f.render_widget(chart, area);
    }
}

fn render_removed_items_window(f: &mut Frame, app: &mut App, area: Rect) {
    let title = if app.is_running {
        "📋 Operation Progress"
    } else if app.show_progress_screen {
        "📋 Cleaning Results - Removed Items"
    } else {
        "📋 Removed Items Details"
    };

    let block = Block::default()
        .title(title)
        .title_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let inner_area = block.inner(area);

    let mut display_items = Vec::new();

    // Show operation logs if running, otherwise show removed items
    if app.is_running && !app.operation_logs.is_empty() {
        for log_entry in app.operation_logs.iter().rev().take(15) {
            let (icon, color) = if log_entry.contains("✅") {
                ("✅", Color::Green)
            } else if log_entry.contains("❌") {
                ("❌", Color::Red)
            } else if log_entry.contains("🔄") {
                ("🔄", Color::Yellow)
            } else if log_entry.contains("📊") {
                ("📊", Color::Cyan)
            } else {
                ("ℹ️", Color::White)
            };

            display_items.push(ListItem::new(Line::from(vec![
                Span::styled(format!("{} ", icon), Style::default().fg(color)),
                Span::styled(log_entry.clone(), Style::default().fg(Color::White)),
            ])));
        }
    } else {
        // Get sample cleaned items for display plus additional entries for demo
        let filtered_items = app.get_filtered_detailed_items();

        if !filtered_items.is_empty() {
            for (index, item) in filtered_items.iter().enumerate() {
                let icon = match item.item_type {
                    CleanedItemType::File => "📄",
                    CleanedItemType::Directory => "📁",
                    CleanedItemType::Log => "📝",
                };

                // File path and size on one line
                display_items.push(ListItem::new(Line::from(vec![
                    Span::styled(format!("{} ", icon), Style::default().fg(Color::Yellow)),
                    Span::styled(item.path.clone(), Style::default().fg(Color::White)),
                    Span::raw(" "),
                    Span::styled(
                        format!("({})", format_size(item.size)),
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                ])));

                // Category and cleaner info on next line (indented)
                display_items.push(ListItem::new(Line::from(vec![
                    Span::raw("   "),
                    Span::styled("📂 ", Style::default().fg(Color::Blue)),
                    Span::styled(item.category.clone(), Style::default().fg(Color::Blue)),
                    Span::raw(" • "),
                    Span::styled("🔧 ", Style::default().fg(Color::Cyan)),
                    Span::styled(item.cleaner_name.clone(), Style::default().fg(Color::Cyan)),
                ])));

                // Add spacing between entries
                if index < filtered_items.len() - 1 {
                    display_items.push(ListItem::new(Line::from(vec![])));
                }
            }
        } else if !app.is_running && app.show_progress_screen && app.total_bytes_cleaned > 0 {
            // Show summary when cleaning is complete but no detailed items
            display_items.push(ListItem::new(Line::from(vec![
                Span::styled("✅ ", Style::default().fg(Color::Green)),
                Span::styled(
                    "Cleaning completed successfully",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ])));
            display_items.push(ListItem::new(Line::from(vec![])));

            display_items.push(ListItem::new(Line::from(vec![
                Span::styled("📊 ", Style::default().fg(Color::Cyan)),
                Span::styled("Total space freed: ", Style::default().fg(Color::White)),
                Span::styled(
                    format_size(app.total_bytes_cleaned),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ])));
            display_items.push(ListItem::new(Line::from(vec![])));

            // Show which cleaners were executed
            for category in &app.categories {
                for item in &category.items {
                    if item.bytes_cleaned > 0 {
                        display_items.push(ListItem::new(Line::from(vec![
                            Span::styled("🔧 ", Style::default().fg(Color::Yellow)),
                            Span::styled(item.name.clone(), Style::default().fg(Color::White)),
                            Span::raw(": "),
                            Span::styled(
                                format_size(item.bytes_cleaned),
                                Style::default().fg(Color::Green),
                            ),
                        ])));
                    }
                }
            }

            if display_items.len() == 3 {
                // No items were cleaned with bytes > 0
                display_items.push(ListItem::new(Line::from(vec![])));
                display_items.push(ListItem::new(Line::from(vec![
                    Span::styled("ℹ️ ", Style::default().fg(Color::Blue)),
                    Span::styled(
                        "Detailed file list not available in TUI mode",
                        Style::default().fg(Color::DarkGray),
                    ),
                ])));
            }
        }
    }

    let items_list = List::new(display_items)
        .block(Block::default())
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("► ");

    f.render_stateful_widget(items_list, inner_area, &mut app.detailed_list_scroll_state);
    f.render_widget(block, area);
}

/// Colour a size by how much it is: small = normal, big = warm.
fn size_style(bytes: u64) -> Style {
    const GB: u64 = 1 << 30;
    const MB: u64 = 1 << 20;
    if bytes >= 5 * GB {
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
    } else if bytes >= 500 * MB {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    }
}

/// One animated spinner glyph from the `tui-spinner` crate, as spans that can be dropped
/// into any `Line` (the widget implements `Into<Text>`).
fn spinner_spans(app: &App) -> Vec<Span<'static>> {
    FluxSpinner::new(app.animation_frame as u64)
        .frames(FluxFrames::CLASSIC)
        .color(Color::Cyan)
        .to_lines()
        .into_iter()
        .next()
        .map(|l| l.spans)
        .unwrap_or_default()
}

/// ` (X to free)` for the footer — a spinner instead of numbers while the scan is still
/// running (the partial sum would be misleading).
fn to_free_spans(app: &App) -> Vec<Span<'static>> {
    if app.board.is_scanning() {
        let mut v = vec![Span::styled(" (", Style::default().fg(Color::DarkGray))];
        v.extend(spinner_spans(app));
        v.push(Span::styled(
            " measuring…)",
            Style::default().fg(Color::DarkGray),
        ));
        return v;
    }
    let b = app.board.selected_bytes(&app.categories);
    if b > 0 {
        vec![Span::styled(
            format!(" ({} to free)", format_size(b)),
            Style::default().fg(Color::Green),
        )]
    } else {
        Vec::new()
    }
}

/// `left` spans, then `right` pushed against the right edge of `width` columns.
fn row_line<'a>(mut left: Vec<Span<'a>>, right: Span<'a>, width: usize) -> Line<'a> {
    let used: usize = left
        .iter()
        .map(|s| s.content.chars().count())
        .sum::<usize>()
        + right.content.chars().count();
    let gap = width.saturating_sub(used).max(1);
    left.push(Span::raw(" ".repeat(gap)));
    left.push(right);
    Line::from(left)
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
            format!(" scanning {done}/{}", app.board.total),
            Style::default().fg(Color::Cyan),
        ));
        Line::from(spans)
    } else if app.board.complete() {
        Line::from(Span::styled(
            format!("{} can be freed", format_size(app.board.total_bytes())),
            Style::default().fg(Color::Green),
        ))
    } else {
        Line::default()
    }
}

fn render_categories(f: &mut Frame, app: &App, area: Rect) {
    let inner_w = area.width.saturating_sub(4) as usize;
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
            items.push(ListItem::new(Line::from(Span::styled(
                "USER LAND",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            ))));
        }
        if root && !shown_root_header {
            shown_root_header = true;
            items.push(ListItem::new(Line::from(Span::styled(
                "SYSTEM · ROOT",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            ))));
        }

        let active = i == app.category_index && app.filter.is_empty();
        let ticked = category.items.iter().filter(|it| it.selected).count();
        let name = category
            .name
            .trim_end_matches(cleansys_core::model::ROOT_SUFFIX)
            .to_string();
        let tick_text = if ticked > 0 {
            format!(" ({ticked}✓)")
        } else {
            String::new()
        };
        let right = match app.board.category_bytes_when_done(i) {
            Some(b) if b > 0 => Span::styled(format_size(b), size_style(b)),
            Some(_) => Span::styled("—", Style::default().fg(Color::DarkGray)),
            None if app.board.is_scanning() => spinner_spans(app)
                .into_iter()
                .next()
                .unwrap_or_else(|| Span::raw("…")),
            None => Span::raw(""),
        };
        // Truncate the name so the size column never gets pushed out.
        let budget = inner_w
            .saturating_sub(2 + tick_text.chars().count() + right.content.chars().count() + 1);
        let name = if name.chars().count() > budget && budget > 1 {
            format!("{}…", name.chars().take(budget - 1).collect::<String>())
        } else {
            name
        };
        let mut left = vec![Span::raw(if active { "▶ " } else { "  " }), Span::raw(name)];
        if ticked > 0 {
            left.push(Span::styled(tick_text, Style::default().fg(Color::Green)));
        }
        let style = if active {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        items.push(ListItem::new(row_line(left, right, inner_w)).style(style));
    }

    let list = List::new(items).block(
        Block::default()
            .title("📂 Categories")
            .title_bottom(scan_line(app).right_aligned())
            .borders(Borders::ALL),
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
            (true, true) => ("[x]", Style::default().fg(Color::Green)),
            (true, false) => ("[ ]", Style::default().fg(Color::White)),
            (false, _) => (" · ", Style::default().fg(Color::DarkGray)),
        };
        let size = format_size(e.bytes);
        let head = format!("    {mark} {}", e.label);
        let used = head.chars().count() + size.chars().count() + 4;
        let room = width.saturating_sub(used);
        let path = shorten_left(&e.path, room);
        let left = vec![
            Span::styled(head, style),
            Span::raw("  "),
            Span::styled(path, Style::default().fg(Color::DarkGray)),
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
        if hidden > 0 {
            format!("{hint}  (+{hidden} smaller not listed)")
        } else {
            hint.to_string()
        },
        Style::default().fg(Color::DarkGray),
    ))));
    rows
}

/// Keep the tail of `s` (the interesting part of a path) within `max` chars.
fn shorten_left(s: &str, max: usize) -> String {
    let n = s.chars().count();
    if n <= max {
        return s.to_string();
    }
    if max <= 1 {
        return String::new();
    }
    let tail: String = s.chars().skip(n - (max - 1)).collect();
    format!("…{tail}")
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
        let bar = Paragraph::new(Line::from(
            vec![
                Span::styled("‹ ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    cat.name.clone(),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("  {pos}/{}{size}", visible.len()),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(" › (Tab)", Style::default().fg(Color::DarkGray)),
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
        ));
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
    let inner_w = area.width.saturating_sub(4) as usize; // borders + highlight symbol
    let visible = app.view_items();
    let filtering = !app.filter.trim().is_empty();

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
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let dimmed = (item.requires_root && !app.is_root)
                || info.is_some_and(|s| s.bytes == 0 && s.error.is_none());
            // DIM (not a dark colour): stays readable on the highlighted row.
            let name_style = if dimmed {
                Style::default().fg(Color::Gray).add_modifier(Modifier::DIM)
            } else {
                Style::default().fg(Color::White)
            };

            let mut left = vec![
                Span::styled(checkbox_symbol, checkbox_style),
                Span::raw(" "),
                Span::styled(item.name.clone(), name_style),
            ];
            match item.risk {
                cleansys_core::Risk::Safe => {}
                cleansys_core::Risk::Moderate => {
                    left.push(Span::styled(" ~", Style::default().fg(Color::Yellow)))
                }
                cleansys_core::Risk::Caution => left.push(Span::styled(
                    " !",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )),
            }
            if item.requires_root {
                left.push(Span::styled(" (root)", Style::default().fg(Color::Red)));
            }
            if filtering {
                left.push(Span::styled(
                    format!("  · {}", app.categories[ci].name),
                    Style::default().fg(Color::DarkGray),
                ));
            }

            // Right column: run status, else scanned size.
            let right = match &item.status {
                Some(Status::Running) => Span::styled(
                    format!(
                        "{} running",
                        Status::Running.get_animation_frame(app.animation_frame)
                    ),
                    Style::default().fg(Color::Yellow),
                ),
                Some(Status::Success(_)) if item.bytes_cleaned > 0 => Span::styled(
                    format!("✓ freed {}", format_size(item.bytes_cleaned)),
                    Style::default().fg(Color::Green),
                ),
                Some(Status::Success(_)) => {
                    Span::styled("✓ done", Style::default().fg(Color::Green))
                }
                Some(Status::Error(_)) => Span::styled("✗ error", Style::default().fg(Color::Red)),
                Some(Status::Pending) => {
                    Span::styled("queued", Style::default().fg(Color::DarkGray))
                }
                None => match info {
                    Some(s) if s.error.is_some() => {
                        Span::styled("scan failed", Style::default().fg(Color::Red))
                    }
                    Some(s) if s.bytes > 0 => {
                        Span::styled(format_size(s.bytes), size_style(s.bytes))
                    }
                    Some(_) => Span::styled("—", Style::default().fg(Color::DarkGray)),
                    None if app.board.is_scanning() => spinner_spans(app)
                        .into_iter()
                        .next()
                        .unwrap_or_else(|| Span::raw("…")),
                    None => Span::raw(""),
                },
            };
            ListItem::new(row_line(left, right, inner_w))
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

    let mut block = Block::default().title(title).borders(Borders::ALL);
    if app.filter_active {
        block = block.border_style(Style::default().fg(Color::Cyan));
    }
    let block = block.title_bottom({
        let mut spans = scan_line(app).spans;
        spans.push(Span::styled(
            "   ~ moderate  ! caution",
            Style::default().fg(Color::DarkGray),
        ));
        Line::from(spans).right_aligned()
    });

    if visible.is_empty() {
        let msg = if filtering {
            "No cleaner matches your search."
        } else if app.board.is_scanning() {
            "Scanning your system…"
        } else {
            "Nothing to clean here — already tidy ✨  (e: show empty)"
        };
        f.render_widget(
            Paragraph::new(Span::styled(msg, Style::default().fg(Color::DarkGray))).block(block),
            area,
        );
        return;
    }

    // Interleave the expanded cleaner's per-path rows right under it.
    let mut display_sel = app.item_list_state.selected();
    if let Some((ec, ei)) = app.expanded {
        if let Some(p) = visible.iter().position(|&v| v == (ec, ei)) {
            let rows = entry_rows(app, ec, ei, inner_w);
            let n = rows.len();
            items.splice(p + 1..p + 1, rows);
            display_sel = match (app.entry_cursor, display_sel) {
                (Some(k), _) => Some(p + 1 + k.min(n.saturating_sub(1))),
                (None, Some(s)) if s > p => Some(s + n),
                (None, s) => s,
            };
        }
    }
    app.list_view_state.select(display_sel);

    let items_list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::DarkGray),
        )
        .highlight_symbol("> ");

    f.render_stateful_widget(items_list, area, &mut app.list_view_state);
}

/// Description + scan details of the highlighted cleaner.
fn render_item_detail(f: &mut Frame, app: &App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();
    if let Some((ci, ii)) = app.current_item() {
        let item = &app.categories[ci].items[ii];
        let info = app.board.get(ci, ii);
        let risk = match item.risk {
            cleansys_core::Risk::Safe => ("safe", Color::Green),
            cleansys_core::Risk::Moderate => ("moderate", Color::Yellow),
            cleansys_core::Risk::Caution => ("CAUTION", Color::Red),
        };
        let mut head = vec![
            Span::styled(
                item.name.clone(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(format!("[{}]", risk.0), Style::default().fg(risk.1)),
        ];
        if item.requires_root {
            head.push(Span::styled(" [root]", Style::default().fg(Color::Red)));
        }
        lines.push(Line::from(head));
        lines.push(Line::from(item.description.clone()));
        lines.push(Line::from(match info {
            Some(s) if s.error.is_some() => Span::styled(
                format!("Scan failed: {}", s.error.clone().unwrap_or_default()),
                Style::default().fg(Color::Red),
            ),
            Some(s) if s.bytes > 0 => Span::styled(
                format!("Would free {} in {} item(s)", format_size(s.bytes), s.items),
                size_style(s.bytes),
            ),
            Some(_) => Span::styled(
                "Nothing to clean right now",
                Style::default().fg(Color::DarkGray),
            ),
            None => Span::styled("Not measured yet…", Style::default().fg(Color::DarkGray)),
        }));
        if let Some(top) = info.and_then(|s| s.top_path.as_ref()) {
            lines.push(Line::from(Span::styled(
                format!("Biggest: {top}"),
                Style::default().fg(Color::DarkGray),
            )));
        }
    }
    let widget = Paragraph::new(lines)
        .block(Block::default().title("Details").borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    f.render_widget(widget, area);
}

fn render_details(f: &mut Frame, app: &App, area: Rect) {
    let Some((ci, ii)) = app.current_item() else {
        return;
    };
    let item = &app.categories[ci].items[ii];

    let mut text = vec![
        Line::from(vec![Span::styled(
            format!("{} Keyboard Controls", item.name),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![
            Span::raw("Description: "),
            Span::styled(&item.description, Style::default().fg(Color::White)),
        ]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![
            Span::raw("Requires root: "),
            if item.requires_root {
                Span::styled("Yes", Style::default().fg(Color::Red))
            } else {
                Span::styled("No", Style::default().fg(Color::Green))
            },
        ]),
        Line::from(vec![
            Span::raw("Status: "),
            match &item.status {
                Some(Status::Running) => {
                    let spinner = Status::Running.get_animation_frame(app.animation_frame);
                    Span::styled(
                        format!("{} Running...", spinner),
                        Style::default().fg(Color::Yellow),
                    )
                }
                Some(Status::Success(msg)) => {
                    Span::styled(format!("✓ {}", msg), Style::default().fg(Color::Green))
                }
                Some(Status::Error(msg)) => {
                    Span::styled(format!("✗ Error: {}", msg), Style::default().fg(Color::Red))
                }
                Some(Status::Pending) => {
                    Span::styled("• Waiting to start", Style::default().fg(Color::DarkGray))
                }
                None => Span::raw("Not run"),
            },
        ]),
    ];

    if item.bytes_cleaned > 0 {
        text.push(Line::from(vec![
            Span::raw("Space freed: "),
            Span::styled(
                format_size(item.bytes_cleaned),
                Style::default().fg(Color::Green),
            ),
        ]));
    }

    let details = Paragraph::new(text)
        .block(Block::default().title("Details").borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    f.render_widget(details, area);
}

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner_area = block.inner(area);

    if app.is_running || app.show_progress_screen {
        // Progress mode footer - clean and simple
        let footer_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60), // Status info
                Constraint::Percentage(40), // Controls
            ])
            .split(inner_area);

        // Status information
        let status_text = vec![Line::from(vec![
            Span::styled(
                "Status: ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            if app.paused {
                Span::styled(
                    "PAUSED",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
            } else if app.is_running {
                Span::styled(
                    "CLEANING",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )
            } else if app.operation_end_time.is_some() {
                Span::styled(
                    "FINISHED",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(
                    "READY",
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                )
            },
            Span::raw("  •  "),
            Span::styled("Total Freed: ", Style::default().fg(Color::White)),
            Span::styled(
                format_size(app.total_bytes_cleaned),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ])];

        // Controls - different for running vs completed operations
        let controls_text = if app.is_running {
            vec![Line::from(vec![
                Span::styled(
                    "ESC",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(": Cancel  "),
                Span::styled(
                    "↑/↓",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(": Scroll Items  "),
                Span::styled(
                    "q",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::raw(": Quit"),
            ])]
        } else {
            // Operations completed - show different controls
            vec![Line::from(vec![
                Span::styled(
                    "ESC",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(": Return to Menu  "),
                Span::styled(
                    "↑/↓",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(": Scroll Items  "),
                Span::styled(
                    "q",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::raw(": Quit"),
            ])]
        };

        let status_para = Paragraph::new(status_text);
        let controls_para =
            Paragraph::new(controls_text).alignment(ratatui::layout::Alignment::Right);

        f.render_widget(status_para, footer_chunks[0]);
        f.render_widget(controls_para, footer_chunks[1]);
    } else if inner_area.width < 100 {
        // Narrow terminals: selection summary on one line, compact key hints below.
        let n = app
            .categories
            .iter()
            .flat_map(|c| &c.items)
            .filter(|i| i.selected)
            .count();
        let key = |k: &'static str, label: &'static str, color: Color| {
            vec![
                Span::styled(k, Style::default().fg(color).add_modifier(Modifier::BOLD)),
                Span::raw(format!(" {label}  ")),
            ]
        };
        let mut hints: Vec<Span> = Vec::new();
        hints.extend(key("Space", "Sel", Color::Yellow));
        hints.extend(key("Enter", "Run", Color::Green));
        hints.extend(key("Tab", "Cat", Color::Blue));
        hints.extend(key("r", "Rec", Color::Cyan));
        hints.extend(key("S", "Sched", Color::Cyan));
        hints.extend(key("?", "Help", Color::Magenta));
        hints.extend(key("q", "Quit", Color::Red));
        let mut status_spans = vec![
            Span::styled("Selected: ", Style::default().fg(Color::White)),
            Span::styled(
                n.to_string(),
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            ),
        ];
        status_spans.push(Span::raw("  •  "));
        status_spans.push(Span::styled(
            format!(
                "Idle ≥ {} ([ ])",
                cleansys_core::engine::config::min_age_label(app.min_age_days)
            ),
            Style::default().fg(Color::Cyan),
        ));
        status_spans.extend(to_free_spans(app));
        let status = Line::from(status_spans);
        let lines = if inner_area.height >= 2 {
            vec![status, Line::from(hints)]
        } else {
            vec![Line::from(hints)]
        };
        f.render_widget(Paragraph::new(lines), inner_area);
    } else {
        // Main menu footer - organized and clean
        let footer_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(40), // Status info
                Constraint::Percentage(60), // Controls
            ])
            .split(inner_area);

        // Status information
        let mut status_spans = vec![
            Span::styled(
                "User: ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            if app.is_root {
                Span::styled(
                    "root",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(
                    "standard",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )
            },
            Span::raw("  •  "),
            Span::styled("Selected: ", Style::default().fg(Color::White)),
            Span::styled(
                // Live count (the cached counter only refreshes on the progress screen).
                format!(
                    "{}",
                    app.categories
                        .iter()
                        .flat_map(|c| &c.items)
                        .filter(|i| i.selected)
                        .count()
                ),
                Style::default()
                    .fg(Color::Blue)
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
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(": Select  "),
            Span::styled(
                "Enter",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(": Run  "),
            Span::styled(
                "Tab",
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(": Category  "),
            Span::styled(
                "r",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(": Recommended  "),
            Span::styled(
                "S",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(": Schedule  "),
            Span::styled(
                "?",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(": Help  "),
            Span::styled(
                "q",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
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

fn render_help(f: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from(vec![Span::styled(
            "🔍 Cleansys Help",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::styled(
            "📍 Navigation:",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::raw("  ↑/↓: Navigate items")]),
        Line::from(vec![Span::raw("  Tab/Shift+Tab: Switch categories")]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::styled(
            "🔧 Actions:",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::raw("  Space: Toggle selection")]),
        Line::from(vec![Span::raw(
            "  →: Expand a cleaner's paths (↑/↓ move, Space tick, a/n all/none, ← back)",
        )]),
        Line::from(vec![Span::raw(
            "  [ / ]: Idle days for project build output (fewer / more), then rescans",
        )]),
        Line::from(vec![Span::raw(
            "  Enter: Run selected cleaners (asks for confirmation)",
        )]),
        Line::from(vec![Span::raw(
            "  d: Preview selected cleaners (dry-run, deletes nothing)",
        )]),
        Line::from(vec![Span::raw("  a: Select all in current category")]),
        Line::from(vec![Span::raw("  n: Deselect all in current category")]),
        Line::from(vec![Span::raw("  A: Select all across every category")]),
        Line::from(vec![Span::raw("  N: Deselect all across every category")]),
        Line::from(vec![Span::raw(
            "  r: Select the recommended set (safe, user-land cleaners)",
        )]),
        Line::from(vec![Span::raw(
            "  S: Schedule automatic cleaning (daily/weekly/monthly)",
        )]),
        Line::from(vec![Span::raw(
            "  c: Cycle chart type (Bar → Count Pie → Size Pie → Bar)",
        )]),
        Line::from(vec![Span::raw(
            "  /: Filter cleaners across all categories (Esc clears)",
        )]),
        Line::from(vec![Span::raw(
            "  e: Hide/show cleaners with nothing to clean   R: Re-scan sizes",
        )]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::styled(
            "🎛️ Advanced Controls:",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::raw("  m: Toggle compact mode")]),
        Line::from(vec![Span::raw(
            "  v: Cycle view mode (Standard/Compact/Detailed/Performance)",
        )]),
        Line::from(vec![Span::raw("  p: Toggle performance statistics")]),
        Line::from(vec![Span::raw(
            "  s: Toggle auto-scroll log (during operations)",
        )]),
        Line::from(vec![Span::raw("  o: Cycle sort mode")]),
        Line::from(vec![Span::raw("  f: Cycle filter mode")]),
        Line::from(vec![Span::raw("  y: Toggle confirmation prompts")]),
        Line::from(vec![Span::raw("  x: Clear all errors")]),
        Line::from(vec![Span::raw(
            "  j/k: Scroll detailed items list (vi-style)",
        )]),
        Line::from(vec![Span::raw("  /: Search files/paths in detailed view")]),
        Line::from(vec![Span::raw(
            "  ESC: Clear search / Cancel operation / Return to menu",
        )]),
        Line::from(vec![Span::raw("  Backspace: Remove search character")]),
        Line::from(vec![Span::raw("  PgUp/PgDn: Scroll operation log")]),
        Line::from(vec![Span::raw("  Home/End: Jump to first/last item")]),
        Line::from(vec![Span::raw("  Ctrl+Space: Pause/Resume operations")]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::styled(
            "🔍 Search Features:",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::raw(
            "  Search matches file paths, categories, and cleaner names",
        )]),
        Line::from(vec![Span::raw(
            "  Real-time filtering with highlighted results",
        )]),
        Line::from(vec![Span::raw("  Category distribution shown at bottom")]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::styled(
            "📊 Chart Types (press 'c' to cycle):",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::raw(
            "  Bar Chart: Traditional vertical bars for comparison",
        )]),
        Line::from(vec![Span::raw(
            "  Pie Count: Circular chart showing item distribution by count",
        )]),
        Line::from(vec![Span::raw(
            "  Pie Size: Circular chart showing space usage by category",
        )]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::styled(
            "🔒 System Operations:",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::raw(
            "  System cleaners require sudo/root privileges",
        )]),
        Line::from(vec![Span::raw(
            "  Run 'sudo cleansys' or provide password when prompted",
        )]),
        Line::from(vec![Span::raw(
            "  Items marked (sudo) will request elevated privileges",
        )]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::styled(
            "🔄 Other:",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::raw("  ?: Show/hide help")]),
        Line::from(vec![Span::raw("  q: Exit application")]),
    ];

    let help = Paragraph::new(help_text)
        .block(Block::default().title("📚 Help").borders(Borders::ALL))
        .wrap(Wrap { trim: true });

    f.render_widget(help, area);
}

/// Compute a centered popup `Rect` covering roughly `width_pct`/`height_pct`
/// of `area`, clamped to a sensible minimum/maximum size.
fn centered_popup(area: Rect, width_pct: u16, height_pct: u16) -> Rect {
    let width = (area.width * width_pct / 100).clamp(30, area.width.saturating_sub(2).max(30));
    let height = (area.height * height_pct / 100).clamp(10, area.height.saturating_sub(2).max(10));
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
            "⚠️  Confirm Cleaning",
            Style::default()
                .fg(Color::Yellow)
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
            Style::default().fg(Color::DarkGray),
        )]));
    }

    lines.push(Line::from(vec![Span::raw("")]));
    lines.push(Line::from(vec![Span::styled(
        "Enter/y: Run now    Esc/n: Cancel",
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    )]));

    let popup_widget = Paragraph::new(lines)
        .block(
            Block::default()
                .title("Confirm")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
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
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(match &app.schedule_installed {
            Some(b) => Span::styled(
                format!("Status: ACTIVE via {b} — {}", d.describe()),
                Style::default().fg(Color::Green),
            ),
            None => Span::styled(
                "Status: not scheduled",
                Style::default().fg(Color::DarkGray),
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
            Style::default().fg(Color::DarkGray),
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
                .fg(Color::Black)
                .bg(Color::Cyan)
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
        Style::default().fg(Color::DarkGray),
    )));
    lines.push(Line::from(Span::styled(
        "and never include caution-risk cleaners unless you ticked them (Selected).",
        Style::default().fg(Color::DarkGray),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        app.schedule_message.clone(),
        Style::default().fg(Color::Yellow),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "↑/↓ field   ←/→ change   Enter enable/update   d remove   Esc close",
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    )));

    let widget = Paragraph::new(lines)
        .block(
            Block::default()
                .title("Schedule")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
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
            "🔍 Preview (dry-run)",
            Style::default()
                .fg(Color::Cyan)
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
            Style::default().fg(Color::DarkGray),
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
                Style::default().fg(Color::DarkGray),
            )]));
        }
        lines.push(Line::from(vec![Span::raw("")]));
    }

    lines.push(Line::from(vec![Span::styled(
        "Enter/Esc/q: Close",
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    )]));

    let popup_widget = Paragraph::new(lines)
        .block(
            Block::default()
                .title("Preview")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
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
            "⚠️  Administrator privileges required",
            Style::default()
                .fg(Color::Yellow)
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
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )]),
    ];

    let popup_widget = Paragraph::new(lines)
        .block(
            Block::default()
                .title("Administrator required")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(Clear, popup);
    f.render_widget(popup_widget, popup);
}
