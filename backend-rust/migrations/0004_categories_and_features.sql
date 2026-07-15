-- ═══════════════════════════════════════════════════════════════════════════════
-- Baseline Migration 04: 0004_categories_and_features.sql
-- Consolidated ZafafWorld Database Schema Baseline
-- ═══════════════════════════════════════════════════════════════════════════════


--
-- Name: category_parent_group; Type: TYPE; Schema: public; Owner: zafaf_db_admin
--

CREATE TYPE public.category_parent_group AS ENUM (
    'venues',
    'fashion',
    'beauty',
    'photography',
    'food',
    'entertainment',
    'jewelry_gifts',
    'planning_decor',
    'transportation'
);

--
-- Name: categories; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.categories (
    slug character varying(100) NOT NULL,
    name_ar character varying(200) NOT NULL,
    name_en character varying(200) NOT NULL,
    description_ar text,
    description_en text,
    parent_group public.category_parent_group NOT NULL,
    icon_name character varying(100),
    emoji character varying(10),
    priority character varying(20) DEFAULT 'important'::character varying NOT NULL,
    is_active boolean DEFAULT true NOT NULL,
    sort_order integer DEFAULT 0 NOT NULL,
    available_countries character varying(10)[] DEFAULT ARRAY['sa','ae','kw','qa','bh','om']::character varying[] NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    launch_phase integer DEFAULT 2 NOT NULL,
    CONSTRAINT chk_category_priority CHECK (((priority)::text = ANY ((ARRAY['core'::character varying, 'important'::character varying, 'optional'::character varying])::text[]))),
    CONSTRAINT chk_launch_phase CHECK ((launch_phase = ANY (ARRAY[1, 2, 3])))
);

--
-- Name: category_schemas; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.category_schemas (
    category_slug character varying(100) NOT NULL,
    schema_version integer DEFAULT 1 NOT NULL,
    schema_json jsonb DEFAULT '{}'::jsonb NOT NULL,
    searchable_fields jsonb DEFAULT '[]'::jsonb NOT NULL,
    required_fields jsonb DEFAULT '[]'::jsonb NOT NULL,
    gcc_required_fields jsonb DEFAULT '[]'::jsonb NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    inquiry_fields jsonb DEFAULT '[]'::jsonb NOT NULL,
    card_fields jsonb DEFAULT '[]'::jsonb NOT NULL,
    filter_fields jsonb DEFAULT '[]'::jsonb NOT NULL,
    capacity_mode character varying(20) DEFAULT 'none'::character varying,
    capacity_key character varying(100),
    is_active boolean DEFAULT true NOT NULL,
    CONSTRAINT chk_capacity_mode CHECK (((capacity_mode)::text = ANY ((ARRAY['sum_men_women'::character varying, 'single_key'::character varying, 'none'::character varying])::text[])))
);

--
-- Name: features; Type: TABLE; Schema: public; Owner: zafaf_schema_owner
--

CREATE TABLE public.features (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    name_en character varying(255) NOT NULL,
    name_ar character varying(255) NOT NULL,
    category character varying(100) NOT NULL,
    input_type character varying(50) DEFAULT 'boolean'::character varying NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);

--
-- Name: idx_categories_active; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_categories_active ON public.categories USING btree (is_active) WHERE (is_active = true);

--
-- Name: idx_categories_parent; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_categories_parent ON public.categories USING btree (parent_group);

--
-- Name: idx_features_category; Type: INDEX; Schema: public; Owner: zafaf_schema_owner
--

CREATE INDEX idx_features_category ON public.features USING btree (category);

--
-- Name: features catalog_modify_features; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY catalog_modify_features ON public.features USING ((current_setting('app.current_user_role'::text, true) = 'admin'::text));

--
-- Name: features catalog_select_features; Type: POLICY; Schema: public; Owner: zafaf_schema_owner
--

CREATE POLICY catalog_select_features ON public.features FOR SELECT USING (true);

--
-- Name: features trg_features_updated_at; Type: TRIGGER; Schema: public; Owner: zafaf_schema_owner
--

CREATE TRIGGER trg_features_updated_at BEFORE UPDATE ON public.features FOR EACH ROW EXECUTE FUNCTION public.touch_updated_at();

-- Triggers for updated_at tracking
CREATE TRIGGER trg_categories_updated_at BEFORE UPDATE ON public.categories FOR EACH ROW EXECUTE FUNCTION public.touch_updated_at();
CREATE TRIGGER trg_category_schemas_updated_at BEFORE UPDATE ON public.category_schemas FOR EACH ROW EXECUTE FUNCTION public.touch_updated_at();

ALTER TABLE public.categories OWNER TO zafaf_schema_owner;

ALTER TABLE public.category_schemas OWNER TO zafaf_schema_owner;

ALTER TABLE ONLY public.features FORCE ROW LEVEL SECURITY;

ALTER TABLE public.features OWNER TO zafaf_schema_owner;

--
-- Name: categories categories_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.categories
    ADD CONSTRAINT categories_pkey PRIMARY KEY (slug);

--
-- Name: category_schemas category_schemas_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.category_schemas
    ADD CONSTRAINT category_schemas_pkey PRIMARY KEY (category_slug);

--
-- Name: features features_pkey; Type: CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.features
    ADD CONSTRAINT features_pkey PRIMARY KEY (id);

--
-- Name: features; Type: ROW SECURITY; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE public.features ENABLE ROW LEVEL SECURITY;

--
-- Name: category_schemas category_schemas_category_slug_fkey; Type: FK CONSTRAINT; Schema: public; Owner: zafaf_schema_owner
--

ALTER TABLE ONLY public.category_schemas
    ADD CONSTRAINT category_schemas_category_slug_fkey FOREIGN KEY (category_slug) REFERENCES public.categories(slug) ON DELETE CASCADE;

--
-- Name: COLUMN categories.launch_phase; Type: COMMENT; Schema: public; Owner: zafaf_schema_owner
--

COMMENT ON COLUMN public.categories.launch_phase IS '1=Phase1 Launch, 2=Phase2 Expansion, 3=Phase3 Future. Determines which categories appear in UI at each stage.';

--
-- Name: COLUMN category_schemas.capacity_mode; Type: COMMENT; Schema: public; Owner: zafaf_schema_owner
--

COMMENT ON COLUMN public.category_schemas.capacity_mode IS 'How total_capacity is derived: sum_men_women (adds two keyed fields) | single_key (reads one named field) | none (not applicable)';

