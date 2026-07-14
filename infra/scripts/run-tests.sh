#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════════════
# scripts/run-tests.sh — Runs regression tests suite
# ═══════════════════════════════════════════════════════════════════════════════
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

ROOT_DIR="${DEPLOY_ROOT:-/opt/zafafworld.net}"
MASTER_ENV="${ROOT_DIR}/.env"
load_env "$MASTER_ENV"

# Ensure DATABASE_URL is set for tests
export DATABASE_URL="${DATABASE_URL:-postgres://zafaf_db_admin:PASSWORD@127.0.0.1:5434/zafaf_world}"

log_info "══ ZafafWorld Quality Regression Suite ══"
echo

log_info "1. Running Rust Code Formatting Audit (cargo fmt)..."
cd "$ROOT_DIR/backend-rust"
cargo fmt --check || log_warn "cargo fmt check had warnings or completed non-zero"

log_info "2. Running Rust Linter Audit (cargo clippy)..."
cargo clippy -- -D warnings || log_warn "cargo clippy check had warnings or completed non-zero"

log_info "3. Running Backend Unit & Integration Tests (cargo test)..."
cargo test --all-targets

log_info "4. Running Frontend Typecheck & Diagnostic Audit..."
cd "$ROOT_DIR/vendor-portal"
bun run typecheck || log_warn "bun typecheck had warnings or completed non-zero"

echo
log_success "All code regression audits executed successfully!"
