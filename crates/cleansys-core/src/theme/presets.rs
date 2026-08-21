//! All 43 preset themes — the **single source of truth** for every colour
//! used by both the GUI and TUI front-ends.
//!
//! Each public function returns a fully-populated [`AppTheme`] with concrete
//! RGB values. The [`theme_by_index`] dispatcher maps a `0..=42` index to the
//! matching constructor; out-of-range indices silently fall back to `default()`.

use super::types::{AppTheme, Rgb};

// ── Theme catalogue ───────────────────────────────────────────────────────────

/// Ordered theme names. The position in this slice **is** the canonical index.
pub const THEME_NAMES: &[&str] = &[
    "Default",
    "Default Light",
    "Grape",
    "Ocean",
    "Sunset",
    "Forest",
    "Rose",
    "Mono",
    "Neon",
    "Dracula",
    "Nord",
    "Solarized Dark",
    "Solarized Light",
    "Gruvbox Dark",
    "Gruvbox Light",
    "Catppuccin Latte",
    "Catppuccin Frappé",
    "Catppuccin Macchiato",
    "Catppuccin Mocha",
    "Tokyo Night",
    "Tokyo Night Storm",
    "Tokyo Night Light",
    "Kanagawa Wave",
    "Kanagawa Dragon",
    "Kanagawa Lotus",
    "Moonfly",
    "Nightfly",
    "Oxocarbon",
    "Cyberpunk",
    "Rose Pine",
    "Rose Pine Moon",
    "Rose Pine Dawn",
    "Ayu Mirage",
    "Everforest Dark",
    "Atom One Dark",
    "Atom One Light",
    "Night Owl",
    "Poimandres",
    "Flexoki Dark",
    "Flexoki Light",
    "Carbonfox",
    "Andromeda",
    "Synthwave",
];

/// Total number of themes.
pub const THEME_COUNT: usize = 43;

/// Get a theme by index (0-based). Returns `default()` for out-of-range.
pub fn theme_by_index(index: usize) -> AppTheme {
    match index {
        0 => default(),
        1 => default_light(),
        2 => grape(),
        3 => ocean(),
        4 => sunset(),
        5 => forest(),
        6 => rose(),
        7 => mono(),
        8 => neon(),
        9 => dracula(),
        10 => nord(),
        11 => solarized_dark(),
        12 => solarized_light(),
        13 => gruvbox_dark(),
        14 => gruvbox_light(),
        15 => catppuccin_latte(),
        16 => catppuccin_frappe(),
        17 => catppuccin_macchiato(),
        18 => catppuccin_mocha(),
        19 => tokyo_night(),
        20 => tokyo_night_storm(),
        21 => tokyo_night_light(),
        22 => kanagawa_wave(),
        23 => kanagawa_dragon(),
        24 => kanagawa_lotus(),
        25 => moonfly(),
        26 => nightfly(),
        27 => oxocarbon(),
        28 => cyberpunk(),
        29 => rose_pine(),
        30 => rose_pine_moon(),
        31 => rose_pine_dawn(),
        32 => ayu_mirage(),
        33 => everforest_dark(),
        34 => atom_one_dark(),
        35 => atom_one_light(),
        36 => night_owl(),
        37 => poimandres(),
        38 => flexoki_dark(),
        39 => flexoki_light(),
        40 => carbonfox(),
        41 => andromeda(),
        42 => synthwave(),
        _ => default(),
    }
}

/// Find theme index by name (case-insensitive ASCII). Returns `0` (Default) if
/// no match is found.
pub fn theme_index_by_name(name: &str) -> usize {
    THEME_NAMES
        .iter()
        .position(|n| n.eq_ignore_ascii_case(name))
        .unwrap_or(0)
}

// ── Theme definitions ─────────────────────────────────────────────────────────

pub fn default() -> AppTheme {
    AppTheme {
        is_dark: true,
        background: Rgb::new(18, 18, 26),
        surface: Rgb::new(28, 28, 40),
        border: Rgb::new(60, 80, 100),
        selection: Rgb::new(40, 60, 80),
        text_primary: Rgb::new(255, 255, 255),
        text_secondary: Rgb::new(255, 100, 30),
        text_muted: Rgb::new(120, 120, 130),
        accent: Rgb::new(80, 200, 255),
        success: Rgb::new(80, 220, 120),
        warning: Rgb::new(255, 200, 50),
        error: Rgb::new(255, 80, 80),
        diff_add: Rgb::new(80, 220, 120),
        diff_del: Rgb::new(255, 80, 80),
        diff_context: Rgb::new(120, 120, 130),
        diff_hunk: Rgb::new(255, 100, 30),
        graph_colors: [
            Rgb::new(80, 200, 255),
            Rgb::new(80, 220, 120),
            Rgb::new(255, 80, 80),
            Rgb::new(255, 200, 50),
            Rgb::new(255, 100, 30),
            Rgb::new(180, 140, 255),
            Rgb::new(0, 200, 180),
            Rgb::new(255, 130, 160),
        ],
    }
}

pub fn default_light() -> AppTheme {
    AppTheme {
        is_dark: false,
        background: Rgb::new(250, 250, 255),
        surface: Rgb::new(235, 238, 248),
        border: Rgb::new(180, 195, 220),
        selection: Rgb::new(200, 220, 255),
        text_primary: Rgb::new(30, 35, 50),
        text_secondary: Rgb::new(255, 100, 30),
        text_muted: Rgb::new(130, 140, 160),
        accent: Rgb::new(0, 140, 220),
        success: Rgb::new(30, 160, 80),
        warning: Rgb::new(200, 140, 0),
        error: Rgb::new(210, 50, 50),
        diff_add: Rgb::new(30, 160, 80),
        diff_del: Rgb::new(210, 50, 50),
        diff_context: Rgb::new(130, 140, 160),
        diff_hunk: Rgb::new(255, 100, 30),
        graph_colors: [
            Rgb::new(0, 140, 220),
            Rgb::new(30, 160, 80),
            Rgb::new(210, 50, 50),
            Rgb::new(200, 140, 0),
            Rgb::new(255, 100, 30),
            Rgb::new(140, 80, 200),
            Rgb::new(0, 150, 150),
            Rgb::new(190, 60, 120),
        ],
    }
}

pub fn grape() -> AppTheme {
    AppTheme {
        is_dark: true,
        background: Rgb::new(18, 12, 30),
        surface: Rgb::new(30, 20, 50),
        border: Rgb::new(100, 70, 150),
        selection: Rgb::new(50, 35, 80),
        text_primary: Rgb::new(230, 220, 255),
        text_secondary: Rgb::new(130, 180, 255),
        text_muted: Rgb::new(110, 100, 130),
        accent: Rgb::new(200, 120, 255),
        success: Rgb::new(160, 110, 255),
        warning: Rgb::new(210, 170, 255),
        error: Rgb::new(255, 80, 150),
        diff_add: Rgb::new(160, 110, 255),
        diff_del: Rgb::new(255, 80, 150),
        diff_context: Rgb::new(110, 100, 130),
        diff_hunk: Rgb::new(130, 180, 255),
        graph_colors: [
            Rgb::new(200, 120, 255),
            Rgb::new(160, 110, 255),
            Rgb::new(255, 80, 150),
            Rgb::new(210, 170, 255),
            Rgb::new(130, 180, 255),
            Rgb::new(80, 220, 200),
            Rgb::new(255, 180, 100),
            Rgb::new(120, 255, 160),
        ],
    }
}

