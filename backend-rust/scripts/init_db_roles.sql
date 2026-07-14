-- =============================================================================
-- ZafafWorld Production PostgreSQL Infrastructure Bootstrap Script
-- Execute once as superuser (e.g. 'postgres') on database 'zafaf_world'
-- =============================================================================

BEGIN;

-- 1. Production Hardening: Revoke default CREATE privileges on schema public from PUBLIC
-- Reason: Preventing non-privileged users or connections from creating untracked tables in public schema.
REVOKE CREATE ON SCHEMA public FROM PUBLIC;

-- 2. Create Application Roles as NOLOGIN roles
DO $$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'app_client_role') THEN
        CREATE ROLE app_client_role NOLOGIN;
    END IF;
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'app_vendor_role') THEN
        CREATE ROLE app_vendor_role NOLOGIN;
    END IF;
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'app_admin_role') THEN
        CREATE ROLE app_admin_role NOLOGIN;
    END IF;
END $$;

-- 3. Grant Role Membership to Application Connection Pool User (zafaf_db_admin)
GRANT app_client_role, app_vendor_role, app_admin_role TO zafaf_db_admin;

-- 4. Grant Schema, Table, and Sequence Privileges
GRANT USAGE ON SCHEMA public TO app_client_role, app_vendor_role, app_admin_role;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO app_client_role, app_vendor_role, app_admin_role;
GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO app_client_role, app_vendor_role, app_admin_role;

-- 5. Set Default Privileges for Objects Created by zafaf_db_admin in Schema public
ALTER DEFAULT PRIVILEGES FOR ROLE zafaf_db_admin IN SCHEMA public 
    GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO app_client_role, app_vendor_role, app_admin_role;

ALTER DEFAULT PRIVILEGES FOR ROLE zafaf_db_admin IN SCHEMA public 
    GRANT USAGE, SELECT ON SEQUENCES TO app_client_role, app_vendor_role, app_admin_role;

COMMIT;
