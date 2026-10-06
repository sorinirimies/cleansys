# Builds a throw-away HOME full of synthetic caches and projects so demos and
# screenshots never show real paths. Files are *sparse* (they report a size but
# use no disk), so this is instant and tiny. Never touches your real home.
#
#   source-env demo/fixture.nu       # sets HOME, TMPDIR, CLEANSYS_SCAN_ROOTS, prompt
#   DEMO_HOME=/some/dir nu -c 'source-env demo/fixture.nu; ...'
#
# Requires: nushell, and a `truncate` binary (macOS/Linux).

let demo_home = ($env.DEMO_HOME? | default "/tmp/cleansys-demo-home")
if ($demo_home | path exists) { rm -rf $demo_home }
mkdir $demo_home

let p = ($demo_home | path join "Projects")
let cfg = ($demo_home | path join "Library" "Application Support" "cleansys")

# ── sparse files: [size, path relative to the demo home] ──
let sparse = [
    ["1400M", "Projects/rust-api/target/debug/deps/libbig.rlib"]
    ["700M", "Projects/rust-api/target/debug/incremental/state.bin"]
    ["380M", "Projects/rust-api/target/release/rust-api"]
    ["640M", "Projects/android-shop/app/build/outputs/apk/debug/app-debug.apk"]
    ["210M", "Projects/android-shop/app/build/intermediates/dex.bin"]
    ["900M", "Projects/android-shop/.gradle/8.9/executionHistory.bin"]
    ["120M", "Projects/android-shop/build/reports.bin"]
    ["1100M", "Projects/web-dashboard/node_modules/.cache/big.bin"]
    ["420M", "Projects/web-dashboard/node_modules/typescript/lib.bin"]
    ["310M", "Projects/web-dashboard/.next/cache/webpack.bin"]
    ["2400M", "Projects/ml-notebooks/.venv/lib/torch.bin"]
    ["12M", "Projects/ml-notebooks/__pycache__/m.pyc"]
    ["30M", "Projects/ml-notebooks/.pytest_cache/v/cache.bin"]
    ["760M", "Projects/ios-weather/.build/debug/weather.bin"]
    ["3400M", ".gradle/caches/modules-2/files.bin"]
    ["210M", ".gradle/daemon/8.9/daemon.log"]
    ["780M", ".gradle/wrapper/dists/gradle-8.9-bin/dist.zip"]
    ["1800M", ".cargo/registry/src/index.crates.io/crates.bin"]
    ["420M", ".cargo/git/checkouts/repo/checkout.bin"]
    ["700M", ".npm/_cacache/content.bin"]
    ["520M", ".m2/repository/org/lib.jar"]
    ["340M", ".bun/install/cache/pkgs.bin"]
    ["400M", "Library/Caches/uv/archive.bin"]
    ["90M", ".android/cache/sdk.bin"]
    ["260M", "Library/Caches/JetBrains/AndroidStudio2024.2/caches.bin"]
    ["180M", "Library/Caches/ms-playwright/chromium/browser.bin"]
    ["6400M", ".cache/huggingface/hub/models--llama/blobs/weights.bin"]
    ["9200M", ".ollama/models/blobs/sha256-model"]
    ["3100M", ".cache/torch/hub/checkpoints/resnet.pth"]
    ["22M", ".claude/shell-snapshots/snapshot.sh"]
    ["310M", ".claude/projects/-Users-demo-Projects/session.jsonl"]
    ["140M", ".codex/sessions/2025/rollout.jsonl"]
    ["18M", ".pi/agent/npm/node_modules/pkg.bin"]
    ["64M", "Projects/rust-api/.codegraph/graph.db"]
    ["48M", "Projects/web-dashboard/.codegraph/graph.db"]
    ["1100M", "Library/Caches/Google/Chrome/Default/Cache/data_0"]
    ["240M", "Library/Caches/Google/Chrome/Default/Code Cache/js.bin"]
    ["640M", "Library/Caches/BraveSoftware/Brave-Browser/Default/Cache/data_0"]
    ["330M", "Library/Caches/Firefox/Profiles/abc.default/cache2/entries.bin"]
    ["38M", "Library/Application Support/Google/Chrome/Default/Cookies"]
    ["22M", "Library/Application Support/Google/Chrome/Default/History"]
    ["1500M", "Library/Application Support/Slack/Cache/Cache_Data/data_0"]
    ["220M", "Library/Application Support/Slack/Code Cache/js.bin"]
    ["540M", "Library/Application Support/discord/Cache/Cache_Data/data_0"]
    ["900M", "Library/Caches/com.spotify.client/Storage/audio.bin"]
    ["160M", "Library/Logs/zoom.us/zoom.log"]
    ["420M", "Library/Logs/Homebrew/build.log"]
    ["310M", "Library/Containers/com.apple.Notes/Data/Library/Caches/notes.bin"]
    ["820M", "Library/Caches/com.example.app/cache.bin"]
    ["95M", ".Trash/old-installer.dmg"]
    ["16M", ".zsh_history"]
    ["2M", ".python_history"]
    ["3M", "tmp/build-cache.tmp"]
    ["1M", "tmp/session.lock"]
]
for entry in $sparse {
    let dest = ($demo_home | path join $entry.1)
    mkdir ($dest | path dirname)
    ^truncate -s $entry.0 $dest
}

# ── small real files: project markers (build output is only cleaned next to one) ──
let markers = [
    ["rust-api/Cargo.toml", "[package]\nname = \"rust-api\"\n"]
    ["android-shop/settings.gradle.kts", "rootProject.name = \"shop\"\n"]
    ["android-shop/build.gradle.kts", "plugins {}\n"]
    ["android-shop/app/build.gradle.kts", "plugins {}\n"]
    ["web-dashboard/package.json", "{\"name\":\"web-dashboard\"}\n"]
    ["ml-notebooks/pyproject.toml", "[project]\nname = \"ml-notebooks\"\n"]
    ["ios-weather/Package.swift", "// swift-tools-version:5.9\n"]
]
for m in $markers {
    let dest = ($p | path join $m.0)
    mkdir ($dest | path dirname)
    $m.1 | save -f $dest
}

# Make every project look untouched for months (default: only idle projects are cleaned).
^find $p -exec touch -t 202501150900 {} +

# ── preferences read by the GUI/web: a pleasant theme + a few cleaners pre-ticked ──
let settings = {
    theme_name: "Catppuccin Mocha"
    selected_cleaners: [
        "User Land Cleaners: Application Caches"
        "User Land Cleaners: Browser Caches"
        "User Land Cleaners: Temporary Files"
        "Developer Caches: Gradle Caches"
        "Project Build Artifacts: Rust target/ Directories"
    ]
} | to json
mkdir $cfg
$settings | save -f ($cfg | path join "settings.json")
mkdir ($demo_home | path join ".config" "cleansys")
$settings | save -f ($demo_home | path join ".config" "cleansys" "settings.json")

# ── environment for everything started from this shell ──
$env.HOME = $demo_home
$env.USERPROFILE = $demo_home
$env.TMPDIR = ($demo_home | path join "tmp")
$env.CLEANSYS_SCAN_ROOTS = $p
$env.PROMPT_COMMAND = {|| "" }
$env.PROMPT_COMMAND_RIGHT = {|| "" }
$env.PROMPT_INDICATOR = "$ "
