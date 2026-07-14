<script lang="ts">
    import { upgradeStore, closeUpgradeModal } from '$lib/stores/upgradeStore';
    import { getI18n } from '$lib/i18n/i18n.svelte';
    import { Check, X, AlertCircle, CheckCircle2 } from 'lucide-svelte';
    import { goto } from '$app/navigation';

    const i18n = getI18n();

    export let tiers: any[] = [];
    export let currentTierId: string = '';

    let isSubmitting = false;
    let requestResult = { success: false, error: '' };
    
    let expandedTiers: Record<string, boolean> = {};

    function toggleTier(name: string) {
        expandedTiers[name] = !expandedTiers[name];
        expandedTiers = { ...expandedTiers };
    }

    function handleEnhance() {
        isSubmitting = true;
        requestResult = { success: false, error: '' };
        
        return async ({ result, update }: any) => {
            isSubmitting = false;
            if (result.type === 'success') {
                requestResult = { success: true, error: '' };
                setTimeout(() => {
                    closeUpgradeModal();
                    requestResult = { success: false, error: '' };
                }, 2000);
            } else if (result.type === 'error' || result.type === 'failure') {
                requestResult = { 
                    success: false, 
                    error: result.data?.message || (i18n.locale === 'ar' ? 'حدث خطأ. حاول مرة أخرى.' : 'An error occurred. Please try again.') 
                };
            }
            await update({ reset: false });
        };
    }

    function getTierTheme(name: string) {
        const n = name.toLowerCase();
        if (n.includes('gold')) return 'gold';
        if (n.includes('vip')) return 'vip';
        if (n.includes('diamond')) return 'diamond';
        return 'default';
    }

    function getTierPrice(name: string) {
        const n = name.toLowerCase();
        if (n.includes('gold')) return '20,000';
        if (n.includes('vip')) return '35,000';
        if (n.includes('diamond')) return '70,000';
        return '0';
    }

    function getTierFeatures(name: string, locale: string) {
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
</script>

{#if $upgradeStore.showModal}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div class="modal-overlay" onclick={closeUpgradeModal} role="button" tabindex="0" onkeydown={(e) => e.key === 'Enter' && closeUpgradeModal()}>
        <div class="modal-content glass-effect" onclick={(e) => e.stopPropagation()} role="document">
            <button class="close-btn" onclick={closeUpgradeModal} aria-label="Close modal" disabled={isSubmitting}>
                <X size={20} />
            </button>
            
            <div class="modal-header">
                <h2>{i18n.locale === 'ar' ? 'تم الوصول للحد الأقصى' : 'Limit Reached'}</h2>
                <p class="subtitle">{i18n.locale === 'ar' ? 'لقد وصلت إلى حدود خطتك. قم بالترقية الآن لفتح المزيد من الميزات وزيادة مبيعاتك!' : 'You\'ve reached your plan\'s limits. Upgrade now to unlock more features and boost your sales!'}</p>
            </div>
            
            <div class="modal-body">
                {#if requestResult.success}
                    <div class="success-message">
                        <CheckCircle2 size={48} color="#10b981" />
                        <h3>{i18n.locale === 'ar' ? 'تم إرسال الطلب بنجاح!' : 'Request Sent Successfully!'}</h3>
                        <p>{i18n.locale === 'ar' ? 'سيقوم فريقنا بمراجعة طلبك والتواصل معك قريباً.' : 'Our team will review your request and contact you shortly.'}</p>
                    </div>
                {:else}
                    {#if requestResult.error}
                        <div class="error-banner">
                            <AlertCircle size={16} />
                            {requestResult.error}
                        </div>
                    {/if}
                    
                    <div class="tiers-grid">
                        {#each tiers.filter(t => t.priority_score > 25) as tier}
                            {@const theme = getTierTheme(tier.name)}
                            {@const isPopular = theme === 'vip'}
                            {@const isCurrent = tier.id === currentTierId}
                            {@const allFeatures = getTierFeatures(tier.name, i18n.locale)}
                            {@const visibleFeatures = expandedTiers[tier.name] ? allFeatures : allFeatures.slice(0, 5)}
                            <div class="tier-card theme-{theme}" class:is-popular={isPopular} class:current={isCurrent}>
                                <div class="glow-border"></div>
                                {#if isPopular}
                                    <div class="popular-badge">MOST POPULAR</div>
                                {/if}
                                <div class="tier-badge-container">
                                    <span class="tier-badge">
                                        {#if theme === 'gold'}🏆{:else if theme === 'vip'}👑{:else if theme === 'diamond'}💎{:else}⭐{/if}
                                        {tier.name.toUpperCase()}
                                    </span>
                                    {#if isCurrent}
                                        <span class="current-badge" style="font-size: 0.7rem; font-weight: 700; margin-left: 8px; padding: 2px 8px; background: rgba(0,0,0,0.05); border-radius: 999px;">
                                            {i18n.locale === 'ar' ? 'باقتك الحالية' : 'Current Plan'}
                                        </span>
                                    {/if}
                                </div>
                                
                                <div class="tier-price">
                                    <span class="price-val">{getTierPrice(tier.name)} SAR</span>
                                    <span class="price-curr">YEARLY</span>
                                </div>
                                
                                <ul class="tier-features">
                                    {#each visibleFeatures as feature}
                                        <li>
                                            <Check size={14} class="feature-icon" /> 
                                            <span>{@html feature.text}</span>
                                        </li>
                                    {/each}
                                </ul>
                                
                                {#if allFeatures.length > 5}
                                    <button 
                                        type="button" 
                                        class="btn-read-more" 
                                        onclick={() => toggleTier(tier.name)}
                                    >
                                        {#if expandedTiers[tier.name]}
                                            {i18n.locale === 'ar' ? 'عرض أقل' : 'Show Less'}
                                        {:else}
                                            {i18n.locale === 'ar' ? 'قراءة المزيد' : 'Read More'}
                                        {/if}
                                    </button>
                                {/if}
                                
                                <div class="upgrade-form">
                                    <button 
                                        type="button" 
                                        class="btn-upgrade" 
                                        class:current-plan={isCurrent}
                                        onclick={() => {
                                            closeUpgradeModal();
                                            goto('/dashboard/subscription');
                                        }}
                                        disabled={isCurrent}
                                    >
                                        {#if isCurrent}
                                            {i18n.locale === 'ar' ? 'الباقة الحالية' : 'Current Plan'}
                                        {:else}
                                            {i18n.locale === 'ar' ? 'صفحة الاشتراكات' : 'Go to Subscription'}
                                        {/if}
                                    </button>
                                </div>
                            </div>
                        {/each}
                    </div>
                {/if}
            </div>
        </div>
    </div>
{/if}

<style>
    .modal-overlay {
        position: fixed;
        inset: 0;
        background: rgba(15, 23, 42, 0.5);
        backdrop-filter: blur(8px);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 9999;
        animation: fadeIn 0.25s ease-out;
        padding: 20px;
    }

    .glass-effect {
        background: var(--color-surface);
        border: 1px solid rgba(0, 0, 0, 0.08);
        box-shadow: var(--shadow-2xl);
    }

    .modal-content {
        position: relative;
        width: 100%;
        max-width: 900px;
        max-height: 90vh;
        overflow-y: auto;
        border-radius: 20px;
        padding: 30px;
        animation: slideUp 0.3s cubic-bezier(0.16, 1, 0.3, 1);
        font-family: 'Inter', 'Outfit', sans-serif;
        background: var(--color-surface);
    }

    .close-btn {
        position: absolute;
        top: 20px;
        right: 20px;
        background: transparent;
        border: none;
        width: 32px;
        height: 32px;
        border-radius: 50%;
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: pointer;
        transition: all 0.2s;
        color: #94a3b8;
        z-index: 10;
    }

    .close-btn:hover {
        background: rgba(0,0,0,0.05);
        color: #0f172a;
    }

    .modal-header {
        margin-bottom: 24px;
    }

    .modal-header h2 {
        font-size: 1.5rem;
        font-weight: 800;
        color: #0f172a;
        margin: 0 0 8px;
    }

    .subtitle {
        color: #475569;
        font-size: 0.95rem;
        margin: 0;
        font-weight: 500;
    }

    .tiers-grid {
        display: grid;
        grid-template-columns: repeat(3, 1fr);
        gap: 20px;
        width: 100%;
        position: relative;
        padding-top: 15px; /* Prevent popular-badge from cutting off */
    }

    .tier-card {
        width: 100%;
        border-radius: 16px;
        padding: 24px 16px;
        display: flex;
        flex-direction: column;
        position: relative;
        background: rgba(255, 255, 255, 0.4);
        backdrop-filter: blur(12px);
        -webkit-backdrop-filter: blur(12px);
        border: 1px solid rgba(255, 255, 255, 0.5);
        box-shadow: 0 4px 6px -1px rgba(0,0,0,0.05);
        z-index: 1;
        transition: transform 0.3s ease, border-color 0.3s ease, box-shadow 0.3s ease;
    }

    .glow-border {
        position: absolute;
        inset: 0;
        border-radius: 16px;
        overflow: hidden;
        z-index: -2;
    }

    .glow-border::before {
        content: '';
        position: absolute;
        top: -50%; left: -50%; width: 200%; height: 200%;
        background: conic-gradient(transparent, var(--glow-color, rgba(0,0,0,0.1)), transparent 30%);
        opacity: 0;
        transition: opacity 0.3s ease;
    }

    .tier-card::after {
        content: '';
        position: absolute;
        inset: 2px;
        background: rgba(255, 255, 255, 0.85); /* Glassmorphism inner layer */
        border-radius: 14px;
        z-index: -1;
    }

    .tier-card:hover {
        transform: translateY(-5px);
        border-color: transparent;
        box-shadow: 0 10px 25px -5px rgba(0,0,0,0.1);
    }

    .tier-card:hover .glow-border::before {
        opacity: 1;
        animation: spinBlink 1.5s linear infinite;
    }

    .tier-card.is-popular {
        transform: scale(1.05);
        z-index: 2;
        border-color: #e9d5ff;
    }

    .tier-card.current {
        border-color: rgba(0, 0, 0, 0.15);
        opacity: 0.95;
    }

    .tier-card.current:hover {
        transform: none;
        box-shadow: 0 4px 6px -1px rgba(0,0,0,0.05);
    }

    .tier-card.current:hover .glow-border::before {
        opacity: 0;
    }

    .popular-badge {
        position: absolute;
        top: -12px;
        left: 50%;
        transform: translateX(-50%);
        background: #7c3aed;
        color: white;
        font-size: 0.7rem;
        font-weight: 800;
        padding: 4px 12px;
        border-radius: 999px;
        letter-spacing: 0.05em;
        text-transform: uppercase;
        box-shadow: 0 4px 6px -1px rgba(124, 58, 237, 0.3);
    }

    .tier-badge-container {
        display: flex;
        margin-bottom: 16px;
    }

    .tier-badge {
        display: inline-flex;
        align-items: center;
        gap: 6px;
        font-size: 0.75rem;
        font-weight: 800;
        padding: 4px 10px;
        border-radius: 999px;
        border: 1px solid currentColor;
    }

    /* THEME COLORS */
    .theme-gold { --glow-color: #d97706; }
    .theme-vip { --glow-color: #7c3aed; }
    .theme-diamond { --glow-color: #06b6d4; }

    .theme-gold .tier-badge {
        color: #d97706;
        background: rgba(255, 251, 235, 0.8);
    }
    .theme-vip .tier-badge {
        color: #7c3aed;
        background: rgba(245, 243, 255, 0.8);
    }
    .theme-diamond .tier-badge {
        color: #06b6d4;
        background: rgba(236, 254, 255, 0.8);
    }

    .theme-gold :global(.feature-icon) { color: #d97706; }
    .theme-vip :global(.feature-icon) { color: #7c3aed; }
    .theme-diamond :global(.feature-icon) { color: #06b6d4; }

    .theme-gold .btn-upgrade { background: #d97706; }
    .theme-vip .btn-upgrade { background: #7c3aed; }
    .theme-diamond .btn-upgrade { background: #06b6d4; }

    .tier-price {
        margin-bottom: 16px;
        display: flex;
        flex-direction: column;
    }

    .price-val {
        font-size: 1.4rem;
        font-weight: 800;
        color: #0f172a;
        line-height: 1.1;
    }

    .price-curr {
        font-size: 0.75rem;
        color: #475569;
        font-weight: 600;
        margin-top: 4px;
        letter-spacing: 0.05em;
    }

    .tier-features {
        list-style: none;
        padding: 0;
        margin: 0 0 16px 0;
        flex-grow: 1;
    }

    .tier-features li {
        display: flex;
        align-items: flex-start;
        gap: 8px;
        font-size: 0.8rem;
        color: #334155;
        margin-bottom: 12px;
        line-height: 1.4;
    }

    .tier-features li :global(strong) {
        font-weight: 700;
        color: #1e293b;
    }

    :global(.feature-icon) {
        flex-shrink: 0;
        margin-top: 2px;
    }

    .upgrade-form {
        margin-top: auto;
    }

    .btn-upgrade {
        width: 100%;
        padding: 12px;
        border-radius: 8px;
        color: white;
        font-weight: 700;
        font-size: 0.9rem;
        border: none;
        cursor: pointer;
        transition: all 0.2s;
        box-shadow: 0 4px 6px -1px rgba(0,0,0,0.1);
    }

    .btn-upgrade:hover {
        filter: brightness(1.1);
        transform: translateY(-1px);
        box-shadow: 0 6px 8px -1px rgba(0,0,0,0.15);
    }
    
    .btn-upgrade:disabled:not(.current-plan) {
        opacity: 0.7;
        cursor: not-allowed;
        transform: none;
    }

    .btn-upgrade.current-plan {
        background: #f1f5f9;
        color: #334155;
        border: 1px solid #cbd5e1;
        box-shadow: none;
        opacity: 1;
        cursor: not-allowed;
    }

    .error-banner {
        display: flex;
        align-items: center;
        gap: 8px;
        background: rgba(220, 38, 38, 0.08);
        color: #991b1b;
        padding: 12px;
        border-radius: 8px;
        margin-bottom: 20px;
        font-size: 0.9rem;
        font-weight: 500;
    }
    
    .success-message {
        display: flex;
        flex-direction: column;
        align-items: center;
        text-align: center;
        padding: 40px 20px;
    }
    
    .success-message h3 {
        margin: 16px 0 8px;
        font-size: 1.5rem;
        color: #047857;
    }
    
    .success-message p {
        color: #334155;
        margin: 0;
    }

    .loading-spinner {
        display: inline-block;
        width: 16px;
        height: 16px;
        border: 2px solid rgba(255,255,255,0.3);
        border-radius: 50%;
        border-top-color: #fff;
        animation: spin 1s ease-in-out infinite;
    }

    .btn-read-more {
        background: transparent;
        border: none;
        color: var(--color-primary);
        font-size: 0.85rem;
        font-weight: 600;
        cursor: pointer;
        padding: 0;
        margin-bottom: 20px;
        text-align: left;
        display: inline-block;
        transition: color 0.2s;
    }

    .btn-read-more:hover {
        color: var(--color-primary-hover);
        text-decoration: underline;
    }

    :global(.subscription-page[dir="rtl"]) .btn-read-more {
        text-align: right;
    }

    @media (max-width: 1000px) {
        .tiers-grid {
            grid-template-columns: 1fr;
            gap: 32px;
        }
        .tier-card.is-popular {
            transform: none;
        }
        .tier-card.is-popular:hover {
            transform: translateY(-5px);
        }
    }

    @media (max-width: 900px) {
        .tiers-grid {
            grid-template-columns: 1fr;
        }
        .tier-card.is-popular {
            transform: none;
        }
        .tier-card.is-popular:hover {
            transform: translateY(-5px);
        }
        .modal-content {
            padding: 24px;
            width: 95%;
        }
    }

    @keyframes spin {
        to { transform: rotate(360deg); }
    }

    @keyframes spinBlink {
        0% { transform: rotate(0deg); filter: brightness(1); }
        50% { filter: brightness(1.5); }
        100% { transform: rotate(360deg); filter: brightness(1); }
    }

    @keyframes fadeIn {
        from { opacity: 0; }
        to { opacity: 1; }
    }

    @keyframes slideUp {
        from { opacity: 0; transform: translateY(20px) scale(0.95); }
        to { opacity: 1; transform: translateY(0) scale(1); }
    }
</style>
