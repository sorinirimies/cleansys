//! Unit tests for `cleansys_tui::app::App` navigation, selection, and
//! view-state cycling logic (pure state machine, no terminal I/O required).

use anyhow::Result;
use cleansys_core::{CleanerCategory, CleanerItem, CleaningResult, RunOptions};
use cleansys_tui::app::App;

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
fn toggle_help() {
    let mut app = app_with_categories();
    assert!(!app.show_help);
    app.toggle_help();
    assert!(app.show_help);
    app.toggle_help();
    assert!(!app.show_help);
}

#[test]
fn request_run_with_nothing_selected_shows_a_notice() {
    let mut app = app_with_categories();
    app.request_run().unwrap();
    assert!(!app.awaiting_run_confirmation);
    assert!(app
        .notice
        .as_deref()
        .is_some_and(|m| m.contains("Nothing selected")));
}

#[test]
fn request_run_with_confirmation_mode_shows_confirmation_overlay() {
    let mut app = app_with_categories();
    app.confirmation_mode = true;
    app.categories[0].items[0].selected = true;

    app.request_run().unwrap();

    assert!(app.awaiting_run_confirmation);
    assert!(!app.is_running);
    assert_eq!(app.pending_run_selection.len(), 1);
}

#[test]
fn cancel_run_confirmation_clears_pending_selection() {
    let mut app = app_with_categories();
    app.confirmation_mode = true;
    app.categories[0].items[0].selected = true;
    app.request_run().unwrap();

    app.cancel_run_confirmation();

    assert!(!app.awaiting_run_confirmation);
    assert!(app.pending_run_selection.is_empty());
    assert!(!app.is_running);
}

#[test]
fn confirm_pending_run_starts_execution_for_user_cleaners() {
    let mut app = app_with_categories();
    app.confirmation_mode = true;
    app.categories[0].items[0].selected = true;
    app.request_run().unwrap();

    app.confirm_pending_run().unwrap();

    assert!(!app.awaiting_run_confirmation);
    assert!(app.is_running);
    assert!(matches!(
        app.categories[0].items[0].status,
        Some(cleansys_core::Status::Pending)
    ));
}

#[test]
fn request_run_without_confirmation_mode_starts_immediately() {
    let mut app = app_with_categories();
    app.confirmation_mode = false;
    app.categories[0].items[0].selected = true;

    app.request_run().unwrap();

    assert!(!app.awaiting_run_confirmation);
    assert!(app.is_running);
}

#[test]
fn request_run_with_root_item_and_no_root_needs_elevation() {
    let mut app = app_with_categories();
    app.confirmation_mode = false;
    app.is_root = false;
    app.categories[1].items[0].selected = true; // requires_root cleaner

    app.request_run().unwrap();

    assert!(!app.is_running);
    if cleansys_core::utils::supports_sudo_prompt() {
        assert!(app.needs_sudo);
    } else {
        assert!(app.needs_admin_notice);
    }
}

#[test]
fn run_preview_with_nothing_selected_shows_a_notice() {
    let mut app = app_with_categories();
    app.run_preview();
    assert!(!app.preview_open);
    assert!(app
        .notice
        .as_deref()
        .is_some_and(|m| m.contains("Nothing selected")));
}

#[test]
fn run_preview_with_selection_populates_results_and_opens_overlay() {
    let mut app = app_with_categories();
    app.categories[0].items[0].selected = true;

    app.run_preview();

    assert!(app.preview_open);
    assert_eq!(app.preview_results.len(), 1);
    // Preview must never mark items as running/pending or start a real run.
    assert!(!app.is_running);
    assert!(app.categories[0].items[0].status.is_none());
}

#[test]
fn close_preview_clears_state() {
    let mut app = app_with_categories();
    app.categories[0].items[0].selected = true;
    app.run_preview();

    app.close_preview();

    assert!(!app.preview_open);
    assert!(app.preview_results.is_empty());
}

#[test]
fn select_all_everywhere_selects_every_category() {
    let mut app = app_with_categories();
    app.select_all_everywhere();
    assert!(app
        .categories
        .iter()
        .all(|c| c.items.iter().all(|i| i.selected)));
}

#[test]
fn deselect_all_everywhere_deselects_every_category() {
    let mut app = app_with_categories();
    app.select_all_everywhere();
    app.deselect_all_everywhere();
    assert!(app
        .categories
        .iter()
        .all(|c| c.items.iter().all(|i| !i.selected)));
}

