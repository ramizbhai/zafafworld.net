export interface SchemaField {
    fieldId: string;
    required: boolean;
    inputType?: 'text' | 'number' | 'boolean' | 'dropdown' | 'textarea' | 'radio' | 'multiselect';
    labelEn?: string;
    labelAr?: string;
    options?: { labelEn: string; labelAr: string; value: string }[];
    visibleWhen?: Record<string, any>;
}

export interface SchemaSection {
    titleEn: string;
    titleAr: string;
    fields: SchemaField[];
}

export interface SchemaFeatureOption {
    optionId: string;
    labelEn: string;
    labelAr: string;
}

export interface SchemaFeatureGroup {
    groupId: string;
    titleEn: string;
    titleAr: string;
    options: SchemaFeatureOption[];
}

export interface CategorySchema {
    categoryId: string;
    usesCulturalSettings: boolean;
    stepOverrides: Record<string, { status: string; reasonEn: string; reasonAr: string }>;
    detailSections: SchemaSection[];
    featureGroups: SchemaFeatureGroup[];
}

const defaultStepOverrides = (usesCultural: boolean): Record<string, { status: string; reasonEn: string; reasonAr: string }> => {
    if (usesCultural) return {};
    return {
        "4": {
            status: "not_applicable",
            reasonEn: "Cultural settings not applicable for this category",
            reasonAr: "الإعدادات الثقافية غير قابلة للتطبيق على هذه الفئة"
        }
    };
};

// Helper for generic options
const field = (id: string, labelEn: string, labelAr: string, type: 'text' | 'number' | 'boolean' | 'dropdown' = 'text', options: any = undefined, required = false): SchemaField => ({
    fieldId: id,
    labelEn,
    labelAr,
    inputType: type,
    options,
    required
});

const featureOption = (id: string, en: string, ar: string) => ({ optionId: id, labelEn: en, labelAr: ar });

