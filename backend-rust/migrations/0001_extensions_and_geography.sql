-- ═══════════════════════════════════════════════════════════════════════════════
-- Baseline Migration 01: 0001_extensions_and_geography.sql
-- Consolidated ZafafWorld Database Schema Baseline
-- ═══════════════════════════════════════════════════════════════════════════════


-- ─── SYSTEM ROLES ─────────────────────────────────────────────────────────────
DO $$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'zafaf_schema_owner') THEN
        CREATE ROLE zafaf_schema_owner NOLOGIN;
    END IF;
    IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'zafaf_app_user') THEN
        CREATE ROLE zafaf_app_user NOLOGIN;
    END IF;
    IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'app_client_role') THEN
        CREATE ROLE app_client_role NOLOGIN;
    END IF;
    IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'app_vendor_role') THEN
        CREATE ROLE app_vendor_role NOLOGIN;
    END IF;
    IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'app_admin_role') THEN
        CREATE ROLE app_admin_role NOLOGIN;
    END IF;
    IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'zafaf_db_admin') THEN
        CREATE ROLE zafaf_db_admin NOLOGIN;
    END IF;
END $$;

GRANT USAGE ON SCHEMA public TO zafaf_app_user, app_client_role, app_vendor_role, app_admin_role;

SET lock_timeout = 0;

SET idle_in_transaction_session_timeout = 0;

SET client_encoding = 'UTF8';

SET standard_conforming_strings = on;

SET check_function_bodies = false;

SET xmloption = content;

SET client_min_messages = warning;

SET row_security = off;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: pg_trgm; Type: EXTENSION; Schema: -; Owner: -
--

CREATE EXTENSION IF NOT EXISTS pg_trgm WITH SCHEMA public;

--
-- Name: uuid-ossp; Type: EXTENSION; Schema: -; Owner: -
--

CREATE EXTENSION IF NOT EXISTS "uuid-ossp" WITH SCHEMA public;

--
-- Name: cities; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.cities (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    country_id character varying(10) NOT NULL,
    slug character varying(100) NOT NULL,
    name_ar character varying(100) NOT NULL,
    name_en character varying(100) NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);

--
-- Name: countries; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.countries (
    id character varying(10) NOT NULL,
    slug character varying(100) NOT NULL,
    name_ar character varying(100) NOT NULL,
    name_en character varying(100) NOT NULL,
    currency character varying(10) NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);

--
-- Name: districts; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.districts (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    city_id uuid NOT NULL,
    slug character varying(100) NOT NULL,
    name_ar character varying(100) NOT NULL,
    name_en character varying(100) NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);

--
-- Name: idx_cities_country; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_cities_country ON public.cities USING btree (country_id);

-- idx_districts_city removed: Redundant with unique composite constraint unique_district_slug_per_city

ALTER TABLE public.cities OWNER TO zafaf_schema_owner;

ALTER TABLE public.countries OWNER TO zafaf_schema_owner;

ALTER TABLE public.districts OWNER TO zafaf_schema_owner;

--
-- Name: cities cities_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.cities
    ADD CONSTRAINT cities_pkey PRIMARY KEY (id);

--
-- Name: cities cities_slug_key; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.cities
    ADD CONSTRAINT cities_country_slug_key UNIQUE (country_id, slug);

--
-- Name: countries countries_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.countries
    ADD CONSTRAINT countries_pkey PRIMARY KEY (id);

--
-- Name: countries countries_slug_key; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.countries
    ADD CONSTRAINT countries_slug_key UNIQUE (slug);

--
-- Name: districts districts_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.districts
    ADD CONSTRAINT districts_pkey PRIMARY KEY (id);

--
-- Name: districts unique_district_slug_per_city; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.districts
    ADD CONSTRAINT unique_district_slug_per_city UNIQUE (city_id, slug);

--
-- Name: cities cities_country_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.cities
    ADD CONSTRAINT cities_country_id_fkey FOREIGN KEY (country_id) REFERENCES public.countries(id) ON DELETE CASCADE;

--
-- Name: districts districts_city_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.districts
    ADD CONSTRAINT districts_city_id_fkey FOREIGN KEY (city_id) REFERENCES public.cities(id) ON DELETE CASCADE;

