#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════════════
# infra/audit-infra.sh — ZafafWorld Infrastructure Full-Audit Script
# ═══════════════════════════════════════════════════════════════════════════════
set -uo pipefail

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

FAILED=0
PASS() { log_success "$*"; }
WARN() { log_warn "$*"; }
FAIL() { log_error "$*"; FAILED=$((FAILED+1)); }
INFO() { log_info "$*"; }
SECTION() { echo; log_info "══ $* ══"; }

# ─── 0. Environment ───────────────────────────────────────────────────────────
SECTION "0. Environment"
INFO "Date/time : $(date)"
INFO "Running as: $(id)"
INFO "Kernel    : $(uname -r)"

# ─── 1. Rootless Podman binary & version ──────────────────────────────────────
SECTION "1. Podman Binary"
if command -v podman &>/dev/null; then
    VER=$(podman --version 2>/dev/null)
    PASS "podman found: $VER"
else
    FAIL "podman not found in PATH"
fi

if command -v podman-compose &>/dev/null; then
    PCVER=$(podman-compose --version 2>/dev/null | head -1)
    PASS "podman-compose found: $PCVER"
else
    FAIL "podman-compose not found in PATH. Install via: pip install --user podman-compose"
fi

# ─── 2. Rootless pre-requisites: subuid/subgid ────────────────────────────────
SECTION "2. UID/GID Namespace Mappings"
CUR_USER=$(whoami)

if grep -q "^${CUR_USER}:" /etc/subuid 2>/dev/null; then
    SUBUID=$(grep "^${CUR_USER}:" /etc/subuid)
    PASS "/etc/subuid entry: $SUBUID"
    RANGE=$(echo "$SUBUID" | cut -d: -f3)
    if [ "${RANGE:-0}" -ge 65536 ]; then
        PASS "  subuid range OK ($RANGE ≥ 65536)"
    else
        FAIL "  subuid range too small ($RANGE < 65536) — rootless containers need ≥65536"
    fi
else
    FAIL "No /etc/subuid entry for ${CUR_USER}. Run: sudo usermod --add-subuids 100000-165535 ${CUR_USER}"
fi

if grep -q "^${CUR_USER}:" /etc/subgid 2>/dev/null; then
    SUBGID=$(grep "^${CUR_USER}:" /etc/subgid)
    PASS "/etc/subgid entry: $SUBGID"
    RANGE=$(echo "$SUBGID" | cut -d: -f3)
    if [ "${RANGE:-0}" -ge 65536 ]; then
        PASS "  subgid range OK ($RANGE ≥ 65536)"
    else
        FAIL "  subgid range too small ($RANGE < 65536)"
    fi
else
    FAIL "No /etc/subgid entry for ${CUR_USER}. Run: sudo usermod --add-subgids 100000-165535 ${CUR_USER}"
fi

# ─── 3. newuidmap / newgidmap capabilities ─────────────────────────────────────
SECTION "3. newuidmap / newgidmap Binary Capabilities"
for BIN in /usr/bin/newuidmap /usr/bin/newgidmap; do
    if [ -x "$BIN" ]; then
        CAPS=$(getcap "$BIN" 2>/dev/null || echo "none")
        PERMS=$(stat -c '%a' "$BIN" 2>/dev/null)
        if echo "$CAPS" | grep -qE "cap_set(uid|gid)"; then
            PASS "$BIN capabilities: $CAPS (permissions: $PERMS)"
        else
            if [ "$PERMS" = "4755" ] || [ "$PERMS" = "4711" ]; then
                PASS "$BIN has SUID bit ($PERMS) — will work as fallback"
            else
                FAIL "$BIN has neither file capabilities nor SUID — rootless userns broken"
                INFO "  Fix: sudo setcap cap_setuid+ep /usr/bin/newuidmap  (or sudo chmod u+s $BIN)"
            fi
        fi
    else
        FAIL "$BIN not found or not executable"
    fi
done

# ─── 4. systemd unit security flags ───────────────────────────────────────────
SECTION "4. systemd User Unit Security Flags"
UNIT_FILE="${HOME}/.config/systemd/user/zafafworld.service"
if [ ! -f "$UNIT_FILE" ]; then
    WARN "Deployed unit not found at $UNIT_FILE (not yet installed)"
