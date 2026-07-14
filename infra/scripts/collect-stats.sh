#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════════════
# collect-stats.sh — Logs client-web memory usage periodically
# ═══════════════════════════════════════════════════════════════════════════════
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Source config
INFRA_ENV="$SCRIPT_DIR/../.env"
if [[ -f "$INFRA_ENV" ]]; then
    set -a; source "$INFRA_ENV"; set +a
fi

DEPLOY_ROOT="${DEPLOY_ROOT:-/opt/zafafworld.net}"

TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
MEM_MB=$(podman stats --no-stream --format "{{.MemUsage}}" zafafworld_client-web_1 2>/dev/null | awk '{
    val = $1;
    gsub(/[A-Za-z]/, "", val);
    unit = $1;
    gsub(/[0-9.]/, "", unit);
    if (unit ~ /[Gg]/) {
        print val * 1024;
    } else if (unit ~ /[Kk]/) {
        print val / 1024;
    } else {
        print val;
    }
}' || echo "0")

echo "$TIMESTAMP $MEM_MB" >> "$DEPLOY_ROOT/infra/scripts/stats-history.log"
