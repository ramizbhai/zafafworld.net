#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════════════
# infra/reset-admin-password.sh — Helper script to reset admin user password.
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

PROJECT_ROOT="${DEPLOY_ROOT:-/opt/zafafworld.net}"

if [[ -f "${PROJECT_ROOT}/.env" ]]; then
    set -a; source "${PROJECT_ROOT}/.env"; set +a
fi

DEFAULT_EMAIL="admin@zafafworld.net"
DEFAULT_PASSWORD="${ADMIN_INITIAL_PASSWORD:-Zafafworld@2026}"

if [[ "$#" -eq 2 ]]; then
    ADMIN_EMAIL="$1"
    NEW_PASSWORD="$2"
elif [[ "$#" -eq 0 ]]; then
    read -rp "Enter Admin Email [default: ${DEFAULT_EMAIL}]: " INPUT_EMAIL
    ADMIN_EMAIL="${INPUT_EMAIL:-${DEFAULT_EMAIL}}"
    
    read -rp "Enter New Password [default: ${DEFAULT_PASSWORD}]: " INPUT_PASSWORD
    NEW_PASSWORD="${INPUT_PASSWORD:-${DEFAULT_PASSWORD}}"
else
    log_error "Usage: $0"
    log_info "   or: $0 <admin-email> <new-password>"
    exit 1
fi

log_info "Resetting password for admin user: ${ADMIN_EMAIL}..."

if podman ps --filter "name=zafafworld_backend_1" --filter "status=running" --format "{{.Names}}" | grep -q "zafafworld_backend_1"; then
    log_info "Executing reset inside active 'zafafworld_backend_1' container..."
    podman exec -i zafafworld_backend_1 /usr/local/bin/backend-rust reset-admin-password "${ADMIN_EMAIL}" "${NEW_PASSWORD}"
    log_success "Password reset issued successfully."
else
    log_warn "Active backend container not found. Executing locally via cargo run..."
    cd "${PROJECT_ROOT}/backend-rust"
    cargo run -r -- reset-admin-password "${ADMIN_EMAIL}" "${NEW_PASSWORD}"
fi
