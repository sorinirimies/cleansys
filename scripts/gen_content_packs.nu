#!/usr/bin/env nu
# Generate the repetitive built-in cleaner packs (browsers, apps, games).
#
#   nu scripts/gen_content_packs.nu
#
# Writes crates/cleansys-core/src/engine/builtin/{browsers,apps,games}.toml.
# Hand-written packs (system, privacy, containers, ...) are edited directly.

const OUT = (path self | path dirname | path dirname | path join "crates" "cleansys-core" "src" "engine" "builtin")
const CACHE = "$XDG_CACHE_HOME"
const CONF = "$XDG_CONFIG_HOME"
const DATA = "$XDG_DATA_HOME"
const MAC = "~/Library/Application Support"
const LAD = "$LOCALAPPDATA"
const AD = "$APPDATA"

# TOML basic string / array helpers
def q [s: string] { $s | to json --raw }
def arr [items: list<string>] { "[" + ($items | each { |i| q $i } | str join ", ") + "]" }

# One cleaner = header + one `delete` action per (label, per-os record).
def cleaner [id: string, name: string, desc: string, cat: string, risk: string, os: list<string>, actions: list] {
    mut out = [
        "[[cleaner]]"
        $"id = (q $id)"
        $"name = (q $name)"
        $"description = (q $desc)"
        $"category = (q $cat)"
    ]
    if $risk != "safe" { $out = ($out | append $"risk = \"($risk)\"") }
    if ($os | is-not-empty) { $out = ($out | append $"os = (arr $os)") }
    for a in $actions {
        $out = ($out | append ["[[cleaner.action]]" 'type = "delete"'])
        for k in [paths linux macos windows] {
            let v = ($a.paths | get -o $k | default [])
            if ($v | is-not-empty) { $out = ($out | append $"($k) = (arr $v)") }
        }
        $out = ($out | append $"label = (q $a.label)")
    }
    ($out | str join (char nl)) + (char nl)
}

# base/profile/sub path combinations (null profile/sub entries are skipped)
def join [bases: list<string>, subs: list, profiles?: list] {
    let profs = if ($profiles | is-empty) { [null] } else { $profiles }
    $bases | each { |b|
        $profs | each { |pr|
            $subs | each { |s|
                [$b $pr $s] | where { |x| $x != null } | str join "/"
            }
        } | flatten
    } | flatten
}

def per [linux: list, macos: list, windows: list] {
    {linux: $linux, macos: $macos, windows: $windows}
    | transpose k v | where { |r| ($r.v | is-not-empty) } | reduce -f {} { |r, acc| $acc | insert $r.k $r.v }
}

# ───────────────────────────── browsers ─────────────────────────────

const PROFILES = ["Default" "Profile *"]
const CACHE_ITEMS = ["Cache" "Code Cache" "GPUCache" "DawnCache" "Service Worker/CacheStorage" "Service Worker/ScriptCache" "Application Cache" "Media Cache"]
const ROOT_CACHE_ITEMS = ["ShaderCache" "GrShaderCache" "GraphiteDawnCache"]
const COOKIE_ITEMS = ["Cookies" "Cookies-journal" "Network/Cookies" "Network/Cookies-journal" "Local Storage" "IndexedDB" "Session Storage"]
const HISTORY_ITEMS = ["History" "History-journal" "Visited Links" "Top Sites" "Top Sites-journal" "Shortcuts" "Shortcuts-journal" "Favicons" "Favicons-journal" "Network Action Predictor"]
const SESSION_ITEMS = ["Current Session" "Current Tabs" "Last Session" "Last Tabs" "Sessions"]

