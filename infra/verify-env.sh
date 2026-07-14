#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════════════
# verify-env.sh — Production Environment Variable Verification
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
ENV_FILE="${DEPLOY_ROOT}/.env"

TOTAL=0
PASSED=0
FAILED=0
WARNED=0

PROJECT_PREFIX=""
if [[ "${1:-}" == "--project" ]]; then
    PROJECT_PREFIX="${2:-}"
fi

if [[ -f "$ENV_FILE" ]]; then
    EXPECTED_POSTGRES_USER=$(grep -E '^POSTGRES_USER=' "$ENV_FILE" | cut -d'=' -f2- | tr -d '"' | head -1)
else
    EXPECTED_POSTGRES_USER="postgres"
fi
EXPECTED_POSTGRES_USER="${EXPECTED_POSTGRES_USER:-postgres}"

find_container() {
    local service="$1"
    local container
    container=$(podman ps --format '{{.Names}}' 2>/dev/null | grep -i "${service}" | head -1)
    echo "$container"
}

check_env() {
    local container="$1"
    local var_name="$2"
    local expected="${3:-}"
    local sensitive="${4:-false}"

    TOTAL=$((TOTAL + 1))

    local actual
    actual=$(podman exec "$container" sh -c "echo \"\${${var_name}:-__UNSET__}\"" 2>/dev/null) || {
        log_error "Container '$container': exec failed for variable '$var_name'"
        FAILED=$((FAILED + 1))
        return
    }

    if [[ "$actual" == "__UNSET__" || -z "$actual" ]]; then
        log_error "Container '$container': variable '$var_name' is UNSET or EMPTY"
        FAILED=$((FAILED + 1))
        return
    fi

    if [[ -n "$expected" ]]; then
        if [[ "$actual" == "$expected" ]]; then
            if [[ "$sensitive" == "true" ]]; then
                log_success "  $var_name matches expected [redacted]"
            else
                log_success "  $var_name matches expected: $actual"
            fi
            PASSED=$((PASSED + 1))
        else
            if [[ "$sensitive" == "true" ]]; then
                log_error "  $var_name mismatch! expected [redacted], got [redacted]"
            else
                log_error "  $var_name mismatch! expected '$expected', got '$actual'"
            fi
            FAILED=$((FAILED + 1))
        fi
        return
    fi

    if [[ "$sensitive" == "true" ]]; then
        log_success "  $var_name is set [redacted]"
    else
        log_success "  $var_name is set: $actual"
    fi
    PASSED=$((PASSED + 1))
}

check_env_warn() {
    local container="$1"
    local var_name="$2"

    TOTAL=$((TOTAL + 1))

    local actual
    actual=$(podman exec "$container" sh -c "echo \"\${${var_name}:-__UNSET__}\"" 2>/dev/null) || {
        log_warn "  $var_name: container exec failed"
        WARNED=$((WARNED + 1))
        return
    }

    if [[ "$actual" == "__UNSET__" || -z "$actual" ]]; then
        log_warn "  $var_name: not set (optional)"
        WARNED=$((WARNED + 1))
    else
        log_success "  $var_name is set: $actual"
        PASSED=$((PASSED + 1))
    fi
}

log_info "══ ZafafWorld Container Environment Checks ══"
echo

log_info "Discovering running containers..."
podman ps --format 'table {{.Names}}\t{{.Status}}\t{{.Image}}' 2>/dev/null || {
    log_error "podman commands are not executing or daemon is offline."
    exit 1
}
echo

BACKEND=$(find_container "backend")
CLIENT_WEB=$(find_container "client-web")
VENDOR_PORTAL=$(find_container "vendor-portal")
ADMIN_PANEL=$(find_container "admin-panel")
NGINX=$(find_container "nginx")
POSTGRES=$(find_container "postgres" | grep -v "pgbouncer" | head -1)
PGBOUNCER=$(find_container "pgbouncer")
REDIS=$(find_container "redis")

log_info "Resolved container target names:"
log_info "  backend       = ${BACKEND:-NOT FOUND}"
log_info "  client-web    = ${CLIENT_WEB:-NOT FOUND}"
log_info "  vendor-portal = ${VENDOR_PORTAL:-NOT FOUND}"
log_info "  admin-panel   = ${ADMIN_PANEL:-NOT FOUND}"
log_info "  nginx         = ${NGINX:-NOT FOUND}"
log_info "  postgres      = ${POSTGRES:-NOT FOUND}"
log_info "  pgbouncer     = ${PGBOUNCER:-NOT FOUND}"
log_info "  redis         = ${REDIS:-NOT FOUND}"
echo

# 1. BACKEND
if [[ -n "$BACKEND" ]]; then
    log_info "Checking BACKEND container: $BACKEND"
    check_env "$BACKEND" "APP_ENVIRONMENT"    "production"
    check_env "$BACKEND" "BIND_MODE"          "uds"
    check_env "$BACKEND" "DATABASE_URL"       ""           "true"
    check_env "$BACKEND" "JWT_SECRET"         ""           "true"
    check_env "$BACKEND" "CORS_ORIGINS"       "https://zafafworld.net,https://vendor.zafafworld.net,https://admin.zafafworld.net"
    check_env "$BACKEND" "FRONTEND_URL"       "https://zafafworld.net"
    check_env "$BACKEND" "RUST_LOG"           "info"
    check_env "$BACKEND" "RUST_LOG_FORMAT"    ""
    check_env "$BACKEND" "PORT"               "8080"
    check_env "$BACKEND" "SMTP_HOST"          ""
    check_env "$BACKEND" "SMTP_PORT"          ""
    check_env "$BACKEND" "SMTP_USERNAME"      ""           "true"
    check_env "$BACKEND" "SMTP_PASSWORD"      ""           "true"
    check_env "$BACKEND" "SMTP_FROM_EMAIL"    "noreply@zafafworld.net"
    check_env "$BACKEND" "SMTP_FROM_NAME"     ""
    check_env_warn "$BACKEND" "SMTP_TLS_MODE"
    check_env_warn "$BACKEND" "REDIS_URL"

    TOTAL=$((TOTAL + 1))
    if podman exec "$BACKEND" test -S /var/run/zafaf/zafaf.sock 2>/dev/null; then
        log_success "  UDS socket exists inside backend"
        PASSED=$((PASSED + 1))
    else
        log_error "  UDS socket /var/run/zafaf/zafaf.sock NOT FOUND inside backend"
        FAILED=$((FAILED + 1))
    fi
    echo
