<script lang="ts">
    import { enhance } from "$app/forms";
    import { getI18n } from "$lib/i18n/i18n.svelte";
    import { resolveMediaUrl } from "$lib/shared/utils/media";
    import type { ActionData, PageData } from "./$types";
    import RichTextEditor from "$lib/components/RichTextEditor.svelte";
    import MediaUpload from "$lib/components/MediaUpload.svelte";

    let { data, form } = $props<{ data: PageData; form: ActionData }>();

    const i18n = getI18n();
    // svelte-ignore state_referenced_locally
    const promo = data.promotion;

    // Helper to format ISO date to datetime-local input value
    function toDatetimeLocal(iso: string) {
        if (!iso) return "";
        const d = new Date(iso);
        const pad = (n: number) => n.toString().padStart(2, "0");
        return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
    }

    // ─── Form State (pre-filled from existing promotion) ──────────────
    let titleEn = $state(promo.title_en || "");
    let titleAr = $state(promo.title_ar || "");
    let descriptionEn = $state(promo.description_en || "");
    let descriptionAr = $state(promo.description_ar || "");
    let promoType = $state(promo.promo_type || "discount"); // "discount" or "benefit"
    let discountType = $state(promo.discount_type || "percentage"); // "percentage" or "fixed_amount"
    let discountPercentage = $state<number | null>(promo.discount_percentage);
    let discountFixedAmount = $state<number | null>(promo.discount_fixed_amount ? Number(promo.discount_fixed_amount) : null);
    let benefitDescriptionEn = $state(promo.benefit_description_en || "");
    let benefitDescriptionAr = $state(promo.benefit_description_ar || "");
    let useListingCoverImage = $state(promo.use_listing_cover_image !== undefined ? promo.use_listing_cover_image : true);
    let badgeTextEn = $state(promo.badge_text_en || "");
    let badgeTextAr = $state(promo.badge_text_ar || "");
    let bannerImageUrl = $state(promo.custom_banner_image_url || "");
    let bannerFile = $state<File | null>(null);
    let bannerPreview = $state<string | null>(promo.custom_banner_image_url || null);
    let startAt = $state(toDatetimeLocal(promo.start_at));
    let endAt = $state(toDatetimeLocal(promo.end_at));

    // Listing selector (pre-filled with existing target)
    let selectedListingId = $state<string | null>(promo.listing_id || null);
    let listingSearch = $state("");
    let categoryFilter = $state("all");
    let isSubmitting = $state(false);

    // Derive available categories
    let categories = $derived(() => {
        const cats = new Set<string>();
        data.products.forEach((p: any) => { if (p.productCategory) cats.add(p.productCategory); });
        return Array.from(cats).sort();
    });

    // Filtered listings
    let filteredProducts = $derived.by(() => {
        let result = data.products;
        if (categoryFilter !== "all") {
            result = result.filter((p: any) => p.productCategory === categoryFilter);
        }
        if (listingSearch.trim()) {
            const q = listingSearch.trim().toLowerCase();
            result = result.filter((p: any) =>
                (p.titleEn || "").toLowerCase().includes(q) ||
                (p.titleAr || "").toLowerCase().includes(q)
            );
        }
        return result;
    });

    function selectListing(id: string) {
        selectedListingId = id;
    }

    function isSelected(id: string) {
        return selectedListingId === id;
    }

    function formatCategoryLabel(cat: string) {
        return cat.replace(/-/g, " ").replace(/\b\w/g, (c) => c.toUpperCase());
    }

    function getStatusLabel(status: string) {
        const arMap: Record<string, string> = {
            Active: "نشط الآن", Pending: "قيد المراجعة", Scheduled: "مجدول",
            Paused: "موقوف مؤقتاً", Rejected: "مرفوض", Cancelled: "ملغي", Expired: "منتهي الصلاحية",
        };
        return i18n.locale === "ar" ? (arMap[status] ?? status) : status;
    }

    function getWordCount(html: string): number {
        if (!html) return 0;
        const text = html.replace(/<[^>]*>/g, ' ');
        return text.trim().split(/\s+/).filter(w => w.length > 0).length;
    }

    function handleFormEnhance({ cancel }: any) {
        const descArWords = getWordCount(descriptionAr);
        const descEnWords = getWordCount(descriptionEn);

        if (descArWords > 2000 || descEnWords > 2000) {
            alert(i18n.locale === "ar" 
                ? "يجب ألا يتجاوز الوصف 2000 كلمة." 
                : "Description must not exceed 2000 words.");
            cancel();
            return;
        }

        isSubmitting = true;
        return async ({ result, update }: any) => {
            isSubmitting = false;
            await update();
        };
    }

    function handleBannerFileChange(e: Event) {
        const input = e.target as HTMLInputElement;
        const file = input.files?.[0] || null;
        bannerFile = file;
        if (file) {
            bannerPreview = URL.createObjectURL(file);
            bannerImageUrl = ''; // Clear URL if new file chosen
        } else {
            bannerPreview = promo.custom_banner_image_url || null;
        }
    }
