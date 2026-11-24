#!/usr/bin/env nu
# ── CleanSys · release_after_ci.nu ──────────────────────────────────
# Push a release only once GitHub CI (Linux, macOS and Windows) is green on it.
#
# Pushing the tag starts release.yml straight away; when the tag went out first and a
# platform test failed afterwards, the version was burned (0.21.0, 0.23.0). So:
#   main <version>      push the bump commit, wait for CI on it, then push the tag
#                       `just bump` made. CI red: the local tag is removed.
#   main tag <version>  after fixing: wait for CI on HEAD, then tag HEAD and push it.
# ────────────────────────────────────────────────────────────────────

# CI's verdict on one commit from `gh run list --commit SHA --json status,conclusion,event`
# (newest first): "pending" until a push run has finished, then "success" or "failure"
# (failure, cancelled — e.g. replaced by a newer push — and timed out all count as red).
export def ci_verdict [runs: list<record>]: nothing -> string {
    let push = ($runs | where {|r| ($r.event? | default "push") == "push" })
    if ($push | is-empty) { return "pending" }
    let latest = ($push | first)
    if $latest.status != "completed" { return "pending" }
    if $latest.conclusion == "success" { "success" } else { "failure" }
}

def wait_for_ci [sha: string, timeout_min: int, poll_sec: int] {
    let deadline = ((date now) + ($timeout_min * 1min))
    print $"waiting for CI on ($sha | str substring 0..<7), up to ($timeout_min) min…"
    loop {
        let runs = (^gh run list --commit $sha --workflow ci.yml --json status,conclusion,event | from json)
        let v = (ci_verdict $runs)
        if $v != "pending" { return $v }
        if (date now) > $deadline { return "timeout" }
        sleep ($poll_sec * 1sec)
    }
}

def red [msg: string] { print $"(ansi red)✗(ansi reset) ($msg)" }

def main [version: string, --remote: string = "origin", --timeout-min: int = 90, --poll-sec: int = 30] {
    let tag = $"v($version)"
    if (^git tag --list $tag | str trim | is-empty) {
        red $"no local tag ($tag): run `just bump ($version)` first"
        exit 1
    }
    let branch = (^git branch --show-current | str trim)
    let sha = (^git rev-parse HEAD | str trim)
    ^git push $remote $branch
    let v = (wait_for_ci $sha $timeout_min $poll_sec)
    if $v == "success" {
        ^git push $remote $tag
        print $"(ansi green)✅(ansi reset) CI green — ($tag) pushed, the Release workflow builds the binaries."
        return
    }
    ^git tag -d $tag | ignore
    red $"CI on ($sha | str substring 0..<7) is ($v): ($tag) was NOT pushed (local tag removed)."
    print $"fix it, push the fix, then: just release-tag ($version)"
    exit 1
}

def "main tag" [version: string, --remote: string = "origin", --timeout-min: int = 90, --poll-sec: int = 30] {
    let tag = $"v($version)"
    let branch = (^git branch --show-current | str trim)
    ^git push $remote $branch
    let sha = (^git rev-parse HEAD | str trim)
    let v = (wait_for_ci $sha $timeout_min $poll_sec)
    if $v != "success" {
        red $"CI on ($sha | str substring 0..<7) is ($v): not tagging."
        exit 1
    }
    ^git tag -a $tag -m $"Release ($tag)"
    ^git push $remote $tag
    print $"(ansi green)✅(ansi reset) ($tag) tagged on a green commit and pushed."
}
