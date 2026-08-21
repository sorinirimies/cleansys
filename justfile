# CleanSys workspace — task runner
# Install just:      cargo install just
# Install nu:         cargo install nu   (https://www.nushell.sh)
# Install git-cliff:  cargo install git-cliff
# Install vhs:        brew install vhs  OR  go install github.com/charmbracelet/vhs@latest
# Usage: just <task>

# ── Default ───────────────────────────────────────────────────────────────────

default:
    @just --list

# ── Tool checks ───────────────────────────────────────────────────────────────

_check-git-cliff:
    @command -v git-cliff >/dev/null 2>&1 || { \
        echo "❌ git-cliff not found. Install with: cargo install git-cliff"; exit 1; \
    }

# Check nu (nushell) is available
_check-nu:
    @command -v nu >/dev/null 2>&1 || { \
        echo "❌ nu (nushell) not found. Install: https://www.nushell.sh"; exit 1; \
    }

_check-vhs:
    @command -v vhs >/dev/null 2>&1 || { \
        echo "❌ vhs not found."; \
        echo "   macOS:      brew install vhs"; \
        echo "   Any:        go install github.com/charmbracelet/vhs@latest"; \
        exit 1; \
    }

# Install all recommended development tools
install-tools:
    @echo "Installing development tools…"
    @command -v git-cliff >/dev/null 2>&1 || cargo install git-cliff --locked
    @command -v nu >/dev/null 2>&1 || cargo install nu --locked
    @echo "Note: VHS must be installed separately: https://github.com/charmbracelet/vhs"
    @echo "✅ All tools installed!"

# ── Build ─────────────────────────────────────────────────────────────────────

# Build the entire workspace (dev)
build:
    cargo build --workspace

# Build only the core library (dev)
build-core:
    cargo build -p cleansys-core

# Build only the TUI/CLI crate (dev)
build-tui:
    cargo build -p cleansys-tui

# Build only the GUI crate (dev)
build-gui:
    cargo build -p cleansys-gui

# Build release binaries for TUI and GUI
build-release:
    cargo build --release -p cleansys-tui
    cargo build --release -p cleansys-gui

# ── Run ───────────────────────────────────────────────────────────────────────

# Launch the Ratatui terminal UI (default `cleansys` binary)
run-tui:
    cargo run -p cleansys-tui

# Launch the Iced desktop GUI
run-gui:
    cargo run -p cleansys-gui

# Alias: default run launches the TUI (matches historical `cleansys` behavior)
run: run-tui

# ── Test ──────────────────────────────────────────────────────────────────────

# Run the full workspace test suite
test:
    cargo test --workspace --locked --all-features --all-targets

# Test only the core library
test-core:
    cargo test -p cleansys-core --all-features

# Test only the TUI crate
test-tui:
    cargo test -p cleansys-tui --all-features

# Test only the GUI crate
test-gui:
    cargo test -p cleansys-gui --all-features

# Run Nu script tests
test-nu: _check-nu
    nu scripts/tests/run_all.nu

# Run both Rust and Nu tests
test-all-nu: test test-nu
    @echo "✅ All Rust and Nu tests passed!"

# ── Code quality ──────────────────────────────────────────────────────────────

# Check without building
check:
    cargo check --workspace

# Format all code
fmt:
    cargo fmt --all

# Check formatting without modifying files
fmt-check:
    cargo fmt --all -- --check

# Run clippy across the workspace
clippy:
    cargo clippy --workspace --all-targets --all-features -- -D warnings -A deprecated

# Run all quality checks (format, clippy, test, nu) — must pass before a release.
# Auto-formats first, then verifies no changes remain (catches unstaged format diffs).
check-all: fmt clippy test test-nu
    @echo "🔍 Verifying formatting is clean…"
    cargo fmt --all -- --check
    @echo "✅ All checks passed!"

# Full pre-release quality gate — everything in check-all plus a release build.
check-release: check-all build-release
    @echo "✅ Release quality gate passed (fmt + clippy + test + nu + release build)!"

# ── VHS Demo GIFs ─────────────────────────────────────────────────────────────

