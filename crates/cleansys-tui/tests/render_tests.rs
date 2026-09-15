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
        name: name.to_string(),
        description: format!("{name} description"),
        requires_root,
        selected: false,
        function: noop,
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
