//! Themed, opaque, content-sized dialog box.
//!
//! Temporary copy of `dodai_ratatui::dialog`; switch to the crate once it is published
//! (`dodai-ratatui` on crates.io) and delete this file.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap},
};
use tui_nuances::Theme;

/// Which theme colour frames the dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(dead_code, reason = "mirrors dodai_ratatui::DialogTone")]
pub(crate) enum DialogTone {
    /// The theme accent.
    #[default]
    Accent,
    /// Needs attention (confirmations).
    Warning,
    /// Something failed.
    Error,
    /// Neutral information.
    Info,
    /// Something worked.
    Success,
}

/// A centred modal box: border in the tone colour, title, body lines and an optional hint line.
#[derive(Debug, Clone)]
pub(crate) struct Dialog {
    theme: Theme,
    title: String,
    tone: DialogTone,
    lines: Vec<Line<'static>>,
    hint: Option<String>,
    width: u16,
    max_height: Option<u16>,
}

#[allow(dead_code, reason = "mirrors the dodai_ratatui::Dialog API")]
impl Dialog {
    /// A dialog with a title, 60 columns wide.
    pub(crate) fn new(theme: &Theme, title: impl Into<String>) -> Self {
        Self {
            theme: theme.clone(),
            title: title.into(),
            tone: DialogTone::default(),
            lines: Vec::new(),
            hint: None,
            width: 60,
            max_height: None,
        }
    }

    /// Frame colour.
    #[must_use]
    pub(crate) fn tone(mut self, tone: DialogTone) -> Self {
        self.tone = tone;
        self
    }

    /// Body text, one entry per row.
    #[must_use]
    pub(crate) fn lines(mut self, lines: Vec<Line<'static>>) -> Self {
        self.lines = lines;
        self
    }

    /// Key hint shown bold in the success colour under the body, e.g. `Enter: ok`.
    #[must_use]
    pub(crate) fn hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    /// Preferred width in columns (never wider than the area).
    #[must_use]
    pub(crate) fn width(mut self, width: u16) -> Self {
        self.width = width;
        self
    }

    /// Never taller than this many rows (the body is cut by the border).
    #[must_use]
    pub(crate) fn max_height(mut self, rows: u16) -> Self {
        self.max_height = Some(rows);
        self
    }

    fn tone_color(&self) -> ratatui::style::Color {
        let t = &self.theme;
        match self.tone {
            DialogTone::Accent => t.accent,
            DialogTone::Warning => t.warning,
            DialogTone::Error => t.error,
            DialogTone::Info => t.info,
            DialogTone::Success => t.success,
        }
    }

    /// The rectangle this dialog takes inside `area`: its width, and as tall as its content
    /// (body, hint, borders, plus a spare row for wrapped text), centred.
    #[must_use]
    pub(crate) fn rect_in(&self, area: Rect) -> Rect {
        let rows = self.lines.len() + usize::from(self.hint.is_some()) * 2;
        let mut h = u16::try_from(rows + 3).unwrap_or(u16::MAX);
        if let Some(max) = self.max_height {
            h = h.min(max);
        }
        fit(area, self.width, h)
    }

    /// Draw into `area` itself (no sizing): clear, paint the theme background, frame, text.
    pub(crate) fn render_in(&self, rect: Rect, buf: &mut Buffer) {
        let t = &self.theme;
        paint_background(&self.theme, rect, buf);
        let mut lines = self.lines.clone();
        if let Some(hint) = &self.hint {
            lines.push(Line::raw(""));
            lines.push(Line::styled(
                hint.clone(),
                Style::default().fg(t.success).add_modifier(Modifier::BOLD),
            ));
        }
        Paragraph::new(lines)
            .block(
                Block::default()
                    .title(self.title.clone())
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(self.tone_color())),
            )
            .style(
                Style::default()
                    .fg(t.foreground)
                    .bg(t.background.unwrap_or_default()),
            )
            .wrap(Wrap { trim: true })
            .render(rect, buf);
    }

    /// Size to the content, centre in `area`, draw, and return the rectangle used.
    pub(crate) fn render_centered(&self, area: Rect, buf: &mut Buffer) -> Rect {
        let rect = self.rect_in(area);
        self.render_in(rect, buf);
        rect
    }
}

