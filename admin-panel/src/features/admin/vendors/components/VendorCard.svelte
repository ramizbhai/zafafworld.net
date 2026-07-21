<script lang="ts">
    import { Building2, MapPin, Calendar, Clock, Mail, Phone, MessageCircle, Copy, Check, Sparkles, MessageSquare } from 'lucide-svelte';
    import { enhance } from '$app/forms';
    import { getCategoryLabel } from '../vendors.service';

    let { vendor, vendorsState, tiers } = $props<{ vendor: any, vendorsState: any, tiers: any[] }>();

    let copiedText = $state('');
    function copyText(text: string) {
        navigator.clipboard.writeText(text);
        copiedText = text;
        setTimeout(() => {
            if (copiedText === text) copiedText = '';
        }, 1500);
    }
</script>

<div class="curation-card" class:submitting={vendorsState.submittingIds.includes(vendor.id)}>
    <div class="card-edge" class:pending={vendor.status === 'pending'} class:approved={vendor.status === 'approved' && vendor.subscription_status !== 'stopped'} class:stopped={vendor.subscription_status === 'stopped'}></div>
    <div class="card-inner">
        
        <!-- Left: Information Segment -->
        <div class="vendor-details">
            <div class="vendor-identity">
                <div class="vendor-icon">
                    <Building2 size={24} />
                </div>
                <div class="vendor-names">
                    <div class="name-row">
                        <span class="name-en">{vendor.name_en}</span>
                        {#if vendor.name_ar}
                            <span class="name-ar" dir="rtl">{vendor.name_ar}</span>
                        {/if}
                        {#if vendor.is_featured}
                            <span class="badge-featured-mini" title="Active Premium Placement / إعلان نشط">
                                <Sparkles size={10} /> Featured
                            </span>
                        {/if}
                    </div>
                    
                    <!-- Badges -->
                    <div class="badge-row">
                        <!-- Approval status badge -->
                        <span class="status-badge" class:pending={vendor.status === 'pending'} class:approved={vendor.status === 'approved'} class:suspended={vendor.status === 'suspended'} class:rejected={vendor.status === 'rejected'}>
                            {vendor.status.toUpperCase()}
                        </span>
                        
                        <!-- Subscription status badge -->
                        <span class="sub-badge" class:trial={vendor.subscription_status === 'trial'} class:active={vendor.subscription_status === 'active'} class:stopped={vendor.subscription_status === 'stopped'}>
                            Sub: {vendor.subscription_status.toUpperCase()}
                        </span>

                        <!-- Tier & Quota Badges -->
                        <span class="tier-badge" class:free={!vendor.current_tier || vendor.current_tier === 'Free'} class:gold={vendor.current_tier === 'Gold' || vendor.current_tier === 'Golden'} class:vip={vendor.current_tier === 'VIP'} class:diamond={vendor.current_tier === 'Diamond'} title="Current Tier">
                            🌟 {vendor.current_tier || 'Free'}
                        </span>
                        <span class="quota-badge" title="Usage Quota">
                            📊 {vendor.used_products_count || 0}/{vendor.max_products < 0 ? '∞' : vendor.max_products} Products
                        </span>
                    </div>
                </div>
            </div>

            <!-- Meta Fields -->
            <div class="vendor-meta">
                <div class="meta-item">
                    <MapPin size={14} class="meta-icon" />
                    <span>{vendor.city_name_en} / {vendor.city_name_ar}</span>
                </div>
                <div class="meta-item">
                    <Calendar size={14} class="meta-icon" />
                    <span>{getCategoryLabel(vendor.category)}</span>
                </div>
                <div class="meta-item">
                    <Clock size={14} class="meta-icon" />
                    <span>
                        Registered: {new Date(vendor.created_at).toLocaleDateString(undefined, {
                            year: 'numeric',
                            month: 'short',
                            day: 'numeric'
                        })}
                    </span>
                </div>
            </div>

            <!-- Full Contact Info Section -->
            <div class="vendor-contacts">
                <div class="contact-pill">
                    <Mail size={12} class="contact-icon" />
                    <a href="mailto:{vendor.email}" class="contact-text">{vendor.email}</a>
                    <button class="copy-btn" onclick={() => copyText(vendor.email)} title="Copy Email">
                        {#if copiedText === vendor.email}
                            <Check size={12} class="text-success" />
                        {:else}
                            <Copy size={12} />
                        {/if}
                    </button>
                </div>

                <div class="contact-pill">
                    <Phone size={12} class="contact-icon" />
                    <a href="tel:{vendor.phone}" class="contact-text">{vendor.phone}</a>
                    <button class="copy-btn" onclick={() => copyText(vendor.phone)} title="Copy Phone">
                        {#if copiedText === vendor.phone}
                            <Check size={12} class="text-success" />
                        {:else}
                            <Copy size={12} />
                        {/if}
                    </button>
                    {#if vendor.phone}
                        <a href="https://wa.me/{vendor.phone.replace(/[^0-9]/g, '')}" target="_blank" rel="noopener noreferrer" class="whatsapp-btn" title="Chat on WhatsApp">
                            <MessageCircle size={12} />
                            <span>WhatsApp</span>
                        </a>
                    {/if}
                </div>
            </div>
        </div>

        <!-- Right: Actions Segment -->
        <div class="vendor-action-box">
            <!-- Subscription Switcher for Active Vendors -->
            {#if vendor.status === 'active'}
                <div class="subscription-control-wrapper">
                    <span class="control-label">Subscription Tier / الباقة</span>
                    <form 
                        method="POST" 
                        action="?/updateSubscription" 
                        use:enhance={({ formData, submitter }) => {
                            vendorsState.submittingIds.push(vendor.id);
                            vendorsState.errorMessage = '';
                            vendorsState.successMessage = '';
                            
                            if (submitter?.getAttribute('name') === 'subscription_tier_id') {
                                formData.set('subscription_status', vendor.subscription_status);
                            } else if (submitter?.getAttribute('name') === 'subscription_status') {
                                formData.set('subscription_tier_id', vendor.tier_id || vendor.subscriptionTierId || '');
                            }

                            return async ({ result, update }) => {
                                vendorsState.submittingIds = vendorsState.submittingIds.filter((id: string) => id !== vendor.id);
                                if (result.type === 'success' && result.data?.success) {
                                    vendorsState.successMessage = `Vendor subscription updated successfully.`;
                                } else if (result.type === 'failure') {
                                    vendorsState.errorMessage = (result.data?.error as string) || 'Failed to update subscription';
                                }
                                update({ reset: false });
                            };
                        }}
                    >
                        <input type="hidden" name="id" value={vendor.id} />
                        <div class="subscription-toggle">
                            <button type="submit" name="subscription_status" value="trial" class="sub-btn trial" class:active={vendor.subscription_status === 'trial'} disabled={vendorsState.submittingIds.includes(vendor.id)}>
                                Trial
                            </button>
                            <button type="submit" name="subscription_status" value="active" class="sub-btn active-tier" class:active={vendor.subscription_status === 'active'} disabled={vendorsState.submittingIds.includes(vendor.id)}>
                                Active
                            </button>
                            <button type="submit" name="subscription_status" value="stopped" class="sub-btn stopped" class:active={vendor.subscription_status === 'stopped'} disabled={vendorsState.submittingIds.includes(vendor.id)}>
                                Stopped
                            </button>
                        </div>
                        
                        <div class="tier-toggle" style="margin-top: 8px;">
                            {#each tiers as tier}
                                <button 
                                    type="submit" 
                                    name="subscription_tier_id" 
                                    value={tier.id} 
                                    class="tier-btn" 
                                    class:active={vendor.tier_id === tier.id || vendor.subscriptionTierId === tier.id}
                                    disabled={vendorsState.submittingIds.includes(vendor.id)}
                                    title={tier.name}
                                >
                                    {tier.name}
                                </button>
                            {/each}
                        </div>
                    </form>
                </div>
            {/if}

            <div class="button-group">
                <!-- Suspended / Banned account reactivation -->
                {#if vendor.status === 'suspended' || vendor.status === 'banned'}
                    <form 
                        method="POST" 
                        action="?/reactivate" 
                        use:enhance={() => {
                            vendorsState.submittingIds.push(vendor.id);
                            vendorsState.errorMessage = '';
                            vendorsState.successMessage = '';
                            return async ({ result, update }) => {
                                vendorsState.submittingIds = vendorsState.submittingIds.filter((id: string) => id !== vendor.id);
                                if (result.type === 'success' && result.data?.success) {
                                    vendorsState.successMessage = `Account reactivated successfully.`;
                                } else if (result.type === 'failure') {
                                    vendorsState.errorMessage = (result.data?.error as string) || 'Failed to reactivate account';
                                }
                                update({ reset: false });
                            };
                        }}
                    >
                        <input type="hidden" name="id" value={vendor.id} />
                        <button 
                            type="submit" 
                            class="btn-action approve" 
                            disabled={vendorsState.submittingIds.includes(vendor.id)}
                        >
                            Reactivate Account
                        </button>
                    </form>
                {/if}

                <!-- Active account suspend action -->
                {#if vendor.status === 'active' && vendor.subscription_status !== 'stopped'}
                    <button 
                        class="btn-action suspend" 
                        onclick={() => { vendorsState.promptStatusChange = { id: vendor.id, status: 'suspended' }; vendorsState.statusReason = ''; }}
                        disabled={vendorsState.submittingIds.includes(vendor.id)}
                    >
                        Suspend / إيقاف مؤقت
                    </button>
                {/if}

                <!-- Ad Placement Action -->
                {#if vendor.status === 'active'}
                    <button 
                        type="button"
                        class="btn-action ads-placement" 
                        onclick={() => {
                            vendorsState.promptAdModal = vendor;
                            vendorsState.adIsFeatured = vendor.is_featured || false;
                            
                            if (vendor.featured_expires_at) {
                                const expDate = new Date(vendor.featured_expires_at);
                                vendorsState.adCustomDate = expDate.toISOString().split('T')[0];
                                vendorsState.adExpiryPreset = 'custom';
                            } else {
                                const oneMonthFromNow = new Date();
                                oneMonthFromNow.setMonth(oneMonthFromNow.getMonth() + 1);
                                vendorsState.adCustomDate = oneMonthFromNow.toISOString().split('T')[0];
                                vendorsState.adExpiryPreset = '1month';
                            }
                        }}
                        disabled={vendorsState.submittingIds.includes(vendor.id)}
                    >
                        <Sparkles size={14} />
                        <span>Manage Ad / إدارة الإعلان</span>
                    </button>
                {/if}

                <!-- Messaging support chat drawer button -->
                <button class="btn-action chat" onclick={async () => {
                    vendorsState.chatVendor = vendor;
                    vendorsState.chatMessages = [];
                    vendorsState.chatError = '';
                    vendorsState.isFetchingMessages = true;
                    try {
                        const res = await fetch(`/dashboard/vendors/${vendor.id}/chat`);
                        if (res.ok) {
                            const data = await res.json();
                            vendorsState.chatMessages = data.messages || [];
                            if (typeof window !== 'undefined' && (window as any).__updateAdminUnreadCounts) {
                                (window as any).__updateAdminUnreadCounts();
                            }
                        } else {
                            vendorsState.chatError = 'Failed to load support thread messages';
                        }
                    } catch (e) {
                        vendorsState.chatError = 'Backend service connection error';
                    } finally {
                        vendorsState.isFetchingMessages = false;
                    }
                }}>
                    <MessageSquare size={14} />
                    <span>Support Chat / المحادثة</span>
                </button>

                <!-- View full vendor detail + hall moderation -->
                <a href="/dashboard/vendors/{vendor.id}" class="btn-action detail-link">
                    <Building2 size={14} />
                    <span>View Details & Halls</span>
                </a>
            </div>
        </div>

    </div>
</div>
