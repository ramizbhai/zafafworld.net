--
-- Migration: 0016_robust_search_sync_trigger
-- Description: Implement robust integer/boolean safe cast utilities and update sync_product_search_fields() to prevent crash on malformed inputs.
--

-- ── 1. Create safe integer casting helper ───────────────────────────────────
CREATE OR REPLACE FUNCTION public.safe_cast_int(p_val text) RETURNS int AS $$
BEGIN
    IF p_val IS NULL OR trim(p_val) = '' THEN
        RETURN NULL;
    END IF;
    RETURN p_val::int;
EXCEPTION WHEN OTHERS THEN
    RETURN NULL;
END;
$$ LANGUAGE plpgsql IMMUTABLE;

-- ── 2. Create safe boolean casting helper ────────────────────────────────────
CREATE OR REPLACE FUNCTION public.safe_cast_bool(p_val text) RETURNS boolean AS $$
BEGIN
    IF p_val IS NULL THEN
        RETURN FALSE;
    END IF;
    p_val := lower(trim(p_val));
    IF p_val IN ('true', 't', 'yes', 'y', '1', 'on') THEN
        RETURN TRUE;
    ELSIF p_val IN ('false', 'f', 'no', 'n', '0', 'off') THEN
        RETURN FALSE;
    ELSE
        RETURN p_val::boolean;
    END IF;
EXCEPTION WHEN OTHERS THEN
    RETURN FALSE;
END;
$$ LANGUAGE plpgsql IMMUTABLE;

-- ── 3. Replace trigger function with safe-casting logic ──────────────────────
CREATE OR REPLACE FUNCTION public.sync_product_search_fields() RETURNS trigger
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
    v_feature_keys   TEXT[];
