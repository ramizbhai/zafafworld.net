<script lang="ts">
    import { getI18n } from '$lib/i18n/i18n.svelte';
    import type { ActionData, PageData } from './$types';
    import { createInquiriesState } from '$lib/stores/inquiriesState.svelte';

    import InquiryHeader from '$lib/components/inquiries/InquiryHeader.svelte';
    import InquiryFilterBar from '$lib/components/inquiries/InquiryFilterBar.svelte';
    import InquiryList from '$lib/components/inquiries/InquiryList.svelte';
    import InquiryDetailModal from '$lib/components/inquiries/InquiryDetailModal.svelte';

    import '$lib/components/inquiries/styles.css';

    let { data, form } = $props<{ data: PageData; form: ActionData }>();

    const i18n = getI18n();
    const inquiriesState = createInquiriesState(() => data);
</script>

<svelte:head>
    <title>{i18n.locale === 'ar' ? 'الطلبات والاستفسارات' : 'Leads & Inquiries'} | {i18n.t.common.appName}</title>
</svelte:head>

<div class="inquiries-page" dir={i18n.isRtl ? 'rtl' : 'ltr'}>
    <InquiryHeader {i18n} inquiries={data.inquiries} {form} />
    
    <InquiryFilterBar {i18n} {inquiriesState} />
    
    <InquiryList {i18n} {inquiriesState} />

    <InquiryDetailModal {i18n} {inquiriesState} />
</div>
