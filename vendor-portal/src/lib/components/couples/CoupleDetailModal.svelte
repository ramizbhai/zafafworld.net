<script lang="ts">
    import { enhance } from '$app/forms';
    import { getCountdown, isUrgentLead, getStatusLabel, baselineDate } from "../../services/couples.service";

    let { i18n, couplesState } = $props<{ i18n: any, couplesState: any }>();

    function closeDrawer() {
        couplesState.isDrawerOpen = false;
        couplesState.selectedInquiry = null;
    }

    function handleMarkReadEnhance() {
        couplesState.isSubmitting = true;
        return async ({ result, update }: any) => {
            couplesState.isSubmitting = false;
            await update();
        };
    }
</script>

{#if couplesState.isDrawerOpen && couplesState.activeSelectedInquiry}
    {@const countdown = getCountdown(couplesState.activeSelectedInquiry.wedding_date, i18n.locale)}
    {@const isUrgent = isUrgentLead(couplesState.activeSelectedInquiry.wedding_date, i18n.locale)}
    
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div class="drawer-overlay" onclick={closeDrawer} role="presentation">
        <div class="drawer-card" onclick={(e) => e.stopPropagation()} role="presentation">
            <div class="drawer-glow"></div>
            
            <div class="drawer-header">
                <div class="title-block">
                    <span class="header-pre">{i18n.locale === 'ar' ? 'تفاصيل الطلب' : 'Lead Profile'}</span>
                    <h2>{couplesState.activeSelectedInquiry.customer_name}</h2>
                    <span class="status-indicator" class:pending={couplesState.activeSelectedInquiry.status === 'new'}>
                        {getStatusLabel(couplesState.activeSelectedInquiry.status, i18n)}
                    </span>
                </div>
                <button onclick={closeDrawer} class="close-drawer-btn" aria-label={i18n.locale === 'ar' ? 'إغلاق' : 'Close'}>✕</button>
            </div>

            <div class="drawer-content">
                <div class="countdown-section" class:urgent-countdown={isUrgent} class:past-countdown={countdown.isPast}>
                    <div class="countdown-value">
                        {countdown.text}
                    </div>
                    <p class="countdown-desc">
                        {#if countdown.isPast}
                            {i18n.locale === 'ar' ? 'تاريخ هذه المناسبة قد مر بالفعل.' : "This couple's event date has already occurred."}
                        {:else if isUrgent}
                            ⚠️ {i18n.locale === 'ar' ? 'أولوية قصوى! تقام المناسبة في أقل من 3 أشهر. تواصل مع العرسان فوراً.' : 'High priority! Event occurs in less than 3 months. Contact this couple immediately.'}
                        {:else}
                            {i18n.locale === 'ar' ? 'إطار زمني عادي. هناك متسع من الوقت لجدولة الترتيبات.' : 'Standard priority timeframe. Plenty of time to schedule details.'}
                        {/if}
                    </p>
                </div>

                <div class="drawer-section">
                    <h3>{i18n.locale === 'ar' ? 'معلومات العميل' : 'Customer Parameters'}</h3>
                    <div class="parameters-grid">
                        <div class="param-card">
                            <span class="label">{i18n.locale === 'ar' ? 'رقم الهاتف' : 'Primary Phone'}</span>
                            <span class="value">{couplesState.activeSelectedInquiry.phone}</span>
                            <a href="tel:{couplesState.activeSelectedInquiry.phone}" class="tel-dial-btn">
                                📞 {i18n.locale === 'ar' ? 'اتصال بالرقم' : 'Dial Partner Number'}
                            </a>
                        </div>

                        <div class="param-card">
                            <span class="label">{i18n.t.couples.eventDate}</span>
                            <span class="value">{couplesState.activeSelectedInquiry.wedding_date}</span>
                            <span class="sub-label">{i18n.locale === 'ar' ? 'مقارنة بالحد الحالي' : 'Timeline Baseline'}: {baselineDate}</span>
                        </div>
                    </div>
                </div>

                <div class="drawer-section message-section">
                    <h3>{i18n.locale === 'ar' ? 'رسالة العميل' : 'Customer Note Details'}</h3>
                    <div class="inquiry-message-container">
                        <p class="full-message">{couplesState.activeSelectedInquiry.message || ''}</p>
                    </div>
                </div>
            </div>

            <div class="drawer-footer">
                <button type="button" onclick={closeDrawer} class="back-btn">
                    {i18n.locale === 'ar' ? 'الرجوع للقائمة' : 'Back to Feed'}
                </button>

                {#if couplesState.activeSelectedInquiry.status === 'new'}
                    <form method="POST" action="?/markAsRead" use:enhance={handleMarkReadEnhance}>
                        <input type="hidden" name="id" value={couplesState.activeSelectedInquiry.id} />
                        <button type="submit" disabled={couplesState.isSubmitting} class="action-btn">
                            {#if couplesState.isSubmitting}
                                {i18n.t.common.loading}
                            {:else}
                                {i18n.locale === 'ar' ? 'تحديد كمقروء ✔' : 'Mark as Read ✔'}
                            {/if}
                        </button>
                    </form>
                {:else}
                    <button type="button" disabled class="disabled-read-btn">
                        ✓ {i18n.locale === 'ar' ? 'تمت معالجته' : 'Processed'}
                    </button>
                {/if}
            </div>
        </div>
    </div>
{/if}
