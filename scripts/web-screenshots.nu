#!/usr/bin/env nu
# Screenshots + GIFs of the web UI for the README.
#
#   nu scripts/web-screenshots.nu
#
# Starts cleansys-web against the synthetic demo HOME (demo/fixture.nu), drives it with
# plain HTTP, and captures pages with a headless Chromium-based browser (Brave, Chrome
# or Chromium; override with BROWSER=/path/to/it). GIFs are assembled with ffmpeg.
#
# Output (demo/previews/): web-wide.png web-medium.png web-narrow.png web-preview.png
#                          web-schedule.png web-settings.png web-about.png web-details.png web-flow.gif web-responsive.gif
#
# Requires: nushell, ffmpeg, a Chromium-based browser, and a release build of cleansys-web.

source-env ../demo/fixture.nu

const ROOT = (path self | path dirname | path dirname)
const PORT = 3971

def find-browser [] {
    let candidates = [
        ($env.BROWSER? | default "")
        "/Applications/Brave Browser.app/Contents/MacOS/Brave Browser"
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
        "/Applications/Chromium.app/Contents/MacOS/Chromium"
    ]
    let found = ($candidates | where { |c| ($c | is-not-empty) and ($c | path exists) })
    if ($found | is-not-empty) { return ($found | first) }
    for name in [chromium google-chrome brave-browser chrome] {
        let w = (which $name)
        if ($w | is-not-empty) { return ($w | first | get path) }
    }
    error make {msg: "no Chromium-based browser found; set BROWSER=/path/to/it"}
}

# Run the browser with a watchdog: some Chromium builds leave helper processes holding the
# pipe open after the screenshot is written, which would hang a plain `e>| ignore`. Output
# goes to /dev/null (no pipe to wait on) and perl's alarm kills a browser that never exits.
def run-browser [browser: string, args: list<string>] {
    # The fixture points HOME at the demo directory; a browser started from there has to
    # create (and time out creating) a brand-new profile every time, so give it the real HOME.
    let real_home = (^sh -c 'eval echo "~$(id -un)"' | str trim)
    # No headless instance may be left before/after: a second one would hand off to a
    # still-exiting first one and never write its screenshot.
    try { ^pkill -f "headless=new" }
    with-env {HOME: $real_home} {
        try { ^perl -e 'alarm 15; exec @ARGV' $browser ...$args o> /dev/null e> /dev/null }
    }
    try { ^pkill -f "headless=new" }
}

def shoot [browser: string, url: string, width: int, height: int, out: string] {
    # --force-device-scale-factor=2 keeps text crisp in the README
    run-browser $browser [--headless=new --no-sandbox --disable-gpu --hide-scrollbars --force-device-scale-factor=2 $"--window-size=($width),($height)" $"--screenshot=($out)" $url]
    print $"✓ ($out)"
}

# Chromium refuses windows narrower than ~500 px, so phone-width shots render the page in an
# iframe of the wanted width (media queries see the iframe's width) and crop to it.
def shoot-narrow [browser: string, url: string, width: int, height: int, out: string] {
    let win = 520
    let page = (mktemp -t cleansys-frame.XXXX | $"($in).html")
    $"<!doctype html><body style='margin:0;background:#11111b'><iframe src='($url)' style='border:0;width:($width)px;height:($height)px;display:block;margin:0 auto'></iframe></body>" | save -f $page
    let raw = (mktemp -t cleansys-narrow.XXXX | $"($in).png")
    run-browser $browser [--headless=new --no-sandbox --disable-gpu --hide-scrollbars --force-device-scale-factor=2 $"--window-size=($win),($height)" $"--screenshot=($raw)" $"file://($page)"]
    let x = ((($win - $width) / 2 | into int) * 2)
    ^ffmpeg -v error -y -i $raw -vf $"crop=($width * 2):($height * 2):($x):0" $out
    rm -f $page $raw
    print $"✓ ($out)"
}

def post [path: string, form: record = {}] {
    http post --content-type "application/x-www-form-urlencoded" $"http://127.0.0.1:($PORT)($path)" $form | ignore
}

