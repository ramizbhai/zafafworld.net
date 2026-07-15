-- ═══════════════════════════════════════════════════════════════════════════════
-- Baseline Migration 05: 0005_listings_and_gallery.sql
-- Consolidated ZafafWorld Database Schema Baseline
-- ═══════════════════════════════════════════════════════════════════════════════


--
-- Name: reviews; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

-- Dynamic reviews view replaces the physical table to consolidate reviews while maintaining code compatibility.

--
-- Name: vendor_gallery; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.vendor_gallery (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    vendor_id uuid NOT NULL,
    product_id uuid,
    image_url character varying(512) NOT NULL,
    file_path character varying(512),
    is_cover boolean DEFAULT false NOT NULL,
    sort_order integer DEFAULT 0 NOT NULL,
    caption character varying(255),
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    media_type character varying(50) DEFAULT 'image'::character varying NOT NULL,
    file_url character varying(512) NOT NULL,
    thumbnail_url character varying(512),
    file_size bigint,
    duration_seconds integer,
    CONSTRAINT vendor_gallery_media_type_check CHECK (((media_type)::text = ANY ((ARRAY['image'::character varying, 'video'::character varying])::text[])))
);

--
-- Name: vendor_packages; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.vendor_packages (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    vendor_id uuid NOT NULL,
    product_id uuid NOT NULL,
    name_ar character varying(255) NOT NULL,
    name_en character varying(255) NOT NULL,
    description_ar text,
    description_en text,
    original_price numeric(12,2) NOT NULL,
    discounted_price numeric(12,2) NOT NULL,
    is_zafaf_exclusive boolean DEFAULT false NOT NULL,
    expiry_date date NOT NULL,
    is_active boolean DEFAULT true NOT NULL,
    sort_order integer DEFAULT 0 NOT NULL,
    version integer DEFAULT 1 NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT chk_pkg_discounted_price CHECK ((discounted_price > (0)::numeric)),
    CONSTRAINT chk_pkg_expiry CHECK ((expiry_date >= CURRENT_DATE)),
    CONSTRAINT chk_pkg_original_price CHECK ((original_price > (0)::numeric)),
    CONSTRAINT chk_pkg_price_order CHECK ((discounted_price < original_price))
);

--
-- Name: vendor_products; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.vendor_products (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    vendor_id uuid NOT NULL,
    slug character varying(255) NOT NULL,
    product_category character varying(100) DEFAULT 'wedding_hall'::character varying NOT NULL,
    base_price_sar numeric(12,2),
    deposit_percentage integer DEFAULT 25 NOT NULL,
    coordinator_phone character varying(50),
    coordinator_whatsapp character varying(50),
    coordinator_avatar character varying(512),
    coordinator_gender character varying(10) DEFAULT 'any'::character varying NOT NULL,
    crm_product_id character varying(255),
    city_id uuid,
    status character varying(50) DEFAULT 'draft'::character varying NOT NULL,
    rejection_reason text,
    pre_suspension_status character varying(50),
    is_available boolean DEFAULT true NOT NULL,
    is_featured boolean DEFAULT false NOT NULL,
    version integer DEFAULT 1 NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    title character varying(255) NOT NULL,
    description text,
    coordinator_name character varying(100),
    attributes jsonb DEFAULT '{}'::jsonb NOT NULL,
    featured_until timestamp with time zone,
    title_ar character varying(255),
    title_en character varying(255),
    description_ar text,
    description_en text,
    coordinator_name_ar character varying(100),
    coordinator_name_en character varying(100),
    gender_section character varying(30),
    total_capacity integer,
    searchable_amenities text[] DEFAULT ARRAY[]::text[] NOT NULL,
    quality_score integer DEFAULT 0 NOT NULL,
    price_on_inquiry boolean DEFAULT false NOT NULL,
    published_at timestamp with time zone,
    google_maps_url character varying(512),
    latitude double precision,
    longitude double precision,
    features_selection jsonb DEFAULT '{}'::jsonb NOT NULL,
    meta_title_ar character varying(255),
    meta_title_en character varying(255),
    meta_description_ar text,
    meta_description_en text,
    coordinator_email character varying(255),
    coordinator_mobile character varying(50),
    cultural_attributes jsonb DEFAULT '{}'::jsonb NOT NULL,
    CONSTRAINT chk_base_price CHECK (((base_price_sar IS NULL) OR (base_price_sar > (0)::numeric))),
    CONSTRAINT chk_coordinator_gender CHECK (((coordinator_gender)::text = ANY ((ARRAY['male'::character varying, 'female'::character varying, 'any'::character varying])::text[]))),
    CONSTRAINT chk_deposit_pct CHECK (((deposit_percentage >= 10) AND (deposit_percentage <= 100))),
    CONSTRAINT chk_gender_section CHECK (((gender_section IS NULL) OR ((gender_section)::text = ANY ((ARRAY['women_only'::character varying, 'men_only'::character varying, 'mixed'::character varying, 'dual_parallel'::character varying, 'family'::character varying, 'both_sections'::character varying, 'not_applicable'::character varying])::text[])))),
    CONSTRAINT chk_product_category CHECK (((product_category)::text = ANY ((ARRAY['wedding-palace'::character varying, 'hotel-venue'::character varying, 'villa-resort'::character varying, 'restaurant-event'::character varying, 'outdoor-garden'::character varying, 'rooftop-venue'::character varying, 'private-beach'::character varying, 'chalet'::character varying, 'wedding-gown'::character varying, 'haute-couture'::character varying, 'abaya-jalabiya'::character varying, 'groom-attire'::character varying, 'hair-makeup'::character varying, 'henna-art'::character varying, 'photography-video'::character varying, 'photo-studio'::character varying, 'catering'::character varying, 'wedding-cake'::character varying, 'wedding-sweets'::character varying, 'entertainment-dj'::character varying, 'wedding-jewelry'::character varying, 'wedding-gifts'::character varying, 'wedding-planner'::character varying, 'flowers-floral'::character varying, 'wedding-invitation'::character varying, 'lighting-av'::character varying, 'wedding-car'::character varying])::text[]))),
    CONSTRAINT chk_product_status CHECK (((status)::text = ANY ((ARRAY['draft'::character varying, 'pending_approval'::character varying, 'active'::character varying, 'rejected'::character varying, 'suspended'::character varying, 'archived'::character varying])::text[]))),
    CONSTRAINT chk_quality_score CHECK (((quality_score >= 0) AND (quality_score <= 100)))
);