# name, linux cfg, mac cfg, win cfg, linux cache, mac cache, has_profiles (null = not installed there)
def chromium_table [] {
    [
        ["Google Chrome" "google-chrome" "Google/Chrome" "Google/Chrome/User Data" "google-chrome" "Google/Chrome" true]
        ["Chromium" "chromium" "Chromium" "Chromium/User Data" "chromium" "Chromium" true]
        ["Brave" "BraveSoftware/Brave-Browser" "BraveSoftware/Brave-Browser" "BraveSoftware/Brave-Browser/User Data" "BraveSoftware/Brave-Browser" "BraveSoftware/Brave-Browser" true]
        ["Microsoft Edge" "microsoft-edge" "Microsoft Edge" "Microsoft/Edge/User Data" "microsoft-edge" "Microsoft Edge" true]
        ["Vivaldi" "vivaldi" "Vivaldi" "Vivaldi/User Data" "vivaldi" "Vivaldi" true]
        ["Opera" "opera" "com.operasoftware.Opera" "Opera Software/Opera Stable" "opera" "com.operasoftware.Opera" false]
        ["Arc" null "Arc/User Data" null null "Arc" true]
    ]
}

def chromium_per_os [name: string, row: list, items: list, include_cache_dirs: bool, root_items: list] {
    let lcfg = $row.1
    let mcfg = $row.2
    let wcfg = $row.3
    let lcache = $row.4
    let mcache = $row.5
    let profiles = $row.6
    let profs = if $profiles { $PROFILES } else { [] }
    let win_base = if ($name | str contains "Opera") { $AD } else { $LAD }

    mut linux = []
    mut macos = []
    mut windows = []
    if $lcfg != null { $linux = (join [$"($CONF)/($lcfg)"] $items $profs) }
    if $mcfg != null { $macos = (join [$"($MAC)/($mcfg)"] $items $profs) }
    if $wcfg != null { $windows = (join [$"($win_base)/($wcfg)"] $items $profs) }
    if ($root_items | is-not-empty) {
        if $lcfg != null { $linux = ($linux | append (join [$"($CONF)/($lcfg)"] $root_items)) }
        if $mcfg != null { $macos = ($macos | append (join [$"($MAC)/($mcfg)"] $root_items)) }
        if $wcfg != null { $windows = ($windows | append (join [$"($win_base)/($wcfg)"] $root_items)) }
    }
    if $include_cache_dirs {
        if $lcache != null {
            $linux = ($linux | append (if $profiles { join [$"($CACHE)/($lcache)"] [null] $PROFILES } else { [$"($CACHE)/($lcache)"] }))
        }
        if $mcache != null {
            $macos = ($macos | append (if $profiles { join [$"~/Library/Caches/($mcache)"] [null] $PROFILES } else { [$"~/Library/Caches/($mcache)"] }))
        }
    }
    per $linux $macos $windows
}

def slug [s: string] { $s | str downcase | str replace --all " " "-" }

