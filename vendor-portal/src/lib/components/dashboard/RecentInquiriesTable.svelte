<script lang="ts">
    import { ExternalLink } from 'lucide-svelte';
    import { getWaitTime, formatWeddingDate, formatDate, getInitials } from '../../services/dashboard.service';
    import StatusBadge from '$lib/components/StatusBadge.svelte';
    import Pagination from '$lib/components/Pagination.svelte';
    import { enhance } from '$app/forms';

    let { i18n, dashboardState } = $props<{
        i18n: any;
        dashboardState: any;
    }>();
</script>

<div class="table-card">
    <div class="table-header">
        <div class="table-header-left">
            <span class="table-title">{i18n.t.dashboard.newInquiries}</span>
            {#if dashboardState.inquiries.length > 0}
                <span class="table-count-badge">{dashboardState.inquiries.length}</span>
            {/if}
        </div>
        <div class="table-header-right">
            {#if dashboardState.inquiries.length > 0}
                <a href="/dashboard/couples" class="btn btn-ghost btn-sm">
                    <ExternalLink size={13} />
                    {i18n.locale === 'ar' ? 'عرض الكل' : 'View All'}
                </a>
            {/if}
        </div>
    </div>

    <div class="table-wrap">
        {#if dashboardState.inquiries.length === 0}
            <div class="empty-state">
                <div class="empty-icon">📬</div>
                <h3>{i18n.locale === 'ar' ? 'لا توجد استفسارات بعد' : 'No inquiries yet'}</h3>
                <p>
                    {i18n.locale === 'ar'
                        ? 'ستظهر هنا طلبات العرسان فور إرسالها من خلال الموقع.'
                        : 'Couple inquiries will appear here as soon as they submit via the marketplace.'}
                </p>
            </div>
        {:else}
            <table>
                <thead>
                    <tr>
                        <th>{i18n.t.couples.name}</th>
                        <th>{i18n.t.couples.eventDate}</th>
                        <th>{i18n.t.couples.receivedDate}</th>
                        <th>{i18n.t.couples.waitTime}</th>
                        <th>{i18n.t.couples.status}</th>
                        <th>{i18n.t.common.actions}</th>
                    </tr>
                </thead>
                <tbody>
                    {#each dashboardState.pagedInquiries as row}
                        <tr>
                            <td>
                                <div class="cell-avatar">
                                    <div class="cell-avatar-img">{getInitials(row.customer_name)}</div>
                                    <div>
                                        <div class="cell-avatar-label">{row.customer_name}</div>
                                        {#if row.phone || row.customer_phone}
                                            <div class="cell-avatar-sub">{row.phone || row.customer_phone}</div>
                                        {/if}
                                    </div>
                                </div>
                            </td>
                            <td class="date-cell">{formatWeddingDate(row.wedding_date, i18n.locale)}</td>
                            <td class="date-cell">{formatDate(row.created_at, i18n.locale)}</td>
                            <td>
                                <span class="wait-chip">{getWaitTime(row.created_at, i18n)}</span>
                            </td>
                            <td><StatusBadge status={row.status} /></td>
                            <td>
                                <button
                                    class="btn-icon btn-icon-sm"
                                    onclick={() => dashboardState.openEditModal(row)}
                                    title={i18n.t.common.edit}
                                    aria-label={i18n.t.common.edit}
                                >
                                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                        <path d="M11 4H4a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-7"/>
                                        <path d="M18.5 2.5a2.121 2.121 0 013 3L12 15l-4 1 1-4 9.5-9.5z"/>
                                    </svg>
                                </button>
                            </td>
                        </tr>
                    {/each}
                </tbody>
            </table>
        {/if}
    </div>

    {#if dashboardState.inquiries.length > 0}
        <Pagination bind:current={dashboardState.currentPage} total={dashboardState.totalPages} totalRows={dashboardState.inquiries.length} />
    {/if}
</div>

{#if dashboardState.isModalOpen && dashboardState.selectedInquiry}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
        class="modal-backdrop"
        onclick={dashboardState.closeEditModal}
        role="presentation"
    >
        <div
            class="modal"
            onclick={(e) => e.stopPropagation()}
            role="dialog"
            aria-modal="true"
            aria-labelledby="modal-edit-title"
            tabindex="-1"
        >
            <form
                method="POST"
                action="?/updateStatus"
                use:enhance={() => {
                    return async ({ result }) => {
                        if (result.type === 'success') {
                            dashboardState.closeEditModal();
                        }
                    };
                }}
            >
                <input type="hidden" name="id" value={dashboardState.selectedInquiry.id} />
                <div class="modal-header">
                    <h2 id="modal-edit-title" class="modal-title">
                        {i18n.t.common.edit}
                        <span class="modal-name">— {dashboardState.selectedInquiry.customer_name}</span>
                    </h2>
                    <button
                        type="button"
                        class="modal-close"
                        onclick={dashboardState.closeEditModal}
                        aria-label={i18n.t.common.close}
                    >✕</button>
                </div>
                <div class="modal-body">
                    <div class="form-group">
                        <label class="form-label" for="inquiry-status">
                            {i18n.t.couples.status}
                        </label>
                        <div class="select-wrapper">
                            <select
                                id="inquiry-status"
                                name="status"
                                class="form-select"
                                bind:value={dashboardState.selectedStatus}
                            >
                                <option value="new">{i18n.t.couples.statusNew}</option>
                                <option value="done">{i18n.t.couples.statusDone}</option>
                                <option value="negot">{i18n.t.couples.statusNegot}</option>
                                <option value="unreach">{i18n.t.couples.statusUnreach}</option>
                                <option value="expired">{i18n.t.couples.statusExpired}</option>
                                <option value="rejected">{i18n.t.couples.statusRejected}</option>
                                <option value="paid">{i18n.t.couples.statusPaid}</option>
                            </select>
                        </div>
                    </div>
                </div>
                <div class="modal-footer">
                    <button type="button" class="btn btn-ghost" onclick={dashboardState.closeEditModal}>
                        {i18n.t.common.cancel}
                    </button>
                    <button type="submit" class="btn btn-primary">
                        {i18n.t.common.save}
                    </button>
                </div>
            </form>
        </div>
    </div>
{/if}
