//! Thin adapter from CleanSys's `(name, count, size)` category-distribution
//! tuples to the [`tui_piechart`] crate's own `PieChart`/`PieSlice` widgets.
//!
//! There is no local pie-chart drawing logic here — rendering is handled
//! entirely by `tui_piechart::PieChart`, which already supports everything
//! this crate needs (block/title, legend, percentages, colours) via its own
//! builder API. This module only maps our domain data into `PieSlice`s and
//! applies CleanSys's default styling.

use ratatui::{
    layout::Alignment,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Padding},
};
use tui_piechart::{PieChart, PieSlice};

/// Colour palette cycled across pie slices (by index).
const SLICE_COLORS: [Color; 10] = [
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

/// Build a ready-to-render [`tui_piechart::PieChart`] from a category
/// distribution, with CleanSys's default title/border styling, percentages,
/// and legend enabled.
///
/// The returned widget borrows `distribution`'s and `title`'s string data, so
/// both must outlive the `frame.render_widget(..., area)` call.
pub fn create_pie_chart_from_distribution<'a>(
    distribution: &'a [(String, usize, u64)], // (name, count, size)
    title: &'a str,
    use_size: bool, // true for size-based, false for count-based
) -> PieChart<'a> {
    let slices: Vec<PieSlice<'a>> = distribution
        .iter()
        .enumerate()
        .map(|(i, (name, count, size))| {
            let value = if use_size {
                *size as f64
            } else {
                *count as f64
            };
            PieSlice::new(name.as_str(), value, SLICE_COLORS[i % SLICE_COLORS.len()])
        })
        .collect();

    let block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .title_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .padding(Padding::new(1, 1, 0, 0));

    PieChart::new(slices)
        .block(block)
        .show_percentages(true)
        .show_legend(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_distribution() -> Vec<(String, usize, u64)> {
        vec![
            ("Browser Caches".to_string(), 3, 1_048_576),
            ("Trash".to_string(), 1, 2_097_152),
        ]
    }

    #[test]
    fn create_pie_chart_from_distribution_does_not_panic_count_based() {
        let dist = sample_distribution();
        let _chart = create_pie_chart_from_distribution(&dist, "Count", false);
    }

    #[test]
    fn create_pie_chart_from_distribution_does_not_panic_size_based() {
        let dist = sample_distribution();
        let _chart = create_pie_chart_from_distribution(&dist, "Size", true);
    }

    #[test]
    fn create_pie_chart_from_distribution_handles_empty_distribution() {
        let dist: Vec<(String, usize, u64)> = Vec::new();
        let _chart = create_pie_chart_from_distribution(&dist, "Empty", false);
    }

    #[test]
    fn create_pie_chart_from_distribution_cycles_colors_beyond_palette_size() {
        // 12 entries > SLICE_COLORS.len() (10) exercises the modulo wraparound.
        let dist: Vec<(String, usize, u64)> =
            (0..12).map(|i| (format!("Item {i}"), 1, 1024)).collect();
        let _chart = create_pie_chart_from_distribution(&dist, "Wraparound", false);
    }
}
