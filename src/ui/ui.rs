use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Chart, Dataset, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::ui::app::{App, ChartType, CleanedItemType, Status};
use crate::ui::tui::components::create_pie_chart_from_distribution;
use crate::utils::format_size;

pub fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
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
        .split(f.size());

    render_title(f, app, chunks[0]);

    if app.show_help {
        render_help(f, chunks[1]);
    } else if app.is_running || app.show_progress_screen {
        render_progress_screen(f, app, chunks[1]);
    } else {
        render_main_content(f, app, chunks[1]);
    }

    render_footer(f, app, chunks[2]);
}

fn render_title<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
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
                Span::raw(" - Modern System Cleaner for Linux"),
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
}

fn render_main_content<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {

    // Adjust layout based on terminal width
    let (categories_percent, content_percent) = if app.terminal_width < 80 {
        // Narrow terminals: give more space to content
        (25, 75)
    } else if app.terminal_width < 120 {
        // Medium terminals: balanced layout
        (30, 70)
    } else {
        // Wide terminals: can afford more space for categories
        (35, 65)
    };

    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(categories_percent), // Categories
            Constraint::Percentage(content_percent),    // Cleaners/Details
        ])
        .split(area);

    render_categories(f, app, horizontal_chunks[0]);

    if app.detailed_view {
        render_details(f, app, horizontal_chunks[1]);
    } else {
        render_cleaners(f, app, horizontal_chunks[1]);
    }
}

fn render_progress_screen<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {
    // Render both progress and details in a unified view
    render_unified_progress_view(f, app, area);
}

fn render_unified_progress_view<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {
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

fn render_combined_progress_overview<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
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

fn render_progress_stats_and_chart<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let elapsed_time = app.get_elapsed_time();
    let total_ops = app.operation_count;
    let completed_ops = total_ops.saturating_sub(app.errors_count);
    let progress_percent = if total_ops > 0 {
        (completed_ops * 100) / total_ops
    } else {
        0
    };

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
            Span::raw("█".repeat((progress_percent as usize * 35) / 100)),
            Span::styled(
                "░".repeat(35 - (progress_percent as usize * 35) / 100),
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

fn render_ultra_compact_view<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let elapsed_time = app.get_elapsed_time();
    let total_ops = app.operation_count;
    let completed_ops = total_ops.saturating_sub(app.errors_count);
    let progress_percent = if total_ops > 0 {
        (completed_ops * 100) / total_ops
    } else {
        0
    };

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
            Span::raw(
                "█".repeat(
                    ((progress_percent as usize * (area.width.saturating_sub(10) as usize)) / 100)
                        .min(30),
                ),
            ),
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

fn render_vertical_bar_chart<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    // Get real data from cleaned items
    let category_distribution = app.get_category_distribution();

    // Create chart data from real cleaning results
    let (chart_data, max_value, categories) = if category_distribution.is_empty() {
        // Default data when no items have been cleaned yet
        (
            vec![(0.0, 0.0), (1.0, 0.0), (2.0, 0.0)],
            1.0,
            vec!["Trash", "Packages", "Caches"],
        )
    } else {
        // Use real data, limit to top 6 categories to fit in chart
        let limited_data: Vec<_> = category_distribution.iter().take(6).collect();
        let max_count = limited_data
            .iter()
            .map(|(_, count, _)| *count)
            .max()
            .unwrap_or(1) as f64;

        let data: Vec<(f64, f64)> = limited_data
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
                } else {
                    if name.len() > 12 {
                        &name[..12]
                    } else {
                        name
                    }
                }
            })
            .collect();

        (data, max_count, category_names)
    };

    // Create dataset for bar chart
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
    let x_labels = if categories.len() <= 3 {
        vec![
            Span::raw(categories.get(0).unwrap_or(&"").to_string()),
            Span::raw(categories.get(1).unwrap_or(&"").to_string()),
            Span::raw(categories.get(2).unwrap_or(&"").to_string()),
        ]
    } else {
        vec![
            Span::raw(categories.first().unwrap_or(&"").to_string()),
            Span::raw(
                categories
                    .get(categories.len() / 2)
                    .unwrap_or(&"")
                    .to_string(),
            ),
            Span::raw(categories.last().unwrap_or(&"").to_string()),
        ]
    };

    // Create y-axis labels
    let y_max = (max_value * 1.1).max(1.0); // Add 10% padding, minimum 1
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
                .bounds([0.0, (categories.len().max(3) - 1) as f64])
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

