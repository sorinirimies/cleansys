#!/usr/bin/env nu
# ── CleanSys · retry_cancelled.nu ───────────────────────────────────
# Re-run a Release run whose jobs a GitHub runner cancelled (lost runner, timeout) — but
# never a real failure, and at most twice. Called by .github/workflows/retry-release.yml.
# ────────────────────────────────────────────────────────────────────

export const MAX_ATTEMPTS = 3

# Rerun when some job was cancelled, none failed for real, and attempts are left.
export def should_retry [jobs: list<record>, attempt: int]: nothing -> bool {
    let cancelled = ($jobs | where conclusion == "cancelled" | length)
    let failed = ($jobs | where {|j| $j.conclusion in ["failure" "timed_out"] } | length)
    $cancelled > 0 and $failed == 0 and $attempt < $MAX_ATTEMPTS
}

def main [run_id: string, attempt: int] {
    let jobs = (^gh run view $run_id --json jobs | from json | get jobs | select name conclusion)
    if (should_retry $jobs $attempt) {
        print $"rerunning the cancelled jobs of run ($run_id) \(attempt ($attempt + 1) of ($MAX_ATTEMPTS)\)"
        ^gh run rerun $run_id --failed
    } else {
        print "nothing to retry (no cancelled job, a real failure, or out of attempts)"
    }
}
