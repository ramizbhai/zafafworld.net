<script lang="ts">
    import CoupleCard from "./CoupleCard.svelte";

    let { i18n, couplesState, data } = $props<{ i18n: any, couplesState: any, data: any }>();
</script>

<div class="leads-section">
    {#if !data.inquiries || data.inquiries.length === 0}
        <div class="empty-state">
            <div class="empty-illustration">📬</div>
            <h3>{i18n.locale === 'ar' ? 'قائمة الاستفسارات فارغة' : 'Leads Pipeline Empty'}</h3>
            <p>{i18n.locale === 'ar' ? 'لم تتلقى أي استفسارات حتى الآن. قم بإكمال ملفك الشخصي وتنشيط عروضك للحصول على حجز.' : 'You have not received any wedding queries yet. Complete your profile details and activate packages to gain market placements.'}</p>
        </div>
    {:else if couplesState.paginatedInquiries.length === 0}
        <div class="empty-state">
            <div class="empty-illustration">🔍</div>
            <h3>{i18n.t.common.noData}</h3>
            <p>{i18n.locale === 'ar' ? 'لم تعثر معايير البحث والفلترة الحالية على أي نتائج. حاول تغيير الفلتر.' : 'Your search or status criteria did not yield any entries. Try resetting filters.'}</p>
            <button onclick={() => { couplesState.searchQuery = ''; couplesState.statusFilter = 'all'; }} class="btn btn-outline">
                {i18n.locale === 'ar' ? 'إعادة تعيين الفلترة' : 'Reset Filters'}
            </button>
        </div>
    {:else}
        <div class="leads-grid">
            {#each couplesState.paginatedInquiries as lead (lead.id)}
                <CoupleCard {lead} {i18n} {couplesState} />
            {/each}
        </div>

        {#if couplesState.totalPages > 1}
            <div class="pagination">
                <span class="pagination-info">
                    {i18n.locale === 'ar' ? `صفحة ${couplesState.currentPage} من ${couplesState.totalPages}` : `Page ${couplesState.currentPage} of ${couplesState.totalPages}`}
                </span>
                <div class="pagination-buttons">
                    <button 
                        class="page-btn" 
                        disabled={couplesState.currentPage === 1} 
                        onclick={() => couplesState.currentPage -= 1}
                    >
                        {i18n.t.common.back}
                    </button>
                    {#each Array(couplesState.totalPages) as _, i}
                        <button 
                            class="page-btn" 
                            class:active={couplesState.currentPage === i + 1}
                            onclick={() => couplesState.currentPage = i + 1}
                        >
                            {i + 1}
                        </button>
                    {/each}
                    <button 
                        class="page-btn" 
                        disabled={couplesState.currentPage === couplesState.totalPages} 
                        onclick={() => couplesState.currentPage += 1}
                    >
                        {i18n.t.common.next}
                    </button>
                </div>
            </div>
        {/if}
    {/if}
</div>
