#!/usr/bin/env nu
# ──────────────────────────────────────────────────────────────────────────────
# CleanSys — Pre-publish checks
# ──────────────────────────────────────────────────────────────────────────────
# Runs documentation checks, dry-run publish, and cargo check for all crates
# in the workspace before an actual `cargo publish`.
#
# Usage:
#   nu scripts/check_publish.nu
# ──────────────────────────────────────────────────────────────────────────────

def main [] {
    print "══════════════════════════════════════════════════════════"
    print "  CleanSys — Pre-publish checks"
    print "══════════════════════════════════════════════════════════"
    print ""

    # ── 1. Documentation checks ───────────────────────────────────────────────
    print "── Step 1: Documentation checks ──"
    let doc_crates = ["cleansys-tui" "cleansys-gui"]

    for crate in $doc_crates {
        print $"  📖 Checking docs for ($crate)..."
        let result = (do { cargo doc -p $crate --no-deps } | complete)
        if $result.exit_code != 0 {
            print $"  ❌ Doc check failed for ($crate):"
            print $result.stderr
            exit 1
        }
        print $"  ✅ ($crate) docs OK"
    }
    print ""

    # ── 2. Publish dry-run for cleansys-core ──────────────────────────────────
    print "── Step 2: Publish dry-run (cleansys-core) ──"
    print "  📦 Running cargo publish --dry-run for cleansys-core..."
    let publish_result = (do { cargo publish --dry-run -p cleansys-core } | complete)
    if $publish_result.exit_code != 0 {
        print "  ❌ Publish dry-run failed for cleansys-core:"
        print $publish_result.stderr
        exit 1
    }
    print "  ✅ cleansys-core publish dry-run OK"
    print ""

    # ── 3. Cargo check for TUI and GUI crates ────────────────────────────────
    print "── Step 3: Cargo check (TUI & GUI) ──"
    let check_crates = ["cleansys-tui" "cleansys-gui"]

    for crate in $check_crates {
        print $"  🔍 Running cargo check for ($crate)..."
        let check_result = (do { cargo check -p $crate } | complete)
        if $check_result.exit_code != 0 {
            print $"  ❌ Cargo check failed for ($crate):"
            print $check_result.stderr
            exit 1
        }
        print $"  ✅ ($crate) check OK"
    }
    print ""

    # ── 4. Cargo clippy (workspace) ───────────────────────────────────────────
    print "── Step 4: Cargo clippy (workspace) ──"
    print "  🔎 Running cargo clippy --workspace..."
    let clippy_result = (do { cargo clippy --workspace --all-targets --all-features -- -D warnings -A deprecated } | complete)
    if $clippy_result.exit_code != 0 {
        print "  ❌ Clippy found warnings/errors:"
        print $clippy_result.stderr
        exit 1
    }
    print "  ✅ Clippy passed"
    print ""

    # ── 5. Cargo test (workspace) ─────────────────────────────────────────────
    print "── Step 5: Cargo test (workspace) ──"
    print "  🧪 Running cargo test --workspace..."
    let test_result = (do { cargo test --workspace } | complete)
    if $test_result.exit_code != 0 {
        print "  ❌ Tests failed:"
        print $test_result.stderr
        exit 1
    }
    print "  ✅ All tests passed"
    print ""

    # ── Summary ───────────────────────────────────────────────────────────────
    print "══════════════════════════════════════════════════════════"
    print "  ✅ All pre-publish checks passed!"
    print ""
    print "  Publish order:"
    print "    1. cargo publish -p cleansys-core"
    print "    2. cargo publish -p cleansys-tui"
    print "    3. cargo publish -p cleansys-gui"
    print ""
    print "  Wait ~30-60 seconds between publishes for crates.io"
    print "  to index each crate before its dependents."
    print "══════════════════════════════════════════════════════════"
}
