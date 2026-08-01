--
-- Migration: 0015_seed_category_schemas
-- Description: Seed the category_schemas table for all 31 categories of ZafafWorld with proper validations.
--

-- Ensure the 4 extra active categories exist in categories table first (to prevent foreign key violations on fresh databases)
INSERT INTO public.categories (slug, name_ar, name_en, parent_group, priority, launch_phase)
VALUES
  ('zaffa', 'زفة ومسيرة', 'Zaffa & Procession', 'entertainment', 'important', 1),
  ('nasheed-band', 'إنشاد وفرقة موسيقية', 'Nasheed & Band', 'entertainment', 'important', 1),
  ('khosha-decor', 'كوشة وتزيين المسرح', 'Khosha & Stage Decor', 'planning_decor', 'important', 1),
  ('beauty-skincare', 'عناية بالبشرة وتجميل', 'Skincare & Beauty', 'beauty', 'important', 1)
ON CONFLICT (slug) DO NOTHING;

-- Enable schema population for all categories
INSERT INTO public.category_schemas (
    category_slug,
    schema_version,
    schema_json,
    searchable_fields,
    required_fields,
    gcc_required_fields,
    inquiry_fields,
    filter_fields,
    capacity_mode,
    capacity_key,
    is_active
) VALUES

-- ── VENUES ───────────────────────────────────────────────────────────────────
('wedding-palace', 1, '{
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
'["men_capacity","women_capacity"]', '["has_separate_entrances","prayer_room","external_catering_allowed"]',
'[
  "event_date", "men_guest_count", "women_guest_count",
  "event_type", "external_catering", "special_requests",
  "contact_name", "contact_phone", "budget_range"
]', '[]', 'sum_men_women', 'men_capacity,women_capacity', true),

('hotel-venue', 1, '{
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
'["ballroom_capacity"]', '["prayer_room","halal_kitchen"]',
'[
  "event_date", "men_guest_count", "women_guest_count",
  "event_type", "external_catering", "contact_name", "contact_phone"
]', '[]', 'single_key', 'ballroom_capacity', true),

('villa-resort', 1, '{
  "max_capacity": {"type": "number", "label_ar": "الطاقة الاستيعابية", "label_en": "Max Guest Capacity"},
  "num_bedrooms": {"type": "number", "label_ar": "عدد غرف النوم", "label_en": "Number of Bedrooms"},
  "private_pool": {"type": "boolean", "label_ar": "مسبح خاص", "label_en": "Private Pool"},
  "external_catering_allowed": {"type": "boolean", "label_ar": "ضيافة خارجية مسموحة", "label_en": "External Catering Allowed"},
  "full_exclusivity": {"type": "boolean", "label_ar": "حجز حصري كامل", "label_en": "Full Property Exclusivity"},
  "weekend_surcharge_sar": {"type": "number", "label_ar": "رسوم إضافية للعطلة (ريال)", "label_en": "Weekend Surcharge (SAR)"},
  "minimum_rental_hours": {"type": "number", "label_ar": "الحد الأدنى لساعات الإيجار", "label_en": "Minimum Rental Hours"},
  "outdoor_bbq": {"type": "boolean", "label_ar": "مشواة خارجية", "label_en": "Outdoor BBQ"}
}', '["private_pool","external_catering_allowed","full_exclusivity","outdoor_bbq"]',
'["max_capacity"]', '["external_catering_allowed"]',
'[
  "event_date", "guest_count", "rental_hours", "contact_phone"
]', '[]', 'single_key', 'max_capacity', true),

('restaurant-event', 1, '{
  "private_hall_capacity": {"type": "number", "label_ar": "سعة القاعة الخاصة", "label_en": "Private Hall Capacity"},
  "family_section": {"type": "boolean", "label_ar": "قسم عائلي", "label_en": "Family Section"},
  "private_entrance": {"type": "boolean", "label_ar": "مدخل خاص", "label_en": "Private Entrance"},
  "halal_certified": {"type": "boolean", "label_ar": "شهادة حلال", "label_en": "Halal Certified"},
  "outdoor_seating": {"type": "boolean", "label_ar": "جلسات خارجية", "label_en": "Outdoor Seating"},
  "av_equipment": {"type": "boolean", "label_ar": "معدات صوت وصورة", "label_en": "AV Equipment"},
  "minimum_spend_sar": {"type": "number", "label_ar": "الحد الأدنى للإنفاق (ريال)", "label_en": "Minimum Spend (SAR)"}
}', '["family_section","halal_certified","private_entrance","outdoor_seating"]',
'["private_hall_capacity"]', '["halal_certified","family_section"]',
'[
  "event_date", "guest_count", "contact_name", "contact_phone"
]', '[]', 'single_key', 'private_hall_capacity', true),

