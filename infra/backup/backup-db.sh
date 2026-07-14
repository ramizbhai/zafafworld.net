#!/usr/bin/env bash
# ══════════════════════════════════════════════════════════════════════════════
# infra/backup/backup-db.sh — ZafafWorld Database & Uploads Backup Pipeline
#
# USAGE:  bash backup-db.sh [--execute] [--dry-run]
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

EXECUTE=false
for arg in "$@"; do
    case "$arg" in
        --execute|--force) EXECUTE=true ;;
        --dry-run) EXECUTE=false ;;
        *) log_error "Unknown argument: $arg"; exit 1 ;;
    esac
done

PGDATABASE="${POSTGRES_DB:-zafaf_world}"
PGUSER="${POSTGRES_USER:-postgres}"
BACKUP_DIR="${BACKUP_DIR:-/var/lib/zafafworld/backups}"
RETAIN_DAYS="${BACKUP_RETAIN_DAYS:-5}"
TIMESTAMP="$(date +%Y%m%d_%H%M%S)"
LOG_FILE="${BACKUP_DIR}/logs/backup.log"

log_to_file() {
    local level="$1"
    local message="$2"
    if [[ "$EXECUTE" == "true" ]]; then
        mkdir -p "$(dirname "$LOG_FILE")"
        echo "[$(date '+%Y-%m-%d %H:%M:%S')] [$level] $message" >> "$LOG_FILE"
    fi
}

cleanup_on_exit() {
    local exit_code=$?
    if [[ "$EXECUTE" == "true" ]]; then
        if [[ $exit_code -eq 0 ]]; then
            log_to_file "INFO" "ZafafWorld backup pipeline finished. Status: SUCCESS"
        else
            log_to_file "ERROR" "ZafafWorld backup pipeline finished. Status: FAILURE (Exit code: $exit_code)"
        fi
    fi
}
trap cleanup_on_exit EXIT

log_to_file "INFO" "Starting ZafafWorld backup pipeline (Manual/Automated Run)..."

log_info "══ ZafafWorld Database Backup Tool ══"
log_info "Backup target dir : $BACKUP_DIR"
log_info "Retention days    : $RETAIN_DAYS"
log_info "Execution mode    : $( [[ "$EXECUTE" == "true" ]] && echo "LIVE EXECUTE" || echo "DRY RUN (ReadOnly)" )"
echo

if [[ "$EXECUTE" != "true" ]]; then
    log_warn "DEFAULT SAFE MODE: Performing a dry-run check. Pass --execute to run live."
fi

# Dry-run vs live logic for directory creation
if [[ "$EXECUTE" == "true" ]]; then
    mkdir -p "$BACKUP_DIR"
    chmod 700 "$BACKUP_DIR"
else
    log_info "Would ensure directory exists: $BACKUP_DIR (mode 700)"
fi

# 1. Database Dump
PG_CONTAINER=$(podman ps --filter "name=postgres" --filter "status=running" -q | head -n 1 || true)
if [[ -n "$PG_CONTAINER" ]]; then
    BACKUP_FILE="${BACKUP_DIR}/zafaf_${PGDATABASE}_${TIMESTAMP}.sql.gz"
    if [[ "$EXECUTE" == "true" ]]; then
        log_info "Executing pg_dump in container '$PG_CONTAINER' as user '$PGUSER'..."
        log_to_file "INFO" "Locating active PostgreSQL container..."
        log_to_file "INFO" "Found active database container: ${PG_CONTAINER:0:12}"
        log_to_file "INFO" "Executing pg_dump inside container..."
        
        SQL_FILE="${BACKUP_DIR}/zafaf_${PGDATABASE}_${TIMESTAMP}.sql"
        if podman exec -i "$PG_CONTAINER" pg_dump -U "$PGUSER" -d "$PGDATABASE" -F p > "$SQL_FILE"; then
            log_info "Compressing SQL dump..."
            if gzip -f "$SQL_FILE"; then
                sha256sum "$BACKUP_FILE" > "${BACKUP_FILE}.sha256"
                log_success "Database backup written to: $BACKUP_FILE"
                log_to_file "INFO" "Database dump successful: $(du -sh "$BACKUP_FILE" | cut -f1)"
            else
                log_error "Compression of SQL dump failed!"
                log_to_file "ERROR" "Compression of SQL dump failed!"
                rm -f "$SQL_FILE"
                exit 1
            fi
        else
            log_error "Database dump failed!"
            log_to_file "ERROR" "Database dump failed!"
            exit 1
        fi
    else
        log_info "Would run pg_dump for db '$PGDATABASE' in container '$PG_CONTAINER' -> ${BACKUP_DIR}/zafaf_${PGDATABASE}_${TIMESTAMP}.sql"
        log_info "Would compress SQL dump to $BACKUP_FILE"
    fi
