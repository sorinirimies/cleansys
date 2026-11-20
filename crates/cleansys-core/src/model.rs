//! Framework-agnostic domain model shared by the TUI and GUI front-ends.
//!
//! Nothing in this module depends on `ratatui`, `crossterm`, or `iced` — it is
//! pure application state that both front-ends render in their own way.

use crate::cleaners::cleaned_item::{CleanerFn, CleaningResult, cleaner_fn};
use crate::cleaners::{system_cleaners, user_cleaners};
use crate::engine;
pub use crate::engine::Risk;

/// The outcome of running (or attempting to run) a single cleaner.
#[derive(Debug, Clone)]
pub enum Status {
    /// The cleaner is queued but has not started yet.
    Pending,
    /// The cleaner is currently executing.
    Running,
    /// The cleaner finished successfully; the string is a human-readable summary.
    Success(String),
    /// The cleaner failed; the string is a human-readable error message.
    Error(String),
}

impl Status {
    /// Return a single-glyph representation of this status, using `frame` to
    /// select an animation frame for the `Running` state (spinner).
    pub fn get_animation_frame(&self, frame: usize) -> &'static str {
        match self {
            Status::Running => {
                const SPINNER: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
                SPINNER[frame % SPINNER.len()]
            }
            Status::Success(_) => "✓",
            Status::Error(_) => "✗",
            Status::Pending => "•",
        }
    }
}

/// A single selectable cleaning operation (e.g. "Browser Caches").
pub struct CleanerItem {
    /// Stable machine id (`proj-rust`, `core-user-trash`, ...), usable with
    /// `cleansys clean --id`.
    pub id: String,
    /// How costly it is to get back what this cleaner removes. Drives the
    /// "recommended" preset (only [`Risk::Safe`] cleaners) and UI badges.
    pub risk: Risk,
    /// Human-readable name of the cleaner.
    pub name: String,
    /// Short description of what the cleaner removes.
    pub description: String,
    /// Whether this cleaner needs root/administrator privileges to run.
    pub requires_root: bool,
    /// Whether the user has selected this cleaner to run.
    pub selected: bool,
    /// The function that performs the actual cleaning.
    /// Takes `skip_confirmation: bool` (when `true`, files are removed
    /// without an interactive y/n prompt — always `true` from the TUI/GUI,
    /// which have no stdin prompt loop) and returns the structured set of
    /// items actually removed, with real per-item sizes.
    pub function: CleanerFn,
    /// Bytes freed by the most recent run of this cleaner.
    pub bytes_cleaned: u64,
    /// Structured detail (per-file/per-directory paths and sizes) from the
    /// most recent run of this cleaner, if any.
    pub last_result: Option<CleaningResult>,
    /// Current run status, if the cleaner has been queued/run at least once.
    pub status: Option<Status>,
}

impl std::fmt::Debug for CleanerItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CleanerItem")
            .field("id", &self.id)
            .field("risk", &self.risk)
            .field("selected", &self.selected)
            .field("bytes_cleaned", &self.bytes_cleaned)
            .finish_non_exhaustive()
    }
}

/// A named group of related [`CleanerItem`]s (e.g. "User Land Cleaners").
#[derive(Debug)]
pub struct CleanerCategory {
    /// Category display name.
    pub name: String,
    /// Category description.
    pub description: String,
    /// The cleaners that belong to this category.
    pub items: Vec<CleanerItem>,
}

