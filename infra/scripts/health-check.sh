#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════════════
# health-check.sh — ZafafWorld Production Infrastructure Monitor
# ═══════════════════════════════════════════════════════════════════════════════
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Source config
INFRA_ENV="$SCRIPT_DIR/../.env"
if [[ -f "$INFRA_ENV" ]]; then
    set -a; source "$INFRA_ENV"; set +a
fi

# Source logging helper
if [[ -f "$SCRIPT_DIR/../lib/logging.sh" ]]; then
    source "$SCRIPT_DIR/../lib/logging.sh"
else
    log_info() { echo -e "[INFO] $*"; }
    log_success() { echo -e "\033[0;32m[PASS]\033[0m $*"; }
    log_warn() { echo -e "\033[1;33m[WARN]\033[0m $*"; }
    log_error() { echo -e "\033[0;31m[FAIL]\033[0m $*" >&2; }
fi

DEPLOY_ROOT="${DEPLOY_ROOT:-/opt/zafafworld.net}"

log_info "══ ZafafWorld Production Health Monitor ══"
echo

# 1. Container Status & Stats
log_info "1. Container Resource Stats & Health"
if command -v podman &>/dev/null; then
    podman stats --no-stream --format "table {{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.MemPerc}}\t{{.NetIO}}" || true
else
    log_error "Error: podman command not found."
fi
echo

# 2. Container Restart and Exit Code Check
log_info "2. Container Restarts and Exit Codes"
if command -v podman &>/dev/null; then
    podman ps -a --format "{{.Names}}" | while read -r container; do
        if [ -n "$container" ]; then
            RESTARTS=$(podman inspect "$container" --format '{{.RestartCount}}' 2>/dev/null || echo "0")
            EXIT_CODE=$(podman inspect "$container" --format '{{.State.ExitCode}}' 2>/dev/null || echo "0")
            STATUS=$(podman inspect "$container" --format '{{.State.Status}}' 2>/dev/null || echo "unknown")
            HEALTH=$(podman inspect "$container" --format '{{if .State.Health}}{{.State.Health.Status}}{{else}}N/A{{end}}' 2>/dev/null || echo "N/A")
            
            if [ "$RESTARTS" -gt 0 ] || [ "$EXIT_CODE" -ne 0 ] || [ "$STATUS" != "running" ]; then
                log_warn "  Container: $container Status: $STATUS Health: $HEALTH Restarts: $RESTARTS ExitCode: $EXIT_CODE"
            else
                log_success "  Container: $container Status: $STATUS Health: $HEALTH Restarts: $RESTARTS"
            fi
        fi
    done
fi
echo

# 3. Kernel OOM Killer Check
log_info "3. Kernel OOM Killer Events"
OOM_SYS=$(dmesg 2>/dev/null | grep -iE "oom[-_]killer|killed process" || true)
OOM_JRNL=$(journalctl --user --since "24 hours ago" --no-pager 2>/dev/null | grep -iE "oom[-_]killer|killed process|exit code 137" || true)

if [ -n "$OOM_SYS" ] || [ -n "$OOM_JRNL" ]; then
    log_error "OOM-killer activity detected in system or user logs!"
    if [ -n "$OOM_SYS" ]; then
        log_warn "dmesg events:"
        echo "$OOM_SYS" | tail -n 5 | sed 's/^/  /'
    fi
    if [ -n "$OOM_JRNL" ]; then
        log_warn "journalctl events (last 24 hours):"
        echo "$OOM_JRNL" | tail -n 5 | sed 's/^/  /'
    fi
else
    log_success "No OOM-killer events detected in dmesg or journalctl."
fi
echo

# 4. Nginx Error Log Scans
log_info "4. Nginx Upstream/Proxy Errors"
NGINX_LOG_DIR="${DEPLOY_ROOT}/infra/nginx/logs"
if [ -d "$NGINX_LOG_DIR" ]; then
    ERR_COUNT=$(grep -cE "502|upstream|connection refused|stale" "$NGINX_LOG_DIR"/error.log 2>/dev/null || echo "0")
    if [ "$ERR_COUNT" -gt 0 ]; then
        log_warn "Detected $ERR_COUNT proxy/upstream warning(s) in Nginx error.log:"
        grep -E "502|upstream|connection refused|stale" "$NGINX_LOG_DIR"/error.log 2>/dev/null | tail -n 10 | sed 's/^/  /' || true
    else
        log_success "No upstream proxy errors found in Nginx error.log."
    fi
else
    NGINX_CONTAINER=$(podman ps --format '{{.Names}}' 2>/dev/null | grep -i "nginx" | head -1 || true)
    if [ -n "$NGINX_CONTAINER" ]; then
        ERR_COUNT=$(podman logs "$NGINX_CONTAINER" 2>&1 | grep -cE "502|upstream|connection refused|stale" || echo "0")
        if [ "$ERR_COUNT" -gt 0 ]; then
            log_warn "Detected $ERR_COUNT proxy/upstream warning(s) in container logs:"
            podman logs "$NGINX_CONTAINER" 2>&1 | grep -E "502|upstream|connection refused|stale" | tail -n 10 | sed 's/^/  /' || true
        else
            log_success "No upstream proxy errors found in Nginx container logs."
        fi
    fi
fi