fn render_operations_summary<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    // Split into user and system operations columns
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(48), // User operations
            Constraint::Percentage(4),  // Spacing
            Constraint::Percentage(48), // System operations
        ])
        .split(area);

    // User operations
    let user_operations = vec![
        ListItem::new(Line::from(vec![Span::styled(
            "👤 USER OPERATIONS",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )])),
        ListItem::new(Line::from(vec![])),
        ListItem::new(Line::from(vec![
            Span::styled("📦 ", Style::default().fg(Color::Green)),
            Span::styled("Package Caches", Style::default().fg(Color::White)),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("🗑️ ", Style::default().fg(Color::Green)),
            Span::styled("Trash & Temp Files", Style::default().fg(Color::White)),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("🌐 ", Style::default().fg(Color::Green)),
            Span::styled("Browser Caches", Style::default().fg(Color::White)),
        ])),
    ];

    // System operations
    let system_operations = vec![
        ListItem::new(Line::from(vec![Span::styled(
            "🔒 SYSTEM OPERATIONS",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )])),
        ListItem::new(Line::from(vec![])),
        ListItem::new(Line::from(vec![
            Span::styled(
                "📦 ",
                if app.is_root {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::Yellow)
                },
            ),
            Span::styled("Package Caches", Style::default().fg(Color::White)),
            if !app.is_root {
                Span::styled(" (sudo)", Style::default().fg(Color::Yellow))
            } else {
                Span::raw("")
            },
        ])),
        ListItem::new(Line::from(vec![
            Span::styled(
                "📝 ",
                if app.is_root {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::Yellow)
                },
            ),
            Span::styled("System Logs", Style::default().fg(Color::White)),
            if !app.is_root {
                Span::styled(" (sudo)", Style::default().fg(Color::Yellow))
            } else {
                Span::raw("")
            },
        ])),
        ListItem::new(Line::from(vec![
            Span::styled(
                "🗄️ ",
                if app.is_root {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::Yellow)
                },
            ),
            Span::styled("System Temp Files", Style::default().fg(Color::White)),
            if !app.is_root {
                Span::styled(" (sudo)", Style::default().fg(Color::Yellow))
            } else {
                Span::raw("")
            },
        ])),
    ];

    let user_list = List::new(user_operations);
    let system_list = List::new(system_operations);

    f.render_widget(user_list, columns[0]);
    f.render_widget(system_list, columns[2]);
}

fn render_pie_chart_distribution<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let category_distribution = app.get_category_distribution();

    // Use real data if available, otherwise show sample data for demonstration
    let data_to_use = if category_distribution.is_empty() {
        vec![
            ("Browser Caches".to_string(), 3, 314572800),
            ("Package Caches".to_string(), 5, 82051072),
            ("Trash Files".to_string(), 2, 5242880),
            ("Temp Files".to_string(), 2, 84934656),
            ("System Logs".to_string(), 1, 10485760),
            ("App Caches".to_string(), 2, 786432),
        ]
    } else {
        category_distribution
    };

    // Create pie chart from distribution data
    let pie_chart = create_pie_chart_from_distribution(
        &data_to_use,
        "Items Distribution (Count)",
        false, // Use count-based distribution
    );

    let responsive_chart = pie_chart
        .show_percentages(area.width >= 40)
        .show_legend(area.width >= 50 || area.height >= 16);

    responsive_chart.render(f, area);
}

fn render_pie_chart_size_distribution<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let category_distribution = app.get_category_distribution();

    // Use real data if available, otherwise show sample data for demonstration
    let data_to_use = if category_distribution.is_empty() {
        vec![
            ("Browser Caches".to_string(), 3, 314572800),
            ("Package Caches".to_string(), 5, 82051072),
            ("Trash Files".to_string(), 2, 5242880),
            ("Temp Files".to_string(), 2, 84934656),
            ("System Logs".to_string(), 1, 10485760),
            ("App Caches".to_string(), 2, 786432),
        ]
    } else {
        category_distribution
    };

    // Create pie chart from size-based distribution
    let pie_chart = create_pie_chart_from_distribution(
        &data_to_use,
        "Size Distribution (Bytes)",
        true, // Use size-based distribution
    );

    let responsive_chart = pie_chart
        .show_percentages(area.width >= 40)
        .show_legend(area.width >= 50 || area.height >= 16);

    responsive_chart.render(f, area);
}