</script>

<svelte:head>
    <title>{i18n.locale === "ar" ? "تعديل العرض الترويجي" : "Edit Promotion"} | {i18n.t.common.appName}</title>
</svelte:head>

<div class="edit-page" dir={i18n.isRtl ? "rtl" : "ltr"}>
    <!-- Back link -->
    <a href="/dashboard/offers" class="back-link">
        <span class="back-arrow">{i18n.isRtl ? "→" : "←"}</span>
        {i18n.locale === "ar" ? "العودة للعروض" : "Back to Promotions"}
    </a>

    <div class="page-title-bar">
        <div class="title-row">
            <h1>{i18n.locale === "ar" ? "تعديل العرض الترويجي" : "Edit Promotion"}</h1>
            <span class="current-status-badge status-{promo.derived_status?.toLowerCase()}">
                {getStatusLabel(promo.derived_status)}
            </span>
        </div>

        <!-- Reset-to-pending warning -->
        <div class="reset-warning">
            <span>ℹ️</span>
            <p>
                {i18n.locale === "ar"
                    ? "أي تعديل سيعيد العرض لحالة 'قيد المراجعة' ويتطلب موافقة المسؤول مرة أخرى."
                    : "Any changes will reset the promotion to 'Pending' status and require admin re-approval."}
            </p>
        </div>
    </div>

    <!-- Rejection reason (if applicable) -->
    {#if promo.derived_status === "Rejected" && promo.rejection_reason}
        <div class="rejection-callout">
            <span class="rejection-icon">⚠️</span>
            <div>
                <strong>{i18n.locale === "ar" ? "سبب الرفض السابق:" : "Previous Rejection Reason:"}</strong>
                <p>{promo.rejection_reason}</p>
            </div>
        </div>
    {/if}

    <!-- Error banner -->
    {#if form?.error}
        <div class="error-banner">
            <span>⚠️</span>
            <p>{i18n.locale === "ar" ? (form.errorAr ?? form.error) : form.error}</p>
        </div>
    {/if}

    <form method="POST" action="?/update" use:enhance={handleFormEnhance} enctype="multipart/form-data">
        <!-- Hidden field for listing_id -->
        <input type="hidden" name="listing_id" value={selectedListingId || ""} />

        <!-- ─── SECTION 1: Target Listing ────────────────────────── -->
        <section class="form-section">
            <div class="section-header">
                <h2>🎯 {i18n.locale === "ar" ? "المنتج المستهدف" : "Target Listing"}</h2>
                {#if selectedListingId}
                    <span class="selection-count">
                        {i18n.locale === "ar" ? "تم التحديد" : "1 selected"}
                    </span>
                {/if}
            </div>

            <div class="listing-selector-toolbar">
                <input
                    type="text"
                    class="selector-search"
                    placeholder={i18n.locale === "ar" ? "ابحث عن منتج..." : "Search listings..."}
                    bind:value={listingSearch}
                />
                <select class="selector-category" bind:value={categoryFilter}>
                    <option value="all">{i18n.locale === "ar" ? "كل الفئات" : "All Categories"}</option>
                    {#each categories() as cat}
                        <option value={cat}>{formatCategoryLabel(cat)}</option>
                    {/each}
                </select>
            </div>

            <div class="listing-selector-grid">
                {#each filteredProducts as product (product.id)}
                    <button
                        type="button"
                        class="listing-selector-item"
                        class:selected={isSelected(product.id)}
                        onclick={() => selectListing(product.id)}
                    >
                        <div class="selector-checkbox">
                            {#if isSelected(product.id)}
                                <span class="check-mark" style="color: var(--primary-color, #ff477e); font-size: 1.2rem; line-height: 1;">●</span>
                            {:else}
                                <span class="check-empty"></span>
                            {/if}
                        </div>
                        <div class="selector-info">
                            <strong>{i18n.locale === "ar" ? (product.titleAr || product.titleEn) : (product.titleEn || product.titleAr)}</strong>
                            <span class="selector-cat-tag">{formatCategoryLabel(product.productCategory || "")}</span>
                        </div>
                        {#if product.pricing?.basePriceSar}
                            <span class="selector-price">{Number(product.pricing.basePriceSar).toLocaleString()} SAR</span>
                        {/if}
                    </button>
                {/each}
                {#if filteredProducts.length === 0}
                    <p class="no-matches">{i18n.locale === "ar" ? "لا توجد نتائج" : "No matches found"}</p>
                {/if}
            </div>
        </section>

        <!-- ─── SECTION 2: Promotion Details ──────────────────────── -->
        <section class="form-section">
            <h2>📝 {i18n.locale === "ar" ? "تفاصيل العرض" : "Promotion Details"}</h2>

            <div class="bilingual-row">
                <div class="form-field">
                    <label for="title_ar">{i18n.locale === "ar" ? "العنوان (عربي)" : "Title (Arabic)"} *</label>
                    <input type="text" id="title_ar" name="title_ar" bind:value={titleAr} required dir="rtl" maxlength="255" />
                    <span class="char-counter" class:near-limit={titleAr.length > 230} class:at-limit={titleAr.length >= 255}>
                        {titleAr.length}/255
                    </span>
                </div>
                <div class="form-field">
                    <label for="title_en">{i18n.locale === "ar" ? "العنوان (إنجليزي)" : "Title (English)"} *</label>
                    <input type="text" id="title_en" name="title_en" bind:value={titleEn} required dir="ltr" maxlength="255" />
                    <span class="char-counter" class:near-limit={titleEn.length > 230} class:at-limit={titleEn.length >= 255}>
                        {titleEn.length}/255
                    </span>
                </div>
            </div>

            <div class="bilingual-row">
                <div class="form-field">
                    <label for="description_ar">{i18n.locale === "ar" ? "الوصف (عربي)" : "Description (Arabic)"}</label>
                    <RichTextEditor
                        id="description_ar" name="description_ar"
                        placeholder={i18n.locale === "ar" ? "وصف تفصيلي للعرض..." : "Detailed description..."}
                        bind:value={descriptionAr} dir="rtl"
                    />
                </div>
                <div class="form-field">
                    <label for="description_en">{i18n.locale === "ar" ? "الوصف (إنجليزي)" : "Description (English)"}</label>
                    <RichTextEditor
                        id="description_en" name="description_en"
                        placeholder="Detailed description of the promotion..."
                        bind:value={descriptionEn} dir="ltr"
                    />
                </div>
            </div>
        </section>

        <!-- ─── SECTION 3: Offer Type & Details ────────────────────── -->
        <section class="form-section">
            <h2>📊 {i18n.locale === "ar" ? "تفاصيل نوع العرض والمواعيد" : "Offer Type, Details & Schedule"}</h2>

            <div class="bilingual-row">
                <div class="form-field">
                    <label for="promo_type">{i18n.locale === "ar" ? "نوع العرض الترويجي" : "Promotion Type"} *</label>
                    <select id="promo_type" name="promo_type" bind:value={promoType} style="padding: 10px; border: 1px solid var(--border-color, #e2e8f0); border-radius: 6px; width: 100%; background: white;" required>
                        <option value="discount">{i18n.locale === "ar" ? "خصم مالي أو نسبة" : "Financial Discount"}</option>
                        <option value="benefit">{i18n.locale === "ar" ? "قيمة إضافية / هدية مجانية" : "Added-Value / Gift Benefit"}</option>
                    </select>
                </div>

                {#if promoType === "discount"}
                    <div class="form-field">
                        <label for="discount_type">{i18n.locale === "ar" ? "نوع الخصم" : "Discount Type"} *</label>
                        <select id="discount_type" name="discount_type" bind:value={discountType} style="padding: 10px; border: 1px solid var(--border-color, #e2e8f0); border-radius: 6px; width: 100%; background: white;" required>
                            <option value="percentage">{i18n.locale === "ar" ? "نسبة مئوية (%)" : "Percentage (%)"}</option>
                            <option value="fixed_amount">{i18n.locale === "ar" ? "مبلغ ثابت (ر.س)" : "Fixed Amount (SAR)"}</option>
                        </select>
                    </div>
                {/if}
            </div>

            {#if promoType === "discount"}
                <div class="bilingual-row">
                    {#if discountType === "percentage"}
                        <div class="form-field">
                            <label for="discount_percentage">{i18n.locale === "ar" ? "نسبة الخصم (%)" : "Discount Percentage (%)"} *</label>
                            <input
                                type="number" id="discount_percentage" name="discount_percentage"
                                min="5" max="90" bind:value={discountPercentage} required
                            />
                            <span class="field-hint">{i18n.locale === "ar" ? "بين ٥٪ و ٩٠٪" : "Between 5% and 90%"}</span>
                        </div>
                    {:else}
                        <div class="form-field">
                            <label for="discount_fixed_amount">{i18n.locale === "ar" ? "مبلغ الخصم (ر.س)" : "Discount Amount (SAR)"} *</label>
                            <input
                                type="number" id="discount_fixed_amount" name="discount_fixed_amount"
                                min="1" step="any" bind:value={discountFixedAmount} required
                            />
                            <span class="field-hint">{i18n.locale === "ar" ? "أدخل قيمة الخصم بالريال" : "Enter discount value in SAR"}</span>
                        </div>
                    {/if}
                </div>
            {:else}
                <div class="bilingual-row">
                    <div class="form-field">
                        <label for="benefit_description_ar">{i18n.locale === "ar" ? "وصف الميزة المضافة (عربي) *" : "Benefit Description (Arabic) *"}</label>
                        <textarea
                            id="benefit_description_ar" name="benefit_description_ar" rows="3"
                            placeholder={i18n.locale === "ar" ? "مثال: كتاب زوار مجاني مطبوع مخصص للعروسين..." : "e.g. كتاب زوار مجاني مطبوع مخصص للعروسين..."}
                            bind:value={benefitDescriptionAr} required dir="rtl" maxlength="255"
                        ></textarea>
                        <span class="char-counter" class:near-limit={(benefitDescriptionAr?.length ?? 0) > 230} class:at-limit={(benefitDescriptionAr?.length ?? 0) >= 255}>
                            {benefitDescriptionAr?.length ?? 0}/255
                        </span>
                    </div>
                    <div class="form-field">
                        <label for="benefit_description_en">{i18n.locale === "ar" ? "وصف الميزة المضافة (إنجليزي) *" : "Benefit Description (English) *"}</label>
                        <textarea
                            id="benefit_description_en" name="benefit_description_en" rows="3"
                            placeholder="e.g. Free custom printed guestbook with wedding cover..."
                            bind:value={benefitDescriptionEn} required dir="ltr" maxlength="255"
                        ></textarea>
                        <span class="char-counter" class:near-limit={(benefitDescriptionEn?.length ?? 0) > 230} class:at-limit={(benefitDescriptionEn?.length ?? 0) >= 255}>
                            {benefitDescriptionEn?.length ?? 0}/255
                        </span>
                    </div>
                </div>
            {/if}

            <div class="bilingual-row">
                <div class="form-field">
                    <label for="start_at">{i18n.locale === "ar" ? "تاريخ البداية" : "Start Date"} *</label>
                    <input type="datetime-local" id="start_at" name="start_at" bind:value={startAt} required />
                </div>
                <div class="form-field">
                    <label for="end_at">{i18n.locale === "ar" ? "تاريخ النهاية" : "End Date"} *</label>
                    <input type="datetime-local" id="end_at" name="end_at" bind:value={endAt} required />
                </div>
            </div>
        </section>

        <!-- ─── SECTION 4: Customization & Banners ───────────────── -->
        <section class="form-section">
            <h2>🎨 {i18n.locale === "ar" ? "تصميم البانر والخيارات الإضافية" : "Banner Design & Extra Options"}</h2>

            <div class="bilingual-row">
                <div class="form-field">
                    <label for="badge_text_ar">{i18n.locale === "ar" ? "نص الشارة (عربي)" : "Badge Text (Arabic)"}</label>
                    <input type="text" id="badge_text_ar" name="badge_text_ar"
                        placeholder={i18n.locale === "ar" ? "مثال: عرض محدود" : "e.g. عرض محدود"}
                        bind:value={badgeTextAr} dir="rtl" maxlength="50"
                    />
                </div>
                <div class="form-field">
                    <label for="badge_text_en">{i18n.locale === "ar" ? "نص الشارة (إنجليزي)" : "Badge Text (English)"}</label>
                    <input type="text" id="badge_text_en" name="badge_text_en"
                        placeholder="e.g. Limited Deal" bind:value={badgeTextEn} dir="ltr" maxlength="50"
                    />
                </div>
            </div>

            <div class="form-field">
                <label for="use_listing_cover_image">{i18n.locale === "ar" ? "مصدر صورة البانر" : "Banner Image Source"} *</label>
                <select id="use_listing_cover_image" name="use_listing_cover_image" bind:value={useListingCoverImage} style="padding: 10px; border: 1px solid var(--border-color, #e2e8f0); border-radius: 6px; width: 100%; background: white;" required>
                    <option value={true}>{i18n.locale === "ar" ? "استخدام الغلاف الافتراضي للمنتج" : "Use default listing cover photo"}</option>
                    <option value={false}>{i18n.locale === "ar" ? "رفع صورة بانر مخصصة للعرض" : "Upload custom banner photo"}</option>
                </select>
            </div>

            {#if !useListingCoverImage}
                <div class="form-field">
                    <label for="banner_file">{i18n.locale === "ar" ? "صورة البانر المخصصة" : "Custom Banner Image"}</label>
                    <div class="banner-upload-area">
                        <MediaUpload
                            token={data.token}
                            lang={i18n.locale}
                            allowedTypes={['image/jpeg', 'image/png', 'image/webp']}
                            maxSizeBytes={10 * 1024 * 1024}
                            onSuccess={(media) => {
                                bannerImageUrl = media.url;
                            }}
                            onFailure={(err) => {
                                bannerImageUrl = '';
                            }}
                        />
                        <input type="hidden" name="banner_image_url" value={bannerImageUrl} />
                    </div>
                </div>
            {/if}
        </section>

        <!-- ─── Submit Bar ────────────────────────────────────────── -->
        <div class="submit-bar">
            <a href="/dashboard/offers" class="btn btn-ghost">
                {i18n.t.common.cancel}
            </a>
            <button type="submit" class="btn btn-primary btn-lg" disabled={isSubmitting || !selectedListingId}>
                {#if isSubmitting}
                    ⏳ {i18n.locale === "ar" ? "جارٍ الحفظ..." : "Saving..."}
                {:else}
                    {i18n.locale === "ar" ? "حفظ وإرسال للمراجعة" : "Save & Submit for Review"}
                {/if}
            </button>
        </div>
    </form>
</div>

<style>
    .edit-page {
        max-width: 900px;
        margin: 0 auto;
        display: flex;
        flex-direction: column;
        gap: 20px;
        animation: fade-in 0.3s ease-out;
    }

    @keyframes fade-in {
        from { opacity: 0; transform: translateY(6px); }
        to   { opacity: 1; transform: translateY(0); }
    }

    .back-link {
        display: inline-flex;
        align-items: center;
        gap: 6px;
        font-size: 13px;
        font-weight: 600;
        color: var(--text-sec);
        text-decoration: none;
        transition: color 0.15s;
    }
    .back-link:hover { color: var(--color-primary, #5b21b6); }
    .back-arrow { font-size: 16px; }

    .page-title-bar { display: flex; flex-direction: column; gap: 8px; }

    .title-row {
        display: flex;
        align-items: center;
        gap: 12px;
        flex-wrap: wrap;
    }
    .title-row h1 {
        margin: 0;
        font-size: 22px;
        font-weight: 800;
        color: var(--text);
        letter-spacing: -0.3px;
    }

    .current-status-badge {
        font-size: 11px;
        font-weight: 700;
        padding: 3px 10px;
        border-radius: 12px;
    }
    .status-pending   { background: hsl(45, 90%, 93%);  color: hsl(45, 90%, 25%);  border: 1px solid hsl(45, 90%, 85%); }
    .status-rejected  { background: hsl(0, 80%, 95%);   color: hsl(0, 80%, 30%);   border: 1px solid hsl(0, 80%, 85%); }
    .status-scheduled { background: hsl(200, 80%, 93%); color: hsl(200, 80%, 25%); border: 1px solid hsl(200, 80%, 85%); }
    .status-paused    { background: hsl(30, 90%, 93%);  color: hsl(30, 90%, 25%);  border: 1px solid hsl(30, 90%, 85%); }
    .status-expired   { background: hsl(220, 10%, 93%); color: hsl(220, 10%, 35%); border: 1px solid hsl(220, 10%, 85%); }
    .status-cancelled { background: hsl(220, 10%, 93%); color: hsl(220, 10%, 35%); border: 1px solid hsl(220, 10%, 85%); }

    .reset-warning {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 10px 14px;
        background: hsl(200, 70%, 96%);
        border: 1px solid hsl(200, 70%, 88%);
        border-radius: var(--radius-sm);
        font-size: 12px;
        color: hsl(200, 60%, 30%);
    }
    .reset-warning p { margin: 0; }

    .rejection-callout {
        display: flex;
        gap: 8px;
        align-items: flex-start;
        background: hsl(0, 86%, 97%);
        border: 1px solid hsl(0, 70%, 88%);
        border-radius: var(--radius-sm);
        padding: 12px 14px;
        font-size: 12px;
    }
    .rejection-icon { font-size: 14px; flex-shrink: 0; margin-top: 1px; }
    .rejection-callout strong {
        color: hsl(0, 70%, 35%);
        font-size: 11px;
        text-transform: uppercase;
        letter-spacing: 0.3px;
    }
    .rejection-callout p { margin: 2px 0 0; color: hsl(0, 50%, 40%); line-height: 1.4; }

    .error-banner {
        display: flex;
        align-items: center;
        gap: 10px;
        padding: 12px 16px;
        background: hsl(0, 86%, 97%);
        border: 1px solid hsl(0, 70%, 88%);
        border-radius: var(--radius-sm);
        font-size: 13px;
        color: hsl(0, 70%, 35%);
    }
    .error-banner p { margin: 0; }

    /* ─── FORM SECTIONS ───────────────────────────────────────── */
    .form-section {
        background: var(--white);
        border: 1px solid var(--border);
        border-radius: var(--radius);
        padding: 24px;
        box-shadow: var(--shadow-sm);
    }
    .form-section h2 {
        margin: 0 0 16px;
        font-size: 16px;
        font-weight: 700;
        color: var(--text);
    }

    .section-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }
    .section-header h2 { margin-bottom: 0; }
    .selection-count {
        font-size: 12px;
        font-weight: 700;
        color: var(--teal);
        background: var(--bg);
        padding: 3px 10px;
        border-radius: 12px;
    }

    /* ─── LISTING SELECTOR ────────────────────────────────────── */
    .listing-selector-toolbar {
        display: flex;
        gap: 10px;
        margin: 14px 0 12px;
    }
    .selector-search {
        flex: 1;
        padding: 8px 12px;
        border: 1.5px solid var(--border);
        border-radius: 8px;
        font-size: 13px;
        background: var(--bg);
        transition: border-color 0.15s;
    }
    .selector-search:focus { border-color: var(--teal); outline: none; }
    .selector-category {
        padding: 8px 12px;
        border: 1.5px solid var(--border);
        border-radius: 8px;
        font-size: 12px;
        font-weight: 600;
        background: var(--bg);
        cursor: pointer;
    }

    .listing-selector-grid {
        display: flex;
        flex-direction: column;
        gap: 6px;
        max-height: 300px;
        overflow-y: auto;
        padding-inline-end: 4px;
    }
    .listing-selector-grid::-webkit-scrollbar { width: 4px; }
    .listing-selector-grid::-webkit-scrollbar-track { background: transparent; }
    .listing-selector-grid::-webkit-scrollbar-thumb { background: var(--border); border-radius: 4px; }

    .listing-selector-item {
        display: flex;
        align-items: center;
        gap: 12px;
        padding: 10px 14px;
        border: 1.5px solid var(--border);
        border-radius: 10px;
        background: var(--white);
        cursor: pointer;
        transition: all 0.15s ease;
        text-align: start;
        width: 100%;
    }
    .listing-selector-item:hover { border-color: var(--teal); background: var(--bg); }
    .listing-selector-item.selected {
        border-color: var(--color-primary, #5b21b6);
        background: hsl(271, 65%, 97%);
        box-shadow: 0 0 0 2px hsl(271, 65%, 92%);
    }

    .selector-checkbox { width: 22px; height: 22px; flex-shrink: 0; }
    .check-mark {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 22px;
        height: 22px;
        border-radius: 6px;
        background: var(--color-primary, #5b21b6);
        color: #fff;
        font-size: 13px;
        font-weight: 800;
    }
    .check-empty {
        display: block;
        width: 22px;
        height: 22px;
        border: 2px solid var(--border);
        border-radius: 6px;
        background: var(--white);
    }

    .selector-info {
        flex: 1;
        display: flex;
        flex-direction: column;
        gap: 2px;
        min-width: 0;
    }
    .selector-info strong {
        font-size: 13px;
        font-weight: 700;
        color: var(--text);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }
    .selector-cat-tag {
        font-size: 10px;
        text-transform: uppercase;
        letter-spacing: 0.5px;
        color: var(--text-sec);
        font-weight: 600;
    }
    .selector-price {
        font-size: 12px;
        font-weight: 700;
        color: var(--teal);
        white-space: nowrap;
    }
    .no-matches { padding: 20px; text-align: center; font-size: 13px; color: var(--text-sec); }

    /* ─── FORM FIELDS ─────────────────────────────────────────── */
    .bilingual-row { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; }
    .three-col-row { display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 16px; }

    .form-field {
        display: flex;
        flex-direction: column;
        gap: 5px;
        margin-bottom: 8px;
    }
    .form-field label {
        font-size: 12px;
        font-weight: 700;
        color: var(--text);
        text-transform: uppercase;
        letter-spacing: 0.3px;
    }
    .form-field input,
    .form-field textarea,
    .form-field select {
        padding: 10px 12px;
        border: 1.5px solid var(--border);
        border-radius: 8px;
        font-size: 13px;
        background: var(--white);
        color: var(--text);
        transition: border-color 0.15s;
        font-family: inherit;
    }
    .form-field input:focus,
    .form-field textarea:focus {
        border-color: var(--teal);
        outline: none;
        box-shadow: 0 0 0 3px hsla(174, 60%, 50%, 0.1);
    }
    .form-field textarea { resize: vertical; }
    .field-hint { font-size: 10.5px; color: var(--text-sec); }
    .char-counter {
        display: block;
        font-size: 10.5px;
        color: var(--text-sec, #94a3b8);
        text-align: right;
        margin-top: 3px;
        transition: color 0.2s;
    }
    .char-counter.near-limit { color: #d97706; font-weight: 600; }
    .char-counter.at-limit   { color: #dc2626; font-weight: 700; }


    /* ─── SUBMIT BAR ──────────────────────────────────────────── */
    .submit-bar {
        display: flex;
        justify-content: flex-end;
        gap: 12px;
        padding: 16px 0;
        border-top: 1px solid var(--border-light);
    }
    .btn-lg { padding: 10px 28px !important; font-size: 14px !important; }

    /* ─── BANNER UPLOAD ────────────────────────────────────────── */
    .banner-upload-area {
        display: flex;
        flex-direction: column;
        gap: 12px;
        position: relative;
    }
    .banner-preview {
        position: relative;
        max-width: 100%;
        border-radius: var(--radius-sm);
        overflow: hidden;
        border: 1px solid var(--border);
        box-shadow: var(--shadow-sm);
        aspect-ratio: 16 / 9;
        background: var(--bg);
        display: flex;
        align-items: center;
        justify-content: center;
    }
    .banner-preview img {
        max-width: 100%;
        max-height: 100%;
        object-fit: cover;
    }
    .remove-preview {
        position: absolute;
        top: 8px;
        right: 8px;
        background: rgba(0, 0, 0, 0.6);
        color: white;
        border: none;
        border-radius: 50%;
        width: 24px;
        height: 24px;
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: pointer;
        font-size: 12px;
        transition: background 0.15s;
    }
    .remove-preview:hover {
        background: rgba(220, 38, 38, 0.9);
    }

    /* ─── RESPONSIVE ──────────────────────────────────────────── */
    @media (max-width: 768px) {
        .bilingual-row { grid-template-columns: 1fr; }
        .three-col-row { grid-template-columns: 1fr; }
        .listing-selector-toolbar { flex-direction: column; }
    }
</style>
