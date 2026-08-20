# cleansys-gui

The Iced-based desktop GUI for [CleanSys](https://github.com/sorinirimies/cleansys).
Ships as the `cleansys-gui` binary.

Domain logic (cleaners, permission checks, formatting) is shared with
`cleansys-tui` via the [`cleansys-core`](../cleansys-core) crate — both
front-ends present the exact same set of cleaners and behave identically
under the hood.