fn render_removed_items_window<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {
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

        if filtered_items.is_empty() {
            // Add sample removed items for demonstration
            let sample_items = vec![
            ("📄", "/home/user/.cache/pip/wheels/abc123.whl", "15.0 MB", "Package Manager Caches", "pip cache"),
            ("📁", "/home/user/.cache/mozilla/firefox/profiles/", "100.0 MB", "Browser Caches", "firefox cache"),
            ("📄", "/home/user/.local/share/Trash/files/document.pdf", "20.0 MB", "Trash", "trash"),
            ("📄", "/home/user/.cache/google-chrome/Default/Cache/f_000001", "5.2 MB", "Browser Caches", "chrome cache"),
            ("📁", "/home/user/.cache/npm/_cacache/content-v2/", "25.6 MB", "Package Manager Caches", "npm cache"),
            ("📄", "/home/user/.cargo/registry/cache/github.com-1ecc6299db9ec823/serde-1.0.136.crate", "50.0 MB", "Package Manager Caches", "cargo cache"),
            ("📄", "/tmp/temp_file_12345.tmp", "1.0 MB", "Temporary Files", "temp files"),
            ("📄", "/home/user/.cache/thumbnails/large/abc123.png", "256 KB", "Thumbnail Caches", "thumbnails"),
            ("📁", "/home/user/.cache/JetBrains/IntelliJIdea2023.1/", "45.8 MB", "Application Caches", "application cache"),
            ("📄", "/home/user/.local/share/recently-used.xbel.bak", "32 KB", "Application Caches", "application cache"),
            ("📄", "/home/user/.cache/fontconfig/CACHEDIR.TAG", "43 bytes", "Application Caches", "font cache"),
            ("📁", "/home/user/.cache/yarn/v6/npm-lodash-4.17.21/", "1.5 MB", "Package Manager Caches", "yarn cache"),
            ("📄", "/var/tmp/portage/temp_file", "2.1 MB", "Temporary Files", "portage temp"),
            ("📄", "/home/user/.local/share/Trash/files/screenshot.png", "3.1 MB", "Trash", "trash"),
            ("📁", "/home/user/.cache/gstreamer-1.0/", "512 KB", "Application Caches", "gstreamer cache"),
        ];

            for (index, (icon, path, size, category, cleaner)) in sample_items.iter().enumerate() {
                // File path and size on one line
                display_items.push(ListItem::new(Line::from(vec![
                    Span::styled(format!("{} ", icon), Style::default().fg(Color::Yellow)),
                    Span::styled(path.to_string(), Style::default().fg(Color::White)),
                    Span::raw(" "),
                    Span::styled(
                        format!("({})", size),
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                ])));

                // Category and cleaner info on next line (indented)
                display_items.push(ListItem::new(Line::from(vec![
                    Span::raw("   "),
                    Span::styled("📂 ", Style::default().fg(Color::Blue)),
                    Span::styled(category.to_string(), Style::default().fg(Color::Blue)),
                    Span::raw(" • "),
                    Span::styled("🔧 ", Style::default().fg(Color::Cyan)),
                    Span::styled(cleaner.to_string(), Style::default().fg(Color::Cyan)),
                ])));

                // Add spacing between entries
                if index < sample_items.len() - 1 {
                    display_items.push(ListItem::new(Line::from(vec![])));
                }
            }
        } else {
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



fn render_categories<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    // Add icons to category names
    let categories: Vec<ListItem> = app
        .categories
        .iter()
        .enumerate()
        .map(|(i, category)| {
            let content = Line::from(format!("{} ({})", category.name, category.description));
            let style = if i == app.category_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(content).style(style)
        })
        .collect();

    let categories_list = List::new(categories)
        .block(
            Block::default()
                .title("📂 Categories")
                .borders(Borders::ALL),
        )
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Yellow),
        );

    f.render_widget(categories_list, area);
}