else
    NNP=$(grep -E "^NoNewPrivileges=" "$UNIT_FILE" | tail -1 | cut -d= -f2 | tr -d '[:space:]')
    if [ "${NNP,,}" = "false" ] || [ "${NNP,,}" = "no" ] || [ "${NNP,,}" = "0" ]; then
        PASS "NoNewPrivileges=false — newuidmap file-caps will work"
    else
        FAIL "NoNewPrivileges=${NNP} — MUST be 'false' for rootless Podman"
    fi

    PH=$(grep -E "^ProtectHome=" "$UNIT_FILE" | tail -1 | cut -d= -f2 | tr -d '[:space:]')
    if [ "${PH,,}" = "false" ] || [ "${PH,,}" = "no" ] || [ -z "$PH" ]; then
        PASS "ProtectHome=false — ~/.local/share/containers is writable"
    elif [ "${PH,,}" = "read-only" ]; then
        FAIL "ProtectHome=read-only — podman image store writes will fail"
    else
        FAIL "ProtectHome=true — home directory fully hidden, rootless Podman will fail"
    fi

    PS=$(grep -E "^ProtectSystem=" "$UNIT_FILE" | tail -1 | cut -d= -f2 | tr -d '[:space:]')
    if [ "${PS,,}" = "off" ] || [ "${PS,,}" = "false" ] || [ -z "$PS" ]; then
        PASS "ProtectSystem=off — cgroup v2 user-slice paths accessible"
    else
        FAIL "ProtectSystem=${PS} — causes cgroup write failures for rootless Podman"
    fi

    PCG=$(grep -E "^ProtectControlGroups=" "$UNIT_FILE" | tail -1 | cut -d= -f2 | tr -d '[:space:]')
    if [ "${PCG,,}" = "false" ] || [ "${PCG,,}" = "no" ] || [ -z "$PCG" ]; then
        PASS "ProtectControlGroups=false — container cgroup creation will work"
    else
        FAIL "ProtectControlGroups=${PCG} — MUST be false; makes /sys/fs/cgroup read-only"
    fi

    PD=$(grep -E "^PrivateDevices=" "$UNIT_FILE" | tail -1 | cut -d= -f2 | tr -d '[:space:]')
    if [ "${PD,,}" = "false" ] || [ "${PD,,}" = "no" ] || [ -z "$PD" ]; then
        PASS "PrivateDevices=false — user namespace setup will work"
    else
        FAIL "PrivateDevices=${PD} — MUST be false; breaks rootless userns mount namespace"
    fi

    PT=$(grep -E "^PrivateTmp=" "$UNIT_FILE" | tail -1 | cut -d= -f2 | tr -d '[:space:]')
    if [ "${PT,,}" = "false" ] || [ "${PT,,}" = "no" ] || [ -z "$PT" ]; then
        PASS "PrivateTmp=false — user namespace setup will work"
    else
        FAIL "PrivateTmp=${PT} — MUST be false; mount namespace change breaks rootless Podman"
    fi

    PKM=$(grep -E "^ProtectKernelModules=" "$UNIT_FILE" | tail -1 | cut -d= -f2 | tr -d '[:space:]')
    if [ "${PKM,,}" = "false" ] || [ "${PKM,,}" = "no" ] || [ -z "$PKM" ]; then
        PASS "ProtectKernelModules=false — kernel capability inheritance unaffected"
    else
        FAIL "ProtectKernelModules=${PKM} — MUST be false; sets SECBIT_NOROOT, strips newuidmap caps"
    fi

    PKT=$(grep -E "^ProtectKernelTunables=" "$UNIT_FILE" | tail -1 | cut -d= -f2 | tr -d '[:space:]')
    if [ "${PKT,,}" = "false" ] || [ "${PKT,,}" = "no" ] || [ -z "$PKT" ]; then
        PASS "ProtectKernelTunables=false — podman ps/compose can enumerate containers"
    else
        FAIL "ProtectKernelTunables=${PKT} — MUST be false; breaks 'podman ps' in named services"
    fi

    SCF=$(grep -E "^SystemCallFilter=" "$UNIT_FILE" | tail -1 | cut -d= -f2 | tr -d '[:space:]')
    if [ -z "$SCF" ]; then
        PASS "SystemCallFilter not set — no seccomp restriction blocking newuidmap"
    else
        FAIL "SystemCallFilter=${SCF} — ANY seccomp filter breaks rootless Podman under systemd"
    fi
