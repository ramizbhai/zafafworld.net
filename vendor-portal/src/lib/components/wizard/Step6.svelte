<script lang="ts">
    import { getContext, onMount, onDestroy } from "svelte";
    import { getI18n } from "$lib/i18n/i18n.svelte";
    import { listingStore } from "$lib/stores/listingStore";
    import { Phone } from "lucide-svelte";
    import { goto } from "$app/navigation";
    import { page } from "$app/stores";
    import { getApiUrl } from "$lib/utils/api";
    import { wizardFetch } from "$lib/utils/wizardFetch";
    import ListingSummaryReflection from "./ListingSummaryReflection.svelte";

    const i18n = getI18n();
    let { data } = $props<{ data: { sessionToken: string } }>();

    const wizard = getContext<{
        registerSubmitHandler: (handler: () => Promise<void>) => () => void;
        setCanContinue: (val: boolean) => void;
        setSubmitting: (val: boolean) => void;
    }>('wizard');

    let coordinatorNameEn = $state($listingStore.formData.coordinatorNameEn);
    let coordinatorNameAr = $state($listingStore.formData.coordinatorNameAr);
    let coordinatorPhone = $state($listingStore.formData.coordinatorPhone);
    let coordinatorWhatsapp = $state($listingStore.formData.coordinatorWhatsapp);
    let coordinatorEmail = $state($listingStore.formData.coordinatorEmail);
    let coordinatorMobile = $state($listingStore.formData.coordinatorMobile);

    let isValid = $derived(
        coordinatorNameEn.trim().length > 0 &&
        coordinatorNameAr.trim().length > 0 &&
        coordinatorPhone.trim().length > 0 &&
        coordinatorWhatsapp.trim().length > 0 &&
        (coordinatorEmail.trim().length > 0 && /.+@.+\..+/.test(coordinatorEmail))
    );

    $effect(() => {
        wizard.setCanContinue(isValid);
    });

    // Sync state on unmount (synchronous safety net)
    onDestroy(() => {
        listingStore.updateFormData({
            coordinatorNameEn,
            coordinatorNameAr,
            coordinatorPhone,
            coordinatorWhatsapp,
            coordinatorEmail,
            coordinatorMobile,
        });
    });

    // 300ms debounced background sync
    $effect(() => {
        const _watch = coordinatorNameEn + coordinatorNameAr + coordinatorPhone +
                       coordinatorWhatsapp + coordinatorEmail + coordinatorMobile;
        const timer = setTimeout(() => {
            listingStore.updateFormData({
                coordinatorNameEn, coordinatorNameAr, coordinatorPhone,
                coordinatorWhatsapp, coordinatorEmail, coordinatorMobile,
            });
        }, 300);
        return () => clearTimeout(timer);
    });

    onMount(() => {
        const unregister = wizard.registerSubmitHandler(async () => {
            const hasNameEn = coordinatorNameEn.trim().length > 0;
            const hasNameAr = coordinatorNameAr.trim().length > 0;
            const hasPhone = coordinatorPhone.trim().length > 0;
            const hasWhatsapp = coordinatorWhatsapp.trim().length > 0;
            const hasEmail = coordinatorEmail.trim().length > 0 && /.+@.+\..+/.test(coordinatorEmail);

            if (!hasNameEn || !hasNameAr || !hasPhone || !hasWhatsapp || !hasEmail) {
                listingStore.setError(
                    i18n.locale === "ar"
                        ? "يرجى ملء جميع الحقول المطلوبة بشكل صحيح (الاسم، الهاتف، واتساب، والبريد الإلكتروني)."
                        : "Please fill all required fields correctly (Name, Phone, WhatsApp, and Email)."
                );
                return;
            }

            // Sync state immediately before API call
            listingStore.updateFormData({
                coordinatorNameEn,
                coordinatorNameAr,
                coordinatorPhone,
                coordinatorWhatsapp,
                coordinatorEmail,
                coordinatorMobile,
            });

            // Dirty check bypass
            const isDirty = listingStore.isStepDirty(6, $listingStore);
            if (!isDirty) {
                listingStore.setHighestStep(6);
                goto(`${$page.url.pathname.split("/step-")[0]}/step-7`);
                return;
            }

            wizard.setSubmitting(true);
            listingStore.setError("");

            try {
                const url = getApiUrl(`/api/v1/vendor/products/${$listingStore.productId}`);
                const payload = {
                    version: $listingStore.version,
                    coordinatorNameAr: coordinatorNameAr.trim(),
                    coordinatorNameEn: coordinatorNameEn.trim(),
                    coordinatorPhone: coordinatorPhone.trim(),
                    coordinatorWhatsapp: coordinatorWhatsapp.trim(),
                    coordinatorEmail: coordinatorEmail.trim(),
                    coordinatorMobile: coordinatorMobile.trim() || null,
                };

                const res = await wizardFetch(url, {
                    method: "PUT",
                    headers: {
                        "Content-Type": "application/json",
                        Authorization: `Bearer ${data.sessionToken}`,
                        "X-Trace-ID": listingStore.getTraceId(),
                    },
                    body: JSON.stringify(payload),
                });

                if (!res.ok) {
                    const err = await res.json().catch(() => ({}));
                    throw new Error(err.message || err.error || `Server error ${res.status}`);
                }

                const responseData = await res.json();
                if (responseData.product?.version) {
                    listingStore.setVersion(responseData.product.version);
                }

                listingStore.commitStepSave(6);
                listingStore.setHighestStep(6);
                goto(`${$page.url.pathname.split("/step-")[0]}/step-7`);
            } catch (err: any) {
                listingStore.setError(
                    err.message || "Failed to save coordinator details.",
                );
            } finally {
                wizard.setSubmitting(false);
            }
        });
        return unregister;
    });
