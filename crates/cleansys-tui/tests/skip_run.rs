//! Unticked details paths must be left alone by a run started from the app.
//!
//! The skip set is process-wide, so this lives in its own test binary (one test, no
//! other worker threads clearing it underneath).

use cleansys_core::{CleanerCategory, CleanerItem, CleaningResult, RunOptions};
use cleansys_tui::app::App;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[test]
fn unticked_paths_are_not_cleaned_by_a_run() {
    let skipped_seen = Arc::new(AtomicBool::new(false));
    let probe = Arc::clone(&skipped_seen);
    let item = CleanerItem {
        id: "probe".into(),
        risk: cleansys_core::Risk::Safe,
        name: "Probe".into(),
        description: "d".into(),
        requires_root: false,
        selected: true,
        function: Arc::new(move |opts: RunOptions| {
            assert!(opts.honor_skips);
            probe.store(
                cleansys_core::engine::skip::is_skipped(std::path::Path::new(
                    "/app-test-unticked/project/target",
                )),
                Ordering::SeqCst,
            );
            Ok(CleaningResult::new())
        }),
        bytes_cleaned: 0,
        last_result: None,
        status: None,
    };
    let mut app = App::new();
    app.categories = vec![CleanerCategory {
        name: "User".into(),
        description: "d".into(),
        items: vec![item],
    }];
    app.confirmation_mode = false;
    app.board.toggle_entry("/app-test-unticked/project/target");
    app.request_run().unwrap();
    for _ in 0..500 {
        app.poll_run();
        if !app.is_running {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(!app.is_running, "run did not finish");
    assert!(
        skipped_seen.load(Ordering::SeqCst),
        "the run must skip the unticked path"
    );
    assert!(!cleansys_core::engine::skip::is_skipped(
        std::path::Path::new("/app-test-unticked/project/target")
    ));
}
