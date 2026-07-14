#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════════════
# infra/harden-permissions.sh — ZafafWorld Zero-Trust Permission Hardening
#
# PURPOSE: Apply strict, least-privilege filesystem permissions to the entire
#          production deploy directory.
# ═══════════════════════════════════════════════════════════════════════════════
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Source config
if [[ -f "$SCRIPT_DIR/.env" ]]; then
    set -a; source "$SCRIPT_DIR/.env"; set +a
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
OWNER="noon:noon"

log_info "══ ZafafWorld Permission Hardening ══"
log_info "Deploy root : $DEPLOY_ROOT"
log_info "Target owner: $OWNER"
echo

if [[ ! -d "$DEPLOY_ROOT" ]]; then
    log_error "DEPLOY_ROOT not found: $DEPLOY_ROOT"
    exit 1
fi

EXCLUDE_DIRS=(
    "*/node_modules"
    "*/.git"
    "*/build"
    "*/.svelte-kit"
    "*/.bun"
    "*/target"
    "*/cms-wordpress/wp-content"
    "*/infra/postgres/data"
)

PRUNE_EXPR=()
for excl in "${EXCLUDE_DIRS[@]}"; do
    PRUNE_EXPR+=(-path "$excl" -prune -o)
done

# Step 1: Ownership (skip excluded subtrees)
log_info "Setting owner $OWNER (skipping container volumes)..."
find "$DEPLOY_ROOT" "${PRUNE_EXPR[@]}" -exec chown "$OWNER" {} + 2>/dev/null || {
    log_warn "chown partial failure (may need sudo for some files). Continuing..."
}
log_success "Ownership applied: $OWNER"

# Step 2: Base permissions — 755 dirs, 644 files
log_info "Applying base permissions (755 dirs / 644 files)..."

# Directories → 755 (skip excluded subtrees)
find "$DEPLOY_ROOT" "${PRUNE_EXPR[@]}" -type d -exec chmod 755 {} + 2>/dev/null || true
log_success "Directories set to 755 (rwxr-xr-x)"

# Files → 644 (skip excluded subtrees)
find "$DEPLOY_ROOT" "${PRUNE_EXPR[@]}" -type f -exec chmod 644 {} + 2>/dev/null || true
log_success "Files set to 644 (rw-r--r--)"

# Step 3: Secret files — 600
log_info "Securing secret files (600 - owner read/write)..."
if [[ -f "${DEPLOY_ROOT}/.env" ]]; then
    chmod 600 "${DEPLOY_ROOT}/.env"
    log_success ".env set to 600"
fi
if [[ -f "${DEPLOY_ROOT}/infra/.env" ]]; then
    chmod 600 "${DEPLOY_ROOT}/infra/.env"
    log_success "infra/.env set to 600"
fi

SSL_DIR="${DEPLOY_ROOT}/infra/nginx/ssl"
if [[ -d "$SSL_DIR" ]]; then
    find "$SSL_DIR" -name "*.key" -exec chmod 600 {} + 2>/dev/null || true
    find "$SSL_DIR" -name "*.pem" -exec chmod 600 {} + 2>/dev/null || true
fi

# Step 4: Executable scripts — 755
log_info "Marking shell scripts as executable (755)..."
find "$DEPLOY_ROOT" "${PRUNE_EXPR[@]}" -name "*.sh" -exec chmod 755 {} + 2>/dev/null || true

# Step 5: Container entrypoints / Containerfiles
log_info "Configuring container files permissions..."
find "$DEPLOY_ROOT" "${PRUNE_EXPR[@]}" -name "Containerfile" -exec chmod 644 {} + 2>/dev/null || true
find "$DEPLOY_ROOT" "${PRUNE_EXPR[@]}" -name "entrypoint.sh" -exec chmod 755 {} + 2>/dev/null || true

# Step 6: Log directory permissions
log_info "Applying permissions to /var/log/zafaf..."
if [[ -d "/var/log/zafaf" ]]; then
    chown -R "$OWNER" /var/log/zafaf 2>/dev/null || true
    find /var/log/zafaf -type d -exec chmod 755 {} + 2>/dev/null || true
    find /var/log/zafaf -type f -exec chmod 644 {} + 2>/dev/null || true
    log_success "/var/log/zafaf configured successfully"
fi

# Step 6b: Shared uploads volume permissions
log_info "Checking uploads volume permissions..."
if [[ -f "${DEPLOY_ROOT}/infra/init-storage.sh" ]]; then
    bash "${DEPLOY_ROOT}/infra/init-storage.sh"
fi

# Step 7: Verification audit
log_info "Verifying permissions audit..."
AUDIT_FAILED=0

check_perm() {
    local file="$1"
    local expected_mode="$2"
    local label="${3:-$file}"

    if [[ ! -e "$file" ]]; then
        log_warn "$label not found (skipping verification)"
        return
    fi

    local actual_mode
    actual_mode=$(stat -c "%a" "$file" 2>/dev/null || stat -f "%OLp" "$file" 2>/dev/null)

    if [[ "$actual_mode" == "$expected_mode" ]]; then
        log_success "$label permissions verify OK: $actual_mode"
    else
        log_error "$label permissions mismatch: expected $expected_mode, got $actual_mode"
        AUDIT_FAILED=$((AUDIT_FAILED + 1))
    fi
}

check_perm "${DEPLOY_ROOT}"                         "755" "/opt/zafafworld (root dir)"
check_perm "${DEPLOY_ROOT}/.env"                    "600" ".env"
check_perm "${DEPLOY_ROOT}/infra/.env"              "600" "infra/.env"
check_perm "${DEPLOY_ROOT}/infra/zafafworld.service" "644" "infra/zafafworld.service"
check_perm "${DEPLOY_ROOT}/infra/setup-systemd.sh"  "755" "infra/setup-systemd.sh"
check_perm "${DEPLOY_ROOT}/infra/zero-trust-cleanup.sh"     "755" "infra/zero-trust-cleanup.sh"
check_perm "${DEPLOY_ROOT}/infra/harden-permissions.sh"   "755" "infra/harden-permissions.sh"
check_perm "${DEPLOY_ROOT}/infra/validate-deploy.sh"   "755" "infra/validate-deploy.sh"
check_perm "${DEPLOY_ROOT}/infra/verify-env.sh"     "755" "infra/verify-env.sh"
check_perm "${DEPLOY_ROOT}/podman-compose.yml"       "644" "podman-compose.yml"

if [[ "$AUDIT_FAILED" -gt 0 ]]; then
    log_error "Permission hardening audit FAILED with $AUDIT_FAILED error(s)"
    exit 1
else
    log_success "All permissions audits passed successfully."
fi
