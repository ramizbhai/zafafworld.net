<script lang="ts">
    let { i18n, inquiries, form } = $props<{ i18n: any, inquiries: any[], form: any }>();
</script>

<div class="page-header">
    <div class="title-section">
        <span class="category-tag">{i18n.locale === 'ar' ? 'تدفق الاستفسارات والطلبات' : 'Sales Telemetry Pipeline'}</span>
        <h1>{i18n.t.couples.title}</h1>
        <p class="subtitle">{i18n.locale === 'ar' ? 'إدارة قنوات التواصل المباشرة، مراجعة تفاصيل العرسان وتحديد أولويات المناسبات القادمة.' : 'Secure direct communication channels, review client details, and prioritize upcoming events.'}</p>
    </div>

    <div class="telemetry-pill">
        <span class="pulse-dot"></span>
        <span class="pill-label">
            {(inquiries || []).filter((inq: any) => inq.status === 'new').length} {i18n.locale === 'ar' ? 'جديد' : 'New'}
        </span>
    </div>
</div>

{#if form?.success}
    <div class="status-banner success-banner">
        <span class="icon">✅</span>
        <div class="text">
            <h3>{i18n.locale === 'ar' ? 'تم تحديث الاستفسار' : 'Lead Updated'}</h3>
            <p>{form.message ?? (i18n.locale === 'ar' ? 'تمت العملية بنجاح!' : 'Status changed successfully!')}</p>
        </div>
    </div>
{/if}