('outdoor-garden', 1, '{
  "seated_capacity": {"type": "number", "label_ar": "سعة الجلوس", "label_en": "Seated Capacity"},
  "tent_included": {"type": "boolean", "label_ar": "خيمة أو مظلة متضمنة", "label_en": "Tent/Canopy Included"},
  "backup_indoor": {"type": "boolean", "label_ar": "بديل داخلي متاح", "label_en": "Backup Indoor Space"},
  "lighting_included": {"type": "boolean", "label_ar": "إضاءة متضمنة", "label_en": "Lighting Package"},
  "available_months": {"type": "text", "label_ar": "الأشهر المتاحة", "label_en": "Available Months (Oct–Mar)"},
  "external_catering_allowed": {"type": "boolean", "label_ar": "ضيافة خارجية مسموحة", "label_en": "External Catering Allowed"}
}', '["tent_included","backup_indoor","lighting_included"]',
'["seated_capacity"]', '["tent_included"]',
'[
  "event_date", "guest_count", "contact_phone"
]', '[]', 'single_key', 'seated_capacity', true),

('rooftop-venue', 1, '{
  "capacity": {"type": "number", "label_ar": "الطاقة الاستيعابية", "label_en": "Capacity"},
  "elevator_access": {"type": "boolean", "label_ar": "مصعد", "label_en": "Elevator Access"},
  "wind_protection": {"type": "boolean", "label_ar": "حواجز مضادة للريح", "label_en": "Wind Protection"},
  "city_view": {"type": "boolean", "label_ar": "إطلالة على المدينة", "label_en": "City View"},
  "noise_curfew_time": {"type": "text", "label_ar": "وقت إنهاء الصوت", "label_en": "Noise Curfew Time"}
}', '["elevator_access","wind_protection","city_view"]',
'["capacity"]', '[]',
'[
  "event_date", "guest_count", "contact_phone"
]', '[]', 'single_key', 'capacity', true),

('private-beach', 1, '{
  "capacity": {"type": "number", "label_ar": "الطاقة الاستيعابية", "label_en": "Capacity"},
  "private_or_semi": {"type": "text", "label_ar": "خاص أم شبه خاص", "label_en": "Private or Semi-Private"},
  "floating_stage": {"type": "boolean", "label_ar": "منصة عائمة", "label_en": "Floating Stage"},
  "shade_structures": {"type": "boolean", "label_ar": "مظلات", "label_en": "Shade Structures"}
}', '["floating_stage","shade_structures"]',
'["capacity"]', '[]',
'[
  "event_date", "guest_count", "contact_phone"
]', '[]', 'single_key', 'capacity', true),

('chalet', 1, '{
  "num_units": {"type": "number", "label_ar": "عدد الوحدات", "label_en": "Number of Units"},
  "total_capacity": {"type": "number", "label_ar": "الطاقة الاستيعابية الكلية", "label_en": "Total Capacity"},
  "private_pool": {"type": "boolean", "label_ar": "مسبح خاص", "label_en": "Private Pool"},
  "bbq": {"type": "boolean", "label_ar": "مشواة", "label_en": "BBQ Facilities"},
  "separate_sections": {"type": "boolean", "label_ar": "أقسام منفصلة رجال ونساء", "label_en": "Separate Men/Women Sections"},
  "kitchen_facilities": {"type": "boolean", "label_ar": "مرافق مطبخ", "label_en": "Kitchen Facilities"},
  "security_included": {"type": "boolean", "label_ar": "أمن متضمن", "label_en": "Security Guards Included"},
  "external_catering_allowed": {"type": "boolean", "label_ar": "ضيافة خارجية مسموحة", "label_en": "External Catering Allowed"}
}', '["private_pool","bbq","separate_sections","kitchen_facilities"]',
'["total_capacity"]', '["separate_sections","external_catering_allowed"]',
'[
  "event_date", "guest_count", "contact_phone"
]', '[]', 'single_key', 'total_capacity', true),

