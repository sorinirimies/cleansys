#!/usr/bin/env python3
"""Generate the repetitive built-in cleaner packs (browsers, apps, games).

Usage: python3 scripts/gen_content_packs.py
Writes crates/cleansys-core/src/engine/builtin/{browsers,apps,games}.toml.
Hand-written packs (system, privacy, containers, ...) are edited directly.
"""
import json, pathlib

OUT = pathlib.Path(__file__).resolve().parent.parent / "crates/cleansys-core/src/engine/builtin"
CACHE, CONF, DATA = "$XDG_CACHE_HOME", "$XDG_CONFIG_HOME", "$XDG_DATA_HOME"
MAC, LAD, AD = "~/Library/Application Support", "$LOCALAPPDATA", "$APPDATA"


def q(s):
    return json.dumps(s)


def arr(items):
    return "[" + ", ".join(q(i) for i in items) + "]"


def cleaner(id_, name, desc, cat, risk, os_, actions, root=False):
    out = ["[[cleaner]]", f"id = {q(id_)}", f"name = {q(name)}", f"description = {q(desc)}", f"category = {q(cat)}"]
    if risk != "safe":
        out.append(f'risk = "{risk}"')
    if os_:
        out.append(f"os = {arr(os_)}")
    if root:
        out.append("requires_root = true")
    for label, per_os in actions:
        out.append("[[cleaner.action]]")
        out.append('type = "delete"')
        for k in ("paths", "linux", "macos", "windows"):
            if per_os.get(k):
                out.append(f"{k} = {arr(per_os[k])}")
        out.append(f"label = {q(label)}")
    return "\n".join(out) + "\n"


def join(bases, subs, profiles=None):
    res = []
    for b in bases:
        for pr in (profiles or [None]):
            for s in subs:
                parts = [b] + ([pr] if pr else []) + ([s] if s else [])
                res.append("/".join(parts))
    return res


# ---------------------------------------------------------------- browsers
CHROMIUM = {
    # name: (linux cfg, mac cfg, win cfg, linux cache, mac cache, has_profiles)
    "Google Chrome": ("google-chrome", "Google/Chrome", "Google/Chrome/User Data", "google-chrome", "Google/Chrome", True),
    "Chromium": ("chromium", "Chromium", "Chromium/User Data", "chromium", "Chromium", True),
    "Brave": ("BraveSoftware/Brave-Browser", "BraveSoftware/Brave-Browser", "BraveSoftware/Brave-Browser/User Data", "BraveSoftware/Brave-Browser", "BraveSoftware/Brave-Browser", True),
    "Microsoft Edge": ("microsoft-edge", "Microsoft Edge", "Microsoft/Edge/User Data", "microsoft-edge", "Microsoft Edge", True),
    "Vivaldi": ("vivaldi", "Vivaldi", "Vivaldi/User Data", "vivaldi", "Vivaldi", True),
    "Opera": ("opera", "com.operasoftware.Opera", "Opera Software/Opera Stable", "opera", "com.operasoftware.Opera", False),
    "Arc": (None, "Arc/User Data", None, None, "Arc", True),
}
PROFILES = ["Default", "Profile *"]
CACHE_ITEMS = ["Cache", "Code Cache", "GPUCache", "DawnCache", "Service Worker/CacheStorage", "Service Worker/ScriptCache", "Application Cache", "Media Cache"]
ROOT_CACHE_ITEMS = ["ShaderCache", "GrShaderCache", "GraphiteDawnCache"]
COOKIE_ITEMS = ["Cookies", "Cookies-journal", "Network/Cookies", "Network/Cookies-journal", "Local Storage", "IndexedDB", "Session Storage"]
HISTORY_ITEMS = ["History", "History-journal", "Visited Links", "Top Sites", "Top Sites-journal", "Shortcuts", "Shortcuts-journal", "Favicons", "Favicons-journal", "Network Action Predictor"]
SESSION_ITEMS = ["Current Session", "Current Tabs", "Last Session", "Last Tabs", "Sessions"]


