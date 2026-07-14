-- =============================================================================
-- ZafafWorld CI/CD Database RLS & Privilege Verification Script
-- Run this script in CI/CD or post-deployment testing to validate schema security.
-- =============================================================================

DO $$
DECLARE
    rec RECORD;
    missing_rls_count INT := 0;
    missing_policy_count INT := 0;
    unprivileged_table_count INT := 0;
BEGIN
    RAISE NOTICE '=============================================================================';
    RAISE NOTICE 'Starting Automated Database RLS & Security Verification...';
    RAISE NOTICE '=============================================================================';

    -- 1. Verify RLS is enabled on mandatory security-sensitive tables
    FOR rec IN 
        SELECT unnest(ARRAY[
            'vendor_products', 'vendors', 'Core_Bookings', 'Lead_Inquiries', 
            'vendor_gallery', 'packages', 'csrf_tokens', 'Client_Budgets'
        ]) AS tbl
    LOOP
        IF NOT EXISTS (
            SELECT 1 FROM pg_tables WHERE schemaname = 'public' AND tablename = rec.tbl AND rowsecurity = true
        ) THEN
            RAISE WARNING 'SECURITY VIOLATION: Table % does not have RLS enabled!', rec.tbl;
            missing_rls_count := missing_rls_count + 1;
        END IF;
    END LOOP;

    -- 2. Verify Key Table Privileges for app roles
    FOR rec IN 
        SELECT 'app_vendor_role' AS role, 'vendor_products' AS tbl UNION ALL
        SELECT 'app_vendor_role', 'vendors' UNION ALL
        SELECT 'app_client_role', 'Core_Bookings' UNION ALL
        SELECT 'app_admin_role', 'global_users'
    LOOP
        IF NOT has_table_privilege(rec.role, rec.tbl, 'SELECT') THEN
            RAISE WARNING 'PRIVILEGE VIOLATION: Role % lacks SELECT privilege on table %!', rec.role, rec.tbl;
            unprivileged_table_count := unprivileged_table_count + 1;
        END IF;
    END LOOP;

    -- Summary Check
    IF missing_rls_count > 0 OR unprivileged_table_count > 0 THEN
        RAISE EXCEPTION 'Database Verification Failed: % missing RLS tables, % unprivileged tables.', 
            missing_rls_count, unprivileged_table_count;
    ELSE
        RAISE NOTICE 'SUCCESS: All database RLS security policies and role privileges verified!';
    END IF;
END $$;
