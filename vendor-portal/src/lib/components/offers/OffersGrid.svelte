<script lang="ts">
    import OfferCard from "./OfferCard.svelte";
    import { goto } from "$app/navigation";
    import { page } from "$app/stores";

    let { i18n, offersState, data } = $props<{
        i18n: any;
        offersState: any;
        data: any;
    }>();

    function goToPage(pg: number) {
        const params = new URLSearchParams($page.url.searchParams);
        params.set("page", pg.toString());
        goto(`?${params.toString()}`, { replaceState: true, invalidateAll: true });
    }
</script>

{#if offersState.filteredOffers.length === 0}
    <div class="empty-state">
        <div class="empty-illustration">🎁</div>
        <h3>
            {#if offersState.activeTab === "all" && data.offers.length === 0}
                {i18n.locale === "ar" ? "لا توجد عروض ترويجية بعد" : "No Promotions Yet"}
            {:else}
                {i18n.locale === "ar" ? "لا توجد عروض في هذا التصنيف" : "No Promotions in This Category"}
            {/if}
        </h3>
        <p>
            {i18n.locale === "ar"
                ? "أنشئ عرضاً ترويجياً لتنشيط مبيعاتك وجذب المزيد من العملاء."
                : "Create a promotion to boost your visibility and attract more inquiries."}
        </p>
        {#if offersState.activeTab === "all" && data.offers.length === 0}
            <a href="/dashboard/offers/new" class="btn btn-primary">
                {i18n.locale === "ar" ? "أضف عرضك الأول" : "Create First Promotion"}
            </a>
        {/if}
    </div>
{:else}
    <div class="promotions-grid">
        {#each offersState.filteredOffers as offer (offer.id)}
            <OfferCard {offer} {i18n} products={data.products} {offersState} />
        {/each}
    </div>

    <!-- ─── PAGINATION ────────────────────────────────────────────────── -->
    {#if offersState.totalPages > 1}
        <div class="pagination-bar">
            <button
                class="page-btn"
                disabled={offersState.currentPage <= 1}
                onclick={() => goToPage(offersState.currentPage - 1)}
            >
                {i18n.locale === "ar" ? "→" : "←"}
            </button>
            {#each Array.from({ length: offersState.totalPages }, (_, i) => i + 1) as pg}
                <button
                    class="page-btn"
                    class:active={offersState.currentPage === pg}
                    onclick={() => goToPage(pg)}
                >
                    {pg}
                </button>
            {/each}
            <button
                class="page-btn"
                disabled={offersState.currentPage >= offersState.totalPages}
                onclick={() => goToPage(offersState.currentPage + 1)}
            >
                {i18n.locale === "ar" ? "←" : "→"}
            </button>
        </div>
    {/if}
{/if}
