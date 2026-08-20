#!/usr/bin/env nu
# ── CleanSys · test_bump_version.nu ─────────────────────────────────────────
# Tests the semantic-version validation regex used by scripts/bump_version.nu.

use std/assert
use runner.nu *

def is_valid_semver [version: string]: nothing -> bool {
    let pattern = '^\d+\.\d+\.\d+(-[a-zA-Z0-9.]+)?$'
    ($version | find --regex $pattern | is-not-empty)
}

def "test bump_version: accepts plain semver" [] {
    assert (is_valid_semver "1.2.3")
    assert (is_valid_semver "0.0.1")
    assert (is_valid_semver "10.20.30")
}

def "test bump_version: accepts pre-release semver" [] {
    assert (is_valid_semver "1.0.0-rc.1")
    assert (is_valid_semver "0.5.0-beta")
}

def "test bump_version: rejects missing patch version" [] {
    assert (not (is_valid_semver "1.2"))
}

def "test bump_version: rejects non-numeric version" [] {
    assert (not (is_valid_semver "abc"))
}

def "test bump_version: rejects version with leading v" [] {
    assert (not (is_valid_semver "v1.2.3"))
}

def "test bump_version: rejects empty string" [] {
    assert (not (is_valid_semver ""))
}

def main [] { run-tests }
