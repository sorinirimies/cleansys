//! Background "how much can each cleaner free?" scanning, shared by the TUI
//! and GUI.
//!
//! A scan runs every cleaner in read-only preview mode on a small pool of OS
//! threads and reports per-cleaner results. [`ScanBoard`] holds the results
//! and answers the questions both front-ends ask (sizes per category, which
//! cleaners to list, what is hidden because it is empty, ...).

use std::collections::HashSet;

use crate::cleaners::cleaned_item::CleaningResult;

/// Most entries kept per cleaner (the biggest ones) so memory stays bounded
/// even for cleaners that match thousands of files.
pub const MAX_ENTRIES: usize = 300;

/// One path a cleaner would remove.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanEntry {
    pub path: String,
    pub bytes: u64,
    /// Short label, e.g. `Rust build: myproj`.
    pub label: String,
    /// May be unticked individually (see [`crate::engine::skip`]).
    pub skippable: bool,
}

/// What a scan learned about one cleaner.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScanInfo {
    /// Bytes this cleaner would free right now.
    pub bytes: u64,
    /// Number of files/directories it would remove.
    pub items: usize,
    /// Path of the biggest item, for detail lines.
    pub top_path: Option<String>,
    /// Set when the scan failed.
    pub error: Option<String>,
    /// The biggest paths it would remove (at most [`MAX_ENTRIES`]), largest first.
    pub entries: Vec<ScanEntry>,
}

impl ScanInfo {
    pub fn from_result(result: Result<CleaningResult, String>) -> Self {
        match result {
            Ok(r) => {
                let mut entries: Vec<ScanEntry> = r
                    .items
                    .iter()
                    .map(|i| ScanEntry {
                        path: i.path_str(),
                        bytes: i.size,
                        label: i.label.clone(),
                        skippable: i.skippable,
                    })
                    .collect();
                entries.sort_by(|a, b| b.bytes.cmp(&a.bytes).then_with(|| a.path.cmp(&b.path)));
                entries.truncate(MAX_ENTRIES);
                Self {
                    bytes: r.total_bytes,
                    items: r.item_count(),
                    top_path: entries.first().map(|e| e.path.clone()),
                    error: None,
                    entries,
                }
            }
            Err(e) => Self {
                error: Some(e),
                ..Self::default()
            },
        }
    }
}

/// Scan results for every cleaner, shaped like the category list.
#[derive(Debug, Clone, Default)]
pub struct ScanBoard {
    info: Vec<Vec<Option<ScanInfo>>>,
    /// Scan jobs still running.
    pub pending: usize,
    /// Scan jobs started in the current scan.
    pub total: usize,
    /// Entry paths the user unticked in the details view.
    skipped: HashSet<String>,
    /// Cleaners the user opted out of scanning (they stay unmeasured, which is final).
    off: HashSet<(usize, usize)>,
    /// The scan was stopped (e.g. a clean started) before every cleaner reported.
    interrupted: bool,
}

/// Tick state of a cleaner's entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryState {
    /// No individually-selectable entries.
    NoEntries,
    All,
    Partial,
    Nothing,
}

mod board;
mod spawn;

pub use spawn::{ScanHandle, disabled_positions, spawn_scan, spawn_scan_filtered, spawn_scan_with};
pub(crate) use spawn::{load_scan_times, save_scan_times};

#[cfg(test)]
mod tests {
    use super::spawn::spawn_scan_timeout;
    use super::*;
    use crate::Risk;
    use crate::cleaners::cleaned_item::{CleanedItem, cleaner_fn};
    use crate::model::CleanerCategory;
    use crate::model::CleanerItem;
    use std::path::PathBuf;
    use std::sync::mpsc::channel;

    fn item(id: &str, bytes: u64) -> CleanerItem {
        CleanerItem {
            id: id.into(),
            risk: Risk::Safe,
            name: format!("Name {id}"),
            description: "desc".into(),
            requires_root: false,
            selected: false,
            function: cleaner_fn(move |_| {
                let mut r = CleaningResult::new();
                if bytes > 0 {
                    r.add_item(CleanedItem::file(
                        PathBuf::from(format!("/x/{bytes}")),
                        bytes,
                        "t",
                    ));
                }
                Ok(r)
            }),
            bytes_cleaned: 0,
            last_result: None,
            status: None,
        }
    }

