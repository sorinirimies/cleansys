//! Smoke tests for `cleansys_tui::render::ui` — verifies every overlay/state
//! combination renders without panicking, using ratatui's `TestBackend`
//! (renders into an in-memory buffer, no real terminal needed).
//!
//! Mirrors the equivalent `view_does_not_panic_for_*` suite in
//! `cleansys-gui/src/view.rs`; `render.rs` previously had zero test coverage
//! at all despite being the TUI's entire rendering layer (1600+ lines).

use anyhow::Result;
use cleansys_core::{CleanedItemType, CleanerCategory, CleanerItem, CleaningResult, RunOptions};
use cleansys_tui::app::App;
use cleansys_tui::render::ui;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

fn noop(_opts: RunOptions) -> Result<CleaningResult> {
    Ok(CleaningResult::new())
}

fn sample_item(name: &str, requires_root: bool) -> CleanerItem {
    CleanerItem {
        id: name.to_string(),
        risk: cleansys_core::Risk::Safe,
        name: name.to_string(),
        description: format!("{name} description"),
        requires_root,
        selected: false,
        function: std::sync::Arc::new(noop),
        bytes_cleaned: 0,
        last_result: None,
        status: None,
    }
}

fn app_with_categories() -> App {
    let mut app = App::new();
    app.categories = vec![
        CleanerCategory {
            name: "User".to_string(),
            description: "User cleaners".to_string(),
            items: vec![
                sample_item("Browser Caches", false),
                sample_item("Trash", false),
            ],
        },
        CleanerCategory {
            name: "System".to_string(),
            description: "System cleaners".to_string(),
            items: vec![
                sample_item("Package Cache", true),
                sample_item("Logs", true),
            ],
        },
    ];
    app.category_index = 0;
    app.item_list_state.select(Some(0));
    app
}

/// Render one frame with a fixed-size in-memory backend. Panics (which
/// `#[test]` turns into a failure) are the thing we're guarding against;
/// the rendered content itself isn't asserted on.
fn render_once(app: &mut App) {
    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).expect("TestBackend terminal should construct");
    terminal.draw(|f| ui(f, app)).expect("draw must not error");
}

#[test]
fn renders_default_state_without_panic() {
    let mut app = app_with_categories();
    render_once(&mut app);
}

#[test]
fn renders_with_selection_and_search_active() {
    let mut app = app_with_categories();
    app.categories[0].items[0].selected = true;
    app.search_active = true;
    app.search_query = "cache".to_string();
    render_once(&mut app);
}

#[test]
fn renders_help_overlay() {
    let mut app = app_with_categories();
    app.show_help = true;
    render_once(&mut app);
}

#[test]
fn renders_password_prompt() {
    let mut app = app_with_categories();
    app.password_prompt.show();
    app.password_prompt.add_char('h');
    app.password_prompt.add_char('i');
    render_once(&mut app);
}

#[test]
fn renders_admin_notice_overlay() {
    let mut app = app_with_categories();
    app.needs_admin_notice = true;
    render_once(&mut app);
}

#[test]
fn renders_run_confirmation_overlay() {
    let mut app = app_with_categories();
    app.categories[0].items[0].selected = true;
    app.request_run().unwrap();
    assert!(app.awaiting_run_confirmation);
    render_once(&mut app);
}

#[test]
fn renders_preview_overlay_with_results() {
    let mut app = app_with_categories();
    app.categories[0].items[0].selected = true;
    app.run_preview();
    assert!(app.preview_open);
    render_once(&mut app);
}

#[test]
fn renders_progress_screen_while_running() {
    let mut app = app_with_categories();
    app.confirmation_mode = false;
    app.categories[0].items[0].selected = true;
    app.request_run().unwrap();
    assert!(app.is_running);
    render_once(&mut app);
}

#[test]
fn renders_progress_screen_with_detailed_items_and_errors() {
    let mut app = app_with_categories();
    app.show_progress_screen = true;
    app.categories[0].items[0].status = Some(cleansys_core::Status::Success("ok".to_string()));
    app.categories[0].items[1].status = Some(cleansys_core::Status::Error("boom".to_string()));
    app.categories[1].items[0].status = Some(cleansys_core::Status::Running);
    for i in 0..5 {
        app.add_detailed_cleaned_item(
            format!("/tmp/file-{i}"),
            1024 * (i as u64 + 1),
            "User".to_string(),
            "Browser Caches".to_string(),
            CleanedItemType::File.into(),
        );
    }
    render_once(&mut app);
}

#[test]
fn renders_every_view_mode_without_panic() {
    use cleansys_tui::app::ViewMode;
    for mode in [
        ViewMode::Standard,
        ViewMode::Compact,
        ViewMode::Detailed,
        ViewMode::Performance,
    ] {
        let mut app = app_with_categories();
        app.view_mode = mode;
        render_once(&mut app);
    }
}

