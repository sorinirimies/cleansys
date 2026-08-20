#!/usr/bin/env nu
# ──────────────────────────────────────────────────────────────────────────────
# CleanSys — Bump workspace version
# ──────────────────────────────────────────────────────────────────────────────
# Usage:
#   nu scripts/bump_version.nu <new_version>
#
# Example:
#   nu scripts/bump_version.nu 0.5.0
#
# What it does:
#   1. Validates the supplied semantic version string.
#   2. Updates `workspace.package.version` in the root Cargo.toml.
#   3. Updates the `cleansys-core` dependency version wherever it appears.
#   4. Updates the version badge in README.md (if present).
#   5. Runs `cargo fmt`, `cargo clippy`, and `cargo test`.
#   6. Generates / updates the CHANGELOG via git-cliff (if installed).
#   7. Creates a Git commit and an annotated tag.
# ──────────────────────────────────────────────────────────────────────────────

# ── Helpers ───────────────────────────────────────────────────────────────────

# Validate that a string looks like a semver (MAJOR.MINOR.PATCH with optional pre-release).
def validate_version [version: string] {
    let pattern = '^\d+\.\d+\.\d+(-[a-zA-Z0-9.]+)?$'
    if ($version | find --regex $pattern | is-empty) {
        print $"(ansi red)Error:(ansi reset) '($version)' is not a valid semantic version."
        exit 1
    }
}

# Replace the workspace.package.version line in the root Cargo.toml.
def update_workspace_version [version: string] {
    let cargo = (open Cargo.toml --raw)
    let updated = ($cargo | str replace --regex 'version\s*=\s*"[^"]+"' $'version    = "($version)"' )
    $updated | save --force Cargo.toml
    print $"(ansi green)✓(ansi reset) Updated workspace.package.version → ($version)"
}

# Update the cleansys-core dependency version in the root Cargo.toml.
def update_core_dep_version [version: string] {
    let cargo = (open Cargo.toml --raw)
    let lines = ($cargo | lines)
    let updated_lines = ($lines | each {|line|
        if ($line | find --regex '^cleansys-core\s*=' | is-not-empty) {
            $line | str replace --regex 'version\s*=\s*"[^"]+"' $'version = "($version)"'
        } else {
            $line
        }
    })
    $updated_lines | str join "\n" | save --force Cargo.toml
    print $"(ansi green)✓(ansi reset) Updated cleansys-core dependency → ($version)"
}

# Update the crates.io version badge in README.md (if the badge exists).
def update_readme_badge [version: string] {
    if not ("README.md" | path exists) {
        print $"(ansi yellow)⚠(ansi reset) README.md not found — skipping badge update."
        return
    }
    let readme = (open README.md --raw)
    if ($readme =~ 'version-[0-9]+\.[0-9]+\.[0-9]+-blue') {
        let updated = (
            $readme
            | str replace --all --regex 'version-[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9]+)?-blue' $"version-($version)-blue"
        )
        $updated | save --force README.md
        print $"(ansi green)✓(ansi reset) Updated README.md version badge."
    } else {
        print $"(ansi yellow)⚠(ansi reset) No version badge found in README.md — skipping."
    }
}

# ── Main ──────────────────────────────────────────────────────────────────────

def main [
    new_version: string,  # New version in X.Y.Z format
    --yes (-y),           # Skip confirmation prompt (non-interactive)
] {
    print ""
    print $"(ansi cyan)══════════════════════════════════════════════════════════════(ansi reset)"
    print $"(ansi cyan)  CleanSys — Bump Version(ansi reset)"
    print $"(ansi cyan)══════════════════════════════════════════════════════════════(ansi reset)"
    print ""

    # 0. Read current version
    let current_version = (open Cargo.toml | get workspace.package.version)
    print $"  Current version : (ansi yellow)($current_version)(ansi reset)"
    print $"  New version     : (ansi green)($new_version)(ansi reset)"
    print ""

    if $current_version == $new_version {
        print $"(ansi yellow)⚠(ansi reset) Version is already ($new_version). Nothing to do."
        exit 0
    }

    # 1. Validate
    validate_version $new_version
    print $"(ansi green)✓(ansi reset) Version string validated."

    # 2. Update workspace version
    update_workspace_version $new_version

    # 3. Update cleansys-core dependency version
    update_core_dep_version $new_version

    # 4. Update README badge
    update_readme_badge $new_version

    # 5. cargo fmt
    print ""
    print $"(ansi cyan)── cargo fmt ───────────────────────────────────────────────(ansi reset)"
    run-external "cargo" "fmt" "--all"
    print $"(ansi green)✓(ansi reset) cargo fmt completed."

    # 6. cargo clippy
    print ""
    print $"(ansi cyan)── cargo clippy ────────────────────────────────────────────(ansi reset)"
    run-external "cargo" "clippy" "--workspace" "--all-targets" "--" "-D" "warnings" "-A" "deprecated"
    print $"(ansi green)✓(ansi reset) cargo clippy passed."

    # 7. cargo test
    print ""
    print $"(ansi cyan)── cargo test ──────────────────────────────────────────────(ansi reset)"
    run-external "cargo" "test" "--workspace"
    print $"(ansi green)✓(ansi reset) cargo test passed."

    # 8. Update Cargo.lock to reflect new workspace version
    print ""
    print $"(ansi cyan)── cargo update ────────────────────────────────────────────(ansi reset)"
    run-external "cargo" "update" "-p" "cleansys-core" "-p" "cleansys-tui" "-p" "cleansys-gui"
    print $"(ansi green)✓(ansi reset) Cargo.lock updated."

    # 9. Changelog (git-cliff)
    print ""
    print $"(ansi cyan)── changelog ───────────────────────────────────────────────(ansi reset)"
    if (which git-cliff | is-not-empty) {
        run-external "git-cliff" "--output" "CHANGELOG.md" "--tag" $"v($new_version)"
        print $"(ansi green)✓(ansi reset) CHANGELOG.md updated via git-cliff."
    } else {
        print $"(ansi yellow)⚠(ansi reset) git-cliff not found — skipping changelog generation."
    }

    # 10. Git commit & tag
    print ""
    print $"(ansi cyan)── git commit & tag ────────────────────────────────────────(ansi reset)"
    run-external "git" "add" "-A"
    run-external "git" "commit" "-m" $"chore: bump version to ($new_version)"
    run-external "git" "tag" "-a" $"v($new_version)" "-m" $"Release v($new_version)"
    print $"(ansi green)✓(ansi reset) Committed and tagged v($new_version)."

    print ""
    print $"(ansi green)══════════════════════════════════════════════════════════════(ansi reset)"
    print $"(ansi green)  CleanSys version bumped to ($new_version) 🚀(ansi reset)"
    print $"(ansi green)══════════════════════════════════════════════════════════════(ansi reset)"
    print ""
    print "  Next steps:"
    print $"    git push origin main --tags"
    print ""
}
