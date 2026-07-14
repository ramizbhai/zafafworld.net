<script lang="ts">
    import { goto } from "$app/navigation";
    import { page } from "$app/stores";
    import { tabs, type TabId } from "../../stores/offersState.svelte";

    let { i18n, offersState } = $props<{
        i18n: any;
        offersState: any;
    }>();

    let searchTimeout: ReturnType<typeof setTimeout> | undefined;

    function handleSearch() {
        clearTimeout(searchTimeout);
        searchTimeout = setTimeout(() => {
            const params = new URLSearchParams($page.url.searchParams);
            if (offersState.searchQuery.trim()) {
                params.set("search", offersState.searchQuery.trim());
            } else {
                params.delete("search");
            }
            params.delete("page");
            goto(`?${params.toString()}`, { replaceState: true, invalidateAll: true });
        }, 400);
    }
</script>

<div class="toolbar">
    <div class="tabs-row">
        {#each tabs as tab}
            <button
                class="tab-btn"
                class:active={offersState.activeTab === tab.id}
                onclick={() => offersState.activeTab = tab.id}
            >
                {i18n.locale === "ar" ? tab.labelAr : tab.labelEn}
                {#if offersState.tabCounts[tab.id] > 0}
                    <span class="tab-count">{offersState.tabCounts[tab.id]}</span>
                {/if}
            </button>
        {/each}
    </div>
    <div class="search-bar">
        <span class="search-icon">🔍</span>
        <input
            type="text"
            class="search-input"
            placeholder={i18n.locale === "ar" ? "ابحث في العروض..." : "Search promotions..."}
            bind:value={offersState.searchQuery}
            oninput={handleSearch}
        />
    </div>
</div>
