#!/usr/bin/env nu
# ── CleanSys · test_package_linux.nu ────────────────────────────────────────
# Tests for scripts/ci/package_linux.nu — Linux .deb/.rpm packaging helpers.

use std/assert
use runner.nu *
use ../ci/package_linux.nu [alien-args, is-linux-package-file, gui-deb-control, tui-deb-control]

# ── alien-args ───────────────────────────────────────────────────────────────

def "test alien-args: returns exactly three args" [] {
    let args = (alien-args "dist/cleansys_0.4.0_amd64.deb")
    assert equal ($args | length) 3
}

def "test alien-args: is --to-rpm --scripts <deb_path>" [] {
    let args = (alien-args "dist/cleansys_0.4.0_amd64.deb")
    assert equal ($args | get 0) "--to-rpm"
    assert equal ($args | get 1) "--scripts"
    assert equal ($args | get 2) "dist/cleansys_0.4.0_amd64.deb"
}

def "test alien-args: does not pass --target or a BuildArch-style flag" [] {
    let args = (alien-args "dist/cleansys-gui_2.0.0_arm64.deb")
    assert (not ($args | any { |it| $it == "--target" }))
    assert (not ($args | any { |it| $it | str contains "BuildArch" }))
}

def "test alien-args: works for the GUI deb path too" [] {
    let deb = "dist/cleansys-gui_9.9.9_arm64.deb"
    let args = (alien-args $deb)
    assert equal ($args | length) 3
    assert equal ($args | get 2) $deb
}

# ── is-linux-package-file ────────────────────────────────────────────────────

def "test is-linux-package-file: matches .deb files" [] {
    assert (is-linux-package-file "cleansys_1.2.3_amd64.deb")
}

def "test is-linux-package-file: matches .rpm files" [] {
    assert (is-linux-package-file "cleansys-1.2.3-1.x86_64.rpm")
}

def "test is-linux-package-file: rejects unrelated files" [] {
    assert (not (is-linux-package-file "cleansys"))
    assert (not (is-linux-package-file "cleansys.tar.gz"))
    assert (not (is-linux-package-file "notes.txt"))
}

def "test is-linux-package-file: does not match substrings mid-name" [] {
    assert (not (is-linux-package-file "deb-tui-staging-dir"))
    assert (not (is-linux-package-file "rpmbuild-notes.md"))
}

def "test is-linux-package-file: empty string is not a package" [] {
    assert (not (is-linux-package-file ""))
}

# ── gui-deb-control / tui-deb-control ────────────────────────────────────────

def "test gui-deb-control: contains the literal Iced parenthetical" [] {
    let control = (gui-deb-control "1.1.6" "amd64")
    assert ($control | str contains "(Iced)")
}

def "test gui-deb-control: interpolates version and architecture" [] {
    let control = (gui-deb-control "9.9.9" "arm64")
    assert ($control | str contains "Version: 9.9.9")
    assert ($control | str contains "Architecture: arm64")
}

def "test gui-deb-control: has the expected package name and maintainer" [] {
    let control = (gui-deb-control "1.0.0" "amd64")
    assert ($control | str contains "Package: cleansys-gui")
    assert ($control | str contains "Maintainer: Sorin Albu-Irimies")
}

def "test tui-deb-control: has the expected package name" [] {
    let control = (tui-deb-control "1.0.0" "amd64")
    assert ($control | str contains "Package: cleansys")
    assert (not ($control | str contains "Package: cleansys-gui"))
}

def "test tui-deb-control: interpolates version and architecture" [] {
    let control = (tui-deb-control "3.2.1" "arm64")
    assert ($control | str contains "Version: 3.2.1")
    assert ($control | str contains "Architecture: arm64")
}

# ── Parse-level regression guard ─────────────────────────────────────────────

def "test package_linux.nu: parses without syntax errors" [] {
    assert (nu-check ($env.CURRENT_FILE | path dirname | path join ".." "ci" "package_linux.nu"))
}

def "test package_linux.nu: parses cleanly as a module too" [] {
    assert (nu-check --as-module ($env.CURRENT_FILE | path dirname | path join ".." "ci" "package_linux.nu"))
}

def "test package_linux.nu: uses into glob for the dynamic .rpm lookup pattern" [] {
    let text = (open --raw ($env.CURRENT_FILE | path dirname | path join ".." "ci" "package_linux.nu"))
    assert ($text | str contains "into glob")
}

def "test package_linux.nu: does not invoke rpmbuild" [] {
    let text = (open --raw ($env.CURRENT_FILE | path dirname | path join ".." "ci" "package_linux.nu"))
    assert (not ($text | str contains "run-external \"rpmbuild\""))
    assert (not ($text | str contains "%install"))
}

# ── Main ────────────────────────────────────────────────────────────────────

def main [] { run-tests }