vhs: _check-vhs
    @echo "Running VHS tape to generate demo…"
    vhs demo/demo.tape
    @echo "✅ Demo generated at demo/target/demo.gif"

vhs-userland: _check-vhs
    @echo "Running VHS userland cleaners demo tape…"
    vhs demo/userland-cleaners.tape
    @echo "✅ Userland cleaners demo generated at demo/target/userland-cleaners.gif"

vhs-system: _check-vhs
    @echo "Running VHS system cleaners demo tape…"
    vhs demo/system-cleaners.tape
    @echo "✅ System cleaners demo generated at demo/target/system-cleaners.gif"

vhs-all: vhs vhs-userland vhs-system
    @echo "✅ All demos generated!"

vhs-clean:
    @echo "Cleaning VHS output files…"
    @rm -f demo/target/*.gif
    @echo "✅ VHS outputs cleaned!"

# ── Packaging ────────────────────────────────────────────────────────

# Build Linux .deb and .rpm packages (requires dpkg-deb + alien; run locally on Linux)
package-linux version: build-release
    nu scripts/ci/package_linux.nu {{ version }} $(rustc -vV | grep host | cut -d' ' -f2)

# Build Linux AppImages (requires appimagetool in PATH)
package-appimage version:
    bash scripts/ci/package_appimage.sh {{ version }} $(rustc -vV | grep host | cut -d' ' -f2)

# Cross-compile Windows .exe from Linux (requires mingw-w64)
# Install: sudo pacman -S mingw-w64-gcc   OR   sudo apt install gcc-mingw-w64-x86-64
package-windows version:
    rustup target add x86_64-pc-windows-gnu
    cargo build --release -p cleansys-tui -p cleansys-gui --target x86_64-pc-windows-gnu
    mkdir -p dist
    cp target/x86_64-pc-windows-gnu/release/cleansys.exe     dist/cleansys-tui-x86_64-windows.exe
    cp target/x86_64-pc-windows-gnu/release/cleansys-gui.exe dist/cleansys-gui-x86_64-windows.exe

# Update AUR PKGBUILD to a new version
update-aur version:
    bash scripts/ci/update_aur.sh {{ version }}

# Note: macOS universal DMG is built automatically by GitHub Actions (macos-latest runner)
# Note: Windows NSIS installer is built by GitHub Actions (windows-latest runner)
#       and cross-compiled via mingw-w64 by Gitea CI (no Windows machine needed)
# Note: Linux aarch64 is cross-compiled by both GitHub Actions and Gitea CI

# ── Documentation ─────────────────────────────────────────────────────────────

# Generate and open docs for the TUI crate
doc-tui:
    cargo doc --no-deps -p cleansys-tui --open

# Generate and open docs for the GUI crate
doc-gui:
    cargo doc --no-deps -p cleansys-gui --open

# Generate docs for the full workspace (no browser)
doc:
    cargo doc --no-deps --workspace

# ── Changelog ─────────────────────────────────────────────────────────────────

changelog: _check-git-cliff
    @echo "Generating full changelog…"
    git-cliff --output CHANGELOG.md
    @echo "✅ CHANGELOG.md updated."

changelog-unreleased: _check-git-cliff
    git-cliff --unreleased --prepend CHANGELOG.md
    @echo "✅ Unreleased changes prepended."

changelog-preview: _check-git-cliff
    @git-cliff --unreleased

changelog-latest: _check-git-cliff
    @git-cliff --latest

# ── Version bump ─────────────────────────────────────────────────────────────

# Validate that a version string will produce a valid vX.Y.Z tag.
validate-tag version: _check-nu
    @nu scripts/ci/validate_tag.nu "v{{version}}" 2>&1 >/dev/null

_check-version-changed version: _check-nu
    #!/usr/bin/env sh
    current=$(nu scripts/version.nu)
    if [ "$current" = "{{version}}" ]; then
        echo "❌ Version {{version}} is already the current version. Nothing to bump."
        exit 1
    fi
    echo "✅ Version will change: $current → {{version}}"

# Bump the workspace version, regenerate Cargo.lock + CHANGELOG.md, commit and tag.
# Validation runs first (cheap), quality gate runs second (expensive).
bump version: (validate-tag version) (_check-version-changed version) check-release _check-git-cliff
    nu scripts/bump_version.nu --yes {{ version }}

# ── Publish (crates.io) ───────────────────────────────────────────────────────

# Run the full pre-publish readiness check (fmt, clippy, tests, docs, dry-run)
check-publish: _check-nu
    nu scripts/check_publish.nu

# Dry-run publish for all three crates (in dependency order)
publish-dry: check-all
    @echo "Dry-run: cleansys-core"
    cargo publish --dry-run -p cleansys-core
    @echo "Dry-run: cleansys-tui"
    cargo publish --dry-run -p cleansys-tui
    @echo "Dry-run: cleansys-gui"
    cargo publish --dry-run -p cleansys-gui

# Publish all three in dependency order: core → tui → gui.
publish: check-all publish-core publish-tui publish-gui
    @echo "✅ cleansys-core, cleansys-tui, and cleansys-gui published to crates.io!"

publish-core:
    @echo "📦 Publishing cleansys-core…"
    cargo publish -p cleansys-core
    @echo "⏳ Waiting 30 s for the index to propagate…"
    sleep 30

publish-tui:
    @echo "📦 Publishing cleansys-tui…"
    cargo publish -p cleansys-tui

publish-gui:
    @echo "📦 Publishing cleansys-gui…"
    cargo publish -p cleansys-gui

# Show what would be released without making any changes
release-preview: _check-git-cliff
    @echo "Current version: $(just version)"
    @echo ""
    @echo "Unreleased commits:"
    @git-cliff --unreleased
    @echo ""
    @echo "Published crates:  cleansys-tui  •  cleansys-gui"
    @echo "Internal crate:    cleansys-core (publish = false-able)"

# ── Housekeeping ──────────────────────────────────────────────────────────────

clean:
    cargo clean

update:
    cargo update

update-deps:
    @echo "⬆️  Updating dependencies…"
    cargo update
    @echo "🔍 Running quality gate…"
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings -A deprecated
    cargo test --workspace --locked --all-features --all-targets
    @echo "✅ All checks passed — committing dependency updates…"
    git add Cargo.lock
    git diff --cached --quiet || git commit -m "chore: update dependencies"
    git push origin main
    @echo "✅ Dependency updates pushed to GitHub."

# Upgrade workspace dependency *pins* (Cargo.toml constraints) via cargo-edit,
# then cross-check (fmt/clippy/test/doc). Install cargo-edit first: cargo install cargo-edit
upgrade-deps: _check-nu
    nu scripts/upgrade_deps.nu

# Dry-run: list outdated dependencies without changing anything
upgrade-deps-check: _check-nu
    nu scripts/upgrade_deps.nu --check

outdated:
    cargo outdated

# Show the current workspace version
version: _check-nu
    @nu scripts/version.nu

# Show project info
info:
    @echo "Project: CleanSys"
    @echo "Version: $(just version)"
    @echo "Author: Sorin Albu-Irimies"
    @echo "License: MIT"

view-changelog:
    @cat CHANGELOG.md

# Show all configured remotes
remotes:
    @git remote -v

# ── Git remotes & pushing ────────────────────────────────────────────────────

push:
    git push origin main

push-gitea:
    git push gitea main

# Push the current branch to Gitea (nexus-lab instance)
push-gitea-nexus-lab:
    git push gitea-nexus-lab main

push-all:
    #!/usr/bin/env sh
    failed=""
    git push origin main             || failed="$failed origin"
    git push gitea main              || failed="$failed gitea"
    git push gitea-nexus-lab main     || failed="$failed gitea-nexus-lab"
    if [ -n "$failed" ]; then
        echo "⚠️  Failed to push to:$failed"
    else
        echo "✅ Pushed to GitHub, Gitea, and Gitea (nexus-lab)!"
    fi

push-tags:
    git push origin --tags

push-tags-all:
    git push origin --tags
    git push gitea --tags
    git push gitea-nexus-lab --tags
    @echo "✅ Tags pushed to GitHub, Gitea, and Gitea (nexus-lab)!"

pull:
    git pull origin main

pull-gitea:
    git pull gitea main

# Pull the current branch from Gitea (nexus-lab instance)
pull-gitea-nexus-lab:
    git pull gitea-nexus-lab main

# Push the latest commit and all tags to every remote (no bump).
push-release-all: check-all
    #!/usr/bin/env sh
    failed=""
    git push --follow-tags origin main             || failed="$failed origin"
    git push --follow-tags gitea main              || failed="$failed gitea"
    git push --follow-tags gitea-nexus-lab main     || failed="$failed gitea-nexus-lab"
    if [ -n "$failed" ]; then
        echo "⚠️  Failed to push to:$failed"
    else
        echo "✅ Latest commit + tags pushed to all remotes."
    fi

# Force-sync Gitea with GitHub
sync-gitea:
    git push gitea main --force
    git push gitea --tags --force
    @echo "✅ Gitea synced with GitHub."

# Force-sync Gitea (nexus-lab instance) with GitHub
sync-gitea-nexus-lab:
    git push gitea-nexus-lab main --force
    git push gitea-nexus-lab --tags --force
    @echo "✅ Gitea (nexus-lab) force-synced with GitHub."

# Force-sync all Gitea instances with GitHub (continues on failure)
sync-all-gitea:
    #!/usr/bin/env sh
    failed=""
    git push gitea main --force                  || failed="$failed gitea"
    git push gitea --tags --force                || failed="$failed gitea-tags"
    git push gitea-nexus-lab main --force        || failed="$failed gitea-nexus-lab"
    git push gitea-nexus-lab --tags --force      || failed="$failed gitea-nexus-lab-tags"
    if [ -n "$failed" ]; then
        echo "⚠️  Failed to sync:$failed"
    else
        echo "✅ All Gitea instances force-synced with GitHub."
    fi

# Add a Gitea remote and optionally push — interactive (nu script)
setup-gitea url: _check-nu
    nu scripts/setup_gitea.nu {{ url }}

# Migrate this project to dual GitHub + Gitea hosting (interactive)
migrate-gitea: _check-nu
    nu scripts/migrate_to_gitea.nu

# ── Release workflows ─────────────────────────────────────────────────────────

# Bump, commit, tag, then push to GitHub — triggers the Release workflow.
release version: (bump version)
    @echo "Pushing release v{{version}} to GitHub…"
    git push --follow-tags origin main
    @echo "✅ Release v{{version}} pushed — Release workflow will trigger automatically."

# Bump, commit, tag, then push to Gitea only.
release-gitea version: (bump version)
    @echo "Pushing release v{{version}} to Gitea…"
    git push --follow-tags gitea main
    @echo "✅ Release v{{version}} live on Gitea."

# Bump, commit, tag, then push to Gitea (nexus-lab instance) only.
release-gitea-nexus-lab version: (bump version)
    @echo "Pushing release v{{version}} to Gitea (nexus-lab)…"
    git push --follow-tags gitea-nexus-lab main
    @echo "✅ Release v{{version}} live on Gitea (nexus-lab)."

# Bump, commit, tag, then push to all remotes.
release-all version: (bump version)
    #!/usr/bin/env sh
    echo "Pushing release v{{version}} to all remotes…"
    failed=""
    git push --follow-tags origin main             || failed="$failed origin"
    git push --follow-tags gitea main              || failed="$failed gitea"
    git push --follow-tags gitea-nexus-lab main     || failed="$failed gitea-nexus-lab"
    if [ -n "$failed" ]; then
        echo "⚠️  Release v{{version}} failed to push to:$failed"
    else
        echo "✅ Release v{{version}} pushed to GitHub, Gitea, and Gitea (nexus-lab)!"
    fi

# Manually re-trigger the Release workflow for an existing tag via the gh CLI.
release-retrigger version:
    @command -v gh >/dev/null 2>&1 || { \
        echo "❌ GitHub CLI (gh) not found. Install from https://cli.github.com"; exit 1; \
    }
    @echo "Manually dispatching Release workflow for tag v{{version}}…"
    gh workflow run release.yml --field tag=v{{version}}
    @echo "✅ Dispatched — check progress at: https://github.com/$(gh repo view --json nameWithOwner -q .nameWithOwner)/actions"