--
-- Name: COLUMN category_schemas.capacity_key; Type: COMMENT; Schema: public; Owner: zafaf_schema_owner
--

COMMENT ON COLUMN public.category_schemas.capacity_key IS 'The JSONB key(s) used as the capacity source. Comma-separated for sum_men_women mode.';

--
-- Name: COLUMN category_schemas.is_active; Type: COMMENT; Schema: public; Owner: zafaf_schema_owner
--

COMMENT ON COLUMN public.category_schemas.is_active IS 'Soft-disable a schema without removing it. Inactive schemas are not exposed via API.';

--
-- Name: TABLE categories; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.categories TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.categories TO zafaf_db_admin;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.categories TO app_admin_role;

GRANT SELECT ON TABLE public.categories TO app_vendor_role;

GRANT SELECT ON TABLE public.categories TO app_client_role;

--
-- Name: TABLE category_schemas; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.category_schemas TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.category_schemas TO zafaf_db_admin;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.category_schemas TO app_admin_role;

GRANT SELECT ON TABLE public.category_schemas TO app_vendor_role;

GRANT SELECT ON TABLE public.category_schemas TO app_client_role;

--
-- Name: TABLE features; Type: ACL; Schema: public; Owner: zafaf_schema_owner
--

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.features TO app_admin_role;

GRANT SELECT ON TABLE public.features TO app_vendor_role;

GRANT SELECT ON TABLE public.features TO app_client_role;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.features TO zafaf_app_user;

GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.features TO zafaf_db_admin;

ALTER TYPE public.category_parent_group OWNER TO zafaf_db_admin;



-- ─── SEED DATA ───────────────────────────────────────────────────────────────

-- ─── SEED: 32 CATEGORIES ─────────────────────────────────────────────────────

INSERT INTO categories (slug, name_ar, name_en, parent_group, emoji, priority, sort_order, available_countries) VALUES

-- ── VENUES (A) ──────────────────────────────────────────────────────────────
('wedding-palace',    'قصور الأفراح',         'Wedding Palace',         'venues', '🏛️', 'core',      10, ARRAY['sa','ae','kw','qa','bh','om']::character varying[]),
('hotel-venue',       'فنادق وقاعات',          'Hotel Ballroom & Venue', 'venues', '🏨', 'core',      20, ARRAY['sa','ae','kw','qa','bh','om']::character varying[]),
('villa-resort',      'استراحات وفلل',         'Villa & Private Resort', 'venues', '🏡', 'core',      30, ARRAY['sa','ae','kw','qa','bh','om']::character varying[]),
('restaurant-event',  'مطاعم وقاعات خاصة',    'Restaurant & Dining',    'venues', '🍽️', 'important', 40, ARRAY['sa','ae','kw','qa','bh','om']::character varying[]),
('outdoor-garden',    'حدائق وأماكن مفتوحة',   'Outdoor Garden Venue',   'venues', '🌿', 'important', 50, ARRAY['sa','ae','qa']::character varying[]),
('rooftop-venue',     'أماكن على السطح',       'Rooftop Venue',          'venues', '🌃', 'optional',  60, ARRAY['ae','qa']::character varying[]),
('private-beach',     'شاطئ خاص',              'Private Beach Venue',    'venues', '🏖️', 'optional',  70, ARRAY['ae','qa','bh']::character varying[]),
('chalet',            'شاليهات',               'Chalet & Seasonal Venue','venues', '🏕️', 'important', 80, ARRAY['sa','kw']::character varying[]),

-- ── FASHION (B) ─────────────────────────────────────────────────────────────
('wedding-gown',      'فساتين الزفاف',         'Wedding Gown & Dress',   'fashion', '👗', 'core',      10, ARRAY['sa','ae','kw','qa','bh','om']::character varying[]),
('haute-couture',     'أزياء راقية وتصميم',    'Haute Couture',          'fashion', '✨', 'important', 20, ARRAY['sa','ae','kw','qa','bh','om']::character varying[]),
('abaya-jalabiya',    'عباية وجلابية وكفتان',   'Abaya, Jalabiya & Kaftan','fashion','🧕', 'core',      30, ARRAY['sa','ae','kw','qa','bh','om']::character varying[]),
('groom-attire',      'ملابس العريس والبشت',    'Groom Attire & Bisht',   'fashion', '👘', 'important', 40, ARRAY['sa','ae','kw','qa','bh','om']::character varying[]),

-- ── BEAUTY (C) ──────────────────────────────────────────────────────────────
('hair-makeup',       'تجميل وشعر وعناية بالبشرة', 'Hair, Makeup & Beauty', 'beauty', '💄', 'core',      10, ARRAY['sa','ae','kw','qa','bh','om']::character varying[]),
('henna-art',         'فن الحناء',              'Henna Art',              'beauty', '🌿', 'core',      30, ARRAY['sa','ae','kw','qa','bh','om']::character varying[]),

-- ── PHOTOGRAPHY (D) ─────────────────────────────────────────────────────────
('photography-video', 'تصوير وفيديو',          'Photography & Video',    'photography', '📷', 'core',     10, ARRAY['sa','ae','kw','qa','bh','om']::character varying[]),
('photo-studio',      'استوديو تصوير',          'Photo Studio',           'photography', '📸', 'important', 20, ARRAY['sa','ae','kw','qa','bh','om']::character varying[]),

-- ── FOOD (E) ────────────────────────────────────────────────────────────────
('catering',          'ضيافة وطعام',            'Wedding Catering',       'food', '🍱', 'core',      10, ARRAY['sa','ae','kw','qa','bh','om']::character varying[]),
('wedding-cake',      'كيك الزفاف والحلويات',   'Wedding Cake & Desserts','food', '🎂', 'core',      20, ARRAY['sa','ae','kw','qa','bh','om']::character varying[]),
('wedding-sweets',    'حلويات عربية وتوزيعات',  'Arabic Sweets & Treats', 'food', '🍬', 'important', 30, ARRAY['sa','ae','kw','qa','bh','om']::character varying[]),

-- ── ENTERTAINMENT (F) ────────────────────────────────────────────────────────
('entertainment-dj',  'دي جي، فرق موسيقية وزفات', 'DJ, Bands & Zaffa',    'entertainment', '🎵', 'core',      10, ARRAY['sa','ae','kw','qa','bh','om']::character varying[]),

