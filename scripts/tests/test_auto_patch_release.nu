#!/usr/bin/env nu
# ── CleanSys · test_auto_patch_release.nu ───────────────────────────────────
# Tests for scripts/ci/auto_patch_release.nu — next_patch_version().

use std/assert
use runner.nu *
use ../ci/auto_patch_release.nu [next_patch_version]

def "test next_patch_version: increments patch segment" [] {
    assert equal (next_patch_version "1.2.3") "1.2.4"
}

def "test next_patch_version: leaves major/minor untouched" [] {
    assert equal (next_patch_version "0.4.0") "0.4.1"
    assert equal (next_patch_version "2.0.9") "2.0.10"
}

def "test next_patch_version: handles double-digit patch rollover" [] {
    assert equal (next_patch_version "1.0.9") "1.0.10"
    assert equal (next_patch_version "1.0.99") "1.0.100"
}

def "test next_patch_version: drops pre-release suffix before bumping" [] {
    assert equal (next_patch_version "1.2.3-rc.1") "1.2.4"
    assert equal (next_patch_version "0.4.0-beta") "0.4.1"
}

def "test next_patch_version: zero version bumps to 0.0.1" [] {
    assert equal (next_patch_version "0.0.0") "0.0.1"
}

def "test next_patch_version: rejects missing patch segment" [] {
    let failed = (try { next_patch_version "1.2"; false } catch { true })
    assert $failed
}

def "test next_patch_version: rejects four-segment version" [] {
    let failed = (try { next_patch_version "1.2.3.4"; false } catch { true })
    assert $failed
}

def "test next_patch_version: rejects non-numeric segment" [] {
    let failed = (try { next_patch_version "1.2.x"; false } catch { true })
    assert $failed
}

def "test next_patch_version: is idempotent for repeated calls with same input" [] {
    let a = (next_patch_version "3.3.3")
    let b = (next_patch_version "3.3.3")
    assert equal $a $b
}

def main [] { run-tests }
