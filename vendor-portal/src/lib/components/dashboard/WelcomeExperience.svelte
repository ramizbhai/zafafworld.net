<script lang="ts">
    import { Plus, CheckCircle2, Clock, MessageCircle } from 'lucide-svelte';
    
    let { i18n, userFirstName } = $props<{
        i18n: any;
        userFirstName: string;
    }>();
</script>

<div class="welcome-shell">
    <div class="welcome-glow-1" aria-hidden="true"></div>
    <div class="welcome-glow-2" aria-hidden="true"></div>

    <div class="welcome-brand">
        <div class="welcome-logo-ring">
            <svg width="38" height="38" viewBox="0 0 38 38" fill="none" xmlns="http://www.w3.org/2000/svg" aria-label="ZafafWorld" role="img">
                <rect width="38" height="38" rx="12" fill="url(#zw-grad)"/>
                <text x="19" y="26" text-anchor="middle" font-family="Outfit,Inter,sans-serif" font-size="18" font-weight="800" fill="#fff">★</text>
                <defs>
                    <linearGradient id="zw-grad" x1="0" y1="0" x2="38" y2="38" gradientUnits="userSpaceOnUse">
                        <stop offset="0%" stop-color="hsl(162,72%,36%)"/>
                        <stop offset="100%" stop-color="hsl(162,72%,22%)"/>
                    </linearGradient>
                </defs>
            </svg>
        </div>
        <div>
            <span class="welcome-brand-name">ZafafWorld</span>
            <span class="welcome-brand-badge">{i18n.locale === 'ar' ? 'بوابة الموردين' : 'Vendor Portal'}</span>
        </div>
    </div>

    <div class="welcome-hero">
        <h1 class="welcome-headline">
            {i18n.locale === 'ar'
                ? `أهلاً بك، ${userFirstName} 🎉`
                : `Welcome, ${userFirstName}! 🎉`
            }
        </h1>
        <p class="welcome-subhead">
            {i18n.locale === 'ar'
                ? 'حسابك جاهز. خطوتك التالية هي إنشاء أول إعلان تجاري حتى تظهر خدماتك لآلاف العرسان عبر منصة زفاف وورلد.'
                : 'Your account is live. Your next step is to create your first business listing so couples across the ZafafWorld marketplace can discover your services.'}
        </p>
    </div>

    <div class="welcome-steps">
        <div class="welcome-step done-step">
            <div class="step-num done">1</div>
            <div class="step-body">
                <strong>{i18n.locale === 'ar' ? 'إنشاء الحساب' : 'Account Created'}</strong>
                <span class="step-sub">{i18n.locale === 'ar' ? 'مكتمل ✔' : 'Complete ✔'}</span>
            </div>
        </div>
        <div class="step-connector" aria-hidden="true"></div>
        <div class="welcome-step current-step">
            <div class="step-num current">2</div>
            <div class="step-body">
                <strong>{i18n.locale === 'ar' ? 'إضافة إعلانك الأول' : 'Add Your First Listing'}</strong>
                <span class="step-sub current-label">{i18n.locale === 'ar' ? 'أنت هنا' : 'You are here'}</span>
            </div>
        </div>
        <div class="step-connector" aria-hidden="true"></div>
        <div class="welcome-step">
            <div class="step-num">3</div>
            <div class="step-body">
                <strong>{i18n.locale === 'ar' ? 'موافقة الفريق والنشر' : 'Listing Approved & Live'}</strong>
                <span class="step-sub">{i18n.locale === 'ar' ? 'سيظهر على المنصة' : 'Goes live on marketplace'}</span>
            </div>
        </div>
    </div>

    <div class="welcome-cta-group">
        <a href="/dashboard/products/new" class="cta-primary" id="create-first-listing-btn">
            <Plus size={18} />
            {i18n.locale === 'ar' ? 'إنشاء أول إعلان تجاري' : 'Create My First Listing'}
        </a>
        <a href="/dashboard/products" class="cta-secondary">
            {i18n.locale === 'ar' ? 'استعراض قسم الإعلانات' : 'Browse Listings Section'}
        </a>
    </div>

    <div class="welcome-chips">
        <span class="info-chip">
            <CheckCircle2 size={13} />
            {i18n.locale === 'ar' ? 'تستطيع إدارة أنواع متعددة من الإعلانات' : 'Manage multiple listing categories'}
        </span>
        <span class="info-chip">
            <Clock size={13} />
            {i18n.locale === 'ar' ? 'مراجعة سريعة خلال 24 ساعة' : 'Fast review within 24 hours'}
        </span>
        <span class="info-chip">
            <MessageCircle size={13} />
            {i18n.locale === 'ar' ? 'دعم متوفر دائماً' : 'Support always available'}
        </span>
    </div>
</div>