def chromium_pack [] {
    mut out = []
    for row in (chromium_table) {
        let name = $row.0
        let sl = (slug $name)
        let os = ([[linux $row.1] [macos $row.2] [windows $row.3]] | where { |p| $p.1 != null } | each { |p| $p.0 })
        $out = ($out | append (cleaner $"br-($sl)-cache" $"($name) Cache" $"Page, code, GPU and service-worker caches of ($name) \(all profiles\)" "Web Browsers" "safe" $os [{label: $"($name) cache", paths: (chromium_per_os $name $row $CACHE_ITEMS true $ROOT_CACHE_ITEMS)}]))
        $out = ($out | append (cleaner $"br-($sl)-cookies" $"($name) Cookies & Site Data" $"Cookies, local storage and IndexedDB of ($name) \(signs you out of sites\)" "Web Browsers" "caution" $os [{label: $"($name) site data", paths: (chromium_per_os $name $row $COOKIE_ITEMS false [])}]))
        $out = ($out | append (cleaner $"br-($sl)-history" $"($name) History" $"Browsing history, top sites, favicons and URL predictions of ($name) \(bookmarks are kept\)" "Web Browsers" "moderate" $os [{label: $"($name) history", paths: (chromium_per_os $name $row $HISTORY_ITEMS false [])}]))
        $out = ($out | append (cleaner $"br-($sl)-session" $"($name) Sessions" $"Open/last tab session files of ($name)" "Web Browsers" "moderate" $os [{label: $"($name) session", paths: (chromium_per_os $name $row $SESSION_ITEMS false [])}]))
    }

    # Firefox family: profile + cache bases per OS
    let ff = [
        {name: "Firefox"
         prof: {linux: ["~/.mozilla/firefox/*" "~/snap/firefox/common/.mozilla/firefox/*" "~/.var/app/org.mozilla.firefox/.mozilla/firefox/*"], macos: [$"($MAC)/Firefox/Profiles/*"], windows: [$"($AD)/Mozilla/Firefox/Profiles/*"]}
         cache: {linux: [$"($CACHE)/mozilla/firefox/*" "~/snap/firefox/common/.cache/mozilla/firefox/*" "~/.var/app/org.mozilla.firefox/cache/mozilla/firefox/*"], macos: ["~/Library/Caches/Firefox/Profiles/*"], windows: [$"($LAD)/Mozilla/Firefox/Profiles/*"]}}
        {name: "LibreWolf"
         prof: {linux: ["~/.librewolf/*"], macos: [$"($MAC)/LibreWolf/Profiles/*"], windows: [$"($AD)/librewolf/Profiles/*"]}
         cache: {linux: [$"($CACHE)/librewolf/*"], macos: ["~/Library/Caches/LibreWolf/Profiles/*"], windows: [$"($LAD)/librewolf/Profiles/*"]}}
        {name: "Zen Browser"
         prof: {linux: ["~/.zen/*"], macos: [$"($MAC)/zen/Profiles/*"], windows: [$"($AD)/zen/Profiles/*"]}
         cache: {linux: [$"($CACHE)/zen/*"], macos: ["~/Library/Caches/zen/Profiles/*"], windows: [$"($LAD)/zen/Profiles/*"]}}
    ]
    for b in $ff {
        let name = $b.name
        let sl = (slug $name)
        let sub = { |base: record, items: list|
            per ($base.linux | each { |x| $items | each { |i| $"($x)/($i)" } } | flatten) ($base.macos | each { |x| $items | each { |i| $"($x)/($i)" } } | flatten) ($base.windows | each { |x| $items | each { |i| $"($x)/($i)" } } | flatten)
        }
        $out = ($out | append (cleaner $"br-($sl)-cache" $"($name) Cache" $"Disk, startup and thumbnail caches of ($name) \(all profiles\)" "Web Browsers" "safe" [] [{label: $"($name) cache", paths: (do $sub $b.cache ["cache2" "startupCache" "thumbnails" "OfflineCache" "jumpListCache"])}]))
        $out = ($out | append (cleaner $"br-($sl)-cookies" $"($name) Cookies" $"Cookies of ($name) \(signs you out of sites\)" "Web Browsers" "caution" [] [{label: $"($name) cookies", paths: (do $sub $b.prof ["cookies.sqlite" "cookies.sqlite-wal" "cookies.sqlite-shm" "webappsstore.sqlite"])}]))
        $out = ($out | append (cleaner $"br-($sl)-forms-sessions" $"($name) Form History & Sessions" $"Saved form entries and session-restore files of ($name)" "Web Browsers" "moderate" [] [{label: $"($name) forms/sessions", paths: (do $sub $b.prof ["formhistory.sqlite" "formhistory.sqlite-wal" "sessionstore.jsonlz4" "sessionstore-backups/*" "sessionCheckpoints.json"])}]))
        $out = ($out | append (cleaner $"br-($sl)-crash-telemetry" $"($name) Crash Reports & Telemetry" $"Crash dumps and queued telemetry pings of ($name)" "Web Browsers" "safe" [] [{label: $"($name) crashes", paths: (do $sub $b.prof ["crashes/*" "minidumps/*" "saved-telemetry-pings/*" "datareporting/archived/*"])}]))
    }

    let safari_cache = {macos: ["~/Library/Caches/com.apple.Safari" "~/Library/Caches/com.apple.Safari.SafeBrowsing" "~/Library/Containers/com.apple.Safari/Data/Library/Caches/*" "~/Library/Caches/com.apple.WebKit.Networking" "~/Library/Caches/com.apple.WebKit.WebContent"]}
    let safari_cookies = {macos: ["~/Library/Cookies/Cookies.binarycookies" "~/Library/Containers/com.apple.Safari/Data/Library/Cookies/Cookies.binarycookies" "~/Library/Safari/LocalStorage/*" "~/Library/Safari/Databases/*"]}
    let safari_history = {macos: ["~/Library/Safari/History.db" "~/Library/Safari/History.db-lock" "~/Library/Safari/History.db-shm" "~/Library/Safari/History.db-wal" "~/Library/Safari/LastSession.plist" "~/Library/Safari/RecentlyClosedTabs.plist" "~/Library/Safari/TopSites.plist" "~/Library/Safari/Downloads.plist"]}
    $out = ($out | append (cleaner "br-safari-cache" "Safari Cache" "Safari and WebKit caches" "Web Browsers" "safe" ["macos"] [{label: "Safari cache", paths: $safari_cache}]))
    $out = ($out | append (cleaner "br-safari-cookies" "Safari Cookies & Site Data" "Safari cookies, local storage and databases \(signs you out of sites\)" "Web Browsers" "caution" ["macos"] [{label: "Safari site data", paths: $safari_cookies}]))
    $out = ($out | append (cleaner "br-safari-history" "Safari History & Sessions" "Safari history, last session, recently closed tabs and downloads list \(needs Full Disk Access\)" "Web Browsers" "moderate" ["macos"] [{label: "Safari history", paths: $safari_history}]))
    $out
}

