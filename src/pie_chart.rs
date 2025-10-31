use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::*,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

pub struct PieChartData {
    pub name: String,
    pub value: f64,
    pub color: Color,
}

pub struct PieChart {
    pub title: String,
    pub data: Vec<PieChartData>,
    pub show_percentages: bool,
    pub show_legend: bool,
}

impl Default for PieChart {
    fn default() -> Self {
        Self {
            title: "Distribution".to_string(),
            data: Vec::new(),
            show_percentages: true,
            show_legend: true,
        }
    }
}

impl PieChart {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            ..Default::default()
        }
    }

    pub fn data(mut self, data: Vec<PieChartData>) -> Self {
        self.data = data;
        self
    }

    pub fn show_percentages(mut self, show: bool) -> Self {
        self.show_percentages = show;
        self
    }

    pub fn show_legend(mut self, show: bool) -> Self {
        self.show_legend = show;
        self
    }

    pub fn render<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        if area.width < 20 || area.height < 8 {
            // Too small to render anything meaningful
            self.render_compact_legend(f, area);
            return;
        }

        let total: f64 = self.data.iter().map(|d| d.value).sum();
        if total <= 0.0 {
            self.render_empty_state(f, area);
            return;
        }

        // Calculate layout based on available space
        let (chart_area, legend_area) = if self.show_legend && area.width >= 60 {
            // Side-by-side layout for wide areas
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
                .split(area);
            (chunks[0], Some(chunks[1]))
        } else if self.show_legend && area.height >= 16 {
            // Stacked layout for tall areas
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
                .split(area);
            (chunks[0], Some(chunks[1]))
        } else {
            // Chart only
            (area, None)
        };

        // Render the pie chart
        self.render_pie_chart(f, chart_area, total);

        // Render legend if space allows
        if let Some(legend_area) = legend_area {
            self.render_legend(f, legend_area, total);
        }
    }

    fn render_pie_chart<B: Backend>(&self, f: &mut Frame<B>, area: Rect, total: f64) {
        let block = Block::default()
            .title(self.title.clone())
            .title_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner_area = block.inner(area);
        f.render_widget(block, area);

        if inner_area.width < 10 || inner_area.height < 5 {
            return;
        }

        // Calculate pie chart dimensions
        let center_x = inner_area.width / 2;
        let center_y = inner_area.height / 2;
        let radius = std::cmp::min(center_x, center_y).saturating_sub(1);

        if radius < 2 {
            return;
        }

        // Generate ASCII pie chart
        let chart_lines = self.generate_ascii_pie(center_x, center_y, radius, total);

        let paragraph = Paragraph::new(chart_lines).alignment(Alignment::Center);

        f.render_widget(paragraph, inner_area);
    }

    fn generate_ascii_pie(
        &self,
        _center_x: u16,
        _center_y: u16,
        radius: u16,
        total: f64,
    ) -> Vec<Line<'_>> {
        let mut lines = Vec::new();
        let chart_height = std::cmp::min(radius as usize * 2, 12);
        let chart_width = std::cmp::min(radius as usize * 3, 24); // Wider for better visibility

        // Create a simple but effective circular pie chart
        let pie_chars = ['█', '▓', '▒', '░', '▄', '▀', '◆', '●'];
        let colors = [
            Color::Red,
            Color::Green,
            Color::Blue,
            Color::Yellow,
            Color::Magenta,
            Color::Cyan,
            Color::White,
            Color::LightRed,
        ];

        // Calculate cumulative percentages for pie slices
        let mut cumulative_percent = 0.0;
        let mut slice_info = Vec::new();

        for (i, data) in self.data.iter().enumerate() {
            let percentage = (data.value / total) * 100.0;
            slice_info.push((
                cumulative_percent,
                cumulative_percent + percentage,
                i,
                pie_chars[i % pie_chars.len()],
            ));
            cumulative_percent += percentage;
        }

        // Generate the circular pie chart row by row
        for row in 0..chart_height {
            let y = row as f64 - (chart_height as f64 / 2.0);
            let mut line_spans = Vec::new();

            for col in 0..chart_width {
                let x = col as f64 - (chart_width as f64 / 2.0);
                let distance = (x * x + y * y * 4.0).sqrt(); // Adjust for character aspect ratio

                if distance <= radius as f64 {
                    // Point is inside the circle, determine which slice
                    let angle = (y * 2.0).atan2(x); // Adjust y for aspect ratio
                    let angle_degrees = (angle.to_degrees() + 360.0) % 360.0;
                    let angle_percent = angle_degrees / 3.6; // Convert to percentage (0-100)

                    // Find which slice this point belongs to
                    let mut slice_char = '●';
                    let mut slice_color = Color::Gray;

                    for &(start_percent, end_percent, slice_index, char) in &slice_info {
                        if angle_percent >= start_percent && angle_percent < end_percent {
                            slice_char = char;
                            slice_color = colors[slice_index % colors.len()];
                            break;
                        }
                    }

                    line_spans.push(Span::styled(
                        slice_char.to_string(),
                        Style::default().fg(slice_color),
                    ));
                } else {
                    line_spans.push(Span::raw(" "));
                }
            }

            lines.push(Line::from(line_spans));
        }

        // If no slices, show a simple filled circle
        if slice_info.is_empty() {
            lines.clear();
            let simple_circle = [
                "      ████████      ",
                "    ████████████    ",
                "  ████████████████  ",
                " ████████████████████",
                "████████████████████",
                " ████████████████████",
                "  ████████████████  ",
                "    ████████████    ",
                "      ████████      ",
            ];

            for circle_line in simple_circle.iter().take(chart_height) {
                lines.push(Line::from(vec![Span::styled(
                    *circle_line,
                    Style::default().fg(Color::Cyan),
                )]));
            }
        }

        lines
    }

    fn get_char_color(&self, ch: char) -> Color {
        match ch {
            '█' => Color::Red,
            '▓' => Color::Green,
            '▒' => Color::Blue,
            '░' => Color::Yellow,
            '▄' => Color::Magenta,
            '▀' => Color::Cyan,
            '◆' => Color::White,
            '●' => Color::LightRed,
            _ => Color::Gray,
        }
    }

    fn render_legend<B: Backend>(&self, f: &mut Frame<B>, area: Rect, total: f64) {
        let block = Block::default()
            .title("Legend")
            .title_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));

        let inner_area = block.inner(area);
        f.render_widget(block, area);

        let mut legend_lines = Vec::new();
        let pie_chars = ['█', '▓', '▒', '░', '▄', '▀', '◆', '●'];

        for (i, data) in self.data.iter().enumerate() {
            let percentage = if total > 0.0 {
                (data.value / total) * 100.0
            } else {
                0.0
            };
            let char_index = i % pie_chars.len();
            let symbol = pie_chars[char_index];

            let line = if self.show_percentages {
                Line::from(vec![
                    Span::styled(
                        format!("{} ", symbol),
                        Style::default().fg(self.get_char_color(symbol)),
                    ),
                    Span::styled(&data.name, Style::default().fg(Color::White)),
                    Span::styled(
                        format!(" ({:.1}%)", percentage),
                        Style::default().fg(Color::Gray),
                    ),
                ])
            } else {
                Line::from(vec![
                    Span::styled(
                        format!("{} ", symbol),
                        Style::default().fg(self.get_char_color(symbol)),
                    ),
                    Span::styled(&data.name, Style::default().fg(Color::White)),
                ])
            };

            legend_lines.push(line);
        }

        let legend_paragraph = Paragraph::new(legend_lines);
        f.render_widget(legend_paragraph, inner_area);
    }

    fn render_compact_legend<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let total: f64 = self.data.iter().map(|d| d.value).sum();
        if total <= 0.0 {
            return;
        }

        let mut lines = Vec::new();
        for data in self.data.iter().take(3) {
            // Show only top 3 in compact mode
            let percentage = (data.value / total) * 100.0;
            let line = Line::from(vec![
                Span::styled(format!("• {}", &data.name), Style::default().fg(data.color)),
                Span::styled(
                    format!(" {:.0}%", percentage),
                    Style::default().fg(Color::Gray),
                ),
            ]);
            lines.push(line);
        }

        let paragraph = Paragraph::new(lines).block(
            Block::default()
                .title(self.title.clone())
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );

        f.render_widget(paragraph, area);
    }

    fn render_empty_state<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let empty_msg = Paragraph::new("No data to display")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray))
            .block(
                Block::default()
                    .title(self.title.clone())
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            );

        f.render_widget(empty_msg, area);
    }
}

// Helper function to create pie chart data from category distribution
pub fn create_pie_chart_from_distribution(
    distribution: &[(String, usize, u64)], // (name, count, size)
    title: &str,
    use_size: bool, // true for size-based, false for count-based
) -> PieChart {
    let colors = [
        Color::Red,
        Color::Green,
        Color::Blue,
        Color::Yellow,
        Color::Magenta,
        Color::Cyan,
        Color::White,
        Color::LightRed,
        Color::LightGreen,
        Color::LightBlue,
    ];

    let data: Vec<PieChartData> = distribution
        .iter()
        .enumerate()
        .map(|(i, (name, count, size))| PieChartData {
            name: name.clone(),
            value: if use_size {
                *size as f64
            } else {
                *count as f64
            },
            color: colors[i % colors.len()],
        })
        .collect();

    PieChart::new(title).data(data)
}
