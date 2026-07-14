-- ═══════════════════════════════════════════════════════════════════════════════
-- Baseline Migration 03: 0003_vendors_and_subscriptions.sql
-- Consolidated ZafafWorld Database Schema Baseline
-- ═══════════════════════════════════════════════════════════════════════════════

BEGIN;

--
-- Name: vendor_verification_level; Type: TYPE; Schema: public; Owner: zafaf_db_admin
--

CREATE TYPE public.vendor_verification_level AS ENUM (
    'basic',
    'verified',
    'premium_verified',
    'official_partner'
);

--
-- Name: subscription_tiers; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.subscription_tiers (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    name character varying(50) NOT NULL,
    priority_score integer NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    policy_limits jsonb DEFAULT '{}'::jsonb NOT NULL,
    price numeric(12,2) DEFAULT 0.00 NOT NULL,
    billing_cycle character varying(20) DEFAULT 'YEARLY'::character varying NOT NULL,
    features jsonb DEFAULT '[]'::jsonb NOT NULL
);

--
-- Name: vendor_subscription_requests; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.vendor_subscription_requests (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    vendor_id uuid NOT NULL,
    requested_tier_id uuid NOT NULL,
    status character varying(20) DEFAULT 'pending'::character varying NOT NULL,
    admin_notes text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);

--
-- Name: vendor_wallets; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.vendor_wallets (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    vendor_id uuid NOT NULL,
    available_balance numeric(12,2) DEFAULT 0.00 NOT NULL,
    pending_escrow numeric(12,2) DEFAULT 0.00 NOT NULL,
    lifetime_earnings numeric(12,2) DEFAULT 0.00 NOT NULL,
    updated_at timestamp with time zone DEFAULT now(),
    CONSTRAINT chk_wallet_balances CHECK (available_balance >= 0.00 AND pending_escrow >= 0.00 AND lifetime_earnings >= 0.00)
);

--
-- Name: vendors; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.vendors (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid,
    name_ar character varying(255) NOT NULL,
    name_en character varying(255) NOT NULL,
    slug character varying(255) NOT NULL,
    description_ar text,
    description_en text,
    category character varying(100),
    city_id uuid,
    address_ar character varying(255),
    address_en character varying(255),
    phone character varying(50),
    email character varying(255),
    website character varying(512),
    maps_url character varying(512),
    video_url_1 character varying(512),
    latitude double precision,
    longitude double precision,
    star_rating numeric(3,1),
    has_partition boolean DEFAULT false NOT NULL,
    capacity_min integer,
    capacity_max integer,
    amenities text[],
    coordinator_name_ar character varying(100),
    coordinator_name_en character varying(100),
    coordinator_phone character varying(50),
    coordinator_whatsapp character varying(50),
    coordinator_avatar character varying(512),
    crm_venue_id character varying(255),
    featured_expires_at timestamp with time zone,
    event_spaces_available integer,
    event_type character varying(100),
    status character varying(50) DEFAULT 'active'::character varying NOT NULL,
    subscription_status character varying(50) DEFAULT 'trial'::character varying NOT NULL,
    is_verified boolean DEFAULT false NOT NULL,
    is_featured boolean DEFAULT false NOT NULL,
    is_available boolean DEFAULT true NOT NULL,
    version integer DEFAULT 1 NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    verification_level public.vendor_verification_level DEFAULT 'basic'::public.vendor_verification_level NOT NULL,
    verification_updated_at timestamp with time zone,
    verification_notes text,
    is_solo_artist boolean DEFAULT false NOT NULL,
    instagram_url character varying(500),
    cr_number character varying(100),
    subscription_tier_id uuid,
    subscription_expires_at timestamp with time zone,
    CONSTRAINT chk_vendor_status CHECK (((status)::text = ANY ((ARRAY['active'::character varying, 'suspended'::character varying, 'banned'::character varying])::text[]))),
    CONSTRAINT chk_vendor_capacity CHECK (capacity_min >= 0 AND capacity_max >= capacity_min),
    CONSTRAINT chk_vendor_coordinates CHECK (latitude BETWEEN -90.0 AND 90.0 AND longitude BETWEEN -180.0 AND 180.0),
    CONSTRAINT chk_vendor_star_rating CHECK (star_rating BETWEEN 0.0 AND 5.0)
);

--
-- Name: cascade_vendor_suspension(); Type: FUNCTION; Schema: public; Owner: zafaf_schema_owner
--