/// Build the default set of categories (User + System) with all known
/// cleaners loaded from [`user_cleaners`] and [`system_cleaners`].
///
/// This is shared between the TUI and GUI front-ends so both present the
/// exact same list of cleaners.
pub fn load_categories() -> Vec<CleanerCategory> {
    let (mut user, mut root) = spec_categories(engine::load_specs());

    for cleaner in user_cleaners::get_cleaners() {
        let item = CleanerItem {
            id: format!("core-user-{}", slug(cleaner.name)),
            risk: builtin_risk(cleaner.name),
            name: cleaner.name.to_string(),
            description: cleaner.description.to_string(),
            requires_root: false,
            selected: false,
            function: cleaner_fn(cleaner.function),
            bytes_cleaned: 0,
            last_result: None,
            status: None,
        };
        place(&mut user, builtin_category(cleaner.name, false), item);
    }
    for cleaner in system_cleaners::get_cleaners() {
        let item = CleanerItem {
            id: format!("core-sys-{}", slug(cleaner.name)),
            risk: builtin_risk(cleaner.name),
            name: cleaner.name.to_string(),
            description: cleaner.description.to_string(),
            requires_root: cleaner.requires_root,
            selected: false,
            function: cleaner_fn(cleaner.function),
            bytes_cleaned: 0,
            last_result: None,
            status: None,
        };
        if cleaner.requires_root {
            let name = format!("{}{ROOT_SUFFIX}", builtin_category(cleaner.name, true));
            place(&mut root, &name, item);
        } else {
            place(&mut user, builtin_category(cleaner.name, false), item);
        }
    }

    // Layout: every user-land category first, then every root category, each side in the
    // fixed order of `USER_ORDER` / `ROOT_ORDER` (unknown names last, first-seen order kept).
    sort_by_order(&mut user, USER_ORDER);
    sort_by_order(&mut root, ROOT_ORDER);
    user.extend(root);
    user
}

/// Display order of the user-land categories.
pub const USER_ORDER: &[&str] = &[
    "Everyday Junk",
    "Web Browsers",
    "Developer Caches",
    "Project Build Artifacts",
    "Containers & Virtualization",
    "AI & LLM Caches",
    "Applications",
    "Detected App Caches",
    "Games",
    "Installers & Backups",
    "Large & Old Files",
    "Privacy Traces",
];

/// Display order of the system (root) categories (names without [`ROOT_SUFFIX`]).
pub const ROOT_ORDER: &[&str] = &[
    "System Logs & Dumps",
    "Package Managers",
    "Updates & Snapshots",
    "Maintenance & Tuning",
];

fn sort_by_order(categories: &mut [CleanerCategory], order: &[&str]) {
    categories.sort_by_key(|c| {
        let base = c.name.trim_end_matches(ROOT_SUFFIX);
        order.iter().position(|o| *o == base).unwrap_or(order.len())
    });
}

/// Which category a hard-coded cleaner belongs to (the name is the display name, root
/// categories get [`ROOT_SUFFIX`] added by the caller).
fn builtin_category(cleaner: &str, root: bool) -> &'static str {
    match (root, cleaner) {
        (_, "Browser Caches") => "Web Browsers",
        (false, "Package Manager Caches") => "Developer Caches",
        (true, _) => "Maintenance & Tuning",
        (false, _) => "Everyday Junk",
    }
}

/// Add `item` to the category called `name`, creating it when it does not exist yet.
fn place(target: &mut Vec<CleanerCategory>, name: &str, item: CleanerItem) {
    let idx = if let Some(i) = target.iter().position(|c| c.name == name) {
        i
    } else {
        let base = name.trim_end_matches(ROOT_SUFFIX);
        let mut description = category_description(base).to_string();
        if name.ends_with(ROOT_SUFFIX) {
            description.push_str(" (requires root)");
        }
        target.push(CleanerCategory {
            name: name.to_string(),
            description,
            items: Vec::new(),
        });
        target.len() - 1
    };
    target[idx].items.insert(0, item);
}

/// Name of the flat user-land list shown by the front-ends.
pub const USER_AREA: &str = "User Land Cleaners";
/// Name of the flat system (root) list shown by the front-ends.
pub const SYSTEM_AREA: &str = "System Cleaners";

/// Does a [`load_categories`] category belong to the system (root) side?
pub fn is_system_category(name: &str) -> bool {
    name == SYSTEM_AREA || name.ends_with(ROOT_SUFFIX)
}

/// Suffix appended to the name of engine categories that need root, keeping
/// them visibly (and structurally) separate from user-land ones.
pub const ROOT_SUFFIX: &str = " (root)";