--
-- Name: TABLE cities; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.cities TO app_client_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.cities TO app_vendor_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.cities TO app_admin_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.cities TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.cities TO zafaf_db_admin;

--
-- Name: TABLE countries; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.countries TO app_client_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.countries TO app_vendor_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.countries TO app_admin_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.countries TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.countries TO zafaf_db_admin;

--
-- Name: TABLE districts; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.districts TO app_client_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.districts TO app_vendor_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.districts TO app_admin_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.districts TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.districts TO zafaf_db_admin;



-- ─── SEED DATA ───────────────────────────────────────────────────────────────

-- ─── SEED: Saudi Arabia + 5 Major Cities ─────────────────────────────────────

INSERT INTO countries (id, slug, name_ar, name_en, currency)
VALUES ('sa', 'saudi-arabia', 'المملكة العربية السعودية', 'Saudi Arabia', 'SAR')
ON CONFLICT (id) DO NOTHING;

INSERT INTO cities (id, country_id, slug, name_ar, name_en) VALUES
  ('a3f9b2d8-1c4e-4b2a-8f5d-7a6c9e0b1c2d', 'sa', 'riyadh', 'الرياض',       'Riyadh'),
  ('b7c8d9e0-2f1a-4c3b-9d5e-8b6f0a1c2d3e', 'sa', 'jeddah', 'جدة',           'Jeddah'),
  ('c5d6e7f8-3a2b-4d4c-a1f2-9c7d8e0f1a2b', 'sa', 'dammam', 'الدمام',        'Dammam'),
  ('d4e5f6a7-4b3c-4e5d-b2c3-0d8e9f1a2b3c', 'sa', 'makkah', 'مكة المكرمة',   'Makkah'),
  ('e3f4a5b6-5c4d-4f6e-c3d4-1e9f0a2b3c4d', 'sa', 'medina', 'المدينة المنورة','Medina')
ON CONFLICT (country_id, slug) DO NOTHING;

-- ═══════════════════════════════════════════════════════════════════════════
-- Migration: 20260608100002_v2_gcc_cities_seed.sql
-- Phase A-3: Seed full GCC country database + comprehensive city list
-- ═══════════════════════════════════════════════════════════════════════════

-- ─── SEED: GCC Countries ─────────────────────────────────────────────────────

INSERT INTO countries (id, slug, name_ar, name_en, currency) VALUES
    ('ae', 'uae',    'الإمارات العربية المتحدة', 'United Arab Emirates', 'AED'),
    ('kw', 'kuwait', 'الكويت',                    'Kuwait',               'KWD'),
    ('qa', 'qatar',  'قطر',                        'Qatar',                'QAR'),
    ('bh', 'bahrain','البحرين',                    'Bahrain',              'BHD'),
    ('om', 'oman',   'سلطنة عُمان',               'Oman',                 'OMR')
ON CONFLICT (id) DO NOTHING;

-- ─── SEED: Saudi Arabia — All 13 Regions + Major Cities ─────────────────────

INSERT INTO cities (country_id, slug, name_ar, name_en) VALUES
    -- Existing 5 (skip if already there)
    -- Additional SA cities
    ('sa', 'abha',         'أبها',          'Abha'),
    ('sa', 'taif',         'الطائف',        'Taif'),
    ('sa', 'tabuk',        'تبوك',          'Tabuk'),
    ('sa', 'buraidah',     'بريدة',          'Buraidah'),
    ('sa', 'khobar',       'الخبر',         'Al Khobar'),
    ('sa', 'qatif',        'القطيف',        'Qatif'),
    ('sa', 'jubail',       'الجبيل',        'Jubail'),
    ('sa', 'hafuf',        'الأحساء',       'Al Ahsa'),
    ('sa', 'hail',         'حائل',          'Hail'),
    ('sa', 'jizan',        'جازان',         'Jizan'),
    ('sa', 'najran',       'نجران',          'Najran'),
    ('sa', 'yanbu',        'ينبع',           'Yanbu'),
    ('sa', 'khamis-mushait','خميس مشيط',    'Khamis Mushait'),
    ('sa', 'bisha',        'بيشة',           'Bisha'),
    ('sa', 'arar',         'عرعر',           'Arar'),
    ('sa', 'sakaka',       'سكاكا',          'Sakaka'),
    ('sa', 'wajh',         'الوجه',         'Al Wajh'),
    ('sa', 'umluj',        'أملج',           'Umluj'),
    ('sa', 'turaif',       'طريف',           'Turaif'),
    ('sa', 'rass',         'الرس',           'Ar-Rass')
