#!/usr/bin/env bash
# =============================================================================
# ZafafWorld End-to-End HTTP API Authorization & Security Integration Test Suite
# Tests HTTP API endpoints for status codes, auth enforcement, and isolation.
# =============================================================================

set -e

API_BASE_URL="${API_BASE_URL:-http://127.0.0.1:8080}"

echo "============================================================================="
echo "Starting ZafafWorld E2E HTTP API Security & Authorization Audit..."
echo "Target Base URL: ${API_BASE_URL}"
echo "============================================================================="

FAILED_TESTS=0

test_endpoint() {
    local name="$1"
    local expected_code="$2"
    local http_code

    http_code=$(curl -s -o /dev/null -w "%{http_code}" "$3" "${@:4}")
    if [ "$http_code" -eq "$expected_code" ]; then
        echo "[PASS] $name -> Expected $expected_code, Got $http_code"
    else
        echo "[FAIL] $name -> Expected $expected_code, Got $http_code"
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
}

echo "--- 1. Testing Public Endpoints (Anonymous Access) ---"
test_endpoint "Public Cities Endpoint" 200 "${API_BASE_URL}/api/v1/public/cities"
test_endpoint "Public Categories Endpoint" 200 "${API_BASE_URL}/api/v1/public/categories"

echo "--- 2. Testing Unauthenticated Access to Protected Routes (Expects 401) ---"
test_endpoint "Vendor Products (No JWT)" 401 "${API_BASE_URL}/api/v1/vendor/products"
test_endpoint "Admin Users (No JWT)" 401 "${API_BASE_URL}/api/v1/admin/users"

echo "--- 3. Testing Invalid JWT Authorization (Expects 401) ---"
test_endpoint "Vendor Products (Invalid JWT)" 401 "${API_BASE_URL}/api/v1/vendor/products" -H "Authorization: Bearer invalid_token_xyz"
test_endpoint "Admin Users (Malformed JWT)" 401 "${API_BASE_URL}/api/v1/admin/users" -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.invalid"

echo "--- 4. Testing Malformed Resource Lookups (Expects 400 or 404) ---"
test_endpoint "Non-existent Public Listing" 404 "${API_BASE_URL}/api/v1/public/venues/00000000-0000-0000-0000-000000000000"

echo "============================================================================="
if [ "$FAILED_TESTS" -eq 0 ]; then
    echo "ALL E2E HTTP API SECURITY TESTS PASSED SUCCESSFULLY!"
    exit 0
else
    echo "ERROR: ${FAILED_TESTS} E2E HTTP API security test(s) failed!"
    exit 1
fi
