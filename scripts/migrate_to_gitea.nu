#!/usr/bin/env nu
# ──────────────────────────────────────────────────────────────────────────────
# CleanSys — Migrate Repository to Gitea
# ──────────────────────────────────────────────────────────────────────────────
# Mirrors the CleanSys repository from GitHub (or any origin) to a Gitea
# instance. Supports both creating a new mirror and updating an existing one.
#
# Prerequisites:
#   - git CLI available on PATH
#   - A Gitea instance with API access (token-based auth)
#   - The `http` command (Nushell built-in) or `curl` on PATH
#
# Usage:
#   nu scripts/migrate_to_gitea.nu --url <gitea-url> --token <api-token> [--org <org>] [--repo <repo>] [--mirror]
#
# Examples:
#   nu scripts/migrate_to_gitea.nu --url https://gitea.example.com --token abc123
#   nu scripts/migrate_to_gitea.nu --url https://gitea.example.com --token abc123 --org cleansys --repo cleansys --mirror
# ──────────────────────────────────────────────────────────────────────────────

# Migrate/mirror the CleanSys repository to a Gitea instance.
def main [
    --url: string       # Gitea instance base URL (e.g. https://gitea.example.com)
    --token: string     # Gitea API token for authentication
    --org: string       # Target organisation or user on Gitea (default: cleansys)
    --repo: string      # Target repository name on Gitea (default: cleansys)
    --mirror            # Create a pull mirror instead of a one-time push
    --source: string    # Source repository URL to migrate from (default: current origin)
] {
    let gitea_url = if ($url | is-empty) {
        print "❌ --url is required. Provide your Gitea instance URL."
        exit 1
    } else {
        $url | str trim --right --char "/"
    }

    let api_token = if ($token | is-empty) {
        print "❌ --token is required. Provide a Gitea API token."
        exit 1
    } else {
        $token
    }

    let target_org = if ($org | is-empty) { "cleansys" } else { $org }
    let target_repo = if ($repo | is-empty) { "cleansys" } else { $repo }

    # Resolve the source repository URL
    let source_url = if ($source | is-empty) {
        let origin = (do { git remote get-url origin } | complete)
        if $origin.exit_code != 0 {
            print "❌ Could not determine origin remote. Pass --source explicitly."
            exit 1
        }
        $origin.stdout | str trim
    } else {
        $source
    }

    print $"(ansi cyan_bold)══════════════════════════════════════════════════════════════(ansi reset)"
    print $"(ansi cyan_bold)  CleanSys — Migrate to Gitea(ansi reset)"
    print $"(ansi cyan_bold)══════════════════════════════════════════════════════════════(ansi reset)"
    print ""
    print $"  Source URL   : (ansi green)($source_url)(ansi reset)"
    print $"  Gitea URL    : (ansi green)($gitea_url)(ansi reset)"
    print $"  Organisation : (ansi green)($target_org)(ansi reset)"
    print $"  Repository   : (ansi green)($target_repo)(ansi reset)"
    print $"  Mirror mode  : (ansi green)($mirror)(ansi reset)"
    print ""

    # ── Step 1: Verify Gitea connectivity ────────────────────────────────────
    print $"(ansi yellow)▶ Step 1: Verifying Gitea API connectivity …(ansi reset)"

    let health_result = (do {
        http get $"($gitea_url)/api/v1/version" --headers [
            Authorization $"token ($api_token)"
        ]
    } | complete)

    if $health_result.exit_code != 0 {
        print $"(ansi red)❌ Cannot reach Gitea at ($gitea_url). Check the URL and your network.(ansi reset)"
        exit 1
    }

    print $"  ✅ Gitea is reachable"
    print ""

    # ── Step 2: Check if target repo already exists ──────────────────────────
    print $"(ansi yellow)▶ Step 2: Checking if ($target_org)/($target_repo) already exists …(ansi reset)"

    let repo_check = (do {
        http get $"($gitea_url)/api/v1/repos/($target_org)/($target_repo)" --headers [
            Authorization $"token ($api_token)"
        ]
    } | complete)

    let repo_exists = ($repo_check.exit_code == 0)

    if $repo_exists {
        print $"  ⚠️  Repository ($target_org)/($target_repo) already exists on Gitea."
        print "  Will push updates to the existing repository."
    } else {
        print $"  Repository does not exist yet — will create it."
    }
    print ""

    if $mirror {
        # ── Mirror mode: use Gitea's migration API ───────────────────────────
        print $"(ansi yellow)▶ Step 3: Creating pull mirror via Gitea migration API …(ansi reset)"

        if $repo_exists {
            print $"  ⚠️  Mirror target already exists. Skipping creation."
            print "  If you need to re-create the mirror, delete the repo on Gitea first."
        } else {
            let migrate_body = {
                clone_addr: $source_url,
                repo_name: $target_repo,
                repo_owner: $target_org,
                mirror: true,
                service: "git",
                description: "CleanSys — a modern terminal & desktop system cleaner for Linux (mirror)"
            }

            let migrate_result = (do {
                http post $"($gitea_url)/api/v1/repos/migrate" ($migrate_body | to json) --content-type "application/json" --headers [
                    Authorization $"token ($api_token)"
                ]
            } | complete)

            if $migrate_result.exit_code != 0 {
                print $"(ansi red)❌ Migration API call failed.(ansi reset)"
                print $"  Response: ($migrate_result.stdout)"
                exit 1
            }

            print $"  ✅ Pull mirror created successfully at ($gitea_url)/($target_org)/($target_repo)"
        }
    } else {
        # ── Push mode: add Gitea as a remote and push ────────────────────────
        let gitea_remote_url = $"($gitea_url)/($target_org)/($target_repo).git"

        if not $repo_exists {
            print $"(ansi yellow)▶ Step 3: Creating repository on Gitea …(ansi reset)"

            let create_body = {
                name: $target_repo,
                description: "CleanSys — a modern terminal & desktop system cleaner for Linux",
                private: false,
                auto_init: false
            }

            # Try org endpoint first, fall back to user endpoint
            let create_result = (do {
                http post $"($gitea_url)/api/v1/orgs/($target_org)/repos" ($create_body | to json) --content-type "application/json" --headers [
                    Authorization $"token ($api_token)"
                ]
            } | complete)

            if $create_result.exit_code != 0 {
                print "  Organisation endpoint failed — trying user endpoint …"
                let user_create = (do {
                    http post $"($gitea_url)/api/v1/user/repos" ($create_body | to json) --content-type "application/json" --headers [
                        Authorization $"token ($api_token)"
                    ]
                } | complete)

                if $user_create.exit_code != 0 {
                    print $"(ansi red)❌ Failed to create repository on Gitea.(ansi reset)"
                    print $"  Response: ($user_create.stdout)"
                    exit 1
                }
            }

            print $"  ✅ Repository created on Gitea"
        } else {
            print $"(ansi yellow)▶ Step 3: Repository exists — skipping creation(ansi reset)"
        }

        print ""
        print $"(ansi yellow)▶ Step 4: Configuring git remote 'gitea' …(ansi reset)"

        # Check if the gitea remote already exists
        let remotes = (git remote | lines)
        if "gitea" in $remotes {
            print "  Remote 'gitea' already exists — updating URL …"
            git remote set-url gitea $gitea_remote_url
        } else {
            print "  Adding remote 'gitea' …"
            git remote add gitea $gitea_remote_url
        }

        print $"  Remote URL: ($gitea_remote_url)"
        print ""

        # ── Step 5: Push all branches and tags ───────────────────────────────
        print $"(ansi yellow)▶ Step 5: Pushing all branches and tags to Gitea …(ansi reset)"

        print "  Pushing branches …"
        let push_branches = (do { git push gitea --all --force } | complete)
        if $push_branches.exit_code != 0 {
            print $"(ansi red)  ⚠️  Branch push encountered issues:(ansi reset)"
            print $"  ($push_branches.stderr)"
        } else {
            print "  ✅ All branches pushed"
        }

        print "  Pushing tags …"
        let push_tags = (do { git push gitea --tags --force } | complete)
        if $push_tags.exit_code != 0 {
            print $"(ansi red)  ⚠️  Tag push encountered issues:(ansi reset)"
            print $"  ($push_tags.stderr)"
        } else {
            print "  ✅ All tags pushed"
        }
    }

    print ""
    print $"(ansi cyan_bold)══════════════════════════════════════════════════════════════(ansi reset)"
    print $"(ansi green_bold)  ✅ CleanSys migration to Gitea complete!(ansi reset)"
    print $"(ansi cyan_bold)══════════════════════════════════════════════════════════════(ansi reset)"
    print ""
    print $"  View your repository at:"
    print $"  (ansi blue_underline)($gitea_url)/($target_org)/($target_repo)(ansi reset)"
    print ""
}
