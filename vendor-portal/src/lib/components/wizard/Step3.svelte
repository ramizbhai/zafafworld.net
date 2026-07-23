<script lang="ts">
    import { getContext, onMount } from "svelte";
    import { getI18n } from "$lib/i18n/i18n.svelte";
    import { listingStore } from "$lib/stores/listingStore";
    import RichTextEditor from "$lib/components/RichTextEditor.svelte";
    import { Sparkles, Globe, FileText } from "lucide-svelte";
    import { goto } from "$app/navigation";
    import { page } from "$app/stores";
    import { getApiUrl } from "$lib/utils/api";
    import { wizardFetch } from "$lib/utils/wizardFetch";

    const i18n = getI18n();
    let { data } = $props<{ data: { sessionToken: string } }>();

    const wizard = getContext<{
        registerSubmitHandler: (handler: () => Promise<void>) => () => void;
        setCanContinue: (val: boolean) => void;
        setSubmitting: (val: boolean) => void;
    }>('wizard');

    // Strip html tags to validate minimum text character length
    function getTextLength(html: string): number {
        if (!html) return 0;
        const text = html.replace(/<[^>]*>/g, ' ');
        return text.trim().length;
    }

    let isValid = $derived(
        getTextLength($listingStore.formData.descriptionAr) >= 50 || 
        getTextLength($listingStore.formData.descriptionEn) >= 50
    );

    $effect(() => {
        wizard.setCanContinue(isValid);
    });

    onMount(() => {
        const unregister = wizard.registerSubmitHandler(async () => {
            if (
                getTextLength($listingStore.formData.descriptionAr) < 50 &&
                getTextLength($listingStore.formData.descriptionEn) < 50
            ) {
                listingStore.setError(
                    i18n.locale === "ar"
                        ? "يجب أن يكون طول أحد الوصفين (العربي أو الإنجليزي) 50 حرفاً على الأقل."
                        : "At least one description (English or Arabic) must be 50 characters or longer.",
                );
                return;
            }

            // Dirty check bypass
            const isDirty = listingStore.isStepDirty(3, $listingStore);
            if (!isDirty) {
                listingStore.setHighestStep(3);
                await goto(`${$page.url.pathname.split("/step-")[0]}/step-4`);
                return;
            }

            wizard.setSubmitting(true);
            listingStore.setError("");

            try {
                const url = getApiUrl(`/api/v1/vendor/products/${$listingStore.productId}`);
                const payload = {
                    version: $listingStore.version,
                    descriptionAr: $listingStore.formData.descriptionAr || null,
                    descriptionEn: $listingStore.formData.descriptionEn || null,
                    metaTitleAr: $listingStore.formData.metaTitleAr || null,
                    metaTitleEn: $listingStore.formData.metaTitleEn || null,
                    metaDescriptionAr: $listingStore.formData.metaDescriptionAr || null,
                    metaDescriptionEn: $listingStore.formData.metaDescriptionEn || null,
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

                listingStore.commitStepSave(3);
                listingStore.setHighestStep(3);
                await goto(`${$page.url.pathname.split("/step-")[0]}/step-4`);
            } catch (err: any) {
                listingStore.setError(
                    err.message || "Failed to save description.",
                );
            } finally {
                wizard.setSubmitting(false);
            }
        });
        return unregister;
    });
</script>

<div class="step-pane fade-in">
    <div class="step-heading">
        <div class="step-icon-text">📝</div>
        <div>
            <h1>
                {i18n.locale === "ar" ? "وصف الإعلان وتحسين محركات البحث" : "Listing Description & SEO"}
            </h1>
            <p>
                {i18n.locale === "ar"
                    ? "أكتب تفاصيل إعلانك والخدمات التي تقدمها باستخدام المحرر، وقم بإعداد الميتا الخاص بك للوصول للمزيد من الزوار."
                    : "Describe your service and what you offer using the rich text editor, and configure your Meta tags to reach more visitors."}
            </p>
        </div>
    </div>

    {#if $listingStore.submitError}
        <div class="error-banner">{$listingStore.submitError}</div>
    {/if}

    <div class="description-editors-layout">
        <!-- Arabic Editor Card -->
        <div class="editor-section-card">
            <div class="editor-card-header">
                <div class="header-title-group">
                    <Globe size={18} class="text-primary" />
                    <h3>الوصف باللغة العربية (Arabic Description) <span class="required-asterisk">*</span></h3>
                </div>
                <span class="language-badge">عربي</span>
            </div>
            <p class="editor-card-subtitle">أدخل تفاصيل خدمتك لجمهورك العربي. (50 حرفاً على الأقل في أحد الوصفين)</p>
            <RichTextEditor
                bind:value={$listingStore.formData.descriptionAr}
                dir="rtl"
                placeholder="ابدأ في كتابة تفاصيل الخدمة والمرافق هنا..."
                token={data.sessionToken}
            />
        </div>

        <!-- English Editor Card -->
        <div class="editor-section-card">
            <div class="editor-card-header">
                <div class="header-title-group">
                    <Globe size={18} class="text-primary" />
                    <h3>English Description <span class="required-asterisk">*</span></h3>
                </div>
                <span class="language-badge">EN</span>
            </div>
            <p class="editor-card-subtitle">Describe your offering for your English-speaking clients. (Min 50 characters in either editor)</p>
            <RichTextEditor
                bind:value={$listingStore.formData.descriptionEn}
                dir="ltr"
                placeholder="Start typing your service details, features, and guidelines here..."
                token={data.sessionToken}
            />
        </div>

        <!-- SEO Card -->
        <div class="editor-section-card seo-card">
            <div class="editor-card-header">
                <div class="header-title-group">
                    <Sparkles size={18} class="text-indigo-500" />
                    <h3>
                        {i18n.locale === "ar"
                            ? "تحسين محركات البحث - سيو (اختياري)"
                            : "Search Engine Optimization - SEO (Optional)"}
                    </h3>
                </div>
                <span class="seo-badge">SEO</span>
            </div>
            <p class="editor-card-subtitle">
                {i18n.locale === "ar"
                    ? "أدخل الميتا لتهيئة ظهور إعلانك في محركات بحث جوجل."
                    : "Add custom meta tags to optimize how your listing appears in Google search results."}
            </p>

            <div class="seo-inputs-grid">
                <!-- English SEO -->
                <div class="seo-group">
                    <h4 class="seo-lang-title">English Meta Tags</h4>
                    
                    <div class="form-field">
                        <label for="meta-title-en">Meta Title (EN)</label>
                        <input
                            type="text"
                            id="meta-title-en"
                            bind:value={$listingStore.formData.metaTitleEn}
                            placeholder="e.g. Premium Wedding Venue in Riyadh | Al Nour Palace"
                            maxlength={70}
                        />
                        <div class="char-count">
                            {($listingStore.formData.metaTitleEn || '').length}/70
                        </div>
                    </div>

                    <div class="form-field">
                        <label for="meta-desc-en">Meta Description (EN)</label>
                        <textarea
                            id="meta-desc-en"
                            bind:value={$listingStore.formData.metaDescriptionEn}
                            placeholder="e.g. Al Nour Palace offers premium ballroom settings, customized catering, and dual sections for weddings in Riyadh. Book your venue today..."
                            rows="3"
                            maxlength={160}
                        ></textarea>
                        <div class="char-count">
                            {($listingStore.formData.metaDescriptionEn || '').length}/160
                        </div>
                    </div>
                </div>

                <!-- Arabic SEO -->
                <div class="seo-group">
                    <h4 class="seo-lang-title">الكلمات الدلالية والميتا (العربية)</h4>
                    
                    <div class="form-field">
                        <label for="meta-title-ar">عنوان الميتا (العربية)</label>
                        <input
                            type="text"
                            id="meta-title-ar"
                            bind:value={$listingStore.formData.metaTitleAr}
                            placeholder="مثال: قاعة أفراح راقية بالرياض | قصر النور للمناسبات"
                            dir="rtl"
                            maxlength={70}
                        />
                        <div class="char-count">
                            {($listingStore.formData.metaTitleAr || '').length}/70
                        </div>
                    </div>

                    <div class="form-field">
                        <label for="meta-desc-ar">وصف الميتا (العربية)</label>
                        <textarea
                            id="meta-desc-ar"
                            bind:value={$listingStore.formData.metaDescriptionAr}
                            placeholder="مثال: يقدم قصر النور أرقى صالات الأفراح وخدمات البوفيه المفتوح المتميزة مع أقسام منفصلة للرجال والنساء في الرياض. احجز موعدك الآن..."
                            dir="rtl"
                            rows="3"
                            maxlength={160}
                        ></textarea>
                        <div class="char-count">
                            {($listingStore.formData.metaDescriptionAr || '').length}/160
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>
</div>

<style>
    .description-editors-layout {
        display: flex;
        flex-direction: column;
        gap: 24px;
    }

    .editor-section-card {
        background: white;
        border: 1px solid var(--border, #e5e7eb);
        border-radius: 16px;
        padding: 24px;
        box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
    }

    .editor-card-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        margin-bottom: 6px;
    }

    .header-title-group {
        display: flex;
        align-items: center;
        gap: 8px;
    }

    .header-title-group h3 {
        font-size: 1.05rem;
        font-weight: 700;
        color: var(--text, #1f2937);
        margin: 0;
    }

    .required-asterisk {
        color: #ef4444;
        margin-left: 2px;
    }

    .language-badge, .seo-badge {
        font-size: 0.7rem;
        font-weight: 700;
        padding: 4px 8px;
        border-radius: 6px;
        text-transform: uppercase;
        letter-spacing: 0.05em;
    }

    .language-badge {
        background: #f3f4f6;
        color: #4b5563;
    }

    .seo-badge {
        background: #e0e7ff;
        color: #4f46e5;
    }

    .editor-card-subtitle {
        font-size: 0.85rem;
        color: var(--text-muted, #6b7280);
        margin: 0 0 16px 0;
        line-height: 1.4;
    }

    /* SEO section grid styling */
    .seo-card {
        border-left: 4px solid #6366f1;
    }

    .seo-inputs-grid {
        display: grid;
        grid-template-columns: repeat(2, 1fr);
        gap: 24px;
        margin-top: 16px;
    }

    .seo-group {
        display: flex;
        flex-direction: column;
        gap: 16px;
    }

    .seo-lang-title {
        font-size: 0.9rem;
        font-weight: 700;
        color: #4b5563;
        margin: 0 0 4px 0;
        border-bottom: 1px dashed #e5e7eb;
        padding-bottom: 6px;
    }

    .form-field {
        display: flex;
        flex-direction: column;
        gap: 6px;
    }

    .form-field label {
        font-size: 0.85rem;
        font-weight: 600;
        color: #374151;
    }

    .form-field input, .form-field textarea {
        padding: 10px 12px;
        border: 1px solid #d1d5db;
        border-radius: 8px;
        font-size: 0.9rem;
        color: #111827;
        outline: none;
        transition: border-color 0.15s;
    }

    .form-field input:focus, .form-field textarea:focus {
        border-color: #6366f1;
    }

    .char-count {
        align-self: flex-end;
        font-size: 0.75rem;
        color: #9ca3af;
        margin-top: 2px;
    }

    @media (max-width: 768px) {
        .seo-inputs-grid {
            grid-template-columns: 1fr;
            gap: 20px;
        }
        .editor-section-card {
            padding: 16px;
        }
    }
</style>
