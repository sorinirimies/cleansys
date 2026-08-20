# cleansys-core

Shared, framework-free core logic for [CleanSys](https://github.com/sorinirimies/cleansys):
system/user cleaner implementations, the domain model (`CleanerItem`,
`CleanerCategory`, `Status`), sudo authentication helpers, and small utility
functions (permission checks, byte-size formatting, confirmation prompts).

This crate has no TUI (`ratatui`/`crossterm`) or GUI (`iced`) dependencies —
it is reused as-is by both `cleansys-tui` and `cleansys-gui`.