</script>

<div class="step-pane fade-in">
    <ListingSummaryReflection />
    <div class="step-heading">
        <Phone class="step-icon" size={28} />
        <div>
            <h1>
                {i18n.locale === "ar"
                    ? "معلومات الاتصال بالمنسق"
                    : "Coordinator Contact Info"}
            </h1>
            <p>
                {i18n.locale === "ar"
                    ? "الشخص الذي يتواصل معه العرسان لتقديم الاستفسارات. يظهر هذا في إعلانك."
                    : "The person couples reach out to when making an inquiry. This appears on your listing."}
            </p>
        </div>
    </div>

    {#if $listingStore.submitError}
        <div class="error-banner">{$listingStore.submitError}</div>
    {/if}

    <div class="form-card">
        <div class="form-row">
            <div class="form-group">
                <label for="coord-name-en">
                    {i18n.t("listingsWizard.coordinatorNameEn") || "Coordinator Name (English)"}
                    <span class="required">*</span>
                </label>
                <input
                    id="coord-name-en"
                    type="text"
                    bind:value={coordinatorNameEn}
                    placeholder="e.g. Khalid Al-Shammari"
                    maxlength={100}
                />
            </div>
            <div class="form-group">
                <label for="coord-name-ar">
                    {i18n.t("listingsWizard.coordinatorNameAr") || "Coordinator Name (Arabic)"}
                    <span class="required">*</span>
                </label>
                <input
                    id="coord-name-ar"
                    type="text"
                    bind:value={coordinatorNameAr}
                    placeholder="مثال: خالد الشمري"
                    maxlength={100}
                    dir="rtl"
                    class="rtl-input"
                />
            </div>
        </div>
        <div class="form-row">
            <div class="form-group">
                <label for="coord-phone">
                    {i18n.t("listingsWizard.coordinatorPhone") || "Coordinator Phone"}
                    <span class="required">*</span>
                </label>
                <input
                    id="coord-phone"
                    type="tel"
                    bind:value={coordinatorPhone}
                    placeholder="+966 5X XXX XXXX"
                    maxlength={20}
                />
            </div>
            <div class="form-group">
                <label for="coord-whatsapp">
                    {i18n.t("listingsWizard.coordinatorWhatsapp") || "Coordinator WhatsApp"}
                    <span class="required">*</span>
                </label>
                <input
                    id="coord-whatsapp"
                    type="tel"
                    bind:value={coordinatorWhatsapp}
                    placeholder="+966 5X XXX XXXX"
                    maxlength={20}
                />
            </div>
        </div>
        <div class="form-row">
            <div class="form-group">
                <label for="coord-email">
                    {i18n.locale === "ar" ? "البريد الإلكتروني" : "Email Address"}
                    <span class="required">*</span>
                </label>
                <input
                    id="coord-email"
                    type="email"
                    bind:value={coordinatorEmail}
                    placeholder="e.g. name@example.com"
                    maxlength={255}
                />
            </div>
            <div class="form-group">
                <label for="coord-mobile">
                    {i18n.locale === "ar" ? "رقم الجوال الإضافي" : "Additional Mobile Number"}
                    <span class="hint">({i18n.locale === "ar" ? "اختياري" : "Optional"})</span>
                </label>
                <input
                    id="coord-mobile"
                    type="tel"
                    bind:value={coordinatorMobile}
                    placeholder="+966 5X XXX XXXX"
                    maxlength={20}
                />
            </div>
        </div>
    </div>
</div>