-- ── JEWELRY & GIFTS (G) ──────────────────────────────────────────────────────
('wedding-jewelry',   'مجوهرات وخواتم',         'Bridal Jewelry',         'jewelry_gifts', '💍', 'core',      10, ARRAY['sa','ae','kw','qa','bh','om']::character varying[]),
('wedding-gifts',     'هدايا وتوزيعات',         'Wedding Favors & Gifts', 'jewelry_gifts', '🎁', 'important', 20, ARRAY['sa','ae','kw','qa','bh','om']::character varying[]),

-- ── PLANNING & DECOR (H) ─────────────────────────────────────────────────────
('wedding-planner',   'تخطيط وتنسيق الحفلات والكوش', 'Wedding Planning & Decor', 'planning_decor', '📋', 'core',      10, ARRAY['sa','ae','kw','qa','bh','om']::character varying[]),
('flowers-floral',    'ورد وزهور',               'Flowers & Floral',       'planning_decor', '💐', 'core',      30, ARRAY['sa','ae','kw','qa','bh','om']::character varying[]),
('wedding-invitation','دعوات زفاف',              'Wedding Invitations',    'planning_decor', '✉️', 'important', 40, ARRAY['sa','ae','kw','qa','bh','om']::character varying[]),
('lighting-av',       'إضاءة وتقنية الصوت',      'Lighting & AV',          'planning_decor', '💡', 'important', 50, ARRAY['sa','ae','kw','qa','bh','om']::character varying[]),

-- ── TRANSPORTATION (I) ────────────────────────────────────────────────────────
('wedding-car',       'سيارات الزفاف',           'Wedding Car & Limo',     'transportation', '🚗', 'important', 10, ARRAY['sa','ae','kw','qa','bh','om']::character varying[]);

-- ─── SEED: CATEGORY SCHEMAS (key attributes per category) ────────────────────

INSERT INTO category_schemas (category_slug, schema_json, searchable_fields, required_fields, gcc_required_fields) VALUES

('wedding-palace', '{
  "men_capacity": {"type": "number", "label_ar": "سعة قاعة الرجال", "label_en": "Men Section Capacity"},
  "women_capacity": {"type": "number", "label_ar": "سعة قاعة النساء", "label_en": "Women Section Capacity"},
  "has_separate_entrances": {"type": "boolean", "label_ar": "مداخل منفصلة", "label_en": "Separate Entrances"},
  "has_soundproofed_partition": {"type": "boolean", "label_ar": "فاصل عازل للصوت", "label_en": "Soundproofed Partition"},
  "has_audio_link": {"type": "boolean", "label_ar": "ربط صوتي بين القاعتين", "label_en": "Audio Link Between Halls"},
  "prayer_room": {"type": "boolean", "label_ar": "مصلى / غرفة صلاة", "label_en": "Prayer Room / Musala"},
  "valet_parking": {"type": "boolean", "label_ar": "صف السيارات", "label_en": "Valet Parking"},
  "external_catering_allowed": {"type": "boolean", "label_ar": "ضيافة خارجية مسموحة", "label_en": "External Catering Allowed"},
  "bridal_suite": {"type": "boolean", "label_ar": "جناح العروس", "label_en": "Bridal Suite"},
  "outdoor_garden": {"type": "boolean", "label_ar": "حديقة خارجية", "label_en": "Outdoor Garden"},
  "max_events_per_day": {"type": "number", "label_ar": "أقصى حفلات يومياً", "label_en": "Max Events Per Day"}
}', '["prayer_room","valet_parking","external_catering_allowed","bridal_suite","outdoor_garden","has_separate_entrances"]',
'["men_capacity","women_capacity"]', '["has_separate_entrances","prayer_room","external_catering_allowed"]'),

('hotel-venue', '{
  "star_rating": {"type": "number", "label_ar": "تصنيف النجوم", "label_en": "Hotel Star Rating"},
  "ballroom_capacity": {"type": "number", "label_ar": "سعة القاعة", "label_en": "Ballroom Capacity"},
  "in_house_catering": {"type": "boolean", "label_ar": "ضيافة داخلية", "label_en": "In-house Catering"},
  "catering_mandatory": {"type": "boolean", "label_ar": "ضيافة إلزامية", "label_en": "Catering Mandatory"},
  "prayer_room": {"type": "boolean", "label_ar": "مصلى", "label_en": "Prayer Room"},
  "valet_parking": {"type": "boolean", "label_ar": "صف السيارات", "label_en": "Valet Parking"},
  "bridal_suite": {"type": "boolean", "label_ar": "جناح العروس", "label_en": "Bridal Suite"},
  "halal_kitchen": {"type": "boolean", "label_ar": "مطبخ حلال", "label_en": "Halal Kitchen"},
  "outdoor_terrace": {"type": "boolean", "label_ar": "تراس خارجي", "label_en": "Outdoor Terrace"}
}', '["prayer_room","valet_parking","bridal_suite","halal_kitchen","outdoor_terrace"]',
'["ballroom_capacity"]', '["prayer_room","halal_kitchen"]'),

('villa-resort', '{
  "max_capacity": {"type": "number", "label_ar": "الطاقة الاستيعابية", "label_en": "Max Guest Capacity"},
  "num_bedrooms": {"type": "number", "label_ar": "عدد غرف النوم", "label_en": "Number of Bedrooms"},
  "private_pool": {"type": "boolean", "label_ar": "مسبح خاص", "label_en": "Private Pool"},
  "external_catering_allowed": {"type": "boolean", "label_ar": "ضيافة خارجية مسموحة", "label_en": "External Catering Allowed"},
  "full_exclusivity": {"type": "boolean", "label_ar": "حجز حصري كامل", "label_en": "Full Property Exclusivity"},
  "weekend_surcharge_sar": {"type": "number", "label_ar": "رسوم إضافية للعطلة (ريال)", "label_en": "Weekend Surcharge (SAR)"},
  "minimum_rental_hours": {"type": "number", "label_ar": "الحد الأدنى لساعات الإيجار", "label_en": "Minimum Rental Hours"},
  "outdoor_bbq": {"type": "boolean", "label_ar": "مشواة خارجية", "label_en": "Outdoor BBQ"}
}', '["private_pool","external_catering_allowed","full_exclusivity","outdoor_bbq"]',
'["max_capacity"]', '["external_catering_allowed"]'),