def chromium_pack():
    out = []
    for name, (lcfg, mcfg, wcfg, lcache, mcache, profiles) in CHROMIUM.items():
        slug = name.lower().replace(" ", "-")
        os_ = [o for o, v in (("linux", lcfg), ("macos", mcfg), ("windows", wcfg)) if v]
        profs = PROFILES if profiles else [None]

        def per_os(items, include_cache_dirs=False, root_items=None):
            d = {"linux": [], "macos": [], "windows": []}
            if lcfg:
                d["linux"] = join([f"{CONF}/{lcfg}"], items, profs)
            if mcfg:
                d["macos"] = join([f"{MAC}/{mcfg}"], items, profs)
            if wcfg:
                d["windows"] = join([f"{LAD if 'Opera' not in name else AD}/{wcfg}"], items, profs)
            if root_items:
                if lcfg:
                    d["linux"] += join([f"{CONF}/{lcfg}"], root_items)
                if mcfg:
                    d["macos"] += join([f"{MAC}/{mcfg}"], root_items)
                if wcfg:
                    d["windows"] += join([f"{LAD if 'Opera' not in name else AD}/{wcfg}"], root_items)
            if include_cache_dirs:
                if lcache:
                    d["linux"] += join([f"{CACHE}/{lcache}"], [None], profs if profiles else [None]) if profiles else [f"{CACHE}/{lcache}"]
                if mcache:
                    d["macos"] += join([f"~/Library/Caches/{mcache}"], [None], profs) if profiles else [f"~/Library/Caches/{mcache}"]
            return {k: v for k, v in d.items() if v}

        out.append(cleaner(f"br-{slug}-cache", f"{name} Cache", f"Page, code, GPU and service-worker caches of {name} (all profiles)", "Web Browsers", "safe", os_, [(f"{name} cache", per_os(CACHE_ITEMS, True, ROOT_CACHE_ITEMS))]))
        out.append(cleaner(f"br-{slug}-cookies", f"{name} Cookies & Site Data", f"Cookies, local storage and IndexedDB of {name} (signs you out of sites)", "Web Browsers", "caution", os_, [(f"{name} site data", per_os(COOKIE_ITEMS))]))
        out.append(cleaner(f"br-{slug}-history", f"{name} History", f"Browsing history, top sites, favicons and URL predictions of {name} (bookmarks are kept)", "Web Browsers", "moderate", os_, [(f"{name} history", per_os(HISTORY_ITEMS))]))
        out.append(cleaner(f"br-{slug}-session", f"{name} Sessions", f"Open/last tab session files of {name}", "Web Browsers", "moderate", os_, [(f"{name} session", per_os(SESSION_ITEMS))]))

    # Firefox family
    ff = {
        "Firefox": dict(
            prof={"linux": ["~/.mozilla/firefox/*", "~/snap/firefox/common/.mozilla/firefox/*", "~/.var/app/org.mozilla.firefox/.mozilla/firefox/*"], "macos": [f"{MAC}/Firefox/Profiles/*"], "windows": [f"{AD}/Mozilla/Firefox/Profiles/*"]},
            cache={"linux": [f"{CACHE}/mozilla/firefox/*", "~/snap/firefox/common/.cache/mozilla/firefox/*", "~/.var/app/org.mozilla.firefox/cache/mozilla/firefox/*"], "macos": ["~/Library/Caches/Firefox/Profiles/*"], "windows": [f"{LAD}/Mozilla/Firefox/Profiles/*"]}),
        "LibreWolf": dict(
            prof={"linux": ["~/.librewolf/*"], "macos": [f"{MAC}/LibreWolf/Profiles/*"], "windows": [f"{AD}/librewolf/Profiles/*"]},
            cache={"linux": [f"{CACHE}/librewolf/*"], "macos": ["~/Library/Caches/LibreWolf/Profiles/*"], "windows": [f"{LAD}/librewolf/Profiles/*"]}),
        "Zen Browser": dict(
            prof={"linux": ["~/.zen/*"], "macos": [f"{MAC}/zen/Profiles/*"], "windows": [f"{AD}/zen/Profiles/*"]},
            cache={"linux": [f"{CACHE}/zen/*"], "macos": ["~/Library/Caches/zen/Profiles/*"], "windows": [f"{LAD}/zen/Profiles/*"]}),
    }
    for name, d in ff.items():
        slug = name.lower().replace(" ", "-")

        def sub(base, items):
            return {os_: [f"{b}/{i}" for b in bases for i in items] for os_, bases in base.items()}
        out.append(cleaner(f"br-{slug}-cache", f"{name} Cache", f"Disk, startup and thumbnail caches of {name} (all profiles)", "Web Browsers", "safe", [], [(f"{name} cache", sub(d["cache"], ["cache2", "startupCache", "thumbnails", "OfflineCache", "jumpListCache"]))]))
        out.append(cleaner(f"br-{slug}-cookies", f"{name} Cookies", f"Cookies of {name} (signs you out of sites)", "Web Browsers", "caution", [], [(f"{name} cookies", sub(d["prof"], ["cookies.sqlite", "cookies.sqlite-wal", "cookies.sqlite-shm", "webappsstore.sqlite"]))]))
        out.append(cleaner(f"br-{slug}-forms-sessions", f"{name} Form History & Sessions", f"Saved form entries and session-restore files of {name}", "Web Browsers", "moderate", [], [(f"{name} forms/sessions", sub(d["prof"], ["formhistory.sqlite", "formhistory.sqlite-wal", "sessionstore.jsonlz4", "sessionstore-backups/*", "sessionCheckpoints.json"]))]))
        out.append(cleaner(f"br-{slug}-crash-telemetry", f"{name} Crash Reports & Telemetry", f"Crash dumps and queued telemetry pings of {name}", "Web Browsers", "safe", [], [(f"{name} crashes", sub(d["prof"], ["crashes/*", "minidumps/*", "saved-telemetry-pings/*", "datareporting/archived/*"]))]))

    safari = {
        "cache": {"macos": ["~/Library/Caches/com.apple.Safari", "~/Library/Caches/com.apple.Safari.SafeBrowsing", "~/Library/Containers/com.apple.Safari/Data/Library/Caches/*", "~/Library/Caches/com.apple.WebKit.Networking", "~/Library/Caches/com.apple.WebKit.WebContent"]},
        "cookies": {"macos": ["~/Library/Cookies/Cookies.binarycookies", "~/Library/Containers/com.apple.Safari/Data/Library/Cookies/Cookies.binarycookies", "~/Library/Safari/LocalStorage/*", "~/Library/Safari/Databases/*"]},
        "history": {"macos": ["~/Library/Safari/History.db", "~/Library/Safari/History.db-lock", "~/Library/Safari/History.db-shm", "~/Library/Safari/History.db-wal", "~/Library/Safari/LastSession.plist", "~/Library/Safari/RecentlyClosedTabs.plist", "~/Library/Safari/TopSites.plist", "~/Library/Safari/Downloads.plist"]},
    }
    out.append(cleaner("br-safari-cache", "Safari Cache", "Safari and WebKit caches", "Web Browsers", "safe", ["macos"], [("Safari cache", safari["cache"])]))
    out.append(cleaner("br-safari-cookies", "Safari Cookies & Site Data", "Safari cookies, local storage and databases (signs you out of sites)", "Web Browsers", "caution", ["macos"], [("Safari site data", safari["cookies"])]))
    out.append(cleaner("br-safari-history", "Safari History & Sessions", "Safari history, last session, recently closed tabs and downloads list (needs Full Disk Access)", "Web Browsers", "moderate", ["macos"], [("Safari history", safari["history"])]))
    return out