fn render_cleaners<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {
    let current_category = &app.categories[app.category_index];

    let items: Vec<ListItem> = current_category
        .items
        .iter()
        .map(|item| {
            let mut parts = vec![];

            // Checkbox
            if item.selected {
                parts.push(Span::styled("[X] ", Style::default().fg(Color::Green)));
            } else {
                parts.push(Span::raw("[ ] "));
            }

            // Name
            let name_style = if item.requires_root && !app.is_root {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default().fg(Color::White)
            };
            parts.push(Span::styled(&item.name, name_style));

            // Root indicator
            if item.requires_root {
                parts.push(Span::styled(" (root)", Style::default().fg(Color::Red)));
            }

            // Status indicator
            if let Some(status) = &item.status {
                match status {
                    Status::Running => {
                        parts.push(Span::styled(
                            " [Running]",
                            Style::default().fg(Color::Yellow),
                        ));
                    }
                    Status::Success(msg) => {
                        parts.push(Span::styled(
                            format!(" [{}]", msg),
                            Style::default().fg(Color::Green),
                        ));
                    }
                    Status::Error(msg) => {
                        parts.push(Span::styled(
                            format!(" [Error: {}]", msg),
                            Style::default().fg(Color::Red),
                        ));
                    }
                    Status::Pending => {
                        parts.push(Span::styled(
                            " [Pending]",
                            Style::default().fg(Color::DarkGray),
                        ));
                    }
                }
            }

            // If item has cleaned bytes, show it
            if item.bytes_cleaned > 0 {
                parts.push(Span::styled(
                    format!(" (Freed: {})", format_size(item.bytes_cleaned)),
                    Style::default().fg(Color::Green),
                ));
            }

            ListItem::new(Line::from(parts))
        })
        .collect();

    let items_list = List::new(items)
        .block(
            Block::default()
                .title(format!("{} Items", current_category.name))
                .borders(Borders::ALL),
        )
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::DarkGray),
        )
        .highlight_symbol("> ");

    f.render_stateful_widget(items_list, area, &mut app.item_list_state);
}

fn render_details<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let current_category = &app.categories[app.category_index];

    if let Some(selected) = app.item_list_state.selected() {
        if selected < current_category.items.len() {
            let item = &current_category.items[selected];

            let mut text = vec![
                Line::from(vec![Span::styled(
                    format!("{} Keyboard Controls", &item.name),
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
                        Some(Status::Error(msg)) => Span::styled(
                            format!("✗ Error: {}", msg),
                            Style::default().fg(Color::Red),
                        ),
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
                        format!("{:.2} GB", item.bytes_cleaned as f64 / 1_073_741_824.0),
                        Style::default().fg(Color::Green),
                    ),
                ]));
            }

            let details = Paragraph::new(text)
                .block(Block::default().title("Details").borders(Borders::ALL))
                .wrap(Wrap { trim: true });

            f.render_widget(details, area);
        }
    }
}

fn render_footer<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
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
        let status_text = vec![Line::from(vec![
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
                format!("{}", app.selected_cleaners_count),
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            ),
        ])];

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

fn render_help<B: Backend>(f: &mut Frame<B>, area: Rect) {
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
        Line::from(vec![Span::raw("  Enter: Run selected cleaners")]),
        Line::from(vec![Span::raw("  a: Select all in current category")]),
        Line::from(vec![Span::raw("  n: Deselect all in current category")]),
        Line::from(vec![Span::raw("  l: Toggle detailed cleaned items list")]),
        Line::from(vec![Span::raw(
            "  c: Cycle chart type (Count Pie → Size Pie → Bar → Count Pie)",
        )]),
        Line::from(vec![Span::raw("  /: Search in detailed view")]),
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
        Line::from(vec![Span::raw("  ESC: Clear search / Cancel operation / Return to menu")]),
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
            "  Pie Count: Circular chart showing item distribution by count",
        )]),
        Line::from(vec![Span::raw(
            "  Pie Size: Circular chart showing space usage by category",
        )]),
        Line::from(vec![Span::raw(
            "  Bar Chart: Traditional vertical bars for comparison",
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
