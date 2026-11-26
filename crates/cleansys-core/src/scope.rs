//! What the category menu can show: one category, or **all** the cleaners of an area.
//!
//! The menu is split into *User Land* and *System*. Each section starts with an "All" entry
//! that lists every cleaner of the section in one flat list, followed by its categories.

use crate::model::{CleanerCategory, is_system_category};
use crate::scan::ScanBoard;

/// One entry of the category menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// Every cleaner of the user-land (`system: false`) or system (`system: true`) side.
    All { system: bool },
    /// A single category (index into the categories).
    Category(usize),
}

/// A category is on the system side when it is the built-in system group, a root one, or
/// every cleaner in it needs root.
fn is_system(c: &CleanerCategory) -> bool {
    is_system_category(&c.name) || (!c.items.is_empty() && c.items.iter().all(|i| i.requires_root))
}

/// The menu in display order: `All user land`, its categories, `All system`, its categories.
/// An area without categories gets no "All" entry.
pub fn nav_scopes(categories: &[CleanerCategory]) -> Vec<Scope> {
    nav_scopes_with_off(categories, |_| false)
}

/// [`nav_scopes`] with every category for which `off` is true (all its cleaners have
/// scanning switched off) moved to the end of its section, User land or System.
pub fn nav_scopes_with_off(
    categories: &[CleanerCategory],
    off: impl Fn(usize) -> bool,
) -> Vec<Scope> {
    let mut out = Vec::new();
    for system in [false, true] {
        let mut own: Vec<usize> = (0..categories.len())
            .filter(|&i| is_system(&categories[i]) == system)
            .collect();
        own.sort_by_key(|&i| off(i)); // stable: menu order within each group
        if own.is_empty() {
            continue;
        }
        out.push(Scope::All { system });
        out.extend(own.into_iter().map(Scope::Category));
    }
    out
}

impl Scope {
    /// Indices of the categories this entry lists.
    pub fn categories(self, categories: &[CleanerCategory]) -> Vec<usize> {
        match self {
            Scope::Category(i) => vec![i],
            Scope::All { system } => (0..categories.len())
                .filter(|&i| is_system(&categories[i]) == system)
                .collect(),
        }
    }

    /// Menu label.
    pub fn title(self, categories: &[CleanerCategory]) -> String {
        match self {
            Scope::All { system: false } => "All user land".to_string(),
            Scope::All { system: true } => "All system".to_string(),
            Scope::Category(i) => categories
                .get(i)
                .map(|c| {
                    c.name
                        .trim_end_matches(crate::model::ROOT_SUFFIX)
                        .to_string()
                })
                .unwrap_or_default(),
        }
    }

    /// Number of ticked cleaners in this entry.
    pub fn selected_count(self, categories: &[CleanerCategory]) -> usize {
        self.categories(categories)
            .into_iter()
            .map(|ci| categories[ci].items.iter().filter(|i| i.selected).count())
            .sum()
    }

    /// Tick or untick every cleaner of this entry. Root cleaners are left alone unless
    /// `can_root` (a front-end that asks for the password later passes `true`).
    /// Returns how many cleaners changed.
    pub fn set_selected(
        self,
        categories: &mut [CleanerCategory],
        on: bool,
        can_root: bool,
    ) -> usize {
        let mut changed = 0;
        for ci in self.categories(categories) {
            for item in &mut categories[ci].items {
                if (can_root || !item.requires_root) && item.selected != on {
                    item.selected = on;
                    changed += 1;
                }
            }
        }
        changed
    }

    /// Is this entry on the system (root) side?
    pub fn is_system(self, categories: &[CleanerCategory]) -> bool {
        match self {
            Scope::All { system } => system,
            Scope::Category(i) => categories.get(i).is_some_and(is_system),
        }
    }
}

impl ScanBoard {
    /// `(category, item)` pairs to list for a menu entry; a non-empty `filter` searches
    /// every category regardless of the entry.
    pub fn scope_items(
        &self,
        categories: &[CleanerCategory],
        scope: Scope,
        filter: &str,
        hide_empty: bool,
    ) -> Vec<(usize, usize)> {
        if !filter.trim().is_empty() {
            return self.visible_items(categories, 0, filter, hide_empty);
        }
        let mut out: Vec<(usize, usize)> = scope
            .categories(categories)
            .into_iter()
            .flat_map(|ci| self.visible_items(categories, ci, "", hide_empty))
            .collect();
        self.sort_biggest_first(&mut out);
        out
    }

    /// Whether the entry has anything worth listing.
    pub fn scope_visible(
        &self,
        categories: &[CleanerCategory],
        scope: Scope,
        hide_empty: bool,
    ) -> bool {
        scope
            .categories(categories)
            .into_iter()
            .any(|ci| self.category_visible(categories, ci, hide_empty))
    }

    /// Total of the entry, once every category in it is measured.
    pub fn scope_bytes_when_done(
        &self,
        categories: &[CleanerCategory],
        scope: Scope,
    ) -> Option<u64> {
        let mut sum = 0;
        for ci in scope.categories(categories) {
            sum += self.category_bytes_when_done(ci)?;
        }
        Some(sum)
    }
}

