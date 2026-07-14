<script lang="ts">
    let {
        currentLang = "en",
        slugPreview,
        metaTitleEn,
        metaTitleAr,
        titleEn,
        titleAr,
        metaDescriptionEn,
        metaDescriptionAr,
    } = $props<{
        currentLang?: "ar" | "en";
        slugPreview: string;
        metaTitleEn: string;
        metaTitleAr: string;
        titleEn: string;
        titleAr: string;
        metaDescriptionEn: string;
        metaDescriptionAr: string;
    }>();
</script>

<div class="seo-preview-panel">
    <span class="preview-panel-title">
        {currentLang === "ar" ? "معاينة البحث في جوجل" : "Google Search Snippet Preview"}
    </span>

    <div class="google-snippet-card">
        <div class="snippet-lang-preview">
            <div class="snippet-meta-info">
                <img
                    src="/favicon.webp"
                    alt=""
                    class="snippet-favicon"
                    onerror={(e) => {
                        (e.target as HTMLElement).style.display = "none";
                    }}
                />
                <span class="snippet-domain">https://zafafworld.net › listings › {slugPreview}</span>
            </div>
            <h4 class="snippet-title">
                {metaTitleEn || `${titleEn || "New Listing"} | ZafafWorld`}
            </h4>
            <p class="snippet-description">
                {metaDescriptionEn || "Add listings content to see a live preview of how this listing details page will appear on search engine results..."}
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
                        (e.target as HTMLElement).style.display = "none";
                    }}
                />
                <span class="snippet-domain">https://zafafworld.net › listings › {slugPreview}</span>
            </div>
            <h4 class="snippet-title">
                {metaTitleAr || `${titleAr || "إعلان جديد"} | زفاف وورلد`}
            </h4>
            <p class="snippet-description">
                {metaDescriptionAr || "أضف محتوى الإعلان لمشاهدة معاينة حية لكيفية ظهور صفحة هذا الإعلان على محركات البحث وجوجل..."}
            </p>
        </div>
    </div>
</div>