/// Group declarative [`engine::CleanerSpec`]s into UI categories, returning
/// `(user_land, root)`. A spec category that contains both kinds is split in
/// two; the root half is named `"<category> (root)"`. First-seen order is
/// preserved within each half.
pub fn spec_categories(
    specs: Vec<engine::CleanerSpec>,
) -> (Vec<CleanerCategory>, Vec<CleanerCategory>) {
    let mut user: Vec<CleanerCategory> = Vec::new();
    let mut root: Vec<CleanerCategory> = Vec::new();
    for spec in specs {
        let target = if spec.requires_root {
            &mut root
        } else {
            &mut user
        };
        let display_name = if spec.requires_root {
            format!("{}{ROOT_SUFFIX}", spec.category)
        } else {
            spec.category.clone()
        };
        let idx = if let Some(i) = target.iter().position(|c| c.name == display_name) {
            i
        } else {
            let mut description = category_description(&spec.category).to_string();
            if spec.requires_root {
                description.push_str(" (requires root)");
            }
            target.push(CleanerCategory {
                name: display_name,
                description,
                items: Vec::new(),
            });
            target.len() - 1
        };
        let description = spec.display_description();
        let requires_root = spec.requires_root;
        let name = spec.name.clone();
        target[idx].items.push(CleanerItem {
            id: spec.id.clone(),
            risk: spec.risk,
            name,
            description,
            requires_root,
            selected: false,
            function: cleaner_fn(move |opts| engine::run_spec(&spec, opts)),
            bytes_cleaned: 0,
            last_result: None,
            status: None,
        });
    }
    (user, root)
}

