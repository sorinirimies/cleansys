#!/usr/bin/env bash
# Takes the GUI screenshots used in the README (macOS only).
#
# Runs cleansys-gui against the synthetic demo HOME (demo/fixture.sh), keeps the
# window on top via CLEANSYS_GUI_TOP=1 and captures it by its on-screen rect.
# Output: demo/previews/gui-{wide,medium,narrow,schedule}.png
#
# Requires: osascript, screencapture (both ship with macOS).
set -euo pipefail
cd "$(dirname "$0")/.."
REPO="$PWD"
[ "$(uname)" = "Darwin" ] || { echo "macOS only" >&2; exit 1; }
BIN="$REPO/target/release/cleansys-gui"
[ -x "$BIN" ] || { echo "build first: cargo build --release -p cleansys-gui" >&2; exit 1; }

# JXA helper: prints the bounds of the largest window owned by a pid.
WINJS="$(mktemp -t cleansys-win).js"
cat > "$WINJS" <<'JS'
ObjC.import('CoreGraphics');
function run(argv) {
  const pid = parseInt(argv[0]);
  const list = ObjC.deepUnwrap(ObjC.castRefToObject($.CGWindowListCopyWindowInfo(0, 0)));
  let best = null, area = 0;
  for (const w of list) {
    if (w.kCGWindowOwnerPID !== pid) continue;
    const a = w.kCGWindowBounds.Width * w.kCGWindowBounds.Height;
    if (a > area) { best = w; area = a; }
  }
  const b = best.kCGWindowBounds;
  return [b.X, b.Y, b.Width, b.Height].join(',');
}
JS
trap 'rm -f "$WINJS"' EXIT

shot() { # size out [extra env]
  # shellcheck disable=SC1091
  ( source demo/fixture.sh >/dev/null 2>&1
    cd "$REPO"
    env CLEANSYS_GUI_TOP=1 CLEANSYS_GUI_SIZE="$1" ${3:-} "$BIN" >/dev/null 2>&1 & pid=$!
    sleep 12   # let the background scan finish
    rect=$(osascript -l JavaScript "$WINJS" "$pid")
    sleep 1
    screencapture -x -R "$rect" "$2"
    kill "$pid" )
  echo "✓ $2"
}

mkdir -p demo/previews
shot 1180x780 demo/previews/gui-wide.png
shot 800x720  demo/previews/gui-medium.png
shot 460x820  demo/previews/gui-narrow.png
shot 1000x760 demo/previews/gui-schedule.png CLEANSYS_GUI_OPEN=schedule
