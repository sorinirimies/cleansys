//! Declarative cleaner engine.
//!
//! Cleaners are described as data (TOML) instead of hard-coded Rust.
//! Built-in definitions live in
//! `engine/builtin/*.toml` and are embedded at compile time; users can add or
//! override cleaners by dropping `*.toml` files into
//! `~/.config/cleansys/cleaners.d/`.
//!
//! Three action kinds exist (`delete`, `command`, `project_artifacts`):
//! * `delete` — remove fixed (glob / env-expanded) paths such as global
//!   caches (`~/.gradle/caches`, `~/.cache/huggingface`, ...).
//! * `project_artifacts` — scan project roots for build output next to a
//!   marker file (`Cargo.toml` → `target/`, `build.gradle.kts` → `build/`,
//!   `package.json` → `node_modules/`, ...).

pub mod config;
pub mod exec;
pub mod headless;
pub mod paths;
pub mod registry;
pub mod running;
pub mod safety;
pub mod schedule;
pub mod spec;

pub use config::EngineConfig;
pub use exec::run_spec;
pub use registry::{load_specs, parse_spec_file};
pub use spec::{Action, CleanerSpec, Risk};
