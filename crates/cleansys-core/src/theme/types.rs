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