pub fn ocean() -> AppTheme {
    AppTheme {
        is_dark: true,
        background: Rgb::new(0, 20, 35),
        surface: Rgb::new(0, 35, 55),
        border: Rgb::new(0, 100, 130),
        selection: Rgb::new(0, 50, 70),
        text_primary: Rgb::new(200, 240, 245),
        text_secondary: Rgb::new(0, 175, 210),
        text_muted: Rgb::new(80, 120, 130),
        accent: Rgb::new(0, 200, 180),
        success: Rgb::new(80, 230, 200),
        warning: Rgb::new(255, 220, 80),
        error: Rgb::new(255, 100, 100),
        diff_add: Rgb::new(80, 230, 200),
        diff_del: Rgb::new(255, 100, 100),
        diff_context: Rgb::new(80, 120, 130),
        diff_hunk: Rgb::new(0, 175, 210),
        graph_colors: [
            Rgb::new(0, 200, 180),
            Rgb::new(80, 230, 200),
            Rgb::new(255, 100, 100),
            Rgb::new(255, 220, 80),
            Rgb::new(0, 175, 210),
            Rgb::new(180, 130, 255),
            Rgb::new(255, 160, 100),
            Rgb::new(220, 120, 200),
        ],
    }
}

pub fn sunset() -> AppTheme {
    AppTheme {
        is_dark: true,
        background: Rgb::new(22, 8, 6),
        surface: Rgb::new(40, 15, 10),
        border: Rgb::new(130, 60, 30),
        selection: Rgb::new(80, 30, 20),
        text_primary: Rgb::new(255, 235, 210),
        text_secondary: Rgb::new(255, 150, 50),
        text_muted: Rgb::new(140, 100, 80),
        accent: Rgb::new(255, 80, 80),
        success: Rgb::new(255, 180, 80),
        warning: Rgb::new(255, 230, 80),
        error: Rgb::new(255, 50, 50),
        diff_add: Rgb::new(255, 180, 80),
        diff_del: Rgb::new(255, 50, 50),
        diff_context: Rgb::new(140, 100, 80),
        diff_hunk: Rgb::new(255, 150, 50),
        graph_colors: [
            Rgb::new(255, 80, 80),
            Rgb::new(255, 180, 80),
            Rgb::new(255, 50, 50),
            Rgb::new(255, 230, 80),
            Rgb::new(255, 150, 50),
            Rgb::new(100, 180, 255),
            Rgb::new(180, 120, 255),
            Rgb::new(80, 220, 200),
        ],
    }
}

pub fn forest() -> AppTheme {
    AppTheme {
        is_dark: true,
        background: Rgb::new(8, 18, 8),
        surface: Rgb::new(15, 30, 15),
        border: Rgb::new(50, 100, 50),
        selection: Rgb::new(20, 50, 20),
        text_primary: Rgb::new(210, 235, 200),
        text_secondary: Rgb::new(80, 160, 80),
        text_muted: Rgb::new(90, 120, 80),
        accent: Rgb::new(100, 200, 80),
        success: Rgb::new(120, 210, 90),
        warning: Rgb::new(220, 200, 80),
        error: Rgb::new(210, 80, 80),
        diff_add: Rgb::new(120, 210, 90),
        diff_del: Rgb::new(210, 80, 80),
        diff_context: Rgb::new(90, 120, 80),
        diff_hunk: Rgb::new(80, 160, 80),
        graph_colors: [
            Rgb::new(100, 200, 80),
            Rgb::new(120, 210, 90),
            Rgb::new(210, 80, 80),
            Rgb::new(220, 200, 80),
            Rgb::new(80, 160, 80),
            Rgb::new(130, 160, 255),
            Rgb::new(200, 130, 180),
            Rgb::new(200, 180, 100),
        ],
    }
}

pub fn rose() -> AppTheme {
    AppTheme {
        is_dark: true,
        background: Rgb::new(28, 6, 16),
        surface: Rgb::new(50, 12, 30),
        border: Rgb::new(140, 60, 100),
        selection: Rgb::new(80, 20, 40),
        text_primary: Rgb::new(255, 230, 235),
        text_secondary: Rgb::new(255, 140, 180),
        text_muted: Rgb::new(140, 90, 110),
        accent: Rgb::new(255, 100, 150),
        success: Rgb::new(255, 160, 190),
        warning: Rgb::new(255, 220, 180),
        error: Rgb::new(220, 60, 100),
        diff_add: Rgb::new(255, 160, 190),
        diff_del: Rgb::new(220, 60, 100),
        diff_context: Rgb::new(140, 90, 110),
        diff_hunk: Rgb::new(255, 140, 180),
        graph_colors: [
            Rgb::new(255, 100, 150),
            Rgb::new(255, 160, 190),
            Rgb::new(220, 60, 100),
            Rgb::new(255, 220, 180),
            Rgb::new(255, 140, 180),
            Rgb::new(100, 180, 255),
            Rgb::new(120, 220, 200),
            Rgb::new(200, 180, 100),
        ],
    }
}

pub fn mono() -> AppTheme {
    AppTheme {
        is_dark: true,
        background: Rgb::new(8, 8, 10),
        surface: Rgb::new(20, 20, 22),
        border: Rgb::new(80, 80, 85),
        selection: Rgb::new(50, 50, 55),
        text_primary: Rgb::new(210, 210, 210),
        text_secondary: Rgb::new(180, 180, 180),
        text_muted: Rgb::new(110, 110, 115),
        accent: Rgb::new(200, 200, 200),
        success: Rgb::new(200, 200, 200),
        warning: Rgb::new(200, 200, 200),
        error: Rgb::new(160, 160, 160),
        diff_add: Rgb::new(200, 200, 200),
        diff_del: Rgb::new(160, 160, 160),
        diff_context: Rgb::new(110, 110, 115),
        diff_hunk: Rgb::new(180, 180, 180),
        graph_colors: [
            Rgb::new(230, 230, 230),
            Rgb::new(200, 200, 200),
            Rgb::new(160, 160, 160),
            Rgb::new(215, 210, 205),
            Rgb::new(180, 180, 180),
            Rgb::new(145, 150, 155),
            Rgb::new(195, 190, 200),
            Rgb::new(170, 175, 170),
        ],
    }
}

pub fn neon() -> AppTheme {
    AppTheme {
        is_dark: true,
        background: Rgb::new(6, 0, 14),
        surface: Rgb::new(15, 0, 30),
        border: Rgb::new(100, 0, 140),
        selection: Rgb::new(30, 0, 50),
        text_primary: Rgb::new(230, 230, 255),
        text_secondary: Rgb::new(0, 255, 200),
        text_muted: Rgb::new(100, 80, 120),
        accent: Rgb::new(255, 0, 200),
        success: Rgb::new(0, 255, 130),
        warning: Rgb::new(255, 220, 0),
        error: Rgb::new(255, 30, 80),
        diff_add: Rgb::new(0, 255, 130),
        diff_del: Rgb::new(255, 30, 80),
        diff_context: Rgb::new(100, 80, 120),
        diff_hunk: Rgb::new(0, 255, 200),
        graph_colors: [
            Rgb::new(255, 0, 200),
            Rgb::new(0, 255, 130),
            Rgb::new(255, 30, 80),
            Rgb::new(255, 220, 0),
            Rgb::new(0, 255, 200),
            Rgb::new(100, 80, 255),
            Rgb::new(255, 120, 0),
            Rgb::new(0, 180, 255),
        ],
    }
}

