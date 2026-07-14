#!/usr/bin/env bash
# Check database migrations integrity against SQLx _sqlx_migrations table
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

if [[ ! -f "$ENV_FILE" ]]; then
    log_error ".env file not found at $ENV_FILE"
    exit 1
fi

POSTGRES_USER=$(grep -E '^POSTGRES_USER=' "$ENV_FILE" | cut -d'=' -f2- | tr -d '"' | tr -d "'")
POSTGRES_DB=$(grep -E '^POSTGRES_DB=' "$ENV_FILE" | cut -d'=' -f2- | tr -d '"' | tr -d "'")
POSTGRES_USER="${POSTGRES_USER:-postgres}"
POSTGRES_DB="${POSTGRES_DB:-zafaf_world}"

CONTAINER_NAME=$(podman ps -a --format '{{.Names}}' 2>/dev/null | grep -E 'postgres' | head -1 || echo "")

if [[ -z "$CONTAINER_NAME" ]]; then
    log_info "PostgreSQL container does not exist. Spinning up a temporary postgres container using podman-compose..."
    podman-compose -f "$DEPLOY_ROOT/podman-compose.yml" --env-file "$ENV_FILE" -p zafafworld up -d postgres >/dev/null
    sleep 5
    CONTAINER_NAME="zafafworld_postgres_1"
elif ! podman ps --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
    log_info "PostgreSQL container ($CONTAINER_NAME) is not running. Starting it temporarily..."
    podman start "$CONTAINER_NAME" >/dev/null
    sleep 3
fi

log_info "Fetching applied migrations from database..."
if ! DB_DATA=$(podman exec -i "$CONTAINER_NAME" psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -t -A -F ',' -c "SELECT version, encode(checksum, 'hex') FROM _sqlx_migrations WHERE success = true;" 2>/dev/null); then
    log_warn "_sqlx_migrations table does not exist or database is empty. Skipping checksum validation."
    exit 0
fi

declare -A db_migrations
while IFS=',' read -r version checksum; do
    if [[ -n "$version" && -n "$checksum" ]]; then
        db_migrations["$version"]="$checksum"
    fi
done <<< "$DB_DATA"

log_info "Checking local migration files..."
MISMATCH=0
MIGRATIONS_DIR="$DEPLOY_ROOT/backend-rust/migrations"

for file in "$MIGRATIONS_DIR"/*.sql; do
    [[ -e "$file" ]] || continue
    filename=$(basename "$file")
    version=$(echo "$filename" | cut -d'_' -f1)
    
    local_hash=$(sha384sum "$file" | awk '{print $1}')
    
    if [[ -n "${db_migrations[$version]:-}" ]]; then
        db_hash="${db_migrations[$version]}"
        if [[ "$local_hash" != "$db_hash" ]]; then
            log_error "Migration file modified! File: $filename"
            log_info "   Local SHA-384: $local_hash"
            log_info "   DB SHA-384:    $db_hash"
            MISMATCH=1
        else
            log_success "$filename matches DB checksum"
        fi
    else
        log_info "$filename is a new migration (not yet applied)"
    fi
done

if [[ "$MISMATCH" -eq 1 ]]; then
    log_error "CRITICAL ERROR: Applied database migrations have been modified!"
    log_info "   Every schema correction must be done via a NEW migration file."
    exit 1
else
    log_success "Migration integrity check passed."
    exit 0
fi
