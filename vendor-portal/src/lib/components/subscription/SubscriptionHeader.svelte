<script lang="ts">
    let { i18n } = $props<{ i18n: any }>();
</script>

<div class="page-header">
    <div class="header-text">
        <span class="page-tag">{i18n.locale === 'ar' ? 'الاشتراكات' : 'Subscriptions'}</span>
        <h1>{i18n.locale === 'ar' ? 'الباقات والاشتراكات' : 'Subscription Plans'}</h1>
        <p class="subtitle">
            {i18n.locale === 'ar' ? 'اختر الباقة المناسبة لزيادة مبيعاتك والوصول إلى المزيد من العملاء.' : 'Choose the right plan to boost your sales and reach more customers.'}
        </p>
    </div>
</div>