-- ── FASHION ──────────────────────────────────────────────────────────────────
('wedding-gown', 1, '{
  "delivery_time_weeks": {"type": "number", "label_ar": "مدة التصنيع (أسابيع)", "label_en": "Custom Order Lead Time (Weeks)"},
  "ready_to_wear": {"type": "boolean", "label_ar": "جاهز للارتداء", "label_en": "Ready-to-Wear Available"},
  "custom_bespoke": {"type": "boolean", "label_ar": "تصميم مخصص", "label_en": "Custom / Bespoke Design"},
  "alterations_service": {"type": "boolean", "label_ar": "خدمة الخياطة والتعديل", "label_en": "Alterations Service"},
  "rental_available": {"type": "boolean", "label_ar": "تأجير متاح", "label_en": "Rental Available"},
  "at_home_fitting": {"type": "boolean", "label_ar": "قياس في المنزل", "label_en": "At-Home Fitting"}
}', '["ready_to_wear","rental_available","alterations_service","at_home_fitting"]',
'[]', '[]',
'[
  "appointment_date", "size_info", "style_preference",
  "rental_or_purchase", "contact_phone", "contact_whatsapp"
]', '[]', 'none', null, true),

('haute-couture', 1, '{
  "lead_time_weeks": {"type": "number", "label_ar": "مدة التصميم (أسابيع)", "label_en": "Lead Time (Weeks)"},
  "fitting_sessions": {"type": "number", "label_ar": "جلسات القياس المتضمنة", "label_en": "Fitting Sessions Included"},
  "international_designer": {"type": "boolean", "label_ar": "مصمم دولي", "label_en": "International Designer"},
  "accessories_included": {"type": "boolean", "label_ar": "الإكسسوارات متضمنة", "label_en": "Accessories Included"}
}', '[]',
'["lead_time_weeks"]', '[]',
'[
  "appointment_date", "size_info", "style_preference",
  "rental_or_purchase", "contact_phone", "contact_whatsapp"
]', '[]', 'none', null, true),

('abaya-jalabiya', 1, '{
  "styles_offered": {"type": "text", "label_ar": "الأنواع المتاحة", "label_en": "Styles Offered (abaya/jalabiya/kaftan)"},
  "custom_available": {"type": "boolean", "label_ar": "تصميم مخصص", "label_en": "Custom Available"},
  "lead_time_days": {"type": "number", "label_ar": "مدة التصنيع (أيام)", "label_en": "Lead Time (Days)"},
  "rental_available": {"type": "boolean", "label_ar": "تأجير متاح", "label_en": "Rental Available"},
  "matching_accessories": {"type": "boolean", "label_ar": "إكسسوارات مطابقة", "label_en": "Matching Accessories"}
}', '["rental_available","custom_available"]',
'[]', '[]',
'[
  "appointment_date", "size_info", "style_preference",
  "rental_or_purchase", "contact_phone", "contact_whatsapp"
]', '[]', 'none', null, true),

('groom-attire', 1, '{
  "types_offered": {"type": "text", "label_ar": "الأنواع المتاحة", "label_en": "Types (bisht/thobe/suit/kandura)"},
  "custom_tailoring": {"type": "boolean", "label_ar": "خياطة مخصصة", "label_en": "Custom Tailoring"},
  "lead_time_days": {"type": "number", "label_ar": "مدة التصنيع (أيام)", "label_en": "Lead Time (Days)"},
  "gold_embroidery": {"type": "boolean", "label_ar": "تطريز ذهبي", "label_en": "Gold Thread Embroidery"}
}', '["custom_tailoring","gold_embroidery"]',
'[]', '[]',
'[
  "appointment_date", "size_info", "style_preference",
  "rental_or_purchase", "contact_phone", "contact_whatsapp"
]', '[]', 'none', null, true),

