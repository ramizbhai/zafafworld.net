<script lang="ts">
    import { enhance } from "$app/forms";
    import { goto, invalidateAll } from "$app/navigation";
    import { stripHtml, getTargetListingName, getStatusBadgeClass, getStatusLabel, formatDate } from "../../services/offers.service";

    let { offer, i18n, products, offersState } = $props<{
        offer: any;
        i18n: any;
        products: any[];
        offersState: any;
    }>();

    function handleActionEnhance(actionType: 'pause' | 'resume' | 'duplicate') {
        return ({ formData }: { formData: FormData }) => {
            const id = formData.get('id')?.toString() || '';
            if (actionType === 'pause') offersState.pausingId = id;
            if (actionType === 'resume') offersState.resumingId = id;
            if (actionType === 'duplicate') offersState.duplicatingId = id;

            return async ({ result, update }: any) => {
                offersState.pausingId = null;
                offersState.resumingId = null;
                offersState.duplicatingId = null;
                if (result.type === "success") {
                    await invalidateAll();
                    if (actionType === 'duplicate' && result.data?.duplicatedId) {
                        goto(`/dashboard/offers/${result.data.duplicatedId}/edit`);
                    }
                }
                await update();
            };
        };
    }

    function openRenewModal() {
        offersState.offerToRenew = offer;
        offersState.renewDays = 14;
        offersState.isRenewModalOpen = true;
    }

    function openDeleteConfirm() {
        offersState.offerToDelete = offer;
        offersState.isDeleteConfirmOpen = true;
    }
</script>

