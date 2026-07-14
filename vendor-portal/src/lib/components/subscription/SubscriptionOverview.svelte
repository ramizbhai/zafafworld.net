<script lang="ts">
    import { Check } from 'lucide-svelte';
    import { getTierTheme, getTierFeatures } from "../../services/subscription.service";

    let { i18n, subscriptionState } = $props<{ i18n: any, subscriptionState: any }>();
</script>

<div class="overview-section animate-fade-in">
    {#if subscriptionState.activeTier}
        {@const theme = getTierTheme(subscriptionState.activeTier.name)}
        <div class="active-plan-card theme-{theme}">
            <div class="glow-border"></div>
            <div class="plan-header">
                <div class="tier-badge-container">
                    <span class="tier-badge">
                        {#if theme === 'gold'}🏆{:else if theme === 'vip'}👑{:else if theme === 'diamond'}💎{:else}⭐{/if}
                        {subscriptionState.activeTier.name.toUpperCase()}
                    </span>
                </div>
                {#if subscriptionState.expirationDate}
                    <div class="validity-badge">
                        {i18n.locale === 'ar' ? 'صالح حتى' : 'Valid until'}: 
                        {new Date(subscriptionState.expirationDate).toLocaleDateString(i18n.locale === 'ar' ? 'ar-SA' : 'en-US', { year: 'numeric', month: 'short', day: 'numeric' })}
                    </div>
                {/if}
            </div>
            
            <div class="plan-details">
                <div class="plan-info">
                    <h3>{i18n.locale === 'ar' ? 'تفاصيل الباقة الحالية' : 'Current Plan Details'}</h3>
                    <p>{i18n.locale === 'ar' ? 'أنت تستمتع حالياً بمميزات هذه الباقة.' : 'You are currently enjoying the benefits of this plan.'}</p>
                    
                    {#if subscriptionState.vendorLimits}
                        <div class="limits-box">
                            <h4>{i18n.locale === 'ar' ? 'حدود الاستخدام' : 'Usage Limits'}</h4>
                            <ul>
                                <li>
                                    <strong>{i18n.locale === 'ar' ? 'أقصى عدد للقاعات' : 'Max Venues'}:</strong> 
                                    {subscriptionState.vendorLimits.max_products === -1 ? (i18n.locale === 'ar' ? 'غير محدود' : 'Unlimited') : subscriptionState.vendorLimits.max_products}
                                </li>
                                <li>
                                    <strong>{i18n.locale === 'ar' ? 'أقصى عدد للصور الإضافية' : 'Max Extra Photos'}:</strong> 
                                    {subscriptionState.vendorLimits.max_additional_photos === -1 ? (i18n.locale === 'ar' ? 'غير محدود' : 'Unlimited') : subscriptionState.vendorLimits.max_additional_photos}
                                </li>
                                <li>
                                    <strong>{i18n.locale === 'ar' ? 'أقصى عدد للفيديوهات' : 'Max Videos'}:</strong> 
                                    {subscriptionState.vendorLimits.max_videos === -1 ? (i18n.locale === 'ar' ? 'غير محدود' : 'Unlimited') : subscriptionState.vendorLimits.max_videos}
                                </li>
                            </ul>
                        </div>
                    {/if}
                </div>
                
                <div class="plan-features-list">
                    <h4>{i18n.locale === 'ar' ? 'المميزات' : 'Features'}</h4>
                    <ul class="tier-features">
                        {#each getTierFeatures(subscriptionState.activeTier.name, i18n.locale) as feature}
                            <li>
                                <Check size={14} class="feature-icon" /> 
                                <span>{@html feature.text}</span>
                            </li>
                        {/each}
                    </ul>
                </div>
            </div>
        </div>
    {:else}
        <div class="active-plan-card theme-default free-plan">
            <div class="plan-header">
                <div class="tier-badge-container">
                    <span class="tier-badge">
                        🤍 {i18n.locale === 'ar' ? 'الباقة المجانية' : 'FREE PLAN'}
                    </span>
                </div>
                <div class="validity-badge">
                    {i18n.locale === 'ar' ? 'دائم' : 'Forever'}
                </div>
            </div>
            
            <div class="plan-details">
                <div class="plan-info">
                    <h3>{i18n.locale === 'ar' ? 'أنت على الباقة المجانية الأساسية' : 'You are on the Basic Free Plan'}</h3>
                    <p>{i18n.locale === 'ar' ? 'استكشف باقاتنا المدفوعة للوصول إلى المزيد من العملاء والحصول على مميزات حصرية.' : 'Explore our paid plans to reach more customers and unlock exclusive features.'}</p>
                    
                    <button class="btn-primary mt-4" onclick={() => subscriptionState.activeTab = 'upgrade'}>
                        {i18n.locale === 'ar' ? 'استعرض باقات الترقية' : 'View Upgrade Plans'}
                    </button>
                </div>
                
                <div class="plan-features-list">
                    <h4>{i18n.locale === 'ar' ? 'مميزات الباقة المجانية' : 'Free Plan Features'}</h4>
                    <ul class="tier-features">
                        <li><Check size={14} class="feature-icon" /> <span>{i18n.locale === 'ar' ? 'إضافة قاعة واحدة' : 'Add 1 Venue'}</span></li>
                        <li><Check size={14} class="feature-icon" /> <span>{i18n.locale === 'ar' ? 'صور أساسية' : 'Basic Photos'}</span></li>
                        <li><Check size={14} class="feature-icon" /> <span>{i18n.locale === 'ar' ? 'استقبال الاستفسارات' : 'Receive Inquiries'}</span></li>
                    </ul>
                </div>
            </div>
        </div>
    {/if}
</div>