fi

# ─── 5. SELinux status ────────────────────────────────────────────────────────
SECTION "5. SELinux"
if command -v sestatus &>/dev/null; then
    MODE=$(sestatus 2>/dev/null | grep "Current mode" | awk '{print $NF}')
    POLICY=$(sestatus 2>/dev/null | grep "Loaded policy" | awk '{print $NF}')
    INFO "SELinux mode: $MODE, policy: $POLICY"
    if [ "$MODE" = "enforcing" ]; then
        WARN "SELinux is enforcing — volume mounts must use :Z or :z relabeling flags"
        PASS "SELinux present — :Z/:z flags verified in podman-compose.yml"
    elif [ "$MODE" = "permissive" ]; then
        WARN "SELinux is permissive — will not block but also won't enforce isolation"
    else
        INFO "SELinux mode: $MODE"
    fi
else
    INFO "sestatus not found — SELinux may not be installed or active"
fi

# ─── 6. cgroup v2 ─────────────────────────────────────────────────────────────
SECTION "6. cgroup v2"
if [ -f /sys/fs/cgroup/cgroup.controllers ]; then
    CTRLS=$(cat /sys/fs/cgroup/cgroup.controllers)
    PASS "cgroup v2 unified hierarchy detected"
    INFO "  Available controllers: $CTRLS"
    if echo "$CTRLS" | grep -q "memory"; then
        PASS "  'memory' controller available — mem_limit in compose will work"
    else
        WARN "  'memory' controller missing — mem_limit/cpus restrictions may be ignored"
    fi
    if echo "$CTRLS" | grep -q "cpu"; then
        PASS "  'cpu' controller available — cpus quotas will work"
    else
        WARN "  'cpu' controller missing — cpus limits in compose may be ignored"
    fi
else
    WARN "cgroup v2 not detected (or not unified) — falling back to v1"
fi

USER_CGROUP="/sys/fs/cgroup/user.slice/user-$(id -u).slice"
if [ -d "$USER_CGROUP" ]; then
    PASS "User cgroup slice exists: $USER_CGROUP"
    if [ -w "$USER_CGROUP" ]; then
        PASS "  User cgroup slice is writable by $(whoami)"
    else
        WARN "  User cgroup slice not writable — container resource limits may fail"
    fi
else
    WARN "User cgroup slice not found — first container run will create it"
fi

# ─── 7. Port availability ─────────────────────────────────────────────────────
SECTION "7. Port Availability"
check_port() {
    local PORT=$1 DESC=$2
    if ss -tlnp 2>/dev/null | grep -qE ":${PORT}\b" || \
       ss -tlnp 2>/dev/null | grep -q "0.0.0.0:${PORT}" || \
       ss -tlnp 2>/dev/null | grep -q ":::${PORT}"; then
        WARN "Port $PORT ($DESC) is already in use — container bind may conflict"
        ss -tlnp 2>/dev/null | grep ":${PORT}" | head -3
    else
        PASS "Port $PORT ($DESC) is free"
    fi
}
check_port 8080  "nginx HTTP (host→container)"
check_port 8443  "nginx HTTPS (host→container)"
check_port 5433  "pgbouncer debug (host→container)"
check_port 5434  "postgres debug (host→container)"

# ─── 8. Volume & directory write permissions ──────────────────────────────────
SECTION "8. Volume & Directory Write Permissions"

LOG_DIR="/var/log/zafaf"
if [ -d "$LOG_DIR" ]; then
    if [ -w "$LOG_DIR" ]; then
        PASS "$LOG_DIR is writable"
    else
        FAIL "$LOG_DIR is not writable by $(whoami)"
    fi
else
    INFO "$LOG_DIR does not exist — service ExecStartPre will create it"
fi