('restaurant-event', '{
  "private_hall_capacity": {"type": "number", "label_ar": "سعة القاعة الخاصة", "label_en": "Private Hall Capacity"},
  "family_section": {"type": "boolean", "label_ar": "قسم عائلي", "label_en": "Family Section"},
  "private_entrance": {"type": "boolean", "label_ar": "مدخل خاص", "label_en": "Private Entrance"},
  "halal_certified": {"type": "boolean", "label_ar": "شهادة حلال", "label_en": "Halal Certified"},
  "outdoor_seating": {"type": "boolean", "label_ar": "جلسات خارجية", "label_en": "Outdoor Seating"},
  "av_equipment": {"type": "boolean", "label_ar": "معدات صوت وصورة", "label_en": "AV Equipment"},
  "minimum_spend_sar": {"type": "number", "label_ar": "الحد الأدنى للإنفاق (ريال)", "label_en": "Minimum Spend (SAR)"}
}', '["family_section","halal_certified","private_entrance","outdoor_seating"]',
'["private_hall_capacity"]', '["halal_certified","family_section"]'),

('outdoor-garden', '{
  "seated_capacity": {"type": "number", "label_ar": "سعة الجلوس", "label_en": "Seated Capacity"},
  "tent_included": {"type": "boolean", "label_ar": "خيمة أو مظلة متضمنة", "label_en": "Tent/Canopy Included"},
  "backup_indoor": {"type": "boolean", "label_ar": "بديل داخلي متاح", "label_en": "Backup Indoor Space"},
  "lighting_included": {"type": "boolean", "label_ar": "إضاءة متضمنة", "label_en": "Lighting Package"},
  "available_months": {"type": "text", "label_ar": "الأشهر المتاحة", "label_en": "Available Months (Oct–Mar)"},
  "external_catering_allowed": {"type": "boolean", "label_ar": "ضيافة خارجية مسموحة", "label_en": "External Catering Allowed"}
}', '["tent_included","backup_indoor","lighting_included"]',
'["seated_capacity"]', '["tent_included"]'),

('rooftop-venue', '{
  "capacity": {"type": "number", "label_ar": "الطاقة الاستيعابية", "label_en": "Capacity"},
  "elevator_access": {"type": "boolean", "label_ar": "مصعد", "label_en": "Elevator Access"},
  "wind_protection": {"type": "boolean", "label_ar": "حواجز مضادة للريح", "label_en": "Wind Protection"},
  "city_view": {"type": "boolean", "label_ar": "إطلالة على المدينة", "label_en": "City View"},
  "noise_curfew_time": {"type": "text", "label_ar": "وقت إنهاء الصوت", "label_en": "Noise Curfew Time"}
}', '["elevator_access","wind_protection","city_view"]',
'["capacity"]', '[]'),

('private-beach', '{
  "capacity": {"type": "number", "label_ar": "الطاقة الاستيعابية", "label_en": "Capacity"},
  "private_or_semi": {"type": "text", "label_ar": "خاص أم شبه خاص", "label_en": "Private or Semi-Private"},
  "floating_stage": {"type": "boolean", "label_ar": "منصة عائمة", "label_en": "Floating Stage"},
  "shade_structures": {"type": "boolean", "label_ar": "مظلات", "label_en": "Shade Structures"}
}', '["floating_stage","shade_structures"]',
'["capacity"]', '[]'),

('chalet', '{
  "num_units": {"type": "number", "label_ar": "عدد الوحدات", "label_en": "Number of Units"},
  "total_capacity": {"type": "number", "label_ar": "الطاقة الاستيعابية الكلية", "label_en": "Total Capacity"},
  "private_pool": {"type": "boolean", "label_ar": "مسبح خاص", "label_en": "Private Pool"},
  "bbq": {"type": "boolean", "label_ar": "مشواة", "label_en": "BBQ Facilities"},
  "separate_sections": {"type": "boolean", "label_ar": "أقسام منفصلة رجال ونساء", "label_en": "Separate Men/Women Sections"},
  "kitchen_facilities": {"type": "boolean", "label_ar": "مرافق مطبخ", "label_en": "Kitchen Facilities"},
  "security_included": {"type": "boolean", "label_ar": "أمن متضمن", "label_en": "Security Guards Included"},
  "external_catering_allowed": {"type": "boolean", "label_ar": "ضيافة خارجية مسموحة", "label_en": "External Catering Allowed"}
}', '["private_pool","bbq","separate_sections","kitchen_facilities"]',
'["total_capacity"]', '["separate_sections","external_catering_allowed"]'),

('wedding-gown', '{
  "delivery_time_weeks": {"type": "number", "label_ar": "مدة التصنيع (أسابيع)", "label_en": "Custom Order Lead Time (Weeks)"},
  "ready_to_wear": {"type": "boolean", "label_ar": "جاهز للارتداء", "label_en": "Ready-to-Wear Available"},
  "custom_bespoke": {"type": "boolean", "label_ar": "تصميم مخصص", "label_en": "Custom / Bespoke Design"},
  "alterations_service": {"type": "boolean", "label_ar": "خدمة الخياطة والتعديل", "label_en": "Alterations Service"},
  "rental_available": {"type": "boolean", "label_ar": "تأجير متاح", "label_en": "Rental Available"},
  "at_home_fitting": {"type": "boolean", "label_ar": "قياس في المنزل", "label_en": "At-Home Fitting"}
}', '["ready_to_wear","rental_available","alterations_service","at_home_fitting"]',
'[]', '[]'),

('haute-couture', '{
  "lead_time_weeks": {"type": "number", "label_ar": "مدة التصنيم (أسابيع)", "label_en": "Lead Time (Weeks)"},
  "fitting_sessions": {"type": "number", "label_ar": "جلسات القياس المتضمنة", "label_en": "Fitting Sessions Included"},
  "international_designer": {"type": "boolean", "label_ar": "مصمم دولي", "label_en": "International Designer"},
  "accessories_included": {"type": "boolean", "label_ar": "الإكسسوارات متضمنة", "label_en": "Accessories Included"}
}', '[]',
'["lead_time_weeks"]', '[]'),

('abaya-jalabiya', '{
  "styles_offered": {"type": "text", "label_ar": "الأنواع المتاحة", "label_en": "Styles Offered (abaya/jalabiya/kaftan)"},
  "custom_available": {"type": "boolean", "label_ar": "تصميم مخصص", "label_en": "Custom Available"},
  "lead_time_days": {"type": "number", "label_ar": "مدة التصنيع (أيام)", "label_en": "Lead Time (Days)"},
  "rental_available": {"type": "boolean", "label_ar": "تأجير متاح", "label_en": "Rental Available"},
  "matching_accessories": {"type": "boolean", "label_ar": "إكسسوارات مطابقة", "label_en": "Matching Accessories"}
}', '["rental_available","custom_available"]',
'[]', '[]'),