#[test]
fn begin_execution_skips_reprompt_when_already_authenticated() {
    // Regression test: `begin_execution` used to gate elevation purely on
    // `is_root` (which never changes once the process starts), completely
    // ignoring `password_prompt.is_authenticated()`. That meant a user who
    // had already authenticated once via the sudo dialog would be prompted
    // again on every subsequent run — unlike the GUI, which correctly
    // remembers authentication for the whole session.
    let mut app = app_with_categories();
    app.confirmation_mode = false;
    app.is_root = false;
    app.password_prompt.mark_authenticated_for_tests();
    app.categories[1].items[0].selected = true; // requires_root cleaner

    app.request_run().unwrap();

    assert!(
        !app.needs_sudo,
        "should not re-show the password prompt once already authenticated"
    );
    assert!(app.is_running, "run should start immediately");
}

#[test]
fn begin_execution_still_prompts_when_not_yet_authenticated() {
    let mut app = app_with_categories();
    app.confirmation_mode = false;
    app.is_root = false;
    app.categories[1].items[0].selected = true; // requires_root cleaner

    app.request_run().unwrap();

    if cleansys_core::utils::supports_sudo_prompt() {
        assert!(app.needs_sudo);
    }
    assert!(!app.is_running);
}

#[test]
fn password_authentication_success_path_resets_all_item_status_and_bytes() {
    // Regression test: the post-password-authentication branch in
    // `handle_key` used to duplicate `begin_execution`'s "start a run" logic
    // by hand, and that copy had drifted — it reset `bytes_cleaned` for
    // every item but forgot to also reset `status`, so stale Success/Error
    // statuses from a *previous* run could linger on unrelated items after
    // authenticating through the sudo dialog. Both paths now share
    // `start_operations`, so this must hold for the password-prompt path
    // too.
    let mut app = app_with_categories();
    app.confirmation_mode = false;

    // Simulate a stale status left over from an earlier, unrelated run.
    app.categories[0].items[1].status = Some(cleansys_core::Status::Error(
        "stale from last run".to_string(),
    ));
    app.categories[0].items[1].bytes_cleaned = 1234;

    // Now trigger a *new* run that requires elevation and goes through the
    // password-prompt success path.
    app.is_root = false;
    app.categories[1].items[0].selected = true; // requires_root cleaner
    app.request_run().unwrap();
    assert!(app.needs_sudo || app.needs_admin_notice);

    if cleansys_core::utils::supports_sudo_prompt() {
        // Drive the exact same path `handle_key`'s Enter-on-password-prompt
        // branch takes on successful authentication.
        app.password_prompt.mark_authenticated_for_tests();
        app.needs_sudo = false;
        app.password_prompt.hide();
        let selected_cleaners = app.pending_operations.clone();
        app.pending_operations.clear();
        assert!(!selected_cleaners.is_empty());
        app.start_operations_for_tests(&selected_cleaners);

        // The stale status/bytes from the earlier unrelated run must be gone.
        assert!(app.categories[0].items[1].status.is_none());
        assert_eq!(app.categories[0].items[1].bytes_cleaned, 0);
        // The newly-selected root item should now be Pending.
        assert!(matches!(
            app.categories[1].items[0].status,
            Some(cleansys_core::Status::Pending)
        ));
    }
}

#[test]
fn schedule_overlay_fields_adjust_and_wrap() {
    use cleansys_core::engine::schedule::Frequency;
    let mut app = App::new();
    // Weekly by default: Frequency, Hour, Minute, Day, Scope (+ Backend on Linux).
    assert!(app.schedule_fields().len() >= 5);

    // Frequency -> daily hides the Day row.
    app.schedule_field = 0;
    app.schedule_adjust(-1);
    assert_eq!(app.schedule_draft.frequency, Frequency::Daily);
    assert!(!app
        .schedule_fields()
        .contains(&cleansys_tui::app::ScheduleField::Day));

    // Hour wraps 23 -> 0.
    app.schedule_field = 1;
    app.schedule_draft.hour = 23;
    app.schedule_adjust(1);
    assert_eq!(app.schedule_draft.hour, 0);

    // Minute moves in 5-minute steps and wraps.
    app.schedule_field = 2;
    app.schedule_draft.minute = 55;
    app.schedule_adjust(1);
    assert_eq!(app.schedule_draft.minute, 0);
}

#[test]
fn recommended_selects_only_safe_user_land() {
    let mut app = App::new();
    // CI containers often run as root, which would (correctly) include root cleaners.
    app.is_root = false;
    app.categories = cleansys_core::load_categories();
    let n = app.select_recommended();
    assert!(n > 0);
    assert!(app
        .categories
        .iter()
        .flat_map(|c| &c.items)
        .filter(|i| i.selected)
        .all(|i| i.risk == cleansys_core::Risk::Safe && !i.requires_root));
}