    fn cats() -> Vec<CleanerCategory> {
        vec![
            CleanerCategory {
                name: "A".into(),
                description: "".into(),
                items: vec![item("a1", 100), item("a2", 0)],
            },
            CleanerCategory {
                name: "B".into(),
                description: "".into(),
                items: vec![item("b1", 0)],
            },
        ]
    }

    #[test]
    fn scan_fills_board_and_hides_empty() {
        let c = cats();
        let (rx, total) = spawn_scan(&c);
        assert_eq!(total, 3);
        let mut board = ScanBoard::new(&c);
        board.start(total);
        while board.is_scanning() {
            let (ci, ii, info) = rx.recv().unwrap();
            board.record(ci, ii, info);
        }
        assert!(board.complete());
        assert_eq!(board.total_bytes(), 100);
        assert_eq!(board.category_bytes(0), Some(100));
        assert_eq!(board.get(0, 0).unwrap().top_path.as_deref(), Some("/x/100"));
        // empty a2 and whole category B disappear
        assert_eq!(board.visible_items(&c, 0, "", true), vec![(0, 0)]);
        assert_eq!(board.visible_items(&c, 0, "", false), vec![(0, 0), (0, 1)]);
        assert!(!board.category_visible(&c, 1, true));
        assert!(board.category_visible(&c, 1, false));
        assert_eq!(board.first_visible_category(&c, true), Some(0));
    }

    #[test]
    fn hung_scan_is_reported_as_timed_out_and_late_result_is_dropped() {
        let hang = CleanerItem {
            function: cleaner_fn(|_| {
                std::thread::sleep(std::time::Duration::from_millis(1500));
                Ok(CleaningResult::new())
            }),
            ..item("hang", 0)
        };
        let c = vec![CleanerCategory {
            name: "A".into(),
            description: "".into(),
            items: vec![hang, item("ok", 5)],
        }];
        let (tx, rx) = channel();
        let total = spawn_scan_timeout(
            &c,
            std::time::Duration::from_millis(600),
            move |ci, ii, i| tx.send((ci, ii, i)).is_ok(),
        );
        assert_eq!(total, 2);
        let mut got = Vec::new();
        for _ in 0..2 {
            got.push(rx.recv_timeout(std::time::Duration::from_secs(5)).unwrap());
        }
        let timed_out = got.iter().find(|g| g.1 == 0).unwrap();
        assert!(
            timed_out
                .2
                .error
                .as_deref()
                .unwrap_or("")
                .contains("timed out")
        );
        // the hung cleaner finishing later must not be delivered a second time
        assert!(rx.recv_timeout(std::time::Duration::from_secs(2)).is_err());
    }

    #[test]
    fn category_totals_are_only_given_once_every_cleaner_is_measured() {
        let c = cats();
        let mut board = ScanBoard::new(&c);
        board.start(3);
        assert_eq!(board.category_bytes_when_done(0), None);
        board.record(
            0,
            0,
            ScanInfo {
                bytes: 100,
                ..Default::default()
            },
        );
        assert!(!board.category_done(0), "one of two measured");
        assert_eq!(board.category_bytes_when_done(0), None);
        board.record(0, 1, ScanInfo::default());
        assert_eq!(board.category_bytes_when_done(0), Some(100));
        assert_eq!(board.category_bytes_when_done(1), None);
    }

    #[test]
    fn filter_searches_every_category_and_ignores_hide_empty() {
        let c = cats();
        let board = ScanBoard::new(&c);
        assert_eq!(board.visible_items(&c, 0, "b1", true), vec![(1, 0)]);
        assert_eq!(board.visible_items(&c, 0, "NAME", true).len(), 3);
        assert!(board.visible_items(&c, 0, "zzz", true).is_empty());
    }

