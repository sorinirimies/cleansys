use std::collections::HashSet;

use super::*;
use crate::model::CleanerCategory;

impl ScanBoard {
    pub fn new(categories: &[CleanerCategory]) -> Self {
        Self {
            info: categories
                .iter()
                .map(|c| vec![None; c.items.len()])
                .collect(),
            pending: 0,
            total: 0,
            skipped: HashSet::new(),
            off: HashSet::new(),
            interrupted: false,
        }
    }

    // ── per-entry selection ────────────────────────────────────────

    /// Is this entry ticked (i.e. not skipped)?
    pub fn entry_selected(&self, path: &str) -> bool {
        !self.skipped.contains(path)
    }

    /// Flip one entry. Returns its new ticked state.
    pub fn toggle_entry(&mut self, path: &str) -> bool {
        if !self.skipped.remove(path) {
            self.skipped.insert(path.to_string());
            return false;
        }
        true
    }

    /// A cleaner's own checkbox overrides its paths: make every path follow `item.selected`.
    pub fn sync_entries_to_item(
        &mut self,
        categories: &[CleanerCategory],
        cat: usize,
        item: usize,
    ) {
        if let Some(it) = categories.get(cat).and_then(|c| c.items.get(item)) {
            self.set_item_entries(cat, item, it.selected);
        }
    }

    /// [`Self::sync_entries_to_item`] for every cleaner (after bulk select / deselect).
    pub fn sync_all_entries_to_items(&mut self, categories: &[CleanerCategory]) {
        for (ci, c) in categories.iter().enumerate() {
            for ii in 0..c.items.len() {
                self.sync_entries_to_item(categories, ci, ii);
            }
        }
    }

    /// After a path was (un)ticked: a cleaner with any ticked path is selected, one with
    /// every path unticked is not.
    pub fn sync_item_to_entries(
        &self,
        categories: &mut [CleanerCategory],
        cat: usize,
        item: usize,
    ) {
        let Some(it) = categories.get_mut(cat).and_then(|c| c.items.get_mut(item)) else {
            return;
        };
        match self.entry_state(cat, item) {
            EntryState::Nothing => it.selected = false,
            EntryState::All | EntryState::Partial => it.selected = true,
            EntryState::NoEntries => {}
        }
    }

    /// Tick or untick every skippable entry of one cleaner.
    pub fn set_item_entries(&mut self, cat: usize, item: usize, on: bool) {
        let paths: Vec<String> = self
            .get(cat, item)
            .map(|i| {
                i.entries
                    .iter()
                    .filter(|e| e.skippable)
                    .map(|e| e.path.clone())
                    .collect()
            })
            .unwrap_or_default();
        for p in paths {
            if on {
                self.skipped.remove(&p);
            } else {
                self.skipped.insert(p);
            }
        }
    }

    /// Paths to hand to [`crate::engine::skip::set_skipped`] for a run/preview.
    pub fn skipped_paths(&self) -> Vec<std::path::PathBuf> {
        self.skipped.iter().map(std::path::PathBuf::from).collect()
    }

    /// Tick state of one cleaner's skippable entries.
    pub fn entry_state(&self, cat: usize, item: usize) -> EntryState {
        let Some(info) = self.get(cat, item) else {
            return EntryState::NoEntries;
        };
        let mut total = 0usize;
        let mut off = 0usize;
        for e in info.entries.iter().filter(|e| e.skippable) {
            total += 1;
            off += usize::from(self.skipped.contains(&e.path));
        }
        match (total, off) {
            (0, _) => EntryState::NoEntries,
            (_, 0) => EntryState::All,
            (t, o) if o == t => EntryState::Nothing,
            _ => EntryState::Partial,
        }
    }

    /// Bytes a ticked cleaner would still free after unticked entries are removed.
    pub fn item_selected_bytes(&self, cat: usize, item: usize) -> u64 {
        let Some(info) = self.get(cat, item) else {
            return 0;
        };
        let skipped: u64 = info
            .entries
            .iter()
            .filter(|e| e.skippable && self.skipped.contains(&e.path))
            .map(|e| e.bytes)
            .sum();
        info.bytes.saturating_sub(skipped)
    }

