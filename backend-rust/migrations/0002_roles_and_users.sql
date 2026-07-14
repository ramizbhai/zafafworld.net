-- ═══════════════════════════════════════════════════════════════════════════════
-- Baseline Migration 02: 0002_roles_and_users.sql
-- Consolidated ZafafWorld Database Schema Baseline
-- ═══════════════════════════════════════════════════════════════════════════════

BEGIN;

--


--
-- Name: user_domain_enum; Type: TYPE; Schema: public; Owner: zafaf_db_admin
--

CREATE TYPE public.user_domain_enum AS ENUM (
    'Client',
    'Vendor',
    'Admin'
);

--
-- Name: client_profiles; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.client_profiles (
    client_id uuid NOT NULL,
    first_name character varying(100),
    last_name character varying(100),
    phone character varying(50),
    wedding_date date,
    city_id uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);

--
-- Name: global_users; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.global_users (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    email character varying(255) NOT NULL,
    password_hash character varying(255) NOT NULL,
    domain_type public.user_domain_enum NOT NULL,
    token_valid_after timestamp with time zone DEFAULT now() NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    failed_login_attempts integer DEFAULT 0 NOT NULL,
    locked_until timestamp with time zone,
    scopes character varying(50)[] DEFAULT '{}'::character varying[] NOT NULL,
    display_name character varying(100),
    avatar_url character varying(255),
    is_system_account boolean DEFAULT false NOT NULL,
    CONSTRAINT global_users_email_lowercase_check CHECK (email = LOWER(email))
);

--
-- Name: user_notification_preferences; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.user_notification_preferences (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    email_notifications boolean DEFAULT true NOT NULL,
    in_app_notifications boolean DEFAULT true NOT NULL,
    marketing_notifications boolean DEFAULT false NOT NULL,
    booking_updates boolean DEFAULT true NOT NULL,
    budget_alerts boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now()
);

--
-- Name: fn_update_updated_at_column(); Type: FUNCTION; Schema: public; Owner: zafaf_db_admin
--

CREATE FUNCTION public.fn_update_updated_at_column() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$;

--
-- Name: touch_updated_at(); Type: FUNCTION; Schema: public; Owner: zafaf_schema_owner
--

CREATE FUNCTION public.touch_updated_at() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$;

--
-- Name: fn_normalize_email(); Type: FUNCTION; Schema: public; Owner: zafaf_schema_owner
--

CREATE FUNCTION public.fn_normalize_email() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    NEW.email := LOWER(TRIM(NEW.email));
    RETURN NEW;
END;
$$;

ALTER FUNCTION public.fn_normalize_email() OWNER TO zafaf_schema_owner;

-- Triggers for updated_at tracking
CREATE TRIGGER trg_global_users_updated_at
    BEFORE UPDATE ON public.global_users
    FOR EACH ROW
    EXECUTE FUNCTION public.touch_updated_at();

CREATE TRIGGER trg_client_profiles_updated_at
    BEFORE UPDATE ON public.client_profiles
    FOR EACH ROW
    EXECUTE FUNCTION public.touch_updated_at();

CREATE TRIGGER trg_user_notification_preferences_updated_at
    BEFORE UPDATE ON public.user_notification_preferences
    FOR EACH ROW
    EXECUTE FUNCTION public.touch_updated_at();

-- Trigger for email normalization
CREATE TRIGGER trg_normalize_email
    BEFORE INSERT OR UPDATE OF email ON public.global_users
    FOR EACH ROW
    EXECUTE FUNCTION public.fn_normalize_email();

--
-- Name: idx_client_profiles_city; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_client_profiles_city ON public.client_profiles USING btree (city_id);

--
-- Name: user_notification_preferences notification_preferences_isolation; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY notification_preferences_isolation ON public.user_notification_preferences USING (((user_id = (NULLIF(current_setting('app.current_user_id'::text, true), ''::text))::uuid) OR (current_setting('app.current_user_role'::text, true) = 'admin'::text)));

ALTER TABLE public.client_profiles OWNER TO zafaf_schema_owner;

ALTER TABLE public.global_users OWNER TO zafaf_schema_owner;

--
-- Name: client_profiles client_profiles_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.client_profiles
    ADD CONSTRAINT client_profiles_pkey PRIMARY KEY (client_id);

--
-- Name: global_users global_users_email_key; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.global_users
    ADD CONSTRAINT global_users_email_key UNIQUE (email);

--
-- Name: global_users global_users_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.global_users
    ADD CONSTRAINT global_users_pkey PRIMARY KEY (id);

CREATE INDEX IF NOT EXISTS idx_global_users_domain_type ON public.global_users(domain_type);
CREATE INDEX IF NOT EXISTS idx_global_users_created_at ON public.global_users(created_at DESC);

ALTER TABLE ONLY public.user_notification_preferences FORCE ROW LEVEL SECURITY;

ALTER TABLE public.user_notification_preferences OWNER TO zafaf_schema_owner;

--
-- Name: user_notification_preferences user_notification_preferences_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.user_notification_preferences
    ADD CONSTRAINT user_notification_preferences_pkey PRIMARY KEY (id);