    #[test]
    fn recommended_skips_cleaners_found_empty() {
        let mut c = cats();
        let (rx, total) = spawn_scan(&c);
        let mut board = ScanBoard::new(&c);
        board.start(total);
        while board.is_scanning() {
            let (ci, ii, info) = rx.recv().unwrap();
            board.record(ci, ii, info);
        }
        // a1 has data, a2/b1 are empty -> only a1 stays ticked
        assert_eq!(board.select_recommended(&mut c, false), 1);
        assert!(c[0].items[0].selected && !c[0].items[1].selected && !c[1].items[0].selected);
    }

    #[test]
    fn selected_items_are_never_hidden() {
        let mut c = cats();
        let mut board = ScanBoard::new(&c);
        board.start(1);
        board.record(0, 1, ScanInfo::default());
        board.pending = 0;
        board.total = 3;
        assert!(board.is_hidden_empty(&c, 0, 1, true));
        c[0].items[1].selected = true;
        assert!(!board.is_hidden_empty(&c, 0, 1, true));
    }

    #[test]
    fn entry_selection_adjusts_bytes_and_state() {
        let cats = vec![CleanerCategory {
            name: "c".into(),
            description: String::new(),
            items: vec![item("a", 0)],
        }];
        let mut b = ScanBoard::new(&cats);
        let e = |p: &str, bytes| ScanEntry {
            path: p.into(),
            bytes,
            label: String::new(),
            skippable: true,
        };
        b.start(1);
        b.record(
            0,
            0,
            ScanInfo {
                bytes: 100,
                items: 2,
                entries: vec![e("/x/big", 70), e("/x/small", 30)],
                ..ScanInfo::default()
            },
        );
        assert_eq!(b.entry_state(0, 0), EntryState::All);
        assert_eq!(b.item_selected_bytes(0, 0), 100);
        assert!(!b.toggle_entry("/x/big"));
        assert_eq!(b.entry_state(0, 0), EntryState::Partial);
        assert_eq!(b.item_selected_bytes(0, 0), 30);
        assert_eq!(b.skipped_paths().len(), 1);
        b.set_item_entries(0, 0, false);
        assert_eq!(b.entry_state(0, 0), EntryState::Nothing);
        b.set_item_entries(0, 0, true);
        assert_eq!(b.item_selected_bytes(0, 0), 100);
    }

    #[test]
    fn parent_follows_its_paths_and_overrides_them() {
        let mut cats = vec![CleanerCategory {
            name: "c".into(),
            description: String::new(),
            items: vec![item("a", 0)],
        }];
        let mut b = ScanBoard::new(&cats);
        let e = |p: &str| ScanEntry {
            path: p.into(),
            bytes: 1,
            label: String::new(),
            skippable: true,
        };
        b.start(1);
        b.record(
            0,
            0,
            ScanInfo {
                bytes: 2,
                items: 2,
                entries: vec![e("/a"), e("/b")],
                ..ScanInfo::default()
            },
        );
        cats[0].items[0].selected = true;
        b.toggle_entry("/a");
        b.sync_item_to_entries(&mut cats, 0, 0);
        assert!(cats[0].items[0].selected, "one path left ticked");
        b.toggle_entry("/b");
        b.sync_item_to_entries(&mut cats, 0, 0);
        assert!(
            !cats[0].items[0].selected,
            "all paths unticked -> parent off"
        );
        b.toggle_entry("/b");
        b.sync_item_to_entries(&mut cats, 0, 0);
        assert!(cats[0].items[0].selected, "a path ticked -> parent on");
        cats[0].items[0].selected = false;
        b.sync_entries_to_item(&cats, 0, 0);
        assert_eq!(
            b.entry_state(0, 0),
            EntryState::Nothing,
            "parent off -> paths off"
        );
    }

    #[test]
    fn fresh_scan_entries_follow_unticked_parent() {
        let mut cats = vec![CleanerCategory {
            name: "c".into(),
            description: String::new(),
            items: vec![item("a", 0), item("b", 0)],
        }];
        let info = || ScanInfo {
            bytes: 2,
            items: 2,
            entries: ["/p", "/q"]
                .map(|p| ScanEntry {
                    path: p.into(),
                    bytes: 1,
                    label: String::new(),
                    skippable: true,
                })
                .to_vec(),
            ..ScanInfo::default()
        };
        let mut b = ScanBoard::new(&cats);
        cats[0].items[0].selected = false;
        cats[0].items[1].selected = true;
        b.start(2);
        b.record_synced(&cats, 0, 0, info());
        assert_eq!(b.entry_state(0, 0), EntryState::Nothing, "unticked parent");
        b.record_synced(&cats, 0, 1, info());
        assert_eq!(b.entry_state(0, 1), EntryState::All, "ticked parent");
    }

