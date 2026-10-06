#!/usr/bin/env bash
# Builds a throw-away HOME full of synthetic caches and projects so demos and
# screenshots never show real paths. Files are *sparse* (they report a size
# but use no disk), so this is instant and tiny.
#
#   source demo/fixture.sh        # exports HOME + CLEANSYS_SCAN_ROOTS
#
# Never touches your real home directory.
DEMO_HOME="${DEMO_HOME:-/tmp/cleansys-demo-home}"
rm -rf "$DEMO_HOME"
mkdir -p "$DEMO_HOME"

# f <size> <path>: sparse file of that apparent size (creates parents)
f() { mkdir -p "$(dirname "$2")"; truncate -s "$1" "$2"; }

H="$DEMO_HOME"

# ── Projects (build output is cleaned only next to a marker file) ──
P="$H/Projects"
echo '[package]' > /dev/null
mkdir -p "$P/rust-api" "$P/android-shop/app" "$P/web-dashboard" "$P/ml-notebooks" "$P/ios-weather"
echo '[package]
name = "rust-api"' > "$P/rust-api/Cargo.toml"
f 1400M "$P/rust-api/target/debug/deps/libbig.rlib"
f 700M  "$P/rust-api/target/debug/incremental/state.bin"
f 380M  "$P/rust-api/target/release/rust-api"

echo 'rootProject.name = "shop"' > "$P/android-shop/settings.gradle.kts"
echo 'plugins {}' > "$P/android-shop/build.gradle.kts"
echo 'plugins {}' > "$P/android-shop/app/build.gradle.kts"
f 640M "$P/android-shop/app/build/outputs/apk/debug/app-debug.apk"
f 210M "$P/android-shop/app/build/intermediates/dex.bin"
f 900M "$P/android-shop/.gradle/8.9/executionHistory.bin"
f 120M "$P/android-shop/build/reports.bin"

echo '{"name":"web-dashboard"}' > "$P/web-dashboard/package.json"
f 1100M "$P/web-dashboard/node_modules/.cache/big.bin"
f 420M  "$P/web-dashboard/node_modules/typescript/lib.bin"
f 310M  "$P/web-dashboard/.next/cache/webpack.bin"

echo '[project]
name = "ml-notebooks"' > "$P/ml-notebooks/pyproject.toml"
f 2400M "$P/ml-notebooks/.venv/lib/torch.bin"
f 12M   "$P/ml-notebooks/__pycache__/m.pyc"
f 30M   "$P/ml-notebooks/.pytest_cache/v/cache.bin"

echo '// swift-tools-version:5.9' > "$P/ios-weather/Package.swift"
f 760M "$P/ios-weather/.build/debug/weather.bin"

# Make every project look untouched for months (default: only idle projects are cleaned)
find "$P" -exec touch -t 202501150900 {} + 2>/dev/null

# ── Developer caches ──
f 3400M "$H/.gradle/caches/modules-2/files.bin"
f 210M  "$H/.gradle/daemon/8.9/daemon.log"
f 780M  "$H/.gradle/wrapper/dists/gradle-8.9-bin/dist.zip"
f 1800M "$H/.cargo/registry/src/index.crates.io/crates.bin"
f 420M  "$H/.cargo/git/checkouts/repo/checkout.bin"
f 700M  "$H/.npm/_cacache/content.bin"
f 520M  "$H/.m2/repository/org/lib.jar"
f 340M  "$H/.bun/install/cache/pkgs.bin"
f 400M  "$H/Library/Caches/uv/archive.bin"
f 90M   "$H/.android/cache/sdk.bin"
f 260M  "$H/Library/Caches/JetBrains/AndroidStudio2024.2/caches.bin"
f 180M  "$H/Library/Caches/ms-playwright/chromium/browser.bin"

# ── AI / LLM ──
f 6400M "$H/.cache/huggingface/hub/models--llama/blobs/weights.bin"
f 9200M "$H/.ollama/models/blobs/sha256-model"
f 3100M "$H/.cache/torch/hub/checkpoints/resnet.pth"
f 22M   "$H/.claude/shell-snapshots/snapshot.sh"
f 310M  "$H/.claude/projects/-Users-demo-Projects/session.jsonl"
f 140M  "$H/.codex/sessions/2025/rollout.jsonl"
f 18M   "$H/.pi/agent/npm/node_modules/pkg.bin"
f 64M   "$P/rust-api/.codegraph/graph.db"
f 48M   "$P/web-dashboard/.codegraph/graph.db"

# ── Browsers ──
f 1100M "$H/Library/Caches/Google/Chrome/Default/Cache/data_0"
f 240M  "$H/Library/Caches/Google/Chrome/Default/Code Cache/js.bin"
f 640M  "$H/Library/Caches/BraveSoftware/Brave-Browser/Default/Cache/data_0"
f 330M  "$H/Library/Caches/Firefox/Profiles/abc.default/cache2/entries.bin"
f 38M   "$H/Library/Application Support/Google/Chrome/Default/Cookies"
f 22M   "$H/Library/Application Support/Google/Chrome/Default/History"

# ── Apps ──
f 1500M "$H/Library/Application Support/Slack/Cache/Cache_Data/data_0"
f 220M  "$H/Library/Application Support/Slack/Code Cache/js.bin"
f 540M  "$H/Library/Application Support/discord/Cache/Cache_Data/data_0"
f 900M  "$H/Library/Caches/com.spotify.client/Storage/audio.bin"
f 160M  "$H/Library/Logs/zoom.us/zoom.log"
f 420M  "$H/Library/Logs/Homebrew/build.log"
f 310M  "$H/Library/Containers/com.apple.Notes/Data/Library/Caches/notes.bin"

# ── Core user cleaners / system ──
f 820M  "$H/Library/Caches/com.example.app/cache.bin"
f 95M   "$H/.Trash/old-installer.dmg"
f 16M   "$H/.zsh_history"
f 2M    "$H/.python_history"

# Temp files (the OS temp dir follows $TMPDIR on macOS)
f 3M "$H/tmp/build-cache.tmp"
f 1M "$H/tmp/session.lock"

# Preferences: a pleasant theme and a few cleaners pre-ticked (read by the GUI)
CFG="$H/Library/Application Support/cleansys"
mkdir -p "$CFG" "$H/.config/cleansys"
SETTINGS='{"theme_name":"Catppuccin Mocha","selected_cleaners":["User Land Cleaners: Application Caches","User Land Cleaners: Browser Caches","User Land Cleaners: Temporary Files","Developer Caches: Gradle Caches","Project Build Artifacts: Rust target/ Directories"]}'
echo "$SETTINGS" > "$CFG/settings.json"
echo "$SETTINGS" > "$H/.config/cleansys/settings.json"

export HOME="$DEMO_HOME"
export TMPDIR="$DEMO_HOME/tmp"
export CLEANSYS_SCAN_ROOTS="$P"
export PS1='$ '