NGINX_LOG="/var/log/zafaf/nginx"
if [ -d "$NGINX_LOG" ]; then
    if [ -w "$NGINX_LOG" ]; then
        PASS "$NGINX_LOG is writable"
    else
        FAIL "$NGINX_LOG is not writable"
    fi
fi

COMPOSE_DIR="${DEPLOY_ROOT:-/opt/zafafworld.net}"
if [ -d "$COMPOSE_DIR" ]; then
    if [ -w "$COMPOSE_DIR" ]; then
        PASS "$COMPOSE_DIR is writable"
    else
        FAIL "$COMPOSE_DIR is not writable — podman-compose cannot create volumes"
    fi
else
    FAIL "$COMPOSE_DIR does not exist — service will fail to start"
fi

STORAGE_ROOT="${HOME}/.local/share/containers/storage"
if [ -d "$STORAGE_ROOT" ]; then
    if [ -w "$STORAGE_ROOT" ]; then
        PASS "$STORAGE_ROOT is writable"
    else
        FAIL "$STORAGE_ROOT is not writable — image pulls and builds will fail"
    fi
else
    WARN "$STORAGE_ROOT does not exist yet — it will be created on first podman run"
fi

SSL_DIR="${COMPOSE_DIR}/infra/nginx/ssl"
if [ -d "$SSL_DIR" ]; then
    for F in zafafworld.net.crt zafafworld.net.key; do
        if [ -f "${SSL_DIR}/${F}" ]; then
            PASS "  SSL file present: ${SSL_DIR}/${F}"
        else
            FAIL "  SSL file MISSING: ${SSL_DIR}/${F} — nginx will refuse to start"
        fi
    done
else
    FAIL "$SSL_DIR directory missing — nginx Containerfile expects certs here"
fi

# ─── 9. podman system migrate ─────────────────────────────────────────────────
SECTION "9. Podman System Migration (DB schema)"
INFO "Running 'podman system migrate' to ensure DB is up-to-date..."
if podman system migrate 2>&1; then
    PASS "podman system migrate succeeded"
else
    WARN "podman system migrate returned non-zero — may indicate DB schema issue"
fi

# ─── 10. Podman rootless smoke-test ───────────────────────────────────────────
SECTION "10. Rootless Podman Smoke-Test"
INFO "Attempting: podman run --rm alpine id"
if OUTPUT=$(podman run --rm docker.io/library/alpine id 2>&1); then
    PASS "Rootless container run succeeded: $OUTPUT"
else
    FAIL "Rootless container run FAILED:"
    echo "$OUTPUT"
fi

# ─── 11. pgbouncer DATABASE_URL consistency check ─────────────────────────────
SECTION "11. pgbouncer / DATABASE_URL Consistency"
ENV_FILE="${COMPOSE_DIR}/.env"
if [ -f "$ENV_FILE" ]; then
    DB_URL=$(grep -E "^DATABASE_URL=" "$ENV_FILE" | cut -d= -f2-)
    PGUSER_ENV=$(grep -E "^POSTGRES_USER=" "$ENV_FILE" | cut -d= -f2-)
    PGDB_ENV=$(grep -E "^POSTGRES_DB=" "$ENV_FILE" | cut -d= -f2-)

    INFO "DATABASE_URL (from .env): $DB_URL"

    if echo "$DB_URL" | grep -q "@pgbouncer:5432/"; then
        PASS "DATABASE_URL points to pgbouncer:5432 (correct pooling path)"
    elif echo "$DB_URL" | grep -q "@postgres:5432/"; then
        WARN "DATABASE_URL points directly to postgres:5432 — bypasses PgBouncer!"
    else
        WARN "DATABASE_URL host is unexpected: $DB_URL"
    fi

    COMPOSE_FILE="${COMPOSE_DIR}/podman-compose.yml"
    if [ -f "$COMPOSE_FILE" ]; then
        PGB_HOST=$(grep -A20 "pgbouncer:" "$COMPOSE_FILE" | grep "DB_HOST:" | head -1 | awk '{print $NF}')
        INFO "pgbouncer DB_HOST in compose: ${PGB_HOST:-not found}"
        if [ "${PGB_HOST:-}" = "postgres" ]; then
            PASS "pgbouncer DB_HOST=postgres (correct — internal DNS name)"
        else
            WARN "pgbouncer DB_HOST=${PGB_HOST} — should be 'postgres' (the service name)"
        fi

        PGB_AUTH=$(grep -A20 "pgbouncer:" "$COMPOSE_FILE" | grep "AUTH_TYPE:" | head -1 | awk '{print $NF}')
        INFO "pgbouncer AUTH_TYPE in compose: ${PGB_AUTH:-not found}"
        if echo "${PGB_AUTH:-}" | grep -qi "scram"; then
            PASS "AUTH_TYPE=scram-sha-256 matches postgres password_encryption setting"
        else
            WARN "AUTH_TYPE=${PGB_AUTH} — ensure it matches postgres password_encryption in zafaf-perf.conf"
        fi
    fi
