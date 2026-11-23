# cleansys-core

Shared, framework-free core logic for [CleanSys](https://github.com/sorinirimies/cleansys):
system/user cleaner implementations, the domain model (`CleanerItem`,
`CleanerCategory`, `Status`), sudo authentication helpers, a 43-preset UI
theme catalogue (`AppTheme`, shared by the GUI), JSON settings persistence,
and small utility functions (permission checks, byte-size formatting,
confirmation prompts).

This crate has no TUI (`ratatui`/`crossterm`) or GUI (`iced`) dependencies —
it is reused as-is by both `cleansys-tui` and `cleansys-gui`.

## Cleaner catalog

~370 built-in cleaners are declarative TOML under `src/engine/builtin/` (embedded at
compile time, overridable by id from `~/.config/cleansys/cleaners.d/`). `browsers.toml`,
`apps.toml` and `games.toml` are **generated** by `scripts/gen_content_packs.nu`; hand-written
browser cleaners (autofill, passwords, site preferences, Ladybird) live in `browsers_extra.toml`.

Catalog rules are enforced by tests in `engine/registry.rs`: no two cleaners delete the same
path, credential cleaners are `caution` and refuse while the browser runs, age-gated cleaners
are never `safe`, no template can match a whole home folder, and project artifacts stay inside
their project. Environment knobs: `CLEANSYS_SCAN_ROOTS`, `CLEANSYS_DEEP_ROOTS`,
`CLEANSYS_SCAN_THREADS`, and `CLEANSYS_FS_ROOT` (re-roots literal absolute paths such as
`/var/log/*` — used by the demo fixture and tests to fake the system side).
