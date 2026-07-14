<script lang="ts">
    let {
        metaTitleEn = $bindable(),
        metaTitleAr = $bindable(),
        metaDescriptionEn = $bindable(),
        metaDescriptionAr = $bindable(),
    } = $props<{
        metaTitleEn: string;
        metaTitleAr: string;
        metaDescriptionEn: string;
        metaDescriptionAr: string;
    }>();
</script>

<div class="seo-form-inputs">
    <div class="seo-lang-form">
        <span class="seo-lang-title">🇬🇧 English SEO Meta</span>
        <div class="form-group">
            <label for="meta_title_en">Meta Title</label>
            <input
                type="text"
                id="meta_title_en"
                bind:value={metaTitleEn}
                placeholder="Enter English Meta Title..."
            />
        </div>
        <div class="form-group">
            <label for="meta_desc_en">Meta Description</label>
            <textarea
                id="meta_desc_en"
                bind:value={metaDescriptionEn}
                placeholder="Enter English Meta Description..."
                rows="2"
            ></textarea>
        </div>
    </div>

    <div class="seo-lang-form">
        <span class="seo-lang-title">🇸🇦 Arabic SEO Meta</span>
        <div class="form-group">
            <label for="meta_title_ar">عنوان الميتا (Meta Title)</label>
            <input
                type="text"
                id="meta_title_ar"
                bind:value={metaTitleAr}
                placeholder="أدخل عنوان الميتا بالعربية..."
            />
        </div>
        <div class="form-group">
            <label for="meta_desc_ar">وصف الميتا (Meta Description)</label>
            <textarea
                id="meta_desc_ar"
                bind:value={metaDescriptionAr}
                placeholder="أدخل وصف الميتا بالعربية..."
                rows="2"
            ></textarea>
        </div>
    </div>
</div>