else
    log_warn "No running PostgreSQL container found. Skipping database dump check."
    log_to_file "WARN" "No running PostgreSQL container found. Skipping database dump check."
fi

# 2. Uploads Directory Archive
UPLOADS_DIR="${UPLOADS_VOLUME_DIR:-/var/lib/zafafworld/uploads}"
if [[ -d "$UPLOADS_DIR" ]]; then
    UPLOADS_EXPORT_FILE="${BACKUP_DIR}/uploads_dir_${TIMESTAMP}.tar.gz"
    if [[ "$EXECUTE" == "true" ]]; then
        log_info "Archiving uploads directory '$UPLOADS_DIR'..."
        log_to_file "INFO" "Archiving uploads directory: $UPLOADS_DIR..."
        if tar -czf "$UPLOADS_EXPORT_FILE" -C "$(dirname "$UPLOADS_DIR")" "$(basename "$UPLOADS_DIR")"; then
            sha256sum "$UPLOADS_EXPORT_FILE" > "${UPLOADS_EXPORT_FILE}.sha256"
            log_success "Uploads directory backup written to: $UPLOADS_EXPORT_FILE"
            log_to_file "INFO" "Uploads archive successful: $(du -sh "$UPLOADS_EXPORT_FILE" | cut -f1)"
        else
            log_error "Uploads archiving failed!"
            log_to_file "ERROR" "Uploads archiving failed!"
            exit 1
        fi
    else
        log_info "Would run tar archive on uploads: $UPLOADS_DIR -> $UPLOADS_EXPORT_FILE"
    fi
else
    log_warn "Uploads directory not found: $UPLOADS_DIR"
    log_to_file "WARN" "Uploads directory not found: $UPLOADS_DIR"
fi

# 3. Clean up old backups
log_info "Scanning for backups older than $RETAIN_DAYS days..."
log_to_file "INFO" "Pruning backups older than $RETAIN_DAYS days..."
if [[ "$EXECUTE" == "true" ]]; then
    PRUNED=0
    while IFS= read -r old_file; do
        if [[ -f "$old_file" ]]; then
            rm -f "$old_file" "${old_file}.sha256" || true
            log_info "Pruned old backup file: $(basename "$old_file")"
            log_to_file "INFO" "Pruned old backup file: $(basename "$old_file")"
            PRUNED=$((PRUNED + 1))
        fi
    done < <(find "$BACKUP_DIR" -maxdepth 1 \( -name "zafaf_${PGDATABASE}_*.sql.gz" -o -name "zafaf_${PGDATABASE}_*.sql" -o -name "uploads_dir_*.tar.gz" -o -name "zafaf_${PGDATABASE}_*.dump" \) -mtime +"$RETAIN_DAYS" 2>/dev/null)
    log_success "Pruned $PRUNED old backup file(s)."
    log_to_file "INFO" "Pruned $PRUNED old backup file(s)."
else
    while IFS= read -r old_file; do
        if [[ -f "$old_file" ]]; then
            log_info "Would prune old backup file: $(basename "$old_file")"
        fi
    done < <(find "$BACKUP_DIR" -maxdepth 1 \( -name "zafaf_${PGDATABASE}_*.sql.gz" -o -name "zafaf_${PGDATABASE}_*.sql" -o -name "uploads_dir_*.tar.gz" -o -name "zafaf_${PGDATABASE}_*.dump" \) -mtime +"$RETAIN_DAYS" 2>/dev/null)
fi

log_success "Database backup check completed."