fn slug(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Risk of the hard-coded (non-TOML) cleaners: irreversible or slow-to-undo
/// ones are `Moderate`, everything else is a regenerating cache (`Safe`).
fn builtin_risk(name: &str) -> Risk {
    match name {
        "Trash" | "Recycle Bin" | "Package Manager Caches" => Risk::Moderate,
        _ => Risk::Safe,
    }
}

/// Tick exactly the "recommended" cleaners: [`Risk::Safe`] and not needing
/// root (unless `include_root`). Everything else is unticked. Returns how
/// many cleaners are now selected.
pub fn select_recommended(categories: &mut [CleanerCategory], include_root: bool) -> usize {
    let mut n = 0;
    for item in categories.iter_mut().flat_map(|c| c.items.iter_mut()) {
        item.selected = item.risk == Risk::Safe && (include_root || !item.requires_root);
        n += usize::from(item.selected);
    }
    n
}

fn category_description(name: &str) -> &'static str {
    match name {
        "Developer Caches" => "Global caches of build tools, package managers and IDEs",
        "Project Build Artifacts" => {
            "Build output inside your projects (target/, build/, node_modules/, ...)"
        }
        "AI & LLM Caches" => "Model weights, agent caches and session histories",
        "Web Browsers" => "Cache, cookies, sessions and history of installed browsers",
        "Applications" => "Caches and logs of chat, media, productivity and creative apps",
        "Games" => "Shader caches, launcher caches and logs of game platforms",
        "Detected App Caches" => {
            "Cache folders of apps installed here that have no dedicated cleaner (found automatically)"
        }
        "Everyday Junk" => "Caches, temporary files, thumbnails and the trash",
        "Installers & Backups" => "Downloaded installers, OS update files and device backups",
        "Large & Old Files" => {
            "Big, forgotten files: old downloads, disk images, archives, duplicates"
        }
        "System Logs & Dumps" => "System logs, journals, crash reports and memory dumps",
        "Package Managers" => "System package caches, orphans and old revisions",
        "Updates & Snapshots" => "Downloaded OS updates, local snapshots and delivery caches",
        "Maintenance & Tuning" => "DNS, memory, font and system caches; routine maintenance tasks",
        "Containers & Virtualization" => "Docker, Podman, Flatpak, Snap and Nix garbage",
        "Privacy Traces" => "Recent-file lists, shell/REPL histories and similar traces",
        _ => "Additional cleaners",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn categories_are_themed_and_split_by_side() {
        let cats = load_categories();
        let first_root = cats
            .iter()
            .position(|c| c.name.ends_with(ROOT_SUFFIX))
            .expect("a system category exists");
        // User land first: nothing needs root, and no catch-all names remain.
        for c in &cats[..first_root] {
            assert!(c.items.iter().all(|i| !i.requires_root), "{}", c.name);
            assert!(!c.items.is_empty(), "{} is empty", c.name);
            assert!(!c.name.ends_with(ROOT_SUFFIX));
        }
        // System after: every category is labelled and root-only.
        for c in &cats[first_root..] {
            assert!(c.name.ends_with(ROOT_SUFFIX), "{}", c.name);
            assert!(c.items.iter().all(|i| i.requires_root), "{}", c.name);
        }
        for old in [
            "User Land Cleaners",
            "System Cleaners",
            "System Maintenance",
            "Deep Clean",
        ] {
            assert!(
                cats.iter().all(|c| !c.name.starts_with(old)),
                "{old} still a category"
            );
        }
        // Order follows USER_ORDER / ROOT_ORDER.
        let rank = |names: Vec<&str>, order: &[&str]| -> Vec<usize> {
            names
                .iter()
                .map(|n| order.iter().position(|o| o == n).unwrap_or(order.len()))
                .collect()
        };
        let user = rank(
            cats[..first_root].iter().map(|c| c.name.as_str()).collect(),
            USER_ORDER,
        );
        assert!(user.windows(2).all(|w| w[0] <= w[1]), "{user:?}");
        let root = rank(
            cats[first_root..]
                .iter()
                .map(|c| c.name.trim_end_matches(ROOT_SUFFIX))
                .collect(),
            ROOT_ORDER,
        );
        assert!(root.windows(2).all(|w| w[0] <= w[1]), "{root:?}");
    }

    #[test]
    fn every_category_is_a_known_theme_on_the_right_side() {
        for c in load_categories() {
            let base = c.name.trim_end_matches(ROOT_SUFFIX);
            if c.name.ends_with(ROOT_SUFFIX) {
                assert!(ROOT_ORDER.contains(&base), "unknown system category {base}");
            } else {
                assert!(USER_ORDER.contains(&base), "unknown user category {base}");
            }
        }
    }

    #[test]
    fn builtin_cleaners_land_in_their_theme() {
        let find = |id: &str| {
            load_categories()
                .into_iter()
                .find(|c| c.items.iter().any(|i| i.id == id))
                .map(|c| c.name)
        };
        assert_eq!(
            find("core-user-browser-caches").as_deref(),
            Some("Web Browsers")
        );
        assert_eq!(find("core-user-trash").as_deref(), Some("Everyday Junk"));
        assert_eq!(
            find("core-user-package-manager-caches").as_deref(),
            Some("Developer Caches")
        );
        assert_eq!(
            find("deep-large-files").as_deref(),
            Some("Large & Old Files")
        );
    }

    #[test]
    fn builtin_category_mapping() {
        assert_eq!(builtin_category("Trash", false), "Everyday Junk");
        assert_eq!(builtin_category("Browser Caches", false), "Web Browsers");
        assert_eq!(
            builtin_category("Package Manager Caches", false),
            "Developer Caches"
        );
        assert_eq!(
            builtin_category("System Caches", true),
            "Maintenance & Tuning"
        );
    }

    #[test]
    fn ids_are_unique_and_recommended_is_safe_only() {
        let mut cats = load_categories();
        let mut seen = std::collections::HashSet::new();
        for i in cats.iter().flat_map(|c| &c.items) {
            assert!(seen.insert(i.id.clone()), "duplicate id {}", i.id);
        }
        let n = select_recommended(&mut cats, false);
        assert!(n > 0);
        assert!(
            cats.iter()
                .flat_map(|c| &c.items)
                .filter(|i| i.selected)
                .all(|i| i.risk == Risk::Safe && !i.requires_root)
        );
    }

    #[test]
    fn status_animation_frames() {
        assert_eq!(Status::Pending.get_animation_frame(0), "•");
        assert_eq!(Status::Success("ok".into()).get_animation_frame(0), "✓");
        assert_eq!(Status::Error("bad".into()).get_animation_frame(0), "✗");
        let running = Status::Running;
        assert_eq!(running.get_animation_frame(0), "⠋");
        assert_eq!(running.get_animation_frame(1), "⠙");
    }
}
