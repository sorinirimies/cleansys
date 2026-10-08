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
- **Pick exactly what goes** — expand any cleaner to see every path it would remove (each project's `target/`, each cache folder…) and untick the ones you want to keep, in the TUI, GUI and web UI. See [Details & fine-grained selection](#details--fine-grained-selection).
- **Settings & About in every front-end** — idle days, scan depth, scan folders, never-delete patterns, *hide empty* and *ask before cleaning* are all editable in the app (`o` in the TUI, ⚙ in the GUI, `/settings` on the web), and an **About** tab shows the version, the developer's GitHub and links. The version is always visible. See [Settings & About](#settings--about).
- **Idle-days selector** — choose in the app how long a project must be untouched before its build output is offered (any age … 90 days).
- **User land vs. root, kept separate** — everything that needs your password lives in its own *System · root* section; nothing asks for sudo until you run it.
- **Risk levels** — `safe`, `moderate` (slow to rebuild, marked `~`) and `caution` (model weights, chat history, marked `!`; never part of bulk/unattended runs).
- **Automatic cleaning** — daily/weekly/monthly via systemd/cron, launchd or Task Scheduler, configurable from the TUI, GUI or CLI.
- **Skips apps that are open**, never follows symlinks, refuses to touch `$HOME`, `/usr`, `Documents`, `.ssh`…, honours your exclusion globs.
- **Responsive everywhere** — the TUI, the GUI and the web UI share the same wide / medium / narrow behaviour (sidebar → narrower sidebar → drop-down + icon buttons); 43 colour themes in the GUI and web UI.
- **Same animated progress everywhere** — scans and cleans show a turning spinner (a round ring in the GUI and web, a braille spinner in the terminal; every cleaner and category that is still being measured gets its own) and a progress bar whose filled part has a highlight sweeping along it, with `n/total · percent`, in the TUI, the GUI and the web UI (CSS). The shared maths lives in [`cleansys_core::anim`](crates/cleansys-core/src/anim.rs); the GUI's animation only ticks while something is in progress, so an idle window costs nothing.
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
</tr><tr>
<td><b>Details — pick exactly which paths go</b> (and the <i>Idle ≥</i> day selector in the bar)<br><img src="demo/previews/web-details.png" width="430"></td>
<td><b>Settings</b><br><img src="demo/previews/web-settings.png" width="430"></td>
</tr><tr>
<td><b>About</b> — version, developer, GitHub<br><img src="demo/previews/web-about.png" width="430"></td>
<td></td>
</tr></table>

<p align="center"><img src="demo/previews/web-api.gif" alt="cleansys-web JSON API driven from nushell" width="800"></p>

### Terminal UI

| | |
|---|---|
| **Run it for real** — pick paths (`→`), confirm, live progress and the activity log *under the list*, summary, automatic re-scan<br><img src="demo/previews/tui-clean.gif" width="440"> | **Schedule it** — `S`<br><img src="demo/previews/tui-schedule.gif" width="440"> |
| **User land vs root** — root cleaners ask for your password only when run<br><img src="demo/previews/tui-system-root.gif" width="440"> | **Narrow terminals** — the sidebar collapses to a category bar<br><img src="demo/previews/tui-narrow.gif" width="440"> |
| **Settings & About** — `o` / `i`<br><img src="demo/previews/tui-settings.gif" width="440"> | |
| **Details** — `→` opens a cleaner: every path with its size, `Space` unticks one<br><img src="demo/previews/tui-details.png" width="440"> | **Done** — the outcome, freed space and activity log stay in the main view; `Esc` dismisses<br><img src="demo/previews/tui-clean-done.png" width="440"> |

### Web UI

`cleansys-web` is the same app in a browser tab — **no JavaScript bundle, no Node**: a [Topcoat](https://github.com/tokio-rs/topcoat)
server renders plain HTML, every action is a normal form post, and pages refresh themselves while a scan or a clean is
running.

```bash
cleansys-web                     # http://127.0.0.1:3000, opens your browser
cleansys-web --port 8080 --no-open --theme "Tokyo Night"
sudo cleansys-web                # optional: run as root (no password prompt needed)
```

- **Details**: *▸ Details* under a cleaner lists every path with a checkbox (*Select all* / *Select none*); the **Idle ≥** drop-down in the action bar sets the project idle-days threshold.
- **Settings & About**: `/settings` (idle days, scan depth, folders, exclusions, preferences) and `/about` (version, developer, GitHub); the version is on every page.
- **Pages**: `/` cleaners (sidebar, search, hide-empty, risk badges, sizes) · `/preview` dry run · `/confirm` → `/progress` real clean · `/schedule` automatic cleaning.
- **JSON**: `/api/categories` · `/api/status` · `/api/themes` · `/api/health`.
- **Responsive**: ≥ 980 px sidebar · ≥ 700 px narrower sidebar · below that a category drop-down, icon-only buttons and a full-width *Clean* button.
- **Safe by construction**: binds to `127.0.0.1` (warns loudly otherwise), rejects unknown `Host` headers (DNS-rebinding), uses Topcoat's built-in cross-origin protection for every state-changing route, only redirects to same-site paths, and applies the same protected-path / `caution` / running-app rules as the other front-ends. Root cleaners can be ticked; on a loopback bind, running them asks for your sudo password in the browser (kept in memory only for that clean, then wiped). On a non-loopback bind they stay disabled unless the server itself runs as root.

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

Pre-built packages (`.deb` and `.rpm` for the TUI, GUI and web UI, AppImage, Windows installer and zip, macOS `.dmg`, plus raw `cleansys-web-<target>` binaries) are on the [Releases](https://github.com/sorinirimies/cleansys/releases) page; only the latest version's files are kept. `just --list` shows all development tasks.

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
| `→` | **Expand** a cleaner to see every path it would remove; `↑/↓` move, `Space` ticks one path, `a` / `n` all / none, `←` / `Esc` back |
| `[` / `]` | **Idle days** for project build output (`target/`, `node_modules`, …) — fewer / more; saved, then re-scans |
| `r` | **Recommended** — tick the safe, user-land cleaners that have something to free |
| `a` / `n`, `A` / `N` | Tick / untick everything listed, in this category / everywhere |
| `/` | **Filter** across all categories (Enter keeps it, Esc clears) |
| `e` | Hide / show cleaners with nothing to clean |
| `d` | Preview — exact paths and sizes, deletes nothing |
| `Enter` | Run (asks for confirmation; `y` toggles the prompt). Progress, the outcome and the log appear **under the list** — the view never changes; `q` / `Esc` cancels, `Esc` dismisses the summary |
| `R` | Re-scan sizes |
| `S` | **Schedule** automatic cleaning |
| `L` | Show / hide the **activity log** under the list |
| `o` / `i` | **Settings** / **About** overlay (`Tab` switches; see [Settings & About](#settings--about)) |
| `?`, `Esc`, `q` | Help · back/cancel · quit |

Marks next to names: `~` moderate, `!` caution, `(root)` needs your password.

### Desktop GUI

- **Sidebar** (wide) / drop-down (narrow) of categories with live sizes; **USER LAND** above **SYSTEM · ROOT**.
- **Search box** filters every cleaner; **Hide empty cleaners** keeps the list short once the scan is done.
- **Action bar** is always visible: selection summary + progress, *Recommended*, *Preview*, *Rescan*, *Activity* log, and the big **Clean N · X GB** button.
- **Details**: each cleaner has a *▸ Details* button listing every path with a checkbox — untick the projects you want to keep. The **Idle ≥** drop-down in the action bar sets how long a project must be untouched before its build output is offered.
- **Window resizer** — on macOS the GUI installs the standard **Window** menu (Zoom, *Move & Resize*, Fill, Center, Full Screen Tile), which winit's default menu bar lacks and which the system's tiling shortcuts (e.g. ⌃⌘←) need — see [`macos.rs`](crates/cleansys-gui/src/macos.rs). It also has its own shortcuts: `⌘1`–`⌘4` Compact / Medium / Wide / Large (one per responsive layout), `⌘0` reset, `⌘+` / `⌘−` larger / smaller, `⌘⇧M` maximise, `⌃⌘F` or `F11` full screen (`Ctrl` instead of `⌘` on Windows/Linux). The same controls are in **⚙ Settings → Window**, and the size is remembered for the next launch.
- ⏰ **Schedule** opens the automatic-cleaning dialog. 43 themes (Catppuccin, Dracula, Nord, Tokyo Night…), remembered across restarts.
- Scans and runs happen on background threads — the window never freezes.
- **Details & day selector** — see [below](#details--fine-grained-selection).

### Settings & About

Every front-end has the same two screens, so you never have to edit `engine.json` by hand:

| | Settings | About |
|---|---|---|
| **TUI** | `o` — `↑/↓` move, `←/→` change a value, `Space` flip a switch / add an entry, `d` remove | `i` — `Enter` opens the highlighted link in your browser; `Tab` switches tabs, `Esc` closes |
| **GUI** | **⚙ Settings** in the header | **ℹ About** in the header (links are buttons) |
| **Web** | **⚙ Settings** → `/settings` | **ℹ About** → `/about` |

**Settings**

| Setting | What it does | Stored in |
|---|---|---|
| Project idle for at least | how long a project must be untouched before its build output is offered | `engine.json` (`min_age_days`) |
| Scan depth | how many folders below each root are searched for projects | `engine.json` (`max_depth`) |
| Project folders | the roots scanned for build output (add / remove; `~` and `$VAR` work) | `engine.json` (`scan_roots`) |
| Never delete | glob patterns that are never touched | `engine.json` (`exclude`) |
| Window (GUI) | size preset / larger / smaller / maximise / full screen / reset — the last size is restored on launch | `settings.json` (`window_size`) |
| Hide cleaners with nothing to clean | tidy list after a scan | `settings.json` |
| Ask before cleaning | the confirmation step (the web UI then posts straight to *Run*) | `settings.json` |

Changing a scan setting saves it and re-measures. Invalid input (a folder that doesn't exist, a bad glob) is refused with a message.

**About** shows the version, developer ([Sorin Albu-Irimies](https://github.com/sorinirimies)), GitHub profile, repository, bug tracker, downloads, license, platform and the paths of the two config files. The version (`vX.Y.Z`) is also shown in the TUI title bar, the GUI header and on every web page.

### Details & fine-grained selection

Every front-end can open a cleaner to show **each path it would remove, with its size**, and lets you
untick individual paths. The classic case is *Rust target/ Directories*: expand it, see
`stem-mqtt 44 GB · synthazia 40 GB · …`, and keep the project you're still building.

| | Expand | Pick paths |
|---|---|---|
| **TUI** | `→` on a cleaner | `↑/↓` move · `Space` tick one · `a` / `n` all / none · `←` / `Esc` back |
| **GUI** | **▸ Details** on the row | checkbox per path · *Select all* / *Select none* |
| **Web** | **▸ Details** under the row | checkbox per path · *Select all* / *Select none* |

- The *"X selected"* size and the **Clean N · X GB** button count only the ticked paths; **Preview** and **Run** both leave unticked paths alone.
- Each cleaner lists its 300 biggest paths (the rest are noted as *"… and N smaller not listed"*).
- Cleaners that run a command (e.g. `docker prune`) show their paths for information but are removed as a whole — the list says so.
- Selections are for the current scan; a re-scan resets them.

**Idle-days selector.** Project build output (`target/`, `node_modules`, …) is only offered once its
project has been untouched for *N* days (default 14) — so a category can look empty while you're actively
working. Change it right in the app, no config file needed: **TUI** `[` / `]`, **GUI** and **web** the
*Idle ≥* drop-down in the action bar (any age, 1, 3, 7, 14, 30, 60, 90 days). It's saved to `engine.json`
(`min_age_days`) and triggers a re-scan.

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

`min_age_days` can also be changed live from the TUI (`[` / `]`), GUI and web UI (*Idle ≥* selector).

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
