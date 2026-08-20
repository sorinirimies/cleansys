#!/usr/bin/env nu
# ── CleanSys · test_upgrade_deps.nu ─────────────────────────────────────────
# Tests for scripts/upgrade_deps.nu — commit_label, all_passed pure functions,
# and workspace invariant checks.

use std/assert
use runner.nu *
use ../upgrade_deps.nu [commit_label all_passed]

# ── Helpers ─────────────────────────────────────────────────────────────────

# Build a minimal workspace Cargo.toml with the given version.
def make_workspace_cargo [version: string]: nothing -> string {
    $'[workspace]
members = [
    "crates/cleansys-core",
    "crates/cleansys-gui",
    "crates/cleansys-tui",
]
resolver = "2"

[workspace.package]
version = "($version)"
edition = "2021"

[workspace.dependencies]
cleansys-core = { path = "crates/cleansys-core", version = "($version)" }
'
}

# Read the workspace.package version from a Cargo.toml string.
def read_workspace_version [cargo_toml: string]: nothing -> string {
    $cargo_toml
    | lines
    | each { |l| $l | str trim }
    | where { |l| $l | str starts-with 'version' }
    | where { |l| not ($l | str contains 'workspace = true') }
    | first
    | parse --regex 'version\s*=\s*"(?P<ver>[^"]+)"'
    | get ver
    | first
}

# Read the cleansys-core dependency version from a workspace Cargo.toml string.
def read_core_dep_version [cargo_toml: string]: nothing -> string {
    let dep_line = (
        $cargo_toml
        | lines
        | where { |l| $l =~ '^cleansys-core\s*=' }
        | first
    )
    $dep_line
    | parse --regex 'version\s*=\s*"(?P<ver>[^"]+)"'
    | get ver
    | first
}

# ── Tests: commit_label ─────────────────────────────────────────────────────

def "test commit_label: upgrade when toml dirty" [] {
    let label = (commit_label true true "2026-03-24")
    assert ($label | str contains "upgrade")
    assert ($label | str contains "2026-03-24")
}

def "test commit_label: update when only lock dirty" [] {
    let label = (commit_label false true "2026-03-24")
    assert ($label | str contains "update")
    assert ($label | str contains "2026-03-24")
    assert (not ($label | str contains "upgrade"))
}

def "test commit_label: empty when nothing dirty" [] {
    let label = (commit_label false false "2026-03-24")
    assert ($label | is-empty)
}

def "test commit_label: toml dirty takes precedence" [] {
    let label = (commit_label true true "2026-01-01")
    assert ($label | str contains "upgrade")
}

def "test commit_label: upgrade contains chore prefix" [] {
    let label = (commit_label true false "2026-03-24")
    assert ($label | str starts-with "chore:")
}

def "test commit_label: update contains chore prefix" [] {
    let label = (commit_label false true "2026-03-24")
    assert ($label | str starts-with "chore:")
}

def "test commit_label: contains full date" [] {
    let label = (commit_label false true "2099-12-31")
    assert ($label | str contains "2099-12-31")
}

def "test commit_label: upgrade message is stable" [] {
    let a = (commit_label true true "2026-03-24")
    let b = (commit_label true true "2026-03-24")
    assert equal $a $b
}

def "test commit_label: update message is stable" [] {
    let a = (commit_label false true "2026-03-24")
    let b = (commit_label false true "2026-03-24")
    assert equal $a $b
}

# ── Tests: all_passed ────────────────────────────────────────────────────────

def "test all_passed: all true" [] {
    assert (all_passed [true, true, true])
}

def "test all_passed: single true" [] {
    assert (all_passed [true])
}

def "test all_passed: empty list" [] {
    assert (all_passed [])
}

def "test all_passed: one false" [] {
    assert (not (all_passed [true, false, true]))
}

def "test all_passed: all false" [] {
    assert (not (all_passed [false, false, false]))
}

def "test all_passed: first element false" [] {
    assert (not (all_passed [false, true, true, true]))
}

def "test all_passed: last element false" [] {
    assert (not (all_passed [true, true, true, false]))
}

def "test all_passed: single false" [] {
    assert (not (all_passed [false]))
}

def "test all_passed: six true mirrors full quality gate" [] {
    let gate = [true, true, true, true, true, true]
    assert (all_passed $gate)
}

def "test all_passed: six with one failure" [] {
    let gate = [true, false, true, true, true, true]
    assert (not (all_passed $gate))
}

# ── Tests: workspace invariants ──────────────────────────────────────────────

def "test workspace invariant: versions in sync" [] {
    let cargo = (make_workspace_cargo "0.5.0")
    let ws_ver = (read_workspace_version $cargo)
    let dep_ver = (read_core_dep_version $cargo)
    assert equal $ws_ver $dep_ver
}

def "test workspace invariant: versions in sync after bump" [] {
    let version = "1.2.3"
    let cargo = (make_workspace_cargo $version)
    let ws_ver = (read_workspace_version $cargo)
    let dep_ver = (read_core_dep_version $cargo)
    assert equal $ws_ver $version
    assert equal $dep_ver $version
}

def "test workspace invariant: dep line exists" [] {
    let cargo = (make_workspace_cargo "0.1.0")
    let dep_lines = (
        $cargo
        | lines
        | where { |l| $l =~ '^cleansys-core\s*=' }
    )
    assert equal ($dep_lines | length) 1
}

def "test workspace invariant: dep line has path and version" [] {
    let cargo = (make_workspace_cargo "4.0.0")
    let dep_line = (
        $cargo
        | lines
        | where { |l| $l =~ '^cleansys-core\s*=' }
        | first
    )
    assert ($dep_line | str contains 'path = "crates/cleansys-core"')
    assert ($dep_line | str contains 'version = "4.0.0"')
}

def "test workspace invariant: all three members listed" [] {
    let cargo = (make_workspace_cargo "0.1.0")
    assert ($cargo | str contains '"crates/cleansys-core"')
    assert ($cargo | str contains '"crates/cleansys-gui"')
    assert ($cargo | str contains '"crates/cleansys-tui"')
}

def "test workspace invariant: pre-release versions stay in sync" [] {
    let cargo = (make_workspace_cargo "1.0.0-alpha.3")
    let ws_ver = (read_workspace_version $cargo)
    let dep_ver = (read_core_dep_version $cargo)
    assert equal $ws_ver "1.0.0-alpha.3"
    assert equal $dep_ver "1.0.0-alpha.3"
}

# ── Main ────────────────────────────────────────────────────────────────────

def main [] { run-tests }
