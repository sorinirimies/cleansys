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
    /// Process names (case-insensitive, `.exe` ignored). If any is running,
    /// a real (non-preview) run is refused with a "close it first" error.
    #[serde(default)]
    pub process: Vec<String>,
    /// Lock-file globs (e.g. Firefox `~/.mozilla/firefox/*/lock`); if any
    /// exists the application is considered running.
    #[serde(default)]
    pub lock_files: Vec<String>,
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
        /// Walk each match recursively and remove only *files* (directories
        /// stay). Combine with the regex filters below.
        #[serde(default)]
        files_only: bool,
        /// Only items whose file name matches this regex.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name_regex: Option<String>,
        /// Skip items whose file name matches this regex.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        not_name_regex: Option<String>,
        /// Only items whose full path matches this regex.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        path_regex: Option<String>,
        /// Skip items whose full path matches this regex.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        not_path_regex: Option<String>,
        /// Only items neither modified nor read for this many days (0 = any age).
        #[serde(default)]
        min_age_days: u64,
        /// Judge `min_age_days` by the newest file *inside* each directory instead of
        /// the directory itself (a toolchain in daily use never touches its own
        /// directory, only the binaries below it).
        #[serde(default)]
        age_by_contents: bool,
        /// Remove via `sudo rm -rf` when not already root. Defaults to the
        /// cleaner's `requires_root`.
        #[serde(default)]
        sudo: Option<bool>,
        /// Label shown next to each removed item.
        #[serde(default, skip_serializing_if = "Option::is_none")]
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
        /// A read-only dry run of the same program, run as the current user in preview:
        /// every absolute path it prints (one per line) is what the real run would remove,
        /// and only those files are reported. Without it a preview can only report the
        /// whole `measure` folder, which is far too much for a command that removes just
        /// part of it (`paccache -ruk0` leaves every installed package's file behind).
        /// If the dry run fails, nothing is reported rather than guessing.
        #[serde(default)]
        preview_args: Vec<String>,
        /// Program for that dry run when it is not `program` itself (e.g. `sh` for a
        /// small script that lists what `journalctl --vacuum-size` would remove).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        preview_program: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        label: Option<String>,
    },
    /// Delete rows from tables of SQLite database files (browser cookies, site storage…)
    /// without removing the file itself. Uses the system `sqlite3` program and does nothing
    /// when it is missing. Preview reports the bytes of the affected rows; a real run
    /// deletes them and vacuums the file.
    SqliteRows {
        /// Database file templates (same templating as `delete` paths), per OS.
        #[serde(default)]
        paths: Vec<String>,
        #[serde(default)]
        linux: Vec<String>,
        #[serde(default)]
        macos: Vec<String>,
        #[serde(default)]
        windows: Vec<String>,
        /// Tables whose rows go. Names are plain identifiers (validated at load).
        tables: Vec<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        label: Option<String>,
    },
    /// Find build output next to marker files inside the configured
    /// project roots (see [`crate::engine::EngineConfig`]).
    ProjectArtifacts {
        /// File names (or `*.ext` globs) identifying a project.
        markers: Vec<String>,
        /// Directory names, relative to the marker's directory, to remove.
        artifacts: Vec<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        label: Option<String>,
    },
    /// Deep clean: find big and/or long-untouched files in the deep roots (your home
    /// folder unless `deep_roots` is configured). Every match is listed and can be
    /// unticked individually; nothing is searched outside the roots.
    LargeFiles {
        /// Folders to search instead of the configured deep roots (`~`, `$VAR` allowed).
        #[serde(default)]
        roots: Vec<String>,
        /// Only files at least this many MB.
        #[serde(default)]
        min_mb: u64,
        /// Only files not modified for this many days (0 = any age).
        #[serde(default)]
        min_age_days: u64,
        /// Only these extensions (no dot, case-insensitive). Empty = any file.
        #[serde(default)]
        extensions: Vec<String>,
        /// Only these file names: exact (`.DS_Store`), `*suffix` (`*~`) or `prefix*`
        /// (`._*`), case-insensitive. A file passes if it matches `extensions` or `names`.
        #[serde(default)]
        names: Vec<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        label: Option<String>,
    },
    /// Deep clean: files that exist twice or more (identical bytes) in the deep roots.
    /// One copy of each group — the oldest — is kept; the others are listed.
    Duplicates {
        #[serde(default)]
        roots: Vec<String>,
        /// Only files at least this many MB (default 1).
        #[serde(default = "one")]
        min_mb: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        label: Option<String>,
    },
}

fn one() -> u64 {
    1
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
        self.delete_templates_for(current_os())
    }

    /// Like [`Self::delete_templates`] for an explicit OS name (`linux`, `macos`, `windows`).
    pub fn delete_templates_for(&self, os: &str) -> Vec<&str> {
        match self {
            Action::Delete {
                paths,
                linux,
                macos,
                windows,
                ..
            }
            | Action::SqliteRows {
                paths,
                linux,
                macos,
                windows,
                ..
            } => {
                let per_os = match os {
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
