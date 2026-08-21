#!/usr/bin/env nu
# ──────────────────────────────────────────────────────────────────────────────
# CleanSys — Automatic patch release after a nightly dependency upgrade
# ──────────────────────────────────────────────────────────────────────────────
# Called by the "Nightly Dependency Upgrade" workflow immediately after it has
# committed a dependency bump to `main`. Computes the next patch version,
# then delegates to `scripts/bump_version.nu` (fmt + clippy + test + changelog
# + commit + tag) and pushes the result — fully automatic, no PR/approval.
#
# Usage:
#   nu scripts/ci/auto_patch_release.nu
# ──────────────────────────────────────────────────────────────────────────────

# Compute the next patch version for a `MAJOR.MINOR.PATCH` (optionally with a
# `-prerelease` suffix, which is dropped) semantic version string.
#
# Pure function — no I/O — so it can be unit tested directly.
export def next_patch_version [current: string]: nothing -> string {
    # Drop any pre-release suffix (e.g. "1.2.3-rc.1" -> "1.2.3") before bumping.
    let base = ($current | split row "-" | first)
    let parts = ($base | split row ".")

    if ($parts | length) != 3 {
        error make {
            msg: $"'($current)' is not a MAJOR.MINOR.PATCH semantic version"
        }
    }

    let major = ($parts | get 0 | into int)
    let minor = ($parts | get 1 | into int)
    let patch = ($parts | get 2 | into int) + 1

    $"($major).($minor).($patch)"
}

def main [] {
    let current_version = (open Cargo.toml | get workspace.package.version)
    let new_version = (next_patch_version $current_version)

    print $"(ansi cyan)══════════════════════════════════════════════════════════════(ansi reset)"
    print $"(ansi cyan)  CleanSys — Automatic patch release(ansi reset)"
    print $"(ansi cyan)══════════════════════════════════════════════════════════════(ansi reset)"
    print $"  Current version : (ansi yellow)($current_version)(ansi reset)"
    print $"  New version     : (ansi green)($new_version)(ansi reset)"
    print ""

    # bump_version.nu runs fmt/clippy/test/changelog and creates the commit + tag.
    run-external "nu" "scripts/bump_version.nu" "--yes" $new_version

    print ""
    print "Pushing commit and tag…"
    run-external "git" "push" "--follow-tags" "origin" "main"

    print $"(ansi green)✅ Patch release v($new_version) pushed — Release workflow will trigger automatically.(ansi reset)"
}
