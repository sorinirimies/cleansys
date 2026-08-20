#!/usr/bin/env nu
# ── CleanSys · test_workspace_structure.nu ──────────────────────────────────
# Sanity-checks the Cargo workspace layout: every member listed in the root
# Cargo.toml actually exists on disk and has its own Cargo.toml + src/ dir.

use std/assert
use runner.nu *

def workspace_root []: nothing -> string {
    # scripts/tests/ -> scripts/ -> repo root
    $env.CURRENT_FILE | path dirname | path dirname | path dirname
}

def "test workspace: root Cargo.toml declares expected members" [] {
    let root = (workspace_root)
    let cargo = (open ($root | path join "Cargo.toml"))
    let members = ($cargo | get workspace.members)

    assert ("crates/cleansys-core" in $members)
    assert ("crates/cleansys-tui" in $members)
    assert ("crates/cleansys-gui" in $members)
}

def "test workspace: every member directory exists with a Cargo.toml" [] {
    let root = (workspace_root)
    let cargo = (open ($root | path join "Cargo.toml"))
    let members = ($cargo | get workspace.members)

    for member in $members {
        let member_path = ($root | path join $member)
        assert ($member_path | path exists) $"missing member directory: ($member)"
        assert (($member_path | path join "Cargo.toml") | path exists) $"missing Cargo.toml for: ($member)"
        assert (($member_path | path join "src") | path exists) $"missing src/ for: ($member)"
    }
}

def "test workspace: cleansys-core has no TUI or GUI dependencies" [] {
    let root = (workspace_root)
    let core_cargo = (open ($root | path join "crates/cleansys-core/Cargo.toml" ) --raw)

    assert (not ($core_cargo | str contains "ratatui"))
    assert (not ($core_cargo | str contains "crossterm"))
    assert (not ($core_cargo | str contains "iced"))
}

def "test workspace: tui and gui both depend on cleansys-core" [] {
    let root = (workspace_root)
    let tui_cargo = (open ($root | path join "crates/cleansys-tui/Cargo.toml") --raw)
    let gui_cargo = (open ($root | path join "crates/cleansys-gui/Cargo.toml") --raw)

    assert ($tui_cargo | str contains "cleansys-core")
    assert ($gui_cargo | str contains "cleansys-core")
}

def main [] { run-tests }
