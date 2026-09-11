#!/usr/bin/env nu
# ──────────────────────────────────────────────────────────────────────────────
# CleanSys — Guard against accidental dependency downgrades
# ──────────────────────────────────────────────────────────────────────────────
# Compares a "before" Cargo.lock snapshot against the current (post-upgrade)
# Cargo.lock and fails loudly if any package's resolved version went *down*.
#
# `cargo upgrade` / `cargo update` should never downgrade a crate on their own,
# but registry hiccups, yanked releases, or manual edits could still produce
# one — this script is the explicit, auditable safety net the nightly
# dependency-upgrade workflows rely on before they ever commit or cut a
# release.
#
# Usage:
#   cp Cargo.lock Cargo.lock.before      # snapshot BEFORE running the upgrade
#   ... run cargo upgrade / cargo update ...
#   nu scripts/ci/check_no_downgrade.nu --before Cargo.lock.before --after Cargo.lock
#
# Exits non-zero (and prints every offending crate) if a downgrade is found.
# ──────────────────────────────────────────────────────────────────────────────

# Parse a Cargo.lock file into a table of {name, version} for every
# `[[package]]` entry. Pure-ish (only does file I/O), safe to unit test by
# feeding it a temp file path.
export def parse_lock [path: string]: nothing -> table<name: string, version: string> {
    let raw = (open --raw $path | from toml)
    $raw | get package | select name version
}

# Compare two package tables and return the list of crates whose version
# decreased from `before` to `after`. Pure function — unit testable.
export def find_downgrades [
    before: table<name: string, version: string>
    after: table<name: string, version: string>
]: nothing -> table<name: string, before: string, after: string> {
    $after
    | each { |pkg|
        let prev = ($before | where name == $pkg.name)
        if ($prev | is-empty) {
            null
        } else {
            # A crate can appear multiple times in Cargo.lock only when
            # different major-version lines are resolved simultaneously
            # (rare, but valid). Downgrade == the *lowest* prior version for
            # this name is higher than the *highest* new version for it.
            let prev_versions = ($prev | get version | sort --natural | reverse)
            let after_versions = ($after | where name == $pkg.name | get version | sort --natural | reverse)
            let best_prev = ($prev_versions | first)
            let best_after = ($after_versions | first)
            if ((version_gt $best_prev $best_after)) {
                { name: $pkg.name, before: $best_prev, after: $best_after }
            } else {
                null
            }
        }
    }
    | where {|x| $x != null }
    | uniq-by name
}

def segment_cmp [a: string, b: string]: nothing -> int {
    let an = ($a | into int) 
    let bn = ($b | into int)
    if $an > $bn { 1 } else if $an < $bn { -1 } else { 0 }
}

def safe_segment_cmp [a: string, b: string]: nothing -> int {
    let result = (try { segment_cmp $a $b } catch { null })
    if $result != null {
        $result
    } else {
        if $a > $b { 1 } else if $a < $b { -1 } else { 0 }
    }
}

# Basic SemVer-ish comparison: true if `a` is strictly greater than `b`.
# Falls back to string comparison for non-numeric / malformed segments.
export def version_gt [a: string, b: string]: nothing -> bool {
    let av = ($a | split row "." | each { |x| ($x | split row "-" | first | split row "+" | first) })
    let bv = ($b | split row "." | each { |x| ($x | split row "-" | first | split row "+" | first) })
    let len = ([($av | length) ($bv | length)] | math max)

    mut result = 0
    for i in 0..<$len {
        let ai = ($av | get -o $i | default "0")
        let bi = ($bv | get -o $i | default "0")
        let cmp = (safe_segment_cmp $ai $bi)
        if $result == 0 and $cmp != 0 {
            $result = $cmp
        }
    }
    $result > 0
}

def main [
    --before: string = "Cargo.lock.before"  # snapshot taken before the upgrade
    --after: string = "Cargo.lock"          # current lockfile after the upgrade
] {
    if not ($before | path exists) {
        print $"⚠️  No 'before' snapshot at ($before) — skipping downgrade check."
        exit 0
    }

    let before_pkgs = (parse_lock $before)
    let after_pkgs = (parse_lock $after)
    let downgrades = (find_downgrades $before_pkgs $after_pkgs)

    if ($downgrades | is-empty) {
        print "✅ No dependency downgrades detected."
    } else {
        print "❌ Dependency downgrade(s) detected — aborting:"
        print ($downgrades | table)
        exit 1
    }
}
