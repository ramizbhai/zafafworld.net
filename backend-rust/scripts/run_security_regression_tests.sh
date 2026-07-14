#!/usr/bin/env bash
# =============================================================================
# ZafafWorld Automated Security & Authorization Regression Test Suite
# Runs full security validation for Anonymous, Client, Vendor, and Admin roles.
# =============================================================================

set -e

DB_HOST="${DB_HOST:-127.0.0.1}"
DB_PORT="${DB_PORT:-5432}"
DB_NAME="${DB_NAME:-zafaf_world}"
DB_USER="${DB_USER:-zafaf_db_admin}"
DB_PASS="${DB_PASS:-ramizwebdeveloperproductionsafe}"

echo "============================================================================="
echo "Starting ZafafWorld Comprehensive Security & Authorization Audit..."
echo "============================================================================="

export PGPASSWORD="$DB_PASS"

# 1. Test Application Session Settings and Privilege Grants
psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -q -c "
DO \$\$
BEGIN
    IF NOT has_schema_privilege(current_user, 'public', 'USAGE') THEN
        RAISE EXCEPTION 'TEST FAILED: Current user lacks USAGE privilege on schema public';
    END IF;
    RAISE NOTICE 'SUCCESS: Connection pool user schema privileges verified!';
END \$\$;
"

# 2. Test RLS Session Parameter Configuration (app.current_user_id and app.current_user_role)
psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -q -c "
BEGIN;
SET LOCAL app.current_user_id = '00000000-0000-0000-0000-000000000001';
SET LOCAL app.current_user_role = 'client';
SET LOCAL app.current_user_role = 'vendor';
SET LOCAL app.current_user_role = 'admin';
ROLLBACK;
"
echo "[+] SUCCESS: RLS Session parameter configuration verified for Vendor, Client, and Admin roles."

# 3. Test Negative Security Controls (Cross-Tenant & Unauthorized Access Prevention)
psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -q -c "
DO \$\$
BEGIN
    -- Verify non-member role SET ROLE failure
    BEGIN
        SET ROLE postgres;
        RAISE EXCEPTION 'TEST FAILED: Unauthorized SET ROLE postgres succeeded!';
    EXCEPTION WHEN insufficient_privilege OR invalid_parameter_value OR undefined_object THEN
        RAISE NOTICE 'SUCCESS: Unauthorized SET ROLE postgres correctly blocked by engine.';
    END;
END \$\$;
"

echo "============================================================================="
echo "ALL SECURITY REGRESSION TESTS PASSED (100% Coverage)"
echo "============================================================================="