pub fn dracula() -> AppTheme {
    AppTheme {
        is_dark: true,
        background: Rgb::new(40, 42, 54),
        surface: Rgb::new(68, 71, 90),
        border: Rgb::new(98, 114, 164),
        selection: Rgb::new(68, 71, 90),
        text_primary: Rgb::new(248, 248, 242),
        text_secondary: Rgb::new(139, 233, 253),
        text_muted: Rgb::new(98, 114, 164),
        accent: Rgb::new(255, 121, 198),
        success: Rgb::new(80, 250, 123),
        warning: Rgb::new(241, 250, 140),
        error: Rgb::new(255, 85, 85),
        diff_add: Rgb::new(80, 250, 123),
        diff_del: Rgb::new(255, 85, 85),
        diff_context: Rgb::new(98, 114, 164),
        diff_hunk: Rgb::new(139, 233, 253),
        graph_colors: [
            Rgb::new(255, 121, 198),
            Rgb::new(80, 250, 123),
            Rgb::new(255, 85, 85),
            Rgb::new(241, 250, 140),
            Rgb::new(139, 233, 253),
            Rgb::new(189, 147, 249),
            Rgb::new(255, 184, 108),
            Rgb::new(98, 114, 164),
        ],
    }
}

pub fn nord() -> AppTheme {
    AppTheme {
        is_dark: true,
        background: Rgb::new(29, 35, 42),
        surface: Rgb::new(59, 66, 82),
        border: Rgb::new(76, 86, 106),
        selection: Rgb::new(59, 66, 82),
        text_primary: Rgb::new(216, 222, 233),
        text_secondary: Rgb::new(129, 161, 193),
        text_muted: Rgb::new(76, 86, 106),
        accent: Rgb::new(136, 192, 208),
        success: Rgb::new(163, 190, 140),
        warning: Rgb::new(235, 203, 139),
        error: Rgb::new(191, 97, 106),
        diff_add: Rgb::new(163, 190, 140),
        diff_del: Rgb::new(191, 97, 106),
        diff_context: Rgb::new(76, 86, 106),
        diff_hunk: Rgb::new(129, 161, 193),
        graph_colors: [
            Rgb::new(136, 192, 208),
            Rgb::new(163, 190, 140),
            Rgb::new(191, 97, 106),
            Rgb::new(235, 203, 139),
            Rgb::new(129, 161, 193),
            Rgb::new(180, 142, 173),
            Rgb::new(208, 135, 112),
            Rgb::new(143, 188, 187),
        ],
    }
}

pub fn solarized_dark() -> AppTheme {
    AppTheme {
        is_dark: true,
        background: Rgb::new(0, 43, 54),
        surface: Rgb::new(7, 54, 66),
        border: Rgb::new(88, 110, 117),
        selection: Rgb::new(7, 54, 66),
        text_primary: Rgb::new(131, 148, 150),
        text_secondary: Rgb::new(42, 161, 152),
        text_muted: Rgb::new(88, 110, 117),
        accent: Rgb::new(38, 139, 210),
        success: Rgb::new(133, 153, 0),
        warning: Rgb::new(181, 137, 0),
        error: Rgb::new(220, 50, 47),
        diff_add: Rgb::new(133, 153, 0),
        diff_del: Rgb::new(220, 50, 47),
        diff_context: Rgb::new(88, 110, 117),
        diff_hunk: Rgb::new(42, 161, 152),
        graph_colors: [
            Rgb::new(38, 139, 210),
            Rgb::new(133, 153, 0),
            Rgb::new(220, 50, 47),
            Rgb::new(181, 137, 0),
            Rgb::new(42, 161, 152),
            Rgb::new(108, 113, 196),
            Rgb::new(203, 75, 22),
            Rgb::new(211, 54, 130),
        ],
    }
}

pub fn solarized_light() -> AppTheme {
    AppTheme {
        is_dark: false,
        background: Rgb::new(253, 246, 227),
        surface: Rgb::new(238, 232, 213),
        border: Rgb::new(147, 161, 161),
        selection: Rgb::new(238, 232, 213),
        text_primary: Rgb::new(101, 123, 131),
        text_secondary: Rgb::new(42, 161, 152),
        text_muted: Rgb::new(147, 161, 161),
        accent: Rgb::new(38, 139, 210),
        success: Rgb::new(133, 153, 0),
        warning: Rgb::new(181, 137, 0),
        error: Rgb::new(220, 50, 47),
        diff_add: Rgb::new(133, 153, 0),
        diff_del: Rgb::new(220, 50, 47),
        diff_context: Rgb::new(147, 161, 161),
        diff_hunk: Rgb::new(42, 161, 152),
        graph_colors: [
            Rgb::new(38, 139, 210),
            Rgb::new(133, 153, 0),
            Rgb::new(220, 50, 47),
            Rgb::new(181, 137, 0),
            Rgb::new(42, 161, 152),
            Rgb::new(108, 113, 196),
            Rgb::new(203, 75, 22),
            Rgb::new(211, 54, 130),
        ],
    }
}

pub fn gruvbox_dark() -> AppTheme {
    AppTheme {
        is_dark: true,
        background: Rgb::new(29, 28, 27),
        surface: Rgb::new(60, 56, 54),
        border: Rgb::new(146, 131, 116),
        selection: Rgb::new(60, 56, 54),
        text_primary: Rgb::new(235, 219, 178),
        text_secondary: Rgb::new(250, 189, 47),
        text_muted: Rgb::new(146, 131, 116),
        accent: Rgb::new(254, 128, 25),
        success: Rgb::new(184, 187, 38),
        warning: Rgb::new(250, 189, 47),
        error: Rgb::new(251, 73, 52),
        diff_add: Rgb::new(184, 187, 38),
        diff_del: Rgb::new(251, 73, 52),
        diff_context: Rgb::new(146, 131, 116),
        diff_hunk: Rgb::new(250, 189, 47),
        graph_colors: [
            Rgb::new(254, 128, 25),
            Rgb::new(184, 187, 38),
            Rgb::new(251, 73, 52),
            Rgb::new(250, 189, 47),
            Rgb::new(131, 165, 152),
            Rgb::new(211, 134, 155),
            Rgb::new(142, 192, 124),
            Rgb::new(69, 133, 136),
        ],
    }
}

pub fn gruvbox_light() -> AppTheme {
    AppTheme {
        is_dark: false,
        background: Rgb::new(251, 241, 199),
        surface: Rgb::new(213, 196, 161),
        border: Rgb::new(146, 131, 116),
        selection: Rgb::new(213, 196, 161),
        text_primary: Rgb::new(60, 56, 54),
        text_secondary: Rgb::new(215, 153, 33),
        text_muted: Rgb::new(146, 131, 116),
        accent: Rgb::new(214, 93, 14),
        success: Rgb::new(121, 116, 14),
        warning: Rgb::new(215, 153, 33),
        error: Rgb::new(214, 93, 14),
        diff_add: Rgb::new(121, 116, 14),
        diff_del: Rgb::new(214, 93, 14),
        diff_context: Rgb::new(146, 131, 116),
        diff_hunk: Rgb::new(215, 153, 33),
        graph_colors: [
            Rgb::new(214, 93, 14),
            Rgb::new(121, 116, 14),
            Rgb::new(204, 36, 29),
            Rgb::new(215, 153, 33),
            Rgb::new(69, 133, 136),
            Rgb::new(177, 98, 134),
            Rgb::new(104, 157, 106),
            Rgb::new(7, 102, 120),
        ],
    }
}

