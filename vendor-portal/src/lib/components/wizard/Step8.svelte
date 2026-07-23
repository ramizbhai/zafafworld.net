<script lang="ts">
    import { getContext, onMount } from "svelte";
    import { getI18n } from "$lib/i18n/i18n.svelte";
    import { listingStore } from "$lib/stores/listingStore";
    import Step8Summary from "./Step8Summary.svelte";
    import { goto } from "$app/navigation";
    import { page } from "$app/stores";

    let { data } = $props<{ data: { sessionToken: string, cities: any[] } }>();
    const i18n = getI18n();

    const wizard = getContext<{
        registerSubmitHandler: (handler: () => Promise<void>) => () => void;
        setCanContinue: (val: boolean) => void;
        setSubmitting: (val: boolean) => void;
    }>('wizard');

    const fd = $derived($listingStore.formData);
    const coverItem = $derived($listingStore.formData.coverItem);

    const checklist = $derived([
        { label: i18n.locale === "ar" ? "العنوان بالعربية" : "Arabic Title", ok: !!(fd.titleAr || '').trim() },
        { label: i18n.locale === "ar" ? "العنوان بالإنجليزية" : "English Title", ok: !!(fd.titleEn || '').trim() },
        { label: i18n.locale === "ar" ? "الوصف (50 حرفاً على الأقل بالعربية أو الإنجليزية)" : "Description (at least 50 chars in English or Arabic)", ok: (fd.descriptionAr || '').trim().length >= 50 || (fd.descriptionEn || '').trim().length >= 50 },
        { label: i18n.locale === "ar" ? "تحديد السعر" : "Price set", ok: fd.priceOnInquiry || (!!fd.basePriceSar && parseFloat(String(fd.basePriceSar)) > 0) },
        { label: i18n.locale === "ar" ? "تصنيف الجنسين" : "Gender Section", ok: !!fd.genderSection },
        { label: i18n.locale === "ar" ? "اختيار المدينة" : "City selected", ok: !!fd.selectedCityId },
        { label: i18n.locale === "ar" ? "صورة الغلاف" : "Cover Photo", ok: !!coverItem && coverItem.status === "completed" },
        { label: i18n.locale === "ar" ? "منسق الخدمة" : "Service Coordinator", ok: !!((fd.coordinatorNameAr || '').trim() && (fd.coordinatorNameEn || '').trim() && (fd.coordinatorPhone || '').trim() && (fd.coordinatorWhatsapp || '').trim() && (fd.coordinatorEmail || '').trim()) }
    ]);

    const canSubmit = $derived(checklist.every((c) => c.ok));

    $effect(() => {
        wizard.setCanContinue(true);
    });

    onMount(() => {
        const unregister = wizard.registerSubmitHandler(async () => {
            if (!canSubmit) {
                listingStore.setError("Please complete all required fields before reviewing.");
                return;
            }
            listingStore.setHighestStep(8);
            await goto(`${$page.url.pathname.split("/step-")[0]}/step-9`);
        });
        return unregister;
    });
</script>

<div class="step-pane fade-in">
    <div class="step-heading">
        <div class="step-icon-text">👁️</div>
        <div>
            <h1>{i18n.locale === "ar" ? "معاينة الإعلان" : "Preview Your Listing"}</h1>
            <p>{i18n.locale === "ar" ? "راجع كل شيء قبل التقديم لموافقة الإدارة." : "Review everything before submitting."}</p>
        </div>
    </div>

    {#if $listingStore.submitError}
        <div class="error-banner">{$listingStore.submitError}</div>
    {/if}

    <Step8Summary {data} {checklist} />
</div>
