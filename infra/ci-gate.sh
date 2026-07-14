#!/usr/bin/env bash
# ══════════════════════════════════════════════════════════════════════════════
# infra/ci-gate.sh — ZafafWorld Continuous Integration Gate
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

DEPLOY_ROOT="${DEPLOY_ROOT:-/opt/zafafworld.net}"
ENV_FILE="$DEPLOY_ROOT/.env"
FAILED=0

log_info "══ ZafafWorld CI Gate ══"
echo

if [[ ! -f "$ENV_FILE" ]]; then
    log_error "ERROR: .env file not found at $ENV_FILE"
    exit 1
fi

POSTGRES_USER=$(grep -E '^POSTGRES_USER=' "$ENV_FILE" | cut -d'=' -f2- | tr -d '"' | tr -d "'")
POSTGRES_DB=$(grep -E '^POSTGRES_DB=' "$ENV_FILE" | cut -d'=' -f2- | tr -d '"' | tr -d "'")
POSTGRES_PASSWORD=$(grep -E '^POSTGRES_PASSWORD=' "$ENV_FILE" | cut -d'=' -f2- | tr -d '"' | tr -d "'")

POSTGRES_USER="${POSTGRES_USER:-zafaf_db_admin}"
POSTGRES_DB="${POSTGRES_DB:-zafaf_world}"
POSTGRES_PASSWORD="${POSTGRES_PASSWORD:-}"

CONTAINER_NAME=$(podman ps -a --format '{{.Names}}' 2>/dev/null | grep -E 'postgres' | head -1 || echo "")
DB_STARTED_BY_US=false

# Ensure PostgreSQL is running
if [[ -z "$CONTAINER_NAME" ]]; then
    log_info "PostgreSQL container does not exist. Spinning up a temporary postgres container..."
    podman-compose -f "$DEPLOY_ROOT/podman-compose.yml" --env-file "$ENV_FILE" -p zafafworld up -d postgres >/dev/null
    sleep 5
    CONTAINER_NAME="zafafworld_postgres_1"
    DB_STARTED_BY_US=true
elif ! podman ps --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
    log_info "PostgreSQL container ($CONTAINER_NAME) is not running. Starting it temporarily..."
    if podman start "$CONTAINER_NAME" >/dev/null; then
        DB_STARTED_BY_US=true
        sleep 3
    else
        log_error "Failed to start PostgreSQL container."
        exit 1
    fi
fi

export DATABASE_URL="postgres://${POSTGRES_USER}:${POSTGRES_PASSWORD}@127.0.0.1:5434/${POSTGRES_DB}"

cleanup() {
    if [[ "$DB_STARTED_BY_US" == "true" ]]; then
        log_info "Stopping temporary PostgreSQL container..."
        podman stop "$CONTAINER_NAME" >/dev/null || true
    fi
}
trap cleanup EXIT

# ── 1. Cargo Fmt Check ────────────────────────────────────────────────────────
log_info "[1/6] Running cargo fmt --check..."
if cargo fmt --manifest-path "$DEPLOY_ROOT/backend-rust/Cargo.toml" -- --check; then
    log_success "Formatting OK"
else
    log_error "Formatting errors detected. Run 'cargo fmt' to fix."
    FAILED=$((FAILED + 1))
fi

# ── 2. Cargo Clippy Check ─────────────────────────────────────────────────────
log_info "[2/6] Running cargo clippy..."
if cargo clippy --manifest-path "$DEPLOY_ROOT/backend-rust/Cargo.toml" --all-targets -- -D warnings; then
    log_success "Clippy lints OK"
else
    log_error "Clippy warnings/errors detected."
    FAILED=$((FAILED + 1))
fi

# ── 3. Cargo Test Check ───────────────────────────────────────────────────────
log_info "[3/6] Running cargo test..."
if cargo test --manifest-path "$DEPLOY_ROOT/backend-rust/Cargo.toml" --workspace; then
    log_success "Rust tests OK"
else
    log_error "Cargo tests FAILED."
    FAILED=$((FAILED + 1))
fi

# ── 4. Svelte Frontend Check ──────────────────────────────────────────────────
log_info "[4/6] Running Svelte frontend builds & type checks..."
if bash "$SCRIPT_DIR/validate-build.sh" --skip-install; then
    log_success "Frontend builds & checks OK"
else
    log_error "Frontend checks FAILED."
    FAILED=$((FAILED + 1))
fi

# ── 5. Check Migrations Integrity ─────────────────────────────────────────────
log_info "[5/6] Checking migration integrity..."
if bash "$SCRIPT_DIR/check-migrations.sh"; then
    log_success "Migration integrity check OK"
else
    log_error "Migration integrity check FAILED."
    FAILED=$((FAILED + 1))
fi

# ── 6. Podman Compose Config Check ────────────────────────────────────────────
log_info "[6/6] Validating podman-compose config..."
if podman-compose -f "$DEPLOY_ROOT/podman-compose.yml" --env-file "$ENV_FILE" config >/dev/null; then
    log_success "Compose file configuration OK"
else
    log_error "Compose configuration invalid."
    FAILED=$((FAILED + 1))
fi

echo
log_info "══ CI Gate Summary ══"
if [[ "$FAILED" -eq 0 ]]; then
    log_success "CI Gate PASSED successfully."
    exit 0
else
    log_error "CI Gate FAILED with ${FAILED} error(s)."
    exit 1
fi
