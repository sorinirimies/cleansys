#!/usr/bin/env nu
# Keep only the LATEST release's uploaded files (binaries, packages, installers).
#
# Older releases keep their notes and tag — only their attached assets are deleted, so
# the repository doesn't pile up hundreds of MB per version.
#
#   nu scripts/ci/prune_release_assets.nu github  owner/repo v0.7.0 [--dry-run]
#   nu scripts/ci/prune_release_assets.nu gitea   owner/repo v0.7.0 --url https://git.example.com [--dry-run]
#
# Token: $env.TOKEN (GitHub: GITHUB_TOKEN with contents:write; Gitea: an access token).
# Without --dry-run it really deletes. The release for <keep-tag> is never touched, and the
# newest release by date is also kept, whatever the tag says.

def api-get [url: string, token: string] {
    let headers = if ($token | is-empty) { [] } else { [Authorization $"token ($token)"] }
    http get --headers $headers $url
}

def api-delete [url: string, token: string] {
    http delete --headers [Authorization $"token ($token)"] $url | ignore
}

# All releases, newest first (handles pagination).
def list-releases [platform: string, repo: string, base: string, token: string] {
    mut all = []
    mut page = 1
    loop {
        let url = if $platform == "github" {
            $"https://api.github.com/repos/($repo)/releases?per_page=100&page=($page)"
        } else {
            $"($base)/api/v1/repos/($repo)/releases?limit=50&page=($page)"
        }
        let batch = (api-get $url $token)
        if ($batch | is-empty) { break }
        $all = ($all | append $batch)
        $page += 1
        if $page > 20 { break }
    }
    $all | sort-by --reverse created_at
}

def main [
    platform: string      # github | gitea
    repo: string          # owner/repo
    keep_tag: string      # tag of the release to keep (e.g. v0.7.0)
    --url: string = ""    # Gitea base URL
    --dry-run             # only print what would be deleted
] {
    if $platform not-in [github gitea] { error make {msg: "platform must be github or gitea"} }
    if $platform == "gitea" and ($url | is-empty) { error make {msg: "--url is required for gitea"} }
    let token = ($env.TOKEN? | default "")
    if not $dry_run and ($token | is-empty) { error make {msg: "set $env.TOKEN (needed to delete assets)"} }

    let releases = (list-releases $platform $repo ($url | str trim --right --char "/") $token)
    if ($releases | is-empty) { print "no releases found"; return }

    let newest = ($releases | first | get tag_name)
    let keep = [$keep_tag $newest] | uniq
    print $"keeping assets of: ($keep | str join ', ')"

    mut freed = 0
    mut count = 0
    for r in ($releases | where { |r| $r.tag_name not-in $keep }) {
        for a in ($r.assets? | default []) {
            let size = ($a.size? | default 0)
            print $"  ($r.tag_name)  ($a.name)  ($size | into filesize)"
            if not $dry_run {
                let del = if $platform == "github" {
                    $"https://api.github.com/repos/($repo)/releases/assets/($a.id)"
                } else {
                    $"($url | str trim --right --char '/')/api/v1/repos/($repo)/releases/($r.id)/assets/($a.id)"
                }
                api-delete $del $token
            }
            $freed += $size
            $count += 1
        }
    }
    let verb = if $dry_run { "would delete" } else { "deleted" }
    print $"($verb) ($count) asset\(s\), ($freed | into filesize)"
}
