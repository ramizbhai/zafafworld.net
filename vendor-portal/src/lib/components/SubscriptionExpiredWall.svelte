<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { ShieldAlert, Mail, HelpCircle, Zap } from 'lucide-svelte';
    import { getI18n } from '$lib/i18n/i18n.svelte';
    import { triggerUpgrade } from '$lib/stores/upgradeStore';
    
    import BaseWall from './wall/BaseWall.svelte';
    import ActionRequiredList from './wall/ActionRequiredList.svelte';
    import { WallState } from '../features/vendor/wallState.svelte.js';

    const i18n = getI18n();

    interface Props {
        sessionToken: string;
        vendorName?: string;
        tiers?: any[];
        currentTierId?: string;
    }

    let { sessionToken, vendorName = 'your business', tiers = [], currentTierId = '' }: Props = $props();

    const wallState = new WallState();

    $effect(() => {
        wallState.setSessionToken(sessionToken);
        wallState.startPolling();
        return () => {
            wallState.stopPolling();
        };
    });
</script>

<BaseWall 
    {wallState}
    topbarTitle={i18n.t.expiredWall.topTitle}
    chatTitle={i18n.t.expiredWall.renewalSupport}
    chatEmptyMessage={i18n.t.expiredWall.emptyChat}
>
    <div class="status-header">
        <div class="spinner-circle">
            <ShieldAlert class="lock-icon" size={24} />
        </div>
        <h2>{i18n.t.expiredWall.expiredTitle}</h2>
        <p class="vendor-sub">{@html i18n.t.expiredWall.partnerProfile.replace('{name}', `<strong>${vendorName}</strong>`)}</p>
        <p class="lock-description">
            {i18n.t.expiredWall.haltedDesc}
        </p>
    </div>

    <div class="contact-box">
        <h3>{i18n.t.expiredWall.contactAdmin}</h3>
        <p>{i18n.t.expiredWall.reactivateDesc}</p>
        <div class="contact-details">
            <div class="detail-row">
                <Mail size={16} class="detail-icon" />
                <span>admin@zafafworld.com</span>
            </div>
            <div class="detail-row">
                <HelpCircle size={16} class="detail-icon" />
                <span>{i18n.t.expiredWall.supportLine.replace('{phone}', '+966 50 123 4567')}</span>
            </div>
        </div>
    </div>

    <ActionRequiredList 
        title={i18n.t.expiredWall.submitProof} 
        description={i18n.t.expiredWall.paidDesc} 
        style="margin-bottom: 20px;"
    />

    <div class="upgrade-box" style="background: linear-gradient(135deg, hsl(40, 100%, 98%), #fff); border: 1px solid hsl(40, 96%, 55%); border-radius: 1rem; padding: 1.5rem; text-align: center;">
        <h3 style="margin: 0 0 10px; color: var(--color-text-dark); font-size: 1.1rem; font-family: 'Outfit', 'Cairo', sans-serif;">{i18n.locale === 'ar' ? 'هل تريد الترقية بدلاً من التجديد؟' : 'Want to upgrade instead of renewing?'}</h3>
        <p style="margin: 0 0 15px; color: #475569; font-size: 0.9rem;">{i18n.locale === 'ar' ? 'يمكنك تقديم طلب للترقية إلى باقة أعلى والحصول على ميزات حصرية فور تجديد حسابك.' : 'You can submit a request to upgrade to a higher tier and get exclusive features as soon as your account is renewed.'}</p>
        <button class="btn btn-upgrade" onclick={() => triggerUpgrade('stopped', currentTierId)} style="width: 100%; justify-content: center; background: var(--color-text-dark); color: white; padding: 12px; border-radius: 8px; font-weight: 600; display: flex; align-items: center; gap: 8px; cursor: pointer; border: none;">
            <Zap size={16} />
            {i18n.locale === 'ar' ? 'عرض خيارات الترقية' : 'View Upgrade Options'}
        </button>
    </div>
</BaseWall>

<style>
    .status-header {
        text-align: center;
    }

    .spinner-circle {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 4rem;
        height: 4rem;
        border-radius: 50%;
        background-color: rgba(239, 68, 68, 0.06);
        border: 2px dashed rgba(239, 68, 68, 0.25);
        color: #ef4444;
        margin: 0 auto 1.5rem auto;
    }

    :global(.lock-icon) {
        animation: pulse-warn 2s ease-in-out infinite;
    }

    @keyframes pulse-warn {
        0%, 100% { transform: scale(1); opacity: 1; }
        50% { transform: scale(1.08); opacity: 0.8; }
    }

    .status-header h2 {
        font-size: 1.45rem;
        font-weight: 800;
        color: var(--color-text-dark, #1e293b);
        margin: 0 0 0.5rem 0;
        font-family: var(--font-heading, inherit);
    }

    .vendor-sub {
        font-size: 0.92rem;
        color: #ef4444;
        font-weight: 700;
        margin-top: 0;
        margin-bottom: 1.25rem;
        letter-spacing: 0.2px;
    }

    .lock-description {
        color: #475569;
        font-size: 0.95rem;
        line-height: 1.6;
        margin: 0 0 2rem 0;
    }

    .contact-box {
        background-color: #faf8f5;
        border: 1px solid rgba(0, 0, 0, 0.03);
        border-radius: 1rem;
        padding: 1.5rem;
        width: 100%;
        text-align: start;
    }

    .contact-box h3 {
        font-size: 0.95rem;
        font-weight: 750;
        color: var(--color-text-dark, #1e293b);
        margin-top: 0;
        margin-bottom: 0.5rem;
    }

    .contact-box p {
        font-size: 0.85rem;
        color: #475569;
        line-height: 1.45;
        margin-top: 0;
        margin-bottom: 1rem;
    }

    .contact-details {
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
    }

    .detail-row {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        font-size: 0.88rem;
        color: var(--color-text-dark, #1e293b);
        font-weight: 600;
    }

    :global(.detail-icon) {
        color: var(--color-primary, #5b21b6);
    }

    .btn {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        gap: 0.5rem;
        font-size: 0.85rem;
        font-weight: 700;
        border-radius: 0.5rem;
        border: none;
        cursor: pointer;
        transition: all 0.2s;
    }

    .btn-upgrade:hover {
        background-color: #334155;
    }
</style>
