#!/usr/bin/env nu

# ──────────────────────────────────────────────────────────────
#  CleanSys – CI Release Notes Generator
# ──────────────────────────────────────────────────────────────
#  Called by the release workflow AFTER the tag has been pushed
#  and checked out.  Generates:
#    - CHANGELOG.md      (full history, updated in place)
#    - RELEASE_NOTES.md  (single-release body used by softprops/action-gh-release)
#
#  Usage:
#    nu scripts/ci/release_notes.nu v1.2.3
# ──────────────────────────────────────────────────────────────

def main [raw_tag: string] {
    let version = ($raw_tag | str replace --regex '^v' '')
    let tag     = $"v($version)"

    print $"(ansi cyan)═══ Release Notes — ($tag) ═══(ansi reset)"

    # ── 1. Regenerate the full CHANGELOG.md ───────────────────
    if (which git-cliff | is-not-empty) {
        print "  Regenerating CHANGELOG.md…"
        run-external "git-cliff" "--output" "CHANGELOG.md"
        print "  ✔ CHANGELOG.md updated"
    } else {
        print "  ⚠ git-cliff not found — CHANGELOG.md not updated"
    }

    # ── 2. Extract per-release notes via --latest ─────────────
    let cliff_changes = if (which git-cliff | is-not-empty) {
        let result = (do { run-external "git-cliff" "--latest" "--strip" "header" } | complete)
        if $result.exit_code == 0 and ($result.stdout | str trim | is-not-empty) {
            $result.stdout | str trim
        } else {
            "- See commit history for details."
        }
    } else {
        "- See commit history for details."
    }

    # ── 3. Build RELEASE_NOTES.md ─────────────────────────────
    let notes = [
        $"# CleanSys ($version)"
        ""
        "## Installation"
        ""
        "```sh"
        "# Terminal UI + CLI"
        "cargo install cleansys-tui"
        ""
        "# Desktop GUI"
        "cargo install cleansys-gui"
        "```"
        ""
        "Or download pre-built binaries/installers for your platform from this release."
        ""
        "## What's Changed"
        ""
        $cliff_changes
        ""
        "## Crates"
        ""
        "| Crate | Version |"
        "|-------|---------|"
        $"| `cleansys-tui`  | ($version) |"
        $"| `cleansys-gui`  | ($version) |"
        $"| `cleansys-core` | ($version) |"
    ] | str join "\n"

    $notes | save --force RELEASE_NOTES.md
    print "  ✔ RELEASE_NOTES.md written"
    print $"(ansi green)✅ Release artifacts ready for ($tag)(ansi reset)"
}
