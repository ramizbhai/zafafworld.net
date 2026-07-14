<script lang="ts">
    import InquiryRow from "./InquiryRow.svelte";

    let { i18n, inquiriesState } = $props<{ i18n: any, inquiriesState: any }>();
</script>

<div class="leads-section">
    {#if !inquiriesState.rawInquiries || inquiriesState.rawInquiries.length === 0}
        <div class="empty-state">
            <div class="empty-illustration">📬</div>
            <h3>{i18n.locale === 'ar' ? 'لا توجد طلبات واردة' : 'Leads Pipeline Empty'}</h3>
            <p>{i18n.locale === 'ar' ? 'لم تتلقى أي طلبات أو استفسارات من العملاء حتى الآن. ستظهر هنا بمجرد إرسالها.' : 'You have not received any customer inquiries yet. Active leads will appear here.'}</p>
        </div>
    {:else if inquiriesState.paginatedInquiries.length === 0}
        <div class="empty-state">
            <div class="empty-illustration">🔍</div>
            <h3>{i18n.t.common.noData}</h3>
            <p>{i18n.locale === 'ar' ? 'لم نعثر على أي نتائج مطابقة لبحثك الحالي.' : 'No inquiries match your active search filters.'}</p>
            <button onclick={() => { inquiriesState.searchQuery = ''; inquiriesState.statusFilter = 'all'; }} class="btn-reset">
                {i18n.locale === 'ar' ? 'إعادة تعيين الفلترة' : 'Reset Filters'}
            </button>
        </div>
    {:else}
        <div class="leads-grid">
            {#each inquiriesState.paginatedInquiries as inq (inq.id)}
                <InquiryRow inquiry={inq} {i18n} {inquiriesState} />
            {/each}
        </div>

        {#if inquiriesState.totalPages > 1}
            <div class="pagination">
                <span class="pagination-info">
                    {i18n.locale === 'ar' ? `صفحة ${inquiriesState.currentPage} من ${inquiriesState.totalPages}` : `Page ${inquiriesState.currentPage} of ${inquiriesState.totalPages}`}
                </span>
                <div class="pagination-buttons">
                    <button 
                        class="page-btn" 
                        disabled={inquiriesState.currentPage === 1} 
                        onclick={() => inquiriesState.currentPage -= 1}
                    >
                        {i18n.locale === 'ar' ? 'السابق' : 'Back'}
                    </button>
                    {#each Array.from({ length: inquiriesState.totalPages }) as _, i}
                        <button 
                            class="page-btn" 
                            class:active={inquiriesState.currentPage === i + 1}
                            onclick={() => inquiriesState.currentPage = i + 1}
                        >
                            {i + 1}
                        </button>
                    {/each}
                    <button 
                        class="page-btn" 
                        disabled={inquiriesState.currentPage === inquiriesState.totalPages} 
                        onclick={() => inquiriesState.currentPage += 1}
                    >
                        {i18n.locale === 'ar' ? 'التالي' : 'Next'}
                    </button>
                </div>
            </div>
        {/if}
    {/if}
</div>