/// Tick or untick every cleaner of every category (same root rule as
/// [`Scope::set_selected`]).
pub fn set_all_selected(categories: &mut [CleanerCategory], on: bool, can_root: bool) {
    for system in [false, true] {
        Scope::All { system }.set_selected(categories, on, can_root);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Risk;
    use crate::cleaners::cleaned_item::cleaner_fn;
    use crate::model::CleanerItem;

    fn cat(name: &str, ids: &[&str]) -> CleanerCategory {
        CleanerCategory {
            name: name.into(),
            description: String::new(),
            items: ids
                .iter()
                .map(|id| CleanerItem {
                    id: (*id).into(),
                    risk: Risk::Safe,
                    name: (*id).into(),
                    description: String::new(),
                    requires_root: false,
                    selected: false,
                    function: cleaner_fn(|_| Ok(crate::CleaningResult::new())),
                    bytes_cleaned: 0,
                    last_result: None,
                    status: None,
                })
                .collect(),
        }
    }

    fn sample() -> Vec<CleanerCategory> {
        vec![
            cat("User Land Cleaners", &["u1"]),
            cat("Browsers", &["b1", "b2"]),
            cat("System Cleaners", &["s1"]),
            cat("Logs (root)", &["l1"]),
        ]
    }

    #[test]
    fn menu_has_an_all_entry_at_the_top_of_each_section() {
        assert_eq!(
            nav_scopes(&sample()),
            vec![
                Scope::All { system: false },
                Scope::Category(0),
                Scope::Category(1),
                Scope::All { system: true },
                Scope::Category(2),
                Scope::Category(3),
            ]
        );
    }

    #[test]
    fn an_area_without_categories_has_no_all_entry() {
        let only_user = vec![cat("User Land Cleaners", &["u1"])];
        assert_eq!(
            nav_scopes(&only_user),
            vec![Scope::All { system: false }, Scope::Category(0)]
        );
        assert!(nav_scopes(&[]).is_empty());
    }

    #[test]
    fn all_lists_every_cleaner_of_the_area_only() {
        let c = sample();
        let b = ScanBoard::new(&c);
        let user = b.scope_items(&c, Scope::All { system: false }, "", false);
        assert_eq!(user, vec![(0, 0), (1, 0), (1, 1)]);
        let sys = b.scope_items(&c, Scope::All { system: true }, "", false);
        assert_eq!(sys, vec![(2, 0), (3, 0)]);
        assert_eq!(b.scope_items(&c, Scope::Category(1), "", false).len(), 2);
    }

    #[test]
    fn a_search_ignores_the_entry_and_finds_across_areas() {
        let c = sample();
        let b = ScanBoard::new(&c);
        let hits = b.scope_items(&c, Scope::All { system: false }, "l1", false);
        assert_eq!(hits, vec![(3, 0)]);
    }

    #[test]
    fn titles_and_side() {
        let c = sample();
        assert_eq!(Scope::All { system: true }.title(&c), "All system");
        assert_eq!(Scope::Category(3).title(&c), "Logs");
        assert!(Scope::Category(3).is_system(&c));
        assert!(!Scope::All { system: false }.is_system(&c));
    }

    #[test]
    fn all_total_waits_for_every_category() {
        let c = sample();
        let mut b = ScanBoard::new(&c);
        assert_eq!(
            b.scope_bytes_when_done(&c, Scope::All { system: false }),
            None
        );
        b.start(4);
        for (ci, ii) in [(0, 0), (1, 0), (1, 1)] {
            b.record(
                ci,
                ii,
                crate::ScanInfo {
                    bytes: 10,
                    ..Default::default()
                },
            );
        }
        assert_eq!(
            b.scope_bytes_when_done(&c, Scope::All { system: false }),
            Some(30)
        );
        assert_eq!(
            b.scope_bytes_when_done(&c, Scope::All { system: true }),
            None
        );
    }

    #[test]
    fn selection_counts_and_sets_per_scope_and_respects_root() {
        let mut cats = vec![
            cat("A", &["a1", "a2"]),
            cat("B", &["b1"]),
            cat("Sys", &["s1"]),
        ];
        cats[2].items[0].requires_root = true;
        let all_user = Scope::All { system: false };
        assert_eq!(all_user.selected_count(&cats), 0);
        assert_eq!(all_user.set_selected(&mut cats, true, false), 3);
        assert_eq!(all_user.selected_count(&cats), 3);
        // Setting again changes nothing.
        assert_eq!(all_user.set_selected(&mut cats, true, false), 0);
        // Root cleaners need `can_root`.
        let sys = Scope::Category(2);
        assert_eq!(sys.set_selected(&mut cats, true, false), 0);
        assert_eq!(sys.set_selected(&mut cats, true, true), 1);
        set_all_selected(&mut cats, false, true);
        assert_eq!(Scope::Category(0).selected_count(&cats), 0);
        assert_eq!(sys.selected_count(&cats), 0);
    }
}
