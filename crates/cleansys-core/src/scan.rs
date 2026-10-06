//! Background "how much can each cleaner free?" scanning, shared by the TUI
//! and GUI.
//!
//! A scan runs every cleaner in read-only preview mode on a small pool of OS
//! threads and reports per-cleaner results. [`ScanBoard`] holds the results
//! and answers the questions both front-ends ask (sizes per category, which
//! cleaners to list, what is hidden because it is empty, ...).

use std::collections::VecDeque;
use std::sync::mpsc::{channel, Receiver};
use std::sync::{Arc, Mutex};

use crate::cleaners::cleaned_item::{CleaningResult, RunOptions};
use crate::model::CleanerCategory;

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
}

impl ScanInfo {
    pub fn from_result(result: Result<CleaningResult, String>) -> Self {
        match result {
            Ok(r) => Self {
                bytes: r.total_bytes,
                items: r.item_count(),
                top_path: r.items.iter().max_by_key(|i| i.size).map(|i| i.path_str()),
                error: None,
            },
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
}

impl ScanBoard {
    pub fn new(categories: &[CleanerCategory]) -> Self {
        Self {
            info: categories
                .iter()
                .map(|c| vec![None; c.items.len()])
                .collect(),
            pending: 0,
            total: 0,
        }
    }

    /// Forget all results and mark `total` jobs as pending.
    pub fn start(&mut self, total: usize) {
        for row in &mut self.info {
            row.iter_mut().for_each(|s| *s = None);
        }
        self.total = total;
        self.pending = total;
    }

    /// Store one result; returns `true` when it was the last pending one.
    pub fn record(&mut self, cat: usize, item: usize, info: ScanInfo) -> bool {
        if let Some(slot) = self.info.get_mut(cat).and_then(|r| r.get_mut(item)) {
            *slot = Some(info);
        }
        self.pending = self.pending.saturating_sub(1);
        self.pending == 0
    }

    pub fn is_scanning(&self) -> bool {
        self.pending > 0
    }

    /// A scan has run and finished.
    pub fn complete(&self) -> bool {
        self.total > 0 && self.pending == 0
    }

    pub fn get(&self, cat: usize, item: usize) -> Option<&ScanInfo> {
        self.info.get(cat)?.get(item)?.as_ref()
    }

    /// Sum for a category, if at least one of its cleaners was measured.
    pub fn category_bytes(&self, cat: usize) -> Option<u64> {
        let row = self.info.get(cat)?;
        row.iter()
            .any(Option::is_some)
            .then(|| row.iter().flatten().map(|s| s.bytes).sum())
    }

    pub fn total_bytes(&self) -> u64 {
        self.info.iter().flatten().flatten().map(|s| s.bytes).sum()
    }

    /// Total of the ticked, measured cleaners.
    pub fn selected_bytes(&self, categories: &[CleanerCategory]) -> u64 {
        categories
            .iter()
            .enumerate()
            .flat_map(|(ci, c)| {
                c.items
                    .iter()
                    .enumerate()
                    .filter(|(_, i)| i.selected)
                    .map(move |(ii, _)| (ci, ii))
            })
            .filter_map(|(c, i)| self.get(c, i))
            .map(|s| s.bytes)
            .sum()
    }

    /// Hidden by "hide empty": scanned, nothing found, no error, not ticked.
    pub fn is_hidden_empty(
        &self,
        categories: &[CleanerCategory],
        cat: usize,
        item: usize,
        hide_empty: bool,
    ) -> bool {
        hide_empty
            && self.complete()
            && self
                .get(cat, item)
                .is_some_and(|s| s.bytes == 0 && s.error.is_none())
            && !categories[cat].items[item].selected
    }

    /// Whether a category has anything worth listing.
    pub fn category_visible(
        &self,
        categories: &[CleanerCategory],
        cat: usize,
        hide_empty: bool,
    ) -> bool {
        if !(hide_empty && self.complete()) {
            return true;
        }
        (0..categories[cat].items.len()).any(|i| !self.is_hidden_empty(categories, cat, i, true))
    }

    /// `(category, item)` pairs to list: filter hits across every category,
    /// or the active category minus hidden-empty cleaners.
    pub fn visible_items(
        &self,
        categories: &[CleanerCategory],
        active: usize,
        filter: &str,
        hide_empty: bool,
    ) -> Vec<(usize, usize)> {
        let q = filter.trim().to_lowercase();
        let mut out = Vec::new();
        for (ci, cat) in categories.iter().enumerate() {
            if q.is_empty() && ci != active {
                continue;
            }
            for (ii, item) in cat.items.iter().enumerate() {
                let hit = q.is_empty()
                    || item.name.to_lowercase().contains(&q)
                    || item.description.to_lowercase().contains(&q)
                    || item.id.to_lowercase().contains(&q)
                    || cat.name.to_lowercase().contains(&q);
                if hit && (!q.is_empty() || !self.is_hidden_empty(categories, ci, ii, hide_empty)) {
                    out.push((ci, ii));
                }
            }
        }
        out
    }

    /// First visible category at or after `active`'s position, if `active`
    /// itself got hidden.
    pub fn first_visible_category(
        &self,
        categories: &[CleanerCategory],
        hide_empty: bool,
    ) -> Option<usize> {
        (0..categories.len()).find(|c| self.category_visible(categories, *c, hide_empty))
    }
}

/// Start a background scan of every cleaner. Results arrive on the returned
/// channel as `(category, item, info)`; the second value is the job count.
pub fn spawn_scan(categories: &[CleanerCategory]) -> (Receiver<(usize, usize, ScanInfo)>, usize) {
    let jobs: VecDeque<_> = categories
        .iter()
        .enumerate()
        .flat_map(|(ci, c)| {
            c.items
                .iter()
                .enumerate()
                .map(move |(ii, i)| (ci, ii, i.function.clone()))
        })
        .collect();
    let total = jobs.len();
    let queue = Arc::new(Mutex::new(jobs));
    let (tx, rx) = channel();
    let workers = std::thread::available_parallelism()
        .map(|n| n.get().clamp(2, 6))
        .unwrap_or(4);
    for _ in 0..workers {
        let queue = Arc::clone(&queue);
        let tx = tx.clone();
        std::thread::spawn(move || loop {
            let job = queue.lock().unwrap_or_else(|e| e.into_inner()).pop_front();
            let Some((ci, ii, f)) = job else { break };
            let info = ScanInfo::from_result(f(RunOptions::preview()).map_err(|e| e.to_string()));
            if tx.send((ci, ii, info)).is_err() {
                break; // receiver dropped (app closed)
            }
        });
    }
    (rx, total)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cleaners::cleaned_item::{cleaner_fn, CleanedItem};
    use crate::model::CleanerItem;
    use crate::Risk;
    use std::path::PathBuf;

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
    fn filter_searches_every_category_and_ignores_hide_empty() {
        let c = cats();
        let board = ScanBoard::new(&c);
        assert_eq!(board.visible_items(&c, 0, "b1", true), vec![(1, 0)]);
        assert_eq!(board.visible_items(&c, 0, "NAME", true).len(), 3);
        assert!(board.visible_items(&c, 0, "zzz", true).is_empty());
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
}
