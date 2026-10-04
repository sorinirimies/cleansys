#!/usr/bin/env nu
# ──────────────────────────────────────────────────────────────────────────────
# CleanSys — Did any *direct* dependency actually change?
# ──────────────────────────────────────────────────────────────────────────────
# Cargo.lock moves almost daily because of transitive patch bumps (syn, libc,
# tokio, ...). Those are worth committing, but not worth a new release.
# A release is only warranted when a crate we declare in
# [workspace.dependencies] resolved to a different version.
#
# Prints "true" / "false" on stdout.
#
# Usage:
#   nu scripts/ci/direct_deps_changed.nu --before <old Cargo.lock> --after Cargo.lock
# ──────────────────────────────────────────────────────────────────────────────

# Sorted unique versions locked for `name` in a parsed package table.
export def versions_of [pkgs: table, name: string]: nothing -> list<string> {
    $pkgs | where name == $name | get version | uniq | sort
}

# Names of direct deps whose locked version set differs. Pure — unit testable.
export def changed_direct [
    direct: list<string>
    before: table
    after: table
]: nothing -> list<string> {
    $direct | where {|n| (versions_of $before $n) != (versions_of $after $n) }
}

def main [
    --before: string
    --after: string = "Cargo.lock"
    --manifest: string = "Cargo.toml"
] {
    let direct = (open $manifest | get workspace.dependencies | columns
        | where {|n| $n != "cleansys-core" })
    let b = (open --raw $before | from toml | get package | select name version)
    let a = (open --raw $after | from toml | get package | select name version)
    let changed = (changed_direct $direct $b $a)

    # Human-readable detail goes to stderr so stdout stays "true"/"false".
    if ($changed | is-empty) {
        print -e "No direct dependency changed (transitive-only lock churn)."
        print "false"
    } else {
        print -e $"Direct dependencies changed: ($changed | str join ', ')"
        print "true"
    }
}
