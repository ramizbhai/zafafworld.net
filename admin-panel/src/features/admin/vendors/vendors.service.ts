export function getCategoryLabel(category: string | null | undefined): string {
    if (!category) return 'Uncategorized / غير مصنف';
    const mapping: Record<string, string> = {
        'wedding-palace':    'Wedding Palace / قصر أفراح',
        'hotel-venue':       'Hotel Ballroom / فندق وقاعة',
        'villa-resort':      'Villa & Resort / استراحة وفيلا',
        'restaurant-event':  'Restaurant & Dining / مطعم وقاعة',
        'outdoor-garden':    'Outdoor Garden / حديقة مفتوحة',
        'rooftop-venue':     'Rooftop Venue / سطح خارجي',
        'private-beach':     'Private Beach / شاطئ خاص',
        'chalet':            'Chalet / شاليه',
        'wedding-gown':      'Wedding Gown / فستان زفاف',
        'haute-couture':     'Haute Couture / أزياء راقية',
        'abaya-jalabiya':    'Abaya & Jalabiya / عباءات وجلابيات',
        'groom-attire':      'Groom Attire / ملابس عريس',
        'hair-makeup':       'Hair & Makeup / شعر ومكياج',
        'beauty-skincare':   'Beauty & Skincare / تجميل وعناية',
        'henna-art':         'Henna Art / فن الحناء',
        'male-grooming':     'Male Grooming / حلاقة رجالية',
        'photography-video': 'Photography & Video / تصوير وفيديو',
        'photo-studio':      'Photo Studio / استوديو تصوير',
        'catering':          'Catering / ضيافة وطعام',
        'wedding-cake':      'Wedding Cake / كيك الزفاف',
        'wedding-sweets':    'Arabic Sweets / حلويات عربية',
        'entertainment-dj':  'DJ & Entertainment / دي جي وترفيه',
        'zaffa':             'Zaffa & Procession / زفة وموكب',
        'nasheed-band':      'Nasheed & Live Band / أناشيد وفرقة',
        'wedding-jewelry':   'Bridal Jewelry / مجوهرات',
        'wedding-gifts':     'Wedding Gifts / هدايا وتوزيعات',
        'wedding-planner':   'Wedding Planner / منظم حفلات',
        'khosha-decor':      'Khosha & Decor / كوشة وديكور',
        'flowers-floral':    'Flowers & Floral / ورد وزهور',
        'wedding-invitation':'Wedding Invitations / دعوات زفاف',
        'lighting-av':       'Lighting & AV / إضاءة وصوتيات',
        'wedding-car':       'Wedding Car / سيارة زفاف',
        'photographers-and-videographers': 'Photography / تصوير',
        'wedding-planning': 'Coordinator / تنظيم حفلات',
        'uncategorized': 'Uncategorized / غير مصنف',
    };
    return mapping[category] || category.split('-').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ');
}

export function filterVendors(
    vendors: any[],
    activeTab: string,
    search: string,
    accountStatus: string = 'all',
    subTier: string = 'all',
    subStatus: string = 'all'
) {
    return vendors.filter(v => {
        // Tab filtering
        if (activeTab === 'active' && (v.status !== 'active' || v.subscription_status === 'stopped')) return false;
        if (activeTab === 'stopped' && v.subscription_status !== 'stopped') return false;

        // Account status filter
        if (accountStatus !== 'all' && v.status !== accountStatus) return false;

        // Subscription Tier filter
        if (subTier !== 'all') {
            const tierName = v.current_tier || 'Free';
            if (subTier === 'Gold' || subTier === 'Golden') {
                if (tierName !== 'Gold' && tierName !== 'Golden') return false;
            } else if (tierName.toLowerCase() !== subTier.toLowerCase()) {
                return false;
            }
        }

        // Subscription status filter
        if (subStatus !== 'all' && v.subscription_status !== subStatus) return false;

        // Search text matching
        if (search) {
            const q = search.toLowerCase();
            const nameEn = (v.name_en || '').toLowerCase();
            const nameAr = (v.name_ar || '').toLowerCase();
            const email = (v.email || '').toLowerCase();
            const phone = (v.phone || '').toLowerCase();
            const category = (v.category || '').toLowerCase();
            const cityEn = (v.city_name_en || '').toLowerCase();
            const cityAr = (v.city_name_ar || '').toLowerCase();

            return nameEn.includes(q) ||
                   nameAr.includes(q) ||
                   email.includes(q) ||
                   phone.includes(q) ||
                   category.includes(q) ||
                   cityEn.includes(q) ||
                   cityAr.includes(q);
        }
        return true;
    });
}
