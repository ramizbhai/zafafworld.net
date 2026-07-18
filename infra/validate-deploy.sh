#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════════════
# infra/validate-deploy.sh — ZafafWorld Post-Deployment Health Validation
# ═══════════════════════════════════════════════════════════════════════════════
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Source config
INFRA_ENV="$SCRIPT_DIR/.env"
if [[ -f "$INFRA_ENV" ]]; then
    set -a; source "$INFRA_ENV"; set +a
fi

# Source logging helper
if [[ -f "$SCRIPT_DIR/lib/logging.sh" ]]; then
    source "$SCRIPT_DIR/lib/logging.sh"
else
    log_info() { echo -e "[INFO] $*"; }
    log_success() { echo -e "\033[0;32m[PASS]\033[0m $*"; }
    log_warn() { echo -e "\033[1;33m[WARN]\033[0m $*"; }
    log_error() { echo -e "\033[0;31m[FAIL]\033[0m $*" >&2; }
fi

DEPLOY_ROOT="${DEPLOY_ROOT:-/opt/zafafworld.net}"
SERVICE_NAME="${SERVICE_NAME:-zafafworld.service}"
WAIT_SECONDS="${WAIT_SECONDS:-15}"

EXPECTED_SERVICES=(postgres pgbouncer redis backend client nginx vendor admin minio)

TOTAL=0
PASSED=0
FAILED=0
WARNED=0

pass()  { log_success "$1"; PASSED=$((PASSED+1)); TOTAL=$((TOTAL+1)); }
fail()  { log_error "$1"; FAILED=$((FAILED+1)); TOTAL=$((TOTAL+1)); }
warn()  { log_warn "$1"; WARNED=$((WARNED+1)); TOTAL=$((TOTAL+1)); }
info()  { log_info "$1"; }

log_info "══ ZafafWorld Post-Deployment Validation Suite ══"
echo

# ── 1. systemd Service Status ─────────────────────────────────────────────────
log_info "Checking systemd User Service Status..."
if systemctl --user is-active --quiet "$SERVICE_NAME" 2>/dev/null; then
    pass "systemd: $SERVICE_NAME is active (running)"
else
    SVC_STATE=$(systemctl --user is-active "$SERVICE_NAME" 2>/dev/null || echo "unknown")
    fail "systemd: $SERVICE_NAME is currently '$SVC_STATE' (expected: active)"
    echo
    log_warn "Last 20 journal lines:"
    journalctl --user -u "$SERVICE_NAME" -n 20 --no-pager 2>/dev/null | sed 's/^/    /' || true
fi

# Check for recent panics in journal
PANIC_COUNT=$(journalctl --user -u "$SERVICE_NAME" --since "2 minutes ago" --no-pager 2>/dev/null | grep -ciE "panic|FATAL|fatal error|thread 'main' panicked" || true)
if [[ "$PANIC_COUNT" -eq 0 ]]; then
    pass "journalctl: no panics or fatal errors in last 2 minutes"
else
    fail "journalctl: $PANIC_COUNT panic/fatal log line(s) detected!"
    journalctl --user -u "$SERVICE_NAME" --since "2 minutes ago" --no-pager 2>/dev/null | grep -iE "panic|FATAL|fatal error" | head -10 | sed 's/^/    /' || true
fi

# ── 2. Container Runtime Status ───────────────────────────────────────────────
log_info "Checking container engine runtime states (podman)..."
if [[ "$WAIT_SECONDS" -gt 0 ]]; then
    log_info "Waiting ${WAIT_SECONDS}s for container services to initialize..."
    sleep "$WAIT_SECONDS"
fi

podman ps --format "table {{.Names}}\t{{.Status}}\t{{.Image}}" 2>/dev/null || fail "podman: failed to list running containers"

for svc in "${EXPECTED_SERVICES[@]}"; do
    CONTAINER=$(podman ps --format '{{.Names}}' 2>/dev/null | grep -i "$svc" | head -1 || true)
    if [[ -n "$CONTAINER" ]]; then
        STATUS=$(podman inspect "$CONTAINER" --format '{{.State.Health.Status}}' 2>/dev/null || echo "N/A")
        if [[ "$STATUS" == "healthy" || "$STATUS" == "N/A" ]]; then
            pass "Container '$CONTAINER' running (health: $STATUS)"
        elif [[ "$STATUS" == "starting" ]]; then
            warn "Container '$CONTAINER' health: $STATUS (initializing...)"
        else
            fail "Container '$CONTAINER' health: $STATUS"
        fi
    else
        fail "Service '$svc': no running container found!"
    fi
done

# Ghost container checks
ALL_RUNNING=$(podman ps --format '{{.Names}}' 2>/dev/null)
GHOST_COUNT=0
while IFS= read -r name; do
    [[ -z "$name" ]] && continue
    KNOWN=false
    for svc in "${EXPECTED_SERVICES[@]}"; do
        if echo "$name" | grep -qi "$svc"; then
            KNOWN=true
            break
        fi
    done
    if [[ "$KNOWN" == "false" ]]; then
        warn "Ghost container running: '$name' (not in deployment profile)"
        GHOST_COUNT=$((GHOST_COUNT + 1))
    fi