('groom-attire', '{
  "types_offered": {"type": "text", "label_ar": "الأنواع المتاحة", "label_en": "Types (bisht/thobe/suit/kandura)"},
  "custom_tailoring": {"type": "boolean", "label_ar": "خياطة مخصصة", "label_en": "Custom Tailoring"},
  "lead_time_days": {"type": "number", "label_ar": "مدة التصنيع (أيام)", "label_en": "Lead Time (Days)"},
  "gold_embroidery": {"type": "boolean", "label_ar": "تطريز ذهبي", "label_en": "Gold Thread Embroidery"}
}', '["custom_tailoring","gold_embroidery"]',
'[]', '[]'),

('hair-makeup', '{
  "female_only_staff": {"type": "boolean", "label_ar": "فريق نسائي فقط", "label_en": "Female-Only Staff / Salon"},
  "home_service": {"type": "boolean", "label_ar": "خدمة منزلية", "label_en": "Home/On-Location Service"},
  "trial_session": {"type": "boolean", "label_ar": "جلسة تجريبية", "label_en": "Trial Session Available"},
  "airbrush_makeup": {"type": "boolean", "label_ar": "مكياج بالرش", "label_en": "Airbrush Makeup"},
  "team_size": {"type": "number", "label_ar": "حجم الفريق", "label_en": "Team Size"},
  "services_offered": {"type": "text", "label_ar": "الخدمات المتاحة", "label_en": "Services (facial/laser/peel/body)"},
  "bride_package_weeks": {"type": "number", "label_ar": "مدة باقة العروس (أسابيع)", "label_en": "Bride Package Duration (Weeks)"},
  "licensed_clinic": {"type": "boolean", "label_ar": "عيادة مرخصة", "label_en": "Licensed Medical Clinic"}
}', '["female_only_staff","home_service","airbrush_makeup","trial_session","licensed_clinic"]',
'[]', '["female_only_staff"]'),

('henna-art', '{
  "home_service": {"type": "boolean", "label_ar": "خدمة منزلية", "label_en": "Home Service Available"},
  "bridal_package": {"type": "boolean", "label_ar": "باقة عروس (يدين وقدمين)", "label_en": "Bridal Henna (Hands & Feet)"},
  "natural_henna_only": {"type": "boolean", "label_ar": "حناء طبيعية 100%", "label_en": "100% Natural Henna Only"},
  "design_styles": {"type": "text", "label_ar": "أنماط التصميم", "label_en": "Design Styles (traditional/modern)"},
  "female_artist": {"type": "boolean", "label_ar": "فنانة حناء", "label_en": "Female Artist"}
}', '["home_service","natural_henna_only","female_artist","bridal_package"]',
'[]', '["natural_henna_only","female_artist"]'),

('photography-video', '{
  "female_team_available": {"type": "boolean", "label_ar": "مصورة متاحة", "label_en": "Female Team Available"},
  "women_section_coverage": {"type": "boolean", "label_ar": "تغطية قاعة النساء", "label_en": "Women Section Coverage"},
  "drone_available": {"type": "boolean", "label_ar": "تصوير بالطائرة المسيّرة", "label_en": "Drone Available"},
  "raw_files_provided": {"type": "boolean", "label_ar": "ملفات RAW متضمنة", "label_en": "RAW Files Provided"},
  "team_size": {"type": "number", "label_ar": "حجم الفريق", "label_en": "Team Size"},
  "delivery_weeks": {"type": "number", "label_ar": "مدة التسليم (أسابيع)", "label_en": "Delivery Time (Weeks)"},
  "highlight_reel": {"type": "boolean", "label_ar": "فيديو ملخص", "label_en": "Highlight Reel Included"},
  "second_shooter": {"type": "boolean", "label_ar": "مصور ثانٍ", "label_en": "Second Shooter"},
  "destination_available": {"type": "boolean", "label_ar": "تصوير خارج المدينة", "label_en": "Destination Coverage Available"}
}', '["female_team_available","women_section_coverage","drone_available","highlight_reel","second_shooter"]',
'[]', '["female_team_available","women_section_coverage"]'),

('photo-studio', '{
  "session_types": {"type": "text", "label_ar": "أنواع الجلسات", "label_en": "Session Types (engagement/bridal/family)"},
  "outdoor_locations": {"type": "boolean", "label_ar": "مواقع خارجية", "label_en": "Outdoor Locations Offered"},
  "same_day_editing": {"type": "boolean", "label_ar": "تعديل نفس اليوم", "label_en": "Same-Day Editing"},
  "printed_albums": {"type": "boolean", "label_ar": "ألبومات مطبوعة", "label_en": "Printed Albums"}
}', '["outdoor_locations","same_day_editing"]',
'[]', '[]'),

('catering', '{
  "min_guests": {"type": "number", "label_ar": "الحد الأدنى للضيوف", "label_en": "Minimum Guests Required"},
  "serving_staff_included": {"type": "boolean", "label_ar": "طاقم الخدمة متضمن", "label_en": "Serving Staff Included"},
  "halal_certified": {"type": "boolean", "label_ar": "شهادة حلال", "label_en": "Halal Certified"},
  "taste_testing": {"type": "boolean", "label_ar": "تذوق مسبق", "label_en": "Taste Testing Available"},
  "setup_cleanup": {"type": "boolean", "label_ar": "تجهيز وترتيب متضمن", "label_en": "Setup & Cleanup Included"},
  "equipment_rental": {"type": "boolean", "label_ar": "تأجير المعدات متضمن", "label_en": "Equipment Rental Included"},
  "buffet_or_plated": {"type": "text", "label_ar": "نوع الخدمة", "label_en": "Service Type (buffet/plated)"}
}', '["halal_certified","serving_staff_included","setup_cleanup","taste_testing"]',
'["min_guests"]', '["halal_certified"]'),

