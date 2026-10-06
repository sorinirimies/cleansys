#!/usr/bin/env nu
# Keep only the LATEST release's uploaded files (binaries, packages, installers).
#
# Older releases keep their notes and tag — only their attached assets are deleted, so the
# repository never piles up hundreds of MB per version.
#
# Everything is auto-detected, so locally this is just:
#
#   nu scripts/ci/prune_release_assets.nu --dry-run      # preview (default remote: origin)
#   nu scripts/ci/prune_release_assets.nu                # really delete
#   nu scripts/ci/prune_release_assets.nu --remote gitea-starscream --url http://192.168.1.44:3000
#   nu scripts/ci/prune_release_assets.nu --keep 2       # keep the two newest versions
#
# CI passes everything explicitly:
#   nu scripts/ci/prune_release_assets.nu github owner/repo v0.7.0
#   nu scripts/ci/prune_release_assets.nu gitea  owner/repo v0.7.0 --url https://git.example.com
#
# Auto-detection
#   platform/repo  from `git remote get-url <remote>` (github.com => github, anything else => gitea)
#   token          $env.TOKEN, then $env.GITHUB_TOKEN / $env.GITEA_TOKEN, then `gh auth token`
#   kept release   <keep-tag> if given, plus the newest --keep releases by date

# `git@github.com:owner/repo.git` / `https://host/owner/repo` -> {host, repo}
export def parse_remote_url [raw: string]: nothing -> record {
    let m = ($raw | str trim | parse --regex '^(?:[a-z+]+://)?(?:[^@/]+@)?(?<host>[^/:]+)(?::\d+)?[:/](?<repo>[^/]+/[^/]+?)(?:\.git)?/?$')
    if ($m | is-empty) { error make {msg: $"cannot parse remote url: ($raw)"} }
    $m | first
}

# Tags of the `n` newest releases (input is already newest-first).
export def newest_tags [releases: list, n: int]: nothing -> list {
    $releases | first $n | get tag_name
}

# Which assets to delete: those of every release whose tag is not in `keepers`.
# Fails when `keepers` is empty, so a mistake can never wipe everything.
export def assets_to_prune [releases: list, keepers: list]: nothing -> list {
    if ($keepers | is-empty) {
        error make --unspanned { msg: "no release to keep; refusing to prune" }
    }
    $releases
    | where { |r| $r.tag_name not-in $keepers }
    | each { |r|
        $r.assets? | default [] | each { |a|
            { release: $r.id, tag: $r.tag_name, id: $a.id, name: $a.name, size: ($a.size? | default 0) }
        }
    }
    | flatten
}

# Total bytes of a list of assets (0 for an empty list).
export def total_size [assets: list]: nothing -> int {
    $assets | each { |a| $a.size } | append 0 | math sum
}

def parse-remote [remote: string] {
    parse_remote_url (^git remote get-url $remote)
}

def find-token [platform: string] {
    let t = ($env.TOKEN? | default ($env | get -o (if $platform == "github" { "GITHUB_TOKEN" } else { "GITEA_TOKEN" })) | default "")
    if ($t | is-not-empty) { return $t }
    if $platform == "github" and (which gh | is-not-empty) {
        return (try { ^gh auth token | str trim } catch { "" })
    }
    ""
}

def auth-headers [platform: string, token: string] {
    if ($token | is-empty) { [] } else if $platform == "github" {
        [Authorization $"Bearer ($token)" Accept "application/vnd.github+json"]
    } else {
        [Authorization $"token ($token)"]
    }
}

# All releases, newest first (handles pagination).
def list-releases [platform: string, repo: string, base: string, headers: list] {
    mut all = []
    mut page = 1
    loop {
        let url = if $platform == "github" {
            $"https://api.github.com/repos/($repo)/releases?per_page=100&page=($page)"
        } else {
            $"($base)/api/v1/repos/($repo)/releases?limit=50&page=($page)"
        }
        let batch = (http get --headers $headers $url)
        if ($batch | is-empty) { break }
        $all = ($all | append $batch)
        $page += 1
        if $page > 30 { break }
    }
    $all | sort-by --reverse created_at
}

def main [
    platform?: string        # github | gitea  (default: detected from the git remote)
    repo?: string            # owner/repo       (default: detected from the git remote)
    keep_tag?: string        # always keep this release too (e.g. v0.7.0)
    --remote: string = "origin"
    --url: string = ""       # Gitea base URL (default: https://<remote host>)
    --keep: int = 1          # how many of the newest releases keep their assets
    --dry-run                # only print what would be deleted
] {
    let det = if ($platform | is-empty) or ($repo | is-empty) { parse-remote $remote } else { null }
    let det_host = ($det | get -o host | default "")
    let platform = ($platform | default (if $det_host == "github.com" { "github" } else { "gitea" }))
    let repo = ($repo | default ($det | get -o repo | default ""))
    if $platform not-in [github gitea] { error make {msg: "platform must be github or gitea"} }
    let base = if $platform == "gitea" {
        let b = (if ($url | is-not-empty) { $url } else if ($det_host | is-not-empty) { $"https://($det_host)" } else { "" })
        if ($b | is-empty) { error make {msg: "--url is required for gitea"} }
        $b | str trim --right --char "/"
    } else { "" }

    let token = (find-token $platform)
    if not $dry_run and ($token | is-empty) {
        error make {msg: "no token: set $env.TOKEN (or GITHUB_TOKEN/GITEA_TOKEN, or `gh auth login`)"}
    }
    let headers = (auth-headers $platform $token)

    let releases = (list-releases $platform $repo $base $headers)
    if ($releases | is-empty) { print "no releases found"; return }

    # An explicit keep-tag must exist: a typo must never turn into "delete everything".
    if ($keep_tag | is-not-empty) and ($releases | where tag_name == $keep_tag | is-empty) {
        error make {msg: $"release ($keep_tag) not found; refusing to prune"}
    }
    let keepers = ([$keep_tag] | append (newest_tags $releases $keep) | where { |t| $t != null } | uniq)
    print $"($platform):($repo)  keeping assets of: ($keepers | str join ', ')"

    let doomed = (assets_to_prune $releases $keepers)
    for a in $doomed {
        print $"  ($a.tag)  ($a.name)  ($a.size | into filesize)"
        if not $dry_run {
            let del = if $platform == "github" {
                $"https://api.github.com/repos/($repo)/releases/assets/($a.id)"
            } else {
                $"($base)/api/v1/repos/($repo)/releases/($a.release)/assets/($a.id)"
            }
            http delete --headers $headers $del | ignore
        }
    }
    let verb = if $dry_run { "would delete" } else { "deleted" }
    print $"($verb) ($doomed | length) asset\(s\), (total_size $doomed | into filesize)"
}
