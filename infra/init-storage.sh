#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════════════
# infra/init-storage.sh — Initialize and fix permissions for shared media uploads volume
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

VOLUME_DIR="${UPLOADS_VOLUME_DIR:-/var/lib/zafafworld/uploads}"

log_info "Initializing storage permissions on: $VOLUME_DIR"

# Ensure the volume directory exists
mkdir -p "$VOLUME_DIR"

# Fix ownership: owner=10001 (backend), group=33 (www-data)
podman unshare chown -R 10001:33 "$VOLUME_DIR"

# Set directory permissions to 2775 (SGID enabled, group writable)
podman unshare find "$VOLUME_DIR" -type d -exec chmod 2775 {} +

# Set file permissions to 664 (group writable)
podman unshare find "$VOLUME_DIR" -type f -exec chmod 664 {} +

log_success "Storage volume permissions initialized successfully."
