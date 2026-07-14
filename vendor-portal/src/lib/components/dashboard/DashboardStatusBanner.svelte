<script lang="ts">
    import { CheckCircle2, AlertCircle } from 'lucide-svelte';

    let { i18n, vendorStatus } = $props<{
        i18n: any;
        vendorStatus: string;
    }>();
</script>

{#if vendorStatus === 'active'}
    <div class="status-banner status-approved">
        <div class="status-banner-glow"></div>
        <div class="status-icon-wrap">
            <CheckCircle2 size={22} />
        </div>
        <div class="status-text">
            <strong>
                {i18n.locale === 'ar' ? 'حسابك نشط ومباشر' : 'Account Active & Live'}
            </strong>
            <span>
                {i18n.locale === 'ar'
                    ? 'تظهر إعلاناتك المعتمدة بشكل كامل على سوق زفاف العالمي للعرسان.'
                    : 'Your approved listings are fully visible on the ZafafWorld marketplace to couples.'}
            </span>
        </div>
        <div class="status-pulse">
            <span class="pulse-dot green"></span>
            <span class="pulse-label">
                {i18n.locale === 'ar' ? 'مباشر الآن' : 'Live now'}
            </span>
        </div>
    </div>
{:else if vendorStatus === 'suspended'}
    <div class="status-banner status-pending">
        <div class="status-banner-glow pending-glow"></div>
        <div class="status-icon-wrap pending-icon">
            <AlertCircle size={22} />
        </div>
        <div class="status-text">
            <strong>
                {i18n.locale === 'ar' ? 'تم تعليق حسابك مؤقتاً' : 'Account Temporarily Suspended'}
            </strong>
            <span>
                {i18n.locale === 'ar'
                    ? 'بعض إعلاناتك موقوفة. تواصل مع فريق الدعم لمعرفة التفاصيل.'
                    : 'Some listings are paused. Contact support for details on reinstatement.'}
            </span>
        </div>
        <div class="status-pulse">
            <span class="pulse-dot amber"></span>
            <span class="pulse-label">
                {i18n.locale === 'ar' ? 'موقوف' : 'Suspended'}
            </span>
        </div>
    </div>
{/if}