# ---------------------------------------------------------------- apps
ELECTRON_ITEMS = ["Cache", "Code Cache", "GPUCache", "DawnCache", "Service Worker/CacheStorage", "Service Worker/ScriptCache", "logs", "Crashpad/completed", "blob_storage"]
ELECTRON = {
    "Slack": ("Slack", "Slack", "Slack", ["~/.var/app/com.slack.Slack/config/Slack"]),
    "Discord": ("discord", "discord", "discord", ["~/.var/app/com.discordapp.Discord/config/discord"]),
    "Microsoft Teams": ("Microsoft/Microsoft Teams", "Microsoft/Teams", "Microsoft/Teams", []),
    "Signal": ("Signal", "Signal", "Signal", ["~/.var/app/org.signal.Signal/config/Signal"]),
    "Notion": ("Notion", "Notion", "Notion", []),
    "Obsidian": ("obsidian", "obsidian", "obsidian", ["~/.var/app/md.obsidian.Obsidian/config/obsidian"]),
    "Postman": ("Postman", "Postman", "Postman", []),
    "Insomnia": ("Insomnia", "Insomnia", "Insomnia", []),
    "Figma": ("Figma", "Figma", "Figma", []),
    "Linear": ("Linear", "Linear", "Linear", []),
    "Element": ("Element", "Element", "Element", []),
    "WhatsApp": ("WhatsApp", "WhatsApp", "WhatsApp", []),
    "1Password": ("1Password", "1Password", "1Password", []),
    "GitHub Desktop": ("GitHub Desktop", "GitHub Desktop", "GitHub Desktop", []),
    "Zed": (None, None, None, []),
}


