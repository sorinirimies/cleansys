//! Serde types describing a declarative cleaner (see [`crate::engine`]).

use serde::{Deserialize, Serialize};

/// A TOML file: zero or more `[[cleaner]]` tables.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SpecFile {
    #[serde(default, rename = "cleaner")]
    pub cleaners: Vec<CleanerSpec>,
}

/// How costly it is to get back what a cleaner removes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Risk {
    /// Regenerated automatically and cheaply (caches, build output).
    #[default]
    Safe,
    /// Regenerated, but takes noticeable time/bandwidth (dependency caches).
    Moderate,
    /// Large downloads or user data (LLM models, session transcripts).
    Caution,
}

/// One selectable cleaner (e.g. "Gradle Caches").
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CleanerSpec {
    /// Unique, stable id. A user spec with the same id replaces the built-in.
    pub id: String,
    pub name: String,
    pub description: String,
    /// Category shown in the UI (e.g. "AI & LLM Caches").
    #[serde(default = "default_category")]
    pub category: String,
    /// Restrict to these OSes (`linux`, `macos`, `windows`). Empty = all.
    #[serde(default)]
    pub os: Vec<String>,
    #[serde(default)]
    pub requires_root: bool,
    #[serde(default)]
    pub risk: Risk,
    #[serde(default, rename = "action")]
    pub actions: Vec<Action>,
}

fn default_category() -> String {
    "Other Caches".to_string()
}

/// A single step of a cleaner.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    /// Delete fixed paths. Supports `~`, `$VAR`, `${VAR:-default}`,
    /// `%VAR%` and glob patterns.
    Delete {
        /// Paths for every OS.
        #[serde(default)]
        paths: Vec<String>,
        #[serde(default)]
        linux: Vec<String>,
        #[serde(default)]
        macos: Vec<String>,
        #[serde(default)]
        windows: Vec<String>,
        /// Remove the *children* of each match, keeping the directory.
        #[serde(default)]
        contents_only: bool,
        /// Recreate the (now empty) directory afterwards.
        #[serde(default)]
        recreate: bool,
        /// Remove via `sudo rm -rf` when not already root. Defaults to the
        /// cleaner's `requires_root`.
        #[serde(default)]
        sudo: Option<bool>,
        /// Label shown next to each removed item.
        #[serde(default)]
        label: Option<String>,
    },
    /// Run an external program (e.g. `docker system prune -f`). Skipped when
    /// the program is not on `PATH`. Never run in preview mode; instead the
    /// `measure` paths are reported. Args support the same templating as
    /// paths.
    Command {
        program: String,
        #[serde(default)]
        args: Vec<String>,
        /// Run through `sudo` when not already root.
        #[serde(default)]
        sudo: bool,
        /// Directories whose size is measured before/after the command to
        /// report the real bytes freed.
        #[serde(default)]
        measure: Vec<String>,
        #[serde(default)]
        label: Option<String>,
    },
    /// Find build output next to marker files inside the configured
    /// project roots (see [`crate::engine::EngineConfig`]).
    ProjectArtifacts {
        /// File names (or `*.ext` globs) identifying a project.
        markers: Vec<String>,
        /// Directory names, relative to the marker's directory, to remove.
        artifacts: Vec<String>,
        #[serde(default)]
        label: Option<String>,
    },
}

impl CleanerSpec {
    /// Whether this cleaner applies to the OS cleansys was built for.
    pub fn applies_to_current_os(&self) -> bool {
        self.os.is_empty() || self.os.iter().any(|o| o.eq_ignore_ascii_case(current_os()))
    }

    /// Description including a trailing risk note, for display.
    pub fn display_description(&self) -> String {
        match self.risk {
            Risk::Safe => self.description.clone(),
            Risk::Moderate => format!("{} [re-download/rebuild takes time]", self.description),
            Risk::Caution => format!(
                "{} [CAUTION: large downloads or user data]",
                self.description
            ),
        }
    }
}

/// `linux` / `macos` / `windows` (anything else: `other`).
pub const fn current_os() -> &'static str {
    if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        "other"
    }
}

impl Action {
    /// For `Delete`: the raw path templates that apply to this OS.
    pub fn delete_templates(&self) -> Vec<&str> {
        match self {
            Action::Delete {
                paths,
                linux,
                macos,
                windows,
                ..
            } => {
                let per_os = match current_os() {
                    "linux" => linux,
                    "macos" => macos,
                    "windows" => windows,
                    _ => return paths.iter().map(String::as_str).collect(),
                };
                paths
                    .iter()
                    .chain(per_os.iter())
                    .map(String::as_str)
                    .collect()
            }
            _ => Vec::new(),
        }
    }
}
