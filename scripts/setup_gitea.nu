#!/usr/bin/env nu
# ─────────────────────────────────────────────────────────────────────────────
# CleanSys — Setup Gitea Instance
# ─────────────────────────────────────────────────────────────────────────────
# Bootstraps a Gitea instance for hosting/mirroring the CleanSys repository.
#
# Prerequisites:
#   - Docker (or Podman) installed and running
#   - Network access to pull the Gitea image
#
# Usage:
#   nu scripts/setup_gitea.nu [--port <port>] [--data-dir <path>] [--name <container>]
#
# Options:
#   --port      HTTP port for the Gitea web UI        (default: 3000)
#   --data-dir  Host directory for persistent storage  (default: ./gitea-data)
#   --name      Docker container name                  (default: cleansys-gitea)
# ─────────────────────────────────────────────────────────────────────────────

def main [
    --port: int = 3000        # HTTP port for the Gitea web UI
    --data-dir: string = "./gitea-data"  # Host directory for persistent storage
    --name: string = "cleansys-gitea"    # Docker container name
] {
    print "═══════════════════════════════════════════════════════════════"
    print "  ⚡ CleanSys — Gitea Instance Setup"
    print "═══════════════════════════════════════════════════════════════"
    print ""

    # ── Step 0: Detect container runtime ──────────────────────────────────
    let runtime = detect_runtime
    print $"  ▸ Container runtime: ($runtime)"
    print $"  ▸ Container name:    ($name)"
    print $"  ▸ HTTP port:         ($port)"
    print $"  ▸ Data directory:    ($data_dir)"
    print ""

    # ── Step 1: Check for existing container ──────────────────────────────
    print "── Step 1: Checking for existing container ──"
    let existing = (do { ^$runtime ps -a --format "{{.Names}}" } | complete)
    if ($existing.exit_code == 0) and ($existing.stdout | str contains $name) {
        print $"  ⚠ Container '($name)' already exists."
        print "  Removing existing container..."
        ^$runtime stop $name | ignore
        ^$runtime rm $name | ignore
        print "  ✓ Old container removed."
    } else {
        print "  ✓ No existing container found."
    }
    print ""

    # ── Step 2: Create data directory ─────────────────────────────────────
    print "── Step 2: Creating data directory ──"
    let data_path = ($data_dir | path expand)
    if not ($data_path | path exists) {
        mkdir $data_path
        print $"  ✓ Created ($data_path)"
    } else {
        print $"  ✓ Data directory already exists: ($data_path)"
    }
    print ""

    # ── Step 3: Pull the Gitea image ──────────────────────────────────────
    print "── Step 3: Pulling latest Gitea image ──"
    let pull_result = (do { ^$runtime pull "gitea/gitea:latest" } | complete)
    if $pull_result.exit_code != 0 {
        print "  ✗ Failed to pull Gitea image."
        print $"    ($pull_result.stderr)"
        exit 1
    }
    print "  ✓ Gitea image is up to date."
    print ""

    # ── Step 4: Start the Gitea container ─────────────────────────────────
    print "── Step 4: Starting Gitea container ──"
    let ssh_port = ($port + 22)
    let run_result = (
        do {
            ^$runtime run -d
                --name $name
                --restart unless-stopped
                -p $"($port):3000"
                -p $"($ssh_port):22"
                -v $"($data_path):/data"
                -e "GITEA__server__ROOT_URL"=$"http://localhost:($port)/"
                -e "GITEA__server__HTTP_PORT"="3000"
                -e "GITEA__repository__DEFAULT_BRANCH"="main"
                "gitea/gitea:latest"
        } | complete
    )

    if $run_result.exit_code != 0 {
        print "  ✗ Failed to start Gitea container."
        print $"    ($run_result.stderr)"
        exit 1
    }
    print $"  ✓ Gitea container '($name)' is running."
    print ""

    # ── Step 5: Wait for Gitea to become ready ────────────────────────────
    print "── Step 5: Waiting for Gitea to become ready ──"
    let max_attempts = 30
    mut ready = false
    for attempt in 1..($max_attempts + 1) {
        sleep 2sec
        let health = (do { curl -s -o /dev/null -w "%{http_code}" $"http://localhost:($port)/" } | complete)
        if ($health.exit_code == 0) and (($health.stdout | str trim) =~ '^(200|302)$') {
            $ready = true
            break
        }
        print $"  … attempt ($attempt)/($max_attempts)"
    }

    if not $ready {
        print "  ⚠ Gitea did not become ready within the timeout."
        print "    The container is running — it may just need more time."
        print $"    Try opening http://localhost:($port) in your browser."
    } else {
        print "  ✓ Gitea is responding."
    }
    print ""

    # ── Done ──────────────────────────────────────────────────────────────
    print "═══════════════════════════════════════════════════════════════"
    print "  ✓ CleanSys Gitea setup complete!"
    print ""
    print $"  Web UI:      http://localhost:($port)"
    print $"  SSH clone:   ssh://git@localhost:($ssh_port)/<owner>/<repo>.git"
    print $"  Data dir:    ($data_path)"
    print $"  Container:   ($name)"
    print ""
    print "  Next steps:"
    print "    1. Open the Web UI and complete the initial configuration."
    print "    2. Create an admin account."
    print "    3. Create a 'cleansys' repository (or use migrate_to_gitea.nu)."
    print ""
    print "  To stop:   docker stop ($name)"
    print "  To start:  docker start ($name)"
    print "  To remove: docker rm -f ($name)"
    print "═══════════════════════════════════════════════════════════════"
}

# Detect whether Docker or Podman is available.
def detect_runtime [] -> string {
    let docker_check = (do { which docker } | complete)
    if $docker_check.exit_code == 0 and ($docker_check.stdout | str trim | str length) > 0 {
        return "docker"
    }

    let podman_check = (do { which podman } | complete)
    if $podman_check.exit_code == 0 and ($podman_check.stdout | str trim | str length) > 0 {
        return "podman"
    }

    print "  ✗ Neither Docker nor Podman found. Please install one first."
    exit 1
}