def apps_pack():
    out = []
    for name, (l, m, w, extra) in ELECTRON.items():
        if l is None:
            continue
        slug = name.lower().replace(" ", "-")
        d = {
            "linux": [f"{CONF}/{l}/{i}" for i in ELECTRON_ITEMS] + [f"{e}/{i}" for e in extra for i in ELECTRON_ITEMS],
            "macos": [f"{MAC}/{m}/{i}" for i in ELECTRON_ITEMS],
            "windows": [f"{AD}/{w}/{i}" for i in ELECTRON_ITEMS],
        }
        out.append(cleaner(f"app-{slug}", f"{name} Cache & Logs", f"Chromium/Electron caches and logs of {name}", "Applications", "safe", [], [(f"{name} cache", d)]))

    def simple(id_, name, desc, d, risk="safe", cat="Applications", os_=None):
        out.append(cleaner(id_, name, desc, cat, risk, os_ or [], [(name, d)]))

    simple("app-spotify", "Spotify Cache", "Offline-song cache and browse data of Spotify", {"linux": [f"{CACHE}/spotify", "~/.var/app/com.spotify.Client/cache/spotify"], "macos": ["~/Library/Caches/com.spotify.client", f"{MAC}/Spotify/PersistentCache"], "windows": [f"{LAD}/Spotify/Storage", f"{LAD}/Spotify/Data"]})
    simple("app-zoom", "Zoom Logs & Cache", "Logs and updater data of Zoom", {"paths": ["~/.zoom/logs"], "macos": ["~/Library/Logs/zoom.us", f"{MAC}/zoom.us/AutoUpdater"], "windows": [f"{AD}/Zoom/logs"]})
    simple("app-telegram", "Telegram Desktop Cache", "Downloaded media cache of Telegram Desktop", {"linux": [f"{DATA}/TelegramDesktop/tdata/user_data/cache", f"{DATA}/TelegramDesktop/tdata/user_data/media_cache"], "macos": [f"{MAC}/Telegram Desktop/tdata/user_data/cache", f"{MAC}/Telegram Desktop/tdata/user_data/media_cache"], "windows": [f"{AD}/Telegram Desktop/tdata/user_data/cache", f"{AD}/Telegram Desktop/tdata/user_data/media_cache"]}, "moderate")
    simple("app-dropbox", "Dropbox Cache", "Dropbox deleted-file cache (~/Dropbox/.dropbox.cache)", {"paths": ["~/Dropbox/.dropbox.cache/*"]})
    simple("app-thunderbird", "Thunderbird Cache", "Message and attachment caches of Thunderbird", {"linux": [f"{CACHE}/thunderbird/*/cache2", "~/.var/app/org.mozilla.Thunderbird/cache/thunderbird/*/cache2"], "macos": ["~/Library/Caches/Thunderbird/Profiles/*/cache2"], "windows": [f"{LAD}/Thunderbird/Profiles/*/cache2"]})
    simple("app-vlc", "VLC Cache & Art", "Album-art and media-library caches of VLC", {"linux": [f"{CACHE}/vlc"], "macos": ["~/Library/Caches/org.videolan.vlc"], "windows": [f"{AD}/vlc/art"]})
    simple("app-libreoffice", "LibreOffice Backups & Temp", "Autosave backups and temp files of LibreOffice", {"linux": [f"{CONF}/libreoffice/4/user/backup/*", f"{CONF}/libreoffice/4/user/tmp/*"], "macos": [f"{MAC}/LibreOffice/4/user/backup/*", f"{MAC}/LibreOffice/4/user/tmp/*"], "windows": [f"{AD}/LibreOffice/4/user/backup/*", f"{AD}/LibreOffice/4/user/tmp/*"]}, "moderate")
    simple("app-creative-caches", "GIMP / Inkscape / Blender Caches", "Thumbnail and render caches of open-source creative apps", {"linux": [f"{CACHE}/gimp", f"{CACHE}/inkscape", f"{CACHE}/blender", f"{CACHE}/krita"], "macos": ["~/Library/Caches/org.gimp.gimp-*", "~/Library/Caches/org.inkscape.Inkscape", "~/Library/Caches/Blender Foundation", "~/Library/Caches/org.krita.*"], "windows": [f"{LAD}/Temp/Blender", f"{AD}/GIMP/*/tmp", f"{AD}/GIMP/*/cache"]})
    simple("app-adobe", "Adobe Media & Application Caches", "Premiere/After Effects media caches and Adobe app caches", {"macos": ["~/Library/Caches/Adobe", f"{MAC}/Adobe/Common/Media Cache Files", f"{MAC}/Adobe/Common/Media Cache", f"{MAC}/Adobe/Common/Peak Files"], "windows": [f"{AD}/Adobe/Common/Media Cache Files", f"{AD}/Adobe/Common/Media Cache", f"{AD}/Adobe/Common/Peak Files", f"{LAD}/Adobe/*/Cache"]}, "moderate")
    simple("app-office-caches", "Microsoft Office Caches", "Office file cache and web-view caches", {"macos": ["~/Library/Containers/com.microsoft.Word/Data/Library/Caches/*", "~/Library/Containers/com.microsoft.Excel/Data/Library/Caches/*", "~/Library/Containers/com.microsoft.Powerpoint/Data/Library/Caches/*", "~/Library/Containers/com.microsoft.Outlook/Data/Library/Caches/*"], "windows": [f"{LAD}/Microsoft/Office/16.0/OfficeFileCache/*", f"{LAD}/Microsoft/Office/16.0/WebServiceCache/*"]})
    simple("app-jetbrains-logs", "JetBrains IDE Logs", "Log directories of JetBrains IDEs and Android Studio", {"linux": [f"{CACHE}/JetBrains/*/log", f"{CACHE}/Google/AndroidStudio*/log"], "macos": ["~/Library/Logs/JetBrains/*", "~/Library/Logs/Google/AndroidStudio*"], "windows": [f"{LAD}/JetBrains/*/log", f"{LAD}/Google/AndroidStudio*/log"]})
    simple("app-flatpak-app-caches", "Flatpak Application Caches", "Per-app cache directories of Flatpak apps (~/.var/app/*/cache)", {"linux": ["~/.var/app/*/cache/*"]}, "moderate", os_=["linux"])
    simple("app-snap-app-caches", "Snap Application Caches", "Per-app caches of Snap packages (~/snap/*/*/.cache)", {"linux": ["~/snap/*/*/.cache/*"]}, "moderate", os_=["linux"])
    simple("app-macos-sandbox-caches", "Sandboxed App Caches", "Cache folders inside ~/Library/Containers (Mail, Notes, Maps, third-party App Store apps)", {"macos": ["~/Library/Containers/*/Data/Library/Caches/*"]}, "moderate", os_=["macos"])
    simple("app-macos-app-logs", "App Logs", "Log files in ~/Library/Logs", {"macos": ["~/Library/Logs/*"]}, os_=["macos"])
    simple("app-macos-saved-state", "Saved Application State", "Window-restore state of apps (~/Library/Saved Application State)", {"macos": ["~/Library/Saved Application State/*"]}, os_=["macos"])
    return out