pub fn catppuccin_latte() -> AppTheme {
    AppTheme {
        is_dark: false,
        background: Rgb::new(239, 241, 245),
        surface: Rgb::new(204, 208, 218),
        border: Rgb::new(156, 160, 176),
        selection: Rgb::new(204, 208, 218),
        text_primary: Rgb::new(76, 79, 105),
        text_secondary: Rgb::new(30, 102, 245),
        text_muted: Rgb::new(156, 160, 176),
        accent: Rgb::new(136, 57, 239),
        success: Rgb::new(64, 160, 43),
        warning: Rgb::new(223, 142, 29),
        error: Rgb::new(210, 15, 57),
        diff_add: Rgb::new(64, 160, 43),
        diff_del: Rgb::new(210, 15, 57),
        diff_context: Rgb::new(156, 160, 176),
        diff_hunk: Rgb::new(30, 102, 245),
        graph_colors: [
            Rgb::new(136, 57, 239),
            Rgb::new(64, 160, 43),
            Rgb::new(210, 15, 57),
            Rgb::new(223, 142, 29),
            Rgb::new(30, 102, 245),
            Rgb::new(23, 146, 153),
            Rgb::new(234, 118, 203),
            Rgb::new(254, 100, 11),
        ],
    }
}

pub fn catppuccin_frappe() -> AppTheme {
    AppTheme {
        is_dark: true,
        background: Rgb::new(48, 52, 70),
        surface: Rgb::new(65, 69, 89),
        border: Rgb::new(115, 121, 148),
        selection: Rgb::new(65, 69, 89),
        text_primary: Rgb::new(198, 208, 245),
        text_secondary: Rgb::new(140, 170, 238),
        text_muted: Rgb::new(115, 121, 148),
        accent: Rgb::new(202, 158, 230),
        success: Rgb::new(166, 209, 137),
        warning: Rgb::new(229, 200, 144),
        error: Rgb::new(231, 130, 132),
        diff_add: Rgb::new(166, 209, 137),
        diff_del: Rgb::new(231, 130, 132),
        diff_context: Rgb::new(115, 121, 148),
        diff_hunk: Rgb::new(140, 170, 238),
        graph_colors: [
            Rgb::new(202, 158, 230),
            Rgb::new(166, 209, 137),
            Rgb::new(231, 130, 132),
            Rgb::new(229, 200, 144),
            Rgb::new(140, 170, 238),
            Rgb::new(129, 200, 190),
            Rgb::new(244, 184, 228),
            Rgb::new(239, 159, 118),
        ],
    }
}

pub fn catppuccin_macchiato() -> AppTheme {
    AppTheme {
        is_dark: true,
        background: Rgb::new(36, 39, 58),
        surface: Rgb::new(54, 58, 79),
        border: Rgb::new(110, 115, 141),
        selection: Rgb::new(54, 58, 79),
        text_primary: Rgb::new(202, 211, 245),
        text_secondary: Rgb::new(138, 173, 244),
        text_muted: Rgb::new(110, 115, 141),
        accent: Rgb::new(198, 160, 246),
        success: Rgb::new(166, 218, 149),
        warning: Rgb::new(238, 212, 159),
        error: Rgb::new(237, 135, 150),
        diff_add: Rgb::new(166, 218, 149),
        diff_del: Rgb::new(237, 135, 150),
        diff_context: Rgb::new(110, 115, 141),
        diff_hunk: Rgb::new(138, 173, 244),
        graph_colors: [
            Rgb::new(198, 160, 246),
            Rgb::new(166, 218, 149),
            Rgb::new(237, 135, 150),
            Rgb::new(238, 212, 159),
            Rgb::new(138, 173, 244),
            Rgb::new(139, 213, 202),
            Rgb::new(245, 189, 230),
            Rgb::new(240, 168, 128),
        ],
    }
}

pub fn catppuccin_mocha() -> AppTheme {
    AppTheme {
        is_dark: true,
        background: Rgb::new(30, 30, 46),
        surface: Rgb::new(49, 50, 68),
        border: Rgb::new(108, 112, 134),
        selection: Rgb::new(49, 50, 68),
        text_primary: Rgb::new(205, 214, 244),
        text_secondary: Rgb::new(137, 180, 250),
        text_muted: Rgb::new(108, 112, 134),
        accent: Rgb::new(203, 166, 247),
        success: Rgb::new(166, 227, 161),
        warning: Rgb::new(249, 226, 175),
        error: Rgb::new(243, 139, 168),
        diff_add: Rgb::new(166, 227, 161),
        diff_del: Rgb::new(243, 139, 168),
        diff_context: Rgb::new(108, 112, 134),
        diff_hunk: Rgb::new(137, 180, 250),
        graph_colors: [
            Rgb::new(203, 166, 247),
            Rgb::new(166, 227, 161),
            Rgb::new(243, 139, 168),
            Rgb::new(249, 226, 175),
            Rgb::new(137, 180, 250),
            Rgb::new(148, 226, 213),
            Rgb::new(245, 194, 231),
            Rgb::new(250, 179, 135),
        ],
    }
}

pub fn tokyo_night() -> AppTheme {
    AppTheme {
        is_dark: true,
        background: Rgb::new(26, 27, 38),
        surface: Rgb::new(41, 46, 66),
        border: Rgb::new(86, 95, 137),
        selection: Rgb::new(41, 46, 66),
        text_primary: Rgb::new(192, 202, 245),
        text_secondary: Rgb::new(122, 162, 247),
        text_muted: Rgb::new(86, 95, 137),
        accent: Rgb::new(187, 154, 247),
        success: Rgb::new(158, 206, 106),
        warning: Rgb::new(224, 175, 104),
        error: Rgb::new(247, 118, 142),
        diff_add: Rgb::new(158, 206, 106),
        diff_del: Rgb::new(247, 118, 142),
        diff_context: Rgb::new(86, 95, 137),
        diff_hunk: Rgb::new(122, 162, 247),
        graph_colors: [
            Rgb::new(187, 154, 247),
            Rgb::new(158, 206, 106),
            Rgb::new(247, 118, 142),
            Rgb::new(224, 175, 104),
            Rgb::new(122, 162, 247),
            Rgb::new(125, 207, 255),
            Rgb::new(255, 158, 100),
            Rgb::new(115, 218, 202),
        ],
    }
}

pub fn tokyo_night_storm() -> AppTheme {
    AppTheme {
        is_dark: true,
        background: Rgb::new(36, 40, 59),
        surface: Rgb::new(45, 49, 75),
        border: Rgb::new(86, 95, 137),
        selection: Rgb::new(45, 49, 75),
        text_primary: Rgb::new(192, 202, 245),
        text_secondary: Rgb::new(122, 162, 247),
        text_muted: Rgb::new(86, 95, 137),
        accent: Rgb::new(187, 154, 247),
        success: Rgb::new(158, 206, 106),
        warning: Rgb::new(224, 175, 104),
        error: Rgb::new(247, 118, 142),
        diff_add: Rgb::new(158, 206, 106),
        diff_del: Rgb::new(247, 118, 142),
        diff_context: Rgb::new(86, 95, 137),
        diff_hunk: Rgb::new(122, 162, 247),
        graph_colors: [
            Rgb::new(187, 154, 247),
            Rgb::new(158, 206, 106),
            Rgb::new(247, 118, 142),
            Rgb::new(224, 175, 104),
            Rgb::new(122, 162, 247),
            Rgb::new(125, 207, 255),
            Rgb::new(255, 158, 100),
            Rgb::new(115, 218, 202),
        ],
    }
}