('wedding-cake', '{
  "serves_max": {"type": "number", "label_ar": "أقصى عدد حصص", "label_en": "Max Portions/Servings"},
  "advance_order_days": {"type": "number", "label_ar": "الطلب المسبق (أيام)", "label_en": "Advance Order Required (Days)"},
  "delivery_to_venue": {"type": "boolean", "label_ar": "توصيل للقاعة", "label_en": "Delivery to Venue"},
  "tasting_session": {"type": "boolean", "label_ar": "جلسة تذوق", "label_en": "Tasting Session"},
  "gluten_free": {"type": "boolean", "label_ar": "خالٍ من الغلوتين", "label_en": "Gluten-Free Option"},
  "sugar_free": {"type": "boolean", "label_ar": "خالٍ من السكر", "label_en": "Sugar-Free Option"},
  "custom_design": {"type": "boolean", "label_ar": "تصميم مخصص", "label_en": "Custom Design"}
}', '["delivery_to_venue","tasting_session","gluten_free","sugar_free"]',
'["serves_max","advance_order_days"]', '[]'),

('wedding-sweets', '{
  "sweet_types": {"type": "text", "label_ar": "أنواع الحلويات", "label_en": "Sweet Types Offered"},
  "min_order": {"type": "number", "label_ar": "الحد الأدنى للطلب", "label_en": "Minimum Order"},
  "delivery_available": {"type": "boolean", "label_ar": "توصيل متاح", "label_en": "Delivery Available"},
  "custom_packaging": {"type": "boolean", "label_ar": "تغليف مخصص", "label_en": "Custom Branding/Packaging"},
  "calligraphy_labels": {"type": "boolean", "label_ar": "ملصقات خط عربي", "label_en": "Arabic Calligraphy Labels"},
  "sugar_free": {"type": "boolean", "label_ar": "خالٍ من السكر", "label_en": "Sugar-Free Option"}
}', '["delivery_available","custom_packaging","calligraphy_labels"]',
'["min_order"]', '[]'),

('entertainment-dj', '{
  "women_only_events": {"type": "boolean", "label_ar": "مناسبات نسائية", "label_en": "Women-Only Events Available"},
  "sound_system_included": {"type": "boolean", "label_ar": "نظام صوتي متضمن", "label_en": "Sound System Included"},
  "mc_service": {"type": "boolean", "label_ar": "خدمة مقدم حفل", "label_en": "MC / Emcee Service"},
  "lighting_rig": {"type": "boolean", "label_ar": "منظومة إضاءة", "label_en": "Lighting Rig Included"},
  "coverage_hours": {"type": "number", "label_ar": "ساعات التغطية", "label_en": "Hours of Coverage"},
  "gender_of_dj": {"type": "text", "label_ar": "جنس الفنان", "label_en": "DJ/Artist Gender (male/female)"},
  "troupe_size": {"type": "number", "label_ar": "حجم الفرقة", "label_en": "Troupe Size"},
  "female_troupe": {"type": "boolean", "label_ar": "فرقة نسائية متاحة", "label_en": "Female Troupe Available"},
  "instruments": {"type": "text", "label_ar": "الآلات الموسيقية", "label_en": "Instruments (drums/bagpipe/flame)"},
  "custom_song": {"type": "boolean", "label_ar": "أغنية مخصصة", "label_en": "Custom Song Performed"},
  "travel_available": {"type": "boolean", "label_ar": "تنقل لخارج المدينة", "label_en": "Travel Available"},
  "performance_type": {"type": "text", "label_ar": "نوع الأداء", "label_en": "Performance Type (nasheed/instrumental/mixed)"},
  "equipment_provided": {"type": "boolean", "label_ar": "معدات متضمنة", "label_en": "Equipment Provided"}
}', '["women_only_events","sound_system_included","mc_service","lighting_rig","female_troupe","travel_available","custom_song"]',
'[]', '["women_only_events","gender_of_dj"]'),

('wedding-jewelry', '{
  "metal_types": {"type": "text", "label_ar": "أنواع المعادن", "label_en": "Metal Types (gold/platinum/silver)"},
  "gold_karat_options": {"type": "text", "label_ar": "عيارات الذهب", "label_en": "Gold Karat Options (18/21/22K)"},
  "custom_design": {"type": "boolean", "label_ar": "تصميم مخصص", "label_en": "Custom Design Available"},
  "authenticity_certificate": {"type": "boolean", "label_ar": "شهادة أصالة", "label_en": "Authenticity Certificate"},
  "engraving": {"type": "boolean", "label_ar": "نقش", "label_en": "Engraving Available"},
  "installment_payment": {"type": "boolean", "label_ar": "تقسيط", "label_en": "Installment Payment"},
  "trade_in": {"type": "boolean", "label_ar": "استبدال المجوهرات", "label_en": "Trade-In Accepted"},
  "repair_service": {"type": "boolean", "label_ar": "خدمة إصلاح", "label_en": "Repair Service"}
}', '["custom_design","authenticity_certificate","engraving","installment_payment","trade_in"]',
'[]', '["gold_karat_options","authenticity_certificate"]'),

('wedding-gifts', '{
  "min_order_quantity": {"type": "number", "label_ar": "الحد الأدنى للطلب", "label_en": "Minimum Order Quantity"},
  "custom_branding": {"type": "boolean", "label_ar": "طباعة مخصصة", "label_en": "Custom Box Branding"},
  "delivery_available": {"type": "boolean", "label_ar": "توصيل متاح", "label_en": "Delivery Available"},
  "calligraphy_labels": {"type": "boolean", "label_ar": "ملصقات خط عربي", "label_en": "Arabic Calligraphy Labels"},
  "lead_time_days": {"type": "number", "label_ar": "مدة الإعداد (أيام)", "label_en": "Lead Time (Days)"},
  "rush_order": {"type": "boolean", "label_ar": "طلب عاجل", "label_en": "Rush Order Available"}
}', '["delivery_available","custom_branding","calligraphy_labels","rush_order"]',
'["min_order_quantity"]', '[]'),