def games_pack():
    out = []

    def simple(id_, name, desc, d, risk="safe", os_=None):
        out.append(cleaner(id_, name, desc, "Games", risk, os_ or [], [(name, d)]))

    simple("game-steam-shader", "Steam Shader Cache", "Compiled shader caches in steamapps/shadercache (rebuilt on next launch)", {"linux": ["~/.steam/steam/steamapps/shadercache/*", f"{DATA}/Steam/steamapps/shadercache/*", "~/.var/app/com.valvesoftware.Steam/.local/share/Steam/steamapps/shadercache/*"], "macos": [f"{MAC}/Steam/steamapps/shadercache/*"], "windows": ["C:/Program Files (x86)/Steam/steamapps/shadercache/*"]}, "moderate")
    simple("game-steam-web-logs", "Steam Web Cache & Logs", "Steam browser/HTTP caches and logs", {"linux": ["~/.steam/steam/appcache/httpcache", "~/.steam/steam/config/htmlcache", "~/.steam/steam/logs/*", f"{DATA}/Steam/appcache/httpcache", f"{DATA}/Steam/config/htmlcache", f"{DATA}/Steam/logs/*"], "macos": [f"{MAC}/Steam/appcache/httpcache", f"{MAC}/Steam/config/htmlcache", f"{MAC}/Steam/logs/*"], "windows": ["C:/Program Files (x86)/Steam/appcache/httpcache", "C:/Program Files (x86)/Steam/config/htmlcache", "C:/Program Files (x86)/Steam/logs/*", f"{LAD}/Steam/htmlcache"]})
    simple("game-epic", "Epic Games Launcher Cache", "Web cache and logs of the Epic Games Launcher", {"macos": ["~/Library/Caches/com.epicgames.EpicGamesLauncher", f"{MAC}/Epic/EpicGamesLauncher/Saved/webcache*", f"{MAC}/Epic/EpicGamesLauncher/Saved/Logs/*"], "windows": [f"{LAD}/EpicGamesLauncher/Saved/webcache*", f"{LAD}/EpicGamesLauncher/Saved/Logs/*"]})
    simple("game-heroic-lutris", "Heroic / Lutris Caches", "Launcher caches of Heroic Games Launcher and Lutris", {"linux": [f"{CONF}/heroic/Cache", f"{CONF}/heroic/Code Cache", f"{CONF}/heroic/GPUCache", f"{CACHE}/lutris"]}, os_=["linux"])
    simple("game-wine", "Wine / Winetricks Caches", "Downloaded installers and caches of Wine and winetricks", {"linux": [f"{CACHE}/wine", f"{CACHE}/winetricks"], "macos": ["~/Library/Caches/winetricks", "~/Library/Caches/Wine"]}, "moderate", os_=["linux", "macos"])
    simple("game-minecraft", "Minecraft Logs & Crash Reports", "Launcher/game logs and crash reports", {"paths": ["~/.minecraft/logs/*", "~/.minecraft/crash-reports/*"], "macos": [f"{MAC}/minecraft/logs/*", f"{MAC}/minecraft/crash-reports/*"], "windows": [f"{AD}/.minecraft/logs/*", f"{AD}/.minecraft/crash-reports/*"]})
    simple("game-gpu-shaders", "GPU Shader Caches", "Mesa, NVIDIA, AMD and DirectX shader caches (rebuilt on demand)", {"linux": [f"{CACHE}/mesa_shader_cache", f"{CACHE}/mesa_shader_cache_db", f"{CACHE}/radv_builtin_shaders*", f"{CACHE}/nvidia/GLCache", f"{CACHE}/nvidia/DXCache", "~/.nv/GLCache", "~/.nv/ComputeCache", f"{CACHE}/AMD/GLCache"], "windows": [f"{LAD}/D3DSCache/*", f"{LAD}/NVIDIA/DXCache/*", f"{LAD}/NVIDIA/GLCache/*", f"{LAD}/AMD/DxCache/*", f"{LAD}/AMD/GLCache/*"]}, "moderate", os_=["linux", "windows"])
    return out


for fname, items in (("browsers.toml", chromium_pack()), ("apps.toml", apps_pack()), ("games.toml", games_pack())):
    header = "# GENERATED by scripts/gen_content_packs.py — edit the script, not this file.\n\n"
    (OUT / fname).write_text(header + "\n".join(items))
    print(fname, len(items), "cleaners")
