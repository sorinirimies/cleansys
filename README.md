# CleanSys - Modern System Cleaner (Linux, macOS, Windows)

[![Crates.io](https://img.shields.io/crates/v/cleansys)](https://crates.io/crates/cleansys)
[![Documentation](https://docs.rs/cleansys/badge.svg)](https://docs.rs/cleansys)
[![TUI Downloads](https://img.shields.io/crates/d/cleansys?label=TUI%20downloads)](https://crates.io/crates/cleansys)
[![GUI Downloads](https://img.shields.io/crates/d/cleansys-gui?label=GUI%20downloads)](https://crates.io/crates/cleansys-gui)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Release](https://github.com/sorinirimies/cleansys/actions/workflows/release.yml/badge.svg)](https://github.com/sorinirimies/cleansys/actions/workflows/release.yml)
[![CI](https://github.com/sorinirimies/cleansys/actions/workflows/ci.yml/badge.svg)](https://github.com/sorinirimies/cleansys/actions/workflows/ci.yml)

**CleanSys** finds and removes what quietly eats your disk — browser and app caches, package-manager and
build-tool caches, `target/` and `node_modules/` folders in old projects, AI/LLM model caches, logs and crash dumps —
from a **terminal UI**, a **desktop GUI**, a **local web UI** or the **command line**. It shows exactly how much each cleaner can free
*before* you delete anything, never touches system folders, and can run **automatically on a schedule**.

<p align="center">
  <img src="demo/previews/tui-overview.gif" alt="CleanSys terminal UI: live sizes, recommended preset, preview and search" width="900">
</p>

## ✨ Highlights

- **See before you delete** — every cleaner is measured in the background on start-up; sizes appear per cleaner and per category, biggest first. Preview (`d` / 🔍) lists the exact paths.
- **One-key smart selection** — `r` / ✨ *Recommended* ticks only the safe, user-land cleaners that actually have something to free.
- **~190 cleaners** — browsers, chat/media apps, IDEs, Gradle/Android, Cargo, npm, Docker, Hugging Face/Ollama, Claude/Codex session data, OS logs and more. See [What it cleans](#-what-it-cleans).
- **Project build output** — finds `target/`, `build/`, `node_modules/`, `.venv/`… next to a marker file (`Cargo.toml`, `build.gradle.kts`, `package.json`…) and, by default, only in projects you haven't touched for 14 days.
- **User land vs. root, kept separate** — everything that needs your password lives in its own *System · root* section; nothing asks for sudo until you run it.
- **Risk levels** — `safe`, `moderate` (slow to rebuild, marked `~`) and `caution` (model weights, chat history, marked `!`; never part of bulk/unattended runs).
- **Automatic cleaning** — daily/weekly/monthly via systemd/cron, launchd or Task Scheduler, configurable from the TUI, GUI or CLI.
- **Skips apps that are open**, never follows symlinks, refuses to touch `$HOME`, `/usr`, `Documents`, `.ssh`…, honours your exclusion globs.
- **Responsive everywhere** — the TUI, the GUI and the web UI share the same wide / medium / narrow behaviour (sidebar → narrower sidebar → drop-down + icon buttons); 43 colour themes in the GUI and web UI.
- **Four front-ends, one engine** — TUI, GUI, [web UI (Topcoat)](crates/cleansys-web) and CLI all run the same cleaners, scan, scheduler and safety rules from `cleansys-core`.
- **Scriptable** — `--json` output, stable cleaner ids, exit codes; extend with your own TOML cleaners.

## 🎬 Demo

### Desktop GUI — responsive layouts

| Wide | Medium | Narrow |
|:---:|:---:|:---:|
| <img src="demo/previews/gui-wide.png" width="520"> | <img src="demo/previews/gui-medium.png" width="320"> | <img src="demo/previews/gui-narrow.png" width="190"> |

Sidebar with live sizes · search across every cleaner · risk badges · always-visible action bar ·
the sidebar becomes a drop-down on narrow windows.

<p align="center"><img src="demo/previews/gui-schedule.png" alt="GUI schedule dialog" width="560"></p>

### Web UI (Topcoat) — `cleansys-web`

| Wide | Medium | Narrow |
|:---:|:---:|:---:|
| <img src="demo/previews/web-wide.png" width="520"> | <img src="demo/previews/web-medium.png" width="320"> | <img src="demo/previews/web-narrow.png" width="190"> |

<table><tr>
<td><b>Tick → confirm → clean → progress</b><br><img src="demo/previews/web-flow.gif" width="430"></td>
<td><b>Same page, three widths</b><br><img src="demo/previews/web-responsive.gif" width="430"></td>
</tr><tr>
<td><b>Preview (dry run)</b><br><img src="demo/previews/web-preview.png" width="430"></td>
<td><b>Schedule</b><br><img src="demo/previews/web-schedule.png" width="430"></td>
</tr></table>

<p align="center"><img src="demo/previews/web-api.gif" alt="cleansys-web JSON API driven from nushell" width="800"></p>

### Terminal UI

| | |
|---|---|
| **Run it for real** — confirm → progress → freed → automatic re-scan<br><img src="demo/previews/tui-clean.gif" width="440"> | **Schedule it** — `S`<br><img src="demo/previews/tui-schedule.gif" width="440"> |
| **User land vs root** — root cleaners ask for your password only when run<br><img src="demo/previews/tui-system-root.gif" width="440"> | **Narrow terminals** — the sidebar collapses to a category bar<br><img src="demo/previews/tui-narrow.gif" width="440"> |

### Web UI

`cleansys-web` is the same app in a browser tab — **no JavaScript bundle, no Node**: a [Topcoat](https://github.com/tokio-rs/topcoat)
server renders plain HTML, every action is a normal form post, and pages refresh themselves while a scan or a clean is
running.

```bash
cleansys-web                     # http://127.0.0.1:3000, opens your browser
cleansys-web --port 8080 --no-open --theme "Tokyo Night"
sudo cleansys-web                # also enables the System · root cleaners
```

- **Pages**: `/` cleaners (sidebar, search, hide-empty, risk badges, sizes) · `/preview` dry run · `/confirm` → `/progress` real clean · `/schedule` automatic cleaning.
- **JSON**: `/api/categories` · `/api/status` · `/api/themes` · `/api/health`.
- **Responsive**: ≥ 980 px sidebar · ≥ 700 px narrower sidebar · below that a category drop-down, icon-only buttons and a full-width *Clean* button.
- **Safe by construction**: binds to `127.0.0.1` (warns loudly otherwise), rejects unknown `Host` headers (DNS-rebinding), uses Topcoat's built-in cross-origin protection for every state-changing route, only redirects to same-site paths, and applies the same protected-path / `caution` / running-app rules as the other front-ends. Root cleaners are shown but disabled unless the server itself runs as root.

### Command line

<p align="center"><img src="demo/previews/cli.gif" alt="cleansys auto, scan --json and schedule help" width="900"></p>

> Every demo runs against a synthetic home directory ([`demo/fixture.nu`](demo/fixture.nu)) — no real paths, nothing deleted. See [`demo/previews`](demo/previews/README.md).

## 🧱 Project Structure

CleanSys is a Cargo workspace with four crates:

| Crate | Binary | Description |
|-------|--------|-------------|
| [`cleansys-core`](crates/cleansys-core) | *(library)* | Framework-free logic shared by both front-ends: cleaner engine, scan board, scheduler, safety rules, sudo auth |
| [`cleansys-tui`](crates/cleansys-tui) | `cleansys` | Ratatui terminal UI **and** the CLI — published to crates.io as `cleansys` |
| [`cleansys-gui`](crates/cleansys-gui) | `cleansys-gui` | Iced desktop GUI |
| [`cleansys-web`](crates/cleansys-web) | `cleansys-web` | Local web UI built with [Topcoat](https://github.com/tokio-rs/topcoat) (needs Rust 1.98+) |

## 📦 Installation

```bash
cargo install cleansys           # terminal UI + CLI
cargo install cleansys-gui       # desktop GUI
cargo install cleansys-web       # local web UI (Rust 1.98+)
```

From source:

```bash
git clone https://github.com/sorinirimies/cleansys && cd cleansys
cargo build --workspace --release          # binaries in target/release/
cargo install --path crates/cleansys-tui   # or crates/cleansys-gui
```

Pre-built packages (`.deb`, `.rpm`, AppImage, Windows installer, macOS `.dmg`) are on the [Releases](https://github.com/sorinirimies/cleansys/releases) page. `just --list` shows all development tasks.

## 🚀 Quick start

```bash
cleansys                 # interactive TUI — press r, then Enter
cleansys-gui             # desktop GUI
cleansys-web             # web UI on http://127.0.0.1:3000 (opens your browser)
cleansys auto            # no UI: scan the recommended set, show it, ask once, clean
cleansys auto -n         # …only report (dry run)
sudo cleansys            # also unlock the system (root) cleaners
```

## 🧹 What it cleans

Cleaners are grouped into categories; the UIs list **user-land categories first, then a separate System · root section**.

| Category | Examples |
|----------|----------|
| **User Land Cleaners** | Browser/app/thumbnail caches, temp files, package-manager caches, Trash |
| **Developer Caches** | Gradle (caches, wrapper), Android SDK & Studio, Maven, Cargo git/src, sccache, npm/yarn/pnpm/bun, Go, uv/Poetry, Bazel, Playwright/Cypress, VS Code/Cursor/Windsurf, Xcode, Ruby, NuGet, ccache… |
| **Project Build Artifacts** | Rust `target/`, Gradle/Android `build/` + `.gradle/` + `.cxx`, Maven, `node_modules`, Next/Nuxt/Vite caches, Python caches & venvs, SwiftPM, CocoaPods, .NET `bin/obj`, Flutter, Zig, CMake, Terraform |
| **AI & LLM Caches** | Hugging Face, PyTorch/Keras, Ollama, LM Studio/GPT4All/Jan, Claude Code, Codex/Gemini CLI, agent session histories, code-index caches |
| **Web Browsers** | Cache, cookies & site data, history, sessions — Chrome, Chromium, Brave, Edge, Vivaldi, Opera, Arc, Firefox, LibreWolf, Zen, Safari |
| **Applications / Games** | Slack, Discord, Teams, Signal, Spotify, Zoom, Telegram, Office, Adobe, JetBrains logs · Steam shader cache, Epic, Lutris/Heroic, Wine, GPU shader caches |
| **System Maintenance** | Linux: APT lists/autoremove, rotated logs, journal vacuum, core dumps, snap/flatpak · macOS: DNS flush, Quick Look, update leftovers, Time Machine snapshots, iOS backups · Windows: Prefetch, WER, CBS logs, thumbnail cache, DISM cleanup |
| **Containers & Virtualization** | Docker/Podman prune, Flatpak unused runtimes, Nix GC, Conda, Kubernetes/Helm caches, Vagrant boxes |
| **Privacy Traces** | Recent-file lists, shell/REPL/editor histories |
| **System Cleaners** *(root)* | Package-manager caches, logs, old kernels, crash reports, Homebrew/Xcode/Simulator (macOS), Windows Update cache, Recycle Bin |

Run `cleansys list` for every cleaner and its id. Cleaners for software you don't have are hidden automatically.

## 🖥️ The interfaces

### Terminal UI

A sidebar of categories (with sizes and tick counts) next to the cleaner list, a details box for the highlighted
cleaner, and a footer with what's selected and how much it would free. On terminals narrower than 90 columns the
sidebar collapses into a one-line category bar (Tab / Shift+Tab).

| Key | Action |
|-----|--------|
| `↑/↓`, `Tab` / `Shift+Tab` | Move through cleaners / categories (empty categories are skipped) |
| `Space` | Tick / untick |
| `r` | **Recommended** — tick the safe, user-land cleaners that have something to free |
| `a` / `n`, `A` / `N` | Tick / untick everything listed, in this category / everywhere |
| `/` | **Filter** across all categories (Enter keeps it, Esc clears) |
| `e` | Hide / show cleaners with nothing to clean |
| `d` | Preview — exact paths and sizes, deletes nothing |
| `Enter` | Run (asks for confirmation; `y` toggles the prompt) |
| `R` | Re-scan sizes |
| `S` | **Schedule** automatic cleaning |
| `c` `m` `v` `p` `s` | Chart type · compact · view mode · performance stats · auto-scroll log |
| `?`, `Esc`, `q` | Help · back/cancel · quit |

Marks next to names: `~` moderate, `!` caution, `(root)` needs your password.

### Desktop GUI

- **Sidebar** (wide) / drop-down (narrow) of categories with live sizes; **USER LAND** above **SYSTEM · ROOT**.
- **Search box** filters every cleaner; **Hide empty cleaners** keeps the list short once the scan is done.
- **Action bar** is always visible: selection summary + progress, *Recommended*, *Preview*, *Rescan*, *Activity* log, and the big **Clean N · X GB** button.
- ⏰ **Schedule** opens the automatic-cleaning dialog. 43 themes (Catppuccin, Dracula, Nord, Tokyo Night…), remembered across restarts.
- Scans and runs happen on background threads — the window never freezes.

### Command line

```bash
cleansys auto [-n] [-y] [--json]        # recommended set: scan → show → confirm once → clean
cleansys list                           # all cleaners, grouped like the UIs, with ids
cleansys scan   [selection] [--json]    # preview only
cleansys clean  [selection] [-n] [-y]   # run (prompts per item unless -y)
cleansys schedule install|show|remove|run-now
cleansys config show|add-root|remove-root|exclude|min-age
cleansys user|system [-y]               # the classic one-shot runners
cleansys menu                           # plain-text menu
```

*Selection* flags: `-i <id>` (repeatable) · `-c <category>` (repeatable; also `user` / `system`) · `-a/--all` ·
`-r/--recommended` · `--moderate` · `--include-caution`. `caution` cleaners are only included when you name them with
`-i` or pass `--include-caution`.

```bash
cleansys scan -c user                                  # all user-land categories
cleansys scan -c "AI & LLM Caches" --include-caution
cleansys clean -i proj-rust -i dev-gradle-caches -n    # dry run
cleansys scan --all --json | jq '.[] | select(.bytes>1e9) | .name'
```

## 📱 Responsive design

All three UIs use the same three tiers:

| Tier | TUI (columns) | GUI / web (px) | Layout |
|------|---------------|----------------|--------|
| **Wide** | ≥ 100 | ≥ 980 | category sidebar (with sizes, tick counts, USER LAND above SYSTEM · ROOT) + list + full action bar |
| **Medium** | 90 – 99 | 700 – 979 | narrower sidebar, compact action bar / footer |
| **Narrow** | < 90 | < 700 | no sidebar: a category drop-down / one-line category bar, compact footer, icon-only buttons with tooltips |

## ⏰ Automatic cleaning

```bash
cleansys schedule install --every weekly --day sun --at 03:30     # recommended (safe) scope
cleansys schedule install --every daily --at 02:00 --scope extended
cleansys schedule install --scope selected -i proj-rust -i dev-gradle-caches
cleansys schedule show        # when, what, active?, last run
cleansys schedule remove
```

| OS | Mechanism |
|----|-----------|
| Linux | systemd **user timer** (preferred) or your **crontab** (managed block, other lines untouched) |
| macOS | launchd agent in `~/Library/LaunchAgents` |
| Windows | Task Scheduler (`schtasks`) |

The job runs `cleansys auto --scheduled`, which reads `~/.config/cleansys/schedule.json` — so changing the *scope*
needs no reinstall. Unattended runs **only touch user-land cleaners, skip applications that are running, and never
include `caution` cleaners** unless you explicitly ticked them (scope *selected*). The result ("freed 3.2 GB,
12 cleaners, 1 skipped") is recorded and shown in the TUI/GUI schedule screen; the log is
`~/.config/cleansys/scheduled.log` (`~/Library/Logs/cleansys-scheduled.log` on macOS).

## 🧰 Configuration & custom cleaners

**Project scanning** looks in `~/Projects`, `~/dev`, `~/src`, `~/code`, … (auto-detected). Adjust with
`cleansys config` or `~/.config/cleansys/engine.json`:

```json
{ "scan_roots": ["~/work"], "max_depth": 6, "min_age_days": 14, "exclude": ["~/work/keep-me/**"] }
```

`min_age_days` (default **14**) means build output is only removed when nothing in the project changed for that
long — set `0` to disable. `CLEANSYS_SCAN_ROOTS` overrides `scan_roots` (path-list).

**Your own cleaners** are plain TOML in `~/.config/cleansys/cleaners.d/*.toml` (same `id` overrides a built-in):

```toml
[[cleaner]]
id = "my-app-cache"
name = "My App Cache"
description = "Cache of my app"
category = "My Cleaners"
risk = "safe"                      # safe | moderate | caution
process = ["myapp"]                # refuse to run while this process is open
[[cleaner.action]]
type = "delete"                    # delete | command | project_artifacts
paths = ["~/.myapp/cache"]         # ~, $VAR, ${VAR:-default}, %VAR%, globs
linux = ["$XDG_CACHE_HOME/myapp"]  # per-OS lists: linux / macos / windows
```

Other actions: `command` (`program`, `args`, `sudo`, `measure = [paths]` to report freed bytes) and
`project_artifacts` (`markers`, `artifacts`). `delete` also supports `contents_only`, `files_only`, and regex filters
(`name_regex`, `not_name_regex`, `path_regex`, `not_path_regex`). Set `requires_root = true` for anything that
needs elevation — it will appear in the System · root section.

## 🛡️ Safety

- Nothing is deleted without a **confirmation** (TUI/GUI/CLI) except in explicit `--yes` / scheduled runs; every action has a **dry-run** (`-n`, `d`, 🔍 Preview).
- **Protected paths** can never be removed: filesystem roots, system directories, `$HOME` and its ancestors, `Documents`, `Desktop`, `.ssh`, `.gnupg`, `.config`… Paths are absolute, `..` is rejected, **symlinks are never followed**.
- Your **exclusion globs** (`cleansys config exclude '<glob>'`) always win.
- **Running apps** (browsers, Slack, …) make a cleaner refuse instead of corrupting a live profile.
- Project build output needs a **marker file** next to it and (by default) **14 days of inactivity**.
- Every byte reported is **measured**, not estimated; failures surface as real errors, not silent zeros.

## 🖥️ Platform Support

Cleaners are **real and platform-native** on every OS — not just Linux paths
that happen to compile elsewhere. Each cleaner resolves OS-appropriate
locations (see [`cleansys-core::cleaners::platform`](crates/cleansys-core/src/cleaners/platform.rs))
and measures real freed bytes (no estimates).

| Platform | `cleansys` (TUI/CLI) | `cleansys-gui` | User cleaners | System cleaners |
|----------|:---:|:---:|-------|-------|
| Linux (x86_64, aarch64) | ✅ | ✅ | Firefox/Chrome/Chromium caches, `~/.cache`, thumbnails, `/tmp`, pip/npm/cargo caches, XDG Trash | apt/pacman/dnf caches, rotated logs + journald vacuum, `/var/cache`, old kernels, crash reports (root required) |
| macOS (Intel + Apple Silicon) | ✅ | ✅ | Firefox/Chrome/Safari caches, `~/Library/Caches`, QuickLook thumbnails, `$TMPDIR`, pip/npm/cargo caches, `~/.Trash` | Homebrew cache (**not** root — brew refuses to run as root), Xcode DerivedData, unavailable Simulator caches, rotated system logs, diagnostic/crash reports (root required) |
| Windows (x86_64) | ✅ | ✅ | Chrome/Edge/Firefox caches, Windows INetCache/CrashDumps, thumbnail cache, `%TEMP%`, pip/npm/cargo caches | Windows Update download cache, `C:\Windows\Temp` (Administrator), Recycle Bin (via Shell32, no Administrator needed for your own bin) |

Windowing / desktop-environment support for `cleansys-gui` (via [Iced](https://iced.rs)/winit):
- **Linux**: X11 and Wayland (both enabled by default)
- **macOS**: native Cocoa/AppKit windowing (Intel + Apple Silicon, universal binary in releases)
- **Windows**: native Win32 windowing

The terminal UI (`cleansys`, via [Ratatui](https://github.com/ratatui-org/ratatui)/[Crossterm](https://github.com/crossterm-rs/crossterm)) works in any terminal emulator on any of the three platforms.

See [Releases](https://github.com/sorinirimies/cleansys/releases) for pre-built binaries: Linux `.deb`/`.rpm`/AppImage, Windows NSIS installer, and a universal macOS `.dmg`.

## 🏗️ Architecture

```
crates/
├── cleansys-core/            # shared, UI-free
│   └── src/
│       ├── engine/           # declarative cleaners: spec (TOML) · exec · paths · safety · running guard ·
│       │   │                 #   schedule · headless (scan/clean/auto) · config
│       │   └── builtin/*.toml    # ~175 built-in cleaners (browsers/apps/games are generated by scripts/gen_content_packs.nu)
│       ├── cleaners/         # the original hard-coded user/system cleaners + platform paths
│       ├── model.rs          # CleanerItem (id, risk, root), categories, user-land/root layout, recommended preset
│       ├── scan.rs           # background scan pool + ScanBoard (sizes, visibility, filtering) used by TUI and GUI
│       ├── auth.rs · utils.rs · settings.rs · theme/
├── cleansys-tui/             # Ratatui TUI + CLI (binary `cleansys`)
├── cleansys-gui/             # Iced GUI (binary `cleansys-gui`): state · update · view (responsive layouts)
└── cleansys-web/             # Topcoat web UI (binary `cleansys-web`): state · pages · components · api · style
```

Adding a cleaner is usually just a TOML entry in `crates/cleansys-core/src/engine/builtin/` (or, for yourself, in
`~/.config/cleansys/cleaners.d/`). Repetitive packs (Chromium-family browsers, Electron apps, games) are generated:
edit `scripts/gen_content_packs.nu` and run `just gen-packs`. Hard-coded cleaners remain possible via the `cleaner!` macro in
`cleaners/user_cleaners.rs` / `system_cleaners.rs`; privileged ones should go through `execute_with_sudo`.

## 🧪 Testing

Run the test suite:

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# One crate
cargo test -p cleansys-core
cargo test -p cleansys-web        # end-to-end: real server on an ephemeral port, sandboxed HOME
```

`cleansys-web` needs Rust 1.98+ (Topcoat). Debug builds are kept small by `.cargo/config.toml` (line-tables-only debuginfo).

## 🎥 Demos & screenshots

Everything shown above is generated by **nushell scripts** (no bash/python) against a **synthetic home directory**
(`demo/fixture.nu`, sparse files — no real paths, nothing of yours is touched). See
[`demo/previews/README.md`](demo/previews/README.md).

```bash
just vhs-all              # re-record every terminal tape (demo/*.tape → demo/previews/*.gif)   needs vhs, ttyd
just gui-screenshots      # nu scripts/gui-screenshots.nu — GUI PNGs (macOS)
just web-screenshots      # nu scripts/web-screenshots.nu — web PNGs + GIFs (headless Chromium + ffmpeg)
just vhs-refresh-previews # all of the above
just gen-packs            # nu scripts/gen_content_packs.nu — regenerate the browser/app/game cleaner TOML
```

The preview files are tracked with **git-lfs** (`*.gif`, `demo/previews/*.png`).

## 🤖 Automated Maintenance

A nightly Gitea Actions job (`.gitea/workflows/deps-update.yml`) upgrades every
workspace dependency pin, runs the full quality gate (fmt, clippy, test), and—
if anything actually changed—**automatically cuts a new patch release**: it
commits the dependency bump, then bumps the workspace version (`X.Y.Z` →
`X.Y.Z+1`), regenerates `CHANGELOG.md`, and pushes the tag directly to `main`.
No PR, no manual approval — pushing the tag triggers the normal Release
workflow (build, package, crates.io publish, AUR update). If the quality gate
fails at any point the job simply stops; nothing broken is ever released.

Run the same flow locally with `just auto-patch-release`.

## 🗺️ Roadmap

- Secure shredder / free-space wipe, duplicate and large-file finder, undo via a quarantine folder.
- `sqlite_vacuum` / JSON actions (e.g. browser databases), an exclusion editor in the UIs, run-history chart.
- Windows registry cleaning and more app definitions — contributions welcome (cleaners are just TOML).

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🔗 Links

- [Repository](https://github.com/sorinirimies/cleansys)
- [Crates.io](https://crates.io/crates/cleansys)
- [Documentation](https://docs.rs/cleansys)