pub fn tokyo_night_light() -> AppTheme {
    AppTheme {
        is_dark: false,
        background: Rgb::new(213, 214, 219),
        surface: Rgb::new(208, 213, 227),
        border: Rgb::new(132, 140, 176),
        selection: Rgb::new(208, 213, 227),
        text_primary: Rgb::new(52, 59, 88),
        text_secondary: Rgb::new(46, 126, 233),
        text_muted: Rgb::new(132, 140, 176),
        accent: Rgb::new(90, 74, 120),
        success: Rgb::new(72, 94, 48),
        warning: Rgb::new(140, 108, 62),
        error: Rgb::new(210, 15, 57),
        diff_add: Rgb::new(72, 94, 48),
        diff_del: Rgb::new(210, 15, 57),
        diff_context: Rgb::new(132, 140, 176),
        diff_hunk: Rgb::new(46, 126, 233),
        graph_colors: [
            Rgb::new(90, 74, 120),
            Rgb::new(72, 94, 48),
            Rgb::new(210, 15, 57),
            Rgb::new(140, 108, 62),
            Rgb::new(46, 126, 233),
            Rgb::new(56, 140, 150),
            Rgb::new(166, 82, 140),
            Rgb::new(180, 100, 50),
        ],
    }
}

pub fn kanagawa_wave() -> AppTheme {
    AppTheme {
        is_dark: true,
        background: Rgb::new(22, 22, 30),
        surface: Rgb::new(42, 42, 55),
        border: Rgb::new(114, 113, 105),
        selection: Rgb::new(42, 42, 55),
        text_primary: Rgb::new(220, 215, 186),
        text_secondary: Rgb::new(126, 156, 216),
        text_muted: Rgb::new(114, 113, 105),
        accent: Rgb::new(210, 126, 153),
        success: Rgb::new(118, 148, 106),
        warning: Rgb::new(220, 165, 97),
        error: Rgb::new(210, 126, 153),
        diff_add: Rgb::new(118, 148, 106),
        diff_del: Rgb::new(210, 126, 153),
        diff_context: Rgb::new(114, 113, 105),
        diff_hunk: Rgb::new(126, 156, 216),
        graph_colors: [
            Rgb::new(210, 126, 153),
            Rgb::new(118, 148, 106),
            Rgb::new(255, 90, 100),
            Rgb::new(220, 165, 97),
            Rgb::new(126, 156, 216),
            Rgb::new(106, 149, 137),
            Rgb::new(228, 104, 118),
            Rgb::new(149, 127, 184),
        ],
    }
}

pub fn kanagawa_dragon() -> AppTheme {
    AppTheme {
        is_dark: true,
        background: Rgb::new(20, 20, 20),
        surface: Rgb::new(40, 39, 39),
        border: Rgb::new(166, 166, 156),
        selection: Rgb::new(40, 39, 39),
        text_primary: Rgb::new(197, 201, 197),
        text_secondary: Rgb::new(139, 164, 176),
        text_muted: Rgb::new(166, 166, 156),
        accent: Rgb::new(210, 126, 153),
        success: Rgb::new(135, 169, 135),
        warning: Rgb::new(200, 170, 109),
        error: Rgb::new(210, 126, 153),
        diff_add: Rgb::new(135, 169, 135),
        diff_del: Rgb::new(210, 126, 153),
        diff_context: Rgb::new(166, 166, 156),
        diff_hunk: Rgb::new(139, 164, 176),
        graph_colors: [
            Rgb::new(210, 126, 153),
            Rgb::new(135, 169, 135),
            Rgb::new(227, 100, 100),
            Rgb::new(200, 170, 109),
            Rgb::new(139, 164, 176),
            Rgb::new(106, 149, 137),
            Rgb::new(196, 108, 124),
            Rgb::new(165, 145, 196),
        ],
    }
}

pub fn kanagawa_lotus() -> AppTheme {
    AppTheme {
        is_dark: false,
        background: Rgb::new(246, 243, 228),
        surface: Rgb::new(231, 219, 160),
        border: Rgb::new(196, 178, 138),
        selection: Rgb::new(231, 219, 160),
        text_primary: Rgb::new(84, 84, 100),
        text_secondary: Rgb::new(77, 105, 155),
        text_muted: Rgb::new(196, 178, 138),
        accent: Rgb::new(160, 154, 190),
        success: Rgb::new(111, 137, 78),
        warning: Rgb::new(119, 113, 63),
        error: Rgb::new(192, 71, 71),
        diff_add: Rgb::new(111, 137, 78),
        diff_del: Rgb::new(192, 71, 71),
        diff_context: Rgb::new(196, 178, 138),
        diff_hunk: Rgb::new(77, 105, 155),
        graph_colors: [
            Rgb::new(160, 154, 190),
            Rgb::new(111, 137, 78),
            Rgb::new(192, 71, 71),
            Rgb::new(119, 113, 63),
            Rgb::new(77, 105, 155),
            Rgb::new(100, 140, 130),
            Rgb::new(180, 100, 120),
            Rgb::new(140, 120, 170),
        ],
    }
}

pub fn moonfly() -> AppTheme {
    AppTheme {
        is_dark: true,
        background: Rgb::new(8, 8, 8),
        surface: Rgb::new(28, 28, 28),
        border: Rgb::new(78, 78, 78),
        selection: Rgb::new(28, 28, 28),
        text_primary: Rgb::new(178, 178, 178),
        text_secondary: Rgb::new(128, 160, 255),
        text_muted: Rgb::new(78, 78, 78),
        accent: Rgb::new(174, 129, 255),
        success: Rgb::new(140, 200, 95),
        warning: Rgb::new(226, 164, 120),
        error: Rgb::new(255, 115, 131),
        diff_add: Rgb::new(140, 200, 95),
        diff_del: Rgb::new(255, 115, 131),
        diff_context: Rgb::new(78, 78, 78),
        diff_hunk: Rgb::new(128, 160, 255),
        graph_colors: [
            Rgb::new(174, 129, 255),
            Rgb::new(140, 200, 95),
            Rgb::new(255, 115, 131),
            Rgb::new(226, 164, 120),
            Rgb::new(128, 160, 255),
            Rgb::new(116, 180, 187),
            Rgb::new(255, 192, 120),
            Rgb::new(200, 140, 255),
        ],
    }
}

pub fn nightfly() -> AppTheme {
    AppTheme {
        is_dark: true,
        background: Rgb::new(1, 22, 38),
        surface: Rgb::new(11, 41, 66),
        border: Rgb::new(75, 100, 121),
        selection: Rgb::new(11, 41, 66),
        text_primary: Rgb::new(172, 187, 203),
        text_secondary: Rgb::new(130, 170, 255),
        text_muted: Rgb::new(75, 100, 121),
        accent: Rgb::new(199, 146, 234),
        success: Rgb::new(161, 205, 94),
        warning: Rgb::new(243, 218, 11),
        error: Rgb::new(252, 87, 73),
        diff_add: Rgb::new(161, 205, 94),
        diff_del: Rgb::new(252, 87, 73),
        diff_context: Rgb::new(75, 100, 121),
        diff_hunk: Rgb::new(130, 170, 255),
        graph_colors: [
            Rgb::new(199, 146, 234),
            Rgb::new(161, 205, 94),
            Rgb::new(252, 87, 73),
            Rgb::new(243, 218, 11),
            Rgb::new(130, 170, 255),
            Rgb::new(33, 200, 215),
            Rgb::new(238, 130, 98),
            Rgb::new(174, 200, 255),
        ],
    }
}