    #[test]
    fn select_recommended_resyncs_entries() {
        let mut cats = vec![CleanerCategory {
            name: "c".into(),
            description: String::new(),
            items: vec![item("a", 0)],
        }];
        let mut b = ScanBoard::new(&cats);
        b.start(1);
        b.record(
            0,
            0,
            ScanInfo {
                bytes: 1,
                items: 1,
                entries: vec![ScanEntry {
                    path: "/p".into(),
                    bytes: 1,
                    label: String::new(),
                    skippable: true,
                }],
                ..ScanInfo::default()
            },
        );
        cats[0].items[0].selected = false;
        b.select_recommended(&mut cats, false);
        let on = cats[0].items[0].selected;
        let want = if on {
            EntryState::All
        } else {
            EntryState::Nothing
        };
        assert_eq!(b.entry_state(0, 0), want, "paths mirror the cleaner");
    }

    #[test]
    fn opted_out_cleaners_are_not_scanned_and_do_not_block_completion() {
        let c = cats();
        let off = disabled_positions(&c, &["a2".to_string()]);
        assert_eq!(off, HashSet::from([(0, 1)]));
        let (tx, rx) = channel();
        let handle = spawn_scan_filtered(
            &c,
            |ci, ii, _| !off.contains(&(ci, ii)),
            move |ci, ii, info| tx.send((ci, ii, info)).is_ok(),
        );
        assert_eq!(handle.total, 2);
        let mut board = ScanBoard::new(&c);
        board.start_with(handle.total, off);
        while board.is_scanning() {
            let (ci, ii, info) = rx.recv_timeout(std::time::Duration::from_secs(10)).unwrap();
            board.record(ci, ii, info);
        }
        assert!(board.complete());
        assert!(board.is_off(0, 1) && board.get(0, 1).is_none());
        // Category A is "done" although a2 was never measured.
        assert_eq!(board.category_bytes_when_done(0), Some(100));
    }

    #[test]
    fn cancel_keeps_results_so_far_and_drops_late_ones() {
        let c = cats();
        let mut board = ScanBoard::new(&c);
        board.start(3);
        board.record(
            0,
            0,
            ScanInfo {
                bytes: 100,
                ..Default::default()
            },
        );
        board.cancel();
        assert!(!board.is_scanning() && !board.complete() && board.was_interrupted());
        // A result still in flight when the scan was stopped is ignored.
        assert!(!board.record(
            0,
            1,
            ScanInfo {
                bytes: 5,
                ..Default::default()
            }
        ));
        assert!(board.get(0, 1).is_none());
        // What was measured before the stop is still usable (clean can use it).
        assert_eq!(board.get(0, 0).map(|i| i.bytes), Some(100));
        // A fresh scan clears the interruption.
        board.start(1);
        assert!(board.is_scanning() && !board.was_interrupted());
    }

    #[test]
    fn cancelled_scan_delivers_nothing_more() {
        let slow = |id: &str| {
            let mut i = item(id, 1);
            i.function = cleaner_fn(|_| {
                std::thread::sleep(std::time::Duration::from_millis(300));
                Ok(CleaningResult::new())
            });
            i
        };
        let c = vec![CleanerCategory {
            name: "A".into(),
            description: "".into(),
            items: (0..40).map(|n| slow(&format!("s{n}"))).collect(),
        }];
        let (tx, rx) = channel();
        let handle = spawn_scan_filtered(
            &c,
            |_, _, _| true,
            move |ci, ii, info| tx.send((ci, ii, info)).is_ok(),
        );
        handle.cancel();
        std::thread::sleep(std::time::Duration::from_millis(1200));
        let got = std::iter::from_fn(|| rx.try_recv().ok()).count();
        assert_eq!(got, 0, "no result after cancel");
    }
}