-- ── BEAUTY ───────────────────────────────────────────────────────────────────
('hair-makeup', 1, '{
  "female_only_staff": {"type": "boolean", "label_ar": "فريق نسائي فقط", "label_en": "Female-Only Staff / Salon"},
  "home_service": {"type": "boolean", "label_ar": "خدمة منزلية", "label_en": "Home/On-Location Service"},
  "trial_session": {"type": "boolean", "label_ar": "جلسة تجريبية", "label_en": "Trial Session Available"},
  "airbrush_makeup": {"type": "boolean", "label_ar": "مكياج بالرش", "label_en": "Airbrush Makeup"},
  "team_size": {"type": "number", "label_ar": "حجم الفريق", "label_en": "Team Size"},
  "services_offered": {"type": "text", "label_ar": "الخدمات المتاحة", "label_en": "Services (facial/laser/peel/body)"},
  "bride_package_weeks": {"type": "number", "label_ar": "مدة باقة العروس (أسابيع)", "label_en": "Bride Package Duration (Weeks)"},
  "licensed_clinic": {"type": "boolean", "label_ar": "عيادة مرخصة", "label_en": "Licensed Medical Clinic"}
}', '["female_only_staff","home_service","airbrush_makeup","trial_session","licensed_clinic"]',
'[]', '["female_only_staff"]',
'[
  "event_date", "services_needed", "female_artist_required",
  "events_covered", "location_type", "bridal_party_size",
  "contact_phone"
]', '[]', 'none', null, true),

('henna-art', 1, '{
  "home_service": {"type": "boolean", "label_ar": "خدمة منزلية", "label_en": "Home Service Available"},
  "bridal_package": {"type": "boolean", "label_ar": "باقة عروس (يدين وقدمين)", "label_en": "Bridal Henna (Hands & Feet)"},
  "natural_henna_only": {"type": "boolean", "label_ar": "حناء طبيعية 100%", "label_en": "100% Natural Henna Only"},
  "design_styles": {"type": "text", "label_ar": "أنماط التصميم", "label_en": "Design Styles (traditional/modern)"},
  "female_artist": {"type": "boolean", "label_ar": "فنانة حناء", "label_en": "Female Artist"}
}', '["home_service","natural_henna_only","female_artist","bridal_package"]',
'[]', '["natural_henna_only","female_artist"]',
'[
  "event_date", "services_needed", "contact_phone"
]', '[]', 'none', null, true),

('beauty-skincare', 1, '{
  "female_only_staff": {"type": "boolean", "label_ar": "فريق نسائي فقط", "label_en": "Female-Only Staff / Salon"},
  "home_service": {"type": "boolean", "label_ar": "خدمة منزلية", "label_en": "Home/On-Location Service"}
}', '["female_only_staff","home_service"]',
'[]', '["female_only_staff"]',
'[
  "event_date", "services_needed", "female_artist_required",
  "bridal_party_size", "contact_phone"
]', '[]', 'none', null, true),

-- ── PHOTOGRAPHY ──────────────────────────────────────────────────────────────
('photography-video', 1, '{
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
'[]', '["female_team_available","women_section_coverage"]',
'[
  "event_date", "coverage_hours", "services_needed",
  "female_photographer", "womens_hall_coverage", "event_type",
  "venue_city", "contact_phone"
]', '[]', 'none', null, true),

('photo-studio', 1, '{
  "session_types": {"type": "text", "label_ar": "أنواع الجلسات", "label_en": "Session Types (engagement/bridal/family)"},
  "outdoor_locations": {"type": "boolean", "label_ar": "مواقع خارجية", "label_en": "Outdoor Locations Offered"},
  "same_day_editing": {"type": "boolean", "label_ar": "تعديل نفس اليوم", "label_en": "Same-Day Editing"},
  "printed_albums": {"type": "boolean", "label_ar": "ألبومات مطبوعة", "label_en": "Printed Albums"}
}', '["outdoor_locations","same_day_editing"]',
'[]', '[]',
'[
  "event_date", "session_type", "contact_phone"
]', '[]', 'none', null, true),

-- ── FOOD ─────────────────────────────────────────────────────────────────────
('catering', 1, '{
  "min_guests": {"type": "number", "label_ar": "الحد الأدنى للضيوف", "label_en": "Minimum Guests Required"},
  "serving_staff_included": {"type": "boolean", "label_ar": "طاقم الخدمة متضمن", "label_en": "Serving Staff Included"},
  "halal_certified": {"type": "boolean", "label_ar": "شهادة حلال", "label_en": "Halal Certified"},
  "taste_testing": {"type": "boolean", "label_ar": "تذوق مسبق", "label_en": "Taste Testing Available"},
  "setup_cleanup": {"type": "boolean", "label_ar": "تجهيز وترتيب متضمن", "label_en": "Setup & Cleanup Included"},
  "equipment_rental": {"type": "boolean", "label_ar": "تأجير المعدات متضمن", "label_en": "Equipment Rental Included"},
  "buffet_or_plated": {"type": "text", "label_ar": "نوع الخدمة", "label_en": "Service Type (buffet/plated)"}
}', '["halal_certified","serving_staff_included","setup_cleanup","taste_testing"]',
'["min_guests"]', '["halal_certified"]',
'[
  "event_date", "guest_count", "service_type", "meal_type",
  "cuisine_preference", "event_city", "contact_name", "contact_phone",
  "dietary_requirements"
]', '[]', 'none', null, true),