BEGIN
    -- Read capacity_mode AND capacity_key from category_schemas
    SELECT capacity_mode, capacity_key
    INTO v_capacity_mode, v_capacity_key
    FROM category_schemas
    WHERE category_slug = NEW.product_category;

    v_capacity_mode := COALESCE(v_capacity_mode, 'none');

    -- Compute total_capacity
    CASE v_capacity_mode

        WHEN 'sum_men_women' THEN
            v_men_cap   := public.safe_cast_int(NEW.attributes->>(split_part(v_capacity_key, ',', 1)));
            v_women_cap := public.safe_cast_int(NEW.attributes->>(split_part(v_capacity_key, ',', 2)));
            IF v_men_cap IS NOT NULL AND v_women_cap IS NOT NULL THEN
                NEW.total_capacity := v_men_cap + v_women_cap;
            ELSE
                NEW.total_capacity := NULL;
            END IF;

        WHEN 'single_key' THEN
            IF v_capacity_key IS NOT NULL THEN
                v_single_cap := public.safe_cast_int(NEW.attributes->>v_capacity_key);
                NEW.total_capacity := v_single_cap;
            ELSE
                NEW.total_capacity := NULL;
            END IF;

        ELSE
            NEW.total_capacity := NULL;

    END CASE;

    -- Compute searchable_amenities from BOTH attributes columns
    v_merged_attrs := COALESCE(NEW.attributes, '{}'::jsonb)
                   || COALESCE(NEW.cultural_attributes, '{}'::jsonb);

    -- Venue / GCC cultural amenities
    IF public.safe_cast_bool(v_merged_attrs->>'prayer_room') = TRUE
    OR public.safe_cast_bool(v_merged_attrs->>'prayer_room_available') = TRUE THEN
        v_amenities := array_append(v_amenities, 'prayer_room');
    END IF;

    IF public.safe_cast_bool(v_merged_attrs->>'valet_parking') = TRUE THEN
        v_amenities := array_append(v_amenities, 'valet_parking');
    END IF;

    IF public.safe_cast_bool(v_merged_attrs->>'bridal_suite') = TRUE THEN
        v_amenities := array_append(v_amenities, 'bridal_suite');
    END IF;

    IF public.safe_cast_bool(v_merged_attrs->>'outdoor_garden') = TRUE THEN
        v_amenities := array_append(v_amenities, 'outdoor_garden');
    END IF;

    IF public.safe_cast_bool(v_merged_attrs->>'external_catering_allowed') = TRUE THEN
        v_amenities := array_append(v_amenities, 'external_catering_allowed');
    END IF;

    IF public.safe_cast_bool(v_merged_attrs->>'halal_certified') = TRUE
    OR public.safe_cast_bool(v_merged_attrs->>'halal_kitchen') = TRUE THEN
        v_amenities := array_append(v_amenities, 'halal_certified');
    END IF;

    IF public.safe_cast_bool(v_merged_attrs->>'private_pool') = TRUE
    OR public.safe_cast_bool(v_merged_attrs->>'has_private_pool') = TRUE THEN
        v_amenities := array_append(v_amenities, 'private_pool');
    END IF;

    -- Service provider amenities
    IF public.safe_cast_bool(v_merged_attrs->>'female_staff_only') = TRUE
    OR public.safe_cast_bool(v_merged_attrs->>'female_only_staff') = TRUE
    OR public.safe_cast_bool(v_merged_attrs->>'female_team_available') = TRUE THEN
        v_amenities := array_append(v_amenities, 'female_staff');
    END IF;

    IF public.safe_cast_bool(v_merged_attrs->>'female_only_establishment') = TRUE THEN
        v_amenities := array_append(v_amenities, 'female_only_establishment');
    END IF;

    IF public.safe_cast_bool(v_merged_attrs->>'home_service') = TRUE
    OR public.safe_cast_bool(v_merged_attrs->>'home_service_available') = TRUE
    OR public.safe_cast_bool(v_merged_attrs->>'at_home_service') = TRUE THEN
        v_amenities := array_append(v_amenities, 'home_service');
    END IF;

    IF public.safe_cast_bool(v_merged_attrs->>'drone_available') = TRUE THEN
        v_amenities := array_append(v_amenities, 'drone');
    END IF;

    IF public.safe_cast_bool(v_merged_attrs->>'natural_henna_only') = TRUE THEN
        v_amenities := array_append(v_amenities, 'natural_henna');
    END IF;

    IF public.safe_cast_bool(v_merged_attrs->>'female_artist') = TRUE THEN
        v_amenities := array_append(v_amenities, 'female_artist');
    END IF;

    IF public.safe_cast_bool(v_merged_attrs->>'women_only_events') = TRUE
    OR public.safe_cast_bool(v_merged_attrs->>'female_only_event') = TRUE THEN
        v_amenities := array_append(v_amenities, 'women_only_events');
    END IF;

    -- Entertainment amenities
    IF public.safe_cast_bool(v_merged_attrs->>'sound_system_included') = TRUE THEN
        v_amenities := array_append(v_amenities, 'sound_system');
    END IF;

    -- Food / catering amenities
    IF public.safe_cast_bool(v_merged_attrs->>'serving_staff_included') = TRUE THEN
        v_amenities := array_append(v_amenities, 'serving_staff');
    END IF;

    IF public.safe_cast_bool(v_merged_attrs->>'taste_testing') = TRUE
    OR public.safe_cast_bool(v_merged_attrs->>'tasting_session') = TRUE THEN
        v_amenities := array_append(v_amenities, 'tasting_available');
    END IF;

    IF public.safe_cast_bool(v_merged_attrs->>'delivery_to_venue') = TRUE
    OR public.safe_cast_bool(v_merged_attrs->>'delivery_available') = TRUE
    OR public.safe_cast_bool(v_merged_attrs->>'delivery_setup') = TRUE THEN
        v_amenities := array_append(v_amenities, 'delivery_available');
    END IF;

    -- Features selection dynamic entries
    IF NEW.features_selection IS NOT NULL AND jsonb_typeof(NEW.features_selection) = 'object' THEN
        SELECT COALESCE(array_agg(key), ARRAY[]::text[])
        INTO v_feature_keys
        FROM jsonb_each_text(NEW.features_selection)
        WHERE value = 'true';

        v_amenities := v_amenities || v_feature_keys;
    END IF;

    NEW.searchable_amenities := v_amenities;

    RETURN NEW;
END;
$$;