export function getSchemaForCategory(categoryId: string): CategorySchema | null {
    // 1. Wedding Favors (wedding-gifts)
    if (categoryId === 'wedding-gifts') {
        return {
            categoryId,
            usesCulturalSettings: false,
            stepOverrides: defaultStepOverrides(false),
            detailSections: [
                {
                    titleEn: "General Info",
                    titleAr: "معلومات عامة",
                    fields: [
                        field('delivery_period', 'Delivery Period', 'مدة التوصيل', 'text'),
                        field('min_capacity', 'Minimum Capacity', 'أقل سعة', 'number'),
                        field('max_capacity', 'Maximum Capacity', 'أكبر سعة', 'number')
                    ]
                }
            ],
            featureGroups: [
                {
                    groupId: "additional_features",
                    titleEn: "Additional Features (Optional)",
                    titleAr: "الميزات الإضافية (اختياري)",
                    options: [
                        featureOption('custom_designs', 'Custom designs', 'تصاميم خاصة حسب الطلب'),
                        featureOption('free_delivery', 'Free delivery', 'توصيل مجاني'),
                        featureOption('out_of_town_shipping', 'Out-of-town shipping', 'شحن خارج المدينة'),
                        featureOption('perfumes', 'Perfumes', 'عطور'),
                        featureOption('scented_candles', 'Scented candles', 'شموع معطرة'),
                        featureOption('chocolate', 'Chocolate', 'شوكولاتة')
                    ]
                }
            ]
        };
    }

    // 2. Bridal Jewelry (wedding-jewelry)
    if (categoryId === 'wedding-jewelry') {
        return {
            categoryId,
            usesCulturalSettings: false,
            stepOverrides: defaultStepOverrides(false),
            detailSections: [
                {
                    titleEn: "Main Details",
                    titleAr: "التفاصيل الأساسية",
                    fields: [
                        field('custom_designs', 'Custom designs on-demand', 'تصاميم خاصة حسب الطلب', 'boolean'),
                        field('name_engraving', 'Name engraving inside rings', 'حفر الأسماء داخل الخواتم', 'boolean'),
                        field('handmade', 'Handmade / Handcrafted', 'صناعة يدوية / حرفية', 'boolean'),
                        field('warranty', 'Warranty certificate', 'شهادة ضمان', 'boolean')
                    ]
                }
            ],
            featureGroups: [
                {
                    groupId: "additional_features",
                    titleEn: "Additional Features (Optional)",
                    titleAr: "الميزات الإضافية (اختياري)",
                    options: [
                        featureOption('necklaces', 'Necklaces', 'قلائد'),
                        featureOption('bracelets', 'Bracelets', 'أساور'),
                        featureOption('earrings', 'Earrings', 'أقراط'),
                        featureOption('rings', 'Rings', 'خواتم'),
                        featureOption('diamond_rings', 'Diamond rings', 'خواتم ألماس'),
                        featureOption('solitaire_rings', 'Solitaire rings', 'خواتم سوليتير'),
                        featureOption('wedding_bands', 'Wedding bands', 'دبل زفاف'),
                        featureOption('engagement_rings', 'Engagement rings', 'خواتم خطوبة'),
                        featureOption('full_gold_sets', 'Full gold sets', 'أطقم ذهب كاملة'),
                        featureOption('watches', 'Watches', 'ساعات')
                    ]
                }
            ]
        };
    }

    // 3. Kosha & Decor + Wedding Planner (khosha-decor, wedding-planner)
    if (categoryId === 'khosha-decor' || categoryId === 'wedding-planner') {
        return {
            categoryId,
            usesCulturalSettings: true,
            stepOverrides: defaultStepOverrides(true),
            detailSections: [
                {
                    titleEn: "Main Details",
                    titleAr: "التفاصيل الأساسية",
                    fields: [
                        field('plan_theme', 'Plan and design a theme for the event', 'تخطيط وتصميم ثيم للحفل', 'boolean'),
                        field('out_of_town', 'Out-of-town service', 'خدمة خارج المدينة', 'boolean')
                    ]
                },
                {
                    titleEn: "General Info",
                    titleAr: "معلومات عامة",
                    fields: [
                        field('events_hosted', 'What events can be hosted?', 'ما هي المناسبات التي يمكن استضافتها؟', 'text'),
                        field('preparation_time', 'How long does preparation take before the event?', 'كم تستغرق التحضيرات قبل الحدث؟', 'text')
                    ]
                }
            ],
            featureGroups: [
                {
                    groupId: "additional_features",
                    titleEn: "Additional Features (Optional)",
                    titleAr: "الميزات الإضافية (اختياري)",
                    options: [
                        featureOption('security', 'Security team', 'طاقم أمني'),
                        featureOption('decor_styling', 'Decor design and styling', 'تصميم وتنسيق الديكور'),
                        featureOption('invitation_cards', 'Invitation cards', 'بطاقات الدعوة'),
                        featureOption('wedding_cake', 'Wedding cake', 'كيك الزفاف'),
                        featureOption('kosha_design', 'Kosha design and coordination', 'تصميم وتنسيق الكوشة'),
                        featureOption('service_staff', 'Service and supervision staff', 'طاقم خدمة وإشراف'),
                        featureOption('dj_sound', 'DJ with sound system', 'دي جي مع نظام صوت'),
                        featureOption('videography', 'Videography with editing', 'تصوير فيديو ومونتاج'),
                        featureOption('hospitality', 'Hospitality & giveaways', 'ضيافة وتوزيعات'),
                        featureOption('flower_arrangements', 'Flower arrangements', 'تنسيق زهور'),
                        featureOption('table_decorations', 'Table decorations', 'تزيين الطاولات'),
                        featureOption('tables_chairs', 'Tables and chairs', 'طاولات وكراسي'),
                        featureOption('lighting', 'Lighting engineering', 'هندسة إضاءة'),
                        featureOption('photography', 'Photography', 'تصوير فوتوغرافي'),
                        featureOption('dinner', 'Dinner', 'عشاء'),
                        featureOption('hosting_artists', 'Hosting artists / performers', 'استضافة فنانين')
                    ]
                }
            ]
        };
    }

    // 4. Flowers & Floral (flowers-floral)
    if (categoryId === 'flowers-floral') {
        return {
            categoryId,
            usesCulturalSettings: false,
            stepOverrides: defaultStepOverrides(false),
            detailSections: [
                {
                    titleEn: "Main Details",
                    titleAr: "التفاصيل الأساسية",
                    fields: [
                        field('custom_designs', 'Custom designs', 'تصاميم خاصة حسب الطلب', 'boolean'),
                        field('out_of_town', 'Out-of-town service', 'خدمة خارج المدينة', 'boolean')
                    ]
                },
                {
                    titleEn: "General Info",
                    titleAr: "معلومات عامة",
                    fields: [
                        field('delivery_time', 'Delivery time / Delivery period', 'وقت / فترة التوصيل', 'text')
                    ]
                }
            ],
            featureGroups: [
                {
                    groupId: "additional_features",
                    titleEn: "Additional Features (Optional)",
                    titleAr: "الميزات الإضافية (اختياري)",
                    options: [
                        featureOption('artificial_flowers', 'Artificial flowers', 'زهور صناعية'),
                        featureOption('natural_flowers', 'Natural / Fresh flowers', 'زهور طبيعية'),
                        featureOption('kosha_decor', 'Kosha decoration', 'تنسيق كوشة'),
                        featureOption('bridal_bouquet', 'Bridal bouquet', 'مسكة العروس'),
                        featureOption('table_decor', 'Table decoration', 'تزيين طاولات'),
                        featureOption('entrance_decor', 'Entrance decoration', 'تنسيق المداخل'),
                        featureOption('car_decor', 'Car decoration', 'تزيين سيارات')
                    ]
                }
            ]
        };
    }

    // 5. Entertainment (Merged: entertainment-dj, zaffa, nasheed-band)
    if (['entertainment-dj', 'zaffa', 'nasheed-band'].includes(categoryId)) {
        return {
            categoryId,
            usesCulturalSettings: true,
            stepOverrides: defaultStepOverrides(true),
            detailSections: [
                {
                    titleEn: "General Info",
                    titleAr: "معلومات عامة",
                    fields: [
                        field('events_hosted', 'What events can be hosted?', 'ما هي المناسبات التي يمكن استضافتها؟', 'text'),
                        field('preparation_time', 'How long does preparation take before the performance?', 'كم يستغرق التحضير قبل الأداء؟', 'text')
                    ]
                }
            ],
            featureGroups: [
                {
                    groupId: "additional_features",
                    titleEn: "Additional Features (Optional)",
                    titleAr: "الميزات الإضافية (اختياري)",
                    options: [
                        featureOption('zaffa_show', 'Zaffa show', 'عروض زفة'),
                        featureOption('full_band', 'Full band', 'فرقة كاملة'),
                        featureOption('half_band', 'Half band', 'نصف فرقة'),
                        featureOption('instruments', 'Musical instruments', 'آلات موسيقية'),
                        featureOption('dj', 'DJ', 'دي جي'),
                        featureOption('speakers', 'Speakers & sound systems', 'سماعات وأنظمة صوت'),
                        featureOption('zaffa', 'Zaffa', 'زفة'),
                        featureOption('singer', 'Singer', 'مطرب'),
                        featureOption('zaffa_lyrics', 'Zaffa with pre-written lyrics', 'زفات بكلمات مكتوبة مسبقاً'),
                        featureOption('out_of_town', 'Out-of-town services', 'خدمات خارج المدينة'),
                        featureOption('full_zaffas', 'Complete and comprehensive zaffas', 'زفات كاملة وشاملة'),
                        featureOption('no_music_zaffa', 'Music-free zaffas (Vocal only)', 'زفات بدون موسيقى'),
                        featureOption('music_zaffa', 'Zaffas with music', 'زفات بموسيقى')
                    ]
                }
            ]
        };
    }

    // 6. Food & Catering (catering, wedding-cake, wedding-sweets)
    if (['catering', 'wedding-cake', 'wedding-sweets'].includes(categoryId)) {
        return {
            categoryId,
            usesCulturalSettings: false,
            stepOverrides: defaultStepOverrides(false),
            detailSections: [
                {
                    titleEn: "Main Details",
                    titleAr: "التفاصيل الأساسية",
                    fields: [
                        field('min_guests', 'Minimum guests', 'الحد الأدنى للضيوف', 'number'),
                        field('max_guests', 'Maximum guests', 'الحد الأقصى للضيوف', 'number'),
                        field('serving_style', 'Buffet / Plated / Both', 'بوفيه / صحون / كلاهما', 'dropdown', [
                            { value: 'buffet', labelEn: 'Buffet', labelAr: 'بوفيه' },
                            { value: 'plated', labelEn: 'Plated', labelAr: 'صحون' },
                            { value: 'both', labelEn: 'Both', labelAr: 'كلاهما' }
                        ]),
                        field('hotel_certified', 'Hotel certified', 'معتمد لدى الفنادق', 'boolean'),
                        field('serving_staff', 'Serving staff included', 'يشمل طاقم الخدمة', 'boolean'),
                        field('taste_testing', 'Taste testing', 'تذوق مجاني', 'boolean'),
                        field('setup_cleanup', 'Setup & cleanup', 'التجهيز والتنظيف', 'boolean')
                    ]
                },
                {
                    titleEn: "General Info",
                    titleAr: "معلومات عامة",
                    fields: [
                        field('supervisor_name', 'Supervisor name', 'اسم المشرف', 'text'),
                        field('cuisine_types', 'Types of cuisines provided', 'أنواع المطابخ المقدمة', 'text'),
                        field('booking_deadline', 'Booking deadline', 'الموعد النهائي للحجز', 'text'),
                        field('min_booking_time', 'Minimum booking time', 'أقل مدة للحجز', 'text'),
                        field('prep_time', 'Preparation time before the event', 'وقت التحضير قبل الحدث', 'text'),
                        field('price_per_person', 'Price per person', 'السعر للشخص الواحد', 'number'),
                        field('price_per_meter', 'Buffet price per meter', 'سعر البوفيه للمتر', 'number'),
                        field('price_per_plate', 'Price per plate', 'السعر للصحن', 'number')
                    ]
                }
            ],
            featureGroups: [
                {
                    groupId: "additional_features",
                    titleEn: "Additional Features (Optional)",
                    titleAr: "الميزات الإضافية (اختياري)",
                    options: [
                        featureOption('table_setting', 'Table setting / arrangement', 'تنسيق وترتيب الطاولات'),
                        featureOption('on_demand_menu', 'On-demand menu', 'قائمة طعام حسب الطلب'),
                        featureOption('delivery_service', 'Delivery service', 'خدمة توصيل'),
                        featureOption('coffee_servers', 'Coffee servers', 'صبابات قهوة'),
                        featureOption('wedding_cake', 'Wedding cake', 'كيكة زفاف'),
                        featureOption('open_buffet', 'Open buffet', 'بوفيه مفتوح'),
                        featureOption('table_platters', 'Table platters', 'أطباق طاولات'),
                        featureOption('all_beverages', 'All beverage types', 'جميع أنواع المشروبات'),
                        featureOption('desserts', 'Desserts', 'حلويات'),
                        featureOption('sacrifices', 'Sacrifices (carcasses) and compartments', 'ذبائح وأقسام'),
                        featureOption('table_hospitality', 'Table hospitality / service', 'ضيافة طاولات'),
                        featureOption('appetizers', 'Appetizers', 'مقبلات'),
                        featureOption('serving_desserts', 'Serving desserts', 'تقديم حلويات'),
                        featureOption('serving_coffee', 'Serving coffee', 'تقديم قهوة'),
                        featureOption('serving_beverages', 'Serving beverages', 'تقديم مشروبات')
                    ]
                }
            ]
        };
    }

    // 7. Photography (photography-video, photo-studio)
    if (['photography-video', 'photo-studio'].includes(categoryId)) {
        return {
            categoryId,
            usesCulturalSettings: true,
            stepOverrides: defaultStepOverrides(true),
            detailSections: [
                {
                    titleEn: "General Info",
                    titleAr: "معلومات عامة",
                    fields: [
                        field('photography_types', 'Types of photography', 'أنواع التصوير', 'text'),
                        field('event_types', 'Types of events filmed', 'أنواع المناسبات المصورة', 'text')
                    ]
                }
            ],
            featureGroups: [
                {
                    groupId: "additional_features",
                    titleEn: "Additional Features (Optional)",
                    titleAr: "الميزات الإضافية (اختياري)",
                    options: [
                        featureOption('crane', 'Crane photography / videography', 'تصوير كرين'),
                        featureOption('drone', 'Drone photography / videography', 'تصوير درون'),
                        featureOption('live_streaming', 'Live streaming', 'بث مباشر'),
                        featureOption('outdoor_shooting', 'Outdoor shooting', 'تصوير خارجي'),
                        featureOption('studio', 'Studio photography', 'تصوير استوديو'),
                        featureOption('out_of_town', 'Out-of-town shooting', 'تصوير خارج المدينة'),
                        featureOption('videography', 'Videography', 'تصوير فيديو'),
                        featureOption('photography_crew', 'Photography crew', 'طاقم تصوير'),
                        featureOption('digital_delivery', 'Digital delivery of original copies', 'تسليم نسخ رقمية أصلية')
                    ]
                }
            ]
        };
    }

    // 8. Henna Art (henna-art)
    if (categoryId === 'henna-art') {
        return {
            categoryId,
            usesCulturalSettings: false,
            stepOverrides: defaultStepOverrides(false),
            detailSections: [],
            featureGroups: [
                {
                    groupId: "additional_features",
                    titleEn: "Additional Features (Optional)",
                    titleAr: "الميزات الإضافية (اختياري)",
                    options: [
                        featureOption('soft_henna', 'Soft henna design', 'نقش حناء ناعم'),
                        featureOption('bold_henna', 'Bold henna design', 'نقش حناء عريض'),
                        featureOption('elegant_evening', 'Elegant evening designs', 'نقوش سهرات أنيقة'),
                        featureOption('daytime_occasion', 'Daytime occasion designs', 'نقوش مناسبات نهارية'),
                        featureOption('emirati', 'Emirati design', 'نقش إماراتي'),
                        featureOption('indian', 'Indian design', 'نقش هندي'),
                        featureOption('sudanese', 'Sudanese design', 'نقش سوداني'),
                        featureOption('red_henna', 'Red henna', 'حناء حمراء'),
                        featureOption('white_henna', 'White henna', 'حناء بيضاء'),
                        featureOption('gulf', 'Gulf (Khaleeji) design', 'نقش خليجي'),
                        featureOption('yemeni', 'Yemeni design', 'نقش يمني'),
                        featureOption('saudi', 'Saudi design', 'نقش سعودي'),
                        featureOption('custom_ondemand', 'Custom designs on-demand', 'نقوش خاصة حسب الطلب'),
                        featureOption('henna_staff', 'Henna artist staff', 'طاقم حناء')
                    ]
                }
            ]
        };
    }

    // 9. Hair & Makeup + Beauty & Skincare (hair-makeup, beauty-skincare)
    if (['hair-makeup', 'beauty-skincare'].includes(categoryId)) {
        return {
            categoryId,
            usesCulturalSettings: false,
            stepOverrides: defaultStepOverrides(false),
            detailSections: [
                {
                    titleEn: "General Info",
                    titleAr: "معلومات عامة",
                    fields: [
                        field('companions_count', 'Number of bride\'s companions', 'عدد مرافقات العروس', 'number')
                    ]
                }
            ],
            featureGroups: [
                {
                    groupId: "additional_features",
                    titleEn: "Additional Features (Optional)",
                    titleAr: "الميزات الإضافية (اختياري)",
                    options: [
                        featureOption('hands_feet', 'Hands & Feet care', 'عناية باليدين والقدمين'),
                        featureOption('facial_skin', 'Facial & Skin care', 'عناية بالبشرة والوجه'),
                        featureOption('hair_care', 'Hair care', 'عناية بالشعر'),
                        featureOption('eyelash_ext', 'Eyelash extensions', 'تركيب رموش'),
                        featureOption('moroccan_bath', 'Moroccan bath', 'حمام مغربي'),
                        featureOption('hair_removal', 'Hair removal', 'إزالة الشعر'),
                        featureOption('massage', 'Massage', 'مساج'),
                        featureOption('body_mask', 'Body mask', 'ماسك للجسم'),
                        featureOption('nail_ext', 'Nail extensions', 'تركيب أظافر'),
                        featureOption('pedicure', 'Pedicure', 'بديكير'),
                        featureOption('manicure', 'Manicure', 'منيكير'),
                        featureOption('eyebrow_care', 'Eyebrow care', 'تشقير وترتيب حواجب'),
                        featureOption('hall_service', 'Hall service', 'خدمة في القاعة'),
                        featureOption('home_service', 'Home service', 'خدمة منزلية'),
                        featureOption('out_of_town', 'Out-of-town service', 'خدمة خارج المدينة'),
                        featureOption('makeup', 'Makeup', 'مكياج'),
                        featureOption('free_rehearsal', 'Free rehearsal', 'بروفة مجانية'),
                        featureOption('accommodate_companions', 'Accommodate the bride\'s companions', 'استقبال مرافقات العروس')
                    ]
                }
            ]
        };
    }

    // 10. Fashion Categories (wedding-gown, haute-couture, abaya-jalabiya, groom-attire)
    if (['wedding-gown', 'haute-couture', 'abaya-jalabiya', 'groom-attire'].includes(categoryId)) {
        return {
            categoryId,
            usesCulturalSettings: false,
            stepOverrides: defaultStepOverrides(false),
            detailSections: [
                {
                    titleEn: "General Info",
                    titleAr: "معلومات عامة",
                    fields: [
                        field('time_to_detail', 'Time to detail', 'مدة التفصيل', 'text'),
                        field('rehearsals_count', 'Number of rehearsals', 'عدد البروفات', 'number')
                    ]
                }
            ],
            featureGroups: [
                {
                    groupId: "additional_features",
                    titleEn: "Additional Features (Optional)",
                    titleAr: "الميزات الإضافية (اختياري)",
                    options: [
                        featureOption('ready_to_wear', 'Ready-to-wear styles', 'أزياء جاهزة'),
                        featureOption('dress_detailing', 'Dress detailing', 'تفصيل فساتين'),
                        featureOption('notifications', 'Notifications', 'إشعارات'),
                        featureOption('evening_dresses', 'Evening dresses', 'فساتين سهرة'),
                        featureOption('engagement_dresses', 'Engagement dresses', 'فساتين خطوبة'),
                        featureOption('veil', 'Veil', 'طرحة'),
                        featureOption('crown', 'Crown', 'تاج'),
                        featureOption('hair_piece', 'Hair piece', 'إكسسوار شعر')
                    ]
                }
            ]
        };
    }

    // 11. Wedding Palace
    if (categoryId === 'wedding-palace') {
        return {
            categoryId,
            usesCulturalSettings: true,
            stepOverrides: defaultStepOverrides(true),
            detailSections: [
                {
                    titleEn: "Section Capacity",
                    titleAr: "الطاقة الاستيعابية للأقسام",
                    fields: [
                        field('men_capacity', 'Men Section Capacity', 'سعة قسم الرجال', 'number'),
                        field('women_capacity', 'Women Section Capacity', 'سعة قسم النساء', 'number')
                    ]
                },
                {
                    titleEn: "Venue Facilities",
                    titleAr: "تجهيزات ومرافق المكان",
                    fields: [
                        field('has_separate_entrances', 'Separate Entrances for Men & Women', 'مداخل منفصلة للرجال والنساء', 'boolean'),
                        field('has_audio_link', 'Audio Link between Sections', 'ربط صوتي بين الأقسام', 'boolean'),
                        field('max_events_per_day', 'Max Events per Day', 'الحد الأقصى للمناسبات في اليوم', 'number')
                    ]
                }
            ],
            featureGroups: [
                {
                    groupId: "additional_features",
                    titleEn: "Additional Features (Optional)",
                    titleAr: "الميزات الإضافية (اختياري)",
                    options: [
                        featureOption('a347de4c-c4ea-4339-b1fb-878664394f65', 'Abayas official', 'عبايات رسمية'),
                        featureOption('40d9639c-7b7e-4b13-b8ff-2f9d6a28b0e4', 'Coffee server for men', 'قهوجي للرجال'),
                        featureOption('b27534f3-5946-4fc3-a774-8c58aa42d8c4', 'DJ / Audio Equipment', 'دي جي ومعدات صوت'),
                        featureOption('42961fa7-30f8-4f05-a062-6d08e12886cc', 'Free suite for newlyweds', 'جناح مجاني للعروسين'),
                        featureOption('83a454a1-320e-439c-9830-03675cfafed1', 'Hot and cold drinks', 'مشروبات ساخنة وباردة'),
                        featureOption('9a593cdc-58d4-4e68-b754-4693810ea1bd', 'Lighting', 'إضاءة'),
                        featureOption('77149a8c-4673-4852-8591-a8ead8168d0c', 'Mobile phone inspector', 'مفتش الجوالات'),
                        featureOption('eeb24dcf-4a80-4498-af68-e20f540ecdc2', 'Outdoor space for events', 'مساحة خارجية'),
                        featureOption('616c371d-f856-4f07-8d25-48ddaa2a3d91', 'Possibility of holding several parties at the same time', 'إمكانية إقامة أكثر من حفل'),
                        featureOption('9089c534-5c84-4e32-8d9c-25d1b04cd815', 'Separate dining hall', 'قاعة طعام منفصلة'),
                        featureOption('792426c6-8430-4f8c-b32f-39e50f7fe09b', 'Staircase for the wedding procession', 'درج للزفة'),
                        featureOption('fbf4d86e-8186-4d0c-beae-b24be29b2330', 'The stage and hall decoration', 'المسرح وتزيين القاعة'),
                        featureOption('69f25cc9-66d8-4125-9dec-a040693bfcec', 'Wedding cake', 'كيكة زفاف'),
                        featureOption('548a6f8e-2c6a-48c0-9943-67d6b02de1d7', 'Car parking', 'موقف سيارات'),
                        featureOption('94bca934-f9f9-409a-a309-0e11b0db2b39', 'Cooking carcasses', 'طبخ أكارع'),
                        featureOption('8ddb6ad1-52d0-48aa-b8cc-13f580137f72', 'Female workers', 'موظفات'),
                        featureOption('ef68c13a-157c-44b3-940c-e43ba8186ff1', 'Hall supervisor', 'مشرف القاعة'),
                        featureOption('d00e50b0-2fce-4d5f-8b84-8016be3e2b87', 'Laser', 'ليزر'),
                        featureOption('a51a8631-f045-4df9-b858-8fac6acfe945', 'Meeting room', 'قاعة اجتماعات'),
                        featureOption('770b4986-baac-4aa7-8773-54e49987a962', 'Open buffet', 'بوفيه مفتوح'),
                        featureOption('f27facb5-a553-4119-98ae-9aafca26d539', 'Photography and video', 'تصوير وفيديو'),
                        featureOption('093842eb-5e2e-4fdb-aeeb-0856811d0617', 'Preparation room for the bride', 'غرفة تجهيز العروس'),
                        featureOption('2533f621-e043-44e1-87ab-e62e337d1c96', 'Separate entrance for the bride', 'مدخل منفصل للعروس'),
                        featureOption('86421e96-3f25-4a38-9733-a22de450a643', 'Steam', 'بخار'),
                        featureOption('9d7d7467-001c-417b-ae2f-232d3ef6bcd3', 'Valves and fittings', 'صمامات وتجهيزات')
                    ]
                }
            ]
        };
    }

    // 12. Hotel Ballroom
    if (categoryId === 'hotel-venue') {
        return {
            categoryId,
            usesCulturalSettings: true,
            stepOverrides: defaultStepOverrides(true),
            detailSections: [
                {
                    titleEn: "Section Capacity",
                    titleAr: "الطاقة الاستيعابية للأقسام",
                    fields: [
                        field('men_capacity', 'Men Section Capacity', 'سعة قسم الرجال', 'number'),
                        field('women_capacity', 'Women Section Capacity', 'سعة قسم النساء', 'number')
                    ]
                },
                {
                    titleEn: "Venue Facilities",
                    titleAr: "تجهيزات ومرافق المكان",
                    fields: [
                        field('has_separate_entrances', 'Separate Entrances for Men & Women', 'مداخل منفصلة للرجال والنساء', 'boolean'),
                        field('has_audio_link', 'Audio Link between Sections', 'ربط صوتي بين الأقسام', 'boolean'),
                        field('max_events_per_day', 'Max Events per Day', 'الحد الأقصى للمناسبات في اليوم', 'number')
                    ]
                },
                {
                    titleEn: "Hotel Services",
                    titleAr: "خدمات الفندق",
                    fields: [
                        field('in_house_catering', 'In-house Catering Available', 'بوفيه داخلي متوفر', 'boolean')
                    ]
                }
            ],
            featureGroups: [
                {
                    groupId: "additional_features",
                    titleEn: "Additional Features (Optional)",
                    titleAr: "الميزات الإضافية (اختياري)",
                    options: [
                        featureOption('a347de4c-c4ea-4339-b1fb-878664394f65', 'Abayas official', 'عبايات رسمية'),
                        featureOption('40d9639c-7b7e-4b13-b8ff-2f9d6a28b0e4', 'Coffee server for men', 'قهوجي للرجال'),
                        featureOption('b27534f3-5946-4fc3-a774-8c58aa42d8c4', 'DJ / Audio Equipment', 'دي جي ومعدات صوت'),
                        featureOption('42961fa7-30f8-4f05-a062-6d08e12886cc', 'Free suite for newlyweds', 'جناح مجاني للعروسين'),
                        featureOption('83a454a1-320e-439c-9830-03675cfafed1', 'Hot and cold drinks', 'مشروبات ساخنة وباردة'),
                        featureOption('9a593cdc-58d4-4e68-b754-4693810ea1bd', 'Lighting', 'إضاءة'),
                        featureOption('77149a8c-4673-4852-8591-a8ead8168d0c', 'Mobile phone inspector', 'مفتش الجوالات'),
                        featureOption('eeb24dcf-4a80-4498-af68-e20f540ecdc2', 'Outdoor space for events', 'مساحة خارجية'),
                        featureOption('616c371d-f856-4f07-8d25-48ddaa2a3d91', 'Possibility of holding several parties at the same time', 'إمكانية إقامة أكثر من حفل'),
                        featureOption('9089c534-5c84-4e32-8d9c-25d1b04cd815', 'Separate dining hall', 'قاعة طعام منفصلة'),
                        featureOption('792426c6-8430-4f8c-b32f-39e50f7fe09b', 'Staircase for the wedding procession', 'درج للزفة'),
                        featureOption('fbf4d86e-8186-4d0c-beae-b24be29b2330', 'The stage and hall decoration', 'المسرح وتزيين القاعة'),
                        featureOption('69f25cc9-66d8-4125-9dec-a040693bfcec', 'Wedding cake', 'كيكة زفاف'),
                        featureOption('548a6f8e-2c6a-48c0-9943-67d6b02de1d7', 'Car parking', 'موقف سيارات'),
                        featureOption('94bca934-f9f9-409a-a309-0e11b0db2b39', 'Cooking carcasses', 'طبخ أكارع'),
                        featureOption('8ddb6ad1-52d0-48aa-b8cc-13f580137f72', 'Female workers', 'موظفات'),
                        featureOption('ef68c13a-157c-44b3-940c-e43ba8186ff1', 'Hall supervisor', 'مشرف القاعة'),
                        featureOption('d00e50b0-2fce-4d5f-8b84-8016be3e2b87', 'Laser', 'ليزر'),
                        featureOption('a51a8631-f045-4df9-b858-8fac6acfe945', 'Meeting room', 'قاعة اجتماعات'),
                        featureOption('770b4986-baac-4aa7-8773-54e49987a962', 'Open buffet', 'بوفيه مفتوح'),
                        featureOption('f27facb5-a553-4119-98ae-9aafca26d539', 'Photography and video', 'تصوير وفيديو'),
                        featureOption('093842eb-5e2e-4fdb-aeeb-0856811d0617', 'Preparation room for the bride', 'غرفة تجهيز العروس'),
                        featureOption('2533f621-e043-44e1-87ab-e62e337d1c96', 'Separate entrance for the bride', 'مدخل منفصل للعروس'),
                        featureOption('86421e96-3f25-4a38-9733-a22de450a643', 'Steam', 'بخار'),
                        featureOption('9d7d7467-001c-417b-ae2f-232d3ef6bcd3', 'Valves and fittings', 'صمامات وتجهيزات')
                    ]
                }
            ]
        };
    }

    // Other Venue Categories (restaurant-event, outdoor-garden, rooftop-venue, private-beach, villa-resort, chalet)
    const venueCategories = ['restaurant-event', 'outdoor-garden', 'rooftop-venue', 'private-beach', 'villa-resort', 'chalet'];
    if (venueCategories.includes(categoryId)) {
        return {
            categoryId,
            usesCulturalSettings: true,
            stepOverrides: defaultStepOverrides(true),
            detailSections: [
                {
                    titleEn: "General Info",
                    titleAr: "معلومات عامة",
                    fields: [
                        field('capacity', 'Capacity', 'السعة', 'number'),
                        field('events_hosted', 'What events can be hosted?', 'المناسبات التي يمكن استضافتها؟', 'text')
                    ]
                }
            ],
            featureGroups: [
                {
                    groupId: "additional_features",
                    titleEn: "Additional Features (Optional)",
                    titleAr: "الميزات الإضافية (اختياري)",
                    options: [
                        featureOption('mens_majlis', 'Men\'s majlis', 'مجلس رجال'),
                        featureOption('indoor_hall', 'Indoor hall', 'صالة داخلية'),
                        featureOption('outdoor_pool', 'Outdoor area with pool', 'منطقة خارجية مع مسبح'),
                        featureOption('outdoor_area', 'Outdoor area', 'مساحة خارجية'),
                        featureOption('chairs_tables', 'Chairs & tables', 'كراسي وطاولات'),
                        featureOption('event_decor', 'Event décor', 'ديكور مناسبات'),
                        featureOption('parking_garage', 'Parking garage', 'مواقف سيارات'),
                        featureOption('open_buffet', 'Open buffet', 'بوفيه مفتوح'),
                        featureOption('terrace', 'Terrace', 'تراس'),
                        featureOption('table_platters', 'Table platters', 'أطباق طاولات'),
                        featureOption('sacrifices', 'Sacrifices', 'ذبائح'),
                        featureOption('staff_male_female', 'Male & female staff', 'طاقم رجال ونساء'),
                        featureOption('dj_sound_systems', 'DJ & sound systems', 'دي جي وأنظمة صوت')
                    ]
                }
            ]
        };
    }

    // Default fallback (e.g. wedding-car or others not specified)
    return {
        categoryId,
        usesCulturalSettings: true,
        stepOverrides: defaultStepOverrides(true),
        detailSections: [],
        featureGroups: []
    };
}