('wedding-cake', 1, '{
  "serves_max": {"type": "number", "label_ar": "أقصى عدد حصص", "label_en": "Max Portions/Servings"},
  "advance_order_days": {"type": "number", "label_ar": "الطلب المسبق (أيام)", "label_en": "Advance Order Required (Days)"},
  "delivery_to_venue": {"type": "boolean", "label_ar": "توصيل للقاعة", "label_en": "Delivery to Venue"},
  "tasting_session": {"type": "boolean", "label_ar": "جلسة تذوق", "label_en": "Tasting Session"},
  "gluten_free": {"type": "boolean", "label_ar": "خالٍ من الغلوتين", "label_en": "Gluten-Free Option"},
  "sugar_free": {"type": "boolean", "label_ar": "خالٍ من السكر", "label_en": "Sugar-Free Option"},
  "custom_design": {"type": "boolean", "label_ar": "تصميم مخصص", "label_en": "Custom Design"}
}', '["delivery_to_venue","tasting_session","gluten_free","sugar_free"]',
'["serves_max","advance_order_days"]', '[]',
'[
  "event_date", "portions_needed", "delivery_required",
  "design_description", "contact_phone", "flavor_preferences"
]', '[]', 'none', null, true),

('wedding-sweets', 1, '{
  "sweet_types": {"type": "text", "label_ar": "أنواع الحلويات", "label_en": "Sweet Types Offered"},
  "min_order": {"type": "number", "label_ar": "الحد الأدنى للطلب", "label_en": "Minimum Order"},
  "delivery_available": {"type": "boolean", "label_ar": "توصيل متاح", "label_en": "Delivery Available"},
  "custom_packaging": {"type": "boolean", "label_ar": "تغليف مخصص", "label_en": "Custom Branding/Packaging"},
  "calligraphy_labels": {"type": "boolean", "label_ar": "ملصقات خط عربي", "label_en": "Arabic Calligraphy Labels"},
  "sugar_free": {"type": "boolean", "label_ar": "خالٍ من السكر", "label_en": "Sugar-Free Option"}
}', '["delivery_available","custom_packaging","calligraphy_labels"]',
'["min_order"]', '[]',
'[
  "event_date", "quantity_needed", "contact_phone"
]', '[]', 'none', null, true),

-- ── ENTERTAINMENT ────────────────────────────────────────────────────────────
('entertainment-dj', 1, '{
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
'[]', '["women_only_events","gender_of_dj"]',
'[
  "event_date", "hours_needed", "contact_phone"
]', '[]', 'none', null, true),

('zaffa', 1, '{
  "women_only_events": {"type": "boolean", "label_ar": "مناسبات نسائية", "label_en": "Women-Only Events Available"},
  "female_troupe": {"type": "boolean", "label_ar": "فرقة نسائية متاحة", "label_en": "Female Troupe Available"}
}', '["women_only_events", "female_troupe"]',
'[]', '["women_only_events"]',
'[
  "event_date", "female_troupe_required", "instruments",
  "performance_location", "custom_song", "contact_phone"
]', '[]', 'none', null, true),

('nasheed-band', 1, '{
  "women_only_events": {"type": "boolean", "label_ar": "مناسبات نسائية", "label_en": "Women-Only Events Available"},
  "female_troupe": {"type": "boolean", "label_ar": "فرقة نسائية متاحة", "label_en": "Female Troupe Available"}
}', '["women_only_events", "female_troupe"]',
'[]', '["women_only_events"]',
'[
  "event_date", "female_troupe_required", "instruments",
  "performance_location", "custom_song", "contact_phone"
]', '[]', 'none', null, true),

