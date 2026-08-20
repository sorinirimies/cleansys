//! Unit tests for `cleansys_tui::app::App` navigation, selection, and
//! view-state cycling logic (pure state machine, no terminal I/O required).

use anyhow::Result;
use cleansys_core::{CleanerCategory, CleanerItem};
use cleansys_tui::app::{App, ChartType, FilterMode, SortMode, ViewMode};

fn noop(_dry_run: bool) -> Result<u64> {
    Ok(0)
}

fn sample_item(name: &str, requires_root: bool) -> CleanerItem {
    CleanerItem {
        name: name.to_string(),
        description: format!("{name} description"),
        requires_root,
        selected: false,
        function: noop,
        bytes_cleaned: 0,
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
                sample_item("Temp Files", false),
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

#[test]
fn next_item_wraps_around() {
    let mut app = app_with_categories();
    assert_eq!(app.item_list_state.selected(), Some(0));

    app.next_item();
    assert_eq!(app.item_list_state.selected(), Some(1));
    app.next_item();
    assert_eq!(app.item_list_state.selected(), Some(2));
    // Wraps back to the first item.
    app.next_item();
    assert_eq!(app.item_list_state.selected(), Some(0));
}

#[test]
fn previous_item_wraps_around() {
    let mut app = app_with_categories();
    app.item_list_state.select(Some(0));

    // Wraps to the last item.
    app.previous_item();
    assert_eq!(app.item_list_state.selected(), Some(2));
    app.previous_item();
    assert_eq!(app.item_list_state.selected(), Some(1));
}

#[test]
fn next_and_previous_category_wrap_and_reset_selection() {
    let mut app = app_with_categories();
    app.item_list_state.select(Some(2));

    app.next_category();
    assert_eq!(app.category_index, 1);
    // Selection resets to the first item of the new category.
    assert_eq!(app.item_list_state.selected(), Some(0));

    // Wraps back to category 0.
    app.next_category();
    assert_eq!(app.category_index, 0);

    app.previous_category();
    assert_eq!(app.category_index, 1);
}

#[test]
fn toggle_selected_flips_current_item() {
    let mut app = app_with_categories();
    app.item_list_state.select(Some(0));
    assert!(!app.categories[0].items[0].selected);

    app.toggle_selected();
    assert!(app.categories[0].items[0].selected);

    app.toggle_selected();
    assert!(!app.categories[0].items[0].selected);
}

#[test]
fn select_all_and_deselect_all_affect_current_category_only() {
    let mut app = app_with_categories();
    app.select_all();
    assert!(app.categories[0].items.iter().all(|i| i.selected));
    assert!(app.categories[1].items.iter().all(|i| !i.selected));

    app.deselect_all();
    assert!(app.categories[0].items.iter().all(|i| !i.selected));
}

#[test]
fn cycle_view_mode_goes_through_all_variants() {
    let mut app = app_with_categories();
    app.view_mode = ViewMode::Standard;

    app.cycle_view_mode();
    assert_eq!(app.view_mode, ViewMode::Compact);
    app.cycle_view_mode();
    assert_eq!(app.view_mode, ViewMode::Detailed);
    app.cycle_view_mode();
    assert_eq!(app.view_mode, ViewMode::Performance);
    app.cycle_view_mode();
    assert_eq!(app.view_mode, ViewMode::Standard);
}

#[test]
fn cycle_sort_mode_goes_through_all_variants() {
    let mut app = app_with_categories();
    app.sort_mode = SortMode::Name;

    app.cycle_sort_mode();
    assert_eq!(app.sort_mode, SortMode::Size);
    app.cycle_sort_mode();
    assert_eq!(app.sort_mode, SortMode::Status);
    app.cycle_sort_mode();
    assert_eq!(app.sort_mode, SortMode::Category);
    app.cycle_sort_mode();
    assert_eq!(app.sort_mode, SortMode::Name);
}

#[test]
fn cycle_filter_mode_goes_through_all_variants() {
    let mut app = app_with_categories();
    app.filter_mode = FilterMode::All;

    app.cycle_filter_mode();
    assert_eq!(app.filter_mode, FilterMode::Selected);
    app.cycle_filter_mode();
    assert_eq!(app.filter_mode, FilterMode::Completed);
    app.cycle_filter_mode();
    assert_eq!(app.filter_mode, FilterMode::Errors);
    app.cycle_filter_mode();
    assert_eq!(app.filter_mode, FilterMode::UserOnly);
    app.cycle_filter_mode();
    assert_eq!(app.filter_mode, FilterMode::SystemOnly);
    app.cycle_filter_mode();
    assert_eq!(app.filter_mode, FilterMode::All);
}

#[test]
fn toggle_chart_type_cycles_bar_pie_count_pie_size() {
    let mut app = app_with_categories();
    app.chart_type = ChartType::Bar;

    app.toggle_chart_type();
    assert_eq!(app.chart_type, ChartType::PieCount);
    app.toggle_chart_type();
    assert_eq!(app.chart_type, ChartType::PieSize);
    app.toggle_chart_type();
    assert_eq!(app.chart_type, ChartType::Bar);
}

#[test]
fn toggle_help_and_compact_mode() {
    let mut app = app_with_categories();
    assert!(!app.show_help);
    app.toggle_help();
    assert!(app.show_help);
    app.toggle_help();
    assert!(!app.show_help);

    let was_compact = app.compact_mode;
    app.toggle_compact_mode();
    assert_eq!(app.compact_mode, !was_compact);
}

#[test]
fn search_toggle_and_input() {
    let mut app = app_with_categories();
    assert!(!app.search_active);

    app.toggle_search();
    assert!(app.search_active);

    app.add_search_char('a');
    app.add_search_char('b');
    assert_eq!(app.search_query, "ab");

    app.remove_search_char();
    assert_eq!(app.search_query, "a");

    app.clear_search();
    assert!(!app.search_active);
    assert!(app.search_query.is_empty());
}

#[test]
fn get_category_distribution_groups_by_cleaner_name() {
    // App::new() seeds sample cleaned items via add_sample_cleaned_items().
    let app = App::new();
    let distribution = app.get_category_distribution();
    assert!(!distribution.is_empty());
    // Sorted by total size descending.
    for pair in distribution.windows(2) {
        assert!(pair[0].2 >= pair[1].2);
    }
}

#[test]
fn get_filtered_detailed_items_respects_search_query() {
    let mut app = App::new();
    app.search_query = "firefox".to_string();
    let filtered = app.get_filtered_detailed_items();
    assert!(!filtered.is_empty());
    assert!(filtered
        .iter()
        .all(|item| item.path.to_lowercase().contains("firefox")
            || item.category.to_lowercase().contains("firefox")
            || item.cleaner_name.to_lowercase().contains("firefox")));
}

#[test]
fn get_filtered_detailed_items_sort_by_size_is_descending() {
    let mut app = App::new();
    app.sort_mode = SortMode::Size;
    let filtered = app.get_filtered_detailed_items();
    for pair in filtered.windows(2) {
        assert!(pair[0].size >= pair[1].size);
    }
}

#[test]
fn clear_errors_resets_error_status_and_counter() {
    let mut app = app_with_categories();
    app.categories[0].items[0].status = Some(cleansys_core::Status::Error("boom".to_string()));
    app.errors_count = 1;

    app.clear_errors();

    assert!(app.categories[0].items[0].status.is_none());
    assert_eq!(app.errors_count, 0);
}

#[test]
fn update_counters_counts_selected_errors_and_operations() {
    let mut app = app_with_categories();
    app.categories[0].items[0].selected = true;
    app.categories[0].items[1].status = Some(cleansys_core::Status::Error("x".to_string()));
    app.categories[0].items[2].status = Some(cleansys_core::Status::Success("ok".to_string()));

    app.update_counters();

    assert_eq!(app.selected_cleaners_count, 1);
    assert_eq!(app.errors_count, 1);
    assert_eq!(app.operation_count, 2);
}
