--
-- Migration: 0011_sync_features_in_searchable_amenities
-- Description: Update sync_product_search_fields() trigger function to index dynamic feature choices
--

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
    -- ── Read capacity_mode AND capacity_key from category_schemas ─────────────
    SELECT capacity_mode, capacity_key
    INTO v_capacity_mode, v_capacity_key
    FROM category_schemas
    WHERE category_slug = NEW.product_category;

    v_capacity_mode := COALESCE(v_capacity_mode, 'none');

    -- ── Compute total_capacity ────────────────────────────────────────────────
    CASE v_capacity_mode

        WHEN 'sum_men_women' THEN
            v_men_cap   := (NEW.attributes->>(split_part(v_capacity_key, ',', 1)))::int;
            v_women_cap := (NEW.attributes->>(split_part(v_capacity_key, ',', 2)))::int;
            IF v_men_cap IS NOT NULL AND v_women_cap IS NOT NULL THEN
                NEW.total_capacity := v_men_cap + v_women_cap;
            ELSE
                NEW.total_capacity := NULL;
            END IF;

        WHEN 'single_key' THEN
            IF v_capacity_key IS NOT NULL THEN
                v_single_cap := (NEW.attributes->>v_capacity_key)::int;
                NEW.total_capacity := v_single_cap;
            ELSE
                NEW.total_capacity := NULL;
            END IF;

        ELSE
            NEW.total_capacity := NULL;

    END CASE;

    -- ── Compute searchable_amenities from BOTH attributes columns ─────────────
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

    -- ── Features selection dynamic entries ────────────────────────────────────
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
