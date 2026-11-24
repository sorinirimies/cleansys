//! Platform-agnostic theme types.
//!
//! These are the shared colour definitions that both the GUI (Iced) and TUI
//! (Ratatui) frontends convert from. Defined once in `cleansys-core` so that
//! every frontend renders exactly the same palette for a given theme name.

/// A single RGB color, platform-agnostic (0–255 range per channel).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    /// Create a new `Rgb` value. Usable in `const` contexts.
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// WCAG relative luminance (0 = black, 1 = white).
    pub fn luminance(self) -> f64 {
        let f = |v: u8| {
            let v = f64::from(v) / 255.0;
            if v <= 0.039_28 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * f(self.r) + 0.7152 * f(self.g) + 0.0722 * f(self.b)
    }

    /// WCAG contrast ratio of two colours (1 = identical, 21 = black on white).
    pub fn contrast(self, other: Self) -> f64 {
        let (a, b) = (self.luminance(), other.luminance());
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }

    /// `t` of the way from `self` to `other` (0 = self, 1 = other).
    #[must_use]
    pub fn mix(self, other: Self, t: f64) -> Self {
        let m = |a: u8, b: u8| {
            let v = f64::from(a) + (f64::from(b) - f64::from(a)) * t.clamp(0.0, 1.0);
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let out = v.round().clamp(0.0, 255.0) as u8;
            out
        };
        Self::new(m(self.r, other.r), m(self.g, other.g), m(self.b, other.b))
    }

    /// `self` moved toward `toward` in small steps until it reaches `min` contrast with
    /// `against` (unchanged when it already does).
    #[must_use]
    pub fn readable_on(self, against: Self, min: f64, toward: Self) -> Self {
        (0..=20)
            .map(|step| self.mix(toward, f64::from(step) / 20.0))
            .find(|c| c.contrast(against) >= min)
            .unwrap_or(toward)
    }
}

/// Minimum contrast against the background a theme must keep for each use
/// (WCAG AA for body text; the rest are what stays legible at a glance).
pub mod min_contrast {
    pub const BODY: f64 = 4.5;
    pub const SECONDARY: f64 = 3.0;
    pub const HINT: f64 = 1.5;
    pub const ACCENT: f64 = 2.0;
    pub const SELECTED_TEXT: f64 = 3.0;
    pub const STATUS: f64 = 2.0;
    pub const CHART: f64 = 1.3;
}

/// A complete UI colour theme — every semantic colour slot that both the GUI
/// and TUI need.
///
/// Defined once in core; each frontend converts to its framework-specific
/// colour type (e.g. `iced::Color`, `ratatui::style::Color`).
#[derive(Debug, Clone)]
pub struct AppTheme {
    /// `true` for dark themes, `false` for light themes.
    pub is_dark: bool,

    // ── Structural colours ───────────────────────────────────────────────
    /// Main window / terminal background.
    pub background: Rgb,
    /// Slightly elevated surface (panels, cards).
    pub surface: Rgb,
    /// Borders and dividers.
    pub border: Rgb,
    /// Background for selected / highlighted rows.
    pub selection: Rgb,

    // ── Text colours ─────────────────────────────────────────────────────
    /// Primary (body) text.
    pub text_primary: Rgb,
    /// Secondary / less prominent text.
    pub text_secondary: Rgb,
    /// Muted / disabled text.
    pub text_muted: Rgb,

    // ── Semantic colours ─────────────────────────────────────────────────
    /// Accent / primary action colour.
    pub accent: Rgb,
    /// Success (e.g. staged, added).
    pub success: Rgb,
    /// Warning (e.g. modified, in-progress).
    pub warning: Rgb,
    /// Error / danger (e.g. deleted, conflict).
    pub error: Rgb,

    // ── Diff colours ─────────────────────────────────────────────────────
    /// Added lines.
    pub diff_add: Rgb,
    /// Deleted lines.
    pub diff_del: Rgb,
    /// Context (unchanged) lines.
    pub diff_context: Rgb,
    /// Hunk headers.
    pub diff_hunk: Rgb,

    // ── Graph lane colours ───────────────────────────────────────────────
    /// Eight colours cycled across branch lanes in the commit graph.
    /// Each theme defines its own palette so that lanes remain legible
    /// against both dark and light backgrounds.
    pub graph_colors: [Rgb; 8],
}

impl AppTheme {
    /// The same theme with every colour that is too faint for its use nudged until it is
    /// readable (text toward black/white, the selection toward the background). Colours
    /// that already pass are left exactly as designed.
    #[must_use]
    pub fn readable(mut self) -> Self {
        use min_contrast as m;
        let bg = self.background;
        let ink = if bg.luminance() < 0.5 {
            Rgb::new(255, 255, 255)
        } else {
            Rgb::new(0, 0, 0)
        };
        self.text_primary = self.text_primary.readable_on(bg, m::BODY, ink);
        self.text_secondary = self.text_secondary.readable_on(bg, m::SECONDARY, ink);
        self.text_muted = self.text_muted.readable_on(bg, m::HINT, ink);
        self.accent = self.accent.readable_on(bg, m::ACCENT, ink);
        self.success = self.success.readable_on(bg, m::STATUS, ink);
        self.warning = self.warning.readable_on(bg, m::STATUS, ink);
        self.error = self.error.readable_on(bg, m::STATUS, ink);
        for g in &mut self.graph_colors {
            *g = g.readable_on(bg, m::CHART, ink);
        }
        // A selection the text cannot be read on moves toward the background first,
        // then (if the text is close to the background too) toward the opposite ink.
        let text = self.text_primary;
        let sel = self.selection.readable_on(text, m::SELECTED_TEXT, bg);
        self.selection = if sel.contrast(text) >= m::SELECTED_TEXT {
            sel
        } else {
            let away = if text.luminance() < 0.5 {
                Rgb::new(255, 255, 255)
            } else {
                Rgb::new(0, 0, 0)
            };
            self.selection.readable_on(text, m::SELECTED_TEXT, away)
        };
        self
    }
}