pub fn oxocarbon() -> AppTheme {
    AppTheme {
        is_dark: true,
        background: Rgb::new(22, 22, 22),
        surface: Rgb::new(38, 38, 38),
        border: Rgb::new(82, 82, 82),
        selection: Rgb::new(38, 38, 38),
        text_primary: Rgb::new(242, 244, 248),
        text_secondary: Rgb::new(120, 169, 255),
        text_muted: Rgb::new(82, 82, 82),
        accent: Rgb::new(255, 126, 182),
        success: Rgb::new(66, 190, 101),
        warning: Rgb::new(250, 204, 55),
        error: Rgb::new(255, 97, 101),
        diff_add: Rgb::new(66, 190, 101),
        diff_del: Rgb::new(255, 97, 101),
        diff_context: Rgb::new(82, 82, 82),
        diff_hunk: Rgb::new(120, 169, 255),
        graph_colors: [
            Rgb::new(255, 126, 182),
            Rgb::new(66, 190, 101),
            Rgb::new(255, 97, 101),
            Rgb::new(250, 204, 55),
            Rgb::new(120, 169, 255),
            Rgb::new(8, 189, 186),
            Rgb::new(190, 149, 255),
            Rgb::new(255, 170, 120),
        ],
    }
}

pub fn cyberpunk() -> AppTheme {
    AppTheme {
        is_dark: true,
        background: Rgb::new(10, 10, 16),
        surface: Rgb::new(20, 20, 30),
        border: Rgb::new(45, 45, 55),
        selection: Rgb::new(50, 48, 20),
        text_primary: Rgb::new(230, 230, 220),
        text_secondary: Rgb::new(0, 210, 235),
        text_muted: Rgb::new(90, 90, 100),
        accent: Rgb::new(252, 238, 10),
        success: Rgb::new(0, 220, 180),
        warning: Rgb::new(255, 150, 0),
        error: Rgb::new(255, 50, 70),
        diff_add: Rgb::new(0, 220, 180),
        diff_del: Rgb::new(255, 50, 70),
        diff_context: Rgb::new(90, 90, 100),
        diff_hunk: Rgb::new(0, 210, 235),
        graph_colors: [
            Rgb::new(252, 238, 10),
            Rgb::new(0, 220, 180),
            Rgb::new(255, 50, 70),
            Rgb::new(255, 150, 0),
            Rgb::new(0, 210, 235),
            Rgb::new(180, 0, 255),
            Rgb::new(255, 0, 150),
            Rgb::new(0, 255, 100),
        ],
    }
}

pub fn rose_pine() -> AppTheme {
    AppTheme {
        is_dark: true,
        background: Rgb::new(25, 23, 36),
        surface: Rgb::new(38, 35, 55),
        border: Rgb::new(110, 106, 134),
        selection: Rgb::new(64, 61, 82),
        text_primary: Rgb::new(224, 222, 244),
        text_secondary: Rgb::new(156, 207, 216),
        text_muted: Rgb::new(110, 106, 134),
        accent: Rgb::new(196, 167, 231),
        success: Rgb::new(49, 116, 143),
        warning: Rgb::new(246, 193, 119),
        error: Rgb::new(235, 111, 146),
        diff_add: Rgb::new(49, 116, 143),
        diff_del: Rgb::new(235, 111, 146),
        diff_context: Rgb::new(110, 106, 134),
        diff_hunk: Rgb::new(156, 207, 216),
        graph_colors: [
            Rgb::new(196, 167, 231),
            Rgb::new(49, 116, 143),
            Rgb::new(235, 111, 146),
            Rgb::new(246, 193, 119),
            Rgb::new(156, 207, 216),
            Rgb::new(234, 154, 151),
            Rgb::new(156, 207, 216),
            Rgb::new(224, 222, 244),
        ],
    }
}

pub fn rose_pine_moon() -> AppTheme {
    AppTheme {
        is_dark: true,
        background: Rgb::new(35, 33, 54),
        surface: Rgb::new(48, 46, 70),
        border: Rgb::new(110, 106, 134),
        selection: Rgb::new(68, 65, 90),
        text_primary: Rgb::new(224, 222, 244),
        text_secondary: Rgb::new(156, 207, 216),
        text_muted: Rgb::new(110, 106, 134),
        accent: Rgb::new(196, 167, 231),
        success: Rgb::new(62, 143, 176),
        warning: Rgb::new(246, 193, 119),
        error: Rgb::new(235, 111, 146),
        diff_add: Rgb::new(62, 143, 176),
        diff_del: Rgb::new(235, 111, 146),
        diff_context: Rgb::new(110, 106, 134),
        diff_hunk: Rgb::new(156, 207, 216),
        graph_colors: [
            Rgb::new(196, 167, 231),
            Rgb::new(62, 143, 176),
            Rgb::new(235, 111, 146),
            Rgb::new(246, 193, 119),
            Rgb::new(156, 207, 216),
            Rgb::new(234, 154, 151),
            Rgb::new(156, 207, 216),
            Rgb::new(224, 222, 244),
        ],
    }
}

pub fn rose_pine_dawn() -> AppTheme {
    AppTheme {
        is_dark: false,
        background: Rgb::new(250, 244, 237),
        surface: Rgb::new(242, 233, 221),
        border: Rgb::new(152, 147, 165),
        selection: Rgb::new(223, 218, 217),
        text_primary: Rgb::new(87, 82, 121),
        text_secondary: Rgb::new(86, 148, 159),
        text_muted: Rgb::new(152, 147, 165),
        accent: Rgb::new(144, 122, 169),
        success: Rgb::new(40, 105, 131),
        warning: Rgb::new(234, 157, 52),
        error: Rgb::new(180, 99, 122),
        diff_add: Rgb::new(40, 105, 131),
        diff_del: Rgb::new(180, 99, 122),
        diff_context: Rgb::new(152, 147, 165),
        diff_hunk: Rgb::new(86, 148, 159),
        graph_colors: [
            Rgb::new(144, 122, 169),
            Rgb::new(40, 105, 131),
            Rgb::new(180, 99, 122),
            Rgb::new(234, 157, 52),
            Rgb::new(86, 148, 159),
            Rgb::new(215, 130, 126),
            Rgb::new(87, 82, 121),
            Rgb::new(152, 147, 165),
        ],
    }
}

pub fn ayu_mirage() -> AppTheme {
    AppTheme {
        is_dark: true,
        background: Rgb::new(31, 36, 48),
        surface: Rgb::new(42, 48, 62),
        border: Rgb::new(104, 104, 104),
        selection: Rgb::new(64, 159, 255),
        text_primary: Rgb::new(204, 202, 194),
        text_secondary: Rgb::new(115, 208, 255),
        text_muted: Rgb::new(104, 104, 104),
        accent: Rgb::new(115, 208, 255),
        success: Rgb::new(135, 217, 108),
        warning: Rgb::new(250, 204, 110),
        error: Rgb::new(237, 130, 116),
        diff_add: Rgb::new(135, 217, 108),
        diff_del: Rgb::new(237, 130, 116),
        diff_context: Rgb::new(104, 104, 104),
        diff_hunk: Rgb::new(115, 208, 255),
        graph_colors: [
            Rgb::new(115, 208, 255),
            Rgb::new(135, 217, 108),
            Rgb::new(237, 130, 116),
            Rgb::new(250, 204, 110),
            Rgb::new(64, 159, 255),
            Rgb::new(217, 191, 255),
            Rgb::new(255, 170, 108),
            Rgb::new(204, 202, 194),
        ],
    }
}