CREATE FUNCTION public.cascade_vendor_suspension() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF NEW.status = 'suspended' AND OLD.status != 'suspended' THEN
        UPDATE vendor_products
        SET pre_suspension_status = status,
            status = 'suspended',
            updated_at = NOW()
        WHERE vendor_id = NEW.id
          AND status IN ('active', 'pending_approval', 'draft');
    END IF;

    IF NEW.status = 'active' AND OLD.status = 'suspended' THEN
        UPDATE vendor_products
        SET status = COALESCE(pre_suspension_status, 'draft'),
            pre_suspension_status = NULL,
            updated_at = NOW()
        WHERE vendor_id = NEW.id
          AND status = 'suspended'
          AND pre_suspension_status IS NOT NULL;
    END IF;

    IF NEW.status = 'banned' THEN
        UPDATE vendor_products
        SET status = 'archived',
            updated_at = NOW()
        WHERE vendor_id = NEW.id
          AND status != 'archived';
    END IF;

    RETURN NEW;
END;
$$;

--
-- Name: idx_vendor_sub_req_status; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vendor_sub_req_status ON public.vendor_subscription_requests USING btree (status);

--
-- Name: idx_vendor_sub_req_vendor_id; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vendor_sub_req_vendor_id ON public.vendor_subscription_requests USING btree (vendor_id);

--
-- Name: idx_vendors_amenities_gin; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vendors_amenities_gin ON public.vendors USING gin (amenities);

--
-- Name: idx_vendors_category; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vendors_category ON public.vendors USING btree (category);

--
-- Name: idx_vendors_city; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vendors_city ON public.vendors USING btree (city_id);

--
-- Name: idx_vendors_created_at; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vendors_created_at ON public.vendors USING btree (created_at DESC);

--
-- Name: idx_vendors_desc_ar_gin; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vendors_desc_ar_gin ON public.vendors USING gin (description_ar public.gin_trgm_ops);

--
-- Name: idx_vendors_desc_en_gin; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vendors_desc_en_gin ON public.vendors USING gin (description_en public.gin_trgm_ops);

--
-- Name: idx_vendors_name_ar_gin; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vendors_name_ar_gin ON public.vendors USING gin (name_ar public.gin_trgm_ops);

--
-- Name: idx_vendors_name_en_gin; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vendors_name_en_gin ON public.vendors USING gin (name_en public.gin_trgm_ops);

--
-- Name: idx_vendors_status; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vendors_status ON public.vendors USING btree (status);

--
-- Name: idx_vendors_verification; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vendors_verification ON public.vendors USING btree (verification_level);

CREATE INDEX idx_vendors_subscription_tier ON public.vendors USING btree (subscription_tier_id);

--
-- Name: subscription_tiers catalog_modify_tiers; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY catalog_modify_tiers ON public.subscription_tiers USING ((current_setting('app.current_user_role'::text, true) = 'admin'::text));

--
-- Name: subscription_tiers catalog_select_tiers; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY catalog_select_tiers ON public.subscription_tiers FOR SELECT USING (true);

--
-- Name: vendor_wallets vendor_wallets_isolation_policy; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY vendor_wallets_isolation_policy ON public.vendor_wallets USING (((vendor_id = (NULLIF(current_setting('app.current_user_id'::text, true), ''::text))::uuid) OR (current_setting('app.current_user_role'::text, true) = 'admin'::text)));

--
-- Name: vendors trg_cascade_vendor_suspension; Type: TRIGGER; Schema: public; Owner: zafaf_schema_owner
--

CREATE TRIGGER trg_cascade_vendor_suspension AFTER UPDATE OF status ON public.vendors FOR EACH ROW EXECUTE FUNCTION public.cascade_vendor_suspension();

CREATE TRIGGER trg_vendors_updated_at BEFORE UPDATE ON public.vendors FOR EACH ROW EXECUTE FUNCTION public.touch_updated_at();

CREATE TRIGGER trg_subscription_tiers_updated_at BEFORE UPDATE ON public.subscription_tiers FOR EACH ROW EXECUTE FUNCTION public.touch_updated_at();

CREATE TRIGGER trg_vendor_wallets_updated_at BEFORE UPDATE ON public.vendor_wallets FOR EACH ROW EXECUTE FUNCTION public.touch_updated_at();

--
-- Name: vendor_subscription_requests trg_vendor_sub_req_updated_at; Type: TRIGGER; Schema: public; Owner: zafaf_schema_owner
--

CREATE TRIGGER trg_vendor_sub_req_updated_at BEFORE UPDATE ON public.vendor_subscription_requests FOR EACH ROW EXECUTE FUNCTION public.touch_updated_at();

ALTER TABLE ONLY public.subscription_tiers FORCE ROW LEVEL SECURITY;

ALTER TABLE public.subscription_tiers OWNER TO zafaf_schema_owner;

ALTER TABLE public.vendor_subscription_requests OWNER TO zafaf_schema_owner;

ALTER TABLE ONLY public.vendor_wallets FORCE ROW LEVEL SECURITY;