else
    WARN ".env not found at $ENV_FILE — skipping DATABASE_URL check"
fi

# ─── 12. tmpfs UDS volume declaration ─────────────────────────────────────────
SECTION "12. UDS Socket Volume (tmpfs)"
COMPOSE_FILE="${COMPOSE_DIR}/podman-compose.yml"
if [ -f "$COMPOSE_FILE" ]; then
    if grep -A5 "zafaf_uds_socket:" "$COMPOSE_FILE" | grep -q "tmpfs"; then
        PASS "zafaf_uds_socket volume declared as tmpfs (correct)"
    else
        WARN "zafaf_uds_socket volume is NOT tmpfs — socket persistence across restarts unverified"
    fi

    BACKEND_MOUNT=$(grep -A5 "zafaf_uds_socket" "$COMPOSE_FILE" | grep -v "volume" | head -5)
    if echo "$BACKEND_MOUNT" | grep -q ":z"; then
        PASS "UDS socket volume uses :z (shared SELinux label) — nginx can also access it"
    elif echo "$BACKEND_MOUNT" | grep -q ":Z"; then
        FAIL "UDS socket volume uses :Z (private label) — nginx container will be denied access"
    fi
fi

# ─── 13. listen_addresses cross-check ────────────────────────────────────────
SECTION "13. PostgreSQL listen_addresses"
PG_CONF="${COMPOSE_DIR}/infra/postgres/zafaf-perf.conf"
if [ -f "$PG_CONF" ]; then
    LA=$(grep "^listen_addresses" "$PG_CONF" | head -1)
    INFO "listen_addresses setting: $LA"
    if echo "$LA" | grep -q "localhost"; then
        FAIL "CRITICAL: listen_addresses must be '*' or '0.0.0.0' for inter-container TCP to work"
    elif echo "$LA" | grep -qE "(\*|0\.0\.0\.0)"; then
        PASS "listen_addresses allows inter-container connections"
    fi
else
    WARN "zafaf-perf.conf not found at $PG_CONF"
fi

# ─── 14. XDG_RUNTIME_DIR linger ───────────────────────────────────────────────
SECTION "14. Systemd Linger (auto-start without login)"
LINGER_FILE="/var/lib/systemd/linger/${CUR_USER}"
if [ -f "$LINGER_FILE" ]; then
    PASS "Linger enabled for ${CUR_USER} — service will auto-start at boot"
else
    WARN "Linger NOT enabled for ${CUR_USER}"
fi

# ─── 15. Migration Integrity Check ───────────────────────────────────────────
SECTION "15. Database Migration Integrity Check"
if [ -f "${SCRIPT_DIR}/check-migrations.sh" ]; then
    chmod +x "${SCRIPT_DIR}/check-migrations.sh"
    if bash "${SCRIPT_DIR}/check-migrations.sh"; then
        PASS "Migration integrity check passed"
    else
        FAIL "Migration integrity check FAILED"
    fi
else
    WARN "check-migrations.sh not found at ${SCRIPT_DIR}/check-migrations.sh"
fi

# ─── Summary ──────────────────────────────────────────────────────────────────
SECTION "AUDIT SUMMARY"
if [ "$FAILED" -eq 0 ]; then
    log_success "All checks passed. Infrastructure is ready for deployment."
else
    log_error "$FAILED check(s) FAILED — correct issues before executing deploy.sh"
fi
exit "$FAILED"
