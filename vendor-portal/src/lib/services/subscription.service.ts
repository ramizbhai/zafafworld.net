export function getTierTheme(name: string) {
    const n = name.toLowerCase();
    if (n.includes('gold')) return 'gold';
    if (n.includes('vip')) return 'vip';
    if (n.includes('diamond')) return 'diamond';
    return 'default';
}

export function getTierPrice(name: string) {
    const n = name.toLowerCase();
    if (n.includes('gold')) return '20,000';
    if (n.includes('vip')) return '35,000';
    if (n.includes('diamond')) return '70,000';
    return '0';
}

export function getTierFeatures(name: string, locale: string) {
    const n = name.toLowerCase();
    const isAr = locale === 'ar';

    if (n.includes('gold')) {
        return isAr ? [
            { text: `يظهر فوق المجاني` },
            { text: `العملاء المحتملين` },
            { text: `صفحة شخصية محسنة لمحركات البحث` },
            { text: `إشعارات عبر البريد الإلكتروني والرسائل القصيرة` },
            { text: `دعم مخصص بعد البيع` },
            { text: `15 صورة` },
            { text: `1 فيديو` },
            { text: `الظهور في صفحة المنافسين` },
            { text: `لن يتم عرض إعلانات في صفحتك` },
            { text: `سيتم عرض العروض/الخصومات في صفحة شركتك، صفحات الخصومات` },
            { text: `الوصول إلى إحصائيات الصفحة` },
            { text: `الوصول إلى تطبيق Zafaf.net` }
        ] : [
            { text: `Appears above free` },
            { text: `Leads` },
            { text: `SEO optimized profile page` },
            { text: `Email & SMS notifications` },
            { text: `Post-Sales customized support` },
            { text: `15 Pictures` },
            { text: `1 Video` },
            { text: `Appear on competitors page` },
            { text: `No Ads will be shown on your page` },
            { text: `Offer/discount will be shown on your company page, discount pages` },
            { text: `Access to page statistics` },
            { text: `Access to Zafaf.net App` }
        ];
    }
    if (n.includes('vip')) {
        return isAr ? [
            { text: `يظهر فوق الذهبي` },
            { text: `العملاء المحتملين` },
            { text: `صفحة شخصية محسنة لمحركات البحث` },
            { text: `إشعارات عبر البريد الإلكتروني والرسائل القصيرة` },
            { text: `دعم مخصص بعد البيع` },
            { text: `30 صورة` },
            { text: `10 فيديوهات` },
            { text: `الظهور في صفحة المنافسين` },
            { text: `لن يتم عرض إعلانات في صفحتك` },
            { text: `سيتم عرض العروض/الخصومات في صفحة شركتك، صفحات الخصومات` },
            { text: `الوصول إلى إحصائيات الصفحة` },
            { text: `الوصول إلى تطبيق Zafaf.net` },
            { text: `راعي المدينة (شهر واحد)` },
            { text: `راعي القسم (شهر واحد)` },
            { text: `إعلانات جوجل مخصصة لصفحة عملك` },
            { text: `4 منشورات على منصاتنا (إنستغرام - فيسبوك - يوتيوب)، عند الطلب` }
        ] : [
            { text: `Appears above Gold` },
            { text: `Leads` },
            { text: `SEO optimized profile page` },
            { text: `Email & SMS notifications` },
            { text: `Post-Sales customized support` },
            { text: `30 Pictures` },
            { text: `10 Videos` },
            { text: `Appear on competitors page` },
            { text: `No Ads will be shown on your page` },
            { text: `Offer/discount will be shown on your Company page, discount pages` },
            { text: `Access to page statistics` },
            { text: `Access to Zafaf.net App` },
            { text: `City sponsor (1 Month)` },
            { text: `Category sponsor (1 Month)` },
            { text: `Customized Google Ads for your business page` },
            { text: `4 Posts on our platforms (Instagram - FB - YouTube), upon request` }
        ];
    }
    if (n.includes('diamond')) {
        return isAr ? [
            { text: `يظهر في القمة فوق الجميع` },
            { text: `العملاء المحتملين` },
            { text: `صفحة شخصية محسنة لمحركات البحث` },
            { text: `إشعارات عبر البريد الإلكتروني والرسائل القصيرة` },
            { text: `دعم مخصص بعد البيع` },
            { text: `صور غير محدودة` },
            { text: `فيديوهات غير محدودة` },
            { text: `الظهور في صفحة المنافسين` },
            { text: `لن يتم عرض إعلانات في صفحتك` },
            { text: `سيتم عرض العروض/الخصومات في صفحة شركتك، صفحات الخصومات والصفحة الرئيسية` },
            { text: `الوصول إلى إحصائيات الصفحة` },
            { text: `الوصول إلى تطبيق Zafaf.net` },
            { text: `راعي المدينة (3 أشهر)` },
            { text: `راعي القسم (3 أشهر)` },
            { text: `إعلانات جوجل مخصصة لصفحة عملك` },
            { text: `6 منشورات على منصاتنا (إنستغرام - فيسبوك - يوتيوب)، عند الطلب` },
            { text: `منشوران على سناب شات، عند الطلب` },
            { text: `قصة مميزة على إنستغرام` },
            { text: `نافذة منبثقة للخصم (شهر واحد)` },
            { text: `عرض في الصفحة الرئيسية (قاعات الزفاف)` }
        ] : [
            { text: `Appears top placement above all` },
            { text: `Leads` },
            { text: `SEO optimized profile page` },
            { text: `Email & SMS notifications` },
            { text: `Post-Sales customized support` },
            { text: `Unlimited pictures` },
            { text: `Unlimited videos` },
            { text: `Appear on competitors page` },
            { text: `No Ads will be shown on your page` },
            { text: `Offer/discount will be shown on your company page, discount pages and & home page` },
            { text: `Access to page statistics` },
            { text: `Access to Zafaf.net App` },
            { text: `City sponsor (3 Month)` },
            { text: `Category sponsor (3 Month)` },
            { text: `Customized Google Ads for your business page` },
            { text: `6 Posts on our platforms (Instagram - FB - YouTube), upon request` },
            { text: `2 Snapchat Posts, upon request` },
            { text: `Highlight Story on Instagram` },
            { text: `Discount popup (1 month)` },
            { text: `Homepage Showcase (Wedding Venues)` }
        ];
    }
    return [];
}

export function isRequested(tierId: string, requests: any[]) {
    return requests.some((r: any) => r.requested_tier_id === tierId && r.status === 'pending');
}

export function getApprovedRequest(requests: any[]) {
    return requests.find((r: any) => r.status === 'approved');
}
