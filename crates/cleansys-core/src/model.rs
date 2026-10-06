//! Framework-agnostic domain model shared by the TUI and GUI front-ends.
//!
//! Nothing in this module depends on `ratatui`, `crossterm`, or `iced` — it is
//! pure application state that both front-ends render in their own way.

use crate::cleaners::cleaned_item::{cleaner_fn, CleanerFn, CleaningResult};
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

/// A named group of related [`CleanerItem`]s (e.g. "User Land Cleaners").
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
    let mut user_items = Vec::new();
    for cleaner in user_cleaners::get_cleaners() {
        user_items.push(CleanerItem {
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
        });
    }

    let mut system_items = Vec::new();
    for cleaner in system_cleaners::get_cleaners() {
        system_items.push(CleanerItem {
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
        });
    }

    let (spec_user, spec_root) = spec_categories(engine::load_specs());

    // Layout (kept deliberately): every user-land category first, then every
    // root/administrator category — so the UI's "user land vs system" split
    // is preserved no matter how many cleaners the engine adds.
    let mut categories = vec![CleanerCategory {
        name: "User Land Cleaners".to_string(),
        description: "Clean user-specific files and caches".to_string(),
        items: user_items,
    }];
    categories.extend(spec_user);
    categories.push(CleanerCategory {
        name: "System Cleaners".to_string(),
        description: "Clean system files and caches (requires root)".to_string(),
        items: system_items,
    });
    categories.extend(spec_root);
    categories
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
        let idx = match target.iter().position(|c| c.name == display_name) {
            Some(i) => i,
            None => {
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
            }
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
        "Trash"
        | "Recycle Bin"
        | "Old Kernels"
        | "Windows Update Cache"
        | "Package Manager Caches" => Risk::Moderate,
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
        "System Maintenance" => {
            "OS-level caches, logs, crash dumps and update leftovers (often needs root)"
        }
        "Containers & Virtualization" => "Docker, Podman, Flatpak, Snap and Nix garbage",
        "Privacy Traces" => "Recent-file lists, shell/REPL histories and similar traces",
        _ => "Additional cleaners",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_categories_has_user_and_system() {
        let categories = load_categories();
        assert!(categories.len() >= 2);
        assert_eq!(categories[0].name, "User Land Cleaners");
        assert!(categories.iter().any(|c| c.name == "System Cleaners"));
        assert!(!categories[0].items.is_empty());
        let system = categories
            .iter()
            .find(|c| c.name == "System Cleaners")
            .unwrap();
        assert!(!system.items.is_empty());
        // System cleaners each declare their own root requirement (e.g.
        // Homebrew on macOS must not run as root), so not every item in the
        // System category necessarily requires root — but user cleaners never do.
        assert!(categories[0].items.iter().all(|i| !i.requires_root));
    }

    #[test]
    fn user_land_precedes_root_and_groups_are_homogeneous() {
        let cats = load_categories();
        let first_root = cats
            .iter()
            .position(|c| c.name == "System Cleaners")
            .expect("System Cleaners present");
        // Everything before "System Cleaners" is user-land and root-free.
        for c in &cats[..first_root] {
            assert!(
                c.items.iter().all(|i| !i.requires_root),
                "{} has root items",
                c.name
            );
        }
        // Engine categories after it are root-only and clearly labelled.
        for c in &cats[first_root + 1..] {
            assert!(c.name.ends_with(ROOT_SUFFIX), "{}", c.name);
            assert!(c.items.iter().all(|i| i.requires_root), "{}", c.name);
        }
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
        assert!(cats
            .iter()
            .flat_map(|c| &c.items)
            .filter(|i| i.selected)
            .all(|i| i.risk == Risk::Safe && !i.requires_root));
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
