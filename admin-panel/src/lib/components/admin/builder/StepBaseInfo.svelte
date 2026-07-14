<script lang="ts">
  import { t, lang } from '$lib/i18n/index.js';
  import { Sparkles } from 'lucide-svelte';
  import type { BuilderState } from '../../../features/admin/builder/builderState.svelte.js';
  import { getI18n } from "$lib/i18n/i18n.svelte";

  let { state, hideSeo = false }: { state: BuilderState, hideSeo?: boolean } = $props();
  const i18n = getI18n();
  let currentLang = $derived(i18n.locale);
  
  let slugPreview = $derived.by(() => {
      const text = state.titleEn || state.titleAr || "new-listing";
      return text
          .toLowerCase()
          .replace(/[^a-z0-9\s-]/g, "")
          .trim()
          .replace(/\s+/g, "-")
          .replace(/-+/g, "-");
  });
</script>

<div class="step-container animate-slide-in">
  <!-- Basic Title Info -->
  <div class="seo-layout-grid mb-8">
    <div class="seo-form-inputs">
      <div class="seo-lang-form">
        <span class="seo-lang-title">🇬🇧 English Details</span>
        <div class="form-group">
            <label for="title_en">Listing Title</label>
            <input
                type="text"
                id="title_en"
                bind:value={state.titleEn}
                oninput={() => state.sync()}
                placeholder="Enter English Title..."
                class="base-input"
            />
        </div>
      </div>
      <div class="seo-lang-form">
        <span class="seo-lang-title">🇸🇦 تفاصيل الإعلان العربية</span>
        <div class="form-group">
            <label for="title_ar">عنوان الإعلان</label>
            <input
                type="text"
                id="title_ar"
                bind:value={state.titleAr}
                oninput={() => state.sync()}
                placeholder="أدخل العنوان بالعربية..."
                class="base-input"
            />
        </div>
      </div>
    </div>
  </div>

  {#if !hideSeo}
    <!-- SEO Optimization Section -->
    <div class="seo-optimization-section">
      <div class="seo-section-head">
          <Sparkles size={16} class="seo-sparkles-icon" />
          <h4>
              {currentLang === "ar"
                  ? "تحسين محركات البحث (SEO)"
                  : "Search Engine Optimization (SEO)"}
          </h4>
          <span class="auto-suggest-pill"
              >{currentLang === "ar"
                  ? "اقتراح تلقائي نشط"
                  : "Auto-Suggestions Active"}</span
          >
      </div>

      <div class="seo-layout-grid">
          <div class="seo-form-inputs">
              <div class="seo-lang-form">
                  <span class="seo-lang-title"
                      >🇬🇧 English SEO Meta</span
                  >
                  <div class="form-group">
                      <label for="meta_title_en">Meta Title</label>
                      <input
                          type="text"
                          id="meta_title_en"
                          class="base-input"
                          bind:value={state.metaTitleEn}
                          oninput={() => state.sync()}
                          placeholder="Enter English Meta Title..."
                      />
                  </div>
                  <div class="form-group">
                      <label for="meta_desc_en"
                          >Meta Description</label
                      >
                      <textarea
                          id="meta_desc_en"
                          class="base-textarea"
                          bind:value={state.metaDescriptionEn}
                          oninput={() => state.sync()}
                          placeholder="Enter English Meta Description..."
                          rows="3"
                      ></textarea>
                  </div>
              </div>

              <div class="seo-lang-form">
                  <span class="seo-lang-title"
                      >🇸🇦 Arabic SEO Meta</span
                  >
                  <div class="form-group">
                      <label for="meta_title_ar"
                          >عنوان الميتا (Meta Title)</label
                      >
                      <input
                          type="text"
                          id="meta_title_ar"
                          class="base-input"
                          bind:value={state.metaTitleAr}
                          oninput={() => state.sync()}
                          placeholder="أدخل عنوان الميتا بالعربية..."
                      />
                  </div>
                  <div class="form-group">
                      <label for="meta_desc_ar"
                          >وصف الميتا (Meta Description)</label
                      >
                      <textarea
                          id="meta_desc_ar"
                          class="base-textarea"
                          bind:value={state.metaDescriptionAr}
                          oninput={() => state.sync()}
                          placeholder="أدخل وصف الميتا بالعربية..."
                          rows="3"
                      ></textarea>
                  </div>
              </div>
          </div>

          <!-- Preview Panel -->
          <div class="seo-preview-panel">
              <span class="preview-panel-title"
                  >{currentLang === "ar"
                      ? "معاينة البحث في جوجل"
                      : "Google Search Snippet Preview"}</span
              >

              <div class="google-snippet-card">
                  <div class="snippet-lang-preview">
                      <div class="snippet-meta-info">
                          <img
                              src="/favicon.webp"
                              alt=""
                              class="snippet-favicon"
                              onerror={(e) => {
                                  (
                                      e.target as HTMLElement
                                  ).style.display = "none";
                              }}
                          />
                          <span class="snippet-domain"
                              >https://zafafworld.net › listings › {slugPreview}</span
                          >
                      </div>
                      <h4 class="snippet-title">
                          {state.metaTitleEn ||
                              `${state.titleEn || "New Listing"} | ZafafWorld`}
                      </h4>
                      <p class="snippet-description">
                          {state.metaDescriptionEn ||
                              "Add listings content to see a live preview of how this listing details page will appear on search engine results..."}
                      </p>
                  </div>

                  <div class="snippet-divider"></div>

                  <div class="snippet-lang-preview ar-preview">
                      <div class="snippet-meta-info">
                          <img
                              src="/favicon.webp"
                              alt=""
                              class="snippet-favicon"
                              onerror={(e) => {
                                  (
                                      e.target as HTMLElement
                                  ).style.display = "none";
                              }}
                          />
                          <span class="snippet-domain"
                              >https://zafafworld.net › listings › {slugPreview}</span
                          >
                      </div>
                      <h4 class="snippet-title">
                          {state.metaTitleAr ||
                              `${state.titleAr || "إعلان جديد"} | زفاف وورلد`}
                      </h4>
                      <p class="snippet-description">
                          {state.metaDescriptionAr ||
                              "أضف محتوى الإعلان لمشاهدة معاينة حية لكيفية ظهور صفحة هذا الإعلان على محركات البحث وجوجل..."}
                      </p>
                  </div>
              </div>
          </div>
      </div>
  </div>
  {/if}
</div>

<style>
  .step-container {
    width: 100%;
    animation: slideIn 300ms ease forwards;
  }
  @keyframes slideIn {
    from { opacity: 0; transform: translateY(10px); }
    to { opacity: 1; transform: translateY(0); }
  }

  .base-input, .base-textarea {
    width: 100%;
    padding: 10px 14px;
    border: 1px solid #e2e8f0;
    border-radius: 8px;
    background: #fff;
    font-size: 0.95rem;
    transition: all 0.2s;
  }
  .base-input:focus, .base-textarea:focus {
    outline: none;
    border-color: #6366f1;
    box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.1);
  }

  /* SEO Styles Extracted */
  .seo-optimization-section {
    background: #f8fafc;
    border: 1px solid #e2e8f0;
    border-radius: 16px;
    padding: 24px;
  }
  .seo-section-head {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 24px;
    padding-bottom: 16px;
    border-bottom: 1px solid #e2e8f0;
  }
  .seo-sparkles-icon { color: #f59e0b; }
  .seo-section-head h4 { margin: 0; font-size: 1.1rem; font-weight: 700; color: #0f172a; }
  .auto-suggest-pill {
    background: #dcfce7;
    color: #166534;
    padding: 4px 12px;
    border-radius: 20px;
    font-size: 0.75rem;
    font-weight: 600;
    margin-left: auto;
  }

  .seo-layout-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 32px;
    align-items: start;
  }
  .seo-form-inputs {
    display: flex;
    flex-direction: column;
    gap: 24px;
  }
  .seo-lang-form {
    background: #fff;
    padding: 20px;
    border-radius: 12px;
    border: 1px solid #e2e8f0;
    box-shadow: 0 1px 2px rgba(0,0,0,0.02);
  }
  .seo-lang-title {
    display: block;
    font-weight: 700;
    color: #334155;
    margin-bottom: 16px;
    font-size: 0.95rem;
  }
  .form-group {
    margin-bottom: 16px;
  }
  .form-group:last-child { margin-bottom: 0; }
  .form-group label {
    display: block;
    font-size: 0.85rem;
    font-weight: 600;
    color: #64748b;
    margin-bottom: 6px;
  }

  .seo-preview-panel {
    background: #fff;
    border-radius: 12px;
    border: 1px solid #e2e8f0;
    padding: 24px;
    position: sticky;
    top: 24px;
  }
  .preview-panel-title {
    display: block;
    font-size: 0.85rem;
    font-weight: 700;
    color: #64748b;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: 16px;
  }
  .google-snippet-card {
    background: #fff;
    border: 1px solid #e2e8f0;
    border-radius: 8px;
    padding: 16px;
  }
  .snippet-lang-preview { margin-bottom: 16px; }
  .snippet-divider { height: 1px; background: #e2e8f0; margin: 16px 0; }
  .ar-preview { direction: rtl; text-align: right; }
  .snippet-meta-info {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 4px;
  }
  .snippet-favicon { width: 16px; height: 16px; border-radius: 50%; }
  .snippet-domain { font-size: 12px; color: #202124; }
  .snippet-title { margin: 0 0 4px 0; color: #1a0dab; font-size: 18px; font-weight: 400; line-height: 1.2; text-decoration: none; }
  .snippet-title:hover { text-decoration: underline; cursor: pointer; }
  .snippet-description { margin: 0; color: #4d5156; font-size: 14px; line-height: 1.58; display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; }

  @media (max-width: 1024px) {
    .seo-layout-grid { grid-template-columns: 1fr; }
    .seo-preview-panel { position: static; }
  }
</style>