('wedding-planner', '{
  "planning_type": {"type": "text", "label_ar": "نوع التخطيط", "label_en": "Planning Type (full/partial/day-of)"},
  "events_per_year": {"type": "number", "label_ar": "حفلات سنوياً", "label_en": "Events Managed Per Year"},
  "vendor_network": {"type": "boolean", "label_ar": "شبكة موردين", "label_en": "Vendor Network"},
  "day_of_coordination": {"type": "boolean", "label_ar": "تنسيق يوم الحفل", "label_en": "Day-of Coordination"},
  "budget_management": {"type": "boolean", "label_ar": "إدارة الميزانية", "label_en": "Budget Management"},
  "international_experience": {"type": "boolean", "label_ar": "خبرة دولية", "label_en": "International Weddings Experience"},
  "khosha_design": {"type": "boolean", "label_ar": "تصميم الكوشة", "label_en": "Khosha Design Included"},
  "floral_integration": {"type": "boolean", "label_ar": "زهور متضمنة", "label_en": "Floral Arrangements Included"},
  "lighting_design": {"type": "boolean", "label_ar": "تصميم إضاءة", "label_en": "Lighting Design"},
  "setup_teardown": {"type": "boolean", "label_ar": "تركيب وفك", "label_en": "Setup & Teardown Service"},
  "led_neon_signs": {"type": "boolean", "label_ar": "لافتات LED", "label_en": "LED/Neon Signage"},
  "ceiling_draping": {"type": "boolean", "label_ar": "تعليق الأقمشة للسقف", "label_en": "Ceiling Draping"},
  "photo_corner": {"type": "boolean", "label_ar": "ركن تصوير", "label_en": "Photo Corner Setup"}
}', '["day_of_coordination","budget_management","vendor_network","floral_integration","lighting_design","setup_teardown","led_neon_signs","ceiling_draping","photo_corner"]',
'["planning_type"]', '[]'),

('flowers-floral', '{
  "fresh_flowers": {"type": "boolean", "label_ar": "زهور طازجة", "label_en": "Fresh Flowers"},
  "artificial_flowers": {"type": "boolean", "label_ar": "زهور صناعية", "label_en": "Artificial Flowers"},
  "delivery_setup": {"type": "boolean", "label_ar": "توصيل وتركيب في القاعة", "label_en": "Delivery & Setup at Venue"},
  "bridal_bouquet": {"type": "boolean", "label_ar": "باقة العروس متضمنة", "label_en": "Bridal Bouquet Included"},
  "centerpieces": {"type": "boolean", "label_ar": "زينة الطاولات", "label_en": "Table Centerpieces"},
  "imported_flowers": {"type": "boolean", "label_ar": "زهور مستوردة", "label_en": "Imported Flowers"}
}', '["fresh_flowers","delivery_setup","bridal_bouquet","centerpieces"]',
'[]', '[]'),

('wedding-invitation', '{
  "min_order_quantity": {"type": "number", "label_ar": "الحد الأدنى للطلب", "label_en": "Minimum Order Quantity"},
  "turnaround_days": {"type": "number", "label_ar": "مدة التنفيذ (أيام)", "label_en": "Turnaround Time (Days)"},
  "custom_design": {"type": "boolean", "label_ar": "تصميم مخصص", "label_en": "Custom Design Service"},
  "digital_invite": {"type": "boolean", "label_ar": "دعوة رقمية / واتساب", "label_en": "Digital/WhatsApp Invite"},
  "bilingual": {"type": "boolean", "label_ar": "ثنائي اللغة (عربي/إنجليزي)", "label_en": "Bilingual (Arabic + English)"},
  "foil_printing": {"type": "boolean", "label_ar": "طباعة بالرقائق", "label_en": "Foil Printing"},
  "calligraphy_service": {"type": "boolean", "label_ar": "خط عربي", "label_en": "Calligraphy Service"},
  "rush_order": {"type": "boolean", "label_ar": "طلب عاجل", "label_en": "Rush Order Available"}
}', '["digital_invite","bilingual","calligraphy_service","rush_order"]',
'["min_order_quantity","turnaround_days"]', '["bilingual"]'),

('lighting-av', '{
  "led_wall": {"type": "boolean", "label_ar": "شاشة LED", "label_en": "LED Wall/Screen"},
  "moving_heads": {"type": "boolean", "label_ar": "إضاءة متحركة", "label_en": "Moving Head Lights"},
  "projection_mapping": {"type": "boolean", "label_ar": "إسقاط ضوئي", "label_en": "Projection Mapping"},
  "crew_included": {"type": "boolean", "label_ar": "فريق تقني متضمن", "label_en": "Technical Crew Included"},
  "generator_provided": {"type": "boolean", "label_ar": "مولد كهرباء", "label_en": "Generator Provided"},
  "live_streaming": {"type": "boolean", "label_ar": "بث مباشر", "label_en": "Live Streaming Capability"}
}', '["led_wall","moving_heads","projection_mapping","crew_included","live_streaming"]',
'["crew_included"]', '[]'),

('wedding-car', '{
  "vehicle_models": {"type": "text", "label_ar": "موديلات السيارات", "label_en": "Vehicle Models Available"},
  "chauffeur_included": {"type": "boolean", "label_ar": "سائق متضمن", "label_en": "Chauffeur Included"},
  "decoration_service": {"type": "boolean", "label_ar": "زينة السيارة", "label_en": "Car Decoration Service"},
  "fleet_size": {"type": "number", "label_ar": "حجم الأسطول", "label_en": "Fleet Size"},
  "hourly_rate_sar": {"type": "number", "label_ar": "الأجر بالساعة (ريال)", "label_en": "Hourly Rate (SAR)"},
  "multi_vehicle_package": {"type": "boolean", "label_ar": "باقة سيارات متعددة", "label_en": "Multi-Vehicle Package"}
}', '["chauffeur_included","decoration_service","multi_vehicle_package"]',
'["fleet_size"]', '[]');

-- Phase 1 — Core categories
UPDATE categories SET launch_phase = 1 WHERE slug IN (
    'wedding-palace', 'hotel-venue', 'villa-resort', 'chalet',
    'wedding-gown', 'abaya-jalabiya', 'groom-attire',
    'hair-makeup', 'henna-art',
    'photography-video',
    'catering', 'wedding-cake', 'wedding-sweets',
    'entertainment-dj',
    'wedding-jewelry', 'wedding-gifts',
    'flowers-floral', 'wedding-invitation'
);

-- Phase 2 — Expansion categories
UPDATE categories SET launch_phase = 2 WHERE slug IN (
    'restaurant-event', 'outdoor-garden', 'haute-couture',
    'photo-studio', 'wedding-planner', 'lighting-av',
    'wedding-car'
);

-- Phase 3 — Future categories (UAE/Qatar specific)
UPDATE categories SET launch_phase = 3 WHERE slug IN (
    'rooftop-venue', 'private-beach'
);

-- Populate inquiry_fields for the most critical Phase 1 categories.
-- Others inherit an empty array and will use the generic inquiry form.

UPDATE category_schemas SET inquiry_fields = '[
  "event_date", "men_guest_count", "women_guest_count",
  "event_type", "external_catering", "special_requests",
  "contact_name", "contact_phone", "budget_range"
]'::jsonb WHERE category_slug = 'wedding-palace';

