#!/usr/bin/env bash
# ══════════════════════════════════════════════════════════════════════════════
# infra/deploy.sh — ZafafWorld Production Deploy & Auto-Heal Script
#
# USAGE:  bash infra/deploy.sh [--skip-build] [--skip-audit] [--force-heal]
# ══════════════════════════════════════════════════════════════════════════════
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Source logging helpers
if [[ -f "$SCRIPT_DIR/lib/logging.sh" ]]; then
    source "$SCRIPT_DIR/lib/logging.sh"
else
    log_info() { echo -e "[INFO] $*"; }
    log_success() { echo -e "\033[0;32m[PASS]\033[0m $*"; }
    log_warn() { echo -e "\033[1;33m[WARN]\033[0m $*"; }
    log_error() { echo -e "\033[0;31m[FAIL]\033[0m $*" >&2; }
    load_env() {
        if [[ -f "$1" ]]; then
            while IFS= read -r line || [[ -n "$line" ]]; do
                line=$(echo "$line" | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//')
                [[ -z "$line" || "$line" =~ ^# ]] && continue
                if [[ "$line" =~ ^([^=]+)=(.*)$ ]]; then
                    local key="${BASH_REMATCH[1]}"
                    local val="${BASH_REMATCH[2]}"
                    key=$(echo "$key" | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//')
                    val=$(echo "$val" | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//')
                    val="${val#\"}"; val="${val%\"}"; val="${val#\'}"; val="${val%\'}"
                    export "$key"="$val"
                fi
            done < "$1"
        fi
    }
fi

# Load configurations safely
INFRA_ENV="$SCRIPT_DIR/.env"
load_env "$INFRA_ENV"

DEPLOY_ROOT="${DEPLOY_ROOT:-/opt/zafafworld.net}"
COMPOSE_FILE="$DEPLOY_ROOT/podman-compose.yml"
ENV_FILE="$DEPLOY_ROOT/.env"

load_env "$ENV_FILE"

PGDATABASE="${POSTGRES_DB:-zafaf_world}"

SKIP_BUILD=false
SKIP_AUDIT=false
FORCE_HEAL=false

for arg in "$@"; do
    case "$arg" in
        --skip-build) SKIP_BUILD=true ;;
        --skip-audit) SKIP_AUDIT=true ;;
        --force-heal) FORCE_HEAL=true ;;
        *) log_error "Unknown argument: $arg"; exit 1 ;;
    esac
done

log_info "══ Starting ZafafWorld Deployment ══"
log_info "Deploy root : $DEPLOY_ROOT"
log_info "Skip build  : $SKIP_BUILD"
log_info "Skip audit  : $SKIP_AUDIT"
log_info "Force heal  : $FORCE_HEAL"
echo

# ── Step 0: Aggressive Force Heal Recovery ────────────────────────────────────
if [[ "$FORCE_HEAL" == "true" ]]; then
    log_warn "Executing aggressive AUTO-HEAL recovery sequence..."
    
    # 1. Port binding check
    log_info "Unfreezing unprivileged port start range..."
    sudo sysctl -w net.ipv4.ip_unprivileged_port_start=1 || log_warn "Failed to set sysctl net.ipv4.ip_unprivileged_port_start"
    
    # 2. Firewall configuration
    log_info "Opening network firewall ports (80/tcp, 443/tcp)..."
    sudo ufw allow 80/tcp >/dev/null 2>&1 || true
    sudo ufw allow 443/tcp >/dev/null 2>&1 || true
    sudo ufw reload >/dev/null || true
    
    # 3. Pull new code if git directory is active
    if [[ -d "$DEPLOY_ROOT/.git" ]]; then
        log_info "Resetting local git repository tracking..."
        cd "$DEPLOY_ROOT"
        export GIT_TERMINAL_PROMPT=0
        git fetch --all || log_warn "Git fetch failed, skipping remote updates"
        git reset --hard origin/main || log_warn "Git reset failed, using current checkout"
        git pull origin main || log_warn "Git pull failed, using current checkout"
    fi
    
    # 4. Correct local ownerships
    log_info "Hardening directory ownerships..."
    sudo chown -R noon:noon "$DEPLOY_ROOT" || log_warn "Failed to chown $DEPLOY_ROOT"
    
    # 5. Purge stuck container engine resources
    log_info "Stopping systemd user service..."
    systemctl --user stop zafafworld.service || true
    
    log_info "Terminating dangling container runtime helper processes..."
    pkill -u noon -9 -f "podman|conmon|slirp4netns|fuse-overlayfs|buildah" || true
    
    log_info "Migrating unprivileged user namespace mapping configurations..."
    podman system migrate || true
    
    log_info "Evicting all user containers forcefully..."
    podman rm --force --all || true
    
    log_info "Bringing down Compose stack resources..."
    podman-compose -f "$COMPOSE_FILE" --env-file "$ENV_FILE" -p zafafworld down || true
    
    log_info "Evicting dangling bridge networks..."
    podman network rm zafafworld_zafaf_network zafafworldnet_zafaf_network 2>/dev/null || true
    podman network prune --force || true
    
    log_info "Purging stale UDS socket bindings..."
    sudo rm -rf /var/run/zafaf/zafaf.sock || true
    
    log_success "Auto-heal recovery phase complete."
fi

# ── Step 1: Pre-deploy Audit ──────────────────────────────────────────────────
if [[ "$SKIP_AUDIT" == "false" ]]; then
    log_info "[1/4] Running pre-deployment audits..."
    if bash "$SCRIPT_DIR/audit-infra.sh"; then
        log_success "Pre-deployment audit checks passed"
    else
        log_error "Pre-deployment audit FAILED — resolve failures before deploying"
        exit 1
    fi
else
    log_warn "[1/4] Audit checks skipped (--skip-audit)"
fi

# ── Step 1.5: Pre-deploy backup gate ──────────────────────────────────────────
log_info "[1.5/4] Executing database backup security gate..."
export TIMESTAMP="$(date +%Y%m%d_%H%M%S)"
if bash "$SCRIPT_DIR/backup/backup-db.sh" --execute; then
    log_success "Pre-deployment database backup generated successfully"
else
    # Check if postgres container is offline (allow for recovery deployments)
    if ! podman ps --filter "name=postgres" --filter "status=running" -q | grep -q . ; then
        log_warn "PostgreSQL container is offline. Skipping backup verification gate for recovery..."
    else
        log_error "Database backup gate failed — aborting deploy to prevent data loss"
        exit 1
    fi
fi

# ── Step 1.6: Record Pre-Deployment State Metadata ─────────────────────────────
log_info "[1.6/4] Recording pre-deployment state metadata..."
PREV_COMMIT=$(git rev-parse HEAD 2>/dev/null || echo "none")
PREV_BACKEND_IMG=$(podman image inspect localhost/zafaf-backend:latest --format '{{.Id}}' 2>/dev/null || echo "none")
PREV_CLIENT_IMG=$(podman image inspect localhost/zafaf-client-web:latest --format '{{.Id}}' 2>/dev/null || echo "none")
PREV_VENDOR_IMG=$(podman image inspect localhost/zafaf-vendor-portal:latest --format '{{.Id}}' 2>/dev/null || echo "none")
PREV_ADMIN_IMG=$(podman image inspect localhost/zafaf-admin-panel:latest --format '{{.Id}}' 2>/dev/null || echo "none")
PREV_NGINX_IMG=$(podman image inspect localhost/zafaf-nginx:latest --format '{{.Id}}' 2>/dev/null || echo "none")
BACKUP_FILE="${BACKUP_DIR}/zafaf_${PGDATABASE}_${TIMESTAMP}.sql.gz"

cat <<EOF > "$SCRIPT_DIR/last_deploy_state.json"
{
  "timestamp": "$(date -Iseconds)",
  "git_commit": "$PREV_COMMIT",
  "backup_filename": "$BACKUP_FILE",
  "image_ids": {
    "zafaf-backend": "$PREV_BACKEND_IMG",
    "zafaf-client-web": "$PREV_CLIENT_IMG",
    "zafaf-vendor-portal": "$PREV_VENDOR_IMG",
    "zafaf-admin-panel": "$PREV_ADMIN_IMG",
    "zafaf-nginx": "$PREV_NGINX_IMG"
  }
}
EOF
log_success "Deployment state cataloged in last_deploy_state.json"

# ── Step 2: Build images ───────────────────────────────────────────────────────
if [[ "$SKIP_BUILD" == "false" ]]; then
    log_info "[2/4] Rebuilding application container images..."
    if podman-compose -f "$COMPOSE_FILE" --env-file "$ENV_FILE" -p zafafworld build; then
        log_success "Container images compiled successfully"
    else
        log_error "Container image build FAILED"
        exit 1
    fi
else
    log_warn "[2/4] Image build skipped (--skip-build) — utilizing cached images"
fi

# ── Step 2.5: Clean stale Podman container state ──────────────────────────────
log_info "[2.5/4] Purging obsolete container states..."
podman-compose -f "$COMPOSE_FILE" --env-file "$ENV_FILE" -p zafafworld down --remove-orphans &>/dev/null || true
zafaf_leftovers=$(podman ps -a --filter "name=zafafworld" --filter "name=zafaf-cms" --format "{{.Names}}" 2>/dev/null)
if [[ -n "$zafaf_leftovers" ]]; then
    log_warn "Evicting legacy leftover containers: $zafaf_leftovers"
    podman rm -f $zafaf_leftovers &>/dev/null || true
fi

if podman system migrate 2>&1 | tail -1; then
    log_success "podman system namespaces synced"
else
    log_warn "podman system namespaces sync returned warning state"
fi

log_info "[2.8/4] Refreshing systemd unit layouts..."
echo "n" | bash "$SCRIPT_DIR/setup-systemd.sh"

log_info "[3/4] Triggering service restart sequence..."
systemctl --user reset-failed zafafworld.service 2>/dev/null || true
systemctl --user restart zafafworld.service
log_success "Service reload request issued to systemd session bus"

# ── Step 4: Wait and verify health via Validation Gate ────────────────────────
log_info "[4/4] Executing health checks validation gate..."
export WAIT_SECONDS=45
if bash "$SCRIPT_DIR/validate-deploy.sh"; then
    log_success "Deployment health checks PASSED — System is ONLINE"
else
    log_error "Deployment health checks FAILED — check journalctl logs"
    exit 1
fi
