use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    widgets::{block::Padding, Block, Borders},
    Frame,
};
use tui_piechart::{PieChart as TuiPieChart, PieSlice};

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

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if area.width < 20 || area.height < 8 {
            // Too small to render anything meaningful
            return;
        }

        let total: f64 = self.data.iter().map(|d| d.value).sum();
        if total <= 0.0 {
            return;
        }

        // Convert our data format to tui-piechart PieSlice format
        let slices: Vec<PieSlice> = self
            .data
            .iter()
            .map(|d| PieSlice::new(&d.name, d.value, d.color))
            .collect();

        // Create block with title, borders, and padding
        let block = Block::default()
            .title(self.title.clone())
            .title_alignment(Alignment::Center)
            .title_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .padding(Padding::new(1, 1, 0, 0));

        // Create the tui-piechart widget with legend and percentages always enabled
        let mut piechart = TuiPieChart::new(slices).block(block).show_percentages(true); // Always show percentages

        // Configure legend based on settings
        if self.show_legend {
            piechart = piechart.show_legend(true);
        }

        // Render the pie chart
        frame.render_widget(piechart, area);
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
