//! Shared look of the progress indicators in every front-end: a spinner glyph that turns,
//! and a bar with a highlight that sweeps along its filled part.
//!
//! The TUI and GUI drive these with a tick counter; the web UI does the same with CSS
//! (`.progress` / `.spin`). Keeping the maths here keeps them in step (and testable).

/// Braille spinner frames (the "dots" spinner).
pub const SPINNER_FRAMES: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// The spinner glyph for animation tick `tick`.
pub fn spinner(tick: u32) -> &'static str {
    SPINNER_FRAMES[tick as usize % SPINNER_FRAMES.len()]
}

/// Where the sweeping highlight is, `0.0..1.0` along the filled part of a bar, for a sweep
/// that takes `period` ticks.
pub fn sweep(tick: u32, period: u32) -> f32 {
    let period = period.max(1);
    (tick % period) as f32 / period as f32
}

/// `done` of `total` as a fraction `0.0..=1.0` (an empty total is 0).
pub fn fraction(done: usize, total: usize) -> f32 {
    if total == 0 {
        0.0
    } else {
        (done as f32 / total as f32).clamp(0.0, 1.0)
    }
}

/// `done` of `total` as a whole percent.
pub fn percent(done: usize, total: usize) -> u32 {
    (fraction(done, total) * 100.0).round() as u32
}

/// Cells of a `width`-wide text bar: how many are filled, and the `(start, len)` of the
/// highlight inside the filled part at `tick`. The highlight is `highlight` cells wide
/// (shrunk to fit) and never leaves the filled cells.
pub fn bar_cells(
    width: usize,
    frac: f32,
    tick: u32,
    period: u32,
    highlight: usize,
) -> (usize, (usize, usize)) {
    let filled = ((width as f32) * frac.clamp(0.0, 1.0)).round() as usize;
    let filled = filled.min(width);
    let len = highlight.min(filled);
    if len == 0 {
        return (filled, (0, 0));
    }
    let travel = filled - len;
    let start = (sweep(tick, period) * (travel as f32 + 1.0)) as usize;
    (filled, (start.min(travel), len))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spinner_cycles_through_every_frame() {
        let seen: std::collections::HashSet<_> = (0..10).map(spinner).collect();
        assert_eq!(seen.len(), 10);
        assert_eq!(spinner(0), spinner(10));
        assert_eq!(spinner(u32::MAX), SPINNER_FRAMES[(u32::MAX as usize) % 10]);
    }

    #[test]
    fn sweep_is_a_sawtooth_in_range() {
        assert_eq!(sweep(0, 20), 0.0);
        assert!((sweep(10, 20) - 0.5).abs() < 1e-6);
        assert_eq!(sweep(20, 20), 0.0, "wraps");
        assert!((0..100).all(|t| (0.0..1.0).contains(&sweep(t, 7))));
        assert_eq!(sweep(5, 0), 0.0, "a zero period is treated as 1");
    }

    #[test]
    fn fraction_and_percent_handle_edges() {
        assert_eq!(fraction(0, 0), 0.0);
        assert_eq!(fraction(5, 0), 0.0);
        assert_eq!(fraction(3, 4), 0.75);
        assert_eq!(fraction(9, 4), 1.0, "clamped");
        assert_eq!(percent(1, 3), 33);
        assert_eq!(percent(2, 3), 67);
    }

    #[test]
    fn highlight_stays_inside_the_filled_cells_and_moves() {
        let (filled, (_, len)) = bar_cells(40, 0.5, 0, 24, 5);
        assert_eq!((filled, len), (20, 5));
        let mut starts = std::collections::HashSet::new();
        for tick in 0..48 {
            let (filled, (start, len)) = bar_cells(40, 0.5, tick, 24, 5);
            assert!(
                start + len <= filled,
                "tick {tick}: {start}+{len} > {filled}"
            );
            starts.insert(start);
        }
        assert!(starts.len() > 5, "the highlight sweeps across: {starts:?}");
        assert_eq!(
            bar_cells(40, 0.0, 3, 24, 5),
            (0, (0, 0)),
            "nothing filled, no highlight"
        );
        let (filled, (start, len)) = bar_cells(10, 1.0, 100, 24, 50);
        assert_eq!((filled, len), (10, 10));
        assert_eq!(start, 0, "a highlight as wide as the bar cannot move");
    }
}
