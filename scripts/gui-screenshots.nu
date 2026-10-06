#!/usr/bin/env nu
# Takes the GUI screenshots used in the README (macOS only).
#
#   nu scripts/gui-screenshots.nu
#
# Runs cleansys-gui against the synthetic demo HOME (demo/fixture.nu), keeps the
# window on top (CLEANSYS_GUI_TOP=1) and captures it by its on-screen rect.
# Output: demo/previews/gui-{wide,medium,narrow,schedule}.png
#
# Requires: macOS (osascript, screencapture) and a release build of cleansys-gui.

source-env ../demo/fixture.nu

const ROOT = (path self | path dirname | path dirname)

def shot [size: string, out: string, extra: record = {}] {
    let bin = ($ROOT | path join "target" "release" "cleansys-gui")
    let vars = ({CLEANSYS_GUI_TOP: "1", CLEANSYS_GUI_SIZE: $size} | merge $extra)
    let j = (job spawn { with-env $vars { ^$bin | ignore } })
    sleep 12sec   # let the background scan finish
    let rect = (^osascript -l JavaScript ($ROOT | path join "scripts" "lib" "window_rect.js") "cleansys-gui" | str trim)
    if ($rect | is-empty) {
        ^pkill -f "release/cleansys-gui"
        error make {msg: $"no window found for ($out)"}
    }
    sleep 1sec
    ^screencapture -x -R $rect $out
    ^pkill -f "release/cleansys-gui"
    try { job kill $j }
    print $"✓ ($out)"
}

def main [] {
    if (sys host | get name) != "Darwin" {
        error make {msg: "macOS only (osascript + screencapture)"}
    }
    let bin = ($ROOT | path join "target" "release" "cleansys-gui")
    if not ($bin | path exists) {
        error make {msg: "build first: cargo build --release -p cleansys-gui"}
    }
    let out = ($ROOT | path join "demo" "previews")
    mkdir $out
    shot "1180x780" ($out | path join "gui-wide.png")
    shot "800x720" ($out | path join "gui-medium.png")
    shot "460x820" ($out | path join "gui-narrow.png")
    shot "1000x760" ($out | path join "gui-schedule.png") {CLEANSYS_GUI_OPEN: "schedule"}
}
