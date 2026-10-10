# Changelog

All notable changes to this project will be documented in this file.

## 0.8.1 - 2026-10-10
### ⚡ Performance
- perf(core): parallel directory sizing (rayon, first 4 levels) — Gradle cache 18s -> 4.5s, full scan 20s -> 7.8s
- perf(core): parallel directory deletion, parallel marker-only project index, streaming file filters
### ✨ Features
- feat(core): declarative cleaner engine with dev, build-artifact and AI/LLM cleaners
- feat(engine): command action, sudo deletes, and ~130 more cleaners
- feat: schedule (systemd/cron/launchd/schtasks), auto command, process guard, recommended preset
- feat(gui): responsive redesign — sidebar nav, background scan with sizes, search, action bar
- feat(tui): responsive sidebar layout, live sizes, filter, hide-empty; shared ScanBoard in core
- feat(web): cleansys-web (Topcoat) + nushell-only tooling, new tapes/GIFs/screenshots (git-lfs)
- feat: sudo prompt in web UI; idle-day selector and expandable per-path selection in TUI/GUI/web
- feat(tui): show clean progress, outcome and activity log in the main view (worker thread); remove the detailed progress screen
- feat: Settings and About in the TUI, GUI and web UI; version always visible; engine/preferences editable in-app
- feat(gui): window resizer — presets, grow/shrink, maximise, full screen shortcuts and settings; remember the window size
- feat: same animated progress in TUI, GUI and web — spinner plus a bar with a sweeping highlight (shared cleansys_core::anim)
- feat(gui): round progress ring per cleaner/category while measuring, and on progress labels (canvas); shared ring_arc maths
- feat(gui): activity panel fills the width, resizable by dragging its top edge, Clear lit only when there is content
### 🐛 Bug Fixes
- fix(gui): stop macOS 'Choose Application' dialog after cleaning
- fix(core): evict stale project index cache; wipe cached sudo password on drop
- fix(test,core): make the web details test independent of the shared sandbox project; write engine.json atomically
- fix(gui,macos): install the standard Window menu so the system's tiling/zoom shortcuts work (as in tokenburn)
- fix(gui): sudo authentication no longer pins the header badge to ROOT; remove dead code (unused settings fns, icon, message); regression tests
### 📚 Documentation
- docs: document details expansion, per-path selection and idle-days selector
### 📦 Other Changes
- docs+ux: responsive redesign docs, new tapes/GIFs/screenshots (git-lfs), synthetic demo fixture
- scripts: prune_release_assets.nu auto-detects platform/repo/token/latest; just prune-releases[-preview]
- scripts: prune_release_assets.nu — pure testable helpers, refuse unknown keep tag, nu tests (as in tokenburn)
- gui/web: header badge shows ROOT while a system (root) category is open
- release: ship cleansys-web with the rest — crates.io publish, raw binaries, Windows zip, .deb/.rpm; check_publish covers it
- ux: show a spinner instead of totals until the scan is done (TUI via tui-spinner, GUI, web)
- docs/demo: re-record TUI tapes and web screenshots for the in-view progress, details and idle-days; silence cleaner output in the TUI; wider footer
### 🔄 CI
- ci: keep only the latest release's uploaded assets
- ci(gitea): checkout@v4 in prune job
- ci: weekly + manual prune of old release assets (GitHub and Gitea)
- ci: fix run 2098 — tests no longer assume non-root (CI runs as root), rustdoc private link, retry apt downloads
### 🔧 Chores
- chore: bump version to 0.7.0
- chore: bump version to 0.7.1
- chore: bump version to 0.7.5
- chore: bump version to 0.7.7
- chore: bump version to 0.7.6
- chore: bump version to 0.7.8
- chore: bump version to 0.7.9
- chore: bump version to 0.8.0
- chore(deps): nightly dependency upgrade 2026-10-10
### 🧪 Testing
- test(web): root-badge test shows empty categories (hide=0) so it passes on Linux CI
- test(web): server tests use a small fake category set instead of scanning the real machine (11 parallel scans timed out on slower/busier hosts); 6s -> 0.15s
- test(tui): cover the clean worker, progress/summary state and run keys; tidy logging and lints
**Full Changelog**: https://github.com/sorinirimies/cleansys/compare/v0.6.22...v0.8.1
## 0.6.22 - 2026-10-06
### 📚 Documentation
- docs(readme): add TUI and GUI crates.io download badges
### 🔄 CI
- ci(deps): only cut patch release when a direct dependency changes
- ci(gitea): retry rustup install on flaky network
### 🔧 Chores
- chore(deps): nightly dependency upgrade 2026-10-05
- chore(deps): nightly dependency upgrade 2026-10-06
- chore: bump version to 0.6.22
**Full Changelog**: https://github.com/sorinirimies/cleansys/compare/v0.6.21...v0.6.22
## 0.6.21 - 2026-10-04
### 🔧 Chores
- chore(deps): nightly dependency upgrade 2026-10-04
- chore: bump version to 0.6.21
**Full Changelog**: https://github.com/sorinirimies/cleansys/compare/v0.6.20...v0.6.21
## 0.6.20 - 2026-10-03
### 🔧 Chores
- chore(deps): nightly dependency upgrade 2026-10-03
- chore: bump version to 0.6.20
**Full Changelog**: https://github.com/sorinirimies/cleansys/compare/v0.6.19...v0.6.20
## 0.6.19 - 2026-10-02
### 🔧 Chores
- chore(deps): nightly dependency upgrade 2026-10-02
- chore: bump version to 0.6.19
**Full Changelog**: https://github.com/sorinirimies/cleansys/compare/v0.6.18...v0.6.19
## 0.6.18 - 2026-10-01
### 🔧 Chores
- chore(deps): nightly dependency upgrade 2026-10-01
- chore: bump version to 0.6.18
**Full Changelog**: https://github.com/sorinirimies/cleansys/compare/v0.6.17...v0.6.18
## 0.6.17 - 2026-09-28
### 🔧 Chores
- chore(deps): nightly dependency upgrade 2026-09-28
- chore: bump version to 0.6.17
**Full Changelog**: https://github.com/sorinirimies/cleansys/compare/v0.6.16...v0.6.17
## 0.6.16 - 2026-09-26
### 🔧 Chores
- chore(deps): nightly dependency upgrade 2026-09-26
- chore: bump version to 0.6.16
**Full Changelog**: https://github.com/sorinirimies/cleansys/compare/v0.6.15...v0.6.16
## 0.6.15 - 2026-09-25
### 🐛 Bug Fixes
- fix(ci): disable Gitea nightly deps-update schedule to prevent dual auto-release
### 📚 Documentation
- docs: update README and CHANGELOG for v0.6.8
### 📦 Other Changes
- merge: reconcile diverged GitHub/gitea-starscream nightly auto-release histories
- merge: reconcile second round of gitea nightly divergence, bump to 0.6.14
- merge: reconcile github's 2026-09-24 nightly run (thiserror bump) into 0.6.14
### 🔧 Chores
- chore: bump version to 0.6.8
- chore(deps): nightly dependency upgrade 2026-09-17
- chore: bump version to 0.6.9
- chore(deps): nightly dependency upgrade 2026-09-19
- chore: bump version to 0.6.10
- chore(deps): nightly dependency upgrade 2026-09-20
- chore: bump version to 0.6.11
- chore(deps): nightly dependency upgrade 2026-09-22
- chore: bump version to 0.6.12
- chore(deps): nightly dependency upgrade 2026-09-23
- chore: bump version to 0.6.13
- chore(deps): nightly dependency upgrade 2026-09-25
- chore: bump version to 0.6.15
**Full Changelog**: https://github.com/sorinirimies/cleansys/compare/v0.6.12...v0.6.15
## 0.6.12 - 2026-09-24
### 🔧 Chores
- chore(deps): nightly dependency upgrade 2026-09-24
- chore: bump version to 0.6.12
**Full Changelog**: https://github.com/sorinirimies/cleansys/compare/v0.6.11...v0.6.12
## 0.6.11 - 2026-09-23
### 🔧 Chores
- chore(deps): nightly dependency upgrade 2026-09-23
- chore: bump version to 0.6.11
**Full Changelog**: https://github.com/sorinirimies/cleansys/compare/v0.6.10...v0.6.11
## 0.6.10 - 2026-09-22
### 🔧 Chores
- chore(deps): nightly dependency upgrade 2026-09-22
- chore: bump version to 0.6.10
**Full Changelog**: https://github.com/sorinirimies/cleansys/compare/v0.6.9...v0.6.10
## 0.6.9 - 2026-09-19
### 🔧 Chores
- chore(deps): nightly dependency upgrade 2026-09-19
- chore: bump version to 0.6.9
**Full Changelog**: https://github.com/sorinirimies/cleansys/compare/v0.6.8...v0.6.9
## 0.6.8 - 2026-09-17
### 🐛 Bug Fixes
- fix(docs): repair corrupted README demo preview + commit real GIF/PNG assets
- fix(gui): activity log (and whole main screen) didn't stretch full width on resize
### 📚 Documentation
- docs: update README and CHANGELOG for v0.6.7
- docs: add real, committed demo previews (GIFs + GUI screenshot)
### 📦 Other Changes
- merge: reconcile GitHub auto-generated README/CHANGELOG update
### 🔧 Chores
- chore(deps): nightly dependency upgrade 2026-09-17
- chore: bump version to 0.6.8
### 🧪 Testing
- test(tui): add render.rs smoke-test suite; cleanup: finish DRY'ing system_cleaners.rs sudo call sites
**Full Changelog**: https://github.com/sorinirimies/cleansys/compare/v0.6.7...v0.6.8
## 0.6.7 - 2026-09-15
### 📚 Documentation
- docs: update README and CHANGELOG for v0.6.4
- docs: update README and CHANGELOG for v0.6.5
- docs: update README and CHANGELOG for v0.6.6
### 📦 Other Changes
- merge: reconcile GitHub auto-generated README/CHANGELOG update
### 🔧 Chores
- chore(deps): nightly dependency upgrade 2026-09-15
- chore: bump version to 0.6.7
**Full Changelog**: https://github.com/sorinirimies/cleansys/compare/v0.6.6...v0.6.7
## 0.6.6 - 2026-09-14
### ♻️ Refactor
- refactor(core): extract cleaner!/run_sudo_step macros+helper, fix silent error swallowing, docs
### 🔧 Chores
- chore: bump version to 0.6.6
**Full Changelog**: https://github.com/sorinirimies/cleansys/compare/v0.6.5...v0.6.6
## 0.6.5 - 2026-09-14
### 🐛 Bug Fixes
- fix(gui): system cleaners silently no-op after password entry
- fix: deep-analysis pass -- TUI/GUI bugs, dedupe, panic hardening
- fix(demo): demo.tape used bare 'cargo run', broken since the cleansys-tui->cleansys package rename
### 📚 Documentation
- docs: regenerate CHANGELOG.md with correct per-version sections
- docs: regenerate CHANGELOG.md after merging Gitea auto-releases
### 📦 Other Changes
- merge: reconcile Gitea nightly-deps auto-releases (v0.6.3, v0.6.4) with GUI sudo fix
### 🔧 Chores
- chore: bump version to 0.6.5
**Full Changelog**: https://github.com/sorinirimies/cleansys/compare/v0.6.4...v0.6.5
## 0.6.4 - 2026-09-14
### 📚 Documentation
- docs: update README and CHANGELOG for v0.6.2
### 🔧 Chores
- chore(deps): nightly dependency upgrade 2026-09-13
- chore: bump version to 0.6.3
- chore(deps): nightly dependency upgrade 2026-09-14
- chore: bump version to 0.6.4
**Full Changelog**: https://github.com/sorinirimies/cleansys/compare/v0.6.3...v0.6.4
## 0.6.3 - 2026-09-14
### 🔄 Updated
- Update jiff crates to latest patch versions
### 🔧 Chores
- chore(deps): nightly dependency upgrade 2026-09-14
- chore: bump version to 0.6.3
**Full Changelog**: https://github.com/sorinirimies/cleansys/compare/v0.6.2...v0.6.3
## 0.6.2 - 2026-09-12
### 🐛 Bug Fixes
- fix(ci): install aarch64 cross-libc + alien/fakeroot for Gitea Linux release job
### 🔧 Chores
- chore: bump version to 0.6.2
**Full Changelog**: https://github.com/sorinirimies/cleansys/compare/v0.6.1...v0.6.2
## 0.6.1 - 2026-09-12
### 📦 Other Changes
- Rename TUI crate package to cleansys across tooling
### 🔄 Updated
- Update lockfile dependency patch versions
### 🔧 Chores
- chore: bump version to 0.6.1
**Full Changelog**: https://github.com/sorinirimies/cleansys/compare/v0.6.0...v0.6.1
## 0.6.0 - 2026-09-11
### ➕ Added
- Add Starscream remote tasks and bulk pull/force push
- Add downgrade guard to dependency upgrade workflows
### 📚 Documentation
- docs: update README and CHANGELOG for v0.5.1
### 📦 Other Changes
- Bump Rust dependencies and refresh lockfile
- Remove CodeGraph gitignore placeholder file
### 🔧 Chores
- chore: bump version to 0.6.0
**Full Changelog**: https://github.com/sorinirimies/cleansys/compare/v0.5.1...v0.6.0
## 0.5.1 - 2026-08-22
### ♻️ Refactor
- refactor(tui): use tui-piechart directly, drop the redundant local wrapper
### 🐛 Bug Fixes
- fix(ci): fix macOS DMG build failure blocking the v0.5.0 release
### 📚 Documentation
- docs: update README and CHANGELOG for v0.5.0
### 🔧 Chores
- chore: bump version to 0.5.1
**Full Changelog**: https://github.com/sorinirimies/cleansys/compare/v0.5.0...v0.5.1
## 0.5.0 - 2026-08-22
### ✨ Features
- feat: restructure into Cargo workspace, add Iced GUI, migrate scripts to Nushell
- feat(gui): redesign layout with tabs/icons, cross-platform support, packaging, tests
- feat(gui): add complete 43-theme selector, matching GitKraft's theme system
- feat: safety, dry-run preview, Windows elevation, macOS dev cleaners, and more
- feat(ci): nightly dependency upgrade now auto-cuts a patch release
- feat(tui): add tui-spinner animated loading indicator; fix flaky GUI tests
### ➕ Added
- Add Gitea dual-hosting scripts and justfile commands
- Add Gitea CI, release, and README update workflows
- Add comprehensive justfile and setup-just.sh for automation
### 🐛 Bug Fixes
- fix(cleaners): real cross-platform cleaning with structured, measured results
### 📚 Documentation
- docs: update README and CHANGELOG for v0.2.6
### 📦 Other Changes
- Revamp demo system: add user/system tapes, update VHS tasks
### 🔄 CI
- ci: fix git-cliff installation by adding Rust toolchain setup
### 🔄 Updated
- update gitea migration files
- Update to ratatui 0.29 and crossterm 0.28, switch pie chart to
### 🔧 Chores
- chore: bump version to 0.2.6
- chore: bump version to 0.2.7
- chore: bump version to 0.2.7
- chore: bump version to 0.2.8
- chore: bump version to 0.2.9
- chore: bump version to 0.3.0
- chore: bump version to 0.3.1
- chore: bump version to 0.3.2
- chore: add gitea-nexus-lab remote + matching justfile recipes
- chore(deps): update to latest crate versions, drop dead dependencies
- chore: bump version to 0.5.0
**Full Changelog**: https://github.com/sorinirimies/cleansys/compare/v0.2.6...v0.5.0
## 0.2.6 - 2025-11-02
### ✨ Features
- feat: add password prompt, fix legend distribution, update categories
### 📚 Documentation
- docs: update README and CHANGELOG for v0.2.5
### 📦 Other Changes
- Merge remote-tracking branch 'origin/main'
### 🔧 Chores
- chore: bump version to 0.2.6
- chore: bump version to 0.2.6
**Full Changelog**: https://github.com/sorinirimies/cleansys/compare/v0.2.5...v0.2.6
## 0.2.5 - 2025-11-02
### 🐛 Bug Fixes
- Fix crates.io categories and update dependencies in Cargo.lock
### 📦 Other Changes
- Remove invalid categories from Cargo.toml
- Bump version to 0.2.5
### 🔧 Chores
- chore: bump version to 0.2.4
- chore: bump version to 0.2.5
**Full Changelog**: https://github.com/sorinirimies/cleansys/compare/v0.2.4...v0.2.5
## 0.2.4 - 2025-11-02
### ➕ Added
- Add password prompt component and TUI sudo authentication
### 📚 Documentation
- docs: update README and CHANGELOG for v0.2.3
### 📦 Other Changes
- Remove os from categories in Cargo.toml
### 🔧 Chores
- chore: bump version to 0.2.4
**Full Changelog**: https://github.com/sorinirimies/cleansys/compare/v0.2.3...v0.2.4
## 0.2.3 - 2025-10-31
### 📚 Documentation
- docs: update README and CHANGELOG for v0.2.2
### 📦 Other Changes
- Remove legacy UI module, docs, and update project structure and README
- Merge remote-tracking branch 'origin/main'
### 🔧 Chores
- chore: bump version to 0.2.3
**Full Changelog**: https://github.com/sorinirimies/cleansys/compare/v0.2.2...v0.2.3
## 0.2.2 - 2025-10-30
### 📚 Documentation
- docs: update README and CHANGELOG for v0.2.1
### 📦 Other Changes
- Integrate tui-checkbox library and improve documentation
- Merge remote-tracking branch 'origin/main'
- Bump version to 0.2.2 and clean up integration tests
- Bump version to 0.2.3 and update categories
### 🔧 Chores
- chore: bump version to 0.2.2
**Full Changelog**: https://github.com/sorinirimies/cleansys/compare/v0.2.1...v0.2.2
## 0.2.1 - 2025-10-15
### 📚 Documentation
- docs: update README and CHANGELOG for v0.0.11
### 🔧 Chores
- chore: bump version to 0.2.1
**Full Changelog**: https://github.com/sorinirimies/cleansys/compare/v0.0.11...v0.2.1
## 0.0.11 - 2025-10-15
### 📚 Documentation
- docs: update README and CHANGELOG for v0.0.10
### 🔄 Updated
- update readme project infp
### 🔧 Chores
- chore: bump version to 0.0.11
**Full Changelog**: https://github.com/sorinirimies/cleansys/compare/v0.0.10...v0.0.11
## 0.0.10 - 2025-10-06
### 🐛 Bug Fixes
- fix: use default GITHUB_TOKEN for releases with proper permissions
### 📚 Documentation
- docs: update README and CHANGELOG for v0.0.9
### 🔧 Chores
- chore: improve release recipe with error handling
- chore: bump version to 0.0.9
- chore: bump version to 0.0.10
**Full Changelog**: https://github.com/sorinirimies/cleansys/compare/v0.0.9...v0.0.10
## 0.0.9 - 2025-10-06
### 🐛 Bug Fixes
- fix compilation target
### 🔧 Chores
- chore: bump version to 0.0.9
**Full Changelog**: https://github.com/sorinirimies/cleansys/compare/v0.0.8...v0.0.9
## 0.0.8 - 2025-10-06
### 📚 Documentation
- docs: update README and CHANGELOG for v0.0.7
### 🔧 Chores
- chore: bump version to 0.0.8
**Full Changelog**: https://github.com/sorinirimies/cleansys/compare/v0.0.7...v0.0.8
## 0.0.7 - 2025-10-06
### 📚 Documentation
- docs: update README and CHANGELOG for v0.0.6
### 🔧 Chores
- chore: bump version to 0.0.6
- chore: bump version to 0.0.7
**Full Changelog**: https://github.com/sorinirimies/cleansys/compare/v0.0.6...v0.0.7
## 0.0.6 - 2025-10-02
### 📦 Other Changes
- keep cargolock
### 🔄 Updated
- update readme
### 🔧 Chores
- chore: bump version to 0.0.5
- chore: bump version to 0.0.5
- chore: bump version to 0.0.6
**Full Changelog**: https://github.com/sorinirimies/cleansys/compare/v0.0.4...v0.0.6
## 0.0.4 - 2025-06-05
### 📦 Other Changes
- Improves progress screen behavior and controls
**Full Changelog**: https://github.com/sorinirimies/cleansys/compare/v0.0.3...v0.0.4
## 0.0.3 - 2025-06-05
### 📦 Other Changes
- Updates crate category to comply with crates.io
**Full Changelog**: https://github.com/sorinirimies/cleansys/compare/v0.0.2...v0.0.3
## 0.0.2 - 2025-06-05
### 📦 Other Changes
- Allows publishing crates with uncommitted changes
**Full Changelog**: https://github.com/sorinirimies/cleansys/compare/v0.0.1...v0.0.2
## 0.0.1 - 2025-06-05
### 📦 Other Changes
- Initial commit
- Implements release workflow with git-cliff
