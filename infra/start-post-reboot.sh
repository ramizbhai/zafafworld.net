#!/usr/bin/env bash
# ══════════════════════════════════════════════════════════════════════════════
# infra/start-post-reboot.sh — Run ONCE after every system reboot
#
# PURPOSE: Re-initialize and boot up the production stack after server reboot.
# ══════════════════════════════════════════════════════════════════════════════
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

SERVICE_NAME="${SERVICE_NAME:-zafafworld.service}"

log_info "══ ZafafWorld Post-Reboot Startup Helper ══"
echo

# Step 1: Linger check
if loginctl show-user noon 2>/dev/null | grep -q "Linger=yes"; then
    log_success "loginctl user linger is enabled (auto-start on boot OK)"
else
    log_warn "loginctl user linger is NOT enabled. Enabling linger for 'noon'..."
    loginctl enable-linger noon 2>/dev/null && log_success "Linger enabled successfully" || log_warn "Could not set linger"
fi

# Step 2: Clean Podman transient state
log_info "Cleaning stale container runtimes..."
podman stop --all --time 3 2>/dev/null | tail -3 || true
podman rm --all --force 2>/dev/null | tail -3 || true
pkill -9 -f "catatonit" 2>/dev/null && log_info "Killed legacy catatonit helpers" || true
podman system migrate 2>&1 | tail -1
log_success "Container runtime state cleaned."

# Step 3: Enable and start service
log_info "Starting systemd user service: $SERVICE_NAME..."
systemctl --user enable "$SERVICE_NAME" 2>/dev/null || true
systemctl --user reset-failed 2>/dev/null || true
if systemctl --user start "$SERVICE_NAME"; then
    log_success "Service start issued successfully."
else
    log_error "Service start failed! Check: journalctl --user -u $SERVICE_NAME"
    exit 1
fi

# Step 4: Validate deployment
log_info "Waiting 60 seconds for service containers to warm up..."
sleep 60

echo
log_info "══ Container Status ══"
podman ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}" 2>/dev/null

echo
if systemctl --user is-active "$SERVICE_NAME" &>/dev/null; then
    log_success "zafafworld.service is active and healthy."
else
    log_error "zafafworld.service is inactive - check logs!"
    exit 1
fi
