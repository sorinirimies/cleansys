#!/usr/bin/env nu
# ──────────────────────────────────────────────────────────────────────────────
# CleanSys — Create a Gitea release and upload the archives
# ──────────────────────────────────────────────────────────────────────────────
# Usage (in .gitea/workflows/release.yml):
#   nu scripts/ci/gitea_release.nu <tag> --api <server>/api/v1/repos/<owner>/<repo> --dir dist
# Env: GITEA_TOKEN (the workflow passes secrets.GITHUB_TOKEN). Notes: RELEASE_NOTES.md.
# ──────────────────────────────────────────────────────────────────────────────

export def release_payload [tag: string, title: string, notes: string] {
    { tag_name: $tag, name: $"($title) ($tag)", body: $notes }
}

# Existing release for the tag (a re-run after a partial upload), or null.
def existing_release [api: string, tag: string, headers: record] {
    try { http get --headers $headers $"($api)/releases/tags/($tag)" } catch { null }
}

# Stream one file with curl: no whole-file read into memory, retries on a reset connection.
def upload [api: string, id: int, file: string, token: string] {
    let name = $file | path basename
    let url = $"($api)/releases/($id)/assets?name=($name)"
    let res = (^curl --silent --show-error --fail-with-body --retry 5 --retry-delay 5 --retry-all-errors --connect-timeout 30
        --header $"Authorization: token ($token)" --form $"attachment=@($file)" $url | complete)
    if $res.exit_code != 0 { error make { msg: $"uploading ($name) failed: ($res.stderr) ($res.stdout)" } }
}

def main [tag: string, --api: string, --dir: path = "dist", --title: string = "CleanSys"] {
    let headers = { Authorization: $"token ($env.GITEA_TOKEN)" }
    let notes = if ("RELEASE_NOTES.md" | path exists) { open --raw RELEASE_NOTES.md } else { "" }
    # Re-runnable: reuse the release (and skip assets already there) instead of failing on a duplicate tag.
    let release = (existing_release $api $tag $headers | default (
        http post --content-type application/json --headers $headers $"($api)/releases" (release_payload $tag $title $notes)
    ))
    let have = ($release.assets? | default [] | get name)
    for f in (glob --no-dir $"($dir)/*") {
        let name = $f | path basename
        if $name in $have {
            print $"⏭  ($name) already uploaded"
            continue
        }
        upload $api $release.id $f $env.GITEA_TOKEN
        print $"📦 ($name)"
    }
}