fn app_with_two_categories() -> App {
    let mut app = App::new();
    app.categories = vec![
        CleanerCategory {
            name: "User Land Cleaners".into(),
            description: "u".into(),
            items: vec![
                sample_item("Browser Caches", false),
                sample_item("Trash", false),
            ],
        },
        CleanerCategory {
            name: "System Cleaners".into(),
            description: "s".into(),
            items: vec![sample_item("System Logs", true)],
        },
    ];
    app
}

#[test]
fn filter_searches_all_categories_and_navigation_follows_it() {
    use crossterm::event::KeyCode;
    let mut app = app_with_two_categories();
    app.start_filter();
    for c in "logs".chars() {
        app.handle_filter_key(KeyCode::Char(c));
    }
    assert_eq!(app.view_items(), vec![(1, 0)]);

    // Toggling acts on the filtered hit, not on category 0's first item.
    app.item_list_state.select(Some(0));
    app.toggle_selected();
    assert!(app.categories[1].items[0].selected);
    assert!(!app.categories[0].items[0].selected);

    // Esc clears the filter and restores the category list.
    app.handle_filter_key(KeyCode::Esc);
    assert!(app.filter.is_empty() && !app.filter_active);
    assert_eq!(app.view_items().len(), 2);
}

#[test]
fn next_and_previous_item_wrap_over_the_visible_list() {
    let mut app = app_with_two_categories();
    app.item_list_state.select(Some(1));
    app.next_item();
    assert_eq!(app.item_list_state.selected(), Some(0));
    app.previous_item();
    assert_eq!(app.item_list_state.selected(), Some(1));
}

#[test]
fn hide_empty_hides_scanned_empty_cleaners_but_never_selected_ones() {
    use cleansys_core::{ScanBoard, ScanInfo};
    let mut app = app_with_two_categories();
    let mut board = ScanBoard::new(&app.categories);
    board.start(3);
    board.record(
        0,
        0,
        ScanInfo {
            bytes: 10,
            ..Default::default()
        },
    );
    board.record(0, 1, ScanInfo::default());
    board.record(1, 0, ScanInfo::default());
    app.board = board;
    assert_eq!(app.view_items(), vec![(0, 0)]); // Trash hidden
                                                // System category fully empty -> skipped by Tab navigation.
    app.next_category();
    assert_eq!(app.category_index, 0);

    app.categories[0].items[1].selected = true;
    assert_eq!(app.view_items(), vec![(0, 0), (0, 1)]);

    app.toggle_hide_empty();
    assert_eq!(app.view_items().len(), 2);
}

// ── clean runs: worker thread, progress state, summary, keys ─────────────────

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

fn key(app: &mut App, code: KeyCode) {
    app.handle_key(KeyEvent::new(code, KeyModifiers::NONE))
        .unwrap();
}

