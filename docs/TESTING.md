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
- `model::tests::every_category_is_a_known_theme_on_the_right_side`: root cleaners need a system category.
- TUI tests index settings rows by position: adding a row shifts `settings_cursor` values in `app_tests.rs`.

## Platform coverage

CI runs the gate on Linux and tests on macOS and Windows (`gh workflow run ci.yml -f full_matrix=true`).
Linux/Windows cleaner paths are type-checked and unit-tested there, but are only exercised against
real system folders on the developer's machine.
