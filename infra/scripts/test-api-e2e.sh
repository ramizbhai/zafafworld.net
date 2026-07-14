#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════════════
# scripts/test-api-e2e.sh — Runs E2E HTTP API security checks
# ═══════════════════════════════════════════════════════════════════════════════
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Source config
INFRA_ENV="$SCRIPT_DIR/../.env"
if [[ -f "$INFRA_ENV" ]]; then
    set -a; source "$INFRA_ENV"; set +a
fi

# Source logging helper
if [[ -f "$SCRIPT_DIR/../lib/logging.sh" ]]; then
    source "$SCRIPT_DIR/../lib/logging.sh"
else
    log_info() { echo -e "[INFO] $*"; }
    log_success() { echo -e "\033[0;32m[PASS]\033[0m $*"; }
    log_warn() { echo -e "\033[1;33m[WARN]\033[0m $*"; }
    log_error() { echo -e "\033[0;31m[FAIL]\033[0m $*" >&2; }
fi

API_BASE_URL="${API_BASE_URL:-http://127.0.0.1:8080}"

log_info "══ ZafafWorld E2E HTTP API Security & Authorization Audit ══"
log_info "Target Base URL: ${API_BASE_URL}"
echo

FAILED_TESTS=0

test_endpoint() {
    local name="$1"
    local expected_code="$2"
    local http_code

    http_code=$(curl -s -o /dev/null -w "%{http_code}" "$3" "${@:4}")
    if [ "$http_code" -eq "$expected_code" ]; then
        log_success "$name -> Expected $expected_code, Got $http_code"
    else
        log_error "$name -> Expected $expected_code, Got $http_code"
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
}

log_info "--- 1. Testing Public Endpoints (Anonymous Access) ---"
test_endpoint "Public Cities Endpoint" 200 "${API_BASE_URL}/api/v1/public/cities"
test_endpoint "Public Categories Endpoint" 200 "${API_BASE_URL}/api/v1/public/categories"

log_info "--- 2. Testing Unauthenticated Access to Protected Routes (Expects 401) ---"
test_endpoint "Vendor Products (No JWT)" 401 "${API_BASE_URL}/api/v1/vendor/products"
test_endpoint "Admin Users (No JWT)" 401 "${API_BASE_URL}/api/v1/admin/users"

log_info "--- 3. Testing Invalid JWT Authorization (Expects 401) ---"
test_endpoint "Vendor Products (Invalid JWT)" 401 "${API_BASE_URL}/api/v1/vendor/products" -H "Authorization: Bearer invalid_token_xyz"
test_endpoint "Admin Users (Malformed JWT)" 401 "${API_BASE_URL}/api/v1/admin/users" -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.invalid"

log_info "--- 4. Testing Malformed Resource Lookups (Expects 400 or 404) ---"
test_endpoint "Non-existent Public Listing" 404 "${API_BASE_URL}/api/v1/public/venues/00000000-0000-0000-0000-000000000000"

echo
if [ "$FAILED_TESTS" -eq 0 ]; then
    log_success "All E2E HTTP API security checks passed cleanly."
    exit 0
else
    log_error "$FAILED_TESTS E2E HTTP API security check(s) failed."
    exit 1
fi