--
-- Name: user_notification_preferences user_notification_preferences_user_id_key; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.user_notification_preferences
    ADD CONSTRAINT user_notification_preferences_user_id_key UNIQUE (user_id);

--
-- Name: client_profiles client_profiles_city_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.client_profiles
    ADD CONSTRAINT client_profiles_city_id_fkey FOREIGN KEY (city_id) REFERENCES public.cities(id) ON DELETE SET NULL;

--
-- Name: client_profiles client_profiles_client_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.client_profiles
    ADD CONSTRAINT client_profiles_client_id_fkey FOREIGN KEY (client_id) REFERENCES public.global_users(id) ON DELETE CASCADE;

--
-- Name: user_notification_preferences; Type: ROW SECURITY; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE public.user_notification_preferences ENABLE ROW LEVEL SECURITY;

--
-- Name: TABLE client_profiles; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.client_profiles TO app_client_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.client_profiles TO app_vendor_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.client_profiles TO app_admin_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.client_profiles TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.client_profiles TO zafaf_db_admin;

--
-- Name: TABLE global_users; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.global_users TO app_client_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.global_users TO app_vendor_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.global_users TO app_admin_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.global_users TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.global_users TO zafaf_db_admin;

--
-- Name: TABLE user_notification_preferences; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.user_notification_preferences TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.user_notification_preferences TO zafaf_db_admin;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.user_notification_preferences TO app_admin_role;

GRANT SELECT ON TABLE public.user_notification_preferences TO app_vendor_role;

GRANT SELECT ON TABLE public.user_notification_preferences TO app_client_role;

ALTER TYPE public.user_domain_enum OWNER TO zafaf_db_admin;

ALTER FUNCTION public.fn_update_updated_at_column() OWNER TO zafaf_db_admin;

ALTER FUNCTION public.touch_updated_at() OWNER TO zafaf_schema_owner;



-- ─── SEED DATA ───────────────────────────────────────────────────────────────

-- Admin seed account insertion removed (Bootstrapped securely via the 'bootstrap-admin' CLI command)

-- Retroactively grant 'super_admin' to Admins
UPDATE global_users SET scopes = '{"super_admin"}' WHERE domain_type = 'Admin';

-- Retroactively grant 'owner' to Vendors
UPDATE global_users SET scopes = '{"owner"}' WHERE domain_type = 'Vendor';

-- Seed Afrah system user if not exists
INSERT INTO global_users (id, email, password_hash, domain_type, display_name, avatar_url, is_system_account, scopes)
VALUES (
    '11111111-1111-1111-1111-111111111111'::uuid,
    'afrah@zafafworld.com',
    '$argon2id$v=19$m=19456,t=2,p=1$w2eXoY3K4yW/vK2bU+ZlOQ$l5b2pU1yG6L3bXvV4Z4Qx2K7J0W6Z5R0F6V2K5Z9X4', -- Dummy hash
    'Admin'::user_domain_enum,
    'Afrah',
    '/afrah_avatar.webp',
    TRUE,
    ARRAY['admin:all']::varchar[]
)
ON CONFLICT (email) DO UPDATE SET
    display_name = EXCLUDED.display_name,
    avatar_url = EXCLUDED.avatar_url,
    is_system_account = EXCLUDED.is_system_account;

-- Correct Afrah system user scope: 'admin:all' -> 'super_admin'
--
-- Migration 20260609221500 originally seeded scopes = ARRAY['admin:all'].
-- The correct scope token for the admin role is 'super_admin'.
-- This migration applies the correction as an idempotent UPDATE.
--
-- WHY A NEW MIGRATION: SQLx computes a SHA384 of each migration file at
-- runtime and compares it against the checksum stored in _sqlx_migrations.
-- Editing an already-applied migration file changes its SHA384 and causes
-- the backend to panic at startup with:
--   "migration was previously applied but has been modified"
-- The correct fix is always to write a NEW migration.

UPDATE global_users
SET scopes = ARRAY['super_admin']::varchar[]
WHERE id = '11111111-1111-1111-1111-111111111111'::uuid
  AND 'admin:all' = ANY(scopes);


-- ─── CENTRALIZED UPLOADED FILES ────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS public.uploaded_files (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bucket_name VARCHAR(63) NOT NULL,
    object_key VARCHAR(1024) NOT NULL UNIQUE,
    file_name VARCHAR(255) NOT NULL,
    file_size BIGINT NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'ready',
    error_message TEXT,
    uploaded_by UUID REFERENCES public.global_users(id) ON DELETE SET NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    CONSTRAINT check_upload_status CHECK (status IN ('processing', 'ready', 'failed'))
);

-- Note: Redundant index idx_uploaded_files_key removed as object_key is UNIQUE and has an implicit backing index.

ALTER TABLE public.uploaded_files OWNER TO zafaf_schema_owner;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.uploaded_files TO app_client_role;
GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.uploaded_files TO app_vendor_role;
GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.uploaded_files TO app_admin_role;
GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.uploaded_files TO zafaf_app_user;
GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.uploaded_files TO zafaf_db_admin;


COMMIT;