ALTER TABLE public.vendor_wallets OWNER TO zafaf_schema_owner;

ALTER TABLE public.vendors OWNER TO zafaf_schema_owner;

--
-- Name: subscription_tiers subscription_tiers_name_key; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.subscription_tiers
    ADD CONSTRAINT subscription_tiers_name_key UNIQUE (name);

--
-- Name: subscription_tiers subscription_tiers_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.subscription_tiers
    ADD CONSTRAINT subscription_tiers_pkey PRIMARY KEY (id);

--
-- Name: vendor_subscription_requests vendor_subscription_requests_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.vendor_subscription_requests
    ADD CONSTRAINT vendor_subscription_requests_pkey PRIMARY KEY (id);

--
-- Name: vendor_wallets vendor_wallets_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.vendor_wallets
    ADD CONSTRAINT vendor_wallets_pkey PRIMARY KEY (id);

--
-- Name: vendor_wallets vendor_wallets_vendor_id_key; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.vendor_wallets
    ADD CONSTRAINT vendor_wallets_vendor_id_key UNIQUE (vendor_id);

--
-- Name: vendors vendors_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.vendors
    ADD CONSTRAINT vendors_pkey PRIMARY KEY (id);

--
-- Name: vendors vendors_slug_key; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.vendors
    ADD CONSTRAINT vendors_slug_key UNIQUE (slug);

--
-- Name: vendors vendors_user_id_key; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.vendors
    ADD CONSTRAINT vendors_user_id_key UNIQUE (user_id);

--
-- Name: subscription_tiers; Type: ROW SECURITY; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE public.subscription_tiers ENABLE ROW LEVEL SECURITY;

--
-- Name: vendor_wallets; Type: ROW SECURITY; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE public.vendor_wallets ENABLE ROW LEVEL SECURITY;

--
-- Name: vendor_subscription_requests vendor_subscription_requests_requested_tier_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.vendor_subscription_requests
    ADD CONSTRAINT vendor_subscription_requests_requested_tier_id_fkey FOREIGN KEY (requested_tier_id) REFERENCES public.subscription_tiers(id) ON DELETE CASCADE;

--
-- Name: vendor_subscription_requests vendor_subscription_requests_vendor_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.vendor_subscription_requests
    ADD CONSTRAINT vendor_subscription_requests_vendor_id_fkey FOREIGN KEY (vendor_id) REFERENCES public.vendors(id) ON DELETE CASCADE;

--
-- Name: vendors vendors_city_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.vendors
    ADD CONSTRAINT vendors_city_id_fkey FOREIGN KEY (city_id) REFERENCES public.cities(id) ON DELETE SET NULL;

--
-- Name: vendors vendors_subscription_tier_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.vendors
    ADD CONSTRAINT vendors_subscription_tier_id_fkey FOREIGN KEY (subscription_tier_id) REFERENCES public.subscription_tiers(id) ON DELETE SET NULL;

--
-- Name: vendors vendors_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.vendors
    ADD CONSTRAINT vendors_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.global_users(id) ON DELETE SET NULL;

--
-- Name: COLUMN vendors.verification_level; Type: COMMENT; Schema: public; Owner: zafaf_schema_owner
--

COMMENT ON COLUMN public.vendors.verification_level IS 'Trust tier: basic (unverified) → verified → premium_verified → official_partner';

--
-- Name: COLUMN vendors.is_solo_artist; Type: COMMENT; Schema: public; Owner: zafaf_schema_owner
--

COMMENT ON COLUMN public.vendors.is_solo_artist IS 'If true, vendor is exempt from CR number requirement. Uses Instagram + phone verification track.';

--
-- Name: TABLE subscription_tiers; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.subscription_tiers TO app_admin_role;

GRANT SELECT ON TABLE public.subscription_tiers TO app_vendor_role;

GRANT SELECT ON TABLE public.subscription_tiers TO app_client_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.subscription_tiers TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.subscription_tiers TO zafaf_db_admin;

--
-- Name: TABLE vendor_subscription_requests; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_subscription_requests TO app_vendor_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_subscription_requests TO app_admin_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_subscription_requests TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_subscription_requests TO zafaf_db_admin;

GRANT SELECT ON TABLE public.vendor_subscription_requests TO app_client_role;

--
-- Name: TABLE vendor_wallets; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_wallets TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_wallets TO zafaf_db_admin;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_wallets TO app_admin_role;

GRANT SELECT ON TABLE public.vendor_wallets TO app_vendor_role;

GRANT SELECT ON TABLE public.vendor_wallets TO app_client_role;

--
-- Name: TABLE vendors; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendors TO app_client_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendors TO app_vendor_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendors TO app_admin_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendors TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendors TO zafaf_db_admin;

ALTER TYPE public.vendor_verification_level OWNER TO zafaf_db_admin;