def wait-scan [] {
    for _ in 1..240 {
        # the server may still be starting: treat connection errors as "not ready yet"
        let s = (try { http get $"http://127.0.0.1:($PORT)/api/status" } catch { null })
        if $s != null and not $s.scanning { return }
        sleep 500ms
    }
    error make {msg: "scan did not finish"}
}

# images -> gif (one frame every `secs` seconds)
def make-gif [frames: list<string>, out: string, secs: float, width: int] {
    let dir = (mktemp -d)
    $frames | enumerate | each { |f| cp $f.item ($dir | path join $"f($f.index | fill -a r -c '0' -w 3).png") } | ignore
    ^ffmpeg -v error -y -framerate (1 / $secs) -i ($dir | path join "f%03d.png") -vf $"scale=($width):-1:flags=lanczos,split[a][b];[a]palettegen=max_colors=160:stats_mode=diff[p];[b][p]paletteuse=dither=sierra2_4a" $out
    rm -rf $dir
    print $"✓ ($out)"
}

def main [] {
    let bin = ($ROOT | path join "target" "release" "cleansys-web")
    if not ($bin | path exists) { error make {msg: "build first: cargo build --release -p cleansys-web"} }
    let browser = (find-browser)
    let out = ($ROOT | path join "demo" "previews")
    let tmp = (mktemp -d)
    mkdir $out

    let server = (job spawn { ^$bin --port $PORT --no-open | ignore })
    sleep 2sec
    wait-scan
    let base = $"http://127.0.0.1:($PORT)"

    # ── responsive layouts (same state: a few cleaners ticked) ──
    post "/toggle" {id: "core-user-browser-caches", back: "/"}
    post "/toggle" {id: "core-user-application-caches", back: "/"}
    post "/toggle" {id: "core-user-temporary-files", back: "/"}
    post "/toggle" {id: "dev-gradle-caches", back: "/"}
    post "/toggle" {id: "proj-rust", back: "/"}
    shoot $browser $"($base)/" 1280 860 ($out | path join "web-wide.png")
    shoot $browser $"($base)/" 820 820 ($out | path join "web-medium.png")
    shoot-narrow $browser $"($base)/" 420 900 ($out | path join "web-narrow.png")
    shoot $browser $"($base)/preview" 1100 900 ($out | path join "web-preview.png")
    shoot $browser $"($base)/schedule" 1100 900 ($out | path join "web-schedule.png")
    shoot $browser $"($base)/settings" 1100 800 ($out | path join "web-settings.png")
    shoot $browser $"($base)/about" 1100 640 ($out | path join "web-about.png")
    make-gif [
        ($out | path join "web-wide.png")
        ($out | path join "web-medium.png")
        ($out | path join "web-narrow.png")
    ] ($out | path join "web-responsive.gif") 1.6 900

    # ── a real clean (inside the demo HOME): tick -> confirm -> progress -> done ──
    post "/select" {op: "none", back: "/"}
    post "/toggle" {id: "proj-rust", back: "/"}
    post "/toggle" {id: "proj-js-build-caches", back: "/"}
    post "/toggle" {id: "proj-gradle", back: "/"}
    # open Gradle's details and untick one path: it is left alone by the clean
    post "/toggle-entry" {path: ($env.HOME | path join "Projects" "android-shop" "app" "build"), back: "/"}
    shoot $browser $"($base)/?cat=2&open=proj-gradle" 1100 900 ($out | path join "web-details.png")
    let f0 = ($tmp | path join "0.png"); shoot $browser $"($base)/?cat=2&open=proj-gradle" 1100 760 $f0
    let f1 = ($tmp | path join "1.png"); shoot $browser $"($base)/?cat=2" 1100 760 $f1
    let f2 = ($tmp | path join "2.png"); shoot $browser $"($base)/confirm" 1100 760 $f2
    post "/run"
    sleep 4sec
    let f3 = ($tmp | path join "3.png"); shoot $browser $"($base)/progress" 1100 760 $f3
    post "/dismiss"
    wait-scan
    let f4 = ($tmp | path join "4.png"); shoot $browser $"($base)/?cat=2" 1100 760 $f4
    make-gif [$f0 $f1 $f2 $f3 $f4] ($out | path join "web-flow.gif") 2.0 900

    ^pkill -f "release/cleansys-web"
    try { job kill $server }
    rm -rf $tmp
}
