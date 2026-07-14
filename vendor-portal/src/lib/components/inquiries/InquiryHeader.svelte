<script lang="ts">
    let { i18n, inquiries, form } = $props<{ i18n: any, inquiries: any[], form: any }>();
</script>

<div class="page-header">
    <div class="title-section">
        <span class="category-tag">{i18n.locale === 'ar' ? 'استفسارات العملاء المباشرة' : 'Direct Customer Leads'}</span>
        <h1>{i18n.locale === 'ar' ? 'الطلبات والاستفسارات' : 'Leads & Inquiries'}</h1>
        <p class="subtitle">{i18n.locale === 'ar' ? 'راجع تفاصيل طلبات العرسان والتوفر، وحدث حالة الطلبات وقم بإنهاء الصفقات بنجاح.' : 'Review couple event requests, manage booking availabilities, and finalize customer quotes.'}</p>
    </div>

    <div class="telemetry-pill">
        <span class="pulse-dot"></span>
        <span class="pill-label">
            {(inquiries || []).filter((inq: any) => inq.status === 'pending').length} {i18n.locale === 'ar' ? 'جديد' : 'Pending'}
        </span>
    </div>
</div>

{#if form?.success}
    <div class="status-banner success-banner">
        <span class="icon">✅</span>
        <div class="text">
            <h3>{i18n.locale === 'ar' ? 'تم تحديث حالة الطلب' : 'Lead Status Updated'}</h3>
            <p>{form.message ?? (i18n.locale === 'ar' ? 'تمت العملية بنجاح!' : 'Status changed successfully!')}</p>
        </div>
    </div>
{/if}
{#if form?.error}
    <div class="status-banner error-banner">
        <span class="icon">❌</span>
        <div class="text">
            <h3>{i18n.locale === 'ar' ? 'خطأ في التحديث' : 'Update Failed'}</h3>
            <p>{form.error}</p>
        </div>
    </div>
{/if}