UPDATE category_schemas SET inquiry_fields = '[
  "event_date", "men_guest_count", "women_guest_count",
  "event_type", "external_catering", "contact_name", "contact_phone"
]'::jsonb WHERE category_slug = 'hotel-venue';

UPDATE category_schemas SET inquiry_fields = '[
  "event_date", "services_needed", "female_artist_required",
  "events_covered", "location_type", "bridal_party_size",
  "contact_phone"
]'::jsonb WHERE category_slug = 'hair-makeup';

UPDATE category_schemas SET inquiry_fields = '[
  "event_date", "coverage_hours", "services_needed",
  "female_photographer", "womens_hall_coverage", "event_type",
  "venue_city", "contact_phone"
]'::jsonb WHERE category_slug = 'photography-video';

UPDATE category_schemas SET inquiry_fields = '[
  "event_date", "guest_count", "service_type", "meal_type",
  "cuisine_preference", "event_city", "contact_name", "contact_phone",
  "dietary_requirements"
]'::jsonb WHERE category_slug = 'catering';

UPDATE category_schemas SET inquiry_fields = '[
  "event_date", "female_troupe_required", "instruments",
  "performance_location", "custom_song", "contact_phone"
]'::jsonb WHERE category_slug = 'zaffa';

UPDATE category_schemas SET inquiry_fields = '[
  "event_date", "portions_needed", "delivery_required",
  "design_description", "contact_phone", "flavor_preferences"
]'::jsonb WHERE category_slug = 'wedding-cake';

UPDATE category_schemas SET inquiry_fields = '[
  "item_type", "metal_type", "gold_karat", "budget_range",
  "custom_design", "contact_phone"
]'::jsonb WHERE category_slug = 'wedding-jewelry';

-- ─── 9. Set open Decision A default: fashion categories use inquiry-only ──────
-- Ensures the `inquiry_fields` for fashion categories triggers the appointment-based flow.

UPDATE category_schemas SET inquiry_fields = '[
  "appointment_date", "size_info", "style_preference",
  "rental_or_purchase", "contact_phone", "contact_whatsapp"
]'::jsonb WHERE category_slug IN ('wedding-gown', 'abaya-jalabiya', 'groom-attire');

-- ─── 10. Set Decision B default: flowers floral filter uses fresh/artificial ──

UPDATE category_schemas SET filter_fields = '[
  {"field": "city_id", "type": "select", "priority": "critical"},
  {"field": "fresh_flowers", "type": "checkbox", "priority": "useful"},
  {"field": "delivery_setup", "type": "checkbox", "priority": "useful"},
  {"field": "base_price_sar", "type": "range", "priority": "useful"}
]'::jsonb WHERE category_slug = 'flowers-floral';

-- 3. Seed Initial Data
INSERT INTO features (name_en, name_ar, category, input_type) VALUES
-- General Info
('Venue ID', 'رقم تعريف المكان', 'General Info', 'string'),
('Hotel name', 'اسم الفندق', 'General Info', 'string'),
('Hotel number', 'رقم الفندق', 'General Info', 'string'),
('City', 'المدينة', 'General Info', 'string'),
('Number of hotel stars', 'عدد النجوم', 'General Info', 'number'),
('Number of hotel rooms', 'عدد الغرف', 'General Info', 'number'),
('The address', 'العنوان', 'General Info', 'string'),
('Location link', 'رابط الموقع', 'General Info', 'string'),
('Hotel capacity', 'سعة الفندق', 'General Info', 'number'),
('The existing halls', 'القاعات المتاحة', 'General Info', 'string'),
('Hall capacity', 'سعة القاعة', 'General Info', 'number'),
('Price per person', 'السعر للشخص', 'General Info', 'number'),
('Admin number', 'رقم المسؤول', 'General Info', 'string'),
('Supervisor name', 'اسم المشرف', 'General Info', 'string'),
('What events can be hosted?', 'أنواع المناسبات', 'General Info', 'string'),
('Pictures of the halls', 'صور القاعات', 'General Info', 'string'), -- Maybe this is better handled by gallery?

-- Amenities
('Hall supervisor', 'مشرف القاعة', 'Amenities', 'boolean'),
('Separate dining hall', 'قاعة طعام منفصلة', 'Amenities', 'boolean'),
('Female workers', 'موظفات', 'Amenities', 'boolean'),
('Valves and fittings', 'صمامات وتجهيزات', 'Amenities', 'boolean'),
('Abayas official', 'عبايات رسمية', 'Amenities', 'boolean'),
('Mobile phone inspector', 'مفتش الجوالات', 'Amenities', 'boolean'),
('Coffee server for men', 'قهوجي للرجال', 'Amenities', 'boolean'),
('Separate entrance for the bride', 'مدخل منفصل للعروس', 'Amenities', 'boolean'),
('Staircase for the wedding procession', 'درج للزفة', 'Amenities', 'boolean'),
('Preparation room for the bride', 'غرفة تجهيز العروس', 'Amenities', 'boolean'),
('Laser', 'ليزر', 'Amenities', 'boolean'),
('Steam', 'بخار', 'Amenities', 'boolean'),
('DJ / Audio Equipment', 'دي جي ومعدات صوت', 'Amenities', 'boolean'),
('The stage and hall decoration', 'المسرح وتزيين القاعة', 'Amenities', 'boolean'),
('Hot and cold drinks', 'مشروبات ساخنة وباردة', 'Amenities', 'boolean'),
('Wedding cake', 'كيكة زفاف', 'Amenities', 'boolean'),
('Cooking carcasses', 'طبخ أكارع', 'Amenities', 'boolean'),
('Car parking', 'موقف سيارات', 'Amenities', 'boolean'),
('Photography and video', 'تصوير وفيديو', 'Amenities', 'boolean'),
('Outdoor space for events', 'مساحة خارجية', 'Amenities', 'boolean'),
('Open buffet', 'بوفيه مفتوح', 'Amenities', 'boolean'),
('Free suite for newlyweds', 'جناح مجاني للعروسين', 'Amenities', 'boolean'),
('Meeting room', 'قاعة اجتماعات', 'Amenities', 'boolean'),
('Lighting', 'إضاءة', 'Amenities', 'boolean'),
('Possibility of holding several parties at the same time', 'إمكانية إقامة أكثر من حفل', 'Amenities', 'boolean');