-- ── JEWELRY & GIFTS ──────────────────────────────────────────────────────────
('wedding-jewelry', 1, '{
  "metal_types": {"type": "text", "label_ar": "أنواع المعادن", "label_en": "Metal Types (gold/platinum/silver)"},
  "gold_karat_options": {"type": "text", "label_ar": "عيارات الذهب", "label_en": "Gold Karat Options (18/21/22K)"},
  "custom_design": {"type": "boolean", "label_ar": "تصميم مخصص", "label_en": "Custom Design Available"},
  "authenticity_certificate": {"type": "boolean", "label_ar": "شهادة أصالة", "label_en": "Authenticity Certificate"},
  "engraving": {"type": "boolean", "label_ar": "نقش", "label_en": "Engraving Available"},
  "installment_payment": {"type": "boolean", "label_ar": "تقسيط", "label_en": "Installment Payment"},
  "trade_in": {"type": "boolean", "label_ar": "استبدال المجوهرات", "label_en": "Trade-In Accepted"},
  "repair_service": {"type": "boolean", "label_ar": "خدمة إصلاح", "label_en": "Repair Service"}
}', '["custom_design","authenticity_certificate","engraving","installment_payment","trade_in"]',
'[]', '["gold_karat_options","authenticity_certificate"]',
'[
  "item_type", "metal_type", "gold_karat", "budget_range",
  "custom_design", "contact_phone"
]', '[]', 'none', null, true),

('wedding-gifts', 1, '{
  "min_order_quantity": {"type": "number", "label_ar": "الحد الأدنى للطلب", "label_en": "Minimum Order Quantity"},
  "custom_branding": {"type": "boolean", "label_ar": "طباعة مخصصة", "label_en": "Custom Box Branding"},
  "delivery_available": {"type": "boolean", "label_ar": "توصيل متاح", "label_en": "Delivery Available"},
  "calligraphy_labels": {"type": "boolean", "label_ar": "ملصقات خط عربي", "label_en": "Arabic Calligraphy Labels"},
  "lead_time_days": {"type": "number", "label_ar": "مدة الإعداد (أيام)", "label_en": "Lead Time (Days)"},
  "rush_order": {"type": "boolean", "label_ar": "طلب عاجل", "label_en": "Rush Order Available"}
}', '["delivery_available","custom_branding","calligraphy_labels","rush_order"]',
'["min_order_quantity"]', '[]',
'[
  "event_date", "quantity_needed", "contact_phone"
]', '[]', 'none', null, true),

-- ── PLANNING & DECOR ─────────────────────────────────────────────────────────
('wedding-planner', 1, '{
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
'["planning_type"]', '[]',
'[
  "event_date", "budget_range", "contact_phone"
]', '[]', 'none', null, true),

('khosha-decor', 1, '{
  "setup_teardown": {"type": "boolean", "label_ar": "تركيب وفك", "label_en": "Setup & Teardown Service"},
  "khosha_design": {"type": "boolean", "label_ar": "تصميم الكوشة", "label_en": "Khosha Design Included"}
}', '["setup_teardown", "khosha_design"]',
'[]', '[]',
'[
  "event_date", "contact_phone"
]', '[]', 'none', null, true),

('flowers-floral', 1, '{
  "fresh_flowers": {"type": "boolean", "label_ar": "زهور طازجة", "label_en": "Fresh Flowers"},
  "artificial_flowers": {"type": "boolean", "label_ar": "زهور صناعية", "label_en": "Artificial Flowers"},
  "delivery_setup": {"type": "boolean", "label_ar": "توصيل وتركيب في القاعة", "label_en": "Delivery & Setup at Venue"},
  "bridal_bouquet": {"type": "boolean", "label_ar": "باقة العروس متضمنة", "label_en": "Bridal Bouquet Included"},
  "centerpieces": {"type": "boolean", "label_ar": "زينة الطاولات", "label_en": "Table Centerpieces"},
  "imported_flowers": {"type": "boolean", "label_ar": "زهور مستوردة", "label_en": "Imported Flowers"}
}', '["fresh_flowers","delivery_setup","bridal_bouquet","centerpieces"]',
'[]', '[]',
'[
  "event_date", "decor_type", "contact_phone"
]', '[
  {"field": "city_id", "type": "select", "priority": "critical"},
  {"field": "fresh_flowers", "type": "checkbox", "priority": "useful"},
  {"field": "delivery_setup", "type": "checkbox", "priority": "useful"},
  {"field": "base_price_sar", "type": "range", "priority": "useful"}
]', 'none', null, true),

