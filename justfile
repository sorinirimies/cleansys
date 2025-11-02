# CleanSys - Modern Terminal-Based System Cleaner for Linux
# Install just: cargo install just
# Install git-cliff: cargo install git-cliff
# Install vhs: https://github.com/charmbracelet/vhs
# Usage: just <task>

# Default task - show available commands
default:
    @just --list

# Install required tools (just, git-cliff)
install-tools:
    @echo "Installing required tools..."
    @command -v just >/dev/null 2>&1 || cargo install just
    @command -v git-cliff >/dev/null 2>&1 || cargo install git-cliff
    @echo "Note: VHS must be installed separately: https://github.com/charmbracelet/vhs"
    @echo "✅ All tools installed!"

# Build the project
build:
    cargo build

# Build release version
build-release:
    cargo build --release

# Run the application
run:
    cargo run

# Run tests
test:
    cargo test

# Check code without building
check:
    cargo check

# Format code
fmt:
    cargo fmt

# Check if code is formatted
fmt-check:
    cargo fmt --check

# Run clippy linter
clippy:
    cargo clippy -- -D warnings

# Run all checks (fmt, clippy, test)
check-all: fmt-check clippy test
    @echo "✅ All checks passed!"

# Clean build artifacts
clean:
    cargo clean

# Install the application locally
install:
    cargo install --path .

# Check if git-cliff is installed
check-git-cliff:
    @command -v git-cliff >/dev/null 2>&1 || { echo "❌ git-cliff not found. Install with: cargo install git-cliff"; exit 1; }

# Generate full changelog from all tags
changelog: check-git-cliff
    @echo "Generating full changelog..."
    git-cliff -o CHANGELOG.md
    @echo "✅ Changelog generated!"

# Generate changelog for unreleased commits only
changelog-unreleased: check-git-cliff
    @echo "Generating unreleased changelog..."
    git-cliff --unreleased --prepend CHANGELOG.md
    @echo "✅ Unreleased changelog generated!"

# Generate changelog for specific version tag
changelog-version version: check-git-cliff
    @echo "Generating changelog for version {{version}}..."
    git-cliff --tag v{{version}} -o CHANGELOG.md
    @echo "✅ Changelog generated for version {{version}}!"

# Preview changelog without writing to file
changelog-preview: check-git-cliff
    @git-cliff

# Preview unreleased changes
changelog-preview-unreleased: check-git-cliff
    @git-cliff --unreleased

# Generate changelog for latest tag only
changelog-latest: check-git-cliff
    @echo "Generating changelog for latest tag..."
    git-cliff --latest -o CHANGELOG.md
    @echo "✅ Latest changelog generated!"

# Update changelog with all commits (force regenerate)
changelog-update: check-git-cliff
    @echo "Regenerating complete changelog from all tags..."
    git-cliff --output CHANGELOG.md
    @echo "✅ Changelog updated from all git history!"

# Bump version (usage: just bump 0.2.5)
bump version: check-git-cliff
    @echo "Bumping version to {{version}}..."
    @./scripts/bump_version.sh {{version}}

# Quick release: format, check, test, and build
release-check: fmt clippy test build-release
    @echo "✅ Ready for release!"

# Publish to crates.io (dry run)
publish-dry:
    cargo publish --dry-run

# Publish to crates.io
publish:
    cargo publish

# Update dependencies
update:
    cargo update

# Show outdated dependencies
outdated:
    cargo outdated

# Generate documentation
doc:
    cargo doc --no-deps --open

# Watch and auto-run on file changes (requires cargo-watch)
watch:
    cargo watch -x run

# Git: commit current changes
commit message:
    git add .
    git commit -m "{{message}}"

# Git: push to origin
push:
    git push origin main

# Git: push tags
push-tags:
    git push --tags

# Full release workflow: bump version and push
release version: (bump version)
    @echo "Pushing to remote..."
    git push origin main
    @echo "Pushing tag v{{version}}..."
    git push origin v{{version}}
    @echo "Verifying tag was pushed..."
    @if git ls-remote --tags origin | grep -q "refs/tags/v{{version}}"; then \
        echo "✅ Release v{{version}} complete! Release workflow should trigger shortly."; \
    else \
        echo "⚠️  Warning: Tag v{{version}} may not have been pushed successfully!"; \
        exit 1; \
    fi

# Show current version
version:
    @grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/'

# Show project info
info:
    @echo "Project: CleanSys"
    @echo "Version: $(just version)"
    @echo "Author: Sorin Albu-Irimies"
    @echo "License: MIT"

# Show git-cliff info
cliff-info:
    @echo "Git-cliff configuration:"
    @echo "  Config file: cliff.toml"
    @echo "  Installed: $(command -v git-cliff >/dev/null 2>&1 && echo '✅ Yes' || echo '❌ No (run: just install-tools)')"
    @command -v git-cliff >/dev/null 2>&1 && git-cliff --version || true

# View changelog
view-changelog:
    @cat CHANGELOG.md

# Check if VHS is installed
check-vhs:
    @command -v vhs >/dev/null 2>&1 || { echo "❌ VHS not found. Install from: https://github.com/charmbracelet/vhs"; exit 1; }

# Run VHS to generate demo GIF
vhs: check-vhs
    @echo "Running VHS tape to generate demo..."
    vhs demo/demo.tape
    @echo "✅ Demo generated at demo/demo.gif"

# Run VHS quick demo
vhs-quick: check-vhs
    @echo "Running VHS quick demo tape..."
    vhs demo/demo-quick.tape
    @echo "✅ Quick demo generated at demo/demo-quick.gif"

# Run VHS showcase demo
vhs-showcase: check-vhs
    @echo "Running VHS showcase tape..."
    vhs demo/demo-showcase.tape
    @echo "✅ Showcase demo generated"

# Generate all VHS demos
vhs-all: vhs vhs-quick vhs-showcase
    @echo "✅ All demos generated!"

# Clean VHS outputs
vhs-clean:
    @echo "Cleaning VHS output files..."
    @rm -f demo/*.gif
    @echo "✅ VHS outputs cleaned!"