#[test]
fn renders_every_chart_type_without_panic() {
    use cleansys_tui::app::ChartType;
    for chart in [ChartType::Bar, ChartType::PieCount, ChartType::PieSize] {
        let mut app = app_with_categories();
        app.chart_type = chart;
        app.show_progress_screen = true;
        for i in 0..3 {
            app.add_detailed_cleaned_item(
                format!("/tmp/file-{i}"),
                2048,
                "User".to_string(),
                "Browser Caches".to_string(),
                CleanedItemType::File.into(),
            );
        }
        render_once(&mut app);
    }
}

#[test]
fn renders_at_small_terminal_size() {
    let mut app = app_with_categories();
    app.handle_resize(60, 18);
    app.compact_mode = true;
    let backend = TestBackend::new(60, 18);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| ui(f, &mut app)).unwrap();
}

#[test]
fn renders_completed_run_summary() {
    let mut app = app_with_categories();
    app.confirmation_mode = false;
    app.categories[0].items[0].selected = true;
    app.request_run().unwrap();
    // Force-complete: mark the only Pending item Success and let
    // update_counters() notice everything finished.
    app.categories[0].items[0].status = Some(cleansys_core::Status::Success("done".to_string()));
    app.update_counters();
    assert!(!app.is_running);
    render_once(&mut app);
}

/// Render and return the whole screen as one string.
fn screen_text(app: &mut App, width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).expect("create terminal");
    terminal.draw(|f| ui(f, app)).expect("draw");
    let buffer = terminal.backend().buffer().clone();
    buffer
        .content()
        .chunks(width as usize)
        .map(|row| row.iter().map(|c| c.symbol()).collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn totals_show_a_spinner_instead_of_numbers_while_the_scan_is_running() {
    use cleansys_core::{ScanBoard, ScanInfo};
    let mut app = app_with_categories();
    app.categories[0].items[0].selected = true;
    let mut board = ScanBoard::new(&app.categories);
    board.start(4);
    // one of four measured: a partial total would be misleading
    board.record(
        0,
        0,
        ScanInfo {
            bytes: 5 << 30,
            ..Default::default()
        },
    );
    app.board = board;

    for (w, h) in [(140, 40), (80, 40)] {
        let text = screen_text(&mut app, w, h);
        assert!(
            text.contains("scanning 1/4"),
            "{w}x{h}: scan progress visible"
        );
        if w >= 100 {
            // the compact footer of narrow terminals only has room for key hints
            assert!(text.contains("measuring"), "{w}x{h}: footer says measuring");
        }
        assert!(!text.contains("to free"), "{w}x{h}: no partial total");
        assert!(
            !text.contains("can be freed"),
            "{w}x{h}: no partial grand total"
        );
    }

    // once finished, the numbers appear
    for (c, i) in [(0, 1), (1, 0), (1, 1)] {
        app.board.record(
            c,
            i,
            ScanInfo {
                bytes: 1 << 20,
                ..Default::default()
            },
        );
    }
    let text = screen_text(&mut app, 140, 40);
    assert!(text.contains("to free") && text.contains("can be freed"));
    assert!(!text.contains("measuring"));
}

fn seed_entries(app: &mut App) {
    use cleansys_core::{ScanBoard, ScanEntry, ScanInfo};
    let e = |p: &str, bytes| ScanEntry {
        path: p.into(),
        bytes,
        label: "Rust build: demo".into(),
        skippable: true,
    };
    app.board = ScanBoard::new(&app.categories);
    app.board.start(1);
    app.board.record(
        0,
        0,
        ScanInfo {
            bytes: 300,
            items: 2,
            top_path: Some("/p/a/target".into()),
            entries: vec![e("/p/a/target", 200), e("/p/b/target", 100)],
            ..ScanInfo::default()
        },
    );
}

#[test]
fn expanded_details_render_paths_and_follow_the_cursor() {
    let mut app = app_with_categories();
    seed_entries(&mut app);
    app.expand_current();
    assert_eq!(app.entry_cursor, Some(0));

    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| ui(f, &mut app)).unwrap();
    let text: String = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|c| c.symbol())
        .collect();
    assert!(text.contains("/p/a/target"), "entry path not rendered");
    assert!(text.contains("/p/b/target"));

    // Moving the cursor and unticking an entry updates the board.
    app.entry_next();
    assert_eq!(app.entry_cursor, Some(1));
    app.entry_toggle();
    assert!(!app.board.entry_selected("/p/b/target"));
    assert!(app.board.entry_selected("/p/a/target"));
    terminal.draw(|f| ui(f, &mut app)).unwrap();

    app.collapse_details();
    assert!(app.expanded.is_none() && app.entry_cursor.is_none());
}