('wedding-invitation', 1, '{
  "min_order_quantity": {"type": "number", "label_ar": "الحد الأدنى للطلب", "label_en": "Minimum Order Quantity"},
  "turnaround_days": {"type": "number", "label_ar": "مدة التنفيذ (أيام)", "label_en": "Turnaround Time (Days)"},
  "custom_design": {"type": "boolean", "label_ar": "تصميم مخصص", "label_en": "Custom Design Service"},
  "digital_invite": {"type": "boolean", "label_ar": "دعوة رقمية / واتساب", "label_en": "Digital/WhatsApp Invite"},
  "bilingual": {"type": "boolean", "label_ar": "ثنائي اللغة (عربي/إنجليزي)", "label_en": "Bilingual (Arabic + English)"},
  "foil_printing": {"type": "boolean", "label_ar": "طباعة بالرقائق", "label_en": "Foil Printing"},
  "calligraphy_service": {"type": "boolean", "label_ar": "خط عربي", "label_en": "Calligraphy Service"},
  "rush_order": {"type": "boolean", "label_ar": "طلب عاجل", "label_en": "Rush Order Available"}
}', '["digital_invite","bilingual","calligraphy_service","rush_order"]',
'["min_order_quantity","turnaround_days"]', '["bilingual"]',
'[
  "event_date", "quantity_needed", "contact_phone"
]', '[]', 'none', null, true),

('lighting-av', 1, '{
  "led_wall": {"type": "boolean", "label_ar": "شاشة LED", "label_en": "LED Wall/Screen"},
  "moving_heads": {"type": "boolean", "label_ar": "إضاءة متحركة", "label_en": "Moving Head Lights"},
  "projection_mapping": {"type": "boolean", "label_ar": "إسقاط ضوئي", "label_en": "Projection Mapping"},
  "crew_included": {"type": "boolean", "label_ar": "فريق تقني متضمن", "label_en": "Technical Crew Included"},
  "generator_provided": {"type": "boolean", "label_ar": "مولد كهرباء", "label_en": "Generator Provided"},
  "live_streaming": {"type": "boolean", "label_ar": "بث مباشر", "label_en": "Live Streaming Capability"}
}', '["led_wall","moving_heads","projection_mapping","crew_included","live_streaming"]',
'["crew_included"]', '[]',
'[
  "event_date", "equipment_needed", "contact_phone"
]', '[]', 'none', null, true),

-- ── TRANSPORTATION ───────────────────────────────────────────────────────────
('wedding-car', 1, '{
  "vehicle_models": {"type": "text", "label_ar": "موديلات السيارات", "label_en": "Vehicle Models Available"},
  "chauffeur_included": {"type": "boolean", "label_ar": "سائق متضمن", "label_en": "Chauffeur Included"},
  "decoration_service": {"type": "boolean", "label_ar": "زينة السيارة", "label_en": "Car Decoration Service"},
  "fleet_size": {"type": "number", "label_ar": "حجم الأسطول", "label_en": "Fleet Size"},
  "hourly_rate_sar": {"type": "number", "label_ar": "الأجر بالساعة (ريال)", "label_en": "Hourly Rate (SAR)"},
  "multi_vehicle_package": {"type": "boolean", "label_ar": "باقة سيارات متعددة", "label_en": "Multi-Vehicle Package"}
}', '["chauffeur_included","decoration_service","multi_vehicle_package"]',
'["fleet_size"]', '[]',
'[
  "event_date", "rental_hours", "contact_phone"
]', '[]', 'none', null, true)

ON CONFLICT (category_slug) DO UPDATE SET
    schema_version = EXCLUDED.schema_version,
    schema_json = EXCLUDED.schema_json,
    searchable_fields = EXCLUDED.searchable_fields,
    required_fields = EXCLUDED.required_fields,
    gcc_required_fields = EXCLUDED.gcc_required_fields,
    inquiry_fields = EXCLUDED.inquiry_fields,
    filter_fields = EXCLUDED.filter_fields,
    capacity_mode = EXCLUDED.capacity_mode,
    capacity_key = EXCLUDED.capacity_key,
    is_active = EXCLUDED.is_active,
    updated_at = NOW();
