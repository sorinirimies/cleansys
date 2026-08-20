#!/usr/bin/env nu
# ── CleanSys · test_version.nu ──────────────────────────────────────────────
# Tests for scripts/version.nu — reading the workspace version from Cargo.toml.

use std/assert
use runner.nu *

# ── Helpers ─────────────────────────────────────────────────────────────────

# Build a minimal workspace Cargo.toml string with the given version.
def make_workspace_cargo [version: string]: nothing -> string {
    $'[workspace]
members = [
    "crates/cleansys-core",
]
resolver = "2"

[workspace.package]
version = "($version)"
edition = "2021"

[workspace.dependencies]
cleansys-core = { path = "crates/cleansys-core", version = "($version)" }
'
}

# Extract the workspace version the same way version.nu does:
# read [workspace.package] version from a Cargo.toml string.
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

# ── Tests ───────────────────────────────────────────────────────────────────

def "test version: reads simple semver" [] {
    let cargo = (make_workspace_cargo "1.2.3")
    let ver = (read_workspace_version $cargo)
    assert equal $ver "1.2.3"
}

def "test version: reads pre-release version" [] {
    let cargo = (make_workspace_cargo "0.5.0-rc.1")
    let ver = (read_workspace_version $cargo)
    assert equal $ver "0.5.0-rc.1"
}

def "test version: reads zero version" [] {
    let cargo = (make_workspace_cargo "0.0.0")
    let ver = (read_workspace_version $cargo)
    assert equal $ver "0.0.0"
}

def "test version: reads large version numbers" [] {
    let cargo = (make_workspace_cargo "12.345.6789")
    let ver = (read_workspace_version $cargo)
    assert equal $ver "12.345.6789"
}

def "test version: workspace dep version matches workspace version" [] {
    let version = "2.0.0"
    let cargo = (make_workspace_cargo $version)

    # The cleansys-core dependency line should carry the same version.
    let dep_line = (
        $cargo
        | lines
        | where { |l| $l =~ '^cleansys-core\s*=' }
        | first
    )

    let dep_ver = (
        $dep_line
        | parse --regex 'version\s*=\s*"(?P<ver>[^"]+)"'
        | get ver
        | first
    )

    assert equal $dep_ver $version
}

def "test version: rejects cargo without version" [] {
    let cargo = '[workspace]
members = ["crates/cleansys-core"]
'
    # Should fail because there is no version line.
    let failed = (try { read_workspace_version $cargo; false } catch { true })
    assert $failed
}

# ── Main ────────────────────────────────────────────────────────────────────

def main [] { run-tests }