# ───────────────────────────── apps ─────────────────────────────

const ELECTRON_ITEMS = ["Cache" "Code Cache" "GPUCache" "DawnCache" "Service Worker/CacheStorage" "Service Worker/ScriptCache" "logs" "Crashpad/completed" "blob_storage"]

# name, linux dir, mac dir, windows dir, extra flatpak config dirs
def electron_table [] {
    [
        ["Slack" "Slack" "Slack" "Slack" ["~/.var/app/com.slack.Slack/config/Slack"]]
        ["Discord" "discord" "discord" "discord" ["~/.var/app/com.discordapp.Discord/config/discord"]]
        ["Microsoft Teams" "Microsoft/Microsoft Teams" "Microsoft/Teams" "Microsoft/Teams" []]
        ["Signal" "Signal" "Signal" "Signal" ["~/.var/app/org.signal.Signal/config/Signal"]]
        ["Notion" "Notion" "Notion" "Notion" []]
        ["Obsidian" "obsidian" "obsidian" "obsidian" ["~/.var/app/md.obsidian.Obsidian/config/obsidian"]]
        ["Postman" "Postman" "Postman" "Postman" []]
        ["Insomnia" "Insomnia" "Insomnia" "Insomnia" []]
        ["Figma" "Figma" "Figma" "Figma" []]
        ["Linear" "Linear" "Linear" "Linear" []]
        ["Element" "Element" "Element" "Element" []]
        ["WhatsApp" "WhatsApp" "WhatsApp" "WhatsApp" []]
        ["1Password" "1Password" "1Password" "1Password" []]
        ["GitHub Desktop" "GitHub Desktop" "GitHub Desktop" "GitHub Desktop" []]
    ]
}