ALTER FUNCTION public.cascade_vendor_suspension() OWNER TO zafaf_schema_owner;



-- ─── SEED DATA ───────────────────────────────────────────────────────────────

INSERT INTO subscription_tiers (name, priority_score) VALUES 
('Diamond', 100),
('VIP', 75),
('Gold', 50),
('Free', 25)
ON CONFLICT (name) DO UPDATE SET priority_score = EXCLUDED.priority_score;

UPDATE vendors 
SET subscription_tier_id = (SELECT id FROM subscription_tiers WHERE name = 'Free')
WHERE subscription_tier_id IS NULL;

-- 2. Seed limits based on business rules
-- Free Plan: 1 Product, 1 Cover Photo, 1 Additional Photo, 0 Videos, 1 Description Block
UPDATE subscription_tiers 
SET policy_limits = '{"max_products": 1, "max_cover_photos": 1, "max_additional_photos": 1, "max_videos": 0, "max_description_blocks": 1}'::jsonb 
WHERE name ILIKE 'free';

-- Gold Plan: 5 Products, 1 Cover Photo, 15 Additional Photos, 1 Video, 5 Description Blocks
UPDATE subscription_tiers 
SET policy_limits = '{"max_products": 5, "max_cover_photos": 1, "max_additional_photos": 15, "max_videos": 1, "max_description_blocks": 5}'::jsonb 
WHERE name ILIKE 'gold';

-- VIP Plan: 15 Products, 1 Cover Photo, 30 Additional Photos, 10 Videos, 10 Description Blocks
UPDATE subscription_tiers 
SET policy_limits = '{"max_products": 15, "max_cover_photos": 1, "max_additional_photos": 30, "max_videos": 10, "max_description_blocks": 10}'::jsonb 
WHERE name ILIKE 'vip';

-- Diamond Plan: 50 Products, 1 Cover Photo, Unlimited (-1) Photos, Unlimited (-1) Videos, 15 Description Blocks
UPDATE subscription_tiers 
SET policy_limits = '{"max_products": 50, "max_cover_photos": 1, "max_additional_photos": -1, "max_videos": -1, "max_description_blocks": 15}'::jsonb 
WHERE name ILIKE 'diamond';

-- 2. Seed Dynamic Features for each Tier

-- Free Tier
UPDATE subscription_tiers
SET 
    price = 0.00,
    features = '[
        "Search Visibility: Standard",
        "Photo Limit: 15 Pictures",
        "Video Limit: 0 Videos"
    ]'::jsonb
WHERE name ILIKE 'free';

-- Gold Tier
UPDATE subscription_tiers
SET 
    price = 20000.00,
    features = '[
        "Search Visibility: Above Free",
        "Photo Limit: 15 Pictures",
        "Video Limit: 1 Videos",
        "No Ads on your page",
        "Featured Sponsorship: City (1 Month)",
        "SEO Profile, Email/SMS, App Access"
    ]'::jsonb
WHERE name ILIKE 'gold';

-- VIP Tier
UPDATE subscription_tiers
SET 
    price = 35000.00,
    features = '[
        "Search Visibility: Above Gold",
        "Photo Limit: 30 Pictures",
        "Video Limit: 10 Videos",
        "No Ads on your page",
        "Featured Sponsorship: City (3 Month)",
        "All Gold perks + Google Ads, 6 Social Posts"
    ]'::jsonb
WHERE name ILIKE 'vip';

-- Diamond Tier
UPDATE subscription_tiers
SET 
    price = 70000.00,
    features = '[
        "Search Visibility: Top placement",
        "Photo Limit: Unlimited",
        "Video Limit: Unlimited",
        "No Ads on your page",
        "Featured Sponsorship: City (3 Month) + Category",
        "All VIP perks + 2 Snap Posts, Insta Story, Popups, Homepage Showcase"
    ]'::jsonb
WHERE name ILIKE 'diamond';

-- Free: 0 promotions (must upgrade to use promotions)
UPDATE subscription_tiers
SET policy_limits = policy_limits || '{"max_promotions": 0}'::jsonb
WHERE name ILIKE 'free';

-- Gold: 2 active/pending promotions at a time
UPDATE subscription_tiers
SET policy_limits = policy_limits || '{"max_promotions": 2}'::jsonb
WHERE name ILIKE 'gold';

-- VIP: 10 active/pending promotions at a time
UPDATE subscription_tiers
SET policy_limits = policy_limits || '{"max_promotions": 10}'::jsonb
WHERE name ILIKE 'vip';

-- Diamond: Unlimited promotions (-1 = unlimited)
UPDATE subscription_tiers
SET policy_limits = policy_limits || '{"max_promotions": -1}'::jsonb
WHERE name ILIKE 'diamond';

COMMIT;