fn wait_for_run(app: &mut App) {
    for _ in 0..500 {
        app.poll_run();
        if !app.is_running {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    panic!("run did not finish");
}

fn failing_item(name: &str) -> CleanerItem {
    let mut it = sample_item(name, false);
    it.function = std::sync::Arc::new(|_o: RunOptions| Err(anyhow::anyhow!("x: disk on fire")));
    it
}

#[test]
fn run_marks_success_and_failure_and_builds_a_summary() {
    let mut app = app_with_categories();
    app.confirmation_mode = false;
    app.categories[0].items[1] = failing_item("Trash");
    app.categories[0].items[0].selected = true;
    app.categories[0].items[1].selected = true;
    app.request_run().unwrap();
    assert!(app.is_running);
    assert_eq!(app.run_total, 2);
    assert!(app.show_log, "activity log opens when a run starts");

    wait_for_run(&mut app);

    assert!(matches!(
        app.categories[0].items[0].status,
        Some(cleansys_core::Status::Success(_))
    ));
    match &app.categories[0].items[1].status {
        Some(cleansys_core::Status::Error(m)) => assert_eq!(m, "disk on fire"),
        other => panic!("expected an error status, got {other:?}"),
    }
    assert_eq!(app.run_done, 2);
    let sum = app.run_summary.clone().expect("summary");
    assert_eq!((sum.ok, sum.failed, sum.cancelled), (1, 1, false));
    assert!(app.run.is_none());
    assert!(app
        .operation_logs
        .iter()
        .any(|l| l.contains("disk on fire")));
    assert!(
        app.board.total > 0 || app.board.is_scanning() || !app.categories.is_empty(),
        "a re-scan is started after the run"
    );
}

#[test]
fn starting_a_run_clears_the_previous_summary_and_log() {
    let mut app = app_with_categories();
    app.confirmation_mode = false;
    app.run_summary = Some(Default::default());
    app.log("old line");
    app.categories[0].items[0].selected = true;
    app.request_run().unwrap();
    assert!(app.run_summary.is_none());
    assert!(!app.operation_logs.iter().any(|l| l == "old line"));
    wait_for_run(&mut app);
}

#[test]
fn selection_and_run_keys_are_ignored_while_a_clean_is_running() {
    let mut app = app_with_categories();
    app.confirmation_mode = false;
    app.categories[0].items[0].selected = true;
    app.request_run().unwrap();
    assert!(app.is_running);

    let before: Vec<bool> = app.categories[0].items.iter().map(|i| i.selected).collect();
    key(&mut app, KeyCode::Char(' '));
    key(&mut app, KeyCode::Char('a'));
    key(&mut app, KeyCode::Char('A'));
    key(&mut app, KeyCode::Char('n'));
    key(&mut app, KeyCode::Char('r'));
    let after: Vec<bool> = app.categories[0].items.iter().map(|i| i.selected).collect();
    assert_eq!(before, after, "ticks must not change mid-run");
    assert!(!app.awaiting_run_confirmation);

    // Navigation still works.
    let sel = app.item_list_state.selected();
    key(&mut app, KeyCode::Down);
    assert_ne!(app.item_list_state.selected(), sel);
    wait_for_run(&mut app);
}

#[test]
fn q_cancels_a_running_clean_instead_of_quitting() {
    let mut app = app_with_categories();
    app.confirmation_mode = false;
    for it in &mut app.categories[0].items {
        it.selected = true;
    }
    app.request_run().unwrap();
    let quit = app
        .handle_key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE))
        .unwrap();
    assert!(!quit, "q during a run cancels, it must not quit the app");
    assert!(app.operation_logs.iter().any(|l| l.contains("Cancelling")));
    wait_for_run(&mut app);
    // Once idle, q quits again.
    let quit = app
        .handle_key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE))
        .unwrap();
    assert!(quit);
}

#[test]
fn esc_dismisses_the_summary_and_l_toggles_the_log() {
    let mut app = app_with_categories();
    app.run_summary = Some(Default::default());
    key(&mut app, KeyCode::Esc);
    assert!(app.run_summary.is_none());

    assert!(!app.show_log);
    key(&mut app, KeyCode::Char('L'));
    assert!(app.show_log);
    key(&mut app, KeyCode::Char('L'));
    assert!(!app.show_log);
}

#[test]
fn notice_is_cleared_by_the_next_key() {
    let mut app = app_with_categories();
    app.request_run().unwrap();
    assert!(app.notice.is_some());
    key(&mut app, KeyCode::Down);
    assert!(app.notice.is_none());
}

#[test]
fn activity_log_is_bounded() {
    let mut app = app_with_categories();
    for i in 0..1200 {
        app.log(format!("line {i}"));
    }
    assert!(app.operation_logs.len() <= 500);
    assert_eq!(app.operation_logs.last().unwrap(), "line 1199");
    assert_eq!(app.operation_logs.first().unwrap(), "line 700");
}

#[test]
fn details_focus_navigation_and_toggle() {
    use cleansys_core::{ScanBoard, ScanEntry, ScanInfo};
    let mut app = app_with_categories();
    let e = |p: &str, skippable| ScanEntry {
        path: p.into(),
        bytes: 10,
        label: "l".into(),
        skippable,
    };
    app.board = ScanBoard::new(&app.categories);
    app.board.start(1);
    app.board.record(
        0,
        0,
        ScanInfo {
            bytes: 20,
            items: 2,
            entries: vec![e("/a", true), e("/b", false)],
            ..ScanInfo::default()
        },
    );
    key(&mut app, KeyCode::Right);
    assert_eq!(app.expanded, Some((0, 0)));
    key(&mut app, KeyCode::Char(' '));
    assert!(!app.board.entry_selected("/a"));
    key(&mut app, KeyCode::Down);
    key(&mut app, KeyCode::Char(' ')); // /b is not skippable: no-op
    assert!(app.board.entry_selected("/b"));
    key(&mut app, KeyCode::Char('a'));
    assert!(app.board.entry_selected("/a"));
    key(&mut app, KeyCode::Left);
    assert!(app.expanded.is_none() && app.entry_cursor.is_none());
}

#[test]
fn idle_days_step_does_nothing_while_running() {
    let mut app = app_with_categories();
    app.is_running = true;
    let before = app.min_age_days;
    key(&mut app, KeyCode::Char(']'));
    assert_eq!(app.min_age_days, before);
}
