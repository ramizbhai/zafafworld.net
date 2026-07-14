<script lang="ts">
    let { i18n, formError, formSuccess, formMessage } = $props<{
        i18n: any;
        formError?: string;
        formSuccess?: boolean;
        formMessage?: string;
    }>();
</script>

<!-- ─── HEADER ────────────────────────────────────────────────────────── -->
<div class="page-header">
    <div class="title-section">
        <span class="category-tag">
            {i18n.locale === "ar" ? "نظام العروض الترويجية" : "Promotions System"}
        </span>
        <h1>
            {i18n.locale === "ar" ? "إدارة العروض الترويجية" : "Manage Promotions"}
        </h1>
        <p class="subtitle">
            {i18n.locale === "ar"
                ? "أنشئ عروضاً مستهدفة لصالاتك وخدماتك واجذب اهتمام العرسان."
                : "Create targeted promotions for your listings and attract more couples."}
        </p>
    </div>
    <a href="/dashboard/offers/new" class="btn btn-primary">
        <span>+</span>
        {i18n.locale === "ar" ? "إنشاء عرض" : "Create Promotion"}
    </a>
</div>

<!-- ─── STATUS NOTIFICATIONS ──────────────────────────────────────────── -->
{#if formError}
    <div class="status-banner error-banner">
        <div class="banner-icon">⚠️</div>
        <div class="banner-text">
            <h3>{i18n.locale === "ar" ? "خطأ" : "Error"}</h3>
            <p>{formError}</p>
        </div>
    </div>
{/if}

{#if formSuccess}
    <div class="status-banner success-banner">
        <div class="banner-icon">✅</div>
        <div class="banner-text">
            <h3>{i18n.locale === "ar" ? "تمت العملية بنجاح" : "Success"}</h3>
            <p>{formMessage ?? (i18n.locale === "ar" ? "تمت العملية بنجاح." : "Operation completed successfully.")}</p>
        </div>
    </div>
{/if}
