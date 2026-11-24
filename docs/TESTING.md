# Testing

```bash
just test            # cargo test --workspace
just check-all       # the quality gate: fmt, clippy -D warnings (pedantic), tests, docs, versions, machete, nu tests
just coverage        # cargo llvm-cov
cargo test -p cleansys-core            # one crate
cargo test -p cleansys-web             # real server on an ephemeral port, sandboxed HOME
```

Tests never touch your files: cleaners run against `tempfile` directories or a sandboxed `HOME`,
and nothing calls `sudo` for real.

## Where tests live

| Crate | Unit tests (in `src/`) | Integration tests (`tests/`) |
|---|---|---|
| `cleansys-core` | engine (spec parsing, exec filters incl. `min_age_days`, path resolution, protected paths, one owner per path across all built-in TOML, discovery), scan board (selection, cancel semantics, biggest-first), settings round-trip, schedule, themes | `utils_tests.rs`: `format_size`, `get_size`, `check_root` |
| `cleansys` (TUI/CLI) | app state, menu, dialog sizing/opacity, run worker | `app_tests.rs` (key handling, settings rows), `render_tests.rs` (TestBackend frames), `integration_tests.rs` (CLI via `assert_cmd`), `skip_run.rs`, `password_prompt_tests.rs` |
| `cleansys-gui` | state/update messages, view building | — |
| `cleansys-web` | pages, components, api | `server.rs` (HTTP end to end) |

## Guard rails worth knowing

- `engine::registry::tests::no_two_builtin_cleaners_delete_the_same_path`: two cleaners may not own one path.
- `builtin_delete_paths_are_never_protected_or_relative`: a delete path is never `/`, `/tmp`, `$HOME`, …
- `engine::registry::tests` catalog guards: credential cleaners are `caution` and refuse while the browser runs; `age_by_contents` cleaners need `min_age_days` and are never `safe`; no template can match a whole home/cache root; every delete cleaner has paths on each OS it claims; Ladybird uses the verified `Profiles/<id>/` layout; project artifacts are plain relative paths; the catalog must not shrink.
- Front-end parity: `scan_off_count` / `scan_off_note` / `scan_off_badge` live in `cleansys-core` (`scan/mod.rs` tests) and each UI is tested against them: TUI `category_panel_counts_cleaners_with_scanning_off…` (render_tests), GUI `view_does_not_panic_when_cleaners_have_scanning_off`, web `a_cleaner_with_scanning_off_is_called_out_on_the_page_in_shared_wording` (server.rs).
- TUI themes (`cleansys-tui/src/theme.rs`): the picker lists every `tui-nuances` palette; every shared core theme resolves to a real palette (three used to fall back to Default silently); saved names round-trip (shared names for GUI/web sync, slugs for TUI-only themes); cycling visits each theme once.
- `engine::exec::tests::unused_runtime_cleaner_keeps_selected_and_recent_versions_end_to_end`: TOML cleaner → preview → real run keeps the selected (`alias/default`, `current`, `bin/` shim) and recently used versions.
- `engine::exec::tests::sqlite_rows_*` / `sql_identifiers_*`: the `sqlite_rows` action previews the row payload, deletes only the named tables, keeps the file and other tables, skips missing tables/files/non-databases, and only splices plain identifiers into SQL (tests are skipped where `sqlite3` is not installed). Two cleaners may share one database file only with disjoint tables.
- `engine::deep::tests::names_and_extensions_are_alternatives`: deep-scan `names` / `extensions` matching.
- `browsers.toml`, `apps.toml`, `games.toml` are generated — add hand-written browser cleaners to `browsers_extra.toml`, never to the generated file (`just gen-packs` overwrites it).
- `model::tests::every_category_is_a_known_theme_on_the_right_side`: root cleaners need a system category.
- TUI tests index settings rows by position: adding a row shifts `settings_cursor` values in `app_tests.rs`.

## Platform coverage

CI runs the gate on Linux and tests on macOS and Windows (`gh workflow run ci.yml -f full_matrix=true`).
Linux/Windows cleaner paths are type-checked and unit-tested there, but are only exercised against
real system folders on the developer's machine.