--
-- Name: vendor_review_attachments; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.vendor_review_attachments (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    review_id uuid NOT NULL,
    file_path character varying(512) NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);

--
-- Name: vendor_reviews; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.vendor_reviews (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    client_id uuid NOT NULL,
    vendor_id uuid NOT NULL,
    rating integer NOT NULL,
    review_text text NOT NULL,
    status character varying(50) DEFAULT 'pending_approval'::character varying NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT chk_vendor_reviews_status CHECK (((status)::text = ANY ((ARRAY['pending_approval'::character varying, 'approved'::character varying, 'rejected'::character varying])::text[]))),
    CONSTRAINT vendor_reviews_rating_check CHECK (((rating >= 1) AND (rating <= 5)))
);

-- View reviews wraps vendor_reviews to maintain compat with legacy queries
CREATE VIEW public.reviews AS
SELECT 
    r.id,
    v.id AS vendor_id,
    r.client_id AS user_id,
    COALESCE(cp.first_name || ' ' || cp.last_name, 'User') AS author_name,
    r.created_at::date AS wedding_date,
    r.rating AS rating_quality,
    r.rating AS rating_staff,
    r.rating AS rating_communication,
    r.review_text AS comment,
    CASE WHEN r.status = 'pending_approval' THEN 'pending'::character varying ELSE r.status END AS status,
    r.created_at
FROM public.vendor_reviews r
JOIN public.vendors v ON r.vendor_id = v.user_id
LEFT JOIN public.client_profiles cp ON r.client_id = cp.client_id;

ALTER VIEW public.reviews OWNER TO zafaf_schema_owner;

--
-- Name: apply_gallery_quality_bonus(uuid); Type: FUNCTION; Schema: public; Owner: zafaf_schema_owner
--

CREATE FUNCTION public.apply_gallery_quality_bonus(p_product_id uuid) RETURNS void
    LANGUAGE plpgsql
    AS $$
DECLARE
    img_count INT;
    current_score INT;
