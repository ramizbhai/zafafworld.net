<script lang="ts">
    let { vendorsState, activeCount, stoppedCount, totalCount } = $props<{ vendorsState: any, activeCount: number, stoppedCount: number, totalCount: number }>();
</script>

<div class="controls-bar">
    <div class="tabs-container">
        <button class="tab-btn" class:active={vendorsState.activeTab === 'active'} onclick={() => { vendorsState.activeTab = 'active'; vendorsState.errorMessage = ''; vendorsState.successMessage = ''; }}>
            Active / نشط <span class="tab-count">{activeCount}</span>
        </button>
        <button class="tab-btn" class:active={vendorsState.activeTab === 'stopped'} onclick={() => { vendorsState.activeTab = 'stopped'; vendorsState.errorMessage = ''; vendorsState.successMessage = ''; }}>
            Stopped / موقوف <span class="tab-count">{stoppedCount}</span>
        </button>
        <button class="tab-btn" class:active={vendorsState.activeTab === 'all'} onclick={() => { vendorsState.activeTab = 'all'; vendorsState.errorMessage = ''; vendorsState.successMessage = ''; }}>
            All Vendors / الكل <span class="tab-count">{totalCount}</span>
        </button>
    </div>

    <div class="search-box">
        <input type="search" placeholder="Search by name, email, phone, category, or city..." bind:value={vendorsState.search} />
    </div>
</div>
