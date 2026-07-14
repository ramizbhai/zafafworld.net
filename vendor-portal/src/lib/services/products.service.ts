export function getCapacityDisplay(product: any): string {
    const attrs = product.attributes || {};
    const m = attrs.men_capacity || attrs.seating_capacity || attrs.max_capacity || attrs.serves_max_persons || null;
    const w = attrs.women_capacity || null;
    if (m && w) return `${m}M + ${w}W`;
    if (m) return `${m}`;
    if (w) return `${w}`;
    return '—';
}

export function formatPrice(price: number | null, locale: string): string {
    if (!price) return '—';
    return new Intl.NumberFormat(locale === 'ar' ? 'ar-SA' : 'en-US', { style: 'currency', currency: 'SAR', maximumFractionDigits: 0 }).format(price);
}

export const statusColors: Record<string, string> = {
    active:           "status-active",
    draft:            "status-draft",
    pending_approval: "status-pending",
    suspended:        "status-suspended",
    archived:         "status-archived",
};

export function getStatusLabels(i18n: any): Record<string, string> {
    return {
        active: i18n.t.listings.statusActive,
        draft: i18n.t.listings.statusDraft,
        pending_approval: i18n.t.listings.statusPendingApproval,
        suspended: i18n.t.listings.statusSuspended,
        archived: i18n.t.listings.statusArchived,
    };
}

export function getGenderLabels(metadata: any) {
    return metadata?.productGenders ?? {
        women_only:    { en: "Women Only",    ar: "نساء فقط",       color: "#ec4899", icon: "♀" },
        men_only:      { en: "Men Only",      ar: "رجال فقط",       color: "#3b82f6", icon: "♂" },
        dual_parallel: { en: "Dual Hall",     ar: "قاعتان منفصلتان", color: "#a855f7", icon: "⚡" },
        mixed:         { en: "Mixed",         ar: "مختلط",           color: "#10b981", icon: "👥" },
        family:        { en: "Family",        ar: "عائلي",           color: "#f59e0b", icon: "🏠" },
    };
}

export function getCategoryLabels(metadata: any) {
    return metadata?.productCategories ?? {
        'wedding-palace':    { en: "Wedding Palace",       ar: "قصور الأفراح" },
        'hotel-venue':       { en: "Hotel Ballroom",       ar: "فنادق وقاعات" },
        'villa-resort':      { en: "Villa & Resort",       ar: "استراحات وفلل" },
        'restaurant-event':  { en: "Restaurant & Dining",  ar: "مطاعم وقاعات" },
        'outdoor-garden':    { en: "Outdoor Garden",       ar: "حدائق مفتوحة" },
        'rooftop-venue':     { en: "Rooftop Venue",        ar: "أسطح خارجية" },
        'private-beach':     { en: "Private Beach",        ar: "شواطئ خاصة" },
        'chalet':            { en: "Chalet",               ar: "شاليهات" },
        'wedding-gown':      { en: "Wedding Gown",         ar: "فساتين زفاف" },
        'haute-couture':     { en: "Haute Couture",        ar: "أزياء راقية" },
        'abaya-jalabiya':    { en: "Abaya & Jalabiya",     ar: "عباءات وجلابيات" },
        'groom-attire':      { en: "Groom Attire",         ar: "ملابس العريس" },
        'hair-makeup':       { en: "Hair & Makeup",        ar: "شعر ومكياج" },
        'beauty-skincare':   { en: "Beauty & Skincare",    ar: "تجميل وعناية" },
        'henna-art':         { en: "Henna Art",            ar: "فن الحناء" },
        'male-grooming':     { en: "Male Grooming",        ar: "حلاقة رجالية" },
        'photography-video': { en: "Photography & Video",  ar: "تصوير وفيديو" },
        'photo-studio':      { en: "Photo Studio",         ar: "استوديو تصوير" },
        'catering':          { en: "Catering",             ar: "ضيافة وطعام" },
        'wedding-cake':      { en: "Wedding Cake",         ar: "كيك الزفاف" },
        'wedding-sweets':    { en: "Arabic Sweets",        ar: "حلويات عربية" },
        'entertainment-dj':  { en: "DJ & Entertainment",   ar: "دي جي وترفيه" },
        'zaffa':             { en: "Zaffa & Procession",   ar: "زفة وموكب" },
        'nasheed-band':      { en: "Nasheed & Live Band",  ar: "أناشيد وفرقة" },
        'wedding-jewelry':   { en: "Bridal Jewelry",       ar: "مجوهرات عروس" },
        'wedding-gifts':     { en: "Wedding Gifts",        ar: "هدايا وتوزيعات" },
        'wedding-planner':   { en: "Wedding Planner",      ar: "منظم حفلات" },
        'khosha-decor':      { en: "Khosha & Decor",       ar: "كوشة وديكور" },
        'flowers-floral':    { en: "Flowers & Floral",     ar: "ورد وزهور" },
        'wedding-invitation':{ en: "Wedding Invitations",  ar: "دعوات زفاف" },
        'lighting-av':       { en: "Lighting & AV",        ar: "إضاءة وصوتيات" },
        'wedding-car':       { en: "Wedding Car",          ar: "سيارة زفاف" },
        'photographers-and-videographers': { en: "Photography & Video", ar: "تصوير وفيديو" },
        'wedding-invitations':             { en: "Wedding Invitations",  ar: "دعوات" },
        'hair-make-up':                    { en: "Hair & Make-up",       ar: "شعر ومكياج" },
        'wedding-planning':                { en: "Khosha & Decor",       ar: "كوشة وديكور" },
        'wedding-gowns':                   { en: "Dresses",              ar: "فساتين" },
        'wedding-cakes':                   { en: "Cakes",                ar: "كيك" },
        'henneh-art':                      { en: "Henna",                ar: "حناء" },
        'band-dj-and-entertainment':       { en: "Entertainment & DJ",   ar: "دي جي وترفيه" },
        'wedding-rings-jewelry':           { en: "Jewelry",              ar: "مجوهرات" },
        'wedding-treats-and-gifts':        { en: "Treats & Distributions", ar: "هدايا وتوزيعات" },
        'wedding-flowers-and-bouquets':    { en: "Wedding Flowers",      ar: "ورد وباقات" },
    };
}
