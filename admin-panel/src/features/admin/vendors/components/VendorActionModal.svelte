<script lang="ts">
    import { fade, fly } from 'svelte/transition';
    import { X, Sparkles } from 'lucide-svelte';
    import { enhance } from '$app/forms';

    let { vendorsState } = $props<{ vendorsState: any }>();

    let computedExpiryDate = $derived.by(() => {
        if (!vendorsState.adIsFeatured) return '';
        if (vendorsState.adExpiryPreset === '1month') {
            const d = new Date();
            d.setMonth(d.getMonth() + 1);
            return d.toISOString().split('T')[0];
        }
        if (vendorsState.adExpiryPreset === '3months') {
            const d = new Date();
            d.setMonth(d.getMonth() + 3);
            return d.toISOString().split('T')[0];
        }
        return vendorsState.adCustomDate;
    });
</script>

{#if vendorsState.promptStatusChange}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div class="dialog-backdrop" role="button" tabindex="-1" onclick={() => vendorsState.promptStatusChange = null} onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') vendorsState.promptStatusChange = null; }} transition:fade></div>
    <div class="dialog-box" transition:fly={{ y: 20, duration: 200 }}>
        <div class="dialog-header">
            <h4>Provide Status Reason</h4>
            <button class="close-btn" onclick={() => vendorsState.promptStatusChange = null}><X size={16} /></button>
        </div>
        <div class="dialog-body">
            <p>Reason for setting vendor status to <strong>{vendorsState.promptStatusChange.status}</strong>:</p>
            <textarea placeholder="Enter reason explaining the rejection or suspension..." bind:value={vendorsState.statusReason}></textarea>
        </div>
        <div class="dialog-actions">
            <button class="btn btn-outline" onclick={() => vendorsState.promptStatusChange = null}>Cancel</button>
            <form 
                method="POST" 
                action="?/updateStatus" 
                use:enhance={() => {
                    vendorsState.submittingIds.push(vendorsState.promptStatusChange!.id);
                    const targetId = vendorsState.promptStatusChange!.id;
                    vendorsState.promptStatusChange = null;
                    vendorsState.errorMessage = '';
                    vendorsState.successMessage = '';
                    return async ({ result, update }) => {
                        vendorsState.submittingIds = vendorsState.submittingIds.filter((id: string) => id !== targetId);
                        if (result.type === 'success' && result.data?.success) {
                            vendorsState.successMessage = `Vendor status updated successfully.`;
                        } else if (result.type === 'failure') {
                            vendorsState.errorMessage = (result.data?.error as string) || 'Failed to update status';
                        }
                        update({ reset: false });
                    };
                }}
            >
                <input type="hidden" name="id" value={vendorsState.promptStatusChange.id} />
                <input type="hidden" name="status" value={vendorsState.promptStatusChange.status} />
                <input type="hidden" name="reason" value={vendorsState.statusReason} />
                <button type="submit" class="btn btn-danger">Confirm</button>
            </form>
        </div>
    </div>
{/if}

{#if vendorsState.promptAdModal}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div class="dialog-backdrop" role="button" tabindex="-1" onclick={() => vendorsState.promptAdModal = null} onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') vendorsState.promptAdModal = null; }} transition:fade></div>
    <div class="dialog-box premium-ad-modal" transition:fly={{ y: 20, duration: 200 }}>
        <div class="dialog-header ad-modal-header">
            <div class="header-title-group">
                <Sparkles size={18} class="sparkle-gold-icon" />
                <h4>Premium Placement Engine</h4>
            </div>
            <button class="close-btn" onclick={() => vendorsState.promptAdModal = null}><X size={16} /></button>
        </div>
        
        <div class="dialog-body ad-modal-body">
            <p class="ad-subtitle">Manage ad status, featured highlights, and campaign expiry for <strong>{vendorsState.promptAdModal.name_en}</strong>.</p>
            
            <div class="ad-form-section">
                <!-- Premium Toggle Switch -->
                <div class="toggle-control-group">
                    <span class="toggle-label">Activate Featured Status / تفعيل الإعلان المميز</span>
                    <button 
                        type="button" 
                        class="toggle-switch-btn" 
                        class:active={vendorsState.adIsFeatured} 
                        onclick={() => vendorsState.adIsFeatured = !vendorsState.adIsFeatured}
                    >
                        <span class="toggle-slider"></span>
                        <span class="toggle-text">{vendorsState.adIsFeatured ? 'ACTIVE / نشط' : 'INACTIVE / غير نشط'}</span>
                    </button>
                </div>

                {#if vendorsState.adIsFeatured}
                    <div class="expiry-duration-section">
                        <span class="section-subtitle">Campaign Duration & Expiry Timeline</span>
                        
                        <!-- Predefined Presets -->
                        <div class="preset-buttons">
                            <button 
                                type="button" 
                                class="preset-btn" 
                                class:active={vendorsState.adExpiryPreset === '1month'} 
                                onclick={() => { vendorsState.adExpiryPreset = '1month'; }}
                            >
                                1 Month / شهر
                            </button>
                            <button 
                                type="button" 
                                class="preset-btn" 
                                class:active={vendorsState.adExpiryPreset === '3months'} 
                                onclick={() => { vendorsState.adExpiryPreset = '3months'; }}
                            >
                                3 Months / ٣ أشهر
                            </button>
                            <button 
                                type="button" 
                                class="preset-btn" 
                                class:active={vendorsState.adExpiryPreset === 'custom'} 
                                onclick={() => { vendorsState.adExpiryPreset = 'custom'; }}
                            >
                                Custom Expiry / مخصص
                            </button>
                        </div>

                        <!-- Date picker -->
                        <div class="date-picker-group">
                            <label for="expiry-date-input">Target Expiration Date (UTC):</label>
                            <input 
                                id="expiry-date-input"
                                type="date" 
                                class="date-input" 
                                min={new Date().toISOString().split('T')[0]} 
                                bind:value={vendorsState.adCustomDate} 
                                disabled={vendorsState.adExpiryPreset !== 'custom'} 
                            />
                            {#if computedExpiryDate}
                                <span class="calculated-expiry-label">
                                    Resolved Expiry: {new Date(computedExpiryDate).toLocaleDateString(undefined, {
                                        year: 'numeric',
                                        month: 'long',
                                        day: 'numeric'
                                    })}
                                </span>
                            {/if}
                        </div>
                    </div>
                {/if}
            </div>
        </div>

        <div class="dialog-actions ad-modal-actions">
            <button class="btn btn-outline" onclick={() => vendorsState.promptAdModal = null}>Cancel</button>
            <form 
                method="POST" 
                action="?/updateFeatured" 
                use:enhance={() => {
                    vendorsState.submittingIds.push(vendorsState.promptAdModal!.id);
                    const targetId = vendorsState.promptAdModal!.id;
                    vendorsState.promptAdModal = null;
                    vendorsState.errorMessage = '';
                    vendorsState.successMessage = '';
                    return async ({ result, update }) => {
                        vendorsState.submittingIds = vendorsState.submittingIds.filter((id: string) => id !== targetId);
                        if (result.type === 'success' && result.data?.success) {
                            vendorsState.successMessage = `Ad placement details updated successfully!`;
                        } else if (result.type === 'failure') {
                            vendorsState.errorMessage = (result.data?.error as string) || 'Failed to update ad placement';
                        }
                        update({ reset: false });
                    };
                }}
            >
                <input type="hidden" name="id" value={vendorsState.promptAdModal.id} />
                <input type="hidden" name="is_featured" value={vendorsState.adIsFeatured ? 'true' : 'false'} />
                <input type="hidden" name="expires_at" value={vendorsState.adIsFeatured ? computedExpiryDate : ''} />
                <button type="submit" class="btn btn-premium-submit">
                    <Sparkles size={14} class="btn-icon" /> Save Changes / حفظ التغييرات
                </button>
            </form>
        </div>
    </div>
{/if}
