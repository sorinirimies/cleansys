# CleanSys - Modern System Cleaner (Linux, macOS, Windows)

[![Crates.io](https://img.shields.io/crates/v/cleansys)](https://crates.io/crates/cleansys)
[![Documentation](https://docs.rs/cleansys/badge.svg)](https://docs.rs/cleansys)
[![TUI Downloads](https://img.shields.io/crates/d/cleansys?label=TUI%20downloads)](https://crates.io/crates/cleansys)
[![GUI Downloads](https://img.shields.io/crates/d/cleansys-gui?label=GUI%20downloads)](https://crates.io/crates/cleansys-gui)
[![Web Downloads](https://img.shields.io/crates/d/cleansys-web?label=Web%20downloads)](https://crates.io/crates/cleansys-web)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![CI](https://github.com/sorinirimies/cleansys/actions/workflows/ci.yml/badge.svg)](https://github.com/sorinirimies/cleansys/actions/workflows/ci.yml)
[![Audit](https://github.com/sorinirimies/cleansys/actions/workflows/security.yml/badge.svg)](https://github.com/sorinirimies/cleansys/actions/workflows/security.yml)
[![Fuzz](https://github.com/sorinirimies/cleansys/actions/workflows/fuzz.yml/badge.svg)](https://github.com/sorinirimies/cleansys/actions/workflows/fuzz.yml)

**CleanSys** finds and removes what quietly eats your disk — browser and app caches, package-manager and
build-tool caches, `target/` and `node_modules/` folders in old projects, AI/LLM model caches, logs and crash dumps —
from a **terminal UI**, a **desktop GUI**, a **local web UI** or the **command line**. It shows exactly how much each cleaner can free
*before* you delete anything, never touches system folders, and can run **automatically on a schedule**.

<p align="center">
  <img src="demo/previews/tui-overview.gif" alt="CleanSys terminal UI: live sizes, recommended preset, preview and search" width="900">
</p>

<p align="center">
  <a href="#-install"><b>Install</b></a> ·
  <a href="#-quick-start"><b>Quick start</b></a> ·
  <a href="#️-screenshots"><b>Screenshots</b></a> ·
  <a href="#-what-it-cleans"><b>What it cleans</b></a> ·
  <a href="#️-safety"><b>Safety</b></a> ·
  <a href="https://github.com/sorinirimies/cleansys/releases/latest"><b>Downloads</b></a>
</p>

## 📦 Install

Pick one — every option installs the terminal UI + command line (`cleansys`); the desktop app
(`cleansys-gui`) and the local web UI (`cleansys-web`) come the same way.

| Platform | Easiest | Download from the [latest release](https://github.com/sorinirimies/cleansys/releases/latest) |
|---|---|---|
| **macOS** (Apple Silicon + Intel) | `cargo install cleansys` | `cleansys-<version>-macos.dmg` — the desktop app plus the `cleansys` and `cleansys-web` commands; or `cleansys-<arch>-apple-darwin.tar.gz` |
| **Windows** (x86_64) | `cargo install cleansys` | `cleansys-<version>-windows-x86_64-setup.exe` installer; or the `.zip` per app |
| **Debian / Ubuntu** | `sudo apt install ./cleansys_<version>_amd64.deb` | `.deb` for `amd64` and `arm64`, one per app (`cleansys`, `cleansys-gui`, `cleansys-web`) |
| **Fedora / openSUSE** | `sudo dnf install ./cleansys-<version>-x86_64.rpm` | `.rpm` for `x86_64` and `aarch64` |
| **Arch Linux** | `makepkg -si` with the release's `PKGBUILD` ([below](#arch-linux)) | builds all three apps into one package |
| **Any Linux** | `chmod +x cleansys-<version>-x86_64.AppImage` and run it | AppImage (`x86_64`, `aarch64`) or `.tar.gz` |
| **Anywhere with Rust** | `cargo install cleansys cleansys-gui cleansys-web` | the web UI needs Rust 1.98+ |

Then run `cleansys` (or `cleansys-gui`, or `cleansys-web`) — see [Quick start](#-quick-start).
More options — building from source, Arch details — are in [Installation details](#-installation-details).

## 🚀 Quick start

```bash
cleansys                 # interactive TUI — press r, then Enter
cleansys-gui             # desktop GUI
cleansys-web             # web UI on http://127.0.0.1:3000 (opens your browser)
cleansys auto            # no UI: scan the recommended set, show it, ask once, clean
cleansys auto -n         # …only report (dry run)
sudo cleansys            # also unlock the system (root) cleaners
```

## 🖼️ Screenshots

Every screenshot runs on a synthetic home folder ([`demo/fixture.nu`](demo/fixture.nu)) — nothing real is shown or deleted.

<table>
<tr>
<td width="50%"><b>Web UI on a phone-sized window</b> — same app, narrow layout<br><img src="demo/previews/web-responsive.gif" alt="Web UI, responsive"></td>
<td width="50%"><b>Desktop GUI</b> — sidebar, sizes, risk badges<br><img src="demo/previews/gui-wide.png" alt="GUI"></td>
</tr>
<tr>
<td><b>Web UI</b> — the same app in a browser tab, no JavaScript bundle<br><img src="demo/previews/web-wide.png" alt="Web UI"></td>
<td><b>Command line</b> — <code>auto</code>, <code>scan --json</code>, schedule<br><img src="demo/previews/cli.gif" alt="CLI"></td>
</tr>
<tr>
<td><b>Clean for real</b> — confirm, live progress, summary<br><img src="demo/previews/tui-clean.gif" alt="TUI clean run"></td>
<td><b>Pick exact paths</b> — every path with its size<br><img src="demo/previews/tui-details.png" alt="TUI details"></td>
</tr>
<tr>
<td><b>User land vs root</b> — a password only when a root cleaner runs<br><img src="demo/previews/tui-system-root.gif" alt="TUI root cleaners"></td>
<td><b>Automatic cleaning</b> — daily, weekly or monthly<br><img src="demo/previews/tui-schedule.gif" alt="TUI schedule"></td>
</tr>
</table>

<details>
<summary><b>More screenshots</b> — responsive GUI and web layouts, web flow, settings, narrow terminals</summary>


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
| **Run it for real** — pick paths (`→`), confirm, live progress and the activity log *under the list*, summary, automatic re-scan<br><img src="demo/previews/tui-clean.gif" width="440"> | **Schedule it** — `C`<br><img src="demo/previews/tui-schedule.gif" width="440"> |
| **User land vs root** — root cleaners ask for your password only when run<br><img src="demo/previews/tui-system-root.gif" width="440"> | **Narrow terminals** — the sidebar collapses to a category bar<br><img src="demo/previews/tui-narrow.gif" width="440"> |
| **Settings & About** — `o` / `i`<br><img src="demo/previews/tui-settings.gif" width="440"> | |
| **Details** — `→` opens a cleaner: every path with its size, `Space` unticks one<br><img src="demo/previews/tui-details.png" width="440"> | **Done** — the outcome, freed space and activity log stay in the main view; `Esc` dismisses<br><img src="demo/previews/tui-clean-done.png" width="440"> |

### Web UI — how it works

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

</details>

## ✨ Highlights

- **Never wait for a scan** — a cleaner with a size can be cleaned right away: starting a clean stops the running scan (the sizes found so far stay), and a fresh scan starts by itself when the clean ends. Slow categories start unscanned and any category or cleaner can be switched on or off ([details](#choosing-what-is-scanned)); each cleaner can also be opted out (`x` in the TUI, `s` on the focused cleaner in the web UI, a *Scan* switch on every row in the GUI and web). Switching scanning off or on never starts a new scan: a switched-off cleaner drops its size and moves to the bottom of the list, a category whose cleaners are all off is parked at the end of its section (User land or System), dimmed, with a *◌ scan off* marker (`X` in the TUI on a category or cleaner, the category switch in the GUI and web), and switching back on measures only those cleaners. A cleaner that is switched off is never silent: its row says *scan off · …* with how to enable it, and the totals line counts them (*N cleaners have scanning off*) — the TUI, GUI and web UI share one wording from `cleansys-core`. The same *All* / *None* buttons tick a whole category in all three (`Space` on the category list in the TUI), and a cleaner's paths have *All paths* (TUI top row) / *Select all · Select none* (GUI, web). Scan: `S` (TUI, `R` too), `⌘/Ctrl+R` (GUI), `Shift+R` (web); stop the scan: `⌘/Ctrl+.` (GUI); the choice is saved. The menu has two sections, User land and System; each starts with an **All** entry listing every cleaner of that side in one flat list, followed by its categories.
- **See before you delete** — every cleaner is measured in the background, in parallel (one shared thread pool, sizes cached per scan, slowest cleaners first; tune with `scan_threads` in `settings.json` or `CLEANSYS_SCAN_THREADS`, default 2× cores), biggest first; preview (`d` / 🔍) lists exact paths.
- **~370 cleaners** — browsers, apps, IDEs, Gradle/Cargo/npm, Docker, AI/LLM caches, OS logs and more. See [What it cleans](#-what-it-cleans).
- **Project build output** — `target/`, `node_modules/`, `.venv/`… next to a marker file, by default only in projects idle for 14 days.
- **Smart selection** — `r` / ✨ *Recommended* ticks only safe, user-land cleaners with something to free; expand any cleaner to [untick single paths](#details--fine-grained-selection).
- **Risk levels** — `safe`, `moderate` (`~`), `caution` (`!`, never in bulk or unattended runs); root cleaners sit in a separate section and nothing asks for sudo until you run it.
- **Safe by design** — skips open apps, never follows symlinks, refuses `$HOME`, `/usr`, `.ssh`…, honours your exclusions. See [Safety](#️-safety).
- **Automatic cleaning** — daily/weekly/monthly via systemd/cron, launchd or Task Scheduler.
- **Four front-ends, one engine** — TUI, GUI, [web UI](crates/cleansys-web) and CLI share cleaners, scanner, scheduler and safety rules; the UIs share [Settings & About](#settings--about), responsive layouts and progress animation (104 themes in all three: the 43 hand-tuned ones plus every other palette of [`tui-nuances`](https://crates.io/crates/tui-nuances) — `cleansys --list-themes`, `T` in the TUI, `--theme "Pip Boy"`, the theme pick-list in the GUI and web. One shared name is saved in `settings.json`, so a theme picked in one front-end is the theme in the others).
- **Scriptable** — `--json`, stable cleaner ids, exit codes, your own cleaners in TOML.

## 🧹 What it cleans

Cleaners are grouped by theme; the UIs list a **User land** section first (everything that needs no password), then a **System · root** section. Each section starts with an **All** entry that lists every cleaner of that side. Homebrew, Xcode and Simulator cleanups live under *Developer Caches*; Browser Caches under *Web Browsers*.

| Category | Examples |
|----------|----------|
| **Everyday Junk** | App/thumbnail caches, temp files, Trash, Quick Look & icon caches, Explorer/WER/Internet caches, desktop-environment caches, user core dumps |
| **Developer Caches** | Gradle (caches, wrapper), Android SDK & Studio, Maven, Cargo git/src, sccache, npm/yarn/pnpm/bun, Go, uv/Poetry, Bazel, Playwright/Cypress, node-gyp/Expo, ESLint/Nx/Turbo, rustup temp + old pinned toolchains, shared `CARGO_TARGET_DIR`, **unused language runtimes** (nvm, fnm, volta, pyenv, uv, rbenv, rvm, sdkman, mise, asdf, ghcup, zvm, … — the selected version always stays), unused conda/virtualenvs, Hex, opam, PlatformIO/Arduino/ESP-IDF/Conan/Emscripten, JetBrains, Android SDK/emulators, Pulumi/Packer/SAM, Scala/Go/Kotlin/Swift tooling, Zed, VS Code/Cursor/Windsurf, Xcode, Ruby, NuGet, ccache… |
| **Project Build Artifacts** | ~55 ecosystems, found next to their marker file and only when idle: Rust `target/`, Gradle/Kotlin Multiplatform/Android, Maven, sbt/Mill/scala-cli, Haskell, Clojure, Elixir/Erlang, OCaml, Gleam, Elm, Crystal, D, `node_modules` + Astro/Nx/Vercel/Next/Nuxt/Vite output, React Native, Python caches/venvs/packaging, Ruby, PHP, Swift/Xcode/Tuist, CocoaPods, .NET, Visual Studio, C/C++ (Meson, clangd), ESP-IDF, PlatformIO, Unreal, Godot, Flutter, Zig, CMake, Terraform/CDK, Jekyll/Hugo/MkDocs/Sphinx, ML experiment output |
| **AI & LLM Caches** | Hugging Face, PyTorch/Keras, Ollama, LM Studio/GPT4All/Jan, Claude Code, Codex/Gemini CLI, agent session histories, code-index caches |
| **Detected App Caches** | Cache folders of apps installed here that have no dedicated cleaner (`~/.cache/*`, `~/Library/Caches/*`, `%LOCALAPPDATA%\*\Cache`): found automatically, one cleaner per app, moderate risk. Turn it off in Settings (next start) or with `CLEANSYS_NO_DISCOVER=1`. |
| **Web Browsers** | Cache, cookies & site data, history, sessions — Chrome, Chromium, Brave, Edge, Vivaldi, Opera, Arc, Firefox, LibreWolf, Zen, Safari, **Ladybird** (cache, history, sessions, crash reports, cookies, site data) · autofill, site preferences and saved passwords (caution, refused while the browser runs) |
| **Applications / Games** | Slack, Discord, Teams, Signal, Spotify, Zoom, Telegram, Office, Adobe, JetBrains logs · Steam shader cache, Epic, Lutris/Heroic, Wine, GPU shader caches |
| **Installers & Backups** | Installer files in `~/Downloads`, old *Install macOS* apps, iOS/iPadOS firmware & device backups, Mail downloads |
| **Large & Old Files** | Files over 500 MB, untouched for a year, old downloads, disk images, archives, dumps, duplicates (nothing ticked by default) |
| **Everyday Junk** *(deep scan)* | `.DS_Store` / `Thumbs.db` / `._*`, Vim swap files, `*~` / `*.bak` / `*.orig` backups |
| **Containers & Virtualization** | Docker/Podman prune, Flatpak unused runtimes, Nix GC, Conda, Kubernetes/Helm caches, Vagrant boxes |
| **Privacy Traces** | Recent-file lists, shell/REPL/editor histories, clipboard |
| **System Logs & Dumps** *(root)* | System logs, journal vacuum, rotated logs, crash reports, core dumps, ASL / setup / event logs, Defender history |
| **Package Managers** *(root)* | APT/DNF/pacman/zypper caches & orphans, old kernels, snap revisions, AppStream, DISM component cleanup |
| **Updates & Snapshots** *(root)* | Downloaded macOS updates, Time Machine local snapshots, Windows Update & Delivery Optimization caches |
| **Maintenance & Tuning** *(root)* | DNS flush, purge memory, font caches, Rosetta cache, periodic scripts, `fstrim`, print spool, hibernation file, Prefetch |

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
| `S` | **Scan** — measure every cleaner (`R` still works as an alias) |
| `x` | Opt the highlighted cleaner in / out of scanning (saved as `disabled_cleaners` in `settings.json`); a switched-off row says *scan off · x to enable* |
| `X` | Opt the whole category of the highlighted cleaner in / out of scanning (saved as `enabled_categories` / `disabled_categories`). Slow categories (project build folders, large and old files, app caches, games, …) start **off**; the list is `crates/cleansys-core/src/scan_defaults.toml`. Precedence: cleaner, then category, then default. The GUI and web UI have a *Scan* switch per category and per cleaner |
| `C` | **Schedule** automatic cleaning |
| `L` | Show / hide the **activity log** under the list |
| `o` / `i` | **Settings** / **About** overlay (`Tab` switches; see [Settings & About](#settings--about)) |
| `?`, `Esc`, `q` | Help · back/cancel · quit |

Marks next to names: `~` moderate, `!` caution, `(root)` needs your password.

### Desktop GUI

- **Sidebar** (wide) / drop-down (narrow) of categories with live sizes; **USER LAND** above **SYSTEM · ROOT**.
- **Search box** filters every cleaner; **Hide empty cleaners** keeps the list short once the scan is done.
- **Action bar** is always visible: selection summary + progress, *Recommended*, *Preview*, *Scan*, *Activity* log, and the big **Clean N · X GB** button.
- **Details**: each cleaner has a *▸ Details* button listing every path with a checkbox — untick the projects you want to keep. The **Idle ≥** drop-down in the action bar sets how long a project must be untouched before its build output is offered.
- **Activity panel** — fills the full width, its height is set by dragging the grip along its top edge (remembered across launches), and **Clear** lights up only when there is something to clear.
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
| Scan threads | worker threads for scans; lower on spinning disks / network mounts (default 2× cores) | `settings.json` (`scan_threads`), `CLEANSYS_SCAN_THREADS` |
| Hide cleaners with nothing to clean | tidy list after a scan | `settings.json` |
| Ask before cleaning | the confirmation step (the web UI then posts straight to *Run*) | `settings.json` |

Changing a scan setting saves it and re-measures. Invalid input (a folder that doesn't exist, a bad glob) is refused with a message.

**About** shows the version, developer ([Sorin Albu-Irimies](https://github.com/sorinirimies)), GitHub profile, repository, bug tracker, downloads, license, platform and the paths of the two config files. The version (`vX.Y.Z`) is also shown in the TUI title bar, the GUI header and on every web page.

### Choosing what is scanned

On a big disk a full scan takes a while, so the interactive UIs only measure the important categories at first and you opt in to the rest. Scanning only measures; it never deletes, and ticking or cleaning a cleaner works the same whether it was scanned or not.

| Front-end | Per cleaner | Per category |
|---|---|---|
| **TUI** | `x` | `X` |
| **GUI** | *Scan* switch on the row | *Scan* switch in the category header |
| **Web** | *Scan* switch on the row | *Scan category* switch in the header |
| **CLI** | `cleansys scanning on\|off <id>` | `cleansys scanning on\|off "<category>"` (`cleansys scanning` lists) |

- Precedence: the cleaner's own choice, then its category's, then the built-in default. Switching a whole category drops the per-cleaner choices inside it.
- Off by default: Project Build Artifacts, Large & Old Files, Applications, Games, Maintenance & Tuning, Privacy Traces, Installers & Backups (the list is [`scan_defaults.toml`](crates/cleansys-core/src/scan_defaults.toml)). Everything else, including your own categories, is on.
- Stored in `settings.json` as `disabled_cleaners`, `enabled_cleaners`, `disabled_categories`, `enabled_categories`; all three front-ends and the CLI share them.
- Explicit commands (`cleansys scan`, `clean`, `auto`, schedules) do exactly what you select and ignore these switches.

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
- A cleaner's checkbox and its paths stay in sync: opening an **unticked** cleaner shows all its paths unticked; ticking any one path ticks the cleaner, and unticking the last one unticks it.

**Idle-days selector.** Project build output (`target/`, `node_modules`, …) is only offered once its
project has been untouched for *N* days (default 14) — so a category can look empty while you're actively
working. Change it right in the app, no config file needed: **TUI** `[` / `]`, **GUI** and **web** the
*Idle ≥* drop-down in the action bar (any age, 1, 3, 7, 14, 30, 60, 90 days). It's saved to `engine.json`
(`min_age_days`) and triggers a re-scan.

### Command line

```bash
cleansys auto [-n] [-y] [--json]        # recommended set: scan → show → confirm once → clean
cleansys list                           # all cleaners, grouped like the UIs, with ids
cleansys scan   [selection] [--json]    # preview only (alias: `rescan` — run it any time, results are never cached)
cleansys clean  [selection] [-n] [-y] [--rescan]   # run (prompts per item unless -y); --rescan shows what's left
cleansys auto   [-y] [--rescan]         # same --rescan after the auto clean
cleansys scan|clean|auto -x <path>      # never touch a path (details-view "untick", repeatable)
cleansys schedule install|show|remove|run-now
cleansys config show|add-root|remove-root|exclude|min-age
cleansys scanning [on|off <category|id>]  # what the TUI/GUI/web scan (see Choosing what is scanned)
cleansys user|system [-y]               # the classic one-shot runners
cleansys menu                           # plain-text menu
```

*Selection* flags: `-i <id>` (repeatable) · `-c <category>` (repeatable; also `user` / `system`) · `-a/--all` ·
`-r/--recommended` · `--moderate` · `--include-caution` · `--saved` (the cleaners last ticked in the TUI/GUI/web).
`caution` cleaners are only included when you name them with `-i` or pass `--include-caution`.

*Exclusion*: `-x/--exclude <path>` (repeatable) keeps a path — and everything under it — out of the preview and the
run, the CLI equivalent of unticking a path in **Details**. Works with `scan`, `clean` and `auto`.

```bash
cleansys scan -c user                                  # all user-land categories
cleansys scan -c "AI & LLM Caches" --include-caution
cleansys clean -i proj-rust -i dev-gradle-caches -n    # dry run
cleansys clean --saved -x ~/code/keep-me/target -y     # last ticked cleaners, minus one project
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

`min_age_days` (default **14**): build output is only removed when nothing in the project changed for that
long — set `0` to disable. Change it live from the TUI (`[` / `]`), GUI or web UI (*Idle ≥* selector). `CLEANSYS_SCAN_ROOTS` overrides `scan_roots` (path-list).

### Deep clean (PC-wide, opt-in)

The **Large & Old Files** category finds big or forgotten files: files over 500 MB, large files untouched for a year, old downloads, disk images, archives, leftover installers and dumps. Every cleaner is `caution`/`moderate`, so nothing is ticked by default, never part of `--recommended`, and a scheduled run skips it unless you ask. Each file is listed and can be unticked (`→` / `v`), and `cleansys scan -c "Large & Old Files" --include-caution` previews it.

**Duplicate Files** hashes same-size files (size, then first 64 KiB, then the whole file, then a byte-for-byte check), keeps the oldest copy of each group and lists the rest. Hard links are not duplicates.

**Where did my disk go?** `cleansys usage [PATH] [-d depth] [-n top]` prints a size tree with bars, biggest first (real on-disk size, so sparse files are not overstated; never leaves the drive). In the TUI, `U` opens the same view as a browser: ↑/↓, Enter opens a folder, ← goes up, and `d` deletes the highlighted entry — after a confirmation, to the **trash** by default (`p` switches to permanent). Protected system paths and your "never delete" patterns are refused, Steam and app-data paths get a warning, and a file on another drive than your trash can only be deleted permanently.

It searches only the **deep roots**: your home folder, or `"deep_roots": ["~", "/mnt/data"]` in `engine.json` (`CLEANSYS_DEEP_ROOTS` overrides). A second drive is included only when you list it (settings `o`, or `cleansys config add-deep-root /mnt/data`). Symlinks, hidden folders, build/dependency trees, Wine prefixes, Steam libraries and the Android SDK are skipped, and a drive mounted below a root is never entered. The project scan likewise stays on the drive of its root.

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
`command` (run a program; `measure` folders report the real bytes freed, and `preview_args` — run with `preview_program` when that is not the program itself — is a read-only dry run whose listed files are what a preview reports; without it a preview can only show the whole folder. In arguments a single `$` followed by a letter is a variable: write `$$name` for a shell variable), `sqlite_rows` (clear named tables of a SQLite file — browser cookies, site storage — without deleting the file; SQLite is built in, so it works on Windows too), `project_artifacts` (`markers`, `artifacts`) and `large_files` (deep scan: `extensions` and/or `names` — exact, `*suffix`, `prefix*`; either list may match). `delete` also supports `contents_only`, `files_only`, `recreate`, regex filters
(`name_regex`, `not_name_regex`, `path_regex`, `not_path_regex`) and `min_age_days` (only items neither written nor
read for that many days; with `age_by_contents = true` a directory is judged by the newest file *inside* it, and the version a manager has selected — a `current`/`default` symlink, a `bin/` shim, a `version` or `alias/default` file — is never removed, nor any version a project under the scan roots pins in `.nvmrc`, `.node-version`, `.python-version`, `.ruby-version`, `.tool-versions`, `.sdkmanrc`, `rust-toolchain(.toml)`, `gradle-wrapper.properties` or `go.mod`). Project build folders that git tracks (a committed `vendor/` or `build/`) are never offered. Set `requires_root = true` for anything that
needs elevation — it will appear in the System · root section.

## 🔧 Installation details

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

### Arch Linux

Not on the AUR yet. Every release attaches a `PKGBUILD`; build and install it with `makepkg`:

```bash
mkdir cleansys-pkg && cd cleansys-pkg
curl -fsSLO https://github.com/sorinirimies/cleansys/releases/latest/download/PKGBUILD
makepkg -si          # builds cleansys, cleansys-gui and cleansys-web into one package
```

Uninstall with `sudo pacman -R cleansys`.

### Release files

Every [release](https://github.com/sorinirimies/cleansys/releases/latest) (only the latest is kept) has, for each
app (`cleansys`, `cleansys-gui`, `cleansys-web`):

| Platform | Files |
|----------|-------|
| Linux (`x86_64`, `aarch64`) | `<app>_<version>_amd64.deb` / `_arm64.deb`, `<app>-<version>-x86_64.rpm` / `-aarch64.rpm`, AppImage (TUI and GUI), `<app>-<target>.tar.gz` |
| macOS (Intel + Apple Silicon) | one universal `cleansys-<version>-macos.dmg` (GUI app + `cleansys` and `cleansys-web`), `<app>-<target>.tar.gz` |
| Windows (`x86_64`) | `cleansys-<version>-windows-x86_64-setup.exe` (all three), `<app>-x86_64-pc-windows-msvc.zip` |

Each `.tar.gz` has a `.sha256` next to it. `just --list` shows all development tasks.

## 🧱 Project Structure

CleanSys is a Cargo workspace with four crates:

| Crate | Binary | Description |
|-------|--------|-------------|
| [`cleansys-core`](crates/cleansys-core) | *(library)* | Framework-free logic shared by all front-ends: cleaner engine, scan board, scheduler, safety rules, sudo auth |
| [`cleansys-tui`](crates/cleansys-tui) | `cleansys` | Ratatui terminal UI **and** the CLI — published to crates.io as `cleansys` |
| [`cleansys-gui`](crates/cleansys-gui) | `cleansys-gui` | Iced desktop GUI |
| [`cleansys-web`](crates/cleansys-web) | `cleansys-web` | Local web UI built with [Topcoat](https://github.com/tokio-rs/topcoat) (needs Rust 1.98+) |

## 🛡️ Safety

- Nothing is deleted without a **confirmation** (TUI/GUI/CLI) except in explicit `--yes` / scheduled runs; every action has a **dry-run** (`-n`, `d`, 🔍 Preview).
- **Protected paths** can never be removed: filesystem roots, system directories, `$HOME` and its ancestors, `Documents`, `Desktop`, `.ssh`, `.gnupg`, `.config`… Paths are absolute, `..` is rejected, **symlinks are never followed**.
- Your **exclusion globs** (`cleansys config exclude '<glob>'`) always win.
- **Running apps** (browsers, Slack, …) make a cleaner refuse instead of corrupting a live profile.
- Project build output needs a **marker file** next to it and (by default) **14 days of inactivity**.
- Every byte reported is **measured**, not estimated; failures surface as real errors, not silent zeros.

## 🖥️ Platform Support

Cleaners are platform-native: each resolves OS-appropriate locations
([`cleansys-core::cleaners::platform`](crates/cleansys-core/src/cleaners/platform.rs)) and measures real freed bytes.

| Platform | TUI / CLI | GUI | Web | Typical cleaners (user · system/root) |
|----------|:---:|:---:|:---:|-------|
| Linux (x86_64, aarch64) | ✅ | ✅ | ✅ | `~/.cache`, browser caches, thumbnails, `/tmp`, Trash · apt/pacman/dnf caches, journald vacuum, old kernels, crash reports |
| macOS (Intel + Apple Silicon) | ✅ | ✅ | ✅ | `~/Library/Caches`, browser caches, QuickLook, `$TMPDIR`, `~/.Trash` · Xcode DerivedData, Simulator caches, system logs, crash reports; Homebrew cache runs **without** root |
| Windows (x86_64) | ✅ | ✅ | ✅ | Browser caches, INetCache, CrashDumps, `%TEMP%`, Recycle Bin · Windows Update cache, `C:\Windows\Temp` (Administrator) |

pip/npm/cargo caches are cleaned on all three. The GUI uses [Iced](https://iced.rs) (X11 + Wayland, Cocoa, Win32), the TUI works in any
terminal ([Ratatui](https://github.com/ratatui-org/ratatui)), and the web UI only needs a browser. Pre-built binaries: see [Releases](https://github.com/sorinirimies/cleansys/releases).

## 🏗️ Architecture

```
crates/
├── cleansys-core/            # shared, UI-free
│   └── src/
│       ├── engine/           # declarative cleaners: spec (TOML) · exec · paths · safety · running guard ·
│       │   │                 #   schedule · headless (scan/clean/auto) · config
│       │   └── builtin/*.toml    # ~370 built-in cleaners (system ones in system*.toml; browsers/apps/games are generated by scripts/gen_content_packs.nu — hand-written browser extras live in browsers_extra.toml, ecosystems in ecosystems.toml)
│       ├── cleaners/         # hard-coded user cleaners, the Windows Recycle Bin (Shell API) + platform paths
│       ├── model.rs          # CleanerItem (id, risk, root), categories, user-land/root layout, recommended preset
│       ├── scan/             # background scan pool + ScanBoard (sizes, visibility, filtering) used by TUI and GUI
│       ├── auth.rs · utils.rs · settings.rs · theme/
├── cleansys-tui/             # Ratatui TUI + CLI (binary `cleansys`)
├── cleansys-gui/             # Iced GUI (binary `cleansys-gui`): state · update · view (responsive layouts)
└── cleansys-web/             # Topcoat web UI (binary `cleansys-web`): state · pages · components · api · style
```

Adding a cleaner is usually just a TOML entry in `crates/cleansys-core/src/engine/builtin/` (or, for yourself, in
`~/.config/cleansys/cleaners.d/`). Repetitive packs (Chromium-family browsers, Electron apps, games) are generated:
edit `scripts/gen_content_packs.nu` and run `just gen-packs`. A `delete` action can filter by
regex (`name_regex`, `path_regex`, …) and by age (`min_age_days`); a `command` action runs a tool (with `sudo = true`
for root) and measures the folders it frees. Hard-coded Rust cleaners are only for what TOML cannot express (the
Windows Recycle Bin needs a Shell API call). See [`docs/TESTING.md`](docs/TESTING.md) for the test layout.

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
(`demo/fixture.nu`, sparse files — no real paths, nothing of yours is touched; it points `HOME` at `/dev/shm/demo` and `CLEANSYS_FS_ROOT` at its fake `root/` so even the System · root cleaners read synthetic data). See
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
- [Downloads](https://github.com/sorinirimies/cleansys/releases/latest) — pre-built TUI, GUI and web UI packages
- [Crates.io](https://crates.io/crates/cleansys)
- [Documentation](https://docs.rs/cleansys)
