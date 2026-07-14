<script lang="ts">
    import { getI18n } from "$lib/i18n/i18n.svelte";
    import { listingStore } from "$lib/stores/listingStore";
    import { GENDER_SECTIONS } from "$lib/constants/wizard";
    import { CATEGORY_GROUPS } from "$lib/constants/wizard";
    import { Check } from "lucide-svelte";
    import { goto } from "$app/navigation";
    import { page } from "$app/stores";
    import { env } from "$env/dynamic/public";
    import { checkSubscriptionQuota } from "$lib/utils/subscriptionGuard";
    import UpgradeModal from "$lib/components/UpgradeModal.svelte";

    const i18n = getI18n();
    let { data } = $props<{ data: { sessionToken: string } }>();
    const apiBase =
        env.PUBLIC_API_URL && env.PUBLIC_API_URL.includes("localhost")
            ? ""
            : env.PUBLIC_API_URL || "";

    const fd = $derived($listingStore.formData);
    const selectedCategoryMeta = $derived.by(() => {
        for (const group of CATEGORY_GROUPS) {
            const found = group.items.find(
                (i: any) => i.value === fd.selectedCategory,
            );
            if (found) return found;
        }
        return null;
    });

    const coverItem = $derived($listingStore.formData.coverItem);
    const galleryItems = $derived($listingStore.formData.galleryItems || []);
    let isUpgradeModalOpen = $state(false);

    async function submitListing() {
        if ($listingStore.isSubmitting) return;
        listingStore.setSubmitting(true);
        listingStore.setError("");

        try {
            const url = `${apiBase}/api/v1/vendor/products/${$listingStore.productId}/status`;
            const payload = {
                status: "pending_approval",
            };

            const { blocked, response: res } = await checkSubscriptionQuota(async () => {
                return await fetch(url, {
                    method: "PATCH",
                    headers: {
                        "Content-Type": "application/json",
                        Authorization: `Bearer ${data.sessionToken}`,
                    },
                    body: JSON.stringify(payload),
                });
            });

            if (blocked) {
                isUpgradeModalOpen = true;
                listingStore.setError("Subscription quota limit reached.");
                listingStore.setSubmitting(false);
                return;
            }

            if (!res.ok) {
                const err = await res.json().catch(() => ({}));
                throw new Error(err.error || `Server error ${res.status}`);
            }

            const finalProductId = $listingStore.productId;
            
            // Wait for navigation to complete FIRST, so the wizard layout unmounts
            await goto('/dashboard/products');
            
            // Now safe to clear store without triggering layout's $effect guard
            listingStore.reset();
        } catch (err: any) {
            listingStore.setError(err.message || "Failed to submit listing.");
        } finally {
            listingStore.setSubmitting(false);
        }
    }
</script>

<div class="step-pane fade-in">
    <div class="step-heading">
        <div class="step-icon-text">🚀</div>
        <div>
            <h1>
                {i18n.locale === "ar"
                    ? "إرسال للمراجعة والتفعيل"
                    : "Submit for Approval"}
            </h1>
            <p>
                {i18n.locale === "ar"
                    ? "سيتم مراجعة قائمتك من قبل فريقنا في غضون 24 ساعة. ستتلقى إشعاراً بمجرد تفعيلها."
                    : "Your listing will be reviewed by our team within 24 hours. You'll receive a notification once it's approved."}
            </p>
        </div>
    </div>

    {#if $listingStore.submitError}
        <div class="error-banner">
            ⚠️ {$listingStore.submitError}
        </div>
    {/if}

    <div class="submit-summary form-card">
        <div class="summary-row">
            <strong>{i18n.locale === "ar" ? "الفئة:" : "Category:"}</strong>
            {i18n.locale === "ar"
                ? selectedCategoryMeta?.ar
                : selectedCategoryMeta?.en}
        </div>
        <div class="summary-row">
            <strong
                >{i18n.locale === "ar"
                    ? "العنوان الإنجليزي:"
                    : "Title EN:"}</strong
            >
            {fd.titleEn || "—"}
        </div>
        <div class="summary-row">
            <strong
                >{i18n.locale === "ar"
                    ? "العنوان العربي:"
                    : "Title AR:"}</strong
            >
            {fd.titleAr || "—"}
        </div>
        <div class="summary-row">
            <strong>{i18n.locale === "ar" ? "السعر:" : "Price:"}</strong>
            {fd.priceOnInquiry
                ? i18n.locale === "ar"
                    ? "عند الطلب"
                    : "On Inquiry"
                : `SAR ${fd.basePriceSar}`}
        </div>
        <div class="summary-row">
            <strong
                >{i18n.locale === "ar"
                    ? "تصنيف الجنسين:"
                    : "Gender Setup:"}</strong
            >
            {GENDER_SECTIONS.find((g) => g.value === fd.genderSection)?.[
                i18n.locale === "ar" ? "ar" : "en"
            ] || "—"}
        </div>
        <div class="summary-row">
            <strong>{i18n.locale === "ar" ? "المعرض:" : "Gallery:"}</strong>
            {#if i18n.locale === "ar"}
                {(coverItem ? 1 : 0) + galleryItems.length}
                {(coverItem ? 1 : 0) + galleryItems.length === 1
                    ? "ملف جاهز"
                    : "ملفات جاهزة"} للرفع
            {:else}
                {(coverItem ? 1 : 0) + galleryItems.length} media item(s) ready
            {/if}
        </div>
    </div>

    <button
        type="button"
        class="submit-btn"
        onclick={submitListing}
        disabled={$listingStore.isSubmitting}
    >
        {#if $listingStore.isSubmitting}
            <span class="spinner">⏳</span>
            {i18n.locale === "ar" ? "جاري الإرسال..." : "Submitting…"}
        {:else}
            <Check size={20} />
            {i18n.locale === "ar"
                ? "إرسال الإعلان للمراجعة"
                : "Submit Listing for Approval"}
        {/if}
    </button>
</div>

<UpgradeModal isOpen={isUpgradeModalOpen} onClose={() => isUpgradeModalOpen = false} />

<style>
    .submit-summary {
        margin-bottom: 24px;
    }
    .summary-row {
        display: flex;
        justify-content: space-between;
        padding: 12px 0;
        border-bottom: 1px solid var(--border);
        font-size: 0.95rem;
        color: var(--text);
    }
    .summary-row:last-child {
        border-bottom: none;
    }
    .summary-row strong {
        color: var(--text-muted);
        font-weight: 600;
    }
    .submit-btn {
        width: 100%;
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 10px;
        background: var(--primary);
        color: white;
        border: none;
        padding: 16px;
        font-size: 1.1rem;
        font-weight: 700;
        border-radius: var(--radius-lg);
        cursor: pointer;
        transition: all 0.2s;
        box-shadow: 0 4px 14px rgba(108, 63, 160, 0.3);
    }
    .submit-btn:hover:not(:disabled) {
        background: var(--primary-dark);
        transform: translateY(-2px);
        box-shadow: 0 6px 20px rgba(108, 63, 160, 0.4);
    }
    .submit-btn:disabled {
        opacity: 0.7;
        cursor: not-allowed;
    }
    .spinner {
        display: inline-block;
        animation: spin 1s linear infinite;
    }
    @keyframes spin {
        from {
            transform: rotate(0deg);
        }
        to {
            transform: rotate(360deg);
        }
    }
</style>
