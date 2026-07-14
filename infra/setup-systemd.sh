#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════════════
# setup-systemd.sh — Convert ZafafWorld podman-compose to a systemd user service
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
UNIT_NAME="${SERVICE_NAME:-zafafworld.service}"
UNIT_SOURCE="${SCRIPT_DIR}/${UNIT_NAME}"
UNIT_DIR="${HOME}/.config/systemd/user"
UNIT_DEST="${UNIT_DIR}/${UNIT_NAME}"

log_info "══ ZafafWorld systemd User Service Setup ══"
echo

# ── Pre-flight checks ───────────────────────────────────────────────────────
log_info "Running systemd environment pre-flight checks..."

if [[ "$(id -u)" -eq 0 ]]; then
    log_error "Do NOT run this as root. Run as user 'noon'."
    exit 1
fi

if ! command -v podman-compose &>/dev/null; then
    log_error "podman-compose not found in PATH."
    exit 1
fi

if [[ ! -f "$UNIT_SOURCE" ]]; then
    log_error "Unit file template not found at $UNIT_SOURCE"
    exit 1
fi

if [[ ! -d "${DEPLOY_ROOT}" ]]; then
    log_error "Production deploy directory not found: ${DEPLOY_ROOT}"
    exit 1
fi

if [[ ! -f "${DEPLOY_ROOT}/.env" ]]; then
    log_error ".env not found at ${DEPLOY_ROOT}/.env"
    exit 1
fi

COMPOSE_PATH=$(command -v podman-compose)
log_info "podman-compose: $COMPOSE_PATH"
log_info "Deploy root   : $DEPLOY_ROOT"
log_info "Unit source   : $UNIT_SOURCE"
log_info "Unit dest     : $UNIT_DEST"
echo

# ── Step 1: Create user systemd directory ────────────────────────────────────
log_info "[1/5] Creating user systemd configurations directory..."
mkdir -p "${UNIT_DIR}"
log_success "Created folder: ${UNIT_DIR}"

# ── Step 2: Copy unit file ───────────────────────────────────────────────────
log_info "[2/5] Installing systemd unit file..."
sed -E \
    -e "s|/home/noon/\.local/bin/podman-compose|${COMPOSE_PATH}|g" \
    -e "s|/usr/(local/)?bin/podman-compose|${COMPOSE_PATH}|g" \
    "${UNIT_SOURCE}" > "${UNIT_DEST}"
log_success "Unit installed to ${UNIT_DEST}"

# ── Step 2b: Create log directories ──────────────────────────────────────────
log_info "[2b/5] Creating nginx log directories..."
mkdir -p "${DEPLOY_ROOT}/infra/nginx/logs"
log_success "Logs directory ready: ${DEPLOY_ROOT}/infra/nginx/logs"

# ── Step 2c: Install logrotate configs ───────────────────────────────────────
log_info "[2c/5] Installing logrotate rules configurations..."
if [[ -w /etc/logrotate.d ]]; then
    cp "${DEPLOY_ROOT}/infra/logrotate/nginx"          /etc/logrotate.d/zafaf-nginx
    cp "${DEPLOY_ROOT}/infra/logrotate/app-journal"    /etc/logrotate.d/zafaf-app
    chmod 644 /etc/logrotate.d/zafaf-nginx /etc/logrotate.d/zafaf-app
    log_success "System logs logrotate rules installed in /etc/logrotate.d"
else
    log_warn "/etc/logrotate.d not writable (needs sudo/root privileges)."
    log_info "To install manually: sudo cp ${DEPLOY_ROOT}/infra/logrotate/nginx /etc/logrotate.d/zafaf-nginx"
fi

# ── Step 2d: Install journald retention override ──────────────────────────────
log_info "[2d/5] Installing journald log retention overrides..."
if [[ -w /etc/systemd/journald.conf.d ]] || sudo -n mkdir -p /etc/systemd/journald.conf.d 2>/dev/null; then
    sudo -n cp "${DEPLOY_ROOT}/infra/logrotate/journald-zafaf.conf" \
        /etc/systemd/journald.conf.d/zafaf.conf 2>/dev/null && \
    log_success "journald logs retention limits configured (500MB max)" || \
    log_warn "journald log config could not be copied automatically."
fi

# ── Step 3: Enable lingering ─────────────────────────────────────────────────
log_info "[3/5] Checking linger activation status..."
if loginctl show-user "$(whoami)" 2>/dev/null | grep -q "Linger=yes"; then
    log_success "Linger already active."
else
    log_info "Enabling linger for user '$(whoami)'..."
    loginctl enable-linger "$(whoami)"
    log_success "Linger enabled."
fi

# ── Step 4: Reload systemd user daemon ────────────────────────────────────────
log_info "[4/5] Reloading systemd user daemon bus..."
systemctl --user daemon-reload
log_success "Daemon reloaded."

# ── Step 5: Enable service ───────────────────────────────────────────────────
log_info "[5/5] Enabling $UNIT_NAME auto-boot timer..."
systemctl --user enable "${UNIT_NAME}"
log_success "Service enabled successfully."
echo
log_success "Systemd configuration setup complete!"