done <<< "$ALL_RUNNING"
if [[ "$GHOST_COUNT" -eq 0 ]]; then
    pass "No ghost containers detected"
fi

# ── 3. UDS socket (backend ↔ nginx communication) ─────────────────────────────
log_info "Checking backend UDS socket..."
BACKEND_CONTAINER=$(podman ps --format '{{.Names}}' 2>/dev/null | grep -i "backend" | head -1 || true)
if [[ -n "$BACKEND_CONTAINER" ]] && podman exec "$BACKEND_CONTAINER" test -S /var/run/zafaf/zafaf.sock 2>/dev/null; then
    pass "backend container: /var/run/zafaf/zafaf.sock exists and is a valid socket"
else
    fail "backend container: /var/run/zafaf/zafaf.sock NOT found inside container"
fi

NGINX_CONTAINER=$(podman ps --format '{{.Names}}' 2>/dev/null | grep -i "nginx" | head -1 || true)
if [[ -n "$NGINX_CONTAINER" ]]; then
    if podman exec "$NGINX_CONTAINER" test -S /var/run/zafaf/zafaf.sock 2>/dev/null; then
        pass "nginx container: /var/run/zafaf/zafaf.sock socket visible"
    else
        warn "nginx container: /var/run/zafaf/zafaf.sock NOT visible (check SELinux labels)"
    fi
fi

# ── 4. Port Binding ───────────────────────────────────────────────────────────
log_info "Verifying network port bindings..."
check_port() {
    local port="$1"
    if ss -tlnp 2>/dev/null | grep -q ":${port} " || netstat -tlnp 2>/dev/null | grep -q ":${port} "; then
        pass "Port $port is bound and listening"
    else
        fail "Port $port is NOT bound (nginx/service may be offline)"
    fi
}
check_port 80
check_port 443

# ── 5. HTTP/HTTPS Connectivity ────────────────────────────────────────────────
log_info "Performing HTTP/HTTPS endpoint tests..."
http_check() {
    local url="$1"
    local label="${2:-$url}"
    local expected_codes="${3:-200 301 302}"

    local http_code
    if command -v curl &>/dev/null; then
        http_code=$(curl -sk --max-time 10 \
            --resolve zafafworld.net:80:127.0.0.1 \
            --resolve zafafworld.net:443:127.0.0.1 \
            --resolve api.zafafworld.net:443:127.0.0.1 \
            --resolve vendor.zafafworld.net:443:127.0.0.1 \
            --resolve admin.zafafworld.net:443:127.0.0.1 \
            -o /dev/null -w "%{http_code}" "$url" 2>/dev/null || echo "000")
    elif command -v wget &>/dev/null; then
        http_code=$(wget --server-response --spider -q "$url" 2>&1 | grep "HTTP/" | tail -1 | awk '{print $2}' || echo "000")
    else
        warn "$label: curl/wget not available (skipped)"
        return
    fi

    if echo "$expected_codes" | grep -qw "$http_code"; then
        pass "$label -> HTTP $http_code"
    elif [[ "$http_code" == "000" ]]; then
        fail "$label -> Connection refused (HTTP 000)"
    else
        warn "$label -> HTTP $http_code (unexpected status)"
    fi
}

http_check "http://127.0.0.1:80/" "HTTP Localhost Nginx (port 80)" "200 301 302 404"
http_check "http://zafafworld.net/healthz" "Nginx-to-client-web Connectivity" "200"
http_check "https://zafafworld.net:443/" "HTTPS Client Homepage" "200 301 302"
http_check "https://api.zafafworld.net:443/health" "HTTPS API Health Check" "200"

# ── 6. Nginx Config Syntax ────────────────────────────────────────────────────
log_info "Verifying Nginx configuration syntax..."
if [[ -n "$NGINX_CONTAINER" ]]; then
    if podman exec "$NGINX_CONTAINER" nginx -t 2>/dev/null; then
        pass "nginx: configuration syntax check passed"
    else
        fail "nginx: configuration syntax has errors"
        podman exec "$NGINX_CONTAINER" nginx -t 2>&1 | sed 's/^/    /' || true
    fi
fi

# ── Summary ───────────────────────────────────────────────────────────────────
echo
log_info "══ Validation Summary ══"
log_info "Total Checks: $TOTAL"
log_info "Passed      : $PASSED"
log_info "Warnings    : $WARNED"
log_info "Failed      : $FAILED"

if [[ "$FAILED" -gt 0 ]]; then
    log_error "Post-deployment checks FAILED with $FAILED critical failure(s)"
    exit 1
else
    log_success "Post-deployment validation passed successfully!"
fi
