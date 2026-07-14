<script lang="ts">
    import { getI18n } from "$lib/i18n/i18n.svelte";
    import type { GalleryItem } from "$lib/stores/listingStore";

    let { item }: { item: GalleryItem } = $props();
    const i18n = getI18n();
</script>

{#if item.status === "uploading"}
    <div class="text-white text-xs font-semibold mb-1">
        {i18n.locale === "ar" ? "جاري الرفع..." : "Uploading..."}
    </div>
    <div class="w-2/3 bg-white/30 h-2 rounded-full overflow-hidden mb-1">
        <div
            class="bg-indigo-500 h-full transition-all duration-150"
            style="width: {item.progress || 0}%"
        ></div>
    </div>
    <div class="text-white text-[0.7rem] flex gap-2">
        <span>{item.progress || 0}%</span>
        <span>•</span>
        <span>{item.speed || "0 KB/s"}</span>
    </div>
{:else if item.status === "processing"}
    <div class="text-white text-xs font-semibold mb-1 animate-pulse">
        {i18n.locale === "ar" ? "جاري المعالجة..." : "Processing..."}
    </div>
{:else if item.status === "queued"}
    <div class="text-white text-xs font-semibold mb-1">
        {i18n.locale === "ar" ? "في الانتظار..." : "Queued..."}
    </div>
{/if}
