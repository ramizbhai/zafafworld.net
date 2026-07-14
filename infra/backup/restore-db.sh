#!/usr/bin/env bash
# ══════════════════════════════════════════════════════════════════════════════
# infra/backup/restore-db.sh — ZafafWorld Database Restore Pipeline
#
# USAGE:  bash restore-db.sh <backup-file.dump> [--execute]
#         (Default is --dry-run safety mode, which prints actions without executing)
# ══════════════════════════════════════════════════════════════════════════════
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Source logging helper first to inherit load_env function
if [[ -f "$SCRIPT_DIR/../lib/logging.sh" ]]; then
    source "$SCRIPT_DIR/../lib/logging.sh"
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
INFRA_ENV="$SCRIPT_DIR/../.env"
load_env "$INFRA_ENV"

DEPLOY_ROOT="${DEPLOY_ROOT:-/opt/zafafworld.net}"
MASTER_ENV="${DEPLOY_ROOT}/.env"
load_env "$MASTER_ENV"

BACKUP_FILE=""
EXECUTE=false

for arg in "$@"; do
    case "$arg" in
        --execute|--force) EXECUTE=true ;;
        --dry-run) EXECUTE=false ;;
        *)
            if [[ -z "$BACKUP_FILE" ]]; then
                BACKUP_FILE="$arg"
            else
                log_error "Unknown argument: $arg"
                exit 1
            fi
            ;;
    esac
done

if [[ -z "$BACKUP_FILE" ]]; then
    log_error "Usage: $0 <backup-file.dump> [--execute]"
    exit 1
fi

if [[ ! -f "$BACKUP_FILE" ]]; then
    log_error "Backup file not found: $BACKUP_FILE"
    exit 1
fi

PGHOST="${PGHOST:-localhost}"
PGPORT="${PGPORT:-5432}"
PGDATABASE="${POSTGRES_DB:-zafaf_world}"
PGUSER="${POSTGRES_USER:-postgres}"
RESTORE_TARGET_DB="${RESTORE_TARGET_DB:-$PGDATABASE}"

log_info "══ ZafafWorld Database Restore Tool ══"
log_info "Backup source file : $BACKUP_FILE"
log_info "Target Database    : $RESTORE_TARGET_DB"
log_info "Execution mode     : $( [[ "$EXECUTE" == "true" ]] && echo "LIVE EXECUTE" || echo "DRY RUN (ReadOnly)" )"
echo

if [[ "$EXECUTE" != "true" ]]; then
    log_warn "DEFAULT SAFE MODE: Performing a dry-run check. Pass --execute to run live."
fi

# 1. Verify Checksum
CHECKSUM_FILE="${BACKUP_FILE}.sha256"
if [[ -f "$CHECKSUM_FILE" ]]; then
    log_info "Verifying backup file SHA-256 integrity..."
    if sha256sum --check "$CHECKSUM_FILE" --status; then
        log_success "Checksum verification: OK"
    else
        log_error "Checksum verification FAILED. Backup may be corrupted!"
        exit 1
    fi
else
    log_warn "No checksum file found at $CHECKSUM_FILE. Skipping integrity check."
fi

if [[ "$EXECUTE" == "true" ]]; then
    log_warn "WARNING: Recreating target database '$RESTORE_TARGET_DB' on $PGHOST:$PGPORT"
    
    export PGPASSWORD="${POSTGRES_PASSWORD:-}"
    
    log_info "Terminating active connections and dropping database '$RESTORE_TARGET_DB'..."
    psql \
        --host="$PGHOST" \
        --port="$PGPORT" \
        --username="$PGUSER" \
        --dbname=postgres \
        --no-password \
        -c "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '${RESTORE_TARGET_DB}' AND pid <> pg_backend_pid();" \
        -c "DROP DATABASE IF EXISTS \"${RESTORE_TARGET_DB}\";" \
        -c "CREATE DATABASE \"${RESTORE_TARGET_DB}\" ENCODING='UTF8';"
    
    if [[ "$BACKUP_FILE" =~ \.sql\.gz$ ]]; then
        log_info "Restoring schemas and tables from compressed SQL backup..."
        if gunzip -c "$BACKUP_FILE" | psql \
            --host="$PGHOST" \
            --port="$PGPORT" \
            --username="$PGUSER" \
            --dbname="$RESTORE_TARGET_DB" \
            --no-password; then
            log_success "Database restore executed successfully."
        else
            log_error "psql restore returned non-zero exit status."
            exit 1
        fi
    else
        log_info "Restoring schemas and tables from backup archive..."
        if pg_restore \
            --host="$PGHOST" \
            --port="$PGPORT" \
            --username="$PGUSER" \
            --dbname="$RESTORE_TARGET_DB" \
            --no-password \
            --jobs=4 \
            --no-owner \
            --no-acl \
            "$BACKUP_FILE"; then
            log_success "Database restore executed successfully."
        else
            log_error "pg_restore returned non-zero exit status."
            exit 1
        fi
    fi
    
    TABLE_COUNT=$(psql --host="$PGHOST" --port="$PGPORT" --username="$PGUSER" --dbname="$RESTORE_TARGET_DB" --no-password --tuples-only --command="SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public';" | tr -d '[:space:]')
    log_success "Database restore complete. Active tables count: $TABLE_COUNT"
else
    log_info "Would terminate active connections and drop target database: $RESTORE_TARGET_DB"
    if [[ "$BACKUP_FILE" =~ \.sql\.gz$ ]]; then
        log_info "Would run gunzip and psql on target database '$RESTORE_TARGET_DB' using file $BACKUP_FILE"
    else
        log_info "Would run pg_restore on target database '$RESTORE_TARGET_DB' using file $BACKUP_FILE"
    fi
fi