def apps_pack [] {
    mut out = []
    for e in (electron_table) {
        let name = $e.0
        let sl = (slug $name)
        let linux = (($ELECTRON_ITEMS | each { |i| $"($CONF)/($e.1)/($i)" }) | append ($e.4 | each { |x| $ELECTRON_ITEMS | each { |i| $"($x)/($i)" } } | flatten))
        let macos = ($ELECTRON_ITEMS | each { |i| $"($MAC)/($e.2)/($i)" })
        let windows = ($ELECTRON_ITEMS | each { |i| $"($AD)/($e.3)/($i)" })
        $out = ($out | append (cleaner $"app-($sl)" $"($name) Cache & Logs" $"Chromium/Electron caches and logs of ($name)" "Applications" "safe" [] [{label: $"($name) cache", paths: {linux: $linux, macos: $macos, windows: $windows}}]))
    }

    # id, name, description, paths record, risk, os
    let simple = [
        ["app-spotify" "Spotify Cache" "Offline-song cache and browse data of Spotify" {linux: [$"($CACHE)/spotify" "~/.var/app/com.spotify.Client/cache/spotify"], macos: ["~/Library/Caches/com.spotify.client" $"($MAC)/Spotify/PersistentCache"], windows: [$"($LAD)/Spotify/Storage" $"($LAD)/Spotify/Data"]} "safe" []]
        ["app-zoom" "Zoom Logs & Cache" "Logs and updater data of Zoom" {paths: ["~/.zoom/logs"], macos: ["~/Library/Logs/zoom.us" $"($MAC)/zoom.us/AutoUpdater"], windows: [$"($AD)/Zoom/logs"]} "safe" []]
        ["app-telegram" "Telegram Desktop Cache" "Downloaded media cache of Telegram Desktop" {linux: [$"($DATA)/TelegramDesktop/tdata/user_data/cache" $"($DATA)/TelegramDesktop/tdata/user_data/media_cache"], macos: [$"($MAC)/Telegram Desktop/tdata/user_data/cache" $"($MAC)/Telegram Desktop/tdata/user_data/media_cache"], windows: [$"($AD)/Telegram Desktop/tdata/user_data/cache" $"($AD)/Telegram Desktop/tdata/user_data/media_cache"]} "moderate" []]
        ["app-dropbox" "Dropbox Cache" "Dropbox deleted-file cache (~/Dropbox/.dropbox.cache)" {paths: ["~/Dropbox/.dropbox.cache/*"]} "safe" []]
        ["app-thunderbird" "Thunderbird Cache" "Message and attachment caches of Thunderbird" {linux: [$"($CACHE)/thunderbird/*/cache2" "~/.var/app/org.mozilla.Thunderbird/cache/thunderbird/*/cache2"], macos: ["~/Library/Caches/Thunderbird/Profiles/*/cache2"], windows: [$"($LAD)/Thunderbird/Profiles/*/cache2"]} "safe" []]
        ["app-vlc" "VLC Cache & Art" "Album-art and media-library caches of VLC" {linux: [$"($CACHE)/vlc"], macos: ["~/Library/Caches/org.videolan.vlc"], windows: [$"($AD)/vlc/art"]} "safe" []]
        ["app-libreoffice" "LibreOffice Backups & Temp" "Autosave backups and temp files of LibreOffice" {linux: [$"($CONF)/libreoffice/4/user/backup/*" $"($CONF)/libreoffice/4/user/tmp/*"], macos: [$"($MAC)/LibreOffice/4/user/backup/*" $"($MAC)/LibreOffice/4/user/tmp/*"], windows: [$"($AD)/LibreOffice/4/user/backup/*" $"($AD)/LibreOffice/4/user/tmp/*"]} "moderate" []]
        ["app-creative-caches" "GIMP / Inkscape / Blender Caches" "Thumbnail and render caches of open-source creative apps" {linux: [$"($CACHE)/gimp" $"($CACHE)/inkscape" $"($CACHE)/blender" $"($CACHE)/krita"], macos: ["~/Library/Caches/org.gimp.gimp-*" "~/Library/Caches/org.inkscape.Inkscape" "~/Library/Caches/Blender Foundation" "~/Library/Caches/org.krita.*"], windows: [$"($LAD)/Temp/Blender" $"($AD)/GIMP/*/tmp" $"($AD)/GIMP/*/cache"]} "safe" []]
        ["app-adobe" "Adobe Media & Application Caches" "Premiere/After Effects media caches and Adobe app caches" {macos: ["~/Library/Caches/Adobe" $"($MAC)/Adobe/Common/Media Cache Files" $"($MAC)/Adobe/Common/Media Cache" $"($MAC)/Adobe/Common/Peak Files"], windows: [$"($AD)/Adobe/Common/Media Cache Files" $"($AD)/Adobe/Common/Media Cache" $"($AD)/Adobe/Common/Peak Files" $"($LAD)/Adobe/*/Cache"]} "moderate" []]
        ["app-office-caches" "Microsoft Office Caches" "Office file cache and web-view caches" {macos: ["~/Library/Containers/com.microsoft.Word/Data/Library/Caches/*" "~/Library/Containers/com.microsoft.Excel/Data/Library/Caches/*" "~/Library/Containers/com.microsoft.Powerpoint/Data/Library/Caches/*" "~/Library/Containers/com.microsoft.Outlook/Data/Library/Caches/*"], windows: [$"($LAD)/Microsoft/Office/16.0/OfficeFileCache/*" $"($LAD)/Microsoft/Office/16.0/WebServiceCache/*"]} "safe" []]
        ["app-jetbrains-logs" "JetBrains IDE Logs" "Log directories of JetBrains IDEs and Android Studio" {linux: [$"($CACHE)/JetBrains/*/log" $"($CACHE)/Google/AndroidStudio*/log"], macos: ["~/Library/Logs/JetBrains/*" "~/Library/Logs/Google/AndroidStudio*"], windows: [$"($LAD)/JetBrains/*/log" $"($LAD)/Google/AndroidStudio*/log"]} "safe" []]
        ["app-flatpak-app-caches" "Flatpak Application Caches" "Per-app cache directories of Flatpak apps (~/.var/app/*/cache)" {linux: ["~/.var/app/*/cache/*"]} "moderate" ["linux"]]
        ["app-snap-app-caches" "Snap Application Caches" "Per-app caches of Snap packages (~/snap/*/*/.cache)" {linux: ["~/snap/*/*/.cache/*"]} "moderate" ["linux"]]
        ["app-macos-sandbox-caches" "Sandboxed App Caches" "Cache folders inside ~/Library/Containers (Mail, Notes, Maps, third-party App Store apps)" {macos: ["~/Library/Containers/*/Data/Library/Caches/*"]} "moderate" ["macos"]]
        ["app-macos-app-logs" "App Logs" "Log files in ~/Library/Logs" {macos: ["~/Library/Logs/*"]} "safe" ["macos"]]
        ["app-macos-saved-state" "Saved Application State" "Window-restore state of apps (~/Library/Saved Application State)" {macos: ["~/Library/Saved Application State/*"]} "safe" ["macos"]]
    ]
    for s in $simple {
        $out = ($out | append (cleaner $s.0 $s.1 $s.2 "Applications" $s.4 $s.5 [{label: $s.1, paths: $s.3}]))
    }
    $out
}