<div class="promo-card" class:dimmed={offer.derived_status === "Expired" || offer.derived_status === "Cancelled"}>
    <!-- Discount/Benefit Ribbon -->
    {#if offer.promo_type === "discount"}
        <div class="discount-badge">
            {#if offer.discount_type === "percentage"}
                {offer.discount_percentage}% {i18n.locale === "ar" ? "خصم" : "OFF"}
            {:else}
                {offer.discount_fixed_amount} {i18n.locale === "ar" ? "ر.س" : "SAR"}
            {/if}
        </div>
    {:else}
        <div class="discount-badge benefit-badge">
            🎁 {i18n.locale === "ar" ? "عرض قيمة" : "Added Value"}
        </div>
    {/if}

    <div class="promo-body">
        <!-- Status badge -->
        <div class="status-row">
            <span class="status-indicator-badge {getStatusBadgeClass(offer.derived_status)}">
                {getStatusLabel(offer.derived_status, i18n.locale)}
            </span>
            {#if offer.badge_text_en || offer.badge_text_ar}
                <span class="custom-badge-tag">
                    🏷️ {i18n.locale === "ar" ? offer.badge_text_ar || offer.badge_text_en : offer.badge_text_en || offer.badge_text_ar}
                </span>
            {/if}
        </div>

        <!-- Title -->
        <h3>{i18n.locale === "ar" ? offer.title_ar : offer.title_en}</h3>

        <!-- Description preview -->
        {#if offer.description_en || offer.description_ar}
            <p class="description-preview">
                {stripHtml(i18n.locale === "ar" ? offer.description_ar || offer.description_en : offer.description_en || offer.description_ar)}
            </p>
        {/if}

        <!-- Benefit Details -->
        {#if offer.promo_type === "benefit" && (offer.benefit_description_en || offer.benefit_description_ar)}
            <div class="benefit-callout">
                🌟 <strong>{i18n.locale === "ar" ? "الميزة المضافة:" : "Added Value Benefit:"}</strong>
                <p>{i18n.locale === "ar" ? offer.benefit_description_ar || offer.benefit_description_en : offer.benefit_description_en || offer.benefit_description_ar}</p>
            </div>
        {/if}

        <!-- Rejection reason -->
        {#if offer.derived_status === "Rejected" && offer.rejection_reason}
            <div class="rejection-callout">
                <span class="rejection-icon">⚠️</span>
                <div>
                    <strong>{i18n.locale === "ar" ? "سبب الرفض:" : "Rejection Reason:"}</strong>
                    <p>{offer.rejection_reason}</p>
                </div>
            </div>
        {/if}

        <!-- Target listing -->
        <div class="target-listings">
            <span class="meta-label">🎯 {i18n.locale === "ar" ? "المستهدف:" : "Target:"}</span>
            <span class="meta-value">{getTargetListingName(offer.listing_id, products, i18n.locale) || "—"}</span>
        </div>

        <!-- Dates -->
        <div class="dates-row">
            <span class="date-chip">📅 {formatDate(offer.start_at, i18n.locale)} → {formatDate(offer.end_at, i18n.locale)}</span>
        </div>

        <!-- Analytics -->
        {#if offer.derived_analytics && (offer.derived_analytics.views > 0 || offer.derived_analytics.clicks > 0)}
            <div class="analytics-metrics-panel">
                <div class="metric-item">
                    <span class="metric-val">{offer.derived_analytics.views}</span>
                    <span class="metric-lbl">{i18n.locale === "ar" ? "مشاهدات" : "Views"}</span>
                </div>
                <div class="metric-item">
                    <span class="metric-val">{offer.derived_analytics.clicks}</span>
                    <span class="metric-lbl">{i18n.locale === "ar" ? "نقرات" : "Clicks"}</span>
                </div>
                <div class="metric-item">
                    <span class="metric-val">{Math.round(offer.derived_analytics.ctr * 100)}%</span>
                    <span class="metric-lbl">CTR</span>
                </div>
            </div>
        {/if}
    </div>

    <!-- Card Actions -->
    <div class="promo-actions">
        <div class="state-toggles">
            {#if offer.derived_status === "Active"}
                <form method="POST" action="?/pause" use:enhance={handleActionEnhance('pause')}>
                    <input type="hidden" name="id" value={offer.id} />
                    <button type="submit" class="action-toggle-btn pause-btn" disabled={offersState.pausingId === offer.id} title={i18n.locale === "ar" ? "إيقاف مؤقت" : "Pause"}>
                        {#if offersState.pausingId === offer.id}⏳{:else}⏸️ {i18n.locale === "ar" ? "إيقاف" : "Pause"}{/if}
                    </button>
                </form>
            {:else if offer.derived_status === "Paused"}
                <form method="POST" action="?/resume" use:enhance={handleActionEnhance('resume')}>
                    <input type="hidden" name="id" value={offer.id} />
                    <button type="submit" class="action-toggle-btn resume-btn" disabled={offersState.resumingId === offer.id} title={i18n.locale === "ar" ? "استئناف" : "Resume"}>
                        {#if offersState.resumingId === offer.id}⏳{:else}▶️ {i18n.locale === "ar" ? "تنشيط" : "Resume"}{/if}
                    </button>
                </form>
            {/if}
        </div>

        <div class="crud-buttons">
            {#if offer.derived_status === "Draft" || offer.derived_status === "Pending" || offer.derived_status === "Scheduled" || offer.derived_status === "Paused" || offer.derived_status === "Rejected" || offer.derived_status === "Active"}
                <a href="/dashboard/offers/{offer.id}/edit" class="action-btn edit-btn" title={i18n.t.common.edit}>✏️ {i18n.locale === "ar" ? "تعديل" : "Edit"}</a>
            {/if}

            {#if offer.derived_status === "Expired" || offer.derived_status === "Rejected" || offer.derived_status === "Cancelled"}
                <form method="POST" action="?/duplicate" use:enhance={handleActionEnhance('duplicate')}>
                    <input type="hidden" name="id" value={offer.id} />
                    <button type="submit" class="action-btn duplicate-btn" disabled={offersState.duplicatingId === offer.id} title={i18n.locale === "ar" ? "نسخ" : "Duplicate"}>
                        {#if offersState.duplicatingId === offer.id}⏳{:else}👯 {i18n.locale === "ar" ? "نسخ" : "Copy"}{/if}
                    </button>
                </form>
            {/if}

            {#if offer.derived_status === "Expired"}
                <button onclick={openRenewModal} class="action-btn renew-btn" title={i18n.locale === "ar" ? "تجديد" : "Renew"}>🔄 {i18n.locale === "ar" ? "تجديد" : "Renew"}</button>
            {/if}

            {#if offer.derived_status !== "Cancelled" && offer.derived_status !== "Expired"}
                <button onclick={openDeleteConfirm} class="action-btn delete-btn" title={i18n.t.common.delete}>🗑️</button>
            {/if}
        </div>
    </div>
</div>
