<script lang="ts">
    import { getCountdown, isUrgentLead, getStatusLabel } from "../../services/couples.service";

    let { lead, i18n, couplesState } = $props<{ lead: any, i18n: any, couplesState: any }>();

    let countdown = $derived(getCountdown(lead.wedding_date, i18n.locale));
    let isUrgent = $derived(isUrgentLead(lead.wedding_date, i18n.locale));

    function openInquiryDetail() {
        couplesState.selectedInquiry = lead;
        couplesState.isDrawerOpen = true;
    }
</script>

<button 
    type="button" 
    onclick={openInquiryDetail} 
    class="lead-card"
    class:unread-card={lead.status === 'new'}
    class:selected-card={couplesState.activeSelectedInquiry?.id === lead.id}
>
    <!-- Unread Gold indicator ribbon -->
    {#if lead.status === 'new'}
        <div class="unread-gold-ribbon"></div>
    {/if}

    <div class="card-header">
        <div class="user-block">
            <div class="avatar" class:unread-avatar={lead.status === 'new'}>
                {lead.customer_name.charAt(0).toUpperCase()}
            </div>
            <div class="info">
                <div class="name-row">
                    <span class="customer-name">{lead.customer_name}</span>
                    {#if lead.status === 'new'}
                        <span class="new-tag">{i18n.locale === 'ar' ? 'جديد' : 'NEW'}</span>
                    {/if}
                </div>
                <span class="receipt-time">{i18n.locale === 'ar' ? 'تم الاستلام في' : 'Received on'} {lead.created_at.substring(0, 16).replace('T', ' ')}</span>
            </div>
        </div>

        <div class="countdown-badge" class:urgent-badge={isUrgent} class:past-badge={countdown.isPast}>
            {countdown.text}
        </div>
    </div>

    <div class="card-body">
        <p class="message-snippet">{lead.message || ''}</p>
    </div>

    <div class="card-footer">
        <div class="contact-pill">
            📞 {lead.phone}
        </div>
        <div class="wedding-pill">
            📅 {i18n.t.couples.eventDate}: {lead.wedding_date}
        </div>
        <div class="status-pill badge" class:badge-new={lead.status === 'new'} class:badge-done={lead.status === 'done' || lead.status === 'paid'} class:badge-negot={lead.status === 'negotiation'} class:badge-unreach={lead.status === 'unreachable'}>
            {getStatusLabel(lead.status, i18n)}
        </div>
    </div>
</button>
