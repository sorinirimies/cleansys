# Changelog

All notable changes to this project will be documented in this file.

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
