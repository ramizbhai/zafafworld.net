<script lang="ts">
    import { AlertTriangle, CheckCircle2, Building2, Mail, Phone, Clock, CheckCircle, AlertCircle } from 'lucide-svelte';
    import { enhance } from '$app/forms';

    let { pendingListings, vendorsState } = $props<{ pendingListings: any[], vendorsState: any }>();
</script>

<section class="listings-queue-section">
    <div class="listings-queue-header">
        <div class="listings-queue-title-group">
            <div class="listings-queue-icon">
                <AlertTriangle size={18} />
            </div>
            <div>
                <h2 class="listings-queue-title">Pending Listing Approvals / قائمة انتظار الإعلانات</h2>
                <p class="listings-queue-sub">Review and moderate individual business listings submitted by vendors. Listings go live only after approval.</p>
            </div>
        </div>
        {#if pendingListings.length > 0}
            <span class="queue-count-badge">{pendingListings.length} pending</span>
        {/if}
    </div>

    {#if pendingListings.length === 0}
        <div class="queue-empty">
            <CheckCircle2 size={40} class="queue-empty-icon" />
            <h3>No Pending Listings</h3>
            <p>All submitted listings have been reviewed. New submissions will appear here automatically.</p>
        </div>
    {:else}
        <div class="listing-cards">
            {#each pendingListings as listing (listing.id)}
                <div class="listing-card" class:submitting={vendorsState.submittingListingIds.includes(listing.id)}>
                    <div class="listing-card-left">
                        <div class="listing-meta-top">
                            <span class="listing-category-badge">{listing.product_category}</span>
                            {#if listing.attributes?.gender_section}
                                <span class="listing-gender-badge">{listing.attributes.gender_section}</span>
                            {/if}
                            {#if listing.base_price_sar}
                                <span class="listing-price">SAR {listing.base_price_sar.toLocaleString()}</span>
                            {/if}
                        </div>
                        <div class="listing-names">
                            <span class="listing-name-en">{listing.title}</span>
                        </div>
                        <div class="listing-vendor-row">
                            <Building2 size={13} class="listing-vendor-icon" />
                            <span class="listing-vendor-label">
                                {listing.vendor_name_en}
                                {#if listing.vendor_name_ar}
                                    <span dir="rtl"> / {listing.vendor_name_ar}</span>
                                {/if}
                            </span>
                            <span class="listing-city">{listing.city_name_en}</span>
                        </div>
                        <div class="listing-contact-row">
                            {#if listing.vendor_email}
                                <a href="mailto:{listing.vendor_email}" class="listing-contact-link">
                                    <Mail size={12} /> {listing.vendor_email}
                                </a>
                            {/if}
                            {#if listing.vendor_phone}
                                <a href="tel:{listing.vendor_phone}" class="listing-contact-link">
                                    <Phone size={12} /> {listing.vendor_phone}
                                </a>
                            {/if}
                        </div>
                        <div class="listing-submitted">
                            <Clock size={12} />
                            Submitted: {new Date(listing.created_at).toLocaleDateString(undefined, { year: 'numeric', month: 'short', day: 'numeric' })}
                        </div>
                    </div>

                    <div class="listing-card-right">
                        <!-- Rejection reason textarea -->
                        <div class="rejection-reason-field">
                            <label class="rejection-label" for="reason-{listing.id}">Rejection Reason (if rejecting):</label>
                            <textarea
                                id="reason-{listing.id}"
                                class="rejection-textarea"
                                placeholder="Explain why this listing is being rejected so the vendor can fix it..."
                                bind:value={vendorsState.rejectionReasonMap[listing.id]}
                                rows="2"
                            ></textarea>
                        </div>

                        <div class="listing-action-row">
                            <!-- Approve button -->
                            <form method="POST" action="?/moderateListing" use:enhance={() => {
                                vendorsState.submittingListingIds = [...vendorsState.submittingListingIds, listing.id];
                                vendorsState.errorMessage = ''; vendorsState.successMessage = '';
                                return async ({ result, update }) => {
                                    vendorsState.submittingListingIds = vendorsState.submittingListingIds.filter((id: string) => id !== listing.id);
                                    if (result.type === 'success' && result.data?.success) {
                                        vendorsState.successMessage = `Listing approved and is now live.`;
                                    } else if (result.type === 'failure') {
                                        vendorsState.errorMessage = (result.data?.error as string) || 'Approval failed';
                                    }
                                    update({ reset: false });
                                };
                            }}>
                                <input type="hidden" name="vendor_id" value={listing.vendor_id} />
                                <input type="hidden" name="product_id" value={listing.id} />
                                <input type="hidden" name="status" value="active" />
                                <button type="submit" class="btn-action approve" disabled={vendorsState.submittingListingIds.includes(listing.id)}>
                                    <CheckCircle size={14} /> Approve / قبول
                                </button>
                            </form>

                            <!-- Reject button -->
                            <form method="POST" action="?/moderateListing" use:enhance={({ cancel }) => {
                                const r = vendorsState.rejectionReasonMap[listing.id] || '';
                                if (r.trim().length < 5) {
                                    vendorsState.errorMessage = 'A descriptive rejection reason (minimum 5 characters) is required.';
                                    cancel();
                                    return;
                                }
                                vendorsState.submittingListingIds = [...vendorsState.submittingListingIds, listing.id];
                                vendorsState.errorMessage = ''; vendorsState.successMessage = '';
                                return async ({ result, update }) => {
                                    vendorsState.submittingListingIds = vendorsState.submittingListingIds.filter((id: string) => id !== listing.id);
                                    if (result.type === 'success' && result.data?.success) {
                                        vendorsState.successMessage = `Listing rejected. Vendor has been notified.`;
                                    } else if (result.type === 'failure') {
                                        vendorsState.errorMessage = (result.data?.error as string) || 'Rejection failed';
                                    }
                                    update({ reset: false });
                                };
                            }}>
                                <input type="hidden" name="vendor_id" value={listing.vendor_id} />
                                <input type="hidden" name="product_id" value={listing.id} />
                                <input type="hidden" name="status" value="rejected" />
                                <input type="hidden" name="reason" value={vendorsState.rejectionReasonMap[listing.id] || ''} />
                                <button type="submit" class="btn-action reject" disabled={vendorsState.submittingListingIds.includes(listing.id)}>
                                    <AlertCircle size={14} /> Reject / رفض
                                </button>
                            </form>

                            <!-- View details link -->
                            <a href="/dashboard/vendors/{listing.vendor_id}" class="btn-action detail-link">
                                <Building2 size={14} /> View Vendor
                            </a>
                        </div>
                    </div>
                </div>
            {/each}
        </div>
    {/if}
</section>