fi

# 2. CLIENT-WEB
if [[ -n "$CLIENT_WEB" ]]; then
    log_info "Checking CLIENT-WEB container: $CLIENT_WEB"
    check_env "$CLIENT_WEB" "PUBLIC_API_URL"    "https://api.zafafworld.net"
    check_env "$CLIENT_WEB" "PUBLIC_WS_URL"     "wss://api.zafafworld.net"
    check_env "$CLIENT_WEB" "ORIGIN"            "https://zafafworld.net"
    check_env "$CLIENT_WEB" "PROTOCOL_HEADER"   "x-forwarded-proto"
    check_env_warn "$CLIENT_WEB" "REDIS_URL"
    check_env_warn "$CLIENT_WEB" "NODE_EXTRA_CA_CERTS"
    echo
fi

# 3. VENDOR-PORTAL
if [[ -n "$VENDOR_PORTAL" ]]; then
    log_info "Checking VENDOR-PORTAL container: $VENDOR_PORTAL"
    check_env "$VENDOR_PORTAL" "PUBLIC_API_URL"    "https://api.zafafworld.net"
    check_env "$VENDOR_PORTAL" "PUBLIC_WS_URL"     "wss://api.zafafworld.net"
    check_env "$VENDOR_PORTAL" "ORIGIN"            "https://vendor.zafafworld.net"
    check_env "$VENDOR_PORTAL" "PROTOCOL_HEADER"   "x-forwarded-proto"
    check_env_warn "$VENDOR_PORTAL" "REDIS_URL"
    check_env_warn "$VENDOR_PORTAL" "NODE_EXTRA_CA_CERTS"
    echo
fi

# 4. ADMIN-PANEL
if [[ -n "$ADMIN_PANEL" ]]; then
    log_info "Checking ADMIN-PANEL container: $ADMIN_PANEL"
    check_env "$ADMIN_PANEL" "PUBLIC_API_URL"    "https://api.zafafworld.net"
    check_env "$ADMIN_PANEL" "ORIGIN"            "https://admin.zafafworld.net"
    check_env "$ADMIN_PANEL" "PROTOCOL_HEADER"   "x-forwarded-proto"
    check_env_warn "$ADMIN_PANEL" "REDIS_URL"
    check_env_warn "$ADMIN_PANEL" "NODE_EXTRA_CA_CERTS"
    echo
fi

# 5. NGINX
if [[ -n "$NGINX" ]]; then
    log_info "Checking NGINX container: $NGINX"
    TOTAL=$((TOTAL + 1))
    if podman exec "$NGINX" nginx -t 2>/dev/null; then
        log_success "  nginx syntax verification passed"
        PASSED=$((PASSED + 1))
    else
        log_error "  nginx -t config validation failed"
        FAILED=$((FAILED + 1))
    fi

    TOTAL=$((TOTAL + 1))
    if podman exec "$NGINX" test -S /var/run/zafaf/zafaf.sock 2>/dev/null; then
        log_success "  UDS socket visible from nginx container"
        PASSED=$((PASSED + 1))
    else
        log_error "  UDS socket /var/run/zafaf/zafaf.sock NOT visible from nginx"
        FAILED=$((FAILED + 1))
    fi
    echo
fi

# 6. POSTGRES
if [[ -n "$POSTGRES" ]]; then
    log_info "Checking POSTGRES container: $POSTGRES"
    check_env "$POSTGRES" "POSTGRES_DB"       "zafaf_world"
    check_env "$POSTGRES" "POSTGRES_USER"     "$EXPECTED_POSTGRES_USER"
    check_env "$POSTGRES" "POSTGRES_PASSWORD" ""          "true"
    echo
fi

# 7. PGBOUNCER
if [[ -n "$PGBOUNCER" ]]; then
    log_info "Checking PGBOUNCER container: $PGBOUNCER"
    check_env "$PGBOUNCER" "DB_HOST"           "postgres"
    check_env "$PGBOUNCER" "DB_USER"           "$EXPECTED_POSTGRES_USER"
    check_env "$PGBOUNCER" "DB_PASSWORD"       ""          "true"
    check_env "$PGBOUNCER" "DB_NAME"           "zafaf_world"
    check_env "$PGBOUNCER" "POOL_MODE"         "transaction"
    check_env "$PGBOUNCER" "MAX_CLIENT_CONN"   "1000"
    check_env "$PGBOUNCER" "DEFAULT_POOL_SIZE" "55"
    echo
fi

echo
log_info "══ Verification Summary ══"
log_info "Total checks : $TOTAL"
log_info "Passed       : $PASSED"
log_info "Warnings     : $WARNED"
log_info "Failed       : $FAILED"

if [[ "$FAILED" -gt 0 ]]; then
    log_error "Environment verification audit FAILED with $FAILED error(s)"
    exit 1
else
    log_success "Environment verification audit passed successfully."
    exit 0
fi
