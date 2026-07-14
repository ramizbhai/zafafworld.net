<script lang="ts">
    import { getI18n } from "$lib/i18n/i18n.svelte";
    import type { ActionData, PageData } from "./$types";
    import { createOffersState } from "$lib/stores/offersState.svelte";

    import OffersHeader from "$lib/components/offers/OffersHeader.svelte";
    import OffersToolbar from "$lib/components/offers/OffersToolbar.svelte";
    import OffersGrid from "$lib/components/offers/OffersGrid.svelte";
    import OffersModals from "$lib/components/offers/OffersModals.svelte";

    import "$lib/components/offers/styles.css";

    let { data, form } = $props<{ data: PageData; form: ActionData }>();

    const i18n = getI18n();
    const offersState = createOffersState(() => data);

</script>

<svelte:head>
    <title>{i18n.locale === "ar" ? "إدارة العروض الترويجية" : "Manage Promotions"} | {i18n.t.common.appName}</title>
</svelte:head>

<div class="promotions-page" dir={i18n.isRtl ? "rtl" : "ltr"}>
    <OffersHeader 
        {i18n} 
        formError={form?.error} 
        formSuccess={form?.success} 
        formMessage={form?.message} 
    />

    <OffersToolbar {i18n} {offersState} />

    <OffersGrid {i18n} {offersState} {data} />

    <OffersModals {i18n} {offersState} />
</div>