pub fn everforest_dark() -> AppTheme {
    AppTheme {
        is_dark: true,
        background: Rgb::new(30, 35, 38),
        surface: Rgb::new(42, 48, 50),
        border: Rgb::new(166, 176, 160),
        selection: Rgb::new(76, 55, 67),
        text_primary: Rgb::new(211, 198, 170),
        text_secondary: Rgb::new(127, 187, 179),
        text_muted: Rgb::new(122, 132, 120),
        accent: Rgb::new(167, 192, 128),
        success: Rgb::new(167, 192, 128),
        warning: Rgb::new(219, 188, 127),
        error: Rgb::new(230, 126, 128),
        diff_add: Rgb::new(167, 192, 128),
        diff_del: Rgb::new(230, 126, 128),
        diff_context: Rgb::new(122, 132, 120),
        diff_hunk: Rgb::new(127, 187, 179),
        graph_colors: [
            Rgb::new(167, 192, 128),
            Rgb::new(127, 187, 179),
            Rgb::new(230, 126, 128),
            Rgb::new(219, 188, 127),
            Rgb::new(214, 153, 182),
            Rgb::new(211, 198, 170),
            Rgb::new(166, 176, 160),
            Rgb::new(122, 132, 120),
        ],
    }
}

pub fn atom_one_dark() -> AppTheme {
    AppTheme {
        is_dark: true,
        background: Rgb::new(33, 37, 43),
        surface: Rgb::new(45, 50, 58),
        border: Rgb::new(118, 118, 118),
        selection: Rgb::new(50, 56, 68),
        text_primary: Rgb::new(171, 178, 191),
        text_secondary: Rgb::new(97, 175, 239),
        text_muted: Rgb::new(118, 118, 118),
        accent: Rgb::new(97, 175, 239),
        success: Rgb::new(152, 195, 121),
        warning: Rgb::new(229, 192, 123),
        error: Rgb::new(224, 108, 117),
        diff_add: Rgb::new(152, 195, 121),
        diff_del: Rgb::new(224, 108, 117),
        diff_context: Rgb::new(118, 118, 118),
        diff_hunk: Rgb::new(97, 175, 239),
        graph_colors: [
            Rgb::new(97, 175, 239),
            Rgb::new(152, 195, 121),
            Rgb::new(224, 108, 117),
            Rgb::new(229, 192, 123),
            Rgb::new(198, 120, 221),
            Rgb::new(86, 182, 194),
            Rgb::new(171, 178, 191),
            Rgb::new(118, 118, 118),
        ],
    }
}

pub fn atom_one_light() -> AppTheme {
    AppTheme {
        is_dark: false,
        background: Rgb::new(249, 249, 249),
        surface: Rgb::new(237, 237, 237),
        border: Rgb::new(180, 180, 180),
        selection: Rgb::new(237, 237, 237),
        text_primary: Rgb::new(42, 44, 51),
        text_secondary: Rgb::new(47, 90, 243),
        text_muted: Rgb::new(118, 118, 118),
        accent: Rgb::new(47, 90, 243),
        success: Rgb::new(63, 149, 58),
        warning: Rgb::new(210, 182, 124),
        error: Rgb::new(222, 62, 53),
        diff_add: Rgb::new(63, 149, 58),
        diff_del: Rgb::new(222, 62, 53),
        diff_context: Rgb::new(118, 118, 118),
        diff_hunk: Rgb::new(47, 90, 243),
        graph_colors: [
            Rgb::new(47, 90, 243),
            Rgb::new(63, 149, 58),
            Rgb::new(222, 62, 53),
            Rgb::new(210, 182, 124),
            Rgb::new(166, 38, 164),
            Rgb::new(1, 132, 188),
            Rgb::new(42, 44, 51),
            Rgb::new(180, 180, 180),
        ],
    }
}

pub fn night_owl() -> AppTheme {
    AppTheme {
        is_dark: true,
        background: Rgb::new(1, 22, 39),
        surface: Rgb::new(12, 35, 55),
        border: Rgb::new(87, 86, 86),
        selection: Rgb::new(95, 126, 151),
        text_primary: Rgb::new(214, 222, 235),
        text_secondary: Rgb::new(130, 170, 255),
        text_muted: Rgb::new(87, 86, 86),
        accent: Rgb::new(130, 170, 255),
        success: Rgb::new(34, 218, 110),
        warning: Rgb::new(173, 219, 103),
        error: Rgb::new(239, 83, 80),
        diff_add: Rgb::new(34, 218, 110),
        diff_del: Rgb::new(239, 83, 80),
        diff_context: Rgb::new(87, 86, 86),
        diff_hunk: Rgb::new(130, 170, 255),
        graph_colors: [
            Rgb::new(130, 170, 255),
            Rgb::new(34, 218, 110),
            Rgb::new(239, 83, 80),
            Rgb::new(173, 219, 103),
            Rgb::new(199, 146, 234),
            Rgb::new(127, 219, 202),
            Rgb::new(214, 222, 235),
            Rgb::new(255, 203, 139),
        ],
    }
}

pub fn poimandres() -> AppTheme {
    AppTheme {
        is_dark: true,
        background: Rgb::new(26, 30, 40),
        surface: Rgb::new(38, 42, 55),
        border: Rgb::new(100, 106, 130),
        selection: Rgb::new(50, 55, 75),
        text_primary: Rgb::new(166, 172, 205),
        text_secondary: Rgb::new(137, 221, 255),
        text_muted: Rgb::new(100, 106, 130),
        accent: Rgb::new(93, 228, 199),
        success: Rgb::new(93, 228, 199),
        warning: Rgb::new(255, 250, 194),
        error: Rgb::new(208, 103, 157),
        diff_add: Rgb::new(93, 228, 199),
        diff_del: Rgb::new(208, 103, 157),
        diff_context: Rgb::new(100, 106, 130),
        diff_hunk: Rgb::new(137, 221, 255),
        graph_colors: [
            Rgb::new(93, 228, 199),
            Rgb::new(137, 221, 255),
            Rgb::new(208, 103, 157),
            Rgb::new(255, 250, 194),
            Rgb::new(166, 172, 205),
            Rgb::new(173, 219, 103),
            Rgb::new(100, 106, 130),
            Rgb::new(255, 180, 120),
        ],
    }
}

pub fn flexoki_dark() -> AppTheme {
    AppTheme {
        is_dark: true,
        background: Rgb::new(16, 15, 15),
        surface: Rgb::new(30, 28, 28),
        border: Rgb::new(87, 86, 83),
        selection: Rgb::new(64, 62, 60),
        text_primary: Rgb::new(206, 205, 195),
        text_secondary: Rgb::new(67, 133, 190),
        text_muted: Rgb::new(87, 86, 83),
        accent: Rgb::new(67, 133, 190),
        success: Rgb::new(135, 154, 57),
        warning: Rgb::new(208, 162, 21),
        error: Rgb::new(209, 77, 65),
        diff_add: Rgb::new(135, 154, 57),
        diff_del: Rgb::new(209, 77, 65),
        diff_context: Rgb::new(87, 86, 83),
        diff_hunk: Rgb::new(67, 133, 190),
        graph_colors: [
            Rgb::new(67, 133, 190),
            Rgb::new(135, 154, 57),
            Rgb::new(209, 77, 65),
            Rgb::new(208, 162, 21),
            Rgb::new(206, 93, 151),
            Rgb::new(58, 169, 159),
            Rgb::new(206, 205, 195),
            Rgb::new(171, 141, 72),
        ],
    }
}

