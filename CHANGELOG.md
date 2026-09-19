# Changelog

All notable changes to this project will be documented in this file.

## 0.6.9 - 2026-09-19
### 🔧 Chores
- chore(deps): nightly dependency upgrade 2026-09-19
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
