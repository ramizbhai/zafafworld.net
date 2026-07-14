<script lang="ts">
    import { getI18n } from '$lib/i18n/i18n.svelte';
    import { page } from '$app/stores';
    import type { ActionData, PageData } from './$types';
    import { createCouplesState } from '$lib/stores/couplesState.svelte';

    import CouplesHeader from '$lib/components/couples/CouplesHeader.svelte';
    import CouplesFilter from '$lib/components/couples/CouplesFilter.svelte';
    import CouplesList from '$lib/components/couples/CouplesList.svelte';
    import CoupleDetailModal from '$lib/components/couples/CoupleDetailModal.svelte';
    import CoupleAddModal from '$lib/components/couples/CoupleAddModal.svelte';

    import '$lib/components/couples/styles.css';

    let { data, form } = $props<{ data: PageData; form: ActionData }>();

    const i18n = getI18n();
    const couplesState = createCouplesState($page.url, () => data);
</script>

<svelte:head>
    <title>{i18n.t.couples.title} | {i18n.t.common.appName}</title>
</svelte:head>

<div class="inquiries-page" dir={i18n.isRtl ? 'rtl' : 'ltr'}>
    <CouplesHeader {i18n} inquiries={data.inquiries} {form} />
    
    <CouplesFilter {i18n} {couplesState} />
    
    <CouplesList {i18n} {couplesState} {data} />

    <CoupleDetailModal {i18n} {couplesState} />

    <CoupleAddModal {i18n} {couplesState} />
</div>