ON CONFLICT (country_id, slug) DO NOTHING;

-- ─── SEED: UAE — All 7 Emirates ──────────────────────────────────────────────

INSERT INTO cities (country_id, slug, name_ar, name_en) VALUES
    ('ae', 'dubai',        'دبي',             'Dubai'),
    ('ae', 'abu-dhabi',    'أبوظبي',          'Abu Dhabi'),
    ('ae', 'sharjah',      'الشارقة',         'Sharjah'),
    ('ae', 'ajman',        'عجمان',           'Ajman'),
    ('ae', 'ras-al-khaimah','رأس الخيمة',     'Ras Al Khaimah'),
    ('ae', 'fujairah',     'الفجيرة',         'Fujairah'),
    ('ae', 'umm-al-quwain','أم القيوين',      'Umm Al Quwain'),
    ('ae', 'al-ain',       'العين',           'Al Ain')
ON CONFLICT (country_id, slug) DO NOTHING;

-- ─── SEED: Kuwait ────────────────────────────────────────────────────────────

INSERT INTO cities (country_id, slug, name_ar, name_en) VALUES
    ('kw', 'kuwait-city',  'مدينة الكويت',   'Kuwait City'),
    ('kw', 'ahmadi',       'الأحمدي',        'Ahmadi'),
    ('kw', 'hawalli',      'حولي',           'Hawalli'),
    ('kw', 'farwaniyah',   'الفروانية',      'Al Farwaniyah'),
    ('kw', 'jahra',        'الجهراء',        'Al Jahra'),
    ('kw', 'mubarak-al-kabeer','مبارك الكبير', 'Mubarak Al Kabeer')
ON CONFLICT (country_id, slug) DO NOTHING;

-- ─── SEED: Qatar ─────────────────────────────────────────────────────────────

INSERT INTO cities (country_id, slug, name_ar, name_en) VALUES
    ('qa', 'doha',         'الدوحة',         'Doha'),
    ('qa', 'al-rayyan',    'الريان',         'Al Rayyan'),
    ('qa', 'al-wakrah',    'الوكرة',         'Al Wakrah'),
    ('qa', 'al-khor',      'الخور',          'Al Khor'),
    ('qa', 'lusail',       'لوسيل',          'Lusail'),
    ('qa', 'al-shamal',    'الشمال',         'Al Shamal')
ON CONFLICT (country_id, slug) DO NOTHING;

-- ─── SEED: Bahrain ───────────────────────────────────────────────────────────

INSERT INTO cities (country_id, slug, name_ar, name_en) VALUES
    ('bh', 'manama',       'المنامة',        'Manama'),
    ('bh', 'muharraq',     'المحرق',         'Muharraq'),
    ('bh', 'riffa',        'الرفاع',         'Riffa'),
    ('bh', 'hamad-town',   'مدينة حمد',      'Hamad Town'),
    ('bh', 'isa-town',     'مدينة عيسى',     'Isa Town'),
    ('bh', 'sitra',        'سترة',           'Sitra'),
    ('bh', 'amwaj',        'جزر أمواج',      'Amwaj Islands')
ON CONFLICT (country_id, slug) DO NOTHING;

-- ─── SEED: Oman ──────────────────────────────────────────────────────────────

INSERT INTO cities (country_id, slug, name_ar, name_en) VALUES
    ('om', 'muscat',       'مسقط',           'Muscat'),
    ('om', 'salalah',      'صلالة',          'Salalah'),
    ('om', 'sohar',        'صحار',           'Sohar'),
    ('om', 'nizwa',        'نزوى',           'Nizwa'),
    ('om', 'sur',          'صور',            'Sur'),
    ('om', 'rustaq',       'الرستاق',        'Rustaq'),
    ('om', 'khasab',       'خصب',            'Khasab'),
    ('om', 'barka',        'بركاء',          'Barka')
ON CONFLICT (country_id, slug) DO NOTHING;