# ───────────────────────────── games ─────────────────────────────

def games_pack [] {
    let steam_win = "C:/Program Files (x86)/Steam"
    let games = [
        ["game-steam-shader" "Steam Shader Cache" "Compiled shader caches in steamapps/shadercache (rebuilt on next launch)" {linux: ["~/.steam/steam/steamapps/shadercache/*" $"($DATA)/Steam/steamapps/shadercache/*" "~/.var/app/com.valvesoftware.Steam/.local/share/Steam/steamapps/shadercache/*"], macos: [$"($MAC)/Steam/steamapps/shadercache/*"], windows: [$"($steam_win)/steamapps/shadercache/*"]} "moderate" []]
        ["game-steam-web-logs" "Steam Web Cache & Logs" "Steam browser/HTTP caches and logs" {linux: ["~/.steam/steam/appcache/httpcache" "~/.steam/steam/config/htmlcache" "~/.steam/steam/logs/*" $"($DATA)/Steam/appcache/httpcache" $"($DATA)/Steam/config/htmlcache" $"($DATA)/Steam/logs/*"], macos: [$"($MAC)/Steam/appcache/httpcache" $"($MAC)/Steam/config/htmlcache" $"($MAC)/Steam/logs/*"], windows: [$"($steam_win)/appcache/httpcache" $"($steam_win)/config/htmlcache" $"($steam_win)/logs/*" $"($LAD)/Steam/htmlcache"]} "safe" []]
        ["game-epic" "Epic Games Launcher Cache" "Web cache and logs of the Epic Games Launcher" {macos: ["~/Library/Caches/com.epicgames.EpicGamesLauncher" $"($MAC)/Epic/EpicGamesLauncher/Saved/webcache*" $"($MAC)/Epic/EpicGamesLauncher/Saved/Logs/*"], windows: [$"($LAD)/EpicGamesLauncher/Saved/webcache*" $"($LAD)/EpicGamesLauncher/Saved/Logs/*"]} "safe" []]
        ["game-heroic-lutris" "Heroic / Lutris Caches" "Launcher caches of Heroic Games Launcher and Lutris" {linux: [$"($CONF)/heroic/Cache" $"($CONF)/heroic/Code Cache" $"($CONF)/heroic/GPUCache" $"($CACHE)/lutris"]} "safe" ["linux"]]
        ["game-wine" "Wine / Winetricks Caches" "Downloaded installers and caches of Wine and winetricks" {linux: [$"($CACHE)/wine" $"($CACHE)/winetricks"], macos: ["~/Library/Caches/winetricks" "~/Library/Caches/Wine"]} "moderate" ["linux" "macos"]]
        ["game-minecraft" "Minecraft Logs & Crash Reports" "Launcher/game logs and crash reports" {paths: ["~/.minecraft/logs/*" "~/.minecraft/crash-reports/*"], macos: [$"($MAC)/minecraft/logs/*" $"($MAC)/minecraft/crash-reports/*"], windows: [$"($AD)/.minecraft/logs/*" $"($AD)/.minecraft/crash-reports/*"]} "safe" []]
        ["game-gpu-shaders" "GPU Shader Caches" "Mesa, NVIDIA, AMD and DirectX shader caches (rebuilt on demand)" {linux: [$"($CACHE)/mesa_shader_cache" $"($CACHE)/mesa_shader_cache_db" $"($CACHE)/radv_builtin_shaders*" $"($CACHE)/nvidia/GLCache" $"($CACHE)/nvidia/DXCache" "~/.nv/GLCache" "~/.nv/ComputeCache" $"($CACHE)/AMD/GLCache"], windows: [$"($LAD)/D3DSCache/*" $"($LAD)/NVIDIA/DXCache/*" $"($LAD)/NVIDIA/GLCache/*" $"($LAD)/AMD/DxCache/*" $"($LAD)/AMD/GLCache/*"]} "moderate" ["linux" "windows"]]
    ]
    $games | each { |g| cleaner $g.0 $g.1 $g.2 "Games" $g.4 $g.5 [{label: $g.1, paths: $g.3}] }
}

def write_pack [file: string, items: list] {
    let header = "# GENERATED by scripts/gen_content_packs.nu — edit the script, not this file.\n\n"
    ($header + ($items | str join (char nl))) | save -f ($OUT | path join $file)
    print $"($file) ($items | length) cleaners"
}

def main [] {
    write_pack "browsers.toml" (chromium_pack)
    write_pack "apps.toml" (apps_pack)
    write_pack "games.toml" (games_pack)
}