    /// Forget all results and mark `total` jobs as pending.
    pub fn start(&mut self, total: usize) {
        self.start_with(total, HashSet::new());
    }

    /// [`Self::start`] for a scan that leaves the cleaners in `off` (category, item) alone.
    pub fn start_with(&mut self, total: usize, off: HashSet<(usize, usize)>) {
        self.off = off;
        self.interrupted = false;
        self.skipped.clear();
        for row in &mut self.info {
            row.iter_mut().for_each(|s| *s = None);
        }
        self.total = total;
        self.pending = total;
    }

    /// Store one result; returns `true` when it was the last pending one. A result that
    /// arrives with no scan running (it was cancelled) is dropped and returns `false`.
    pub fn record(&mut self, cat: usize, item: usize, info: ScanInfo) -> bool {
        if self.pending == 0 {
            return false;
        }
        if let Some(slot) = self.info.get_mut(cat).and_then(|r| r.get_mut(item)) {
            *slot = Some(info);
        }
        self.pending = self.pending.saturating_sub(1);
        self.pending == 0
    }

    /// [`Self::record`], then make the new entries follow the cleaner's checkbox
    /// (an unticked cleaner starts with all paths unticked).
    pub fn record_synced(
        &mut self,
        categories: &[CleanerCategory],
        cat: usize,
        item: usize,
        info: ScanInfo,
    ) -> bool {
        let done = self.record(cat, item, info);
        self.sync_entries_to_item(categories, cat, item);
        done
    }

    pub fn is_scanning(&self) -> bool {
        self.pending > 0
    }

    /// Stop counting a running scan (its remaining results will be ignored). What was
    /// measured so far stays usable; the board is not [`Self::complete`].
    pub fn cancel(&mut self) {
        if self.pending > 0 {
            self.interrupted = true;
        }
        self.pending = 0;
    }

    /// A scan has run and finished (not cancelled).
    pub fn complete(&self) -> bool {
        self.total > 0 && self.pending == 0 && !self.interrupted
    }

    /// The last scan was stopped before it finished.
    pub fn was_interrupted(&self) -> bool {
        self.interrupted
    }

    /// Is this cleaner excluded from scanning by the user's opt-out?
    pub fn is_off(&self, cat: usize, item: usize) -> bool {
        self.off.contains(&(cat, item))
    }

    /// Has a size to show (or failed) — i.e. it may be cleaned without waiting for the
    /// rest of the scan.
    pub fn is_measured(&self, cat: usize, item: usize) -> bool {
        self.get(cat, item).is_some()
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

    /// Every cleaner of the category has been measured (a partial sum would mislead).
    pub fn category_done(&self, cat: usize) -> bool {
        self.info.get(cat).is_some_and(|row| {
            !row.is_empty()
                && row
                    .iter()
                    .enumerate()
                    .all(|(i, s)| s.is_some() || self.off.contains(&(cat, i)))
        })
    }

    /// Category total, only once the whole category is measured.
    pub fn category_bytes_when_done(&self, cat: usize) -> Option<u64> {
        self.category_done(cat)
            .then(|| self.category_bytes(cat).unwrap_or(0))
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
            .map(|(c, i)| self.item_selected_bytes(c, i))
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

    /// The "recommended" preset (safe, user-land cleaners) minus anything a
    /// finished scan found empty — nothing to gain from ticking those.
    /// Returns how many cleaners are now ticked.
    pub fn select_recommended(
        &mut self,
        categories: &mut [CleanerCategory],
        include_root: bool,
    ) -> usize {
        crate::model::select_recommended(categories, include_root);
        let mut n = 0;
        for (ci, cat) in categories.iter_mut().enumerate() {
            for (ii, item) in cat.items.iter_mut().enumerate() {
                if item.selected
                    && self.complete()
                    && self
                        .get(ci, ii)
                        .is_some_and(|s| s.bytes == 0 && s.error.is_none())
                {
                    item.selected = false;
                }
                n += usize::from(item.selected);
            }
        }
        self.sync_all_entries_to_items(categories);
        n
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
