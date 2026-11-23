# cleansys

The Ratatui-based terminal user interface and CLI for
[CleanSys](https://github.com/sorinirimies/cleansys). Published to crates.io
as `cleansys` (directory name `cleansys-tui`). Ships as the `cleansys`
binary.

Domain logic (cleaners, permission checks, formatting) is shared with
`cleansys-gui` via the [`cleansys-core`](../cleansys-core) crate.

## Keys at a glance

`Tab`/`Shift+Tab` move between categories · `Space` ticks a cleaner (on the category list: every
cleaner in it) · `→` opens a cleaner's paths (top row **All**) · `d` preview · `Enter` run ·
`S` scan · `x` opt the cleaner out of / into scanning · `C` schedule · `/` filter · `?` help.