BEGIN
    SELECT COUNT(*) INTO img_count
    FROM public.vendor_gallery
    WHERE product_id = p_product_id;

    SELECT quality_score INTO current_score
    FROM vendor_products
    WHERE id = p_product_id;

    -- Add +10 if 3+ images and score is below 100 (and doesn't already have bonus)
    IF img_count >= 3 AND current_score < 100 THEN
        UPDATE vendor_products
        SET quality_score = LEAST(quality_score + 10, 100)
        WHERE id = p_product_id
          AND quality_score < 100;
    END IF;
END;
$$;

--
-- Name: compute_listing_quality_score(text, text, text, text, text, uuid, numeric, boolean, text, text, jsonb); Type: FUNCTION; Schema: public; Owner: zafaf_schema_owner
--

CREATE FUNCTION public.compute_listing_quality_score(p_title_ar text, p_title_en text, p_description_ar text, p_description_en text, p_gender_section text, p_city_id uuid, p_base_price_sar numeric, p_price_on_inquiry boolean, p_coordinator_name_en text, p_coordinator_phone text, p_attributes jsonb) RETURNS integer
    LANGUAGE plpgsql IMMUTABLE
    AS $$
DECLARE
    score INT := 0;
BEGIN
    -- Arabic title (15 pts)
    IF p_title_ar IS NOT NULL AND length(trim(p_title_ar)) >= 3 THEN
        score := score + 15;
    END IF;

    -- English title (10 pts)
    IF p_title_en IS NOT NULL AND length(trim(p_title_en)) >= 3 THEN
        score := score + 10;
    END IF;

    -- Arabic description (20 pts for >= 100 chars, 10 pts for >= 30 chars)
    IF p_description_ar IS NOT NULL AND length(trim(p_description_ar)) >= 100 THEN
        score := score + 20;
    ELSIF p_description_ar IS NOT NULL AND length(trim(p_description_ar)) >= 30 THEN
        score := score + 10;
    END IF;

    -- English description (10 pts for >= 80 chars)
    IF p_description_en IS NOT NULL AND length(trim(p_description_en)) >= 80 THEN
        score := score + 10;
    END IF;

    -- Gender section set (10 pts) — critical GCC field
    IF p_gender_section IS NOT NULL AND p_gender_section != '' THEN
        score := score + 10;
    END IF;

    -- City set (10 pts)
    IF p_city_id IS NOT NULL THEN
        score := score + 10;
    END IF;

    -- Price set (10 pts) — price OR price_on_inquiry must be true
    IF p_base_price_sar IS NOT NULL OR p_price_on_inquiry = TRUE THEN
        score := score + 10;
    END IF;

    -- Coordinator info (5 pts)
    IF p_coordinator_name_en IS NOT NULL AND p_coordinator_phone IS NOT NULL THEN
        score := score + 5;
    END IF;

    -- Category attributes filled (10 pts if JSONB has >= 3 meaningful keys)
    IF p_attributes IS NOT NULL AND jsonb_typeof(p_attributes) = 'object' AND
       (SELECT count(*) FROM jsonb_object_keys(p_attributes)) >= 3 THEN
        score := score + 10;
    END IF;

    -- NOTE: Gallery photo count (+10 pts for 3+ images) is applied
    -- in the application layer after upload, by a separate UPDATE.
    -- It cannot be computed here without a subquery on vendor_product_images
    -- which would make this function non-IMMUTABLE.

    RETURN LEAST(score, 90);  -- max 90 from this function; +10 from gallery applied separately
END;
$$;

--
-- Name: compute_search_rank(integer, public.vendor_verification_level, numeric, timestamp with time zone, boolean); Type: FUNCTION; Schema: public; Owner: zafaf_schema_owner
--

CREATE FUNCTION public.compute_search_rank(p_quality_score integer, p_verification_level public.vendor_verification_level, p_avg_rating numeric, p_updated_at timestamp with time zone, p_is_featured boolean) RETURNS numeric
    LANGUAGE plpgsql IMMUTABLE
    AS $$
DECLARE
    verification_boost NUMERIC := 0;
    freshness_boost    NUMERIC := 1.0;
    review_boost       NUMERIC := 1.0;
    days_old           INT;
BEGIN
    -- Verification boost
    CASE p_verification_level
        WHEN 'verified'         THEN verification_boost := 0.10;
        WHEN 'premium_verified' THEN verification_boost := 0.25;
        WHEN 'official_partner' THEN verification_boost := 0.50;
        ELSE verification_boost := 0.0;
    END CASE;

    -- Freshness boost (based on last update)
    days_old := EXTRACT(EPOCH FROM (NOW() - p_updated_at)) / 86400;
    IF days_old <= 30 THEN
        freshness_boost := 1.0;
    ELSIF days_old <= 90 THEN
        freshness_boost := 0.85;
    ELSE
        freshness_boost := 0.70;
    END IF;

    -- Review boost (rating 1–5 maps to 0.8–1.2)
    IF p_avg_rating IS NOT NULL AND p_avg_rating > 0 THEN
        review_boost := 1.0 + (p_avg_rating - 3.0) * 0.10;
        review_boost := GREATEST(0.8, LEAST(1.2, review_boost));
    END IF;

    -- Featured listings get a fixed 100-point pin bonus applied in SQL ORDER BY
    RETURN ROUND(
        (p_quality_score::NUMERIC / 100.0)
        * (1.0 + verification_boost)
        * freshness_boost
        * review_boost
    , 4);
END;
$$;

--
-- Name: sync_product_search_fields(); Type: FUNCTION; Schema: public; Owner: zafaf_db_admin
--

CREATE FUNCTION public.sync_product_search_fields() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE
    v_capacity_mode  VARCHAR(20);
    v_capacity_key   VARCHAR(100);
    v_men_cap        INT;
    v_women_cap      INT;
    v_single_cap     INT;
    v_amenities      TEXT[] := ARRAY[]::TEXT[];
    v_merged_attrs   JSONB;
BEGIN
    -- ── Read capacity_mode AND capacity_key from category_schemas ─────────────
    -- This is the only place category config is read. No slug conditions here.
    SELECT capacity_mode, capacity_key
    INTO v_capacity_mode, v_capacity_key
    FROM category_schemas
    WHERE category_slug = NEW.product_category;

    v_capacity_mode := COALESCE(v_capacity_mode, 'none');

    -- ── Compute total_capacity ────────────────────────────────────────────────
    CASE v_capacity_mode

        WHEN 'sum_men_women' THEN
            -- capacity_key = 'men_capacity,women_capacity'
            -- Split on comma to get each field name independently.
            v_men_cap   := (NEW.attributes->>(split_part(v_capacity_key, ',', 1)))::int;
            v_women_cap := (NEW.attributes->>(split_part(v_capacity_key, ',', 2)))::int;
            IF v_men_cap IS NOT NULL AND v_women_cap IS NOT NULL THEN
                NEW.total_capacity := v_men_cap + v_women_cap;
            ELSE
                NEW.total_capacity := NULL;
            END IF;

        WHEN 'single_key' THEN
            -- capacity_key = exact field name (e.g. 'ballroom_capacity')
            -- The trigger does NOT know which category this is.
            -- It simply reads whatever key the DB config says.
            IF v_capacity_key IS NOT NULL THEN
                v_single_cap := (NEW.attributes->>v_capacity_key)::int;
                NEW.total_capacity := v_single_cap;
            ELSE
                NEW.total_capacity := NULL;
            END IF;

        ELSE
            -- 'none' or unrecognised: capacity not applicable for this category
            NEW.total_capacity := NULL;

    END CASE;

    -- ── Compute searchable_amenities from BOTH attributes columns ─────────────
    -- Merge cultural_attributes on top of attributes.
    -- cultural_attributes wins on key conflict (it is the authoritative GCC store).
    v_merged_attrs := COALESCE(NEW.attributes, '{}'::jsonb)
                   || COALESCE(NEW.cultural_attributes, '{}'::jsonb);

    -- ── Venue / GCC cultural amenities ───────────────────────────────────────
    IF (v_merged_attrs->>'prayer_room')::boolean = TRUE
    OR (v_merged_attrs->>'prayer_room_available')::boolean = TRUE THEN
        v_amenities := array_append(v_amenities, 'prayer_room');
    END IF;

    IF (v_merged_attrs->>'valet_parking')::boolean = TRUE THEN
        v_amenities := array_append(v_amenities, 'valet_parking');
    END IF;

    IF (v_merged_attrs->>'bridal_suite')::boolean = TRUE THEN
        v_amenities := array_append(v_amenities, 'bridal_suite');
    END IF;

    IF (v_merged_attrs->>'outdoor_garden')::boolean = TRUE THEN
        v_amenities := array_append(v_amenities, 'outdoor_garden');
    END IF;

    IF (v_merged_attrs->>'external_catering_allowed')::boolean = TRUE THEN
        v_amenities := array_append(v_amenities, 'external_catering_allowed');
    END IF;

    IF (v_merged_attrs->>'halal_certified')::boolean = TRUE
    OR (v_merged_attrs->>'halal_kitchen')::boolean = TRUE THEN
        v_amenities := array_append(v_amenities, 'halal_certified');
    END IF;

    IF (v_merged_attrs->>'private_pool')::boolean = TRUE
    OR (v_merged_attrs->>'has_private_pool')::boolean = TRUE THEN
        v_amenities := array_append(v_amenities, 'private_pool');
    END IF;

    -- ── Service provider amenities ────────────────────────────────────────────
    IF (v_merged_attrs->>'female_staff_only')::boolean = TRUE
    OR (v_merged_attrs->>'female_only_staff')::boolean = TRUE
    OR (v_merged_attrs->>'female_team_available')::boolean = TRUE THEN
        v_amenities := array_append(v_amenities, 'female_staff');
    END IF;

    IF (v_merged_attrs->>'female_only_establishment')::boolean = TRUE THEN
        v_amenities := array_append(v_amenities, 'female_only_establishment');
    END IF;

    IF (v_merged_attrs->>'home_service')::boolean = TRUE
    OR (v_merged_attrs->>'home_service_available')::boolean = TRUE
    OR (v_merged_attrs->>'at_home_service')::boolean = TRUE THEN
        v_amenities := array_append(v_amenities, 'home_service');
    END IF;

    IF (v_merged_attrs->>'drone_available')::boolean = TRUE THEN
        v_amenities := array_append(v_amenities, 'drone');
    END IF;

    IF (v_merged_attrs->>'natural_henna_only')::boolean = TRUE THEN
        v_amenities := array_append(v_amenities, 'natural_henna');
    END IF;

    IF (v_merged_attrs->>'female_artist')::boolean = TRUE THEN
        v_amenities := array_append(v_amenities, 'female_artist');
    END IF;

    IF (v_merged_attrs->>'women_only_events')::boolean = TRUE
    OR (v_merged_attrs->>'female_only_event')::boolean = TRUE THEN
        v_amenities := array_append(v_amenities, 'women_only_events');
    END IF;

    -- ── Entertainment amenities ───────────────────────────────────────────────
    IF (v_merged_attrs->>'sound_system_included')::boolean = TRUE THEN
        v_amenities := array_append(v_amenities, 'sound_system');
    END IF;

    -- ── Food / catering amenities ─────────────────────────────────────────────
    IF (v_merged_attrs->>'serving_staff_included')::boolean = TRUE THEN
        v_amenities := array_append(v_amenities, 'serving_staff');
    END IF;

    IF (v_merged_attrs->>'taste_testing')::boolean = TRUE
    OR (v_merged_attrs->>'tasting_session')::boolean = TRUE THEN
        v_amenities := array_append(v_amenities, 'tasting_available');
    END IF;

    IF (v_merged_attrs->>'delivery_to_venue')::boolean = TRUE
    OR (v_merged_attrs->>'delivery_available')::boolean = TRUE
    OR (v_merged_attrs->>'delivery_setup')::boolean = TRUE THEN
        v_amenities := array_append(v_amenities, 'delivery_available');
    END IF;

    NEW.searchable_amenities := v_amenities;

    RETURN NEW;
END;
$$;

--
-- Name: update_listing_quality_score(); Type: FUNCTION; Schema: public; Owner: zafaf_schema_owner
--

CREATE FUNCTION public.update_listing_quality_score() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    NEW.quality_score := compute_listing_quality_score(
        NEW.title_ar, NEW.title_en,
        NEW.description_ar, NEW.description_en,
        NEW.gender_section, NEW.city_id,
        NEW.base_price_sar, NEW.price_on_inquiry,
        NEW.coordinator_name_en, NEW.coordinator_phone,
        NEW.attributes
    );
    RETURN NEW;
END;
$$;

--
-- Name: idx_gallery_one_brand_cover_per_vendor; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE UNIQUE INDEX idx_gallery_one_brand_cover_per_vendor ON public.vendor_gallery USING btree (vendor_id) WHERE ((is_cover = true) AND (product_id IS NULL));

--
-- Name: idx_gallery_one_cover_per_product; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE UNIQUE INDEX idx_gallery_one_cover_per_product ON public.vendor_gallery USING btree (product_id) WHERE ((is_cover = true) AND (product_id IS NOT NULL));

--
-- Name: idx_packages_active; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_packages_active ON public.vendor_packages USING btree (is_active, expiry_date) WHERE (is_active = true);

--
-- Name: idx_packages_exclusive; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_packages_exclusive ON public.vendor_packages USING btree (is_zafaf_exclusive) WHERE (is_zafaf_exclusive = true);

--
-- Name: idx_reviews_vendor; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_reviews_vendor ON public.vendor_reviews USING btree (vendor_id);

--
-- Name: idx_vendor_gallery_product; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vendor_gallery_product ON public.vendor_gallery USING btree (product_id) WHERE (product_id IS NOT NULL);

--
-- Name: idx_vendor_gallery_vendor; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vendor_gallery_vendor ON public.vendor_gallery USING btree (vendor_id);

--
-- Name: idx_vendor_products_category; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vendor_products_category ON public.vendor_products USING btree (product_category);

--
-- Name: idx_vendor_products_city; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vendor_products_city ON public.vendor_products USING btree (city_id);

--
-- Name: idx_vendor_products_featured; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vendor_products_featured ON public.vendor_products USING btree (is_featured) WHERE (is_featured = true);

--
-- Name: idx_vendor_products_features_selection; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vendor_products_features_selection ON public.vendor_products USING gin (features_selection);

--
-- Name: idx_vendor_products_status; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vendor_products_status ON public.vendor_products USING btree (vendor_id, status);

--
-- Name: idx_vendor_products_vendor; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vendor_products_vendor ON public.vendor_products USING btree (vendor_id);

--
-- Name: idx_vendor_products_status_only; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vendor_products_status_only ON public.vendor_products USING btree (status);

--
-- Name: idx_vendor_products_created_at; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vendor_products_created_at ON public.vendor_products USING btree (created_at DESC);

--
-- Name: idx_vendor_review_attachments_review; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vendor_review_attachments_review ON public.vendor_review_attachments USING btree (review_id);

--
-- Name: idx_vendor_reviews_client; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vendor_reviews_client ON public.vendor_reviews USING btree (client_id);

--
-- Name: idx_vendor_reviews_vendor_status; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vendor_reviews_vendor_status ON public.vendor_reviews USING btree (vendor_id, status);

--
-- Name: idx_vp_active_listings; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vp_active_listings ON public.vendor_products USING btree (vendor_id, updated_at DESC) WHERE ((status)::text = 'active'::text);

--
-- Name: idx_vp_amenities_gin; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vp_amenities_gin ON public.vendor_products USING gin (searchable_amenities);

--
-- Name: idx_vp_cultural_attributes_gin; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vp_cultural_attributes_gin ON public.vendor_products USING gin (cultural_attributes);

--
-- Name: idx_vp_gender_section; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vp_gender_section ON public.vendor_products USING btree (gender_section) WHERE (gender_section IS NOT NULL);

--
-- Name: idx_vp_pending_approval; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vp_pending_approval ON public.vendor_products USING btree (created_at) WHERE ((status)::text = 'pending_approval'::text);

--
-- Name: idx_vp_price_on_inquiry; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vp_price_on_inquiry ON public.vendor_products USING btree (price_on_inquiry) WHERE (price_on_inquiry = true);

--
-- Name: idx_vp_published_at; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vp_published_at ON public.vendor_products USING btree (published_at DESC) WHERE (published_at IS NOT NULL);

--
-- Name: idx_vp_quality_score; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vp_quality_score ON public.vendor_products USING btree (quality_score DESC);

--
-- Name: idx_vp_title_ar_trgm; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vp_title_ar_trgm ON public.vendor_products USING gin (title_ar public.gin_trgm_ops) WHERE (title_ar IS NOT NULL);

--
-- Name: idx_vp_title_en_trgm; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vp_title_en_trgm ON public.vendor_products USING gin (title_en public.gin_trgm_ops) WHERE (title_en IS NOT NULL);

--
-- Name: idx_vp_total_capacity; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vp_total_capacity ON public.vendor_products USING btree (total_capacity) WHERE (total_capacity IS NOT NULL);

--
-- Name: idx_vp_vendor_active; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_vp_vendor_active ON public.vendor_packages USING btree (vendor_id, is_active) WHERE (is_active = true);

--
-- Name: vendor_review_attachments review_attachments_modify; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY review_attachments_modify ON public.vendor_review_attachments USING (((EXISTS ( SELECT 1
   FROM public.vendor_reviews r
  WHERE ((r.id = vendor_review_attachments.review_id) AND (r.client_id = (NULLIF(current_setting('app.current_user_id'::text, true), ''::text))::uuid)))) OR (current_setting('app.current_user_role'::text, true) = 'admin'::text)));

--
-- Name: vendor_review_attachments review_attachments_select; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY review_attachments_select ON public.vendor_review_attachments FOR SELECT USING ((EXISTS ( SELECT 1
   FROM public.vendor_reviews r
  WHERE (r.id = vendor_review_attachments.review_id))));

--
-- Name: vendor_reviews reviews_insert; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY reviews_insert ON public.vendor_reviews FOR INSERT WITH CHECK ((client_id = (NULLIF(current_setting('app.current_user_id'::text, true), ''::text))::uuid));

--
-- Name: vendor_reviews reviews_modify; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY reviews_modify ON public.vendor_reviews USING (((client_id = (NULLIF(current_setting('app.current_user_id'::text, true), ''::text))::uuid) OR (current_setting('app.current_user_role'::text, true) = 'admin'::text)));

--
-- Name: vendor_reviews reviews_select; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY reviews_select ON public.vendor_reviews FOR SELECT USING ((((status)::text = 'approved'::text) OR (client_id = (NULLIF(current_setting('app.current_user_id'::text, true), ''::text))::uuid) OR (vendor_id = (NULLIF(current_setting('app.current_user_id'::text, true), ''::text))::uuid) OR (current_setting('app.current_user_role'::text, true) = 'admin'::text)));

--
-- Name: vendor_gallery vendor_gallery_modify; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY vendor_gallery_modify ON public.vendor_gallery USING (((vendor_id = ( SELECT vendors.id
   FROM public.vendors
  WHERE (vendors.user_id = (NULLIF(current_setting('app.current_user_id'::text, true), ''::text))::uuid)
 LIMIT 1)) OR (current_setting('app.current_user_role'::text, true) = 'admin'::text)));

--
-- Name: vendor_gallery vendor_gallery_select; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY vendor_gallery_select ON public.vendor_gallery FOR SELECT USING (true);

--
-- Name: vendor_products vendor_products_modify; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY vendor_products_modify ON public.vendor_products USING (((vendor_id = ( SELECT vendors.id
   FROM public.vendors
  WHERE (vendors.user_id = (NULLIF(current_setting('app.current_user_id'::text, true), ''::text))::uuid)
 LIMIT 1)) OR (current_setting('app.current_user_role'::text, true) = 'admin'::text)));

--
-- Name: vendor_products vendor_products_select; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY vendor_products_select ON public.vendor_products FOR SELECT USING ((((status)::text = 'active'::text) OR (vendor_id = ( SELECT vendors.id
   FROM public.vendors
  WHERE (vendors.user_id = (NULLIF(current_setting('app.current_user_id'::text, true), ''::text))::uuid)
 LIMIT 1)) OR (current_setting('app.current_user_role'::text, true) = 'admin'::text)));

--
-- Name: vendor_products trg_listing_quality_score; Type: TRIGGER; Schema: public; Owner: zafaf_schema_owner
--

CREATE TRIGGER trg_listing_quality_score BEFORE INSERT OR UPDATE ON public.vendor_products FOR EACH ROW EXECUTE FUNCTION public.update_listing_quality_score();

--
-- Name: vendor_products trg_sync_product_search_fields; Type: TRIGGER; Schema: public; Owner: zafaf_schema_owner
--

CREATE TRIGGER trg_sync_product_search_fields BEFORE INSERT OR UPDATE ON public.vendor_products FOR EACH ROW EXECUTE FUNCTION public.sync_product_search_fields();

CREATE TRIGGER trg_vendor_products_updated_at BEFORE UPDATE ON public.vendor_products FOR EACH ROW EXECUTE FUNCTION public.touch_updated_at();

CREATE TRIGGER trg_vendor_reviews_updated_at BEFORE UPDATE ON public.vendor_reviews FOR EACH ROW EXECUTE FUNCTION public.touch_updated_at();

ALTER TABLE ONLY public.vendor_gallery FORCE ROW LEVEL SECURITY;

ALTER TABLE public.vendor_gallery OWNER TO zafaf_schema_owner;

ALTER TABLE public.vendor_packages OWNER TO zafaf_schema_owner;

ALTER TABLE ONLY public.vendor_products FORCE ROW LEVEL SECURITY;

ALTER TABLE public.vendor_products OWNER TO zafaf_schema_owner;

ALTER TABLE ONLY public.vendor_review_attachments FORCE ROW LEVEL SECURITY;

ALTER TABLE public.vendor_review_attachments OWNER TO zafaf_schema_owner;

ALTER TABLE ONLY public.vendor_reviews FORCE ROW LEVEL SECURITY;

ALTER TABLE public.vendor_reviews OWNER TO zafaf_schema_owner;

--
-- Name: vendor_products uq_listing_slug_global; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.vendor_products
    ADD CONSTRAINT uq_listing_slug_global UNIQUE (slug);

--
-- Name: vendor_gallery vendor_gallery_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.vendor_gallery
    ADD CONSTRAINT vendor_gallery_pkey PRIMARY KEY (id);

--
-- Name: vendor_packages vendor_packages_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.vendor_packages
    ADD CONSTRAINT vendor_packages_pkey PRIMARY KEY (id);

--
-- Name: vendor_products vendor_products_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.vendor_products
    ADD CONSTRAINT vendor_products_pkey PRIMARY KEY (id);

--
-- Name: vendor_review_attachments vendor_review_attachments_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.vendor_review_attachments
    ADD CONSTRAINT vendor_review_attachments_pkey PRIMARY KEY (id);

--
-- Name: vendor_reviews vendor_reviews_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.vendor_reviews
    ADD CONSTRAINT vendor_reviews_pkey PRIMARY KEY (id);

--
-- Name: vendor_gallery; Type: ROW SECURITY; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE public.vendor_gallery ENABLE ROW LEVEL SECURITY;

--
-- Name: vendor_products; Type: ROW SECURITY; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE public.vendor_products ENABLE ROW LEVEL SECURITY;

--
-- Name: vendor_review_attachments; Type: ROW SECURITY; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE public.vendor_review_attachments ENABLE ROW LEVEL SECURITY;

--
-- Name: vendor_reviews; Type: ROW SECURITY; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE public.vendor_reviews ENABLE ROW LEVEL SECURITY;

--
-- Name: vendor_gallery vendor_gallery_product_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.vendor_gallery
    ADD CONSTRAINT vendor_gallery_product_id_fkey FOREIGN KEY (product_id) REFERENCES public.vendor_products(id) ON DELETE CASCADE;

--
-- Name: vendor_gallery vendor_gallery_vendor_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.vendor_gallery
    ADD CONSTRAINT vendor_gallery_vendor_id_fkey FOREIGN KEY (vendor_id) REFERENCES public.vendors(id) ON DELETE CASCADE;

--
-- Name: vendor_packages vendor_packages_product_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.vendor_packages
    ADD CONSTRAINT vendor_packages_product_id_fkey FOREIGN KEY (product_id) REFERENCES public.vendor_products(id) ON DELETE CASCADE;

--
-- Name: vendor_packages vendor_packages_vendor_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.vendor_packages
    ADD CONSTRAINT vendor_packages_vendor_id_fkey FOREIGN KEY (vendor_id) REFERENCES public.vendors(id) ON DELETE CASCADE;

--
-- Name: vendor_products vendor_products_city_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.vendor_products
    ADD CONSTRAINT vendor_products_city_id_fkey FOREIGN KEY (city_id) REFERENCES public.cities(id) ON DELETE SET NULL;

--
-- Name: vendor_products vendor_products_vendor_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.vendor_products
    ADD CONSTRAINT vendor_products_vendor_id_fkey FOREIGN KEY (vendor_id) REFERENCES public.vendors(id) ON DELETE CASCADE;

--
-- Name: vendor_review_attachments vendor_review_attachments_review_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.vendor_review_attachments
    ADD CONSTRAINT vendor_review_attachments_review_id_fkey FOREIGN KEY (review_id) REFERENCES public.vendor_reviews(id) ON DELETE CASCADE;

--
-- Name: vendor_reviews vendor_reviews_client_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.vendor_reviews
    ADD CONSTRAINT vendor_reviews_client_id_fkey FOREIGN KEY (client_id) REFERENCES public.global_users(id) ON DELETE CASCADE;

--
-- Name: vendor_reviews vendor_reviews_vendor_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.vendor_reviews
    ADD CONSTRAINT vendor_reviews_vendor_id_fkey FOREIGN KEY (vendor_id) REFERENCES public.global_users(id) ON DELETE CASCADE;

--
-- Name: TABLE vendor_packages; Type: COMMENT; Schema: public; Owner: zafaf_schema_owner
--

COMMENT ON TABLE public.vendor_packages IS 'Listing-specific promotional packages (Silver/Gold/Premium tiers). Max 5 per listing.';

--
-- Name: COLUMN vendor_products.cultural_attributes; Type: COMMENT; Schema: public; Owner: zafaf_schema_owner
--

COMMENT ON COLUMN public.vendor_products.cultural_attributes IS 'GCC cultural amenity flags (prayer_room, valet_parking, etc.) collected in wizard Step 4. Separate from category-specific attributes to allow independent indexing.';

--
-- Name: TABLE reviews; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT ON TABLE public.reviews TO app_client_role;

GRANT SELECT ON TABLE public.reviews TO app_vendor_role;

GRANT SELECT ON TABLE public.reviews TO app_admin_role;

GRANT SELECT ON TABLE public.reviews TO zafaf_app_user;

GRANT SELECT ON TABLE public.reviews TO zafaf_db_admin;

--
-- Name: TABLE vendor_gallery; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_gallery TO app_client_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_gallery TO app_vendor_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_gallery TO app_admin_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_gallery TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_gallery TO zafaf_db_admin;

--
-- Name: TABLE vendor_packages; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_packages TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_packages TO zafaf_db_admin;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_packages TO app_admin_role;

GRANT SELECT ON TABLE public.vendor_packages TO app_vendor_role;

GRANT SELECT ON TABLE public.vendor_packages TO app_client_role;

--
-- Name: TABLE vendor_products; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_products TO app_client_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_products TO app_vendor_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_products TO app_admin_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_products TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_products TO zafaf_db_admin;

--
-- Name: TABLE vendor_review_attachments; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_review_attachments TO app_client_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_review_attachments TO app_vendor_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_review_attachments TO app_admin_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_review_attachments TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_review_attachments TO zafaf_db_admin;

--
-- Name: TABLE vendor_reviews; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_reviews TO app_client_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_reviews TO app_vendor_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_reviews TO app_admin_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_reviews TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.vendor_reviews TO zafaf_db_admin;

ALTER FUNCTION public.apply_gallery_quality_bonus(p_product_id uuid) OWNER TO zafaf_schema_owner;

ALTER FUNCTION public.compute_listing_quality_score(p_title_ar text, p_title_en text, p_description_ar text, p_description_en text, p_gender_section text, p_city_id uuid, p_base_price_sar numeric, p_price_on_inquiry boolean, p_coordinator_name_en text, p_coordinator_phone text, p_attributes jsonb) OWNER TO zafaf_schema_owner;

ALTER FUNCTION public.compute_search_rank(p_quality_score integer, p_verification_level public.vendor_verification_level, p_avg_rating numeric, p_updated_at timestamp with time zone, p_is_featured boolean) OWNER TO zafaf_schema_owner;

ALTER FUNCTION public.sync_product_search_fields() OWNER TO zafaf_db_admin;

ALTER FUNCTION public.update_listing_quality_score() OWNER TO zafaf_schema_owner;


ALTER TABLE public.vendor_gallery ADD COLUMN IF NOT EXISTS file_id UUID REFERENCES public.uploaded_files(id) ON DELETE SET NULL;
CREATE INDEX IF NOT EXISTS idx_vendor_gallery_file ON public.vendor_gallery(file_id);

-- Gallery metadata synchronization trigger function & trigger
CREATE OR REPLACE FUNCTION public.sync_gallery_metadata_from_uploaded_files()
RETURNS trigger AS $$
DECLARE
    v_file RECORD;
BEGIN
    IF NEW.file_id IS NOT NULL THEN
        SELECT file_path, file_url, media_type, thumbnail_url, file_size
        INTO v_file
        FROM public.uploaded_files
        WHERE id = NEW.file_id;

        IF FOUND THEN
            NEW.file_path := COALESCE(NEW.file_path, v_file.file_path);
            NEW.file_url := COALESCE(NEW.file_url, v_file.file_url);
            NEW.image_url := COALESCE(NEW.image_url, v_file.file_url);
            NEW.media_type := COALESCE(NEW.media_type, v_file.media_type);
            NEW.thumbnail_url := COALESCE(NEW.thumbnail_url, v_file.thumbnail_url);
            NEW.file_size := COALESCE(NEW.file_size, v_file.file_size);
        END IF;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

ALTER FUNCTION public.sync_gallery_metadata_from_uploaded_files() OWNER TO zafaf_schema_owner;

CREATE TRIGGER trg_sync_gallery_metadata
BEFORE INSERT OR UPDATE OF file_id ON public.vendor_gallery
FOR EACH ROW
EXECUTE FUNCTION public.sync_gallery_metadata_from_uploaded_files();

