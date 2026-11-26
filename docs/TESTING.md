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
- Scan opt-in/out: the rules (cleaner > category > `scan_defaults.toml`) live in `Settings` (`cleansys-core/src/settings.rs`, tests there incl. one that every default-off category exists in the catalog); each front-end has its own toggle test (TUI `shift_x_opts_the_whole_category…` and `settings_start_slow_categories_off`, GUI `category_scan_toggle_switches_every_cleaner…`, web `category_scan_toggle_switches_all_its_cleaners`, CLI `scanning_lists_switches_a_category_off…`). A new front-end must build its disabled list with `Settings::scan_off_ids`, never from `disabled_cleaners` alone.
- Themes are one catalogue in `cleansys-core` (43 hand-tuned + 61 generated from `tui-nuances` into `theme/extra.rs`). `theme_tables_core_extras_match_tui_nuances` fails when the table drifts from the crate; regenerate with `UPDATE_THEMES=1 cargo test -p cleansys theme_tables`. Generated themes are checked for readable text/accents, and each front-end is tested against an extra theme (`Pip Boy`).
- TUI themes (`cleansys-tui/src/theme.rs`): the picker lists every `tui-nuances` palette; every shared core theme resolves to a real palette (three used to fall back to Default silently); saved names round-trip (shared names for GUI/web sync, slugs for TUI-only themes); cycling visits each theme once.
- `engine::exec::tests::unused_runtime_cleaner_keeps_selected_and_recent_versions_end_to_end`: TOML cleaner → preview → real run keeps the selected (`alias/default`, `current`, `bin/` shim) and recently used versions.
- `engine::exec::tests::sqlite_rows_*` / `sql_identifiers_*`: the `sqlite_rows` action previews the row payload, deletes only the named tables, keeps the file and other tables, skips missing tables/files/non-databases, and only splices plain identifiers into SQL (SQLite is built in; they run everywhere). Two cleaners may share one database file only with disjoint tables.
- `cleansys-tui/tests/cleaner_paths_tests.rs`: every plain home cleaner (≈180) on the OS the tests run on (Linux, macOS, Windows in CI): a file is planted at each of its path templates in a fake home (`CLEANSYS_HOME`, `LOCALAPPDATA`, `XDG_*`) and `cleansys scan --json` must report it. Catches templating, glob, OS-selection and protection bugs per platform — not whether a template matches where an app really stores its data.
- `cleansys-tui/tests/cli_tests.rs`: the CLI end to end in a fake home — scan/clean/auto JSON, dry runs, a real clean that keeps neighbouring files, usage, themes, config and schedule. Config and deleting tests are Linux-only (only there can the config folder be redirected). `config_changes_are_saved_and_shown` guards that a session's `CLEANSYS_SCAN_ROOTS` is never written to `engine.json`.
- Safety guards (`engine::exec::tests`): `committed_build_folders_are_never_offered` (git-tracked artifact folders), `pin_files_*` / `installed_folders_match_*` / `a_runtime_a_project_pins_*` (project-pinned runtime versions are kept).
- Scan switches: `scan::tests::switching_*` / `switched_off_cleaners_and_categories_sort_last` (core: no rescan, the running scan stops waiting, a switched-on cleaner is measured alone); TUI `switching_a_cleaner_off_does_not_rescan_*`, `shift_x_parks_the_category_*`, `a_parked_category_is_drawn_last_*`; GUI `scan_toggle_opts_a_cleaner_out_and_back_in_without_a_new_scan`; web `a_category_with_scanning_off_is_listed_last_in_its_section`.
- Scan speed/safety: `deep::tests::bundles_are_never_entered_and_os_trees_only_at_the_home_root` (deep scan skips `~/Library` on macOS, `AppData`/`OneDrive` on Windows, and document bundles such as `.photoslibrary`); `registry::tests::no_two_project_cleaners_measure_the_same_folder` (a folder next to a marker is sized and counted once).
- Command previews: `journal_preview_*` and `old_kernel_preview_*` run the real dry-run scripts against fake folders; `shell_scripts_in_command_cleaners_reach_the_shell_intact` and `paths::tests::shell_and_powershell_code_*` guard that `$1`, `$_` and `%s` in scripts are not mistaken for variables (they once emptied the snap and Windows event-log commands).
- Themes: `every_theme_keeps_text_status_and_chart_colours_readable` checks body/secondary text, hints, accent, selected-row text, status and chart colours of all 104 themes (core passes every palette through `AppTheme::readable`); TUI `the_selected_row_is_readable_in_every_theme` renders each theme.
- Scripts: `test_gen_content_packs.nu` (the generated packs match their generator; also a quality-gate step), `release_after_ci` / `retry_cancelled` decisions in `test_ci_scripts.nu`.
- `engine::exec::tests::command_preview_*` / `a_failing_or_silent_dry_run_*`: a command that removes only part of a folder previews just the files its dry run lists (not the whole folder), a failing or silent dry run previews 0, and a real run reports what was actually freed. Registry guard: pacman and snap must carry `preview_args`. GUI `when_the_last_cleaner_finishes_the_sizes_are_measured_again`: finishing a clean drops the old size and starts a rescan whose result replaces it.
- `engine::deep::tests::names_and_extensions_are_alternatives`: deep-scan `names` / `extensions` matching.
- `browsers.toml`, `apps.toml`, `games.toml` are generated — add hand-written browser cleaners to `browsers_extra.toml`, never to the generated file (`just gen-packs` overwrites it).
- `model::tests::every_category_is_a_known_theme_on_the_right_side`: root cleaners need a system category.
- TUI tests index settings rows by position: adding a row shifts `settings_cursor` values in `app_tests.rs`.

## Platform coverage

CI runs the gate on Linux and tests on macOS and Windows (`gh workflow run ci.yml -f full_matrix=true`).
Linux/Windows cleaner paths are type-checked and unit-tested there, but are only exercised against
real system folders on the developer's machine.

## Releasing

`just release X.Y.Z` runs the gate, bumps and tags locally, pushes the commit, **waits for
GitHub CI (Linux, macOS, Windows) to pass on it**, and only then pushes the tag that starts
`release.yml` (`scripts/ci/release_after_ci.nu`). Red, cancelled or timed-out CI pushes no
tag and removes the local one; after the fix is pushed, `just release-tag X.Y.Z` waits for
CI on HEAD and tags it. A Release run whose jobs a hosted runner cancelled is re-run
automatically, at most twice (`retry-release.yml`). The gate runs tests without
`CARGO_TARGET_DIR`, like CI, keeping the build cache through `CARGO_BUILD_TARGET_DIR`.
