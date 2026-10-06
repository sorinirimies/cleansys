#!/usr/bin/env nu
# ── CleanSys · test_prune_release_assets.nu ─────────────────────────────────
# Tests for scripts/ci/prune_release_assets.nu — which release files get deleted.

use std/assert
use runner.nu *
use ../ci/prune_release_assets.nu [assets_to_prune total_size newest_tags parse_remote_url]

def release [id: int, tag: string, assets: int]: nothing -> record {
    {
        id: $id
        tag_name: $tag
        assets: (0..<$assets | each { |i| { id: $i, name: $"($tag)-($i).bin", size: 1000 } })
    }
}

# newest first, like the API after sorting
def releases []: nothing -> list {
    [(release 3 "v0.7.0" 3) (release 2 "v0.6.22" 2) (release 1 "v0.6.21" 1)]
}

def "test prune: only the latest release keeps its files" [] {
    let doomed = (assets_to_prune (releases) ["v0.7.0"])
    assert equal ($doomed | length) 3
    assert equal ($doomed | where tag == "v0.7.0" | length) 0 "the kept release is untouched"
    assert equal ($doomed | get tag | uniq | sort) ["v0.6.21" "v0.6.22"]
    assert equal (total_size $doomed) 3000
}

def "test prune: --keep 2 keeps the two newest" [] {
    let keepers = (newest_tags (releases) 2)
    assert equal $keepers ["v0.7.0" "v0.6.22"]
    let doomed = (assets_to_prune (releases) $keepers)
    assert equal ($doomed | get tag | uniq) ["v0.6.21"]
}

def "test prune: each asset carries what the delete call needs" [] {
    let first = (assets_to_prune (releases) ["v0.7.0"] | where tag == "v0.6.22" | first)
    assert equal $first.release 2
    assert equal $first.id 0
    assert equal $first.name "v0.6.22-0.bin"
}

def "test prune: nothing to keep refuses instead of wiping everything" [] {
    let failed = (try { assets_to_prune (releases) []; false } catch { true })
    assert $failed
}

def "test prune: releases without assets are fine" [] {
    assert equal (assets_to_prune [{id: 1, tag_name: "v1"} {id: 2, tag_name: "v2", assets: []}] ["v2"]) []
    assert equal (total_size []) 0
}

def "test remote urls: ssh, https and ports" [] {
    assert equal (parse_remote_url "git@github.com:sorinirimies/cleansys.git") {host: "github.com", repo: "sorinirimies/cleansys"}
    assert equal (parse_remote_url "https://github.com/a/b") {host: "github.com", repo: "a/b"}
    assert equal (parse_remote_url "gitea@192.168.1.44:sorin/cleansys.git") {host: "192.168.1.44", repo: "sorin/cleansys"}
    assert equal (parse_remote_url "http://git.local:3000/o/r.git/") {host: "git.local", repo: "o/r"}
}

def main [] { run-tests }