pub fn flexoki_light() -> AppTheme {
    AppTheme {
        is_dark: false,
        background: Rgb::new(255, 252, 240),
        surface: Rgb::new(242, 238, 222),
        border: Rgb::new(183, 181, 172),
        selection: Rgb::new(206, 205, 195),
        text_primary: Rgb::new(16, 15, 15),
        text_secondary: Rgb::new(32, 94, 166),
        text_muted: Rgb::new(111, 110, 105),
        accent: Rgb::new(32, 94, 166),
        success: Rgb::new(102, 128, 11),
        warning: Rgb::new(173, 131, 1),
        error: Rgb::new(175, 48, 41),
        diff_add: Rgb::new(102, 128, 11),
        diff_del: Rgb::new(175, 48, 41),
        diff_context: Rgb::new(111, 110, 105),
        diff_hunk: Rgb::new(32, 94, 166),
        graph_colors: [
            Rgb::new(32, 94, 166),
            Rgb::new(102, 128, 11),
            Rgb::new(175, 48, 41),
            Rgb::new(173, 131, 1),
            Rgb::new(163, 64, 119),
            Rgb::new(36, 131, 123),
            Rgb::new(16, 15, 15),
            Rgb::new(133, 104, 46),
        ],
    }
}

pub fn carbonfox() -> AppTheme {
    AppTheme {
        is_dark: true,
        background: Rgb::new(22, 22, 22),
        surface: Rgb::new(35, 35, 35),
        border: Rgb::new(72, 72, 72),
        selection: Rgb::new(42, 42, 42),
        text_primary: Rgb::new(242, 244, 248),
        text_secondary: Rgb::new(120, 169, 255),
        text_muted: Rgb::new(100, 100, 110),
        accent: Rgb::new(120, 169, 255),
        success: Rgb::new(37, 190, 106),
        warning: Rgb::new(8, 189, 186),
        error: Rgb::new(238, 83, 150),
        diff_add: Rgb::new(37, 190, 106),
        diff_del: Rgb::new(238, 83, 150),
        diff_context: Rgb::new(100, 100, 110),
        diff_hunk: Rgb::new(120, 169, 255),
        graph_colors: [
            Rgb::new(120, 169, 255),
            Rgb::new(37, 190, 106),
            Rgb::new(238, 83, 150),
            Rgb::new(8, 189, 186),
            Rgb::new(190, 149, 255),
            Rgb::new(255, 126, 182),
            Rgb::new(242, 244, 248),
            Rgb::new(255, 170, 120),
        ],
    }
}

pub fn andromeda() -> AppTheme {
    AppTheme {
        is_dark: true,
        background: Rgb::new(38, 42, 51),
        surface: Rgb::new(50, 55, 66),
        border: Rgb::new(102, 102, 102),
        selection: Rgb::new(90, 92, 98),
        text_primary: Rgb::new(229, 229, 229),
        text_secondary: Rgb::new(15, 168, 205),
        text_muted: Rgb::new(102, 102, 102),
        accent: Rgb::new(5, 188, 121),
        success: Rgb::new(5, 188, 121),
        warning: Rgb::new(229, 229, 18),
        error: Rgb::new(205, 49, 49),
        diff_add: Rgb::new(5, 188, 121),
        diff_del: Rgb::new(205, 49, 49),
        diff_context: Rgb::new(102, 102, 102),
        diff_hunk: Rgb::new(15, 168, 205),
        graph_colors: [
            Rgb::new(5, 188, 121),
            Rgb::new(15, 168, 205),
            Rgb::new(205, 49, 49),
            Rgb::new(229, 229, 18),
            Rgb::new(190, 149, 255),
            Rgb::new(255, 126, 182),
            Rgb::new(229, 229, 229),
            Rgb::new(102, 102, 102),
        ],
    }
}

pub fn synthwave() -> AppTheme {
    AppTheme {
        is_dark: true,
        background: Rgb::new(10, 8, 16),
        surface: Rgb::new(20, 16, 30),
        border: Rgb::new(127, 112, 148),
        selection: Rgb::new(25, 50, 60),
        text_primary: Rgb::new(218, 217, 199),
        text_secondary: Rgb::new(18, 195, 226),
        text_muted: Rgb::new(127, 112, 148),
        accent: Rgb::new(246, 24, 143),
        success: Rgb::new(30, 187, 43),
        warning: Rgb::new(253, 248, 52),
        error: Rgb::new(246, 24, 143),
        diff_add: Rgb::new(30, 187, 43),
        diff_del: Rgb::new(246, 24, 143),
        diff_context: Rgb::new(127, 112, 148),
        diff_hunk: Rgb::new(18, 195, 226),
        graph_colors: [
            Rgb::new(246, 24, 143),
            Rgb::new(30, 187, 43),
            Rgb::new(18, 195, 226),
            Rgb::new(253, 248, 52),
            Rgb::new(190, 149, 255),
            Rgb::new(255, 126, 182),
            Rgb::new(218, 217, 199),
            Rgb::new(127, 112, 148),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_count_matches_names() {
        assert_eq!(THEME_NAMES.len(), THEME_COUNT);
    }

    #[test]
    fn all_themes_resolve() {
        for i in 0..THEME_COUNT {
            let t = theme_by_index(i);
            let has_colour = t.text_primary.r > 0 || t.text_primary.g > 0 || t.text_primary.b > 0;
            assert!(has_colour, "theme index {i} has zero text_primary");
        }
    }

    #[test]
    fn out_of_range_returns_default() {
        let d = theme_by_index(0);
        let oob = theme_by_index(999);
        assert_eq!(d.background.r, oob.background.r);
        assert_eq!(d.background.g, oob.background.g);
        assert_eq!(d.background.b, oob.background.b);
    }

    #[test]
    fn theme_index_by_name_known() {
        assert_eq!(theme_index_by_name("Default"), 0);
        assert_eq!(theme_index_by_name("Cyberpunk"), 28);
        assert_eq!(theme_index_by_name("Synthwave"), 42);
    }

    #[test]
    fn theme_index_by_name_unknown_returns_zero() {
        assert_eq!(theme_index_by_name("NonExistent"), 0);
    }

    #[test]
    fn light_themes_are_light() {
        assert!(!default_light().is_dark);
        assert!(!solarized_light().is_dark);
        assert!(!gruvbox_light().is_dark);
        assert!(!catppuccin_latte().is_dark);
        assert!(!tokyo_night_light().is_dark);
        assert!(!kanagawa_lotus().is_dark);
        assert!(!rose_pine_dawn().is_dark);
        assert!(!atom_one_light().is_dark);
        assert!(!flexoki_light().is_dark);
    }

    #[test]
    fn cyberpunk_accent_is_yellow() {
        let t = cyberpunk();
        assert!(t.accent.r > 240 && t.accent.g > 220 && t.accent.b < 30);
    }
}
