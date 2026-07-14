<script lang="ts">
    import { getI18n } from '$lib/i18n/i18n.svelte';
    import { CalendarClock, Zap } from 'lucide-svelte';
    import { triggerUpgrade } from '$lib/stores/upgradeStore';

    const i18n = getI18n();
    export let vendor: any = {};

    let tierName = vendor.tier_name || vendor.subscriptionTierName || 'Free';
    let expiresAt = vendor.subscription_expires_at || vendor.subscriptionExpiresAt;
    
    let daysLeft = 0;
    let isExpiringSoon = false;

    if (expiresAt) {
        const expiryDate = new Date(expiresAt);
        const now = new Date();
        const diffTime = expiryDate.getTime() - now.getTime();
        daysLeft = Math.ceil(diffTime / (1000 * 60 * 60 * 24));
        isExpiringSoon = daysLeft <= 14 && daysLeft > 0;
    }

    function formatDate(dateStr: string): string {
        if (!dateStr) return '';
        return new Date(dateStr).toLocaleDateString(
            i18n.locale === 'ar' ? 'ar-EG' : 'en-US',
            { month: 'short', day: 'numeric', year: 'numeric' }
        );
    }

    // Tier theme: icon emoji + color scheme
    function getTierTheme(name: string): { emoji: string; theme: 'gold' | 'vip' | 'diamond' | 'free' } {
        const n = (name || '').toLowerCase();
        if (n.includes('diamond')) return { emoji: '💎', theme: 'diamond' };
        if (n.includes('vip'))     return { emoji: '👑', theme: 'vip' };
        if (n.includes('gold'))    return { emoji: '🏆', theme: 'gold' };
        return { emoji: '⭐', theme: 'free' };
    }

    $: theme = getTierTheme(tierName);

    function handleUpgrade() {
        triggerUpgrade('dashboard', tierName);
    }
</script>

<div class="subscription-card" class:expiring={isExpiringSoon} class:tier-gold={theme.theme === 'gold'} class:tier-vip={theme.theme === 'vip'} class:tier-diamond={theme.theme === 'diamond'}>
    <div class="card-left">
        <div class="plan-icon-wrapper tier-{theme.theme}">
            <span class="tier-emoji">{theme.emoji}</span>
        </div>
        <div class="plan-details">
            <h3 class="plan-title">
                {i18n.locale === 'ar' ? 'الباقة الحالية:' : 'Current Plan:'}
                <strong class="tier-name-text">{tierName}</strong>
            </h3>
            <div class="plan-meta">
                {#if expiresAt && isExpiringSoon}
                    <span class="days-left alert-text">
                        <CalendarClock size={13} />
                        {i18n.locale === 'ar' ? `ينتهي خلال ${daysLeft} أيام` : `Expires in ${daysLeft} days`}
                    </span>
                {:else if expiresAt && daysLeft > 0}
                    <span class="expiry-date">
                        <CalendarClock size={14} />
                        {i18n.locale === 'ar' ? 'ينتهي في:' : 'Expires on:'} {formatDate(expiresAt)}
                    </span>
                {/if}
            </div>
        </div>
    </div>
    
    <div class="card-right">
        <button class="btn-upgrade tier-btn-{theme.theme}" onclick={handleUpgrade}>
            <Zap size={16} />
            {i18n.locale === 'ar' ? 'ترقية الباقة' : 'Upgrade Plan'}
        </button>
    </div>
</div>

<style>
    .subscription-card {
        display: flex;
        align-items: center;
        justify-content: space-between;
        background: var(--white);
        border: 1px solid var(--border);
        border-radius: var(--radius-lg);
        padding: 20px 24px;
        box-shadow: var(--shadow-sm);
        margin-bottom: 24px;
        transition: all 0.2s ease;
        flex-wrap: wrap;
        gap: 16px;
    }

    .subscription-card.expiring {
        border-color: hsl(38, 92%, 50%);
        background: hsl(40, 100%, 98%);
    }

    /* Tier accent borders */
    .subscription-card.tier-gold    { border-color: hsl(40, 96%, 70%); }
    .subscription-card.tier-vip     { border-color: hsl(258, 80%, 75%); }
    .subscription-card.tier-diamond { border-color: hsl(192, 90%, 65%); }

    .card-left {
        display: flex;
        align-items: center;
        gap: 16px;
    }

    /* Icon wrapper — theme-specific gradients */
    .plan-icon-wrapper {
        width: 48px;
        height: 48px;
        border-radius: 12px;
        display: flex;
        align-items: center;
        justify-content: center;
        background: var(--bg);
        flex-shrink: 0;
    }

    .plan-icon-wrapper.tier-free {
        background: var(--bg);
    }
    .plan-icon-wrapper.tier-gold {
        background: linear-gradient(135deg, hsl(40, 96%, 55%), hsl(30, 96%, 50%));
        box-shadow: 0 4px 12px rgba(234, 179, 8, 0.25);
    }
    .plan-icon-wrapper.tier-vip {
        background: linear-gradient(135deg, hsl(258, 80%, 60%), hsl(270, 70%, 50%));
        box-shadow: 0 4px 12px rgba(124, 58, 237, 0.25);
    }
    .plan-icon-wrapper.tier-diamond {
        background: linear-gradient(135deg, hsl(192, 90%, 50%), hsl(210, 80%, 50%));
        box-shadow: 0 4px 12px rgba(6, 182, 212, 0.25);
    }

    .tier-emoji {
        font-size: 22px;
        line-height: 1;
    }

    .plan-title {
        font-size: 16px;
        margin: 0 0 4px 0;
        color: var(--text-sec);
        font-weight: 500;
    }

    .tier-name-text {
        font-weight: 800;
        font-size: 18px;
        color: var(--text);
        margin-inline-start: 4px;
    }

    .plan-meta {
        display: flex;
        align-items: center;
        gap: 12px;
        font-size: 13px;
        color: var(--text-sec);
        flex-wrap: wrap;
    }

    .expiry-date {
        display: flex;
        align-items: center;
        gap: 6px;
    }

    .alert-text {
        display: flex;
        align-items: center;
        gap: 5px;
        color: hsl(0, 84%, 45%);
        font-weight: 600;
    }

    .card-right {
        display: flex;
        align-items: center;
    }

    /* Upgrade button base */
    .btn-upgrade {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 10px 20px;
        border-radius: 8px;
        background: var(--text);
        color: white;
        font-weight: 600;
        font-size: 14px;
        border: none;
        cursor: pointer;
        transition: all 0.2s ease;
    }

    .btn-upgrade:hover {
        transform: translateY(-1px);
        filter: brightness(1.12);
    }

    /* Tier-specific button colours */
    .tier-btn-gold    { background: linear-gradient(135deg, hsl(40, 96%, 50%), hsl(30, 96%, 46%)); }
    .tier-btn-vip     { background: linear-gradient(135deg, hsl(258, 80%, 58%), hsl(270, 70%, 50%)); }
    .tier-btn-diamond { background: linear-gradient(135deg, hsl(192, 90%, 44%), hsl(210, 80%, 46%)); }
    .tier-btn-free    { background: var(--text); }
</style>