/// Wipe `rect` and fill it with the theme's background and text colour, so the area is opaque
/// even where the reset colour is transparent. For popups you draw yourself.
pub(crate) fn paint_background(t: &Theme, rect: Rect, buf: &mut Buffer) {
    Clear.render(rect, buf);
    Block::default()
        .style(
            Style::default()
                .fg(t.foreground)
                .bg(t.background.unwrap_or_default()),
        )
        .render(rect, buf);
}

/// A `width` × `height` rectangle centred in `area`, clamped to it.
#[must_use]
pub(crate) fn fit(area: Rect, width: u16, height: u16) -> Rect {
    let w = width.min(area.width);
    let h = height.min(area.height);
    Rect::new(
        area.x + (area.width - w) / 2,
        area.y + (area.height - h) / 2,
        w,
        h,
    )
}

/// A centred rectangle of `w_pct`/`h_pct` percent of `area`, at least 30×10 (or the whole
/// area when smaller) and at most `max_width` columns wide.
#[must_use]
pub(crate) fn centered(area: Rect, w_pct: u16, h_pct: u16, max_width: u16) -> Rect {
    let (min_w, min_h) = (30.min(area.width), 10.min(area.height));
    let w = (area.width * w_pct / 100)
        .clamp(min_w, area.width.saturating_sub(2).max(min_w))
        .min(max_width);
    let h = (area.height * h_pct / 100).clamp(min_h, area.height.saturating_sub(2).max(min_h));
    fit(area, w, h)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Color;

    fn theme() -> Theme {
        Theme::default().background(Some(Color::Rgb(1, 2, 3)))
    }

    #[test]
    fn height_follows_content_not_the_terminal() {
        let d = Dialog::new(&theme(), "t").lines(vec![Line::raw("a"), Line::raw("b")]);
        let small = d.rect_in(Rect::new(0, 0, 100, 30));
        let huge = d.rect_in(Rect::new(0, 0, 300, 90));
        assert_eq!(small.height, 5);
        assert_eq!(huge.height, 5);
        assert_eq!(huge.width, 60);
    }

    #[test]
    fn hint_adds_a_blank_row_and_the_hint() {
        let d = Dialog::new(&theme(), "t")
            .lines(vec![Line::raw("a")])
            .hint("ok");
        assert_eq!(d.rect_in(Rect::new(0, 0, 100, 30)).height, 6);
    }

    #[test]
    fn never_exceeds_the_area_or_max_height() {
        let d = Dialog::new(&theme(), "t")
            .lines(vec![Line::raw("x"); 40])
            .max_height(12);
        assert_eq!(d.rect_in(Rect::new(0, 0, 100, 30)).height, 12);
        let tiny = Rect::new(0, 0, 20, 4);
        let r = d.rect_in(tiny);
        assert!(r.width <= 20 && r.height <= 4);
    }

    #[test]
    fn is_opaque_and_framed_in_the_tone_colour() {
        let area = Rect::new(0, 0, 40, 10);
        let mut buf = Buffer::empty(area);
        // Stale content underneath must not survive.
        buf.set_string(0, 0, "XXXXXXXXXXXXXXXXXXXX", Style::default());
        let th = theme();
        let r = Dialog::new(&th, "Hi")
            .tone(DialogTone::Error)
            .lines(vec![Line::raw("body")])
            .render_centered(area, &mut buf);
        let inner = &buf[(r.x + 1, r.y + 1)];
        assert_eq!(inner.bg, Color::Rgb(1, 2, 3));
        assert_eq!(buf[(r.x, r.y)].fg, th.error);
        assert_eq!(buf[(r.x + 1, r.y + 1)].symbol(), "b");
    }

    #[test]
    fn centered_is_capped_in_width() {
        let r = centered(Rect::new(0, 0, 300, 60), 82, 88, 100);
        assert_eq!(r.width, 100);
        let small = centered(Rect::new(0, 0, 60, 20), 82, 88, 100);
        assert!(small.width <= 60 && small.x + small.width <= 60);
    }
}
